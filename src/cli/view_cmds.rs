//! View command handlers (portfolio, network, dynamic, neuron, validators, history, account, analytics).

use crate::chain::Client;
use crate::cli::helpers::*;
use crate::cli::ViewCommands;
use crate::types::{Balance, NetUid};
use anyhow::Result;

pub async fn handle_view(
    cmd: ViewCommands,
    client: &Client,
    wallet_dir: &str,
    wallet_name: &str,
    output: &str,
    live_interval: Option<u64>,
) -> Result<()> {
    match cmd {
        ViewCommands::Portfolio { address, at_block } => {
            let addr = resolve_coldkey_address(address, wallet_dir, wallet_name);
            if let Some(bn) = at_block {
                return handle_portfolio_at_block(client, &addr, output, bn).await;
            }
            if let Some(interval) = live_interval {
                return crate::live::live_portfolio(client, &addr, interval).await;
            }
            handle_portfolio(client, &addr, output).await
        }
        ViewCommands::Network { at_block } => handle_network(client, output, at_block).await,
        ViewCommands::Dynamic { at_block } => {
            if let Some(bn) = at_block {
                return handle_dynamic_at_block(client, output, bn).await;
            }
            if let Some(interval) = live_interval {
                return crate::live::live_dynamic(client, interval).await;
            }
            handle_dynamic(client, output).await
        }
        ViewCommands::Neuron {
            netuid,
            uid,
            at_block,
        } => handle_neuron(client, netuid, uid, at_block).await,
        ViewCommands::Validators {
            netuid,
            limit,
            at_block,
        } => handle_validators(client, output, netuid, limit, at_block).await,
        ViewCommands::History { address, limit } => {
            let addr = resolve_coldkey_address(address, wallet_dir, wallet_name);
            handle_history(&addr, output, limit).await
        }
        ViewCommands::Account { address, at_block } => {
            let addr = resolve_coldkey_address(address, wallet_dir, wallet_name);
            handle_account_explorer(client, &addr, output, at_block).await
        }
        ViewCommands::SubnetAnalytics { netuid } => {
            handle_subnet_analytics(client, netuid, output).await
        }
        ViewCommands::StakingAnalytics { address } => {
            let addr = resolve_coldkey_address(address, wallet_dir, wallet_name);
            handle_staking_analytics(client, &addr, output).await
        }
        ViewCommands::SwapSim { netuid, tao, alpha } => {
            handle_swap_sim(client, netuid, tao, alpha, output).await
        }
        ViewCommands::Nominations { hotkey } => handle_nominations(client, &hotkey, output).await,
    }
}

async fn handle_portfolio(client: &Client, addr: &str, output: &str) -> Result<()> {
    let portfolio = crate::queries::portfolio::fetch_portfolio(client, addr).await?;
    if output == "json" {
        print_json_ser(&portfolio);
    } else {
        if output != "csv" {
            println!("Portfolio for {}", crate::utils::short_ss58(addr));
            println!("  Free:   {}", portfolio.free_balance.display_tao());
            println!("  Staked: {}", portfolio.total_staked.display_tao());
            println!(
                "  Total:  {}",
                (portfolio.free_balance + portfolio.total_staked).display_tao()
            );
        }
        if !portfolio.positions.is_empty() {
            render_rows(
                output,
                &portfolio.positions,
                "netuid,subnet_name,hotkey,alpha_stake,tao_equiv_rao,price",
                |p| {
                    format!(
                        "{},{},{},{},{},{:.6}",
                        p.netuid,
                        p.subnet_name,
                        p.hotkey_ss58,
                        p.alpha_stake,
                        p.tao_equivalent.rao(),
                        p.price
                    )
                },
                &["Subnet", "Name", "Hotkey", "Alpha", "TAO Equiv", "Price"],
                |p| {
                    vec![
                        format!("SN{}", p.netuid),
                        p.subnet_name.clone(),
                        crate::utils::short_ss58(&p.hotkey_ss58),
                        format!("{}", p.alpha_stake),
                        format!("{}", p.tao_equivalent),
                        format!("{:.4}", p.price),
                    ]
                },
                None,
            );
        }
    }
    Ok(())
}

async fn handle_network(client: &Client, output: &str, at_block: Option<u32>) -> Result<()> {
    // Historical wayback mode
    if let Some(block_num) = at_block {
        let block_hash = client.get_block_hash(block_num).await?;
        let (total_stake, total_issuance) =
            tokio::try_join!(client.get_total_stake_at_block(block_hash), async {
                // Total issuance at block
                let addr = crate::api::storage().balances().total_issuance();
                let val = client.subxt().storage().at(block_hash).fetch(&addr).await?;
                Ok::<_, anyhow::Error>(Balance::from_rao(val.unwrap_or(0) as u64))
            },)?;
        let staking_ratio = if total_issuance.rao() > 0 {
            total_stake.tao() / total_issuance.tao() * 100.0
        } else {
            0.0
        };
        if output == "json" {
            print_json(&serde_json::json!({
                "block": block_num,
                "block_hash": format!("{:?}", block_hash),
                "total_issuance_rao": total_issuance.rao(),
                "total_issuance_tao": total_issuance.tao(),
                "total_stake_rao": total_stake.rao(),
                "total_stake_tao": total_stake.tao(),
                "staking_ratio_pct": staking_ratio,
            }));
        } else {
            println!("Network Overview (at block {})", block_num);
            println!("  Block hash:   {:?}", block_hash);
            println!("  Total issued: {}", total_issuance.display_tao());
            println!("  Total staked: {}", total_stake.display_tao());
            println!("  Staking ratio: {:.1}%", staking_ratio);
        }
        return Ok(());
    }

    let (block, total_stake, total_networks, total_issuance, emission) = tokio::try_join!(
        client.get_block_number(),
        client.get_total_stake(),
        client.get_total_networks(),
        client.get_total_issuance(),
        client.get_block_emission(),
    )?;
    let staking_ratio = if total_issuance.rao() > 0 {
        total_stake.tao() / total_issuance.tao() * 100.0
    } else {
        0.0
    };
    if output == "json" {
        print_json(&serde_json::json!({
            "block": block,
            "subnets": total_networks,
            "total_issuance_rao": total_issuance.rao(),
            "total_issuance_tao": total_issuance.tao(),
            "total_stake_rao": total_stake.rao(),
            "total_stake_tao": total_stake.tao(),
            "emission_per_block_rao": emission.rao(),
            "staking_ratio_pct": staking_ratio,
        }));
    } else {
        println!("Network Overview");
        println!("  Block:        {}", block);
        println!("  Subnets:      {}", total_networks);
        println!("  Total issued: {}", total_issuance.display_tao());
        println!("  Total staked: {}", total_stake.display_tao());
        println!("  Emission/blk: {}", emission.display_tao());
        println!("  Staking ratio: {:.1}%", staking_ratio);
    }
    Ok(())
}

async fn handle_dynamic(client: &Client, output: &str) -> Result<()> {
    let dynamic = client.get_all_dynamic_info().await?;
    render_rows(
        output,
        &dynamic,
        "netuid,name,symbol,tempo,price,tao_in_rao,alpha_in,alpha_out,emission,volume",
        |d| {
            format!(
                "{},{},{},{},{:.6},{},{},{},{},{}",
                d.netuid,
                d.name,
                d.symbol,
                d.tempo,
                d.price,
                d.tao_in.rao(),
                d.alpha_in.raw(),
                d.alpha_out.raw(),
                d.total_emission(),
                d.subnet_volume
            )
        },
        &[
            "NetUID",
            "Name",
            "Symbol",
            "Price (τ/α)",
            "TAO In",
            "Alpha In",
            "Alpha Out",
            "Emission",
            "Tempo",
        ],
        |d| {
            vec![
                format!("{}", d.netuid),
                d.name.clone(),
                d.symbol.clone(),
                format!("{:.6}", d.price),
                d.tao_in.display_tao(),
                format!("{}", d.alpha_in),
                format!("{}", d.alpha_out),
                format!("{:.4} τ", d.total_emission() as f64 / 1e9),
                format!("{}", d.tempo),
            ]
        },
        Some(&format!("Dynamic TAO — {} subnets", dynamic.len())),
    );
    Ok(())
}

async fn handle_portfolio_at_block(
    client: &Client,
    addr: &str,
    output: &str,
    block_num: u32,
) -> Result<()> {
    let block_hash = client.get_block_hash(block_num).await?;
    let (balance, stakes) = tokio::try_join!(
        client.get_balance_at_block(addr, block_hash),
        client.get_stake_for_coldkey_at_block(addr, block_hash),
    )?;
    let total_staked: u64 = stakes.iter().map(|s| s.stake.rao()).sum();
    if output == "json" {
        print_json(&serde_json::json!({
            "address": addr,
            "block": block_num,
            "free_balance_rao": balance.rao(),
            "free_balance_tao": balance.tao(),
            "total_staked_rao": total_staked,
            "total_staked_tao": total_staked as f64 / 1e9,
            "stakes": stakes.iter().map(|s| serde_json::json!({
                "hotkey": s.hotkey,
                "netuid": s.netuid.0,
                "stake_rao": s.stake.rao(),
                "stake_tao": s.stake.tao(),
            })).collect::<Vec<_>>(),
        }));
    } else {
        println!(
            "Portfolio for {} (block {})",
            crate::utils::short_ss58(addr),
            block_num
        );
        println!("  Free:   {}", balance.display_tao());
        println!(
            "  Staked: {}",
            Balance::from_rao(total_staked).display_tao()
        );
        println!(
            "  Total:  {}",
            (balance + Balance::from_rao(total_staked)).display_tao()
        );
        if !stakes.is_empty() {
            render_rows(
                output,
                &stakes,
                "netuid,hotkey,stake_rao",
                |s| format!("{},{},{}", s.netuid, s.hotkey, s.stake.rao()),
                &["NetUID", "Hotkey", "Stake"],
                |s| {
                    vec![
                        format!("{}", s.netuid),
                        crate::utils::short_ss58(&s.hotkey),
                        s.stake.display_tao(),
                    ]
                },
                None,
            );
        }
    }
    Ok(())
}

async fn handle_dynamic_at_block(client: &Client, output: &str, block_num: u32) -> Result<()> {
    let block_hash = client.get_block_hash(block_num).await?;
    let dynamic = client.get_all_dynamic_info_at_block(block_hash).await?;
    render_rows(
        output,
        &dynamic,
        "netuid,name,symbol,tempo,price,tao_in_rao,alpha_in,alpha_out,emission,volume",
        |d| {
            format!(
                "{},{},{},{},{:.6},{},{},{},{},{}",
                d.netuid,
                d.name,
                d.symbol,
                d.tempo,
                d.price,
                d.tao_in.rao(),
                d.alpha_in.raw(),
                d.alpha_out.raw(),
                d.total_emission(),
                d.subnet_volume
            )
        },
        &[
            "NetUID",
            "Name",
            "Symbol",
            "Price (τ/α)",
            "TAO In",
            "Alpha In",
            "Alpha Out",
            "Emission",
            "Tempo",
        ],
        |d| {
            vec![
                format!("{}", d.netuid),
                d.name.clone(),
                d.symbol.clone(),
                format!("{:.6}", d.price),
                d.tao_in.display_tao(),
                format!("{}", d.alpha_in),
                format!("{}", d.alpha_out),
                format!("{:.4} τ", d.total_emission() as f64 / 1e9),
                format!("{}", d.tempo),
            ]
        },
        Some(&format!(
            "Dynamic TAO at block {} — {} subnets",
            block_num,
            dynamic.len()
        )),
    );
    Ok(())
}

async fn handle_neuron(
    client: &Client,
    netuid: u16,
    uid: u16,
    at_block: Option<u32>,
) -> Result<()> {
    let neuron = if let Some(bn) = at_block {
        let bh = client.get_block_hash(bn).await?;
        client.get_neuron_at_block(NetUid(netuid), uid, bh).await?
    } else {
        client.get_neuron(NetUid(netuid), uid).await?
    };
    match neuron {
        Some(n) => {
            println!("Neuron UID {} on SN{}", uid, netuid);
            println!("  Hotkey:          {}", n.hotkey);
            println!("  Coldkey:         {}", n.coldkey);
            println!("  Active:          {}", n.active);
            println!("  Stake:           {}", n.stake.display_tao());
            println!("  Rank:            {:.6}", n.rank);
            println!("  Trust:           {:.6}", n.trust);
            println!("  Consensus:       {:.6}", n.consensus);
            println!("  Incentive:       {:.6}", n.incentive);
            println!("  Dividends:       {:.6}", n.dividends);
            println!("  Emission:        {:.4} τ", n.emission / 1e9);
            println!("  Val. Trust:      {:.6}", n.validator_trust);
            println!("  Val. Permit:     {}", n.validator_permit);
            println!("  Pruning Score:   {:.6}", n.pruning_score);
            println!("  Last Update:     {}", n.last_update);
            if let Some(axon) = &n.axon_info {
                println!(
                    "  Axon:            {}:{} (v{}, proto {})",
                    axon.ip, axon.port, axon.version, axon.protocol
                );
            }
            if let Some(prom) = &n.prometheus_info {
                println!(
                    "  Prometheus:      {}:{} (v{})",
                    prom.ip, prom.port, prom.version
                );
            }
        }
        None => println!("Neuron UID {} not found on SN{}", uid, netuid),
    }
    Ok(())
}

async fn handle_validators(
    client: &Client,
    output: &str,
    netuid: Option<u16>,
    limit: usize,
    at_block: Option<u32>,
) -> Result<()> {
    if let Some(nuid) = netuid {
        let neurons = if let Some(bn) = at_block {
            let bh = client.get_block_hash(bn).await?;
            client.get_neurons_lite_at_block(NetUid(nuid), bh).await?
        } else {
            client.get_neurons_lite(NetUid(nuid)).await?
        };
        let mut validators: Vec<_> = neurons.into_iter().filter(|n| n.validator_permit).collect();
        validators.sort_by(|a, b| b.stake.rao().cmp(&a.stake.rao()));
        validators.truncate(limit);

        render_rows(
            output,
            &validators,
            "uid,hotkey,coldkey,stake_rao,trust,vtrust,dividends,emission",
            |v| {
                format!(
                    "{},{},{},{},{:.6},{:.6},{:.6},{:.0}",
                    v.uid,
                    v.hotkey,
                    v.coldkey,
                    v.stake.rao(),
                    v.trust,
                    v.validator_trust,
                    v.dividends,
                    v.emission
                )
            },
            &["UID", "Hotkey", "Stake", "VTrust", "Dividends", "Emission"],
            |v| {
                vec![
                    format!("{}", v.uid),
                    crate::utils::short_ss58(&v.hotkey),
                    format!("{:.4}τ", v.stake.tao()),
                    format!("{:.4}", v.validator_trust),
                    format!("{:.4}", v.dividends),
                    format!("{:.4} τ", v.emission / 1e9),
                ]
            },
            Some(&format!(
                "Validators on SN{} ({} with permits)",
                nuid,
                validators.len()
            )),
        );
    } else {
        let delegates = if let Some(bn) = at_block {
            let bh = client.get_block_hash(bn).await?;
            client.get_delegates_at_block(bh).await?
        } else {
            client.get_delegates().await?
        };
        let mut sorted = delegates;
        sorted.sort_by(|a, b| b.total_stake.rao().cmp(&a.total_stake.rao()));
        sorted.truncate(limit);

        // Add rank index for table display
        let ranked: Vec<(usize, _)> = sorted.into_iter().enumerate().collect();
        let title = if let Some(bn) = at_block {
            format!(
                "Top {} validators by total stake (block {})",
                ranked.len(),
                bn
            )
        } else {
            format!("Top {} validators by total stake", ranked.len())
        };
        render_rows(
            output,
            &ranked,
            "rank,hotkey,owner,take_pct,total_stake_rao,nominators,registrations",
            |(_, d)| {
                format!(
                    "{},{},{:.2},{},{},{:?}",
                    d.hotkey,
                    d.owner,
                    d.take * 100.0,
                    d.total_stake.rao(),
                    d.nominators.len(),
                    d.registrations
                )
            },
            &[
                "#",
                "Hotkey",
                "Owner",
                "Take",
                "Total Stake",
                "Nominators",
                "Subnets",
            ],
            |(i, d)| {
                vec![
                    format!("{}", i + 1),
                    crate::utils::short_ss58(&d.hotkey),
                    crate::utils::short_ss58(&d.owner),
                    format!("{:.2}%", d.take * 100.0),
                    d.total_stake.display_tao(),
                    format!("{}", d.nominators.len()),
                    format!("{}", d.registrations.len()),
                ]
            },
            Some(&title),
        );
    }
    Ok(())
}

async fn handle_history(address: &str, output: &str, limit: usize) -> Result<()> {
    println!(
        "Fetching transaction history for {}...",
        crate::utils::short_ss58(address)
    );
    let url = "https://bittensor.api.subscan.io/api/v2/scan/extrinsics";
    let body = serde_json::json!({
        "address": address,
        "row": limit.min(100),
        "page": 0,
    });
    let http = reqwest::Client::new();
    let resp = http
        .post(url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    let json: serde_json::Value = resp.json().await?;
    let extrinsics = json
        .get("data")
        .and_then(|d| d.get("extrinsics"))
        .and_then(|e| e.as_array());

    match extrinsics {
        Some(txs) if !txs.is_empty() => {
            render_rows(
                output,
                txs,
                "block,hash,module,call,success,timestamp",
                |tx| {
                    format!(
                        "{},{},{},{},{},{}",
                        tx.get("block_num").and_then(|v| v.as_u64()).unwrap_or(0),
                        tx.get("extrinsic_hash")
                            .and_then(|v| v.as_str())
                            .unwrap_or(""),
                        tx.get("call_module").and_then(|v| v.as_str()).unwrap_or(""),
                        tx.get("call_module_function")
                            .and_then(|v| v.as_str())
                            .unwrap_or(""),
                        tx.get("success").and_then(|v| v.as_bool()).unwrap_or(false),
                        tx.get("block_timestamp")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    )
                },
                &["Block", "Module", "Call", "Success", "Hash"],
                |tx| {
                    let block = tx.get("block_num").and_then(|v| v.as_u64()).unwrap_or(0);
                    let module = tx
                        .get("call_module")
                        .and_then(|v| v.as_str())
                        .unwrap_or("?");
                    let call = tx
                        .get("call_module_function")
                        .and_then(|v| v.as_str())
                        .unwrap_or("?");
                    let success = tx.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
                    let hash = tx
                        .get("extrinsic_hash")
                        .and_then(|v| v.as_str())
                        .unwrap_or("?");
                    let hash_short = if hash.len() > 18 { &hash[..18] } else { hash };
                    vec![
                        format!("{}", block),
                        module.to_string(),
                        call.to_string(),
                        if success { "OK" } else { "FAIL" }.to_string(),
                        format!("{}...", hash_short),
                    ]
                },
                Some(&format!("Recent transactions ({}):", txs.len())),
            );
        }
        _ => {
            println!(
                "No transactions found for {}",
                crate::utils::short_ss58(address)
            );
            println!("Note: Subscan API may have rate limits or the address may have no activity.");
        }
    }
    Ok(())
}

async fn handle_account_explorer(
    client: &Client,
    address: &str,
    output: &str,
    at_block: Option<u32>,
) -> Result<()> {
    // Historical wayback mode
    if let Some(block_num) = at_block {
        let block_hash = client.get_block_hash(block_num).await?;
        let (balance, stakes, identity) = tokio::try_join!(
            client.get_balance_at_block(address, block_hash),
            client.get_stake_for_coldkey_at_block(address, block_hash),
            client.get_identity_at_block(address, block_hash),
        )?;
        let total_staked: f64 = stakes.iter().map(|s| s.stake.tao()).sum();
        let total_value = balance.tao() + total_staked;

        if output == "json" {
            let positions: Vec<serde_json::Value> = stakes
                .iter()
                .map(|s| {
                    serde_json::json!({
                        "netuid": s.netuid.0,
                        "hotkey": s.hotkey,
                        "stake_rao": s.stake.rao(),
                        "alpha_raw": s.alpha_stake.raw(),
                    })
                })
                .collect();
            print_json(&serde_json::json!({
                "address": address,
                "block": block_num,
                "block_hash": format!("{:?}", block_hash),
                "balance_rao": balance.rao(),
                "balance_tao": balance.tao(),
                "total_staked_tao": total_staked,
                "total_value_tao": total_value,
                "stakes": positions,
                "identity": identity.as_ref().map(|id| serde_json::json!({
                    "name": id.name, "url": id.url, "discord": id.discord,
                })),
            }));
            return Ok(());
        }

        println!("Account: {} (at block {})\n", address, block_num);
        println!("  Block hash:    {:?}", block_hash);
        println!("  Free balance:  {}", balance.display_tao());
        println!("  Total staked:  {:.4} τ", total_staked);
        println!("  Total value:   {:.4} τ", total_value);

        if let Some(id) = &identity {
            if !id.name.is_empty() {
                println!("\n  Identity:");
                println!("    Name:    {}", id.name);
            }
        }

        if !stakes.is_empty() {
            render_rows(
                "table",
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
                Some(&format!("\n  Stake Positions ({}):", stakes.len())),
            );
        }
        return Ok(());
    }

    let (balance, stakes, identity, dynamic, delegate) = tokio::try_join!(
        client.get_balance_ss58(address),
        client.get_stake_for_coldkey(address),
        client.get_identity(address),
        async {
            match client.get_all_dynamic_info().await {
                Ok(d) => Ok::<_, anyhow::Error>(d),
                Err(e) => { tracing::warn!("get_all_dynamic_info failed (non-fatal): {e:#}"); Ok(Default::default()) }
            }
        },
        async {
            Ok::<_, anyhow::Error>(match client.get_delegate(address).await {
                Ok(v) => v,
                Err(e) => { tracing::debug!(error = %e, "get_delegate failed (non-fatal)"); None }
            })
        },
    )?;
    let dynamic_map = build_dynamic_map(&dynamic);

    if output == "json" {
        let total_staked: u64 = stakes.iter().map(|s| s.stake.rao()).sum();
        let positions: Vec<serde_json::Value> = stakes
            .iter()
            .map(|s| {
                let di = dynamic_map.get(&s.netuid.0);
                serde_json::json!({
                    "netuid": s.netuid.0,
                    "hotkey": s.hotkey,
                    "stake_rao": s.stake.rao(),
                    "alpha_raw": s.alpha_stake.raw(),
                    "subnet_name": di.map(|d| d.name.clone()).unwrap_or_default(),
                    "price": di.map(|d| d.price).unwrap_or(0.0),
                })
            })
            .collect();
        print_json(&serde_json::json!({
            "address": address,
            "balance_rao": balance.rao(),
            "balance_tao": balance.tao(),
            "total_staked_rao": total_staked,
            "stakes": positions,
            "identity": identity.as_ref().map(|id| serde_json::json!({
                "name": id.name, "url": id.url, "discord": id.discord,
            })),
            "is_delegate": delegate.is_some(),
        }));
        return Ok(());
    }

    println!("Account: {}\n", address);
    let total_staked = stakes.iter().fold(Balance::ZERO, |acc, s| acc + s.stake);
    let total_value = balance + total_staked;
    println!("  Free balance:  {}", balance.display_tao());
    println!("  Total staked:  {}", total_staked.display_tao());
    println!("  Total value:   {}", total_value.display_tao());

    if let Some(id) = &identity {
        println!("\n  Identity:");
        if !id.name.is_empty() {
            println!("    Name:    {}", id.name);
        }
        if !id.url.is_empty() {
            println!("    URL:     {}", id.url);
        }
        if !id.discord.is_empty() {
            println!("    Discord: {}", id.discord);
        }
        if !id.github.is_empty() {
            println!("    GitHub:  {}", id.github);
        }
    }

    if let Some(d) = &delegate {
        println!("\n  Delegate:");
        println!("    Take:        {:.2}%", d.take * 100.0);
        println!("    Nominators:  {}", d.nominators.len());
        println!("    Subnets:     {:?}", d.registrations);
        println!("    VP subnets:  {:?}", d.validator_permits);
    }

    if !stakes.is_empty() {
        // Pair each stake with its dynamic info for render_rows
        let rows: Vec<_> = stakes
            .iter()
            .map(|s| {
                let di = dynamic_map.get(&s.netuid.0);
                (
                    s,
                    di.map(|d| d.name.clone())
                        .unwrap_or_else(|| "?".to_string()),
                    di.map(|d| format!("{:.6}", d.price)).unwrap_or_default(),
                )
            })
            .collect();
        render_rows(
            "table",
            &rows,
            "",
            |_| String::new(),
            &["Subnet", "Name", "Hotkey", "Stake (τ)", "Alpha", "Price"],
            |(s, name, price)| {
                vec![
                    format!("SN{}", s.netuid.0),
                    name.clone(),
                    crate::utils::short_ss58(&s.hotkey),
                    s.stake.display_tao(),
                    format!("{}", s.alpha_stake),
                    price.clone(),
                ]
            },
            Some(&format!("\n  Stake Positions ({}):", stakes.len())),
        );
    } else {
        println!("\n  No active stakes.");
    }

    Ok(())
}

async fn handle_subnet_analytics(client: &Client, netuid: u16, output: &str) -> Result<()> {
    let nuid = NetUid(netuid);
    let (info, dynamic, neurons, hyperparams, subnet_identity) = tokio::try_join!(
        client.get_subnet_info(nuid),
        async {
            Ok::<_, anyhow::Error>(match client.get_dynamic_info(nuid).await {
                Ok(v) => v,
                Err(e) => { tracing::debug!(netuid = nuid.0, error = %e, "get_dynamic_info failed (non-fatal)"); None }
            })
        },
        client.get_neurons_lite(nuid),
        async {
            Ok::<_, anyhow::Error>(match client.get_subnet_hyperparams(nuid).await {
                Ok(v) => v,
                Err(e) => { tracing::debug!(netuid = nuid.0, error = %e, "get_subnet_hyperparams failed (non-fatal)"); None }
            })
        },
        async {
            Ok::<_, anyhow::Error>(match client.get_subnet_identity(nuid).await {
                Ok(v) => v,
                Err(e) => { tracing::debug!(netuid = nuid.0, error = %e, "get_subnet_identity failed (non-fatal)"); None }
            })
        },
    )?;

    let name = dynamic
        .as_ref()
        .map(|d| d.name.clone())
        .or_else(|| subnet_identity.as_ref().map(|i| i.subnet_name.clone()))
        .or_else(|| info.as_ref().map(|i| i.name.clone()))
        .unwrap_or_else(|| format!("SN{}", netuid));

    let n = neurons.len();
    let validators: Vec<_> = neurons.iter().filter(|n| n.validator_permit).collect();
    let miners: Vec<_> = neurons.iter().filter(|n| !n.validator_permit).collect();

    let total_stake: f64 = neurons.iter().map(|n| n.stake.tao()).sum();
    let total_emission: f64 = neurons.iter().map(|n| n.emission).sum();
    let avg_trust: f64 = if n > 0 {
        neurons.iter().map(|n| n.trust).sum::<f64>() / n as f64
    } else {
        0.0
    };
    let avg_incentive: f64 = if !miners.is_empty() {
        miners.iter().map(|n| n.incentive).sum::<f64>() / miners.len() as f64
    } else {
        0.0
    };
    let avg_dividends: f64 = if !validators.is_empty() {
        validators.iter().map(|n| n.dividends).sum::<f64>() / validators.len() as f64
    } else {
        0.0
    };

    let mut top_miners = miners.clone();
    top_miners.sort_by(|a, b| {
        b.incentive
            .partial_cmp(&a.incentive)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut top_vals = validators.clone();
    top_vals.sort_by(|a, b| {
        b.dividends
            .partial_cmp(&a.dividends)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let unique_coldkeys: std::collections::HashSet<&String> =
        neurons.iter().map(|n| &n.coldkey).collect();

    if output == "json" {
        print_json(&serde_json::json!({
            "netuid": netuid,
            "name": name,
            "total_neurons": n,
            "validators": validators.len(),
            "miners": miners.len(),
            "unique_owners": unique_coldkeys.len(),
            "total_stake_tao": total_stake,
            "total_emission": total_emission,
            "avg_trust": avg_trust,
            "avg_miner_incentive": avg_incentive,
            "avg_validator_dividends": avg_dividends,
            "price": dynamic.as_ref().map(|d| d.price).unwrap_or(0.0),
            "tao_in": dynamic.as_ref().map(|d| d.tao_in.tao()).unwrap_or(0.0),
        }));
        return Ok(());
    }

    println!("=== Subnet Analytics: SN{} ({}) ===\n", netuid, name);

    if let Some(si) = &subnet_identity {
        if !si.description.is_empty() {
            println!("  {}\n", si.description);
        }
        if !si.github_repo.is_empty() {
            println!("  GitHub: {}", si.github_repo);
        }
        if !si.discord.is_empty() {
            println!("  Discord: {}", si.discord);
        }
        println!();
    }

    println!("  Neurons:         {}", n);
    println!(
        "  Validators:      {} ({:.0}%)",
        validators.len(),
        if n > 0 {
            validators.len() as f64 / n as f64 * 100.0
        } else {
            0.0
        }
    );
    println!(
        "  Miners:          {} ({:.0}%)",
        miners.len(),
        if n > 0 {
            miners.len() as f64 / n as f64 * 100.0
        } else {
            0.0
        }
    );
    println!("  Unique owners:   {}", unique_coldkeys.len());
    if let Some(ref i) = info {
        println!("  Max neurons:     {}", i.max_n);
        println!(
            "  Capacity:        {:.0}%",
            if i.max_n > 0 {
                n as f64 / i.max_n as f64 * 100.0
            } else {
                0.0
            }
        );
    }

    println!("\n  Economics:");
    println!("    Total stake:         {:.4} τ", total_stake);
    println!("    Total emission/blk:  {:.4} τ", total_emission / 1e9);
    println!("    Avg trust:           {:.4}", avg_trust);
    println!("    Avg miner incentive: {:.4}", avg_incentive);
    println!("    Avg val dividends:   {:.4}", avg_dividends);

    if let Some(ref d) = dynamic {
        println!("    Price:               {:.6} τ/α", d.price);
        println!("    TAO in pool:         {}", d.tao_in.display_tao());
        println!(
            "    Subnet volume:       {:.4} τ",
            d.subnet_volume as f64 / 1e9
        );
    }

    if let Some(ref h) = hyperparams {
        println!("    Tempo:               {} blocks", h.tempo);
        println!(
            "    Commit-reveal:       {}",
            if h.commit_reveal_weights_enabled {
                "enabled"
            } else {
                "disabled"
            }
        );
    }

    if !top_miners.is_empty() {
        let miners_top: Vec<_> = top_miners.into_iter().take(5).collect();
        render_rows(
            "table",
            &miners_top,
            "",
            |_| String::new(),
            &["UID", "Hotkey", "Incentive", "Trust", "Emission"],
            |m| {
                vec![
                    format!("{}", m.uid),
                    crate::utils::short_ss58(&m.hotkey),
                    format!("{:.4}", m.incentive),
                    format!("{:.4}", m.trust),
                    format!("{:.4} τ", m.emission / 1e9),
                ]
            },
            Some("\n  Top Miners (by incentive):"),
        );
    }

    if !top_vals.is_empty() {
        let vals_top: Vec<_> = top_vals.into_iter().take(5).collect();
        render_rows(
            "table",
            &vals_top,
            "",
            |_| String::new(),
            &["UID", "Hotkey", "Stake", "VTrust", "Dividends", "Emission"],
            |v| {
                vec![
                    format!("{}", v.uid),
                    crate::utils::short_ss58(&v.hotkey),
                    format!("{:.4} τ", v.stake.tao()),
                    format!("{:.4}", v.validator_trust),
                    format!("{:.4}", v.dividends),
                    format!("{:.4} τ", v.emission / 1e9),
                ]
            },
            Some("\n  Top Validators (by dividends):"),
        );
    }

    Ok(())
}

async fn handle_staking_analytics(client: &Client, address: &str, output: &str) -> Result<()> {
    let (stakes, dynamic, block_emission) = tokio::try_join!(
        client.get_stake_for_coldkey(address),
        async {
            match client.get_all_dynamic_info().await {
                Ok(d) => Ok::<_, anyhow::Error>(d),
                Err(e) => { tracing::warn!("get_all_dynamic_info failed (non-fatal): {e:#}"); Ok(Default::default()) }
            }
        },
        client.get_block_emission(),
    )?;
    let dynamic_map = build_dynamic_map(&dynamic);

    #[derive(serde::Serialize)]
    struct PositionAnalytics {
        netuid: u16,
        name: String,
        staked_tao: f64,
        price: f64,
        estimated_daily_emission_tao: f64,
        estimated_apy: f64,
    }

    let mut positions: Vec<PositionAnalytics> = Vec::new();

    for s in &stakes {
        let di = dynamic_map.get(&s.netuid.0);
        let staked_tao = s.stake.tao();
        let price = di.map(|d| d.price).unwrap_or(0.0);
        let subnet_emission = di.map(|d| d.total_emission()).unwrap_or(0);
        let tao_in = di.map(|d| d.tao_in.tao()).unwrap_or(0.0);
        let name = di.map(|d| d.name.clone()).unwrap_or_default();

        let share = if tao_in > 0.0 {
            staked_tao / tao_in
        } else {
            0.0
        };
        let emission_per_block_tao = subnet_emission as f64 / 1e9;
        let daily_emission = emission_per_block_tao * 7200.0 * share;
        let apy = if staked_tao > 0.0 {
            daily_emission / staked_tao * 365.0 * 100.0
        } else {
            0.0
        };

        positions.push(PositionAnalytics {
            netuid: s.netuid.0,
            name,
            staked_tao,
            price,
            estimated_daily_emission_tao: daily_emission,
            estimated_apy: apy,
        });
    }

    positions.sort_by(|a, b| {
        b.estimated_apy
            .partial_cmp(&a.estimated_apy)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let total_staked: f64 = positions.iter().map(|p| p.staked_tao).sum();
    let total_daily: f64 = positions
        .iter()
        .map(|p| p.estimated_daily_emission_tao)
        .sum();
    let weighted_apy = if total_staked > 0.0 {
        total_daily / total_staked * 365.0 * 100.0
    } else {
        0.0
    };

    if output == "json" {
        let pos_json: Vec<serde_json::Value> = positions
            .iter()
            .map(|p| {
                serde_json::json!({
                    "netuid": p.netuid,
                    "name": p.name,
                    "staked_tao": p.staked_tao,
                    "price": p.price,
                    "estimated_daily_tao": p.estimated_daily_emission_tao,
                    "estimated_apy_pct": p.estimated_apy,
                })
            })
            .collect();
        print_json(&serde_json::json!({
            "address": address,
            "total_staked_tao": total_staked,
            "total_daily_emission_tao": total_daily,
            "weighted_apy_pct": weighted_apy,
            "block_emission_rao": block_emission.rao(),
            "positions": pos_json,
        }));
        return Ok(());
    }

    println!(
        "=== Staking Analytics for {} ===\n",
        crate::utils::short_ss58(address)
    );
    println!("  Total staked:       {:.4} τ", total_staked);
    println!("  Est. daily yield:   {:.6} τ", total_daily);
    println!("  Est. monthly yield: {:.4} τ", total_daily * 30.0);
    println!("  Est. yearly yield:  {:.4} τ", total_daily * 365.0);
    println!("  Weighted APY:       {:.2}%", weighted_apy);
    println!("  Block emission:     {}", block_emission.display_tao());

    if !positions.is_empty() {
        render_rows(
            "table",
            &positions,
            "",
            |_| String::new(),
            &["Subnet", "Name", "Staked (τ)", "Price", "Daily (τ)", "APY"],
            |p| {
                vec![
                    format!("SN{}", p.netuid),
                    p.name.clone(),
                    format!("{:.4}", p.staked_tao),
                    format!("{:.6}", p.price),
                    format!("{:.6}", p.estimated_daily_emission_tao),
                    format!("{:.2}%", p.estimated_apy),
                ]
            },
            Some("\n  Position Breakdown:"),
        );
    }

    println!("\n  Note: APY estimates are based on current emission rates and pool sizes.");
    println!(
        "  Actual returns depend on validator performance, weight setting, and network changes."
    );

    Ok(())
}

async fn handle_swap_sim(
    client: &Client,
    netuid: u16,
    tao: Option<f64>,
    alpha: Option<f64>,
    output: &str,
) -> Result<()> {
    use crate::types::NetUid;
    let price_raw = client.current_alpha_price(NetUid(netuid)).await?;
    let price = price_raw as f64 / 1e9;

    // Determine direction and fetch simulation
    let sim = match (tao, alpha) {
        (Some(tao_amt), _) => {
            let (out, tf, af) = client
                .sim_swap_tao_for_alpha(NetUid(netuid), (tao_amt * 1e9) as u64)
                .await?;
            let out_f = out as f64 / 1e9;
            Some((
                "tao_to_alpha",
                "TAO → Alpha",
                tao_amt,
                "τ",
                out_f,
                "α",
                tf as f64 / 1e9,
                af as f64 / 1e9,
                if out_f > 0.0 { tao_amt / out_f } else { 0.0 },
            ))
        }
        (_, Some(alpha_amt)) => {
            let (out, tf, af) = client
                .sim_swap_alpha_for_tao(NetUid(netuid), (alpha_amt * 1e9) as u64)
                .await?;
            let out_f = out as f64 / 1e9;
            Some((
                "alpha_to_tao",
                "Alpha → TAO",
                alpha_amt,
                "α",
                out_f,
                "τ",
                tf as f64 / 1e9,
                af as f64 / 1e9,
                if alpha_amt > 0.0 {
                    out_f / alpha_amt
                } else {
                    0.0
                },
            ))
        }
        _ => None,
    };

    match sim {
        Some((dir, dir_label, amt_in, sym_in, amt_out, sym_out, tao_fee, alpha_fee, eff_price)) => {
            if output == "json" {
                print_json(&serde_json::json!({
                    "direction": dir, "netuid": netuid,
                    "amount_in": amt_in, "amount_out": amt_out,
                    "tao_fee": tao_fee, "alpha_fee": alpha_fee,
                    "effective_price": eff_price, "current_price": price,
                }));
            } else {
                let slippage = if price > 0.0 {
                    ((eff_price - price) / price).abs() * 100.0
                } else {
                    0.0
                };
                println!("Swap Simulation — SN{}", netuid);
                println!("  Direction:       {}", dir_label);
                println!("  In:              {:.4} {}", amt_in, sym_in);
                println!("  Out:             {:.4} {}", amt_out, sym_out);
                println!("  TAO fee:         {:.6} τ", tao_fee);
                println!("  Alpha fee:       {:.6} α", alpha_fee);
                println!("  Effective price: {:.6} τ/α", eff_price);
                println!("  Current price:   {:.6} τ/α", price);
                println!("  Slippage:        {:.2}%", slippage);
            }
        }
        None => {
            if output == "json" {
                print_json(&serde_json::json!({"netuid": netuid, "current_price": price}));
            } else {
                println!("SN{} current alpha price: {:.6} τ/α", netuid, price);
                println!(
                    "Use --tao <amount> to simulate TAO→Alpha, or --alpha <amount> for Alpha→TAO"
                );
            }
        }
    }
    Ok(())
}

async fn handle_nominations(client: &Client, hotkey: &str, output: &str) -> Result<()> {
    let delegates = client.get_delegated(hotkey).await?;
    if output == "json" {
        print_json_ser(&delegates);
        return Ok(());
    }

    if delegates.is_empty() {
        println!(
            "No delegation info found for {}",
            crate::utils::short_ss58(hotkey)
        );
        return Ok(());
    }

    println!(
        "Nominations for hotkey {}",
        crate::utils::short_ss58(hotkey)
    );
    for d in &delegates {
        println!("\n  Delegate: {}", crate::utils::short_ss58(&d.hotkey));
        println!("    Owner:       {}", crate::utils::short_ss58(&d.owner));
        println!("    Take:        {:.2}%", d.take * 100.0);
        println!("    Total Stake: {}", d.total_stake.display_tao());
        println!("    Nominators:  {}", d.nominators.len());
        if !d.nominators.is_empty() {
            let mut sorted = d.nominators.clone();
            sorted.sort_by(|a, b| b.1.rao().cmp(&a.1.rao()));
            println!("    Top nominators:");
            for (addr, stake) in sorted.iter().take(10) {
                println!(
                    "      {} — {}",
                    crate::utils::short_ss58(addr),
                    stake.display_tao()
                );
            }
        }
    }
    Ok(())
}

/// Full security audit of an account: proxies, delegates, stake exposure, permissions.
pub async fn handle_audit(client: &Client, address: &str, output: &str) -> Result<()> {
    let (balance, stakes, identity, proxies, delegate, dynamic, coldkey_swap) = tokio::try_join!(
        client.get_balance_ss58(address),
        client.get_stake_for_coldkey(address),
        client.get_identity(address),
        client.list_proxies(address),
        async {
            Ok::<_, anyhow::Error>(match client.get_delegate(address).await {
                Ok(v) => v,
                Err(e) => { tracing::debug!(error = %e, "get_delegate failed (non-fatal)"); None }
            })
        },
        async {
            match client.get_all_dynamic_info().await {
                Ok(d) => Ok::<_, anyhow::Error>(d),
                Err(e) => { tracing::debug!(error = %e, "get_all_dynamic_info failed (non-fatal)"); Ok(Default::default()) }
            }
        },
        async {
            Ok::<_, anyhow::Error>(match client.get_coldkey_swap_scheduled(address).await {
                Ok(v) => v,
                Err(e) => { tracing::debug!(error = %e, "get_coldkey_swap_scheduled failed (non-fatal)"); None }
            })
        },
    )?;
    let dynamic_map = build_dynamic_map(&dynamic);

    // Query childkey delegations + pending childkey changes for each stake position (parallel)
    let child_key_futures: Vec<_> = stakes
        .iter()
        .map(|s| {
            let hotkey = s.hotkey.clone();
            let netuid = s.netuid;
            async move {
                let (children, pending) = tokio::join!(
                    async {
                        client
                            .get_child_keys(&hotkey, netuid)
                            .await
                            .unwrap_or_default()
                    },
                    async {
                        client
                            .get_pending_child_keys(&hotkey, netuid)
                            .await
                            .ok()
                            .flatten()
                    },
                );
                (hotkey, netuid.0, children, pending)
            }
        })
        .collect();
    let child_key_results = futures::future::join_all(child_key_futures).await;

    // Compute risk findings
    let mut findings: Vec<serde_json::Value> = Vec::new();

    // Check scheduled coldkey swap
    if let Some((exec_block, new_coldkey)) = &coldkey_swap {
        findings.push(serde_json::json!({
            "category": "coldkey_swap",
            "severity": "high",
            "message": format!("Coldkey swap scheduled! New coldkey: {} at block {}. If unauthorized, cancel immediately.", crate::utils::short_ss58(new_coldkey), exec_block),
        }));
    }

    // Check proxies
    let has_any_proxy = proxies.iter().any(|(_, pt, _)| pt == "Any");
    if !proxies.is_empty() {
        for (delegate_ss58, proxy_type, delay) in &proxies {
            let severity = if proxy_type == "Any" && *delay == 0 {
                "high"
            } else if proxy_type == "Any" {
                "medium"
            } else {
                "info"
            };
            findings.push(serde_json::json!({
                "category": "proxy",
                "severity": severity,
                "message": format!("Proxy: {} type={} delay={}", crate::utils::short_ss58(delegate_ss58), proxy_type, delay),
            }));
        }
    }
    if has_any_proxy {
        findings.push(serde_json::json!({
            "category": "proxy",
            "severity": "high",
            "message": "Account has an 'Any' type proxy — this grants full control to another account.",
        }));
    }

    // Check stake concentration
    let total_staked: f64 = stakes.iter().map(|s| s.stake.tao()).sum();
    let total_value = balance.tao() + total_staked;
    if !stakes.is_empty() {
        let top_stake = stakes.iter().map(|s| s.stake.tao()).fold(0.0_f64, f64::max);
        let concentration = if total_staked > 0.0 {
            top_stake / total_staked * 100.0
        } else {
            0.0
        };
        if concentration > 80.0 && stakes.len() > 1 {
            findings.push(serde_json::json!({
                "category": "stake",
                "severity": "medium",
                "message": format!("Stake concentration: {:.1}% of staked TAO is on a single subnet/hotkey", concentration),
            }));
        }
    }

    // Check low-liquidity exposure
    for s in &stakes {
        if let Some(di) = dynamic_map.get(&s.netuid.0) {
            let tao_in = di.tao_in.tao();
            if tao_in > 0.0 && s.stake.tao() > tao_in * 0.1 {
                findings.push(serde_json::json!({
                    "category": "liquidity",
                    "severity": "medium",
                    "message": format!("SN{} ({}): stake is >{:.0}% of pool depth ({:.2}τ in pool). Large unstake will have high slippage.",
                        s.netuid.0, di.name, s.stake.tao() / tao_in * 100.0, tao_in),
                }));
            }
        }
    }

    // Check childkey delegations
    for (hotkey, netuid, children, pending) in &child_key_results {
        if !children.is_empty() {
            let total_proportion: f64 = children
                .iter()
                .map(|(p, _)| *p as f64 / u64::MAX as f64 * 100.0)
                .sum();
            findings.push(serde_json::json!({
                "category": "childkey",
                "severity": "info",
                "message": format!("SN{}: hotkey {} has {} childkey delegation(s) ({:.1}% delegated)",
                    netuid, crate::utils::short_ss58(hotkey), children.len(), total_proportion),
            }));
        }
        if let Some((pending_children, cooldown_block)) = pending {
            let total_pending_pct: f64 = pending_children
                .iter()
                .map(|(p, _)| *p as f64 / u64::MAX as f64 * 100.0)
                .sum();
            findings.push(serde_json::json!({
                "category": "pending_childkey",
                "severity": "medium",
                "message": format!("SN{}: hotkey {} has PENDING childkey change ({} children, {:.1}% delegated, cooldown block {})",
                    netuid, crate::utils::short_ss58(hotkey), pending_children.len(), total_pending_pct, cooldown_block),
            }));
        }
    }

    // Check if delegate with high take
    if let Some(ref d) = delegate {
        if d.take > 0.10 {
            findings.push(serde_json::json!({
                "category": "delegate",
                "severity": "info",
                "message": format!("Delegate take is {:.2}% (high — may deter nominators)", d.take * 100.0),
            }));
        }
    }

    // Check no identity set
    if identity.is_none() {
        findings.push(serde_json::json!({
            "category": "identity",
            "severity": "info",
            "message": "No on-chain identity set. Consider setting one for discoverability.",
        }));
    }

    // Check if balance is very low but has stakes
    if total_staked > 0.0 && balance.tao() < 0.1 {
        findings.push(serde_json::json!({
            "category": "balance",
            "severity": "low",
            "message": format!("Very low free balance ({:.4}τ) with active stakes. May not be able to pay tx fees.", balance.tao()),
        }));
    }

    if output == "json" {
        let positions: Vec<serde_json::Value> = stakes
            .iter()
            .map(|s| {
                let di = dynamic_map.get(&s.netuid.0);
                serde_json::json!({
                    "netuid": s.netuid.0,
                    "hotkey": s.hotkey,
                    "stake_tao": s.stake.tao(),
                    "subnet_name": di.map(|d| d.name.clone()).unwrap_or_default(),
                    "price": di.map(|d| d.price).unwrap_or(0.0),
                    "tao_in_pool": di.map(|d| d.tao_in.tao()).unwrap_or(0.0),
                })
            })
            .collect();
        let proxy_json: Vec<serde_json::Value> = proxies
            .iter()
            .map(|(d, pt, delay)| serde_json::json!({"delegate": d, "proxy_type": pt, "delay": delay}))
            .collect();
        let childkey_json: Vec<serde_json::Value> = child_key_results
            .iter()
            .filter(|(_, _, c, p)| !c.is_empty() || p.is_some())
            .map(|(hk, nuid, children, pending)| {
                let mut obj = serde_json::json!({
                    "hotkey": hk,
                    "netuid": nuid,
                    "children": children.iter().map(|(p, c)| serde_json::json!({
                        "proportion_raw": p,
                        "proportion_pct": *p as f64 / u64::MAX as f64 * 100.0,
                        "child": c,
                    })).collect::<Vec<_>>(),
                });
                if let Some((pending_children, cooldown_block)) = pending {
                    obj["pending"] = serde_json::json!({
                        "children": pending_children.iter().map(|(p, c)| serde_json::json!({
                            "proportion_raw": p,
                            "proportion_pct": *p as f64 / u64::MAX as f64 * 100.0,
                            "child": c,
                        })).collect::<Vec<_>>(),
                        "cooldown_block": cooldown_block,
                    });
                }
                obj
            })
            .collect();
        print_json(&serde_json::json!({
            "address": address,
            "balance_tao": balance.tao(),
            "total_staked_tao": total_staked,
            "total_value_tao": total_value,
            "num_stakes": stakes.len(),
            "num_proxies": proxies.len(),
            "is_delegate": delegate.is_some(),
            "has_identity": identity.is_some(),
            "coldkey_swap_scheduled": coldkey_swap.as_ref().map(|(block, new_ck)| serde_json::json!({
                "execution_block": block,
                "new_coldkey": new_ck,
            })),
            "childkey_delegations": childkey_json,
            "proxies": proxy_json,
            "stakes": positions,
            "findings": findings,
        }));
        return Ok(());
    }

    // Table output
    println!("=== Security Audit: {} ===\n", address);
    println!("  Free balance:  {}", balance.display_tao());
    println!("  Total staked:  {:.4} τ", total_staked);
    println!("  Total value:   {:.4} τ", total_value);
    println!("  Stake positions: {}", stakes.len());
    println!("  Proxies:       {}", proxies.len());
    println!(
        "  Is delegate:   {}",
        if delegate.is_some() { "yes" } else { "no" }
    );
    println!(
        "  Has identity:  {}",
        if identity.is_some() { "yes" } else { "no" }
    );
    if let Some((exec_block, ref new_ck)) = coldkey_swap {
        println!(
            "  CK Swap:       SCHEDULED → {} at block {}",
            crate::utils::short_ss58(new_ck),
            exec_block
        );
    }

    if !proxies.is_empty() {
        render_rows(
            "table",
            &proxies,
            "",
            |_| String::new(),
            &["Delegate", "Type", "Delay"],
            |(d, pt, delay)| {
                vec![
                    crate::utils::short_ss58(d),
                    pt.clone(),
                    format!("{}", delay),
                ]
            },
            Some("\n  Proxy Accounts:"),
        );
    }

    if !stakes.is_empty() {
        // Pair each stake with dynamic info for the table
        let exposure_rows: Vec<_> = stakes
            .iter()
            .map(|s| {
                let di = dynamic_map.get(&s.netuid.0);
                let pct = if total_staked > 0.0 {
                    s.stake.tao() / total_staked * 100.0
                } else {
                    0.0
                };
                (
                    s,
                    di.map(|d| d.name.clone())
                        .unwrap_or_else(|| "?".to_string()),
                    pct,
                    di.map(|d| format!("{:.2}", d.tao_in.tao()))
                        .unwrap_or_else(|| "?".to_string()),
                )
            })
            .collect();
        render_rows(
            "table",
            &exposure_rows,
            "",
            |_| String::new(),
            &[
                "Subnet",
                "Name",
                "Hotkey",
                "Stake (τ)",
                "% of Total",
                "Pool Depth (τ)",
            ],
            |(s, name, pct, depth)| {
                vec![
                    format!("SN{}", s.netuid.0),
                    name.clone(),
                    crate::utils::short_ss58(&s.hotkey),
                    format!("{:.4}", s.stake.tao()),
                    format!("{:.1}%", pct),
                    depth.clone(),
                ]
            },
            Some("\n  Stake Exposure:"),
        );
    }

    if let Some(ref d) = delegate {
        println!("\n  Delegate Info:");
        println!("    Take:        {:.2}%", d.take * 100.0);
        println!("    Nominators:  {}", d.nominators.len());
        println!("    Subnets:     {:?}", d.registrations);
    }

    // Show childkey delegations
    let child_rows: Vec<_> = child_key_results
        .iter()
        .flat_map(|(hk, nuid, children, _)| {
            children.iter().map(move |(proportion, child)| {
                let pct = *proportion as f64 / u64::MAX as f64 * 100.0;
                (nuid, hk.clone(), child.clone(), pct)
            })
        })
        .collect();
    if !child_rows.is_empty() {
        render_rows(
            "table",
            &child_rows,
            "",
            |_| String::new(),
            &["Subnet", "Parent Hotkey", "Child", "Proportion"],
            |(nuid, hk, child, pct)| {
                vec![
                    format!("SN{}", nuid),
                    crate::utils::short_ss58(hk),
                    crate::utils::short_ss58(child),
                    format!("{:.1}%", pct),
                ]
            },
            Some("\n  Childkey Delegations:"),
        );
    }

    // Show pending childkey changes
    let pending_rows: Vec<_> = child_key_results
        .iter()
        .flat_map(|(hk, nuid, _, pending)| {
            pending
                .iter()
                .flat_map(move |(pending_children, cooldown_block)| {
                    pending_children.iter().map(move |(proportion, child)| {
                        let pct = *proportion as f64 / u64::MAX as f64 * 100.0;
                        (nuid, hk.clone(), child.clone(), pct, *cooldown_block)
                    })
                })
        })
        .collect();
    if !pending_rows.is_empty() {
        render_rows(
            "table",
            &pending_rows,
            "",
            |_| String::new(),
            &[
                "Subnet",
                "Parent Hotkey",
                "New Child",
                "Proportion",
                "Cooldown Block",
            ],
            |(nuid, hk, child, pct, cooldown)| {
                vec![
                    format!("SN{}", nuid),
                    crate::utils::short_ss58(hk),
                    crate::utils::short_ss58(child),
                    format!("{:.1}%", pct),
                    format!("{}", cooldown),
                ]
            },
            Some("\n  Pending Childkey Changes:"),
        );
    }

    if !findings.is_empty() {
        println!("\n  Findings:");
        for f in &findings {
            let severity = f["severity"].as_str().unwrap_or("info");
            let marker = match severity {
                "high" => "[!!]",
                "medium" => "[!] ",
                "low" => "[.] ",
                _ => "[i] ",
            };
            println!("    {} {}", marker, f["message"].as_str().unwrap_or(""));
        }
    } else {
        println!("\n  No findings — account looks clean.");
    }

    Ok(())
}
