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
        ViewCommands::Portfolio { address } => {
            let addr = resolve_coldkey_address(address, wallet_dir, wallet_name);
            if let Some(interval) = live_interval {
                return crate::live::live_portfolio(client, &addr, interval).await;
            }
            handle_portfolio(client, &addr, output).await
        }
        ViewCommands::Network => handle_network(client, output).await,
        ViewCommands::Dynamic => {
            if let Some(interval) = live_interval {
                return crate::live::live_dynamic(client, interval).await;
            }
            handle_dynamic(client, output).await
        }
        ViewCommands::Neuron { netuid, uid } => handle_neuron(client, netuid, uid).await,
        ViewCommands::Validators { netuid, limit } => {
            handle_validators(client, output, netuid, limit).await
        }
        ViewCommands::History { address, limit } => {
            let addr = resolve_coldkey_address(address, wallet_dir, wallet_name);
            handle_history(&addr, output, limit).await
        }
        ViewCommands::Account { address } => {
            let addr = resolve_coldkey_address(address, wallet_dir, wallet_name);
            handle_account_explorer(client, &addr, output).await
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
    } else if output == "csv" {
        println!("netuid,subnet_name,hotkey,alpha_stake,tao_equiv_rao,price");
        for p in &portfolio.positions {
            println!(
                "{},{},{},{},{},{:.6}",
                p.netuid,
                p.subnet_name,
                p.hotkey_ss58,
                p.alpha_stake,
                p.tao_equivalent.rao(),
                p.price
            );
        }
    } else {
        println!("Portfolio for {}", crate::utils::short_ss58(addr));
        println!("  Free:   {}", portfolio.free_balance.display_tao());
        println!("  Staked: {}", portfolio.total_staked.display_tao());
        println!(
            "  Total:  {}",
            (portfolio.free_balance + portfolio.total_staked).display_tao()
        );
        if !portfolio.positions.is_empty() {
            let mut table = comfy_table::Table::new();
            table.set_header(vec![
                "Subnet",
                "Name",
                "Hotkey",
                "Alpha",
                "TAO Equiv",
                "Price",
            ]);
            for p in &portfolio.positions {
                table.add_row(vec![
                    format!("SN{}", p.netuid),
                    p.subnet_name.clone(),
                    crate::utils::short_ss58(&p.hotkey_ss58),
                    format!("{}", p.alpha_stake),
                    format!("{}", p.tao_equivalent),
                    format!("{:.4}", p.price),
                ]);
            }
            println!("{table}");
        }
    }
    Ok(())
}

async fn handle_network(client: &Client, output: &str) -> Result<()> {
    let block = client.get_block_number().await?;
    let total_stake = client.get_total_stake().await?;
    let total_networks = client.get_total_networks().await?;
    let total_issuance = client.get_total_issuance().await?;
    let emission = client.get_block_emission().await?;
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
    if output == "json" {
        print_json_ser(&dynamic);
    } else if output == "csv" {
        println!("netuid,name,symbol,tempo,price,tao_in_rao,alpha_in,alpha_out,emission,volume");
        for d in &dynamic {
            println!(
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
            );
        }
    } else {
        println!("Dynamic TAO — {} subnets", dynamic.len());
        let mut table = comfy_table::Table::new();
        table.set_header(vec![
            "NetUID",
            "Name",
            "Symbol",
            "Price (τ/α)",
            "TAO In",
            "Alpha In",
            "Alpha Out",
            "Emission",
            "Tempo",
        ]);
        for d in &dynamic {
            table.add_row(vec![
                format!("{}", d.netuid),
                d.name.clone(),
                d.symbol.clone(),
                format!("{:.6}", d.price),
                d.tao_in.display_tao(),
                format!("{}", d.alpha_in),
                format!("{}", d.alpha_out),
                format!("{:.4} τ", d.total_emission() as f64 / 1e9),
                format!("{}", d.tempo),
            ]);
        }
        println!("{table}");
    }
    Ok(())
}

async fn handle_neuron(client: &Client, netuid: u16, uid: u16) -> Result<()> {
    let neuron = client.get_neuron(NetUid(netuid), uid).await?;
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
) -> Result<()> {
    if let Some(nuid) = netuid {
        let neurons = client.get_neurons_lite(NetUid(nuid)).await?;
        let mut validators: Vec<_> = neurons.into_iter().filter(|n| n.validator_permit).collect();
        validators.sort_by(|a, b| b.stake.rao().cmp(&a.stake.rao()));
        validators.truncate(limit);

        if output == "json" {
            print_json_ser(&validators);
        } else if output == "csv" {
            println!("uid,hotkey,coldkey,stake_rao,trust,vtrust,dividends,emission");
            for v in &validators {
                println!(
                    "{},{},{},{},{:.6},{:.6},{:.6},{:.0}",
                    v.uid,
                    v.hotkey,
                    v.coldkey,
                    v.stake.rao(),
                    v.trust,
                    v.validator_trust,
                    v.dividends,
                    v.emission
                );
            }
        } else {
            println!(
                "Validators on SN{} ({} with permits)",
                nuid,
                validators.len()
            );
            let mut table = comfy_table::Table::new();
            table.set_header(vec![
                "UID",
                "Hotkey",
                "Stake",
                "VTrust",
                "Dividends",
                "Emission",
            ]);
            for v in &validators {
                table.add_row(vec![
                    format!("{}", v.uid),
                    crate::utils::short_ss58(&v.hotkey),
                    format!("{:.4}τ", v.stake.tao()),
                    format!("{:.4}", v.validator_trust),
                    format!("{:.4}", v.dividends),
                    format!("{:.4} τ", v.emission / 1e9),
                ]);
            }
            println!("{table}");
        }
    } else {
        let delegates = client.get_delegates().await?;
        let mut sorted = delegates;
        sorted.sort_by(|a, b| b.total_stake.rao().cmp(&a.total_stake.rao()));
        sorted.truncate(limit);

        if output == "json" {
            print_json_ser(&sorted);
        } else if output == "csv" {
            println!("hotkey,owner,take_pct,total_stake_rao,nominators,registrations");
            for d in &sorted {
                println!(
                    "{},{},{:.2},{},{},{:?}",
                    d.hotkey,
                    d.owner,
                    d.take * 100.0,
                    d.total_stake.rao(),
                    d.nominators.len(),
                    d.registrations
                );
            }
        } else {
            println!("Top {} validators by total stake", sorted.len());
            let mut table = comfy_table::Table::new();
            table.set_header(vec![
                "#",
                "Hotkey",
                "Owner",
                "Take",
                "Total Stake",
                "Nominators",
                "Subnets",
            ]);
            for (i, d) in sorted.iter().enumerate() {
                table.add_row(vec![
                    format!("{}", i + 1),
                    crate::utils::short_ss58(&d.hotkey),
                    crate::utils::short_ss58(&d.owner),
                    format!("{:.2}%", d.take * 100.0),
                    d.total_stake.display_tao(),
                    format!("{}", d.nominators.len()),
                    format!("{}", d.registrations.len()),
                ]);
            }
            println!("{table}");
        }
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
            if output == "json" {
                print_json_ser(&txs);
            } else if output == "csv" {
                println!("block,hash,module,call,success,timestamp");
                for tx in txs {
                    println!(
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
                    );
                }
            } else {
                println!("Recent transactions ({}):", txs.len());
                let mut table = comfy_table::Table::new();
                table.set_header(vec!["Block", "Module", "Call", "Success", "Hash"]);
                for tx in txs {
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
                    table.add_row(vec![
                        format!("{}", block),
                        module.to_string(),
                        call.to_string(),
                        if success { "OK" } else { "FAIL" }.to_string(),
                        format!("{}...", hash_short),
                    ]);
                }
                println!("{table}");
            }
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

async fn handle_account_explorer(client: &Client, address: &str, output: &str) -> Result<()> {
    println!("Account: {}\n", address);

    let (balance, stakes, identity) = tokio::try_join!(
        client.get_balance_ss58(address),
        client.get_stake_for_coldkey(address),
        client.get_identity(address),
    )?;

    let dynamic = client.get_all_dynamic_info().await.unwrap_or_default();
    let dynamic_map: std::collections::HashMap<u16, &crate::types::chain_data::DynamicInfo> =
        dynamic.iter().map(|d| (d.netuid.0, d)).collect();
    let delegate = client.get_delegate(address).await.ok().flatten();

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
        println!("\n  Stake Positions ({}):", stakes.len());
        let mut table = comfy_table::Table::new();
        table.set_header(vec![
            "Subnet",
            "Name",
            "Hotkey",
            "Stake (τ)",
            "Alpha",
            "Price",
        ]);
        for s in &stakes {
            let di = dynamic_map.get(&s.netuid.0);
            table.add_row(vec![
                format!("SN{}", s.netuid.0),
                di.map(|d| d.name.clone())
                    .unwrap_or_else(|| "?".to_string()),
                crate::utils::short_ss58(&s.hotkey),
                s.stake.display_tao(),
                format!("{}", s.alpha_stake),
                di.map(|d| format!("{:.6}", d.price)).unwrap_or_default(),
            ]);
        }
        println!("{table}");
    } else {
        println!("\n  No active stakes.");
    }

    Ok(())
}

async fn handle_subnet_analytics(client: &Client, netuid: u16, output: &str) -> Result<()> {
    let nuid = NetUid(netuid);
    let info = client.get_subnet_info(nuid).await?;
    let dynamic = client.get_dynamic_info(nuid).await.ok().flatten();
    let neurons = client.get_neurons_lite(nuid).await?;
    let hyperparams = client.get_subnet_hyperparams(nuid).await.ok().flatten();
    let subnet_identity = client.get_subnet_identity(nuid).await.ok().flatten();

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
        println!("\n  Top Miners (by incentive):");
        let mut table = comfy_table::Table::new();
        table.set_header(vec!["UID", "Hotkey", "Incentive", "Trust", "Emission"]);
        for m in top_miners.iter().take(5) {
            table.add_row(vec![
                format!("{}", m.uid),
                crate::utils::short_ss58(&m.hotkey),
                format!("{:.4}", m.incentive),
                format!("{:.4}", m.trust),
                format!("{:.4} τ", m.emission / 1e9),
            ]);
        }
        println!("{table}");
    }

    if !top_vals.is_empty() {
        println!("\n  Top Validators (by dividends):");
        let mut table = comfy_table::Table::new();
        table.set_header(vec![
            "UID",
            "Hotkey",
            "Stake",
            "VTrust",
            "Dividends",
            "Emission",
        ]);
        for v in top_vals.iter().take(5) {
            table.add_row(vec![
                format!("{}", v.uid),
                crate::utils::short_ss58(&v.hotkey),
                format!("{:.4} τ", v.stake.tao()),
                format!("{:.4}", v.validator_trust),
                format!("{:.4}", v.dividends),
                format!("{:.4} τ", v.emission / 1e9),
            ]);
        }
        println!("{table}");
    }

    Ok(())
}

async fn handle_staking_analytics(client: &Client, address: &str, output: &str) -> Result<()> {
    let stakes = client.get_stake_for_coldkey(address).await?;
    let dynamic = client.get_all_dynamic_info().await.unwrap_or_default();
    let dynamic_map: std::collections::HashMap<u16, &crate::types::chain_data::DynamicInfo> =
        dynamic.iter().map(|d| (d.netuid.0, d)).collect();
    let block_emission = client.get_block_emission().await?;

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
        println!("\n  Position Breakdown:");
        let mut table = comfy_table::Table::new();
        table.set_header(vec![
            "Subnet",
            "Name",
            "Staked (τ)",
            "Price",
            "Daily (τ)",
            "APY",
        ]);
        for p in &positions {
            table.add_row(vec![
                format!("SN{}", p.netuid),
                p.name.clone(),
                format!("{:.4}", p.staked_tao),
                format!("{:.6}", p.price),
                format!("{:.6}", p.estimated_daily_emission_tao),
                format!("{:.2}%", p.estimated_apy),
            ]);
        }
        println!("{table}");
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
