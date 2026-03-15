//! Weight command handlers.

use crate::chain::Client;
use crate::cli::helpers::*;
use crate::cli::*;
use crate::types::{Balance, NetUid};
use anyhow::Result;

pub(super) async fn handle_weights(
    cmd: WeightCommands,
    client: &Client,
    wallet_dir: &str,
    wallet_name: &str,
    hotkey_name: &str,
    password: Option<&str>,
) -> Result<()> {
    match cmd {
        WeightCommands::Set {
            netuid,
            weights,
            version_key,
            dry_run,
        } => {
            let (uids, wts) = parse_weight_pairs(&weights)?;

            // Pre-flight checks (always run these)
            let hyperparams = client
                .get_subnet_hyperparams(NetUid(netuid))
                .await
                .ok()
                .flatten();

            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            wallet.load_hotkey(hotkey_name)?;

            // Check stake-weight
            let stake_ok = if let Some(hk_ss58) = wallet.hotkey_ss58().map(|s| s.to_string()) {
                let alpha = client
                    .get_total_hotkey_alpha(&hk_ss58, NetUid(netuid))
                    .await
                    .unwrap_or(Balance::ZERO);
                if alpha.tao() < 1000.0 {
                    eprintln!("Warning: hotkey {} has {:.2}τ stake-weight on SN{} (minimum ~1000τ required).",
                        crate::utils::short_ss58(&hk_ss58), alpha.tao(), netuid);
                    tracing::warn!(hotkey = %crate::utils::short_ss58(&hk_ss58), stake = alpha.tao(), netuid = netuid, "Hotkey has low stake-weight (minimum ~1000τ required)");
                    false
                } else {
                    true
                }
            } else {
                false
            };

            // Check commit-reveal
            let cr_enabled = hyperparams
                .as_ref()
                .map(|h| h.commit_reveal_weights_enabled)
                .unwrap_or(false);
            if cr_enabled {
                eprintln!("Warning: SN{} has commit-reveal enabled. Use `agcli weights commit-reveal` instead.", netuid);
                tracing::warn!(netuid = netuid, "Subnet has commit-reveal enabled; use `agcli weights commit-reveal` instead");
            }

            // Check rate limit
            let rate_limit = hyperparams
                .as_ref()
                .map(|h| h.weights_rate_limit)
                .unwrap_or(0);

            if dry_run {
                print_json(&serde_json::json!({
                    "dry_run": true,
                    "netuid": netuid,
                    "num_weights": uids.len(),
                    "version_key": version_key,
                    "stake_sufficient": stake_ok,
                    "commit_reveal_enabled": cr_enabled,
                    "weights_rate_limit_blocks": rate_limit,
                    "weights": uids.iter().zip(wts.iter()).map(|(u, w)| serde_json::json!({"uid": u, "weight": w})).collect::<Vec<_>>(),
                }));
                return Ok(());
            }

            println!(
                "Setting {} weights on SN{} (version_key={})",
                uids.len(),
                netuid,
                version_key
            );
            let hash = client
                .set_weights(wallet.hotkey()?, NetUid(netuid), &uids, &wts, version_key)
                .await?;
            println!("Weights set. Tx: {}", hash);
            Ok(())
        }
        WeightCommands::Commit {
            netuid,
            weights,
            salt,
        } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            wallet.load_hotkey(hotkey_name)?;
            let (uids, wts) = parse_weight_pairs(&weights)?;
            let salt_str = salt.unwrap_or_else(|| {
                use rand::Rng;
                let s: String = rand::thread_rng()
                    .sample_iter(&rand::distributions::Alphanumeric)
                    .take(32)
                    .map(char::from)
                    .collect();
                println!("Generated salt: {}", s);
                s
            });
            let hash_out = crate::extrinsics::compute_weight_commit_hash(&uids, &wts, salt_str.as_bytes())
                .map_err(|e| anyhow::anyhow!("blake2 hash error: {:?}", e))?;
            println!("Committing weights on SN{}", netuid);
            println!("  Commit hash: 0x{}", hex::encode(hash_out));
            println!("  Save this salt for reveal: {}", salt_str);
            let hash = client
                .commit_weights(wallet.hotkey()?, NetUid(netuid), hash_out)
                .await?;
            println!("Weights committed. Tx: {}", hash);
            Ok(())
        }
        WeightCommands::Reveal {
            netuid,
            weights,
            salt,
            version_key,
        } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            wallet.load_hotkey(hotkey_name)?;
            let (uids, wts) = parse_weight_pairs(&weights)?;
            let salt_u16: Vec<u16> = salt
                .as_bytes()
                .chunks(2)
                .map(|chunk| {
                    let b0 = chunk[0] as u16;
                    let b1 = if chunk.len() > 1 { chunk[1] as u16 } else { 0 };
                    (b1 << 8) | b0
                })
                .collect();
            println!(
                "Revealing {} weights on SN{} (version_key={})",
                uids.len(),
                netuid,
                version_key
            );
            let hash = client
                .reveal_weights(
                    wallet.hotkey()?,
                    NetUid(netuid),
                    &uids,
                    &wts,
                    &salt_u16,
                    version_key,
                )
                .await?;
            println!("Weights revealed. Tx: {}", hash);
            Ok(())
        }
        WeightCommands::CommitReveal {
            netuid,
            weights,
            version_key,
            wait,
        } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            wallet.load_hotkey(hotkey_name)?;
            let (uids, wts) = parse_weight_pairs(&weights)?;

            // Pre-flight: check commit-reveal is enabled
            let hyperparams = client.get_subnet_hyperparams(NetUid(netuid)).await?;
            let (cr_enabled, cr_interval, tempo) = match &hyperparams {
                Some(h) => (
                    h.commit_reveal_weights_enabled,
                    h.commit_reveal_weights_interval,
                    h.tempo as u64,
                ),
                None => anyhow::bail!("Subnet SN{} not found or hyperparams unavailable", netuid),
            };

            if !cr_enabled {
                eprintln!("Warning: SN{} does NOT have commit-reveal enabled. Using direct set_weights instead.", netuid);
                tracing::warn!(netuid = netuid, "Subnet does not have commit-reveal enabled; falling back to direct set_weights");
                let hash = client
                    .set_weights(wallet.hotkey()?, NetUid(netuid), &uids, &wts, version_key)
                    .await?;
                println!("Weights set (direct). Tx: {}", hash);
                return Ok(());
            }

            // Generate salt
            let salt_str: String = {
                use rand::Rng;
                rand::thread_rng()
                    .sample_iter(&rand::distributions::Alphanumeric)
                    .take(32)
                    .map(char::from)
                    .collect()
            };

            let commit_hash = crate::extrinsics::compute_weight_commit_hash(&uids, &wts, salt_str.as_bytes())
                .map_err(|e| anyhow::anyhow!("blake2 hash error: {:?}", e))?;

            let block_at_commit = client.get_block_number().await?;

            // Step 1: Commit
            println!("Committing {} weights on SN{}", uids.len(), netuid);
            println!("  Commit hash: 0x{}", hex::encode(commit_hash));
            let commit_tx = client
                .commit_weights(wallet.hotkey()?, NetUid(netuid), commit_hash)
                .await?;
            println!("  Committed. Tx: {}", commit_tx);

            // Step 2: Wait for reveal window
            // Reveal window opens after cr_interval tempos from the commit
            let reveal_after_blocks = cr_interval * tempo;
            let reveal_target = block_at_commit + reveal_after_blocks;
            println!(
                "\n  Waiting for reveal window (~{} blocks, ~{}m)...",
                reveal_after_blocks,
                reveal_after_blocks * 12 / 60
            );

            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(12)).await;
                let current_block = client.get_block_number().await?;
                if current_block >= reveal_target {
                    println!("  Reveal window open at block {}", current_block);
                    break;
                }
                let remaining = reveal_target.saturating_sub(current_block);
                eprint!(
                    "\r  Block {} — {} blocks remaining (~{}m {}s)   ",
                    current_block,
                    remaining,
                    remaining * 12 / 60,
                    (remaining * 12) % 60
                );
            }

            // Step 3: Reveal
            let salt_u16: Vec<u16> = salt_str
                .as_bytes()
                .chunks(2)
                .map(|chunk| {
                    let b0 = chunk[0] as u16;
                    let b1 = if chunk.len() > 1 { chunk[1] as u16 } else { 0 };
                    (b1 << 8) | b0
                })
                .collect();
            println!("\n  Revealing weights...");
            let reveal_tx = client
                .reveal_weights(
                    wallet.hotkey()?,
                    NetUid(netuid),
                    &uids,
                    &wts,
                    &salt_u16,
                    version_key,
                )
                .await?;
            println!("  Revealed. Tx: {}", reveal_tx);

            if wait {
                // Verify reveal was included
                let final_block = client.get_block_number().await?;
                println!(
                    "\n  Confirmed at block {}. Commit-reveal complete.",
                    final_block
                );
                print_json(&serde_json::json!({
                    "status": "complete",
                    "netuid": netuid,
                    "commit_tx": commit_tx,
                    "reveal_tx": reveal_tx,
                    "commit_block": block_at_commit,
                    "reveal_block": final_block,
                    "num_weights": uids.len(),
                }));
            }

            Ok(())
        }
        WeightCommands::Status { netuid } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            wallet.load_hotkey(hotkey_name)?;
            let hotkey_ss58 = wallet
                .hotkey_ss58()
                .ok_or_else(|| anyhow::anyhow!("No hotkey loaded"))?
                .to_string();

            let (commits, block, hyperparams, reveal_period) = tokio::try_join!(
                client.get_weight_commits(NetUid(netuid), &hotkey_ss58),
                client.get_block_number(),
                client.get_subnet_hyperparams(NetUid(netuid)),
                client.get_reveal_period_epochs(NetUid(netuid)),
            )?;

            let cr_enabled = hyperparams
                .as_ref()
                .map(|h| h.commit_reveal_weights_enabled)
                .unwrap_or(false);

            println!("Weight Commit Status — SN{}", netuid);
            println!("  Hotkey:          {}", crate::utils::short_ss58(&hotkey_ss58));
            println!("  Current block:   {}", block);
            println!("  Commit-reveal:   {}", if cr_enabled { "ENABLED" } else { "disabled" });
            println!("  Reveal period:   {} epochs", reveal_period);

            match commits {
                Some(entries) if !entries.is_empty() => {
                    println!("\n  Pending commits: {}\n", entries.len());
                    for (i, (hash, commit_block, first_reveal, last_reveal)) in
                        entries.iter().enumerate()
                    {
                        let status = if block < *first_reveal {
                            let remaining = first_reveal - block;
                            format!("WAITING ({} blocks until reveal window)", remaining)
                        } else if block <= *last_reveal {
                            let remaining = last_reveal - block;
                            format!("READY TO REVEAL ({} blocks remaining)", remaining)
                        } else {
                            "EXPIRED".to_string()
                        };

                        println!("  [{}] Hash:    0x{}", i + 1, hex::encode(hash.0));
                        println!("      Commit:  block {}", commit_block);
                        println!("      Reveal:  blocks {}..{}", first_reveal, last_reveal);
                        println!("      Status:  {}\n", status);
                    }
                }
                _ => {
                    println!("\n  No pending commits.");
                }
            }

            Ok(())
        }
    }
}
