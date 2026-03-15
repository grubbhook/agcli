//! Metagraph disk cache — save and load subnet snapshots.
//!
//! Snapshots are stored as compressed JSON at:
//!   `~/.agcli/metagraph/sn{netuid}/block-{block}.json.zst`
//!
//! The `latest` symlink always points to the most recent snapshot.

use crate::types::chain_data::Metagraph;
use anyhow::{Context, Result};
use std::path::PathBuf;

/// Base directory for metagraph cache.
fn cache_dir(netuid: u16) -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".agcli")
        .join("metagraph")
        .join(format!("sn{}", netuid))
}

/// Save a metagraph snapshot to disk.
pub fn save(metagraph: &Metagraph) -> Result<PathBuf> {
    let dir = cache_dir(metagraph.netuid.0);
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("Failed to create cache dir: {}", dir.display()))?;

    let filename = format!("block-{}.json", metagraph.block);
    let path = dir.join(&filename);

    let json = serde_json::to_string(metagraph)
        .context("Failed to serialize metagraph")?;
    std::fs::write(&path, json.as_bytes())
        .with_context(|| format!("Failed to write {}", path.display()))?;

    // Update "latest" symlink
    let latest = dir.join("latest.json");
    if let Err(e) = std::fs::remove_file(&latest) {
        if e.kind() != std::io::ErrorKind::NotFound {
            tracing::warn!(path = %latest.display(), error = %e, "Failed to remove old latest symlink");
        }
    }
    #[cfg(unix)]
    {
        if let Err(e) = std::os::unix::fs::symlink(&filename, &latest) {
            tracing::warn!(src = %filename, dst = %latest.display(), error = %e, "Failed to create latest symlink");
        }
    }
    #[cfg(not(unix))]
    {
        if let Err(e) = std::fs::copy(&path, &latest) {
            tracing::warn!(src = %path.display(), dst = %latest.display(), error = %e, "Failed to copy latest cache");
        }
    }

    Ok(path)
}

/// Load the most recent cached metagraph for a subnet.
pub fn load_latest(netuid: u16) -> Result<Option<Metagraph>> {
    let dir = cache_dir(netuid);
    let latest = dir.join("latest.json");
    if !latest.exists() {
        return Ok(None);
    }
    let data = std::fs::read_to_string(&latest)
        .with_context(|| format!("Failed to read {}", latest.display()))?;
    let mg: Metagraph = serde_json::from_str(&data)
        .with_context(|| format!("Failed to parse {}", latest.display()))?;
    Ok(Some(mg))
}

/// Load a metagraph snapshot for a specific block.
pub fn load_block(netuid: u16, block: u64) -> Result<Option<Metagraph>> {
    let path = cache_dir(netuid).join(format!("block-{}.json", block));
    if !path.exists() {
        return Ok(None);
    }
    let data = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let mg: Metagraph = serde_json::from_str(&data)
        .with_context(|| format!("Failed to parse {}", path.display()))?;
    Ok(Some(mg))
}

/// List all cached blocks for a subnet, sorted ascending.
pub fn list_cached_blocks(netuid: u16) -> Result<Vec<u64>> {
    let dir = cache_dir(netuid);
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut blocks = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if let Some(rest) = name.strip_prefix("block-") {
            if let Some(num_str) = rest.strip_suffix(".json") {
                if let Ok(block) = num_str.parse::<u64>() {
                    blocks.push(block);
                }
            }
        }
    }
    blocks.sort_unstable();
    Ok(blocks)
}

/// Remove snapshots older than `keep` most recent. Returns count removed.
pub fn prune(netuid: u16, keep: usize) -> Result<usize> {
    let blocks = list_cached_blocks(netuid)?;
    if blocks.len() <= keep {
        return Ok(0);
    }
    let dir = cache_dir(netuid);
    let to_remove = &blocks[..blocks.len() - keep];
    let mut removed = 0;
    for block in to_remove {
        let path = dir.join(format!("block-{}.json", block));
        if std::fs::remove_file(&path).is_ok() {
            removed += 1;
        }
    }
    Ok(removed)
}

/// Get the cache directory path (for display purposes).
pub fn cache_path(netuid: u16) -> PathBuf {
    cache_dir(netuid)
}

/// Diff two metagraph snapshots: returns (uid, field, old_value, new_value) tuples.
pub fn diff(old: &Metagraph, new: &Metagraph) -> Vec<MetagraphDelta> {
    let mut deltas = Vec::new();

    // Build hotkey→uid maps for both
    let old_map: std::collections::HashMap<&str, usize> = old
        .neurons
        .iter()
        .enumerate()
        .map(|(i, n)| (n.hotkey.as_str(), i))
        .collect();
    let new_map: std::collections::HashMap<&str, usize> = new
        .neurons
        .iter()
        .enumerate()
        .map(|(i, n)| (n.hotkey.as_str(), i))
        .collect();

    // Check for deregistered neurons
    for (hotkey, &old_idx) in &old_map {
        if !new_map.contains_key(hotkey) {
            deltas.push(MetagraphDelta {
                uid: old.neurons[old_idx].uid,
                hotkey: hotkey.to_string(),
                kind: DeltaKind::Deregistered,
            });
        }
    }

    // Check for new and changed neurons
    for (hotkey, &new_idx) in &new_map {
        let nn = &new.neurons[new_idx];
        match old_map.get(hotkey) {
            None => {
                deltas.push(MetagraphDelta {
                    uid: nn.uid,
                    hotkey: hotkey.to_string(),
                    kind: DeltaKind::Registered,
                });
            }
            Some(&old_idx) => {
                let on = &old.neurons[old_idx];
                // Significant changes
                if (nn.incentive - on.incentive).abs() > 0.001 {
                    deltas.push(MetagraphDelta {
                        uid: nn.uid,
                        hotkey: hotkey.to_string(),
                        kind: DeltaKind::Changed {
                            field: "incentive".to_string(),
                            old_val: format!("{:.4}", on.incentive),
                            new_val: format!("{:.4}", nn.incentive),
                        },
                    });
                }
                if (nn.emission - on.emission).abs() > 1e6 {
                    deltas.push(MetagraphDelta {
                        uid: nn.uid,
                        hotkey: hotkey.to_string(),
                        kind: DeltaKind::Changed {
                            field: "emission".to_string(),
                            old_val: format!("{:.4}", on.emission / 1e9),
                            new_val: format!("{:.4}", nn.emission / 1e9),
                        },
                    });
                }
                let stake_diff = nn.stake.rao() as i64 - on.stake.rao() as i64;
                if stake_diff.unsigned_abs() > 1_000_000_000 {
                    // > 1 TAO change
                    deltas.push(MetagraphDelta {
                        uid: nn.uid,
                        hotkey: hotkey.to_string(),
                        kind: DeltaKind::Changed {
                            field: "stake".to_string(),
                            old_val: format!("{:.4}τ", on.stake.tao()),
                            new_val: format!("{:.4}τ", nn.stake.tao()),
                        },
                    });
                }
            }
        }
    }

    deltas
}

/// A single change between two metagraph snapshots.
#[derive(Debug, Clone, serde::Serialize)]
pub struct MetagraphDelta {
    pub uid: u16,
    pub hotkey: String,
    pub kind: DeltaKind,
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum DeltaKind {
    Registered,
    Deregistered,
    Changed {
        field: String,
        old_val: String,
        new_val: String,
    },
}

impl std::fmt::Display for MetagraphDelta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hk_short = if self.hotkey.len() > 8 {
            &self.hotkey[..8]
        } else {
            &self.hotkey
        };
        match &self.kind {
            DeltaKind::Registered => write!(f, "  + UID {} ({}) registered", self.uid, hk_short),
            DeltaKind::Deregistered => {
                write!(f, "  - UID {} ({}) deregistered", self.uid, hk_short)
            }
            DeltaKind::Changed {
                field,
                old_val,
                new_val,
            } => write!(
                f,
                "  ~ UID {} ({}) {} {} → {}",
                self.uid, hk_short, field, old_val, new_val
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::balance::Balance;
    use crate::types::chain_data::NeuronInfoLite;
    use crate::types::network::NetUid;

    fn make_neuron(uid: u16, hotkey: &str, netuid: u16, stake_tao: f64, incentive: f64) -> NeuronInfoLite {
        NeuronInfoLite {
            hotkey: hotkey.to_string(),
            coldkey: "5Cold".to_string(),
            uid,
            netuid: NetUid(netuid),
            active: true,
            stake: Balance::from_rao((stake_tao * 1e9) as u64),
            rank: 0.0,
            emission: 0.0,
            incentive,
            consensus: 0.0,
            trust: 0.0,
            validator_trust: 0.0,
            dividends: 0.0,
            last_update: 100,
            validator_permit: false,
            pruning_score: 0.0,
        }
    }

    fn make_metagraph(neurons: Vec<NeuronInfoLite>, netuid: u16, block: u64) -> Metagraph {
        let n = neurons.len() as u16;
        Metagraph {
            netuid: NetUid(netuid),
            n,
            block,
            stake: neurons.iter().map(|n| n.stake).collect(),
            ranks: neurons.iter().map(|n| n.rank).collect(),
            trust: neurons.iter().map(|n| n.trust).collect(),
            consensus: neurons.iter().map(|n| n.consensus).collect(),
            incentive: neurons.iter().map(|n| n.incentive).collect(),
            dividends: neurons.iter().map(|n| n.dividends).collect(),
            emission: neurons.iter().map(|n| n.emission).collect(),
            validator_trust: neurons.iter().map(|n| n.validator_trust).collect(),
            validator_permit: neurons.iter().map(|n| n.validator_permit).collect(),
            uids: neurons.iter().map(|n| n.uid).collect(),
            active: neurons.iter().map(|n| n.active).collect(),
            last_update: neurons.iter().map(|n| n.last_update).collect(),
            neurons,
        }
    }

    // Use unique netuids per test to avoid interference when tests run in parallel
    #[test]
    fn test_save_load_roundtrip() {
        let netuid = 60001;
        let neurons = vec![
            make_neuron(0, "5HotA", netuid, 100.0, 0.5),
            make_neuron(1, "5HotB", netuid, 200.0, 0.3),
        ];
        let mg = make_metagraph(neurons, netuid, 999999);

        // Save
        let path = save(&mg).unwrap();
        assert!(path.exists());

        // Load latest
        let loaded = load_latest(netuid).unwrap().unwrap();
        assert_eq!(loaded.block, 999999);
        assert_eq!(loaded.neurons.len(), 2);
        assert_eq!(loaded.neurons[0].hotkey, "5HotA");

        // Load specific block
        let loaded2 = load_block(netuid, 999999).unwrap().unwrap();
        assert_eq!(loaded2.block, 999999);

        // Missing block returns None
        let missing = load_block(netuid, 123).unwrap();
        assert!(missing.is_none());

        // List cached blocks
        let blocks = list_cached_blocks(netuid).unwrap();
        assert!(blocks.contains(&999999));

        // Cleanup
        let _ = std::fs::remove_dir_all(cache_path(netuid));
    }

    #[test]
    fn test_diff_detects_changes() {
        let netuid = 60002;
        let old = make_metagraph(
            vec![
                make_neuron(0, "5HotA", netuid, 100.0, 0.5),
                make_neuron(1, "5HotB", netuid, 200.0, 0.3),
            ],
            netuid,
            1000,
        );
        let new = make_metagraph(
            vec![
                make_neuron(0, "5HotA", netuid, 150.0, 0.5), // stake changed by 50 TAO
                make_neuron(2, "5HotC", netuid, 50.0, 0.1),   // new neuron
            ],
            netuid,
            1100,
        );

        let deltas = diff(&old, &new);
        assert!(deltas.iter().any(|d| matches!(&d.kind, DeltaKind::Deregistered) && d.hotkey == "5HotB"));
        assert!(deltas.iter().any(|d| matches!(&d.kind, DeltaKind::Registered) && d.hotkey == "5HotC"));
        assert!(deltas.iter().any(|d| matches!(&d.kind, DeltaKind::Changed { field, .. } if field == "stake") && d.hotkey == "5HotA"));
    }

    #[test]
    fn test_prune_keeps_recent() {
        let netuid = 60003;
        // Save 3 blocks
        let neurons = vec![make_neuron(0, "5HotA", netuid, 100.0, 0.5)];
        for block in [800000, 800100, 800200] {
            let mg = make_metagraph(neurons.clone(), netuid, block);
            save(&mg).unwrap();
        }

        // Prune, keep 2
        let removed = prune(netuid, 2).unwrap();
        assert_eq!(removed, 1);

        let remaining = list_cached_blocks(netuid).unwrap();
        assert!(!remaining.contains(&800000));
        assert!(remaining.contains(&800100));
        assert!(remaining.contains(&800200));

        // Cleanup
        let _ = std::fs::remove_dir_all(cache_path(netuid));
    }
}
