//! Block explorer and historical diff handlers.

use crate::chain::Client;
use crate::cli::helpers::*;
use crate::cli::*;
use crate::types::{Balance, NetUid};
use anyhow::Result;

pub(super) async fn handle_block(
    cmd: BlockCommands,
    client: &Client,
    output: OutputFormat,
) -> Result<()> {
    match cmd {
        BlockCommands::Info { number } => {
            let block_hash = client.get_block_hash(number).await?;
            let ((block_num, hash, parent_hash, state_root), ext_count, timestamp) = tokio::try_join!(
                client.get_block_header(block_hash),
                client.get_block_extrinsic_count(block_hash),
                client.get_block_timestamp(block_hash),
            )?;

            if output.is_json() {
                let mut obj = serde_json::json!({
                    "block_number": block_num,
                    "block_hash": format!("{:?}", hash),
                    "parent_hash": format!("{:?}", parent_hash),
                    "state_root": format!("{:?}", state_root),
                    "extrinsic_count": ext_count,
                });
                if let Some(ts) = timestamp {
                    obj["timestamp_ms"] = serde_json::json!(ts);
                    if let Some(dt) = chrono::DateTime::from_timestamp_millis(ts as i64) {
                        obj["timestamp"] = serde_json::json!(dt.to_rfc3339());
                    }
                }
                print_json(&obj);
            } else {
                println!("Block #{}", block_num);
                println!("  Hash:        {:?}", hash);
                println!("  Parent:      {:?}", parent_hash);
                println!("  State root:  {:?}", state_root);
                println!("  Extrinsics:  {}", ext_count);
                if let Some(ts) = timestamp {
                    if let Some(dt) = chrono::DateTime::from_timestamp_millis(ts as i64) {
                        println!("  Timestamp:   {}", dt.format("%Y-%m-%d %H:%M:%S UTC"));
                    } else {
                        println!("  Timestamp:   {} ms", ts);
                    }
                }
            }
            Ok(())
        }
        BlockCommands::Range { from, to } => {
            if from > to {
                anyhow::bail!("--from ({}) must be <= --to ({})", from, to);
            }
            let count = (to - from + 1) as usize;
            if count > 1000 {
                anyhow::bail!(
                    "Range too large ({} blocks). Maximum 1000 blocks per query.",
                    count
                );
            }

            #[derive(serde::Serialize)]
            struct BlockRow {
                block: u32,
                hash: String,
                timestamp: String,
                extrinsics: usize,
            }

            // Fetch all block hashes concurrently instead of sequentially
            let hash_futures: Vec<_> = (from..=to)
                .map(|block_num| client.get_block_hash(block_num))
                .collect();
            let block_hashes = futures::future::try_join_all(hash_futures).await?;

            // Fetch extrinsic counts + timestamps for all blocks concurrently
            let detail_futures: Vec<_> = block_hashes
                .iter()
                .map(|&hash| async move {
                    let (ext_count, timestamp) = tokio::try_join!(
                        client.get_block_extrinsic_count(hash),
                        client.get_block_timestamp(hash),
                    )?;
                    Ok::<_, anyhow::Error>((ext_count, timestamp))
                })
                .collect();
            let details = futures::future::try_join_all(detail_futures).await?;

            let rows: Vec<BlockRow> = (from..=to)
                .zip(block_hashes.iter().zip(details.iter()))
                .map(|(block_num, (hash, (ext_count, timestamp)))| {
                    let ts_str = timestamp
                        .and_then(|ts| chrono::DateTime::from_timestamp_millis(ts as i64))
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_default();
                    BlockRow {
                        block: block_num,
                        hash: format!("{:?}", hash),
                        timestamp: ts_str,
                        extrinsics: *ext_count,
                    }
                })
                .collect();

            render_rows(
                output,
                &rows,
                "block,hash,timestamp,extrinsics",
                |r| format!("{},{},{},{}", r.block, r.hash, r.timestamp, r.extrinsics),
                &["Block", "Hash", "Timestamp", "Exts"],
                |r| {
                    vec![
                        format!("#{}", r.block),
                        r.hash.chars().take(18).collect::<String>() + "…",
                        r.timestamp.clone(),
                        r.extrinsics.to_string(),
                    ]
                },
                Some(&format!("Blocks {} → {} ({} blocks)", from, to, count)),
            );
            Ok(())
        }
        BlockCommands::Latest => {
            let block_num = client.get_block_number().await?;
            let block_hash = client.get_block_hash(block_num as u32).await?;
            let (ext_count, timestamp) = tokio::try_join!(
                client.get_block_extrinsic_count(block_hash),
                client.get_block_timestamp(block_hash),
            )?;

            if output.is_json() {
                let mut obj = serde_json::json!({
                    "block_number": block_num,
                    "block_hash": format!("{:?}", block_hash),
                    "extrinsic_count": ext_count,
                });
                if let Some(ts) = timestamp {
                    obj["timestamp_ms"] = serde_json::json!(ts);
                    if let Some(dt) = chrono::DateTime::from_timestamp_millis(ts as i64) {
                        obj["timestamp"] = serde_json::json!(dt.to_rfc3339());
                    }
                }
                print_json(&obj);
            } else {
                println!("Latest Block: #{}", block_num);
                println!("  Hash:        {:?}", block_hash);
                println!("  Extrinsics:  {}", ext_count);
                if let Some(ts) = timestamp {
                    if let Some(dt) = chrono::DateTime::from_timestamp_millis(ts as i64) {
                        println!("  Timestamp:   {}", dt.format("%Y-%m-%d %H:%M:%S UTC"));
                    } else {
                        println!("  Timestamp:   {} ms", ts);
                    }
                }
            }
            Ok(())
        }
    }
}

pub(super) async fn handle_diff(
    cmd: DiffCommands,
    client: &Client,
    output: OutputFormat,
    wallet_dir: &str,
    wallet_name: &str,
) -> Result<()> {
    match cmd {
        DiffCommands::Portfolio {
            address,
            block1,
            block2,
        } => {
            let addr = resolve_coldkey_address(address, wallet_dir, wallet_name);
            if addr.is_empty() {
                anyhow::bail!("No address provided and no wallet found. Use --address <SS58>.");
            }

            let (hash1, hash2) =
                tokio::try_join!(client.get_block_hash(block1), client.get_block_hash(block2),)?;

            let (bal1, stakes1, bal2, stakes2) = tokio::try_join!(
                client.get_balance_at_block(&addr, hash1),
                client.get_stake_for_coldkey_at_block(&addr, hash1),
                client.get_balance_at_block(&addr, hash2),
                client.get_stake_for_coldkey_at_block(&addr, hash2),
            )?;

            let total_stake1: u64 = stakes1.iter().map(|s| s.stake.rao()).sum();
            let total_stake2: u64 = stakes2.iter().map(|s| s.stake.rao()).sum();
            let total1 = bal1.rao() + total_stake1;
            let total2 = bal2.rao() + total_stake2;

            if output.is_json() {
                print_json(&serde_json::json!({
                    "address": addr,
                    "block1": block1,
                    "block2": block2,
                    "balance_tao": [bal1.tao(), bal2.tao()],
                    "balance_diff_tao": bal2.tao() - bal1.tao(),
                    "total_stake_tao": [Balance::from_rao(total_stake1).tao(), Balance::from_rao(total_stake2).tao()],
                    "stake_diff_tao": Balance::from_rao(total_stake2).tao() - Balance::from_rao(total_stake1).tao(),
                    "total_tao": [Balance::from_rao(total1).tao(), Balance::from_rao(total2).tao()],
                    "total_diff_tao": Balance::from_rao(total2).tao() - Balance::from_rao(total1).tao(),
                    "stakes_block1": stakes1.len(),
                    "stakes_block2": stakes2.len(),
                }));
            } else {
                println!("Portfolio Diff: {} (block {} → {})\n", addr, block1, block2);
                let diff_sym = |a: f64, b: f64| -> String {
                    let d = b - a;
                    if d > 0.0 {
                        format!("+{:.4}", d)
                    } else if d < 0.0 {
                        format!("{:.4}", d)
                    } else {
                        "0".to_string()
                    }
                };
                println!(
                    "  {:>20}  {:>14}  {:>14}  {:>14}",
                    "",
                    format!("Block {}", block1),
                    format!("Block {}", block2),
                    "Change"
                );
                println!(
                    "  {:>20}  {:>14.4}  {:>14.4}  {:>14}",
                    "Free balance (τ)",
                    bal1.tao(),
                    bal2.tao(),
                    diff_sym(bal1.tao(), bal2.tao())
                );
                println!(
                    "  {:>20}  {:>14.4}  {:>14.4}  {:>14}",
                    "Total stake (τ)",
                    Balance::from_rao(total_stake1).tao(),
                    Balance::from_rao(total_stake2).tao(),
                    diff_sym(
                        Balance::from_rao(total_stake1).tao(),
                        Balance::from_rao(total_stake2).tao()
                    )
                );
                println!(
                    "  {:>20}  {:>14.4}  {:>14.4}  {:>14}",
                    "Total (τ)",
                    Balance::from_rao(total1).tao(),
                    Balance::from_rao(total2).tao(),
                    diff_sym(
                        Balance::from_rao(total1).tao(),
                        Balance::from_rao(total2).tao()
                    )
                );
                println!(
                    "  {:>20}  {:>14}  {:>14}",
                    "Stake positions",
                    stakes1.len(),
                    stakes2.len()
                );
            }
            Ok(())
        }
        DiffCommands::Subnet {
            netuid,
            block1,
            block2,
        } => {
            let (hash1, hash2) =
                tokio::try_join!(client.get_block_hash(block1), client.get_block_hash(block2),)?;
            let nuid = NetUid(netuid);

            let (dyn1, dyn2) = tokio::try_join!(
                client.get_dynamic_info_at_block(nuid, hash1),
                client.get_dynamic_info_at_block(nuid, hash2),
            )?;

            let d1 = dyn1.ok_or_else(|| {
                anyhow::anyhow!("Subnet {} not found at block {}", netuid, block1)
            })?;
            let d2 = dyn2.ok_or_else(|| {
                anyhow::anyhow!("Subnet {} not found at block {}", netuid, block2)
            })?;

            if output.is_json() {
                print_json(&serde_json::json!({
                    "netuid": netuid,
                    "name": d2.name,
                    "block1": block1,
                    "block2": block2,
                    "tao_in": [d1.tao_in.tao(), d2.tao_in.tao()],
                    "tao_in_diff": d2.tao_in.tao() - d1.tao_in.tao(),
                    "price": [d1.price, d2.price],
                    "price_diff": d2.price - d1.price,
                    "emission": [d1.emission, d2.emission],
                    "emission_diff": d2.emission as i64 - d1.emission as i64,
                }));
            } else {
                println!(
                    "Subnet {} ({}) Diff: block {} → {}\n",
                    netuid, d2.name, block1, block2
                );
                let diff_f = |a: f64, b: f64| -> String {
                    let d = b - a;
                    if d > 0.0 {
                        format!("+{:.4}", d)
                    } else if d < 0.0 {
                        format!("{:.4}", d)
                    } else {
                        "0".to_string()
                    }
                };
                let diff_pct = |a: f64, b: f64| -> String {
                    if a == 0.0 {
                        return "N/A".to_string();
                    }
                    let pct = (b - a) / a * 100.0;
                    if pct > 0.0 {
                        format!("+{:.1}%", pct)
                    } else if pct < 0.0 {
                        format!("{:.1}%", pct)
                    } else {
                        "0%".to_string()
                    }
                };
                println!(
                    "  {:>18}  {:>14}  {:>14}  {:>12}  {:>8}",
                    "",
                    format!("Block {}", block1),
                    format!("Block {}", block2),
                    "Change",
                    "%"
                );
                println!(
                    "  {:>18}  {:>14.4}  {:>14.4}  {:>12}  {:>8}",
                    "TAO in (τ)",
                    d1.tao_in.tao(),
                    d2.tao_in.tao(),
                    diff_f(d1.tao_in.tao(), d2.tao_in.tao()),
                    diff_pct(d1.tao_in.tao(), d2.tao_in.tao())
                );
                println!(
                    "  {:>18}  {:>14.6}  {:>14.6}  {:>12}  {:>8}",
                    "Price",
                    d1.price,
                    d2.price,
                    diff_f(d1.price, d2.price),
                    diff_pct(d1.price, d2.price)
                );
                println!(
                    "  {:>18}  {:>14}  {:>14}  {:>12}",
                    "Emission",
                    d1.emission,
                    d2.emission,
                    format!("{:+}", d2.emission as i64 - d1.emission as i64)
                );
                println!("  {:>18}  {:>14}  {:>14}", "Tempo", d1.tempo, d2.tempo);
                println!(
                    "  {:>18}  {:>14}  {:>14}",
                    "Owner HK",
                    crate::utils::short_ss58(&d1.owner_hotkey),
                    crate::utils::short_ss58(&d2.owner_hotkey)
                );
            }
            Ok(())
        }
        DiffCommands::Network { block1, block2 } => {
            let (hash1, hash2) =
                tokio::try_join!(client.get_block_hash(block1), client.get_block_hash(block2),)?;

            let (issuance1, stake1, subnets1, issuance2, stake2, subnets2) = tokio::try_join!(
                client.get_total_issuance_at_block(hash1),
                client.get_total_stake_at_block(hash1),
                client.get_all_subnets_at_block(hash1),
                client.get_total_issuance_at_block(hash2),
                client.get_total_stake_at_block(hash2),
                client.get_all_subnets_at_block(hash2),
            )?;

            let ratio1 = if issuance1.rao() > 0 {
                stake1.tao() / issuance1.tao() * 100.0
            } else {
                0.0
            };
            let ratio2 = if issuance2.rao() > 0 {
                stake2.tao() / issuance2.tao() * 100.0
            } else {
                0.0
            };

            if output.is_json() {
                print_json(&serde_json::json!({
                    "block1": block1,
                    "block2": block2,
                    "total_issuance_tao": [issuance1.tao(), issuance2.tao()],
                    "total_stake_tao": [stake1.tao(), stake2.tao()],
                    "staking_ratio_pct": [ratio1, ratio2],
                    "subnet_count": [subnets1.len(), subnets2.len()],
                }));
            } else {
                println!("Network Diff: block {} → {}\n", block1, block2);
                let diff_f = |a: f64, b: f64| -> String {
                    let d = b - a;
                    if d > 0.0 {
                        format!("+{:.4}", d)
                    } else if d < 0.0 {
                        format!("{:.4}", d)
                    } else {
                        "0".to_string()
                    }
                };
                println!(
                    "  {:>20}  {:>16}  {:>16}  {:>14}",
                    "",
                    format!("Block {}", block1),
                    format!("Block {}", block2),
                    "Change"
                );
                println!(
                    "  {:>20}  {:>16.4}  {:>16.4}  {:>14}",
                    "Issuance (τ)",
                    issuance1.tao(),
                    issuance2.tao(),
                    diff_f(issuance1.tao(), issuance2.tao())
                );
                println!(
                    "  {:>20}  {:>16.4}  {:>16.4}  {:>14}",
                    "Total stake (τ)",
                    stake1.tao(),
                    stake2.tao(),
                    diff_f(stake1.tao(), stake2.tao())
                );
                println!(
                    "  {:>20}  {:>15.1}%  {:>15.1}%  {:>14}",
                    "Staking ratio",
                    ratio1,
                    ratio2,
                    diff_f(ratio1, ratio2)
                );
                println!(
                    "  {:>20}  {:>16}  {:>16}  {:>14}",
                    "Subnets",
                    subnets1.len(),
                    subnets2.len(),
                    format!("{:+}", subnets2.len() as i64 - subnets1.len() as i64)
                );
            }
            Ok(())
        }
        DiffCommands::Metagraph {
            netuid,
            block1,
            block2,
        } => {
            let nuid = NetUid(netuid);
            let (hash1, hash2) =
                tokio::try_join!(client.get_block_hash(block1), client.get_block_hash(block2),)?;
            let (neurons1, neurons2) = tokio::try_join!(
                client.get_neurons_lite_at_block(nuid, hash1),
                client.get_neurons_lite_at_block(nuid, hash2),
            )?;

            let map1: std::collections::HashMap<u16, &crate::types::chain_data::NeuronInfoLite> =
                neurons1.iter().map(|n| (n.uid, n)).collect();

            let mut changes = Vec::new();
            for n2 in neurons2.iter() {
                if let Some(n1) = map1.get(&n2.uid) {
                    let stake_diff = n2.stake.tao() - n1.stake.tao();
                    let emission_diff = n2.emission - n1.emission;
                    let incentive_diff = n2.incentive - n1.incentive;
                    if stake_diff.abs() > 0.001
                        || emission_diff.abs() > 0.0001
                        || incentive_diff.abs() > 0.0001
                        || n2.hotkey != n1.hotkey
                    {
                        changes.push(serde_json::json!({
                            "uid": n2.uid, "hotkey": n2.hotkey,
                            "change": if n2.hotkey != n1.hotkey { "replaced" } else { "changed" },
                            "stake_diff": stake_diff, "emission_diff": emission_diff,
                            "incentive_diff": incentive_diff,
                        }));
                    }
                } else {
                    changes.push(serde_json::json!({
                        "uid": n2.uid, "hotkey": n2.hotkey, "change": "new",
                        "stake_diff": n2.stake.tao(), "emission_diff": n2.emission,
                        "incentive_diff": n2.incentive,
                    }));
                }
            }

            if output.is_json() {
                print_json(&serde_json::json!({
                    "netuid": netuid, "block1": block1, "block2": block2,
                    "neurons_block1": neurons1.len(), "neurons_block2": neurons2.len(),
                    "changed": changes.len(), "diffs": changes,
                }));
            } else {
                println!(
                    "Metagraph Diff SN{}: block {} → {} ({} changed)\n",
                    netuid,
                    block1,
                    block2,
                    changes.len()
                );
                for d in &changes {
                    println!(
                        "  UID {:>4} [{}] ({}) stake:{:>+.4}τ emission:{:>+.4} incentive:{:>+.4}",
                        d["uid"],
                        d["change"].as_str().unwrap_or(""),
                        crate::utils::short_ss58(d["hotkey"].as_str().unwrap_or("")),
                        d["stake_diff"].as_f64().unwrap_or(0.0),
                        d["emission_diff"].as_f64().unwrap_or(0.0),
                        d["incentive_diff"].as_f64().unwrap_or(0.0)
                    );
                }
            }
            Ok(())
        }
    }
}
