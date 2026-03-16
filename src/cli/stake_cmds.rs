//! Stake command handlers.

use crate::chain::Client;
use crate::cli::helpers::*;
use crate::cli::{OutputFormat, StakeCommands};
use crate::types::{Balance, NetUid};
use anyhow::Result;

pub async fn handle_stake(cmd: StakeCommands, client: &Client, ctx: &Ctx<'_>) -> Result<()> {
    let (wallet_dir, wallet_name, hotkey_name) = (ctx.wallet_dir, ctx.wallet_name, ctx.hotkey_name);
    let (output, password, mev) = (ctx.output, ctx.password, ctx.mev);
    match cmd {
        StakeCommands::List { address, at_block } => {
            let addr = resolve_coldkey_address(address, wallet_dir, wallet_name);

            // Historical wayback mode
            if let Some(block_num) = at_block {
                let block_hash = client.get_block_hash(block_num).await?;
                let stakes = client
                    .get_stake_for_coldkey_at_block(&addr, block_hash)
                    .await?;
                if output.is_json() {
                    print_json(&serde_json::json!({
                        "address": addr,
                        "block": block_num,
                        "block_hash": format!("{:?}", block_hash),
                        "stakes": stakes.iter().map(|s| serde_json::json!({
                            "netuid": s.netuid.0,
                            "hotkey": s.hotkey,
                            "stake_rao": s.stake.rao(),
                            "alpha_raw": s.alpha_stake.raw(),
                        })).collect::<Vec<_>>(),
                    }));
                } else if stakes.is_empty() {
                    println!(
                        "No stakes found for {} at block {}",
                        crate::utils::short_ss58(&addr),
                        block_num
                    );
                } else {
                    render_rows(
                        OutputFormat::Table,
                        &stakes,
                        "",
                        |_| String::new(),
                        &["Subnet", "Hotkey", "Stake (τ)", "Alpha"],
                        |s| {
                            vec![
                                format!("SN{}", s.netuid),
                                crate::utils::short_ss58(&s.hotkey),
                                s.stake.display_tao(),
                                format!("{}", s.alpha_stake),
                            ]
                        },
                        Some(&format!(
                            "Stakes for {} (at block {}):",
                            crate::utils::short_ss58(&addr),
                            block_num
                        )),
                    );
                }
                return Ok(());
            }

            let stakes = client.get_stake_for_coldkey(&addr).await?;
            if stakes.is_empty() && !output.is_json() && !output.is_csv() {
                println!("No stakes found for {}", crate::utils::short_ss58(&addr));
            } else {
                render_rows(
                    output,
                    &stakes,
                    "netuid,hotkey,stake_rao,alpha_raw",
                    |s| {
                        format!(
                            "{},{},{},{}",
                            s.netuid,
                            s.hotkey,
                            s.stake.rao(),
                            s.alpha_stake.raw()
                        )
                    },
                    &["Subnet", "Hotkey", "Stake (τ)", "Alpha"],
                    |s| {
                        vec![
                            format!("SN{}", s.netuid),
                            crate::utils::short_ss58(&s.hotkey),
                            s.stake.display_tao(),
                            format!("{}", s.alpha_stake),
                        ]
                    },
                    Some(&format!("Stakes for {}:", crate::utils::short_ss58(&addr))),
                );
            }
            Ok(())
        }
        StakeCommands::Add {
            amount,
            netuid,
            hotkey,
            max_slippage,
        } => {
            // Spending limit check
            check_spending_limit(netuid, amount)?;
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey, password)?;
            let bal = Balance::from_tao(amount);
            let pubkey = sp_core::Pair::public(&pair);
            // Pre-flight checks: balance + slippage in parallel when both needed
            if let Some(max_slip) = max_slippage {
                let (current, _) = tokio::try_join!(
                    client.get_balance(&pubkey),
                    check_slippage(client, netuid, amount, max_slip, true),
                )?;
                if current.rao() < bal.rao() {
                    anyhow::bail!("Insufficient balance: you have {} but trying to stake {}.\n  Check: agcli balance",
                        current.display_tao(), bal.display_tao());
                }
            } else {
                let current = client.get_balance(&pubkey).await?;
                if current.rao() < bal.rao() {
                    anyhow::bail!("Insufficient balance: you have {} but trying to stake {}.\n  Check: agcli balance",
                        current.display_tao(), bal.display_tao());
                }
            }
            if mev {
                eprintln!("MEV shield: encrypting stake operation");
                tracing::info!("MEV shield: encrypting stake operation");
            }
            stake_op(
                "Adding",
                "added",
                &hk,
                client
                    .add_stake_mev(&pair, &hk, NetUid(netuid), bal, mev)
                    .await,
            )
        }
        StakeCommands::Remove {
            amount,
            netuid,
            hotkey,
            max_slippage,
        } => {
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey, password)?;
            // Slippage check
            if let Some(max_slip) = max_slippage {
                check_slippage(client, netuid, amount, max_slip, false).await?;
            }
            if mev {
                eprintln!("MEV shield: encrypting unstake operation");
                tracing::info!("MEV shield: encrypting unstake operation");
            }
            stake_op(
                "Removing",
                "removed",
                &hk,
                client
                    .remove_stake_mev(&pair, &hk, NetUid(netuid), Balance::from_tao(amount), mev)
                    .await,
            )
        }
        StakeCommands::Move {
            amount,
            from,
            to,
            hotkey,
        } => {
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey, password)?;
            stake_op(
                "Moving",
                "moved",
                &hk,
                client
                    .move_stake(
                        &pair,
                        &hk,
                        NetUid(from),
                        NetUid(to),
                        Balance::from_tao(amount),
                    )
                    .await,
            )
        }
        StakeCommands::Swap {
            amount,
            netuid,
            from_hotkey,
            to_hotkey,
        } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            let bal = Balance::from_tao(amount);
            println!(
                "Swapping stake: {} on SN{} from {} to {}",
                bal.display_tao(),
                netuid,
                crate::utils::short_ss58(&from_hotkey),
                crate::utils::short_ss58(&to_hotkey)
            );
            let hash = client
                .swap_stake(
                    wallet.coldkey()?,
                    &from_hotkey,
                    NetUid(netuid),
                    NetUid(netuid),
                    bal,
                )
                .await?;
            println!("Stake swapped. Tx: {}", hash);
            Ok(())
        }
        StakeCommands::UnstakeAll { hotkey } => {
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey, password)?;
            stake_op(
                "Unstaking all from",
                "removed",
                &hk,
                client.unstake_all(&pair, &hk).await,
            )
        }
        StakeCommands::ClaimRoot { hotkey: _, netuid } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            let hash = client.claim_root(wallet.coldkey()?, &[netuid]).await?;
            println!("Root claimed for SN{}. Tx: {}", netuid, hash);
            Ok(())
        }
        StakeCommands::AddLimit {
            amount,
            netuid,
            price,
            partial,
            hotkey,
        } => {
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey, password)?;
            let bal = Balance::from_tao(amount);
            let lp = (price * 1_000_000_000.0) as u64;
            println!(
                "Adding stake limit: {} at {:.4} on SN{} (partial={})",
                bal.display_tao(),
                price,
                netuid,
                partial
            );
            let hash = client
                .add_stake_limit(&pair, &hk, NetUid(netuid), bal, lp, partial)
                .await?;
            println!("Limit stake added. Tx: {}", hash);
            Ok(())
        }
        StakeCommands::RemoveLimit {
            amount,
            netuid,
            price,
            partial,
            hotkey,
        } => {
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey, password)?;
            let lp = (price * 1_000_000_000.0) as u64;
            let amt = (amount * 1_000_000_000.0) as u64;
            println!(
                "Removing stake limit: {:.4} at {:.4} on SN{} (partial={})",
                amount, price, netuid, partial
            );
            let hash = client
                .remove_stake_limit(&pair, &hk, NetUid(netuid), amt, lp, partial)
                .await?;
            println!("Limit stake removed. Tx: {}", hash);
            Ok(())
        }
        StakeCommands::ChildkeyTake {
            take,
            netuid,
            hotkey,
        } => {
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey, password)?;
            let take_u16 = (take / 100.0 * 65535.0).min(65535.0) as u16;
            println!(
                "Setting childkey take to {:.2}% on SN{} for {}",
                take,
                netuid,
                crate::utils::short_ss58(&hk)
            );
            let hash = client
                .set_childkey_take(&pair, &hk, NetUid(netuid), take_u16)
                .await?;
            println!("Childkey take set. Tx: {}", hash);
            Ok(())
        }
        StakeCommands::SetChildren { netuid, children } => {
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, None, password)?;
            let children_parsed = parse_children(&children)?;
            println!(
                "Setting {} children on SN{} for {}",
                children_parsed.len(),
                netuid,
                crate::utils::short_ss58(&hk)
            );
            let hash = client
                .set_children(&pair, &hk, NetUid(netuid), &children_parsed)
                .await?;
            println!("Children set. Tx: {}", hash);
            Ok(())
        }
        StakeCommands::RecycleAlpha {
            amount,
            netuid,
            hotkey,
        } => {
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey, password)?;
            stake_op(
                "Recycling alpha via",
                "recycled",
                &hk,
                client
                    .recycle_alpha(&pair, &hk, NetUid(netuid), (amount * 1e9) as u64)
                    .await,
            )
        }
        StakeCommands::UnstakeAllAlpha { hotkey } => {
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey, password)?;
            stake_op(
                "Unstaking all alpha from",
                "unstaked",
                &hk,
                client.unstake_all_alpha(&pair, &hk).await,
            )
        }
        StakeCommands::BurnAlpha {
            amount,
            netuid,
            hotkey,
        } => {
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey, password)?;
            stake_op(
                "Burning alpha via",
                "burned",
                &hk,
                client
                    .burn_alpha(&pair, &hk, (amount * 1e9) as u64, NetUid(netuid))
                    .await,
            )
        }
        StakeCommands::SwapLimit {
            amount,
            from,
            to,
            price,
            partial,
            hotkey,
        } => {
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey, password)?;
            let amt = (amount * 1_000_000_000.0) as u64;
            let lp = (price * 1_000_000_000.0) as u64;
            println!(
                "Swap-limit {:.4} from SN{} to SN{} at price {:.4} (partial={})",
                amount, from, to, price, partial
            );
            let hash = client
                .swap_stake_limit(&pair, &hk, NetUid(from), NetUid(to), amt, lp, partial)
                .await?;
            println!("Swap limit submitted. Tx: {}", hash);
            Ok(())
        }
        StakeCommands::SetAuto { netuid, hotkey } => {
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey, password)?;
            println!(
                "Setting auto-stake on SN{} to hotkey {}...",
                netuid,
                crate::utils::short_ss58(&hk)
            );
            let hash = client.set_auto_stake(&pair, NetUid(netuid), &hk).await?;
            println!("Auto-stake set. Tx: {}", hash);
            Ok(())
        }
        StakeCommands::ShowAuto { address } => {
            let addr = resolve_coldkey_address(address, wallet_dir, wallet_name);
            let subnets = client.get_all_subnets().await?;
            // Parallel fetch: query all subnets concurrently instead of one-by-one
            let addr_ref = &addr;
            let futures: Vec<_> = subnets
                .iter()
                .map(|subnet| {
                    let netuid = subnet.netuid;
                    async move { (netuid, client.get_auto_stake_hotkey(addr_ref, netuid).await) }
                })
                .collect();
            let results = futures::future::join_all(futures).await;
            let mut found = false;
            for (netuid, result) in &results {
                if let Ok(Some(hotkey)) = result {
                    if !found {
                        println!(
                            "Auto-stake destinations for {}:",
                            crate::utils::short_ss58(&addr)
                        );
                        found = true;
                    }
                    println!("  SN{:<4} → {}", netuid, crate::utils::short_ss58(hotkey));
                }
            }
            if !found {
                println!(
                    "No auto-stake destinations set for {}",
                    crate::utils::short_ss58(&addr)
                );
            }
            Ok(())
        }
        StakeCommands::SetClaim {
            claim_type,
            subnets,
        } => {
            let (pair, _) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, None, password)?;
            let subnet_ids: Option<Vec<u16>> = subnets.as_ref().map(|s| {
                let mut ids = Vec::new();
                for n in s.split(',') {
                    let trimmed = n.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    match trimmed.parse::<u16>() {
                        Ok(id) => ids.push(id),
                        Err(_) => {
                            eprintln!("Warning: ignoring invalid subnet ID '{}'", trimmed);
                            tracing::warn!(input = trimmed, "Ignoring invalid subnet ID");
                        }
                    }
                }
                ids
            });
            let keep_subnets = subnet_ids.as_deref();
            println!(
                "Setting root claim type to '{}'{}...",
                claim_type,
                keep_subnets
                    .map(|s| format!(" (subnets: {:?})", s))
                    .unwrap_or_default()
            );
            let hash = client
                .set_root_claim_type(&pair, &claim_type, keep_subnets)
                .await?;
            println!("Root claim type set. Tx: {}", hash);
            Ok(())
        }
        StakeCommands::TransferStake {
            dest,
            amount,
            from,
            to,
            hotkey,
        } => {
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey, password)?;
            let amt = Balance::from_tao(amount);
            println!(
                "Transferring {:.4} TAO stake from SN{} to SN{} → {}",
                amount,
                from,
                to,
                crate::utils::short_ss58(&dest)
            );
            let hash = client
                .transfer_stake(&pair, &dest, &hk, NetUid(from), NetUid(to), amt)
                .await?;
            println!("Stake transferred. Tx: {}", hash);
            Ok(())
        }
        StakeCommands::ProcessClaim { hotkey, netuids } => {
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey, password)?;
            // Parse optional netuid filter
            let filter_netuids: Option<Vec<u16>> = netuids.as_ref().map(|s| {
                s.split(',')
                    .filter_map(|n| n.trim().parse::<u16>().ok())
                    .collect()
            });

            // Fetch stakes to find all netuids where this hotkey has root emissions
            let coldkey_ss58 = resolve_coldkey_address(None, wallet_dir, wallet_name);
            let stakes = client.get_stake_for_coldkey(&coldkey_ss58).await?;

            // Filter to netuids where we have stake on this hotkey
            let target_netuids: Vec<u16> = stakes
                .iter()
                .filter(|s| s.hotkey == hk)
                .filter(|s| {
                    filter_netuids
                        .as_ref()
                        .map(|f| f.contains(&s.netuid.0))
                        .unwrap_or(true)
                })
                .map(|s| s.netuid.0)
                .collect();

            if target_netuids.is_empty() {
                println!(
                    "No stakes found for hotkey {} to claim from.",
                    crate::utils::short_ss58(&hk)
                );
                return Ok(());
            }

            println!(
                "Processing root claims for hotkey {} across {} subnet(s): {:?}",
                crate::utils::short_ss58(&hk),
                target_netuids.len(),
                target_netuids
            );

            let mut success = 0u32;
            let mut failed = 0u32;
            for nuid in &target_netuids {
                match client
                    .submit_raw_call(
                        &pair,
                        "SubtensorModule",
                        "claim_root_dividends",
                        vec![
                            subxt::dynamic::Value::from_bytes(
                                Client::ss58_to_account_id_pub(&hk)?.0,
                            ),
                            subxt::dynamic::Value::u128(*nuid as u128),
                        ],
                    )
                    .await
                {
                    Ok(hash) => {
                        println!("  SN{}: claimed (tx: {})", nuid, hash);
                        success += 1;
                    }
                    Err(e) => {
                        eprintln!("  SN{}: failed — {}", nuid, e);
                        failed += 1;
                    }
                }
            }
            println!(
                "\nDone: {} claimed, {} failed out of {} total",
                success,
                failed,
                target_netuids.len()
            );
            Ok(())
        }
        StakeCommands::Wizard {
            netuid,
            amount,
            hotkey,
        } => staking_wizard(client, ctx, netuid, amount, hotkey).await,
    }
}

/// Common pattern for stake operations: print action, handle result.
fn stake_op(action: &str, past: &str, hotkey: &str, result: Result<String>) -> Result<()> {
    println!("{} {}", action, crate::utils::short_ss58(hotkey));
    let hash = result?;
    println!("Stake {}. Tx: {}", past, hash);
    Ok(())
}

/// Check AMM slippage before a stake/unstake operation. Aborts if slippage exceeds max.
async fn check_slippage(
    client: &Client,
    netuid: u16,
    amount: f64,
    max_slip_pct: f64,
    is_buy: bool,
) -> Result<()> {
    let nuid = NetUid(netuid);
    let rao = (amount * 1e9) as u64;
    let slippage = if is_buy {
        // Staking: TAO → Alpha — parallel fetch price + simulation
        let (price_raw, (out, _tf, _af)) = tokio::try_join!(
            client.current_alpha_price(nuid),
            client.sim_swap_tao_for_alpha(nuid, rao),
        )?;
        let price = price_raw as f64 / 1e9;
        let out_f = out as f64 / 1e9;
        let eff_price = if out_f > 0.0 { amount / out_f } else { 0.0 };
        if price > 0.0 {
            ((eff_price - price) / price).abs() * 100.0
        } else {
            0.0
        }
    } else {
        // Unstaking: Alpha → TAO — parallel fetch price + simulation
        let (price_raw, (out, _tf, _af)) = tokio::try_join!(
            client.current_alpha_price(nuid),
            client.sim_swap_alpha_for_tao(nuid, rao),
        )?;
        let price = price_raw as f64 / 1e9;
        let out_f = out as f64 / 1e9;
        let eff_price = if amount > 0.0 { out_f / amount } else { 0.0 };
        if price > 0.0 {
            ((eff_price - price) / price).abs() * 100.0
        } else {
            0.0
        }
    };
    if slippage > max_slip_pct {
        anyhow::bail!(
            "Slippage {:.2}% exceeds maximum allowed {:.2}% on SN{}.\n  Reduce trade size or use a limit order: agcli stake add-limit / remove-limit",
            slippage, max_slip_pct, netuid
        );
    }
    if slippage > 2.0 {
        eprintln!(
            "Warning: estimated slippage is {:.2}% on SN{}",
            slippage, netuid
        );
        tracing::warn!(
            slippage_pct = slippage,
            netuid = netuid,
            "Estimated slippage exceeds 2%"
        );
    }
    Ok(())
}

async fn staking_wizard(
    client: &Client,
    ctx: &Ctx<'_>,
    netuid_arg: Option<u16>,
    amount_arg: Option<f64>,
    hotkey_arg: Option<String>,
) -> Result<()> {
    let (wallet_dir, wallet_name, hotkey_name, password, yes) = (
        ctx.wallet_dir,
        ctx.wallet_name,
        ctx.hotkey_name,
        ctx.password,
        ctx.yes,
    );
    println!("=== Staking Wizard ===\n");

    let mut wallet = open_wallet(wallet_dir, wallet_name)?;
    let coldkey_ss58 = match wallet.coldkey_ss58() {
        Some(s) => s.to_string(),
        None => {
            // Public key not on disk; unlock to derive it
            unlock_coldkey(&mut wallet, password)?;
            wallet
                .coldkey_ss58()
                .map(|s| s.to_string())
                .ok_or_else(|| anyhow::anyhow!("Could not resolve coldkey address"))?
        }
    };
    println!(
        "Wallet: {} ({})",
        wallet_name,
        crate::utils::short_ss58(&coldkey_ss58)
    );

    let (balance, dynamic) = tokio::try_join!(
        client.get_balance_ss58(&coldkey_ss58),
        client.get_all_dynamic_info(),
    )?;
    println!("Balance: {}\n", balance.display_tao());

    if balance.rao() == 0 {
        println!("You need TAO to stake. Transfer some TAO to your coldkey first.");
        return Ok(());
    }
    let mut subnets_with_pool: Vec<_> = dynamic.iter().filter(|d| d.tao_in.rao() > 0).collect();
    subnets_with_pool.sort_by(|a, b| b.tao_in.rao().cmp(&a.tao_in.rao()));

    println!("\nTop subnets by TAO pool:");
    let display_count = subnets_with_pool.len().min(15);
    for (i, d) in subnets_with_pool.iter().take(display_count).enumerate() {
        println!(
            "  {:>2}. SN{:<3} {:<20} price={:.6} τ/α  pool={:.2} τ",
            i + 1,
            d.netuid,
            &d.name,
            d.price,
            d.tao_in.tao(),
        );
    }

    // Resolve netuid: from CLI flag or interactive prompt
    let netuid: u16 = match netuid_arg {
        Some(n) => n,
        None => {
            let netuid_input: String = dialoguer::Input::new()
                .with_prompt("\nEnter subnet netuid to stake on")
                .interact_text()?;
            netuid_input
                .trim()
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid netuid"))?
        }
    };

    // Resolve amount: from CLI flag or interactive prompt
    let max_tao = balance.tao();
    let amount: f64 = match amount_arg {
        Some(a) => a,
        None => {
            let amount_input: String = dialoguer::Input::new()
                .with_prompt(format!("Amount of TAO to stake (max {:.4})", max_tao))
                .interact_text()?;
            amount_input
                .trim()
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid amount"))?
        }
    };

    if amount <= 0.0 {
        anyhow::bail!("Amount must be greater than 0.");
    }
    if amount > max_tao {
        anyhow::bail!("Amount {:.4} τ exceeds your balance of {:.4} τ. Use a smaller amount or transfer more TAO first.", amount, max_tao);
    }

    let hotkey_ss58 = resolve_hotkey_ss58(hotkey_arg, &mut wallet, hotkey_name)?;
    println!(
        "\nStaking {:.4} τ on SN{} with hotkey {}",
        amount,
        netuid,
        crate::utils::short_ss58(&hotkey_ss58)
    );

    // Confirm: skip if --yes, otherwise prompt
    if !yes {
        let confirm = dialoguer::Confirm::new()
            .with_prompt("Proceed?")
            .default(true)
            .interact()?;

        if !confirm {
            println!("Cancelled.");
            return Ok(());
        }
    }

    unlock_coldkey(&mut wallet, password)?;
    let stake_balance = Balance::from_tao(amount);
    let hash = client
        .add_stake(
            wallet.coldkey()?,
            &hotkey_ss58,
            NetUid(netuid),
            stake_balance,
        )
        .await?;
    println!("Stake added! Tx: {}", hash);

    println!("\nUpdated portfolio:");
    let portfolio = crate::queries::portfolio::fetch_portfolio(client, &coldkey_ss58).await?;
    println!("  Free:   {}", portfolio.free_balance.display_tao());
    println!("  Staked: {}", portfolio.total_staked.display_tao());

    Ok(())
}
