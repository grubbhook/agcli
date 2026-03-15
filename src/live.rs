//! Live mode — poll chain data at intervals and display changes.
//!
//! Provides `--live` functionality for metagraph, dynamic, portfolio, and stake views.
//! Tracks deltas between polls to highlight what changed.

use crate::chain::Client;
use crate::types::chain_data::DynamicInfo;
use crate::types::NetUid;
use crate::utils::truncate;
use anyhow::Result;
use std::collections::HashMap;
use std::io::Write;
use std::time::Duration;
use tokio::signal;

/// Default poll interval (12 seconds = 1 Bittensor block).
pub const DEFAULT_POLL_SECS: u64 = 12;

/// Delta tracking for DynamicInfo changes.
#[derive(Debug)]
pub struct DynamicDelta {
    pub netuid: u16,
    pub name: String,
    pub price_prev: f64,
    pub price_now: f64,
    pub price_pct: f64,
    pub tao_in_prev: u64,
    pub tao_in_now: u64,
    pub volume_prev: u128,
    pub volume_now: u128,
}

/// Compute deltas between two snapshots of DynamicInfo.
pub fn compute_dynamic_deltas(prev: &[DynamicInfo], curr: &[DynamicInfo]) -> Vec<DynamicDelta> {
    let prev_map: HashMap<u16, &DynamicInfo> = prev.iter().map(|d| (d.netuid.0, d)).collect();
    curr.iter()
        .filter_map(|c| {
            let p = prev_map.get(&c.netuid.0)?;
            let price_pct = if p.price > 0.0 {
                (c.price - p.price) / p.price * 100.0
            } else {
                0.0
            };
            // Only report if something actually changed
            if (c.price - p.price).abs() < 1e-15
                && c.tao_in.rao() == p.tao_in.rao()
                && c.subnet_volume == p.subnet_volume
            {
                return None;
            }
            Some(DynamicDelta {
                netuid: c.netuid.0,
                name: c.name.clone(),
                price_prev: p.price,
                price_now: c.price,
                price_pct,
                tao_in_prev: p.tao_in.rao(),
                tao_in_now: c.tao_in.rao(),
                volume_prev: p.subnet_volume,
                volume_now: c.subnet_volume,
            })
        })
        .collect()
}

/// Run live dynamic info polling loop.
pub async fn live_dynamic(client: &Client, interval_secs: u64) -> Result<()> {
    let interval = Duration::from_secs(if interval_secs == 0 {
        DEFAULT_POLL_SECS
    } else {
        interval_secs
    });
    let mut prev = client.get_all_dynamic_info().await?;
    let mut poll_count = 0u64;

    print_dynamic_header();
    print_dynamic_snapshot(&prev);

    loop {
        tokio::select! {
            _ = tokio::time::sleep(interval) => {},
            _ = signal::ctrl_c() => {
                println!("\nStopping live dynamic polling (received Ctrl+C)");
                return Ok(());
            }
        }
        poll_count += 1;
        let curr = match client.get_all_dynamic_info().await {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Warning: poll #{} failed: {} (retrying next interval)", poll_count, e);
                continue;
            }
        };
        let deltas = compute_dynamic_deltas(&prev, &curr);

        if !deltas.is_empty() {
            println!("\n--- Poll #{} ({} changes) ---", poll_count, deltas.len());
            for d in &deltas {
                let arrow = if d.price_pct > 0.0 {
                    "↑"
                } else if d.price_pct < 0.0 {
                    "↓"
                } else {
                    "→"
                };
                println!(
                    "  SN{:<3} {:<16} {:>10.6} → {:>10.6} τ/α  ({}{:>+.2}%)  pool: {:.2} → {:.2} τ",
                    d.netuid,
                    truncate(&d.name, 16),
                    d.price_prev,
                    d.price_now,
                    arrow,
                    d.price_pct,
                    d.tao_in_prev as f64 / 1e9,
                    d.tao_in_now as f64 / 1e9,
                );
            }
            let _ = std::io::stdout().flush();
        }
        prev = curr;
    }
}

/// Run live metagraph polling loop.
pub async fn live_metagraph(client: &Client, netuid: NetUid, interval_secs: u64) -> Result<()> {
    let interval = Duration::from_secs(if interval_secs == 0 {
        DEFAULT_POLL_SECS
    } else {
        interval_secs
    });
    let mut prev_neurons = client.get_neurons_lite(netuid).await?;
    let mut poll_count = 0u64;

    println!(
        "Live metagraph for SN{} (polling every {}s, Ctrl+C to stop)\n",
        netuid.0,
        interval.as_secs()
    );
    println!("Tracking {} neurons...", prev_neurons.len());

    loop {
        tokio::select! {
            _ = tokio::time::sleep(interval) => {},
            _ = signal::ctrl_c() => {
                println!("\nStopping live metagraph polling (received Ctrl+C)");
                return Ok(());
            }
        }
        poll_count += 1;
        let curr_neurons = match client.get_neurons_lite(netuid).await {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Warning: poll #{} failed: {} (retrying next interval)", poll_count, e);
                continue;
            }
        };
        let mut changes = Vec::new();

        let prev_map: HashMap<u16, &crate::types::chain_data::NeuronInfoLite> =
            prev_neurons.iter().map(|n| (n.uid, n)).collect();

        for c in &curr_neurons {
            if let Some(p) = prev_map.get(&c.uid) {
                let stake_diff = c.stake.rao() as i64 - p.stake.rao() as i64;
                let incentive_diff = c.incentive - p.incentive;
                let emission_diff = c.emission - p.emission;

                if stake_diff.unsigned_abs() > 100_000_000 // > 0.1 TAO
                    || incentive_diff.abs() > 0.001
                    || emission_diff.abs() > 0.001
                {
                    changes.push(format!(
                        "  UID {:<4} stake:{:>+.4}τ  incentive:{:>+.4}  emission:{:>+.1}",
                        c.uid,
                        stake_diff as f64 / 1e9,
                        incentive_diff,
                        emission_diff,
                    ));
                }
            } else {
                changes.push(format!(
                    "  UID {:<4} NEW neuron (hotkey: {})",
                    c.uid,
                    &c.hotkey[..8]
                ));
            }
        }

        if !changes.is_empty() {
            println!("\n--- Poll #{} ({} changes) ---", poll_count, changes.len());
            for ch in &changes {
                println!("{}", ch);
            }
            let _ = std::io::stdout().flush();
        }
        prev_neurons = curr_neurons;
    }
}

/// Run live portfolio polling.
pub async fn live_portfolio(client: &Client, coldkey_ss58: &str, interval_secs: u64) -> Result<()> {
    let interval = Duration::from_secs(if interval_secs == 0 {
        DEFAULT_POLL_SECS
    } else {
        interval_secs
    });
    let mut prev = crate::queries::portfolio::fetch_portfolio(client, coldkey_ss58).await?;
    let mut poll_count = 0u64;

    println!(
        "Live portfolio for {} (polling every {}s, Ctrl+C to stop)\n",
        coldkey_ss58,
        interval.as_secs()
    );
    println!(
        "Free: {}  Staked: {}  Positions: {}",
        prev.free_balance,
        prev.total_staked,
        prev.positions.len()
    );

    loop {
        tokio::select! {
            _ = tokio::time::sleep(interval) => {},
            _ = signal::ctrl_c() => {
                println!("\nStopping live portfolio polling (received Ctrl+C)");
                return Ok(());
            }
        }
        poll_count += 1;
        let curr = match crate::queries::portfolio::fetch_portfolio(client, coldkey_ss58).await {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Warning: poll #{} failed: {} (retrying next interval)", poll_count, e);
                continue;
            }
        };

        let free_diff = curr.free_balance.rao() as i64 - prev.free_balance.rao() as i64;
        let staked_diff = curr.total_staked.rao() as i64 - prev.total_staked.rao() as i64;

        if free_diff.unsigned_abs() > 100_000 || staked_diff.unsigned_abs() > 100_000 {
            println!(
                "\n--- Poll #{} ---  Free:{:>+.6}τ  Staked:{:>+.6}τ",
                poll_count,
                free_diff as f64 / 1e9,
                staked_diff as f64 / 1e9,
            );
        }
        prev = curr;
    }
}

/// Print dynamic info table header.
fn print_dynamic_header() {
    println!("Live Dynamic TAO (Ctrl+C to stop)\n");
    println!(
        "{:<5} {:<16} {:>12} {:>12} {:>12} {:>12}",
        "SN", "Name", "Price (τ/α)", "TAO Pool", "Alpha In", "Volume"
    );
    println!("{}", "-".repeat(75));
}

/// Print full dynamic snapshot.
fn print_dynamic_snapshot(subnets: &[DynamicInfo]) {
    for d in subnets {
        if d.tao_in.rao() == 0 {
            continue; // skip empty subnets
        }
        println!(
            "{:<5} {:<16} {:>12.6} {:>12.2} {:>12.2} {:>12}",
            d.netuid.0,
            truncate(&d.name, 16),
            d.price,
            d.tao_in.tao(),
            d.alpha_in.raw() as f64 / 1e9,
            d.subnet_volume,
        );
    }
}
