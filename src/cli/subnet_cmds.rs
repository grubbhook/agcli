//! Subnet command handlers.

use crate::chain::Client;
use crate::cli::helpers::*;
use crate::cli::*;
use crate::types::{Balance, NetUid};
use anyhow::Result;

pub(super) async fn handle_subnet(
    cmd: SubnetCommands,
    client: &Client,
    ctx: &Ctx<'_>,
) -> Result<()> {
    let (wallet_dir, wallet_name, hotkey_name) = (ctx.wallet_dir, ctx.wallet_name, ctx.hotkey_name);
    let (output, password, live_interval) = (ctx.output, ctx.password, ctx.live_interval);
    match cmd {
        SubnetCommands::List { at_block } => {
            let title: Option<String> = if let Some(bn) = at_block {
                let block_hash = client.get_block_hash(bn).await?;
                let (mut subnets, dynamic_result) =
                    tokio::try_join!(client.get_all_subnets_at_block(block_hash), async {
                        Ok::<_, anyhow::Error>(
                            client.get_all_dynamic_info_at_block(block_hash).await,
                        )
                    },)?;
                // Try to enrich names from dynamic info at the same block
                if let Ok(dynamic) = dynamic_result {
                    let name_map: std::collections::HashMap<u16, (String, u64)> = dynamic
                        .iter()
                        .filter(|d| !d.name.is_empty())
                        .map(|d| (d.netuid.0, (d.name.clone(), d.total_emission())))
                        .collect();
                    for s in &mut subnets {
                        if let Some((name, emission)) = name_map.get(&s.netuid.0) {
                            s.name = name.clone();
                            if s.emission_value == 0 {
                                s.emission_value = *emission;
                            }
                        }
                    }
                }
                render_rows(
                    output,
                    &subnets,
                    "netuid,name,n,max_n,tempo,emission,burn_rao,owner",
                    |s| {
                        format!(
                            "{},{},{},{},{},{},{},{}",
                            s.netuid,
                            s.name,
                            s.n,
                            s.max_n,
                            s.tempo,
                            s.emission_value,
                            s.burn.rao(),
                            s.owner
                        )
                    },
                    &[
                        "NetUID", "Name", "N", "Max", "Tempo", "Emission", "Burn", "Owner",
                    ],
                    |s| {
                        vec![
                            format!("{}", s.netuid),
                            s.name.clone(),
                            format!("{}", s.n),
                            format!("{}", s.max_n),
                            format!("{}", s.tempo),
                            format!("{:.4} τ", s.emission_value as f64 / 1e9),
                            s.burn.display_tao(),
                            crate::utils::short_ss58(&s.owner),
                        ]
                    },
                    Some(&format!("Subnets at block {}", bn)),
                );
                return Ok(());
            } else {
                None
            };
            let subnets = crate::queries::subnet::list_subnets(client).await?;
            render_rows(
                output,
                &subnets,
                "netuid,name,n,max_n,tempo,emission,burn_rao,owner",
                |s| {
                    format!(
                        "{},{},{},{},{},{},{},{}",
                        s.netuid,
                        csv_escape(&s.name),
                        s.n,
                        s.max_n,
                        s.tempo,
                        s.emission_value,
                        s.burn.rao(),
                        s.owner
                    )
                },
                &[
                    "NetUID", "Name", "N", "Max", "Tempo", "Emission", "Burn", "Owner",
                ],
                |s| {
                    vec![
                        format!("{}", s.netuid),
                        s.name.clone(),
                        format!("{}", s.n),
                        format!("{}", s.max_n),
                        format!("{}", s.tempo),
                        format!("{:.4} τ", s.emission_value as f64 / 1e9),
                        s.burn.display_tao(),
                        crate::utils::short_ss58(&s.owner),
                    ]
                },
                title.as_deref(),
            );
            Ok(())
        }
        SubnetCommands::Show { netuid, at_block } => {
            let nuid = NetUid(netuid);
            let (info, dynamic) = if let Some(bn) = at_block {
                let bh = client.get_block_hash(bn).await?;
                let subnets = client.get_all_subnets_at_block(bh).await?;
                let si = subnets.into_iter().find(|s| s.netuid == nuid);
                let di = match client.get_dynamic_info_at_block(nuid, bh).await {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::debug!(netuid = nuid.0, block = bn, error = %e, "get_dynamic_info_at_block failed (non-fatal)");
                        None
                    }
                };
                (si, di)
            } else {
                tokio::try_join!(client.get_subnet_info(nuid), async {
                    Ok::<_, anyhow::Error>(match client.get_dynamic_info(nuid).await {
                        Ok(v) => v,
                        Err(e) => {
                            tracing::debug!(netuid = nuid.0, error = %e, "get_dynamic_info failed (non-fatal)");
                            None
                        }
                    })
                })?
            };
            match info {
                Some(mut s) => {
                    if let Some(ref di) = dynamic {
                        if !di.name.is_empty() {
                            s.name = di.name.clone();
                        }
                    }
                    let emission_rao = dynamic
                        .as_ref()
                        .map(|d| d.total_emission())
                        .unwrap_or(s.emission_value);
                    s.emission_value = emission_rao;
                    if output.is_json() {
                        print_json_ser(&s);
                    } else {
                        println!("Subnet {} ({})", s.netuid, s.name);
                        println!("  Symbol:        {}", s.symbol);
                        println!("  Neurons:       {}/{}", s.n, s.max_n);
                        println!("  Tempo:         {}", s.tempo);
                        println!("  Emission:      {:.4} τ/tempo", emission_rao as f64 / 1e9);
                        println!("  Burn:          {}", s.burn.display_tao());
                        println!("  Difficulty:    {}", s.difficulty);
                        println!("  Immunity:      {} blocks", s.immunity_period);
                        println!("  Owner:         {}", s.owner);
                        println!(
                            "  Registration:  {}",
                            if s.registration_allowed {
                                "open"
                            } else {
                                "closed"
                            }
                        );
                        if let Some(ref di) = dynamic {
                            println!("  Price:         {:.6} τ/α", di.price);
                            println!("  TAO in pool:   {}", di.tao_in.display_tao());
                            println!("  Alpha in:      {}", di.alpha_in);
                            println!("  Alpha out:     {}", di.alpha_out);
                            println!("  Volume:        {:.4} τ", di.subnet_volume as f64 / 1e9);
                        }
                    }
                }
                None => anyhow::bail!(
                    "Subnet {} not found.\n  List available subnets: agcli subnet list",
                    netuid
                ),
            }
            Ok(())
        }
        SubnetCommands::Hyperparams { netuid } => {
            let params = client.get_subnet_hyperparams(NetUid(netuid)).await?;
            match params {
                Some(h) => {
                    if output.is_json() {
                        print_json_ser(&h);
                        return Ok(());
                    }
                    let rows: Vec<(String, String)> = vec![
                        ("rho".into(), format!("{}", h.rho)),
                        ("kappa".into(), format!("{}", h.kappa)),
                        ("immunity_period".into(), format!("{}", h.immunity_period)),
                        (
                            "min_allowed_weights".into(),
                            format!("{}", h.min_allowed_weights),
                        ),
                        (
                            "max_weights_limit".into(),
                            format!("{}", h.max_weights_limit),
                        ),
                        ("tempo".into(), format!("{}", h.tempo)),
                        ("min_difficulty".into(), format!("{}", h.min_difficulty)),
                        ("max_difficulty".into(), format!("{}", h.max_difficulty)),
                        ("weights_version".into(), format!("{}", h.weights_version)),
                        (
                            "weights_rate_limit".into(),
                            format!("{}", h.weights_rate_limit),
                        ),
                        (
                            "adjustment_interval".into(),
                            format!("{}", h.adjustment_interval),
                        ),
                        ("activity_cutoff".into(), format!("{}", h.activity_cutoff)),
                        (
                            "registration_allowed".into(),
                            format!("{}", h.registration_allowed),
                        ),
                        (
                            "target_regs_per_interval".into(),
                            format!("{}", h.target_regs_per_interval),
                        ),
                        ("min_burn".into(), h.min_burn.display_tao()),
                        ("max_burn".into(), h.max_burn.display_tao()),
                        ("bonds_moving_avg".into(), format!("{}", h.bonds_moving_avg)),
                        (
                            "max_regs_per_block".into(),
                            format!("{}", h.max_regs_per_block),
                        ),
                        (
                            "serving_rate_limit".into(),
                            format!("{}", h.serving_rate_limit),
                        ),
                        ("max_validators".into(), format!("{}", h.max_validators)),
                        ("adjustment_alpha".into(), format!("{}", h.adjustment_alpha)),
                        ("difficulty".into(), format!("{}", h.difficulty)),
                        (
                            "commit_reveal_weights".into(),
                            format!("{}", h.commit_reveal_weights_enabled),
                        ),
                        (
                            "commit_reveal_interval".into(),
                            format!("{}", h.commit_reveal_weights_interval),
                        ),
                        (
                            "liquid_alpha_enabled".into(),
                            format!("{}", h.liquid_alpha_enabled),
                        ),
                    ];
                    render_rows(
                        output,
                        &rows,
                        "parameter,value",
                        |r| format!("{},{}", csv_escape(&r.0), csv_escape(&r.1)),
                        &["Parameter", "Value"],
                        |r| vec![r.0.clone(), r.1.clone()],
                        Some(&format!("Hyperparameters for SN{}", netuid)),
                    );
                }
                None => println!("Hyperparameters not found for SN{}.", netuid),
            }
            Ok(())
        }
        SubnetCommands::Metagraph {
            netuid,
            uid,
            at_block,
            full,
            save,
        } => {
            // Single-UID lookup (always fetches full info)
            if let Some(target_uid) = uid {
                let neuron = if let Some(bn) = at_block {
                    let bh = client.get_block_hash(bn).await?;
                    client
                        .get_neuron_at_block(NetUid(netuid), target_uid, bh)
                        .await?
                } else {
                    client.get_neuron(NetUid(netuid), target_uid).await?
                };
                match neuron {
                    Some(n) => {
                        if output.is_json() {
                            print_json_ser(&n);
                        } else {
                            println!("Neuron UID {} on SN{}", target_uid, netuid);
                            println!("  Hotkey:      {}", n.hotkey);
                            println!("  Coldkey:     {}", n.coldkey);
                            println!("  Active:      {}", n.active);
                            println!("  Stake:       {}", n.stake.display_tao());
                            println!("  Rank:        {:.6}", n.rank);
                            println!("  Trust:       {:.6}", n.trust);
                            println!("  Consensus:   {:.6}", n.consensus);
                            println!("  Incentive:   {:.6}", n.incentive);
                            println!("  Dividends:   {:.6}", n.dividends);
                            println!("  Emission:    {:.4} τ", n.emission / 1e9);
                            println!("  Val. Trust:  {:.6}", n.validator_trust);
                            println!("  Val. Permit: {}", n.validator_permit);
                            println!("  Last Update: block {}", n.last_update);
                            if let Some(axon) = &n.axon_info {
                                println!(
                                    "  Axon:        {}:{} (v{}, proto {})",
                                    axon.ip, axon.port, axon.version, axon.protocol
                                );
                            }
                            if let Some(prom) = &n.prometheus_info {
                                println!(
                                    "  Prometheus:  {}:{} (v{})",
                                    prom.ip, prom.port, prom.version
                                );
                            }
                        }
                    }
                    None => {
                        if output.is_json() {
                            print_json(
                                &serde_json::json!({"error": format!("UID {} not found on SN{}", target_uid, netuid)}),
                            );
                        } else {
                            println!("UID {} not found on SN{}", target_uid, netuid);
                        }
                    }
                }
                return Ok(());
            }
            // Full metagraph
            if at_block.is_none() {
                if let Some(interval) = live_interval {
                    return crate::live::live_metagraph(client, netuid.into(), interval).await;
                }
            }

            // --full mode: fetch full NeuronInfo (with axon/prometheus) for each neuron
            if full {
                let (neurons_lite, block) = if let Some(bn) = at_block {
                    let bh = client.get_block_hash(bn).await?;
                    let neurons = client.get_neurons_lite_at_block(NetUid(netuid), bh).await?;
                    (neurons, bn as u64)
                } else {
                    tokio::try_join!(
                        client.get_neurons_lite(NetUid(netuid)),
                        client.get_block_number(),
                    )?
                };
                let n_count = neurons_lite.len();

                // Fetch full info for each UID in parallel batches
                tracing::info!("Fetching full info for {} neurons...", n_count);
                let mut full_neurons = Vec::with_capacity(n_count);
                for chunk in neurons_lite.chunks(32) {
                    let futs: Vec<_> = chunk
                        .iter()
                        .map(|n| client.get_neuron(NetUid(netuid), n.uid))
                        .collect();
                    let results = futures::future::join_all(futs).await;
                    for result in results {
                        if let Ok(Some(neuron)) = result {
                            full_neurons.push(neuron);
                        }
                    }
                }

                render_rows(
                    output,
                    &full_neurons,
                    "uid,hotkey,coldkey,stake_rao,rank,trust,consensus,incentive,dividends,emission,validator_permit,last_update,axon_ip,axon_port,axon_version,prometheus_ip,prometheus_port",
                    |n| {
                        let (aip, aport, aver) = n.axon_info.as_ref()
                            .map(|a| (a.ip.as_str(), a.port.to_string(), a.version.to_string()))
                            .unwrap_or(("", String::new(), String::new()));
                        let (pip, pport) = n.prometheus_info.as_ref()
                            .map(|p| (p.ip.as_str(), p.port.to_string()))
                            .unwrap_or(("", String::new()));
                        format!(
                            "{},{},{},{},{:.6},{:.6},{:.6},{:.6},{:.6},{:.0},{},{},{},{},{},{},{}",
                            n.uid, n.hotkey, n.coldkey, n.stake.rao(), n.rank, n.trust,
                            n.consensus, n.incentive, n.dividends, n.emission,
                            n.validator_permit, n.last_update, aip, aport, aver, pip, pport
                        )
                    },
                    &["UID", "Hotkey", "Stake", "Rank", "Trust", "Incentive", "Emission", "Axon", "VP"],
                    |n| {
                        let axon_str = n.axon_info.as_ref()
                            .filter(|a| a.port > 0)
                            .map(|a| format!("{}:{}", a.ip, a.port))
                            .unwrap_or_else(|| "—".to_string());
                        vec![
                            format!("{}", n.uid),
                            crate::utils::short_ss58(&n.hotkey),
                            format!("{:.4}τ", n.stake.tao()),
                            format!("{:.4}", n.rank),
                            format!("{:.4}", n.trust),
                            format!("{:.4}", n.incentive),
                            format!("{:.4} τ", n.emission / 1e9),
                            axon_str,
                            if n.validator_permit { "Y" } else { "" }.to_string(),
                        ]
                    },
                    Some(&format!("Metagraph SN{} (full) — {} neurons, block {}", netuid, full_neurons.len(), block)),
                );
                return Ok(());
            }

            let (neurons, block) = if let Some(bn) = at_block {
                let bh = client.get_block_hash(bn).await?;
                let neurons = client.get_neurons_lite_at_block(NetUid(netuid), bh).await?;
                (neurons, bn as u64)
            } else {
                let (neurons, block) = tokio::try_join!(
                    client.get_neurons_lite(NetUid(netuid)),
                    client.get_block_number(),
                )?;
                (neurons, block)
            };
            let n_count = neurons.len() as u16;

            // --save: cache the metagraph to disk
            if save {
                let mg = crate::types::chain_data::Metagraph {
                    netuid: NetUid(netuid),
                    n: n_count,
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
                    neurons: neurons.clone(),
                };
                let path = crate::queries::cache::save(&mg)?;
                eprintln!("Snapshot saved: {}", path.display());
                tracing::info!(path = %path.display(), "Metagraph snapshot saved");
            }

            render_rows(
                output,
                &neurons,
                "uid,hotkey,coldkey,stake_rao,rank,trust,consensus,incentive,dividends,emission,validator_permit,last_update",
                |n| {
                    format!(
                        "{},{},{},{},{:.6},{:.6},{:.6},{:.6},{:.6},{:.0},{},{}",
                        n.uid, n.hotkey, n.coldkey, n.stake.rao(), n.rank, n.trust,
                        n.consensus, n.incentive, n.dividends, n.emission,
                        n.validator_permit, n.last_update
                    )
                },
                &["UID", "Hotkey", "Stake", "Rank", "Trust", "Incentive", "Emission", "Updated", "VP"],
                |n| {
                    vec![
                        format!("{}", n.uid),
                        crate::utils::short_ss58(&n.hotkey),
                        format!("{:.4}τ", n.stake.tao()),
                        format!("{:.4}", n.rank),
                        format!("{:.4}", n.trust),
                        format!("{:.4}", n.incentive),
                        format!("{:.4} τ", n.emission / 1e9),
                        format!("{}", n.last_update),
                        if n.validator_permit { "Y" } else { "" }.to_string(),
                    ]
                },
                Some(&format!("Metagraph SN{} — {} neurons, block {}", netuid, n_count, block)),
            );
            Ok(())
        }
        SubnetCommands::Register => {
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, None, password)?;
            println!("Registering new subnet...");
            let hash = client.register_network(&pair, &hk).await?;
            println!("Subnet registered. Tx: {}", hash);
            Ok(())
        }
        SubnetCommands::RegisterNeuron { netuid } => {
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, None, password)?;
            println!(
                "Burn-registering on SN{} with hotkey {}",
                netuid,
                crate::utils::short_ss58(&hk)
            );
            let hash = client.burned_register(&pair, NetUid(netuid), &hk).await?;
            println!("Neuron registered. Tx: {}", hash);
            Ok(())
        }
        SubnetCommands::Pow { netuid, threads } => {
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, None, password)?;
            let hotkey_pk = crate::wallet::keypair::from_ss58(&hk)?;
            println!("POW registration on SN{} with {} threads", netuid, threads);
            let (block_number, block_hash) = client.get_block_info_for_pow().await?;
            let difficulty = client.get_difficulty(NetUid(netuid)).await?;
            println!("Difficulty: {}, Block: #{}", difficulty, block_number);

            let attempts_per_thread = 10_000_000u64;
            let mut handles = Vec::new();
            for t in 0..threads {
                let (bh, hk_bytes, diff) = (block_hash, hotkey_pk.0, difficulty);
                let offset = t as u64 * attempts_per_thread;
                handles.push(std::thread::spawn(move || {
                    crate::utils::pow::solve_pow_range(
                        &bh,
                        &hk_bytes,
                        diff,
                        offset,
                        attempts_per_thread,
                    )
                }));
            }
            let result = handles.into_iter().find_map(|h| match h.join() {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!("POW worker thread panicked: {:?}", e);
                    None
                }
            });
            match result {
                Some((nonce, work)) => {
                    println!("POW solved! Nonce: {}", nonce);
                    let hash = client
                        .pow_register(&pair, NetUid(netuid), &hk, block_number, nonce, work)
                        .await?;
                    println!("POW registered. Tx: {}", hash);
                }
                None => println!(
                    "POW not found after {} attempts/thread. Try burn registration.",
                    attempts_per_thread
                ),
            }
            Ok(())
        }
        SubnetCommands::Dissolve { netuid } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            println!("Dissolving subnet SN{} (owner only)", netuid);
            println!("WARNING: This action cannot be undone. The subnet and all its state will be permanently removed.");
            if !is_batch_mode() {
                let proceed = dialoguer::Confirm::new()
                    .with_prompt("Are you sure you want to dissolve this subnet?")
                    .default(false)
                    .interact()?;
                if !proceed {
                    println!("Cancelled.");
                    return Ok(());
                }
            }
            let hash = client
                .dissolve_network(wallet.coldkey()?, NetUid(netuid))
                .await?;
            print_tx_result(output, &hash, "Subnet dissolved.");
            Ok(())
        }
        SubnetCommands::Watch { netuid, interval } => {
            handle_subnet_watch(client, netuid, interval).await
        }
        SubnetCommands::Liquidity { netuid } => {
            handle_subnet_liquidity(client, output, netuid).await
        }
        SubnetCommands::Monitor {
            netuid,
            interval,
            json,
        } => handle_subnet_monitor(client, netuid, interval, json).await,
        SubnetCommands::Health { netuid } => handle_subnet_health(client, netuid, output).await,
        SubnetCommands::Emissions { netuid } => {
            handle_subnet_emissions(client, netuid, output).await
        }
        SubnetCommands::Cost { netuid } => handle_subnet_cost(client, netuid, output).await,
        SubnetCommands::CacheLoad { netuid, block } => {
            let mg = if let Some(b) = block {
                crate::queries::cache::load_block(netuid, b)?
            } else {
                crate::queries::cache::load_latest(netuid)?
            };
            match mg {
                Some(mg) => {
                    if output.is_json() {
                        print_json_ser(&mg);
                    } else {
                        let n_count = mg.neurons.len();
                        render_rows(
                            output,
                            &mg.neurons,
                            "uid,hotkey,coldkey,stake_rao,rank,trust,consensus,incentive,dividends,emission,validator_permit,last_update",
                            |n| {
                                format!(
                                    "{},{},{},{},{:.6},{:.6},{:.6},{:.6},{:.6},{:.0},{},{}",
                                    n.uid, n.hotkey, n.coldkey, n.stake.rao(), n.rank, n.trust,
                                    n.consensus, n.incentive, n.dividends, n.emission,
                                    n.validator_permit, n.last_update
                                )
                            },
                            &["UID", "Hotkey", "Stake", "Rank", "Trust", "Incentive", "Emission", "Updated", "VP"],
                            |n| {
                                vec![
                                    format!("{}", n.uid),
                                    crate::utils::short_ss58(&n.hotkey),
                                    format!("{:.4}τ", n.stake.tao()),
                                    format!("{:.4}", n.rank),
                                    format!("{:.4}", n.trust),
                                    format!("{:.4}", n.incentive),
                                    format!("{:.4} τ", n.emission / 1e9),
                                    format!("{}", n.last_update),
                                    if n.validator_permit { "Y" } else { "" }.to_string(),
                                ]
                            },
                            Some(&format!("Cached Metagraph SN{} — {} neurons, block {} (from disk)", netuid, n_count, mg.block)),
                        );
                    }
                }
                None => {
                    let msg = if let Some(b) = block {
                        format!("No cached snapshot for SN{} at block {}", netuid, b)
                    } else {
                        format!("No cached snapshots for SN{}", netuid)
                    };
                    if output.is_json() {
                        print_json(&serde_json::json!({"error": msg}));
                    } else {
                        println!("{}", msg);
                        println!(
                            "  Tip: run `agcli subnet metagraph --netuid {} --save` to create one",
                            netuid
                        );
                    }
                }
            }
            Ok(())
        }
        SubnetCommands::CacheList { netuid } => {
            let blocks = crate::queries::cache::list_cached_blocks(netuid)?;
            if blocks.is_empty() {
                if output.is_json() {
                    print_json(&serde_json::json!({"netuid": netuid, "snapshots": []}));
                } else {
                    println!("No cached snapshots for SN{}", netuid);
                    println!(
                        "  Tip: run `agcli subnet metagraph --netuid {} --save` to create one",
                        netuid
                    );
                }
            } else {
                if output.is_json() {
                    print_json(&serde_json::json!({"netuid": netuid, "snapshots": blocks}));
                } else {
                    println!(
                        "Cached snapshots for SN{} ({} total):",
                        netuid,
                        blocks.len()
                    );
                    let dir = crate::queries::cache::cache_path(netuid);
                    for b in &blocks {
                        let path = dir.join(format!("block-{}.json", b));
                        let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                        println!("  block {} ({:.1} KB)", b, size as f64 / 1024.0);
                    }
                    println!("  Cache dir: {}", dir.display());
                }
            }
            Ok(())
        }
        SubnetCommands::CacheDiff {
            netuid,
            from_block,
            to_block,
        } => {
            // Load "from" snapshot
            let from_mg = if let Some(fb) = from_block {
                crate::queries::cache::load_block(netuid, fb)?.ok_or_else(|| {
                    anyhow::anyhow!("No cached snapshot for SN{} at block {}", netuid, fb)
                })?
            } else {
                crate::queries::cache::load_latest(netuid)?
                    .ok_or_else(|| anyhow::anyhow!("No cached snapshots for SN{}. Run `agcli subnet metagraph --netuid {} --save` first.", netuid, netuid))?
            };

            // Load "to" snapshot (or fetch live)
            let to_mg = if let Some(tb) = to_block {
                crate::queries::cache::load_block(netuid, tb)?.ok_or_else(|| {
                    anyhow::anyhow!("No cached snapshot for SN{} at block {}", netuid, tb)
                })?
            } else {
                // Fetch live from chain
                crate::queries::fetch_metagraph(client, NetUid(netuid)).await?
            };

            let deltas = crate::queries::cache::diff(&from_mg, &to_mg);
            if output.is_json() {
                print_json_ser(&deltas);
            } else {
                println!(
                    "Diff SN{}: block {} → {} ({} changes)",
                    netuid,
                    from_mg.block,
                    to_mg.block,
                    deltas.len()
                );
                if deltas.is_empty() {
                    println!("  No significant changes detected.");
                } else {
                    for d in &deltas {
                        println!("{}", d);
                    }
                }
            }
            Ok(())
        }
        SubnetCommands::CachePrune { netuid, keep } => {
            let removed = crate::queries::cache::prune(netuid, keep)?;
            if output.is_json() {
                print_json(
                    &serde_json::json!({"netuid": netuid, "removed": removed, "kept": keep}),
                );
            } else {
                println!(
                    "Pruned {} old snapshots for SN{} (kept {})",
                    removed, netuid, keep
                );
            }
            Ok(())
        }
        SubnetCommands::Probe {
            netuid,
            uids,
            timeout_ms,
            concurrency,
        } => handle_subnet_probe(client, netuid, uids, timeout_ms, concurrency, output).await,
        SubnetCommands::Commits { netuid, hotkey } => {
            handle_subnet_commits(client, netuid, hotkey, output).await
        }
        SubnetCommands::SetParam {
            netuid,
            param,
            value,
        } => handle_subnet_set_param(client, netuid, &param, value.as_deref(), ctx).await,
        SubnetCommands::SetSymbol { netuid, symbol } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            println!("Setting symbol for SN{} to \"{}\"", netuid, symbol);
            let hash = client
                .set_subnet_symbol(wallet.coldkey()?, NetUid(netuid), &symbol)
                .await?;
            println!("Symbol set. Tx: {}", hash);
            Ok(())
        }
        SubnetCommands::EmissionSplit { netuid } => {
            match client.get_emission_split(NetUid(netuid)).await? {
                Some(splits) => {
                    let total: u64 = splits.iter().map(|(_, v)| v).sum();
                    println!("Emission split for SN{}:", netuid);
                    for (name, weight) in &splits {
                        let pct = if total > 0 {
                            *weight as f64 / total as f64 * 100.0
                        } else {
                            0.0
                        };
                        println!("  {:<12} {} ({:.1}%)", name, weight, pct);
                    }
                }
                None => {
                    println!(
                        "No emission split configured for SN{} (using default)",
                        netuid
                    );
                }
            }
            Ok(())
        }
        SubnetCommands::Trim { netuid, max_uids } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            println!(
                "Trimming SN{} to max {} UIDs (subnet owner only)",
                netuid, max_uids
            );
            if !crate::cli::helpers::is_batch_mode() {
                let proceed = dialoguer::Confirm::new()
                    .with_prompt("This will deregister neurons above the limit. Proceed?")
                    .default(false)
                    .interact()?;
                if !proceed {
                    println!("Cancelled.");
                    return Ok(());
                }
            }
            let hash = client
                .submit_raw_call(
                    wallet.coldkey()?,
                    "SubtensorModule",
                    "sudo_set_max_allowed_uids",
                    vec![
                        subxt::dynamic::Value::u128(netuid as u128),
                        subxt::dynamic::Value::u128(max_uids as u128),
                    ],
                )
                .await?;
            print_tx_result(
                output,
                &hash,
                &format!("SN{} trimmed to max {} UIDs", netuid, max_uids),
            );
            Ok(())
        }
        SubnetCommands::CheckStart { netuid } => {
            let nuid = NetUid(netuid);
            let is_active = client.is_subnet_active(nuid).await?;
            let hyperparams = client.get_subnet_hyperparams(nuid).await?;
            let neurons = client.get_neurons_lite(nuid).await?;
            let n_neurons = neurons.len();
            if output.is_json() {
                print_json(&serde_json::json!({
                    "netuid": netuid,
                    "active": is_active,
                    "neurons": n_neurons,
                    "min_neurons_to_start": 1,
                    "can_start": !is_active && n_neurons > 0,
                    "tempo": hyperparams.as_ref().map(|h| h.tempo),
                }));
            } else if is_active {
                println!("SN{} is already active (emissions running)", netuid);
            } else if n_neurons == 0 {
                println!(
                    "SN{} has 0 neurons — register at least 1 neuron before starting",
                    netuid
                );
            } else {
                println!(
                    "SN{} is ready to start ({} neurons registered)",
                    netuid, n_neurons
                );
                println!("  Run: agcli subnet start --netuid {}", netuid);
            }
            Ok(())
        }
        SubnetCommands::Start { netuid } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            println!(
                "Starting emission schedule for SN{} (subnet owner only)",
                netuid
            );
            let hash = client
                .submit_raw_call(
                    wallet.coldkey()?,
                    "SubtensorModule",
                    "start_call",
                    vec![subxt::dynamic::Value::u128(netuid as u128)],
                )
                .await?;
            print_tx_result(
                output,
                &hash,
                &format!("SN{} emission schedule started", netuid),
            );
            Ok(())
        }
        SubnetCommands::MechanismCount { netuid } => {
            let nuid = NetUid(netuid);
            let count = client.get_mechanism_count(nuid).await?;
            if output.is_json() {
                print_json(&serde_json::json!({"netuid": netuid, "mechanism_count": count}));
            } else {
                println!("SN{} mechanism count: {}", netuid, count);
            }
            Ok(())
        }
        SubnetCommands::SetMechanismCount { netuid, count } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            println!(
                "Setting SN{} mechanism count to {} (subnet owner only)",
                netuid, count
            );
            let hash = client
                .submit_raw_call(
                    wallet.coldkey()?,
                    "SubtensorModule",
                    "sudo_set_mechanism_count",
                    vec![
                        subxt::dynamic::Value::u128(netuid as u128),
                        subxt::dynamic::Value::u128(count as u128),
                    ],
                )
                .await?;
            print_tx_result(
                output,
                &hash,
                &format!("SN{} mechanism count set to {}", netuid, count),
            );
            Ok(())
        }
        SubnetCommands::SetEmissionSplit { netuid, weights } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            let split: Vec<u16> = weights
                .split(',')
                .map(|s| s.trim().parse::<u16>())
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(|e| {
                    anyhow::anyhow!(
                        "Invalid weight value: {}. Use comma-separated u16 values.",
                        e
                    )
                })?;
            if split.is_empty() {
                anyhow::bail!("At least one emission weight is required.");
            }
            let total: u64 = split.iter().map(|v| *v as u64).sum();
            println!(
                "Setting SN{} emission split: {:?} (total: {})",
                netuid, split, total
            );
            for (i, w) in split.iter().enumerate() {
                let pct = *w as f64 / total as f64 * 100.0;
                println!("  Mechanism {}: {} ({:.1}%)", i, w, pct);
            }
            if !crate::cli::helpers::is_batch_mode() {
                let proceed = dialoguer::Confirm::new()
                    .with_prompt("Proceed?")
                    .default(true)
                    .interact()?;
                if !proceed {
                    println!("Cancelled.");
                    return Ok(());
                }
            }
            let weights_val: Vec<subxt::dynamic::Value> = split
                .iter()
                .map(|w| subxt::dynamic::Value::u128(*w as u128))
                .collect();
            let hash = client
                .submit_raw_call(
                    wallet.coldkey()?,
                    "SubtensorModule",
                    "sudo_set_mechanism_emission_split",
                    vec![
                        subxt::dynamic::Value::u128(netuid as u128),
                        subxt::dynamic::Value::unnamed_composite(weights_val),
                    ],
                )
                .await?;
            print_tx_result(output, &hash, &format!("SN{} emission split set", netuid));
            Ok(())
        }
    }
}

// ──────── Subnet Watch ────────

async fn handle_subnet_watch(client: &Client, netuid: u16, interval: u64) -> Result<()> {
    use std::io::Write;
    let nuid = NetUid(netuid);
    println!(
        "Watching SN{} (Ctrl+C to stop, poll every {}s)\n",
        netuid, interval
    );

    loop {
        let (block, hyperparams, dynamic) = tokio::try_join!(
            client.get_block_number(),
            client.get_subnet_hyperparams(nuid),
            async {
                Ok::<_, anyhow::Error>(match client.get_dynamic_info(nuid).await {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::debug!(netuid = nuid.0, error = %e, "get_dynamic_info failed (non-fatal)");
                        None
                    }
                })
            },
        )?;

        let name = dynamic.as_ref().map(|d| d.name.as_str()).unwrap_or("?");

        match hyperparams {
            Some(h) => {
                let tempo = h.tempo as u64;
                let blocks_into_tempo = block % tempo;
                let blocks_until_tempo = tempo - blocks_into_tempo;
                let secs_until = blocks_until_tempo * 12;

                print!("\x1B[2J\x1B[H"); // clear screen
                println!("=== SN{} ({}) — Block #{} ===\n", netuid, name, block);

                // Tempo countdown
                println!("  Tempo:             {} blocks", tempo);
                println!("  Blocks into tempo: {}/{}", blocks_into_tempo, tempo);
                println!(
                    "  Blocks until next: {} (~{}m {}s)",
                    blocks_until_tempo,
                    secs_until / 60,
                    secs_until % 60
                );

                // Progress bar
                let progress = blocks_into_tempo as f64 / tempo as f64;
                let bar_width = 40;
                let filled = (progress * bar_width as f64) as usize;
                let bar: String = "█".repeat(filled) + &"░".repeat(bar_width - filled);
                println!("  Progress:          [{}] {:.0}%", bar, progress * 100.0);

                // Weights rate limit
                println!(
                    "\n  Weights rate limit: {} blocks (~{}m)",
                    h.weights_rate_limit,
                    h.weights_rate_limit * 12 / 60
                );

                // Commit-reveal status
                if h.commit_reveal_weights_enabled {
                    println!(
                        "  Commit-reveal:     ENABLED (interval={} tempos)",
                        h.commit_reveal_weights_interval
                    );
                } else {
                    println!("  Commit-reveal:     disabled (direct set_weights)");
                }

                // Activity cutoff
                println!("  Activity cutoff:   {} blocks", h.activity_cutoff);
                println!("  Max validators:    {}", h.max_validators);
                println!("  Min allowed wts:   {}", h.min_allowed_weights);

                // Dynamic info
                if let Some(ref d) = dynamic {
                    println!("\n  Price:             {:.6} τ/α", d.price);
                    println!("  TAO in pool:       {}", d.tao_in.display_tao());
                    let emission_tao = d.total_emission() as f64 / 1e9;
                    println!("  Emission/tempo:    {:.4} τ", emission_tao);
                    println!(
                        "  Daily emission:    {:.2} τ",
                        emission_tao * 7200.0 / tempo as f64
                    );
                }

                println!(
                    "\n  Last refresh: {}",
                    chrono::Local::now().format("%H:%M:%S")
                );
            }
            None => {
                println!("Subnet SN{} not found or hyperparams unavailable.", netuid);
                return Ok(());
            }
        }

        std::io::stdout().flush().ok();
        tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
    }
}

// ──────── Subnet Liquidity ────────

async fn handle_subnet_liquidity(
    client: &Client,
    output: OutputFormat,
    netuid: Option<u16>,
) -> Result<()> {
    // Avoid cloning the full Arc<Vec<DynamicInfo>> — keep owned single-subnet or Arc for all
    let single;
    let all;
    let dynamic: &[crate::types::chain_data::DynamicInfo] = match netuid {
        Some(n) => match client.get_dynamic_info(NetUid(n)).await? {
            Some(d) => {
                single = vec![d];
                &single
            }
            None => anyhow::bail!("Subnet SN{} not found", n),
        },
        None => {
            all = client.get_all_dynamic_info().await?;
            &all
        }
    };

    // Common trade sizes for slippage estimation
    let trade_sizes_tao: &[f64] = &[0.1, 1.0, 10.0, 100.0];

    if output.is_json() {
        let mut results = Vec::new();
        for d in dynamic {
            if d.tao_in.rao() == 0 {
                continue;
            }
            let tao_in = d.tao_in.tao();
            let alpha_in_raw = d.alpha_in.raw() as f64 / 1e9;
            let price = d.price;

            let mut slippage_entries = Vec::new();
            for &size in trade_sizes_tao {
                let slippage = estimate_slippage(tao_in, alpha_in_raw, size);
                slippage_entries.push(serde_json::json!({
                    "trade_tao": size,
                    "slippage_pct": slippage,
                }));
            }
            results.push(serde_json::json!({
                "netuid": d.netuid.0,
                "name": d.name,
                "price": price,
                "tao_in": tao_in,
                "alpha_in": alpha_in_raw,
                "liquidity_depth_tao": tao_in * 2.0,
                "slippage_estimates": slippage_entries,
            }));
        }
        print_json_ser(&results);
        return Ok(());
    }

    let mut sorted: Vec<_> = dynamic.iter().filter(|d| d.tao_in.rao() > 0).collect();
    sorted.sort_by(|a, b| b.tao_in.rao().cmp(&a.tao_in.rao()));

    render_rows(
        OutputFormat::Table,
        &sorted,
        "",
        |_| String::new(),
        &[
            "Subnet",
            "Name",
            "Price (τ/α)",
            "TAO Pool",
            "Alpha Pool",
            "0.1τ slip",
            "1τ slip",
            "10τ slip",
            "100τ slip",
        ],
        |d| {
            let tao_in = d.tao_in.tao();
            let alpha_in_raw = d.alpha_in.raw() as f64 / 1e9;
            let slippages: Vec<String> = trade_sizes_tao
                .iter()
                .map(|&size| format_slippage(estimate_slippage(tao_in, alpha_in_raw, size)))
                .collect();
            vec![
                format!("SN{}", d.netuid.0),
                d.name.chars().take(12).collect::<String>(),
                format!("{:.6}", d.price),
                format!("{:.1}τ", tao_in),
                format!("{:.1}", alpha_in_raw),
                slippages[0].clone(),
                slippages[1].clone(),
                slippages[2].clone(),
                slippages[3].clone(),
            ]
        },
        Some("AMM Liquidity Dashboard\n"),
    );
    println!("\nSlippage = price impact from AMM constant-product formula.");
    println!("Higher pool depth = lower slippage. Consider limit orders for large trades on shallow pools.");
    Ok(())
}

/// Estimate slippage % for a constant-product AMM trade of `trade_tao` TAO.
fn estimate_slippage(tao_in_pool: f64, alpha_in_pool: f64, trade_tao: f64) -> f64 {
    if tao_in_pool <= 0.0 || alpha_in_pool <= 0.0 {
        return 0.0;
    }
    // Constant product: k = tao_in * alpha_in
    // After trade: new_tao = tao_in + trade_tao, new_alpha = k / new_tao
    // Alpha received = alpha_in - new_alpha
    let k = tao_in_pool * alpha_in_pool;
    let new_tao = tao_in_pool + trade_tao;
    let new_alpha = k / new_tao;
    let alpha_received = alpha_in_pool - new_alpha;

    // Spot price = tao_in / alpha_in
    let spot_price = tao_in_pool / alpha_in_pool;
    // Ideal alpha = trade_tao / spot_price
    let ideal_alpha = trade_tao / spot_price;

    if ideal_alpha <= 0.0 {
        return 0.0;
    }
    // Slippage % = ((ideal - actual) / ideal) * 100
    ((ideal_alpha - alpha_received) / ideal_alpha * 100.0).max(0.0)
}

fn format_slippage(pct: f64) -> String {
    if pct < 0.01 {
        "<0.01%".to_string()
    } else if pct > 50.0 {
        format!("{:.0}% ⚠", pct)
    } else if pct > 5.0 {
        format!("{:.1}% ⚠", pct)
    } else if pct > 2.0 {
        format!("{:.2}%!", pct)
    } else {
        format!("{:.2}%", pct)
    }
}

// ──────── Subnet Monitor ────────

/// Output an event as JSON or human-readable text.
fn emit_event(json_mode: bool, event: serde_json::Value, human_msg: &str) {
    if json_mode {
        println!("{}", event);
    } else {
        println!("{}", human_msg);
    }
}

async fn handle_subnet_monitor(
    client: &Client,
    netuid: u16,
    interval: u64,
    json_mode: bool,
) -> Result<()> {
    use std::collections::HashMap;
    let nuid = NetUid(netuid);

    if !json_mode {
        eprintln!(
            "Monitoring SN{} (poll every {}s, Ctrl+C to stop)",
            netuid, interval
        );
        eprintln!("Tracking: registrations, deregistrations, emission shifts, stake changes\n");
    }
    tracing::info!(
        netuid = netuid,
        interval_secs = interval,
        "Starting subnet monitor"
    );
    tracing::info!(
        netuid = netuid,
        "Tracking: registrations, deregistrations, emission shifts, stake changes"
    );

    struct NeuronSnapshot {
        hotkey: String,
        coldkey: String,
        incentive: f64,
        emission: f64,
        active: bool,
    }

    let mut prev_map: HashMap<u16, NeuronSnapshot> = HashMap::new();
    let mut prev_uids: std::collections::HashSet<u16> = std::collections::HashSet::new();
    let mut first = true;

    loop {
        let block = client.get_block_number().await?;
        let neurons = client.get_neurons_lite(nuid).await?;
        let mut cur_map: HashMap<u16, NeuronSnapshot> = HashMap::new();
        let mut cur_uids: std::collections::HashSet<u16> = std::collections::HashSet::new();

        for n in &neurons {
            cur_uids.insert(n.uid);
            cur_map.insert(
                n.uid,
                NeuronSnapshot {
                    hotkey: n.hotkey.clone(),
                    coldkey: n.coldkey.clone(),
                    incentive: n.incentive,
                    emission: n.emission,
                    active: n.active,
                },
            );
        }

        if !first {
            for &uid in &cur_uids {
                if !prev_uids.contains(&uid) {
                    let snap = &cur_map[&uid];
                    emit_event(
                        json_mode,
                        serde_json::json!({
                            "event": "registration", "block": block, "netuid": netuid,
                            "uid": uid, "hotkey": snap.hotkey, "coldkey": snap.coldkey,
                        }),
                        &format!(
                            "[{}] NEW UID {} registered — hotkey {} coldkey {}",
                            block,
                            uid,
                            crate::utils::short_ss58(&snap.hotkey),
                            crate::utils::short_ss58(&snap.coldkey)
                        ),
                    );
                }
            }

            for &uid in &prev_uids {
                if !cur_uids.contains(&uid) {
                    let snap = &prev_map[&uid];
                    emit_event(
                        json_mode,
                        serde_json::json!({
                            "event": "deregistration", "block": block, "netuid": netuid,
                            "uid": uid, "hotkey": snap.hotkey,
                        }),
                        &format!(
                            "[{}] UID {} deregistered (was {})",
                            block,
                            uid,
                            crate::utils::short_ss58(&snap.hotkey)
                        ),
                    );
                }
            }

            for &uid in &cur_uids {
                if !prev_uids.contains(&uid) {
                    continue;
                }
                let cur = &cur_map[&uid];
                let prev = &prev_map[&uid];

                if cur.hotkey != prev.hotkey {
                    emit_event(
                        json_mode,
                        serde_json::json!({
                            "event": "hotkey_change", "block": block, "netuid": netuid,
                            "uid": uid, "old_hotkey": prev.hotkey, "new_hotkey": cur.hotkey,
                        }),
                        &format!(
                            "[{}] UID {} hotkey changed: {} → {}",
                            block,
                            uid,
                            crate::utils::short_ss58(&prev.hotkey),
                            crate::utils::short_ss58(&cur.hotkey)
                        ),
                    );
                }

                if prev.emission > 0.0 {
                    let change_pct = ((cur.emission - prev.emission) / prev.emission * 100.0).abs();
                    if change_pct > 20.0 {
                        let dir = if cur.emission > prev.emission {
                            "↑"
                        } else {
                            "↓"
                        };
                        emit_event(
                            json_mode,
                            serde_json::json!({
                                "event": "emission_shift", "block": block, "netuid": netuid,
                                "uid": uid, "hotkey": cur.hotkey,
                                "old_emission": prev.emission, "new_emission": cur.emission,
                                "change_pct": change_pct,
                            }),
                            &format!(
                                "[{}] UID {} emission {}{:.0}% ({:.4}τ → {:.4}τ) — {}",
                                block,
                                uid,
                                dir,
                                change_pct,
                                prev.emission / 1e9,
                                cur.emission / 1e9,
                                crate::utils::short_ss58(&cur.hotkey)
                            ),
                        );
                    }
                }

                let incentive_delta = (cur.incentive - prev.incentive).abs();
                if incentive_delta > 0.05 {
                    let dir = if cur.incentive > prev.incentive {
                        "↑"
                    } else {
                        "↓"
                    };
                    emit_event(
                        json_mode,
                        serde_json::json!({
                            "event": "incentive_shift", "block": block, "netuid": netuid,
                            "uid": uid, "hotkey": cur.hotkey,
                            "old_incentive": prev.incentive, "new_incentive": cur.incentive,
                        }),
                        &format!(
                            "[{}] UID {} incentive {} {:.4} → {:.4} — {}",
                            block,
                            uid,
                            dir,
                            prev.incentive,
                            cur.incentive,
                            crate::utils::short_ss58(&cur.hotkey)
                        ),
                    );
                }

                if prev.active && !cur.active {
                    emit_event(
                        json_mode,
                        serde_json::json!({
                            "event": "inactive", "block": block, "netuid": netuid,
                            "uid": uid, "hotkey": cur.hotkey,
                        }),
                        &format!(
                            "[{}] UID {} became INACTIVE — {}",
                            block,
                            uid,
                            crate::utils::short_ss58(&cur.hotkey)
                        ),
                    );
                }
            }
        } else if !json_mode {
            println!(
                "[{}] Initial snapshot: {} neurons on SN{}",
                block,
                neurons.len(),
                netuid
            );
        }

        first = false;
        prev_map = cur_map;
        prev_uids = cur_uids;
        tokio::select! {
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(interval)) => {},
            _ = tokio::signal::ctrl_c() => {
                if !json_mode {
                    println!("\nStopping subnet monitor (received Ctrl+C)");
                }
                return Ok(());
            }
        }
    }
}

// ──────── Subnet Health ────────

async fn handle_subnet_health(client: &Client, netuid: u16, output: OutputFormat) -> Result<()> {
    let nuid = NetUid(netuid);
    let (neurons, dynamic, hyperparams, block) = tokio::try_join!(
        client.get_neurons_lite(nuid),
        async { client.get_dynamic_info(nuid).await },
        async { client.get_subnet_hyperparams(nuid).await },
        client.get_block_number(),
    )?;

    let n = neurons.len();
    let active_count = neurons.iter().filter(|n| n.active).count();
    let validators: Vec<_> = neurons.iter().filter(|n| n.validator_permit).collect();
    let miners: Vec<_> = neurons.iter().filter(|n| !n.validator_permit).collect();
    let zero_emission = neurons.iter().filter(|n| n.emission == 0.0).count();
    let stale_count = neurons
        .iter()
        .filter(|n| block.saturating_sub(n.last_update) > 1000)
        .count();

    if output.is_json() {
        let neuron_json: Vec<serde_json::Value> = neurons
            .iter()
            .map(|n| {
                serde_json::json!({
                    "uid": n.uid, "hotkey": n.hotkey, "coldkey": n.coldkey,
                    "active": n.active, "stake_rao": n.stake.rao(),
                    "rank": n.rank, "trust": n.trust, "consensus": n.consensus,
                    "incentive": n.incentive, "dividends": n.dividends,
                    "emission": n.emission, "validator_permit": n.validator_permit,
                    "last_update": n.last_update,
                    "blocks_since_update": block.saturating_sub(n.last_update),
                })
            })
            .collect();
        print_json(&serde_json::json!({
            "netuid": netuid, "block": block, "total_neurons": n,
            "active": active_count, "validators": validators.len(),
            "miners": miners.len(), "zero_emission": zero_emission,
            "stale_neurons": stale_count,
            "price": dynamic.as_ref().map(|d| d.price).unwrap_or(0.0),
            "commit_reveal": hyperparams.as_ref().map(|h| h.commit_reveal_weights_enabled).unwrap_or(false),
            "neurons": neuron_json,
        }));
        return Ok(());
    }

    let name = dynamic.as_ref().map(|d| d.name.as_str()).unwrap_or("?");
    println!("=== SN{} ({}) Health — Block {} ===\n", netuid, name, block);
    println!("  Neurons:       {}/{}", active_count, n);
    println!("  Validators:    {}", validators.len());
    println!("  Miners:        {}", miners.len());
    println!(
        "  Zero emission: {} ({:.0}%)",
        zero_emission,
        if n > 0 {
            zero_emission as f64 / n as f64 * 100.0
        } else {
            0.0
        }
    );
    println!("  Stale (>1000 blocks): {}", stale_count);

    if let Some(ref d) = dynamic {
        println!("  Price:         {:.6} τ/α", d.price);
        println!("  TAO pool:      {}", d.tao_in.display_tao());
    }
    if let Some(ref h) = hyperparams {
        println!("  Tempo:         {} blocks", h.tempo);
        println!(
            "  Commit-reveal: {}",
            if h.commit_reveal_weights_enabled {
                "enabled"
            } else {
                "disabled"
            }
        );
        println!("  Rate limit:    {} blocks", h.weights_rate_limit);
    }

    render_rows(
        OutputFormat::Table,
        &neurons,
        "",
        |_| String::new(),
        &[
            "UID",
            "Hotkey",
            "Active",
            "Stake",
            "Incentive",
            "Emission",
            "Trust",
            "Updated",
            "VP",
        ],
        |n| {
            let staleness = block.saturating_sub(n.last_update);
            let stale_mark = if staleness > 1000 { " !" } else { "" };
            vec![
                format!("{}", n.uid),
                crate::utils::short_ss58(&n.hotkey),
                if n.active { "Y" } else { "N" }.to_string(),
                format!("{:.4}τ", n.stake.tao()),
                format!("{:.4}", n.incentive),
                format!("{:.4} τ", n.emission / 1e9),
                format!("{:.4}", n.trust),
                format!("{}{}", staleness, stale_mark),
                if n.validator_permit { "V" } else { "M" }.to_string(),
            ]
        },
        Some("\n  All Neurons:"),
    );
    Ok(())
}

// ──────── Subnet Emissions ────────

async fn handle_subnet_emissions(client: &Client, netuid: u16, output: OutputFormat) -> Result<()> {
    let nuid = NetUid(netuid);
    let (neurons, dynamic) = tokio::try_join!(client.get_neurons_lite(nuid), async {
        Ok::<_, anyhow::Error>(match client.get_dynamic_info(nuid).await {
            Ok(v) => v,
            Err(e) => {
                tracing::debug!(netuid = nuid.0, error = %e, "get_dynamic_info failed (non-fatal)");
                None
            }
        })
    },)?;

    let total_emission: f64 = neurons.iter().map(|n| n.emission).sum();
    let emission_per_block = dynamic
        .as_ref()
        .map(|d| d.total_emission() as f64 / 1e9)
        .unwrap_or(0.0);
    let tempo = dynamic.as_ref().map(|d| d.tempo as f64).unwrap_or(360.0);
    let daily_emission = emission_per_block * 7200.0;

    // Sort by index to avoid cloning the entire Vec<NeuronInfoLite>
    let mut indices: Vec<usize> = (0..neurons.len()).collect();
    indices.sort_unstable_by(|&a, &b| {
        neurons[b]
            .emission
            .partial_cmp(&neurons[a].emission)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    if output.is_json() {
        let entries: Vec<serde_json::Value> = indices
            .iter()
            .map(|&i| {
                let n = &neurons[i];
                let share = if total_emission > 0.0 {
                    n.emission / total_emission * 100.0
                } else {
                    0.0
                };
                serde_json::json!({
                    "uid": n.uid, "hotkey": n.hotkey,
                    "emission_raw": n.emission,
                    "emission_tao": n.emission / 1e9,
                    "share_pct": share,
                    "is_validator": n.validator_permit,
                })
            })
            .collect();
        print_json(&serde_json::json!({
            "netuid": netuid,
            "total_emission_per_block_tao": emission_per_block,
            "daily_emission_tao": daily_emission,
            "tempo": tempo,
            "neurons": entries,
        }));
        return Ok(());
    }

    let name = dynamic.as_ref().map(|d| d.name.as_str()).unwrap_or("?");
    println!("=== SN{} ({}) Emissions ===\n", netuid, name);
    println!("  Emission/block: {:.6} τ", emission_per_block);
    println!("  Daily emission: {:.2} τ", daily_emission);
    println!("  Tempo:          {:.0} blocks\n", tempo);

    let top: Vec<_> = indices.iter().take(50).map(|&i| &neurons[i]).collect();
    render_rows(
        OutputFormat::Table,
        &top,
        "",
        |_| String::new(),
        &[
            "UID",
            "Hotkey",
            "Role",
            "Emission (τ)",
            "Share %",
            "Daily Est.",
        ],
        |n| {
            let share = if total_emission > 0.0 {
                n.emission / total_emission * 100.0
            } else {
                0.0
            };
            let daily_est = share / 100.0 * daily_emission;
            vec![
                format!("{}", n.uid),
                crate::utils::short_ss58(&n.hotkey),
                if n.validator_permit { "V" } else { "M" }.to_string(),
                format!("{:.6}", n.emission / 1e9),
                format!("{:.2}%", share),
                format!("{:.4} τ", daily_est),
            ]
        },
        None,
    );
    Ok(())
}

// ──────── Subnet Cost ────────

async fn handle_subnet_cost(client: &Client, netuid: u16, output: OutputFormat) -> Result<()> {
    let nuid = NetUid(netuid);
    let (info, hyperparams, dynamic) = tokio::try_join!(
        client.get_subnet_info(nuid),
        async {
            Ok::<_, anyhow::Error>(match client.get_subnet_hyperparams(nuid).await {
                Ok(v) => v,
                Err(e) => {
                    tracing::debug!(netuid = nuid.0, error = %e, "get_subnet_hyperparams failed (non-fatal)");
                    None
                }
            })
        },
        async {
            Ok::<_, anyhow::Error>(match client.get_dynamic_info(nuid).await {
                Ok(v) => v,
                Err(e) => {
                    tracing::debug!(netuid = nuid.0, error = %e, "get_dynamic_info failed (non-fatal)");
                    None
                }
            })
        },
    )?;

    let burn = info.as_ref().map(|i| i.burn).unwrap_or(Balance::ZERO);
    let difficulty = info.as_ref().map(|i| i.difficulty).unwrap_or(0);
    let n = info.as_ref().map(|i| i.n).unwrap_or(0);
    let max_n = info.as_ref().map(|i| i.max_n).unwrap_or(0);

    if output.is_json() {
        print_json(&serde_json::json!({
            "netuid": netuid,
            "burn_rao": burn.rao(),
            "burn_tao": burn.tao(),
            "difficulty": difficulty,
            "neurons": n,
            "max_neurons": max_n,
            "registration_allowed": info.as_ref().map(|i| i.registration_allowed).unwrap_or(false),
            "price": dynamic.as_ref().map(|d| d.price).unwrap_or(0.0),
            "min_burn": hyperparams.as_ref().map(|h| h.min_burn.tao()).unwrap_or(0.0),
            "max_burn": hyperparams.as_ref().map(|h| h.max_burn.tao()).unwrap_or(0.0),
        }));
        return Ok(());
    }

    let name = dynamic.as_ref().map(|d| d.name.as_str()).unwrap_or("?");
    let allowed = info
        .as_ref()
        .map(|i| i.registration_allowed)
        .unwrap_or(false);
    println!("=== SN{} ({}) Registration Cost ===\n", netuid, name);
    println!(
        "  Registration: {}",
        if allowed { "OPEN" } else { "CLOSED" }
    );
    println!("  Current burn: {}", burn.display_tao());
    println!("  POW difficulty: {}", difficulty);
    println!("  Capacity:     {}/{}", n, max_n);
    if let Some(ref h) = hyperparams {
        println!("  Min burn:     {}", h.min_burn.display_tao());
        println!("  Max burn:     {}", h.max_burn.display_tao());
        println!("  Target regs:  {}/interval", h.target_regs_per_interval);
        println!("  Max regs/blk: {}", h.max_regs_per_block);
        println!("  Immunity:     {} blocks", h.immunity_period);
    }
    if n >= max_n {
        println!("\n  Note: Subnet is at capacity. New registrations will replace the lowest-scoring neuron.");
    }
    Ok(())
}

// ──────── Axon Probe ────────

/// Probe result for a single neuron's axon endpoint.
#[derive(Debug, serde::Serialize)]
struct ProbeResult {
    uid: u16,
    hotkey: String,
    ip: String,
    port: u16,
    status: String,
    latency_ms: Option<f64>,
    version: u32,
}

async fn handle_subnet_probe(
    client: &Client,
    netuid: u16,
    uids_filter: Option<String>,
    timeout_ms: u64,
    concurrency: usize,
    output: OutputFormat,
) -> Result<()> {
    use futures::stream::{self, StreamExt};
    use std::time::Instant;

    // Parse UID filter if provided
    let uid_set: Option<std::collections::HashSet<u16>> = uids_filter.map(|s| {
        s.split(',')
            .filter_map(|u| u.trim().parse::<u16>().ok())
            .collect()
    });

    // Fetch neurons lite first to get UIDs, then fetch full info for axon endpoints
    let neurons_lite = client.get_neurons_lite(NetUid(netuid)).await?;
    let target_uids: Vec<u16> = neurons_lite
        .iter()
        .filter(|n| uid_set.as_ref().is_none_or(|s| s.contains(&n.uid)))
        .map(|n| n.uid)
        .collect();

    if target_uids.is_empty() {
        if output.is_json() {
            print_json(&serde_json::json!({"error": "No neurons to probe", "netuid": netuid}));
        } else {
            println!("No neurons to probe on SN{}", netuid);
        }
        return Ok(());
    }

    // Fetch full neuron info (with axon endpoints) in parallel
    let mut neurons = Vec::new();
    for chunk in target_uids.chunks(32) {
        let futs: Vec<_> = chunk
            .iter()
            .map(|&uid| client.get_neuron(NetUid(netuid), uid))
            .collect();
        let results = futures::future::join_all(futs).await;
        for r in results {
            if let Ok(Some(n)) = r {
                neurons.push(n);
            }
        }
    }

    // Filter to neurons with axon endpoints
    let probeable: Vec<_> = neurons
        .iter()
        .filter(|n| {
            n.axon_info
                .as_ref()
                .is_some_and(|a| a.port > 0 && a.ip != "0.0.0.0")
        })
        .collect();

    if !output.is_json() {
        println!(
            "Probing {} axon endpoints on SN{} ({} total neurons, {}ms timeout, {} concurrent)...\n",
            probeable.len(),
            netuid,
            neurons.len(),
            timeout_ms,
            concurrency,
        );
    }

    let timeout_dur = std::time::Duration::from_millis(timeout_ms);
    let http_client = reqwest::Client::builder().timeout(timeout_dur).build()?;

    // Probe each axon endpoint concurrently
    let results: Vec<ProbeResult> = stream::iter(probeable.iter().filter_map(|n| {
        let axon = n.axon_info.as_ref()?;
        let http = http_client.clone();
        let uid = n.uid;
        let hotkey = n.hotkey.clone();
        let ip = axon.ip.clone();
        let port = axon.port;
        let version = axon.version;

        Some(async move {
            let url = format!("http://{}:{}/", ip, port);
            let start = Instant::now();
            let res = http.get(&url).send().await;
            let elapsed = start.elapsed().as_secs_f64() * 1000.0;

            match res {
                Ok(resp) => ProbeResult {
                    uid,
                    hotkey,
                    ip,
                    port,
                    status: format!("{}", resp.status().as_u16()),
                    latency_ms: Some(elapsed),
                    version,
                },
                Err(e) => {
                    let status = if e.is_timeout() {
                        "timeout".to_string()
                    } else if e.is_connect() {
                        "refused".to_string()
                    } else {
                        format!("error: {}", e)
                    };
                    ProbeResult {
                        uid,
                        hotkey,
                        ip,
                        port,
                        status,
                        latency_ms: None,
                        version,
                    }
                }
            }
        })
    }))
    .buffer_unordered(concurrency)
    .collect()
    .await;

    // Count stats
    let reachable = results.iter().filter(|r| r.latency_ms.is_some()).count();
    let no_axon = neurons.len() - probeable.len();

    render_rows(
        output,
        &results,
        "uid,hotkey,ip,port,status,latency_ms,version",
        |r| {
            format!(
                "{},{},{},{},{},{},{}",
                r.uid,
                r.hotkey,
                r.ip,
                r.port,
                csv_escape(&r.status),
                r.latency_ms
                    .map(|l| format!("{:.1}", l))
                    .unwrap_or_default(),
                r.version
            )
        },
        &["UID", "Hotkey", "Endpoint", "Status", "Latency", "Version"],
        |r| {
            vec![
                format!("{}", r.uid),
                crate::utils::short_ss58(&r.hotkey),
                format!("{}:{}", r.ip, r.port),
                r.status.clone(),
                r.latency_ms
                    .map(|l| format!("{:.0}ms", l))
                    .unwrap_or_else(|| "—".to_string()),
                format!("v{}", r.version),
            ]
        },
        Some(&format!(
            "Axon Probe SN{} — {}/{} reachable, {} no endpoint",
            netuid,
            reachable,
            probeable.len(),
            no_axon
        )),
    );
    Ok(())
}

// ──────── Subnet Commits ────────

async fn handle_subnet_commits(
    client: &Client,
    netuid: u16,
    hotkey: Option<String>,
    output: OutputFormat,
) -> Result<()> {
    let nuid = NetUid(netuid);

    let (block, hyperparams, reveal_period) = tokio::try_join!(
        client.get_block_number(),
        client.get_subnet_hyperparams(nuid),
        client.get_reveal_period_epochs(nuid),
    )?;

    let cr_enabled = hyperparams
        .as_ref()
        .map(|h| h.commit_reveal_weights_enabled)
        .unwrap_or(false);

    if !cr_enabled {
        if output.is_json() {
            print_json(&serde_json::json!({
                "netuid": netuid,
                "commit_reveal_enabled": false,
                "message": "Commit-reveal is not enabled on this subnet"
            }));
        } else {
            println!(
                "SN{}: commit-reveal is not enabled. Validators use direct set_weights.",
                netuid
            );
        }
        return Ok(());
    }

    #[derive(serde::Serialize)]
    struct CommitEntry {
        hotkey: String,
        hash: String,
        commit_block: u64,
        first_reveal: u64,
        last_reveal: u64,
        status: String,
        blocks_until_action: Option<i64>,
    }

    let mut entries = Vec::new();

    if let Some(hk) = &hotkey {
        // Single hotkey query
        if let Some(commits) = client.get_weight_commits(nuid, hk).await? {
            for (hash, commit_block, first_reveal, last_reveal) in commits {
                let (status, blocks_until) = commit_status(block, first_reveal, last_reveal);
                entries.push(CommitEntry {
                    hotkey: hk.clone(),
                    hash: format!("0x{}", hex::encode(hash.0)),
                    commit_block,
                    first_reveal,
                    last_reveal,
                    status,
                    blocks_until_action: blocks_until,
                });
            }
        }
    } else {
        // All hotkeys on subnet
        let all_commits = client.get_all_weight_commits(nuid).await?;
        for (account, commits) in all_commits {
            let hk_ss58 = account.to_string();
            for (hash, commit_block, first_reveal, last_reveal) in commits {
                let (status, blocks_until) = commit_status(block, first_reveal, last_reveal);
                entries.push(CommitEntry {
                    hotkey: hk_ss58.clone(),
                    hash: format!("0x{}", hex::encode(hash.0)),
                    commit_block,
                    first_reveal,
                    last_reveal,
                    status,
                    blocks_until_action: blocks_until,
                });
            }
        }
    }

    // Sort: ready-to-reveal first, then waiting, then expired
    entries.sort_by(|a, b| {
        let order = |s: &str| match s {
            "READY" => 0,
            "WAITING" => 1,
            _ => 2,
        };
        order(&a.status)
            .cmp(&order(&b.status))
            .then(a.first_reveal.cmp(&b.first_reveal))
    });

    if output.is_json() {
        print_json(&serde_json::json!({
            "netuid": netuid,
            "block": block,
            "commit_reveal_enabled": true,
            "reveal_period_epochs": reveal_period,
            "commits": entries,
        }));
    } else {
        println!(
            "Weight Commits — SN{} (block {}, reveal_period={} epochs)\n",
            netuid, block, reveal_period
        );

        if entries.is_empty() {
            println!("  No pending weight commits.");
        } else {
            render_rows(
                output,
                &entries,
                "hotkey,hash,commit_block,first_reveal,last_reveal,status,blocks_until",
                |e| {
                    format!(
                        "{},{},{},{},{},{},{}",
                        e.hotkey,
                        e.hash,
                        e.commit_block,
                        e.first_reveal,
                        e.last_reveal,
                        csv_escape(&e.status),
                        e.blocks_until_action
                            .map(|b| b.to_string())
                            .unwrap_or_default()
                    )
                },
                &[
                    "Hotkey",
                    "Hash",
                    "Committed",
                    "Reveal Window",
                    "Status",
                    "Blocks",
                ],
                |e| {
                    vec![
                        crate::utils::short_ss58(&e.hotkey),
                        if e.hash.len() > 18 {
                            format!("{}..{}", &e.hash[..10], &e.hash[e.hash.len() - 6..])
                        } else {
                            e.hash.clone()
                        },
                        format!("#{}", e.commit_block),
                        format!("{}..{}", e.first_reveal, e.last_reveal),
                        e.status.clone(),
                        e.blocks_until_action
                            .map(|b| format!("{}", b))
                            .unwrap_or_else(|| "—".to_string()),
                    ]
                },
                Some(&format!(
                    "{} commits ({} ready, {} waiting, {} expired)",
                    entries.len(),
                    entries.iter().filter(|e| e.status == "READY").count(),
                    entries.iter().filter(|e| e.status == "WAITING").count(),
                    entries.iter().filter(|e| e.status == "EXPIRED").count(),
                )),
            );
        }
    }

    Ok(())
}

// ──────── Subnet Set Param ────────

/// Value type for hyperparameter setting.
#[derive(Clone, Copy)]
enum ParamType {
    U16,
    U64,
    Bool,
}

/// A hyperparameter that can be set by the subnet owner.
struct ParamDef {
    /// Friendly name (what the user types)
    name: &'static str,
    /// The on-chain extrinsic call name (e.g., "sudo_set_tempo")
    call: &'static str,
    /// Value type
    ty: ParamType,
    /// Short description
    desc: &'static str,
}

/// All supported subnet hyperparameters.
/// These are the `SubtensorModule::sudo_set_*` extrinsics that take `(netuid, value)`.
const SUBNET_PARAMS: &[ParamDef] = &[
    ParamDef {
        name: "tempo",
        call: "sudo_set_tempo",
        ty: ParamType::U16,
        desc: "Blocks per epoch",
    },
    ParamDef {
        name: "rho",
        call: "sudo_set_rho",
        ty: ParamType::U16,
        desc: "Consensus rho parameter",
    },
    ParamDef {
        name: "kappa",
        call: "sudo_set_kappa",
        ty: ParamType::U16,
        desc: "Consensus kappa parameter",
    },
    ParamDef {
        name: "immunity_period",
        call: "sudo_set_immunity_period",
        ty: ParamType::U16,
        desc: "Blocks a new neuron is immune from deregistration",
    },
    ParamDef {
        name: "min_allowed_weights",
        call: "sudo_set_min_allowed_weights",
        ty: ParamType::U16,
        desc: "Minimum weight entries required per set_weights",
    },
    ParamDef {
        name: "max_allowed_uids",
        call: "sudo_set_max_allowed_uids",
        ty: ParamType::U16,
        desc: "Maximum neurons allowed on subnet",
    },
    ParamDef {
        name: "max_allowed_validators",
        call: "sudo_set_max_allowed_validators",
        ty: ParamType::U16,
        desc: "Maximum validator count",
    },
    ParamDef {
        name: "min_difficulty",
        call: "sudo_set_min_difficulty",
        ty: ParamType::U64,
        desc: "Minimum POW difficulty",
    },
    ParamDef {
        name: "max_difficulty",
        call: "sudo_set_max_difficulty",
        ty: ParamType::U64,
        desc: "Maximum POW difficulty",
    },
    ParamDef {
        name: "weights_version",
        call: "sudo_set_weights_version_key",
        ty: ParamType::U64,
        desc: "Expected weights version key",
    },
    ParamDef {
        name: "weights_rate_limit",
        call: "sudo_set_weights_set_rate_limit",
        ty: ParamType::U64,
        desc: "Min blocks between weight sets",
    },
    ParamDef {
        name: "adjustment_interval",
        call: "sudo_set_adjustment_interval",
        ty: ParamType::U16,
        desc: "Blocks between difficulty adjustments",
    },
    ParamDef {
        name: "adjustment_alpha",
        call: "sudo_set_adjustment_alpha",
        ty: ParamType::U64,
        desc: "EMA smoothing for difficulty adjustment",
    },
    ParamDef {
        name: "activity_cutoff",
        call: "sudo_set_activity_cutoff",
        ty: ParamType::U16,
        desc: "Blocks of inactivity before deregistration",
    },
    ParamDef {
        name: "registration_allowed",
        call: "sudo_set_network_registration_allowed",
        ty: ParamType::Bool,
        desc: "Allow new registrations",
    },
    ParamDef {
        name: "pow_registration_allowed",
        call: "sudo_set_network_pow_registration_allowed",
        ty: ParamType::Bool,
        desc: "Allow POW registrations",
    },
    ParamDef {
        name: "target_regs_per_interval",
        call: "sudo_set_target_registrations_per_interval",
        ty: ParamType::U16,
        desc: "Target registrations per adjustment interval",
    },
    ParamDef {
        name: "min_burn",
        call: "sudo_set_min_burn",
        ty: ParamType::U64,
        desc: "Minimum burn cost (RAO)",
    },
    ParamDef {
        name: "max_burn",
        call: "sudo_set_max_burn",
        ty: ParamType::U64,
        desc: "Maximum burn cost (RAO)",
    },
    ParamDef {
        name: "bonds_moving_average",
        call: "sudo_set_bonds_moving_average",
        ty: ParamType::U64,
        desc: "Bonds moving average period",
    },
    ParamDef {
        name: "max_regs_per_block",
        call: "sudo_set_max_registrations_per_block",
        ty: ParamType::U16,
        desc: "Max registrations per block",
    },
    ParamDef {
        name: "serving_rate_limit",
        call: "sudo_set_serving_rate_limit",
        ty: ParamType::U64,
        desc: "Min blocks between serve_axon calls",
    },
    ParamDef {
        name: "difficulty",
        call: "sudo_set_difficulty",
        ty: ParamType::U64,
        desc: "Current POW difficulty",
    },
    ParamDef {
        name: "commit_reveal_weights_enabled",
        call: "sudo_set_commit_reveal_weights_enabled",
        ty: ParamType::Bool,
        desc: "Enable commit-reveal for weights",
    },
    ParamDef {
        name: "commit_reveal_weights_interval",
        call: "sudo_set_commit_reveal_weights_interval",
        ty: ParamType::U64,
        desc: "Blocks between commit-reveal phases",
    },
    ParamDef {
        name: "liquid_alpha_enabled",
        call: "sudo_set_liquid_alpha_enabled",
        ty: ParamType::Bool,
        desc: "Enable liquid alpha (dynamic dividends)",
    },
    ParamDef {
        name: "bonds_penalty",
        call: "sudo_set_bonds_penalty",
        ty: ParamType::U16,
        desc: "Bonds penalty factor",
    },
    ParamDef {
        name: "bonds_reset_enabled",
        call: "sudo_set_bonds_reset_enabled",
        ty: ParamType::Bool,
        desc: "Allow bonds reset",
    },
    ParamDef {
        name: "commit_reveal_version",
        call: "sudo_set_commit_reveal_version",
        ty: ParamType::U64,
        desc: "Commit-reveal protocol version",
    },
    ParamDef {
        name: "yuma",
        call: "sudo_set_yuma",
        ty: ParamType::Bool,
        desc: "Enable Yuma consensus",
    },
    ParamDef {
        name: "min_allowed_uids",
        call: "sudo_set_min_allowed_uids",
        ty: ParamType::U16,
        desc: "Minimum neurons on subnet",
    },
    ParamDef {
        name: "min_non_immune_uids",
        call: "sudo_set_min_non_immune_uids",
        ty: ParamType::U16,
        desc: "Minimum non-immune neurons",
    },
];

/// Extract the current value of a named parameter from hyperparameters.
fn current_param_value(
    h: &crate::types::chain_data::SubnetHyperparameters,
    name: &str,
) -> Option<String> {
    Some(match name {
        "tempo" => h.tempo.to_string(),
        "rho" => h.rho.to_string(),
        "kappa" => h.kappa.to_string(),
        "immunity_period" => h.immunity_period.to_string(),
        "min_allowed_weights" => h.min_allowed_weights.to_string(),
        "max_allowed_uids" => h.max_weights_limit.to_string(),
        "max_allowed_validators" => h.max_validators.to_string(),
        "min_difficulty" => h.min_difficulty.to_string(),
        "max_difficulty" => h.max_difficulty.to_string(),
        "weights_version" => h.weights_version.to_string(),
        "weights_rate_limit" => h.weights_rate_limit.to_string(),
        "adjustment_interval" => h.adjustment_interval.to_string(),
        "adjustment_alpha" => h.adjustment_alpha.to_string(),
        "activity_cutoff" => h.activity_cutoff.to_string(),
        "registration_allowed" => h.registration_allowed.to_string(),
        "pow_registration_allowed" => return None,
        "target_regs_per_interval" => h.target_regs_per_interval.to_string(),
        "min_burn" => h.min_burn.rao().to_string(),
        "max_burn" => h.max_burn.rao().to_string(),
        "bonds_moving_average" => h.bonds_moving_avg.to_string(),
        "max_regs_per_block" => h.max_regs_per_block.to_string(),
        "serving_rate_limit" => h.serving_rate_limit.to_string(),
        "difficulty" => h.difficulty.to_string(),
        "commit_reveal_weights_enabled" => h.commit_reveal_weights_enabled.to_string(),
        "commit_reveal_weights_interval" => h.commit_reveal_weights_interval.to_string(),
        "liquid_alpha_enabled" => h.liquid_alpha_enabled.to_string(),
        "bonds_penalty"
        | "bonds_reset_enabled"
        | "commit_reveal_version"
        | "yuma"
        | "min_allowed_uids"
        | "min_non_immune_uids" => return None,
        _ => return None,
    })
}

async fn handle_subnet_set_param(
    client: &Client,
    netuid: u16,
    param: &str,
    value: Option<&str>,
    ctx: &Ctx<'_>,
) -> Result<()> {
    let output = ctx.output;
    let wallet_dir = ctx.wallet_dir;
    let wallet_name = ctx.wallet_name;
    let password = ctx.password;
    // List mode
    if param == "list" || param == "help" {
        if output.is_json() {
            let params: Vec<serde_json::Value> = SUBNET_PARAMS
                .iter()
                .map(|p| {
                    serde_json::json!({
                        "name": p.name,
                        "type": match p.ty { ParamType::U16 => "u16", ParamType::U64 => "u64", ParamType::Bool => "bool" },
                        "description": p.desc,
                    })
                })
                .collect();
            print_json(&serde_json::json!({"parameters": params}));
        } else {
            println!("Available subnet hyperparameters:\n");
            let mut table = comfy_table::Table::new();
            table.set_header(vec!["Parameter", "Type", "Description"]);
            for p in SUBNET_PARAMS {
                table.add_row(vec![
                    p.name,
                    match p.ty {
                        ParamType::U16 => "u16",
                        ParamType::U64 => "u64",
                        ParamType::Bool => "bool",
                    },
                    p.desc,
                ]);
            }
            println!("{}", table);
            println!("\nUsage: agcli subnet set-param --netuid <N> --param <name> --value <val>");
        }
        return Ok(());
    }

    // Find the parameter definition
    let def = SUBNET_PARAMS.iter().find(|p| p.name == param);
    let def = match def {
        Some(d) => d,
        None => {
            // Suggest closest match
            let available: Vec<&str> = SUBNET_PARAMS.iter().map(|p| p.name).collect();
            let mut suggestions: Vec<(&str, usize)> = available
                .iter()
                .filter_map(|name| {
                    if name.contains(param) || param.contains(name) {
                        Some((*name, 0))
                    } else {
                        None
                    }
                })
                .collect();
            suggestions.truncate(5);
            let hint = if suggestions.is_empty() {
                format!(
                    "Use --param list to see all {} available parameters.",
                    SUBNET_PARAMS.len()
                )
            } else {
                format!(
                    "Did you mean: {}? Use --param list to see all.",
                    suggestions
                        .iter()
                        .map(|(n, _)| *n)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            };
            anyhow::bail!("Unknown parameter '{}'. {}", param, hint);
        }
    };

    // Require --value
    let value_str = match value {
        Some(v) => v,
        None => anyhow::bail!(
            "Missing --value for parameter '{}' (type: {}, {})",
            def.name,
            match def.ty {
                ParamType::U16 => "u16",
                ParamType::U64 => "u64",
                ParamType::Bool => "bool",
            },
            def.desc,
        ),
    };

    // Parse and build the dynamic Value
    use subxt::dynamic::Value;
    let val = match def.ty {
        ParamType::U16 => {
            let v: u16 = value_str.parse().map_err(|_| {
                anyhow::anyhow!(
                    "Invalid u16 value '{}' for parameter '{}' (range: 0-65535)",
                    value_str,
                    def.name
                )
            })?;
            Value::u128(v as u128)
        }
        ParamType::U64 => {
            let v: u64 = value_str.parse().map_err(|_| {
                anyhow::anyhow!(
                    "Invalid u64 value '{}' for parameter '{}'",
                    value_str,
                    def.name
                )
            })?;
            Value::u128(v as u128)
        }
        ParamType::Bool => {
            let v: bool = match value_str {
                "true" | "1" | "yes" | "on" => true,
                "false" | "0" | "no" | "off" => false,
                _ => anyhow::bail!("Invalid bool value '{}' for parameter '{}' (use: true/false, 1/0, yes/no, on/off)", value_str, def.name),
            };
            Value::bool(v)
        }
    };

    // Fetch current value for display
    let current_display = match client.get_subnet_hyperparams(NetUid(netuid)).await {
        Ok(Some(h)) => current_param_value(&h, param)
            .map(|v| format!(" (current: {})", v))
            .unwrap_or_default(),
        _ => String::new(),
    };

    // Confirm
    println!(
        "Setting SN{} {} = {}{} (via {})",
        netuid, def.name, value_str, current_display, def.call
    );

    if !crate::cli::helpers::is_batch_mode() {
        let proceed = dialoguer::Confirm::new()
            .with_prompt("Proceed?")
            .default(true)
            .interact()?;
        if !proceed {
            println!("Cancelled.");
            return Ok(());
        }
    }

    // Unlock wallet
    let mut wallet = open_wallet(wallet_dir, wallet_name)?;
    unlock_coldkey(&mut wallet, password)?;
    let pair = wallet.coldkey()?.clone();

    // Submit
    let hash = client
        .submit_raw_call(
            &pair,
            "SubtensorModule",
            def.call,
            vec![Value::u128(netuid as u128), val],
        )
        .await?;

    print_tx_result(
        output,
        &hash,
        &format!("SN{} {} set to {}", netuid, def.name, value_str),
    );
    Ok(())
}

/// Determine commit status relative to current block.
pub(super) fn commit_status(
    block: u64,
    first_reveal: u64,
    last_reveal: u64,
) -> (String, Option<i64>) {
    if block < first_reveal {
        let remaining = first_reveal - block;
        ("WAITING".to_string(), Some(remaining as i64))
    } else if block <= last_reveal {
        let remaining = last_reveal - block;
        ("READY".to_string(), Some(remaining as i64))
    } else {
        ("EXPIRED".to_string(), None)
    }
}

#[cfg(test)]
mod tests {
    use super::commit_status;

    #[test]
    fn commit_status_waiting() {
        let (status, blocks) = commit_status(100, 200, 300);
        assert_eq!(status, "WAITING");
        assert_eq!(blocks, Some(100));
    }

    #[test]
    fn commit_status_ready() {
        let (status, blocks) = commit_status(250, 200, 300);
        assert_eq!(status, "READY");
        assert_eq!(blocks, Some(50));
    }

    #[test]
    fn commit_status_ready_at_boundary() {
        let (status, blocks) = commit_status(200, 200, 300);
        assert_eq!(status, "READY");
        assert_eq!(blocks, Some(100));
    }

    #[test]
    fn commit_status_expired() {
        let (status, blocks) = commit_status(301, 200, 300);
        assert_eq!(status, "EXPIRED");
        assert_eq!(blocks, None);
    }

    #[test]
    fn subnet_params_no_duplicate_names() {
        use super::SUBNET_PARAMS;
        let mut names: Vec<&str> = SUBNET_PARAMS.iter().map(|p| p.name).collect();
        let count_before = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(
            names.len(),
            count_before,
            "Duplicate param names in SUBNET_PARAMS"
        );
    }

    #[test]
    fn subnet_params_no_duplicate_calls() {
        use super::SUBNET_PARAMS;
        let mut calls: Vec<&str> = SUBNET_PARAMS.iter().map(|p| p.call).collect();
        let count_before = calls.len();
        calls.sort_unstable();
        calls.dedup();
        assert_eq!(
            calls.len(),
            count_before,
            "Duplicate call names in SUBNET_PARAMS"
        );
    }

    #[test]
    fn subnet_params_all_have_sudo_prefix() {
        use super::SUBNET_PARAMS;
        for p in SUBNET_PARAMS {
            assert!(
                p.call.starts_with("sudo_set_"),
                "Param '{}' call '{}' should start with sudo_set_",
                p.name,
                p.call
            );
        }
    }

    #[test]
    fn subnet_params_cover_common_hyperparams() {
        use super::SUBNET_PARAMS;
        let names: Vec<&str> = SUBNET_PARAMS.iter().map(|p| p.name).collect();
        // Verify the most important params for subnet owners are present
        for expected in &[
            "tempo",
            "immunity_period",
            "max_allowed_uids",
            "max_allowed_validators",
            "registration_allowed",
            "min_burn",
            "max_burn",
            "commit_reveal_weights_enabled",
            "liquid_alpha_enabled",
            "weights_rate_limit",
        ] {
            assert!(
                names.contains(expected),
                "Essential param '{}' missing from SUBNET_PARAMS",
                expected
            );
        }
    }
}
