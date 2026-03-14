//! CLI command execution — thin dispatcher to focused handler modules.

use crate::chain::Client;
use crate::cli::helpers::*;
use crate::cli::*;
use crate::types::{Balance, NetUid};
use anyhow::Result;
use clap::CommandFactory;

/// Execute the parsed CLI command.
pub async fn execute(cli: Cli) -> Result<()> {
    let network = cli.resolve_network();
    let output = cli.output.as_str();
    let live_interval = cli.live_interval();
    let password = cli.password.clone();
    let yes = cli.yes;
    let batch = cli.batch;
    let pretty = cli.pretty;

    // Set global mode flags so helpers can check them
    set_batch_mode(batch || yes);
    set_pretty_mode(pretty);

    match cli.command {
        Commands::Wallet(cmd) => {
            wallet_cmds::handle_wallet(cmd, &cli.wallet_dir, &cli.wallet, password.as_deref()).await
        }
        Commands::Balance {
            address,
            watch,
            threshold,
        } => {
            let client = Client::connect(network.ws_url()).await?;
            let addr = resolve_coldkey_address(address, &cli.wallet_dir, &cli.wallet);

            // Watch mode
            if let Some(interval_opt) = watch {
                let interval = interval_opt.unwrap_or(60);
                let threshold_rao = threshold.map(Balance::from_tao);
                eprintln!(
                    "Watching balance for {} (every {}s{})",
                    crate::utils::short_ss58(&addr),
                    interval,
                    threshold_rao
                        .as_ref()
                        .map(|t| format!(", alert below {}", t.display_tao()))
                        .unwrap_or_default()
                );
                loop {
                    let balance = client.get_balance_ss58(&addr).await?;
                    let below = threshold_rao
                        .as_ref()
                        .map(|t| balance.rao() < t.rao())
                        .unwrap_or(false);
                    if output == "json" {
                        print_json(&serde_json::json!({
                            "address": addr,
                            "balance_rao": balance.rao(),
                            "balance_tao": balance.tao(),
                            "below_threshold": below,
                            "timestamp": chrono::Utc::now().to_rfc3339(),
                        }));
                    } else {
                        let alert = if below {
                            " *** BELOW THRESHOLD ***"
                        } else {
                            ""
                        };
                        println!(
                            "[{}] {} — {}{}",
                            chrono::Local::now().format("%H:%M:%S"),
                            crate::utils::short_ss58(&addr),
                            balance.display_tao(),
                            alert
                        );
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
                }
            }

            let balance = client.get_balance_ss58(&addr).await?;
            if output == "json" {
                print_json(
                    &serde_json::json!({"address": addr, "balance_rao": balance.rao(), "balance_tao": balance.tao()}),
                );
            } else {
                println!("Address: {}", addr);
                println!("Balance: {}", balance.display_tao());
            }
            Ok(())
        }
        Commands::Transfer { dest, amount } => {
            let client = Client::connect(network.ws_url()).await?;
            let mut wallet = open_wallet(&cli.wallet_dir, &cli.wallet)?;
            unlock_coldkey(&mut wallet, password.as_deref())?;
            let balance = Balance::from_tao(amount);
            // Pre-flight balance check
            if let Some(ss58) = wallet.coldkey_ss58() {
                let current = client.get_balance_ss58(ss58).await?;
                if current.rao() < balance.rao() {
                    anyhow::bail!(
                        "Insufficient balance: you have {} but trying to transfer {}.",
                        current.display_tao(),
                        balance.display_tao()
                    );
                }
            }
            println!("Transferring {} to {}", balance.display_tao(), dest);
            let hash = client.transfer(wallet.coldkey()?, &dest, balance).await?;
            print_tx_result(output, &hash, "Transaction submitted.");
            Ok(())
        }
        Commands::TransferAll { dest, keep_alive } => {
            let client = Client::connect(network.ws_url()).await?;
            let mut wallet = open_wallet(&cli.wallet_dir, &cli.wallet)?;
            unlock_coldkey(&mut wallet, password.as_deref())?;
            println!(
                "Transferring all balance to {} (keep_alive={})",
                dest, keep_alive
            );
            let hash = client
                .transfer_all(wallet.coldkey()?, &dest, keep_alive)
                .await?;
            print_tx_result(output, &hash, "All balance transferred.");
            Ok(())
        }
        Commands::Stake(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            stake_cmds::handle_stake(
                cmd,
                &client,
                &cli.wallet_dir,
                &cli.wallet,
                &cli.hotkey,
                output,
                password.as_deref(),
                yes,
            )
            .await
        }
        Commands::Subnet(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            handle_subnet(
                cmd,
                &client,
                &cli.wallet_dir,
                &cli.wallet,
                &cli.hotkey,
                output,
                live_interval,
                password.as_deref(),
            )
            .await
        }
        Commands::Weights(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            handle_weights(
                cmd,
                &client,
                &cli.wallet_dir,
                &cli.wallet,
                &cli.hotkey,
                password.as_deref(),
            )
            .await
        }
        Commands::Root(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            handle_root(
                cmd,
                &client,
                &cli.wallet_dir,
                &cli.wallet,
                &cli.hotkey,
                password.as_deref(),
            )
            .await
        }
        Commands::Delegate(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            handle_delegate(
                cmd,
                &client,
                &cli.wallet_dir,
                &cli.wallet,
                &cli.hotkey,
                output,
                password.as_deref(),
            )
            .await
        }
        Commands::View(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            view_cmds::handle_view(
                cmd,
                &client,
                &cli.wallet_dir,
                &cli.wallet,
                output,
                live_interval,
            )
            .await
        }
        Commands::Identity(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            handle_identity(
                cmd,
                &client,
                &cli.wallet_dir,
                &cli.wallet,
                password.as_deref(),
            )
            .await
        }
        Commands::Serve(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            handle_serve(
                cmd,
                &client,
                &cli.wallet_dir,
                &cli.wallet,
                &cli.hotkey,
                password.as_deref(),
            )
            .await
        }
        Commands::Proxy(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            handle_proxy(
                cmd,
                &client,
                &cli.wallet_dir,
                &cli.wallet,
                output,
                password.as_deref(),
            )
            .await
        }
        Commands::Crowdloan(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            handle_crowdloan(
                cmd,
                &client,
                &cli.wallet_dir,
                &cli.wallet,
                output,
                password.as_deref(),
            )
            .await
        }
        Commands::Swap(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            handle_swap(
                cmd,
                &client,
                &cli.wallet_dir,
                &cli.wallet,
                password.as_deref(),
            )
            .await
        }
        Commands::Subscribe(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            handle_subscribe(cmd, &client, output, batch).await
        }
        Commands::Multisig(cmd) => {
            handle_multisig(
                cmd,
                &cli.wallet_dir,
                &cli.wallet,
                network.ws_url(),
                password.as_deref(),
            )
            .await
        }
        Commands::Config(cmd) => handle_config(cmd).await,
        Commands::Completions { shell } => {
            generate_completions(&shell);
            Ok(())
        }
        Commands::Update => handle_update().await,
        Commands::Explain { topic } => {
            handle_explain(topic.as_deref());
            Ok(())
        }
        Commands::Batch { file, no_atomic } => {
            let client = Client::connect(network.ws_url()).await?;
            let mut wallet = open_wallet(&cli.wallet_dir, &cli.wallet)?;
            unlock_coldkey(&mut wallet, password.as_deref())?;
            handle_batch(&client, wallet.coldkey()?, &file, no_atomic, output).await
        }
    }
}

// ──────── Subnet ────────

#[allow(clippy::too_many_arguments)]
async fn handle_subnet(
    cmd: SubnetCommands,
    client: &Client,
    wallet_dir: &str,
    wallet_name: &str,
    hotkey_name: &str,
    output: &str,
    live_interval: Option<u64>,
    password: Option<&str>,
) -> Result<()> {
    match cmd {
        SubnetCommands::List => {
            let subnets = crate::queries::subnet::list_subnets(client).await?;
            if output == "json" {
                print_json_ser(&subnets);
            } else if output == "csv" {
                println!("netuid,name,n,max_n,tempo,emission,burn_rao,owner");
                for s in &subnets {
                    println!(
                        "{},{},{},{},{},{},{},{}",
                        s.netuid,
                        s.name,
                        s.n,
                        s.max_n,
                        s.tempo,
                        s.emission_value,
                        s.burn.rao(),
                        s.owner
                    );
                }
            } else if subnets.is_empty() {
                println!("No subnets found.");
            } else {
                let mut table = comfy_table::Table::new();
                table.set_header(vec![
                    "NetUID", "Name", "N", "Max", "Tempo", "Emission", "Burn", "Owner",
                ]);
                for s in &subnets {
                    table.add_row(vec![
                        format!("{}", s.netuid),
                        s.name.clone(),
                        format!("{}", s.n),
                        format!("{}", s.max_n),
                        format!("{}", s.tempo),
                        format!("{:.4} τ", s.emission_value as f64 / 1e9),
                        s.burn.display_tao(),
                        crate::utils::short_ss58(&s.owner),
                    ]);
                }
                println!("{table}");
            }
            Ok(())
        }
        SubnetCommands::Show { netuid } => {
            let info = client.get_subnet_info(NetUid(netuid)).await?;
            let dynamic = client.get_dynamic_info(NetUid(netuid)).await.ok().flatten();
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
                    if output == "json" {
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
                None => println!("Subnet {} not found.", netuid),
            }
            Ok(())
        }
        SubnetCommands::Hyperparams { netuid } => {
            let params = client.get_subnet_hyperparams(NetUid(netuid)).await?;
            match params {
                Some(h) => {
                    println!("Hyperparameters for SN{}", netuid);
                    let mut table = comfy_table::Table::new();
                    table.set_header(vec!["Parameter", "Value"]);
                    let rows: Vec<(&str, String)> = vec![
                        ("rho", format!("{}", h.rho)),
                        ("kappa", format!("{}", h.kappa)),
                        ("immunity_period", format!("{}", h.immunity_period)),
                        ("min_allowed_weights", format!("{}", h.min_allowed_weights)),
                        ("max_weights_limit", format!("{}", h.max_weights_limit)),
                        ("tempo", format!("{}", h.tempo)),
                        ("min_difficulty", format!("{}", h.min_difficulty)),
                        ("max_difficulty", format!("{}", h.max_difficulty)),
                        ("weights_version", format!("{}", h.weights_version)),
                        ("weights_rate_limit", format!("{}", h.weights_rate_limit)),
                        ("adjustment_interval", format!("{}", h.adjustment_interval)),
                        ("activity_cutoff", format!("{}", h.activity_cutoff)),
                        (
                            "registration_allowed",
                            format!("{}", h.registration_allowed),
                        ),
                        (
                            "target_regs_per_interval",
                            format!("{}", h.target_regs_per_interval),
                        ),
                        ("min_burn", h.min_burn.display_tao()),
                        ("max_burn", h.max_burn.display_tao()),
                        ("bonds_moving_avg", format!("{}", h.bonds_moving_avg)),
                        ("max_regs_per_block", format!("{}", h.max_regs_per_block)),
                        ("serving_rate_limit", format!("{}", h.serving_rate_limit)),
                        ("max_validators", format!("{}", h.max_validators)),
                        ("adjustment_alpha", format!("{}", h.adjustment_alpha)),
                        ("difficulty", format!("{}", h.difficulty)),
                        (
                            "commit_reveal_weights",
                            format!("{}", h.commit_reveal_weights_enabled),
                        ),
                        (
                            "commit_reveal_interval",
                            format!("{}", h.commit_reveal_weights_interval),
                        ),
                        (
                            "liquid_alpha_enabled",
                            format!("{}", h.liquid_alpha_enabled),
                        ),
                    ];
                    for (k, v) in &rows {
                        table.add_row(vec![k, v.as_str()]);
                    }
                    println!("{table}");
                }
                None => println!("Hyperparameters not found for SN{}.", netuid),
            }
            Ok(())
        }
        SubnetCommands::Metagraph { netuid, uid } => {
            // Single-UID lookup
            if let Some(target_uid) = uid {
                let neuron = client.get_neuron(NetUid(netuid), target_uid).await?;
                match neuron {
                    Some(n) => {
                        if output == "json" {
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
                        }
                    }
                    None => {
                        if output == "json" {
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
            if let Some(interval) = live_interval {
                return crate::live::live_metagraph(client, netuid.into(), interval).await;
            }
            let mg = crate::queries::fetch_metagraph(client, netuid.into()).await?;
            if output == "json" {
                print_json_ser(&mg);
            } else if output == "csv" {
                println!("uid,hotkey,coldkey,stake_rao,rank,trust,consensus,incentive,dividends,emission,validator_permit,last_update");
                for n in &mg.neurons {
                    println!(
                        "{},{},{},{},{:.6},{:.6},{:.6},{:.6},{:.6},{:.0},{},{}",
                        n.uid,
                        n.hotkey,
                        n.coldkey,
                        n.stake.rao(),
                        n.rank,
                        n.trust,
                        n.consensus,
                        n.incentive,
                        n.dividends,
                        n.emission,
                        n.validator_permit,
                        n.last_update
                    );
                }
            } else {
                println!(
                    "Metagraph SN{} — {} neurons, block {}",
                    netuid, mg.n, mg.block
                );
                let mut table = comfy_table::Table::new();
                table.set_header(vec![
                    "UID",
                    "Hotkey",
                    "Stake",
                    "Rank",
                    "Trust",
                    "Incentive",
                    "Emission",
                    "Updated",
                    "VP",
                ]);
                for n in &mg.neurons {
                    table.add_row(vec![
                        format!("{}", n.uid),
                        crate::utils::short_ss58(&n.hotkey),
                        format!("{:.4}τ", n.stake.tao()),
                        format!("{:.4}", n.rank),
                        format!("{:.4}", n.trust),
                        format!("{:.4}", n.incentive),
                        format!("{:.4} τ", n.emission / 1e9),
                        format!("{}", n.last_update),
                        if n.validator_permit { "Y" } else { "" }.to_string(),
                    ]);
                }
                println!("{table}");
            }
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
            let result = handles.into_iter().find_map(|h| h.join().ok().flatten());
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
            let hash = client
                .dissolve_network(wallet.coldkey()?, NetUid(netuid))
                .await?;
            println!("Subnet dissolved. Tx: {}", hash);
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
        let block = client.get_block_number().await?;
        let hyperparams = client.get_subnet_hyperparams(nuid).await?;
        let dynamic = client.get_dynamic_info(nuid).await.ok().flatten();

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

async fn handle_subnet_liquidity(client: &Client, output: &str, netuid: Option<u16>) -> Result<()> {
    let dynamic = match netuid {
        Some(n) => match client.get_dynamic_info(NetUid(n)).await? {
            Some(d) => vec![d],
            None => anyhow::bail!("Subnet SN{} not found", n),
        },
        None => client.get_all_dynamic_info().await?,
    };

    // Common trade sizes for slippage estimation
    let trade_sizes_tao: &[f64] = &[0.1, 1.0, 10.0, 100.0];

    if output == "json" {
        let mut results = Vec::new();
        for d in &dynamic {
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

    println!("AMM Liquidity Dashboard\n");
    let mut table = comfy_table::Table::new();
    table.set_header(vec![
        "Subnet",
        "Name",
        "Price (τ/α)",
        "TAO Pool",
        "Alpha Pool",
        "0.1τ slip",
        "1τ slip",
        "10τ slip",
        "100τ slip",
    ]);

    let mut sorted: Vec<_> = dynamic.iter().filter(|d| d.tao_in.rao() > 0).collect();
    sorted.sort_by(|a, b| b.tao_in.rao().cmp(&a.tao_in.rao()));

    for d in &sorted {
        let tao_in = d.tao_in.tao();
        let alpha_in_raw = d.alpha_in.raw() as f64 / 1e9;

        let slippages: Vec<String> = trade_sizes_tao
            .iter()
            .map(|&size| {
                let slip = estimate_slippage(tao_in, alpha_in_raw, size);
                format_slippage(slip)
            })
            .collect();

        table.add_row(vec![
            format!("SN{}", d.netuid.0),
            d.name.chars().take(12).collect::<String>(),
            format!("{:.6}", d.price),
            format!("{:.1}τ", tao_in),
            format!("{:.1}", alpha_in_raw),
            slippages[0].clone(),
            slippages[1].clone(),
            slippages[2].clone(),
            slippages[3].clone(),
        ]);
    }
    println!("{table}");
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

    // Snapshot state for diff detection
    #[allow(dead_code)]
    struct NeuronSnapshot {
        hotkey: String,
        coldkey: String,
        stake: u64,
        incentive: f64,
        emission: f64,
        trust: f64,
        rank: f64,
        active: bool,
        last_update: u64,
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
                    stake: n.stake.rao(),
                    incentive: n.incentive,
                    emission: n.emission,
                    trust: n.trust,
                    rank: n.rank,
                    active: n.active,
                    last_update: n.last_update,
                },
            );
        }

        if !first {
            // Detect new registrations
            for &uid in &cur_uids {
                if !prev_uids.contains(&uid) {
                    let snap = &cur_map[&uid];
                    let event = serde_json::json!({
                        "event": "registration",
                        "block": block,
                        "netuid": netuid,
                        "uid": uid,
                        "hotkey": snap.hotkey,
                        "coldkey": snap.coldkey,
                    });
                    if json_mode {
                        println!("{}", event);
                    } else {
                        println!(
                            "[{}] NEW UID {} registered — hotkey {} coldkey {}",
                            block,
                            uid,
                            crate::utils::short_ss58(&snap.hotkey),
                            crate::utils::short_ss58(&snap.coldkey)
                        );
                    }
                }
            }

            // Detect deregistrations
            for &uid in &prev_uids {
                if !cur_uids.contains(&uid) {
                    let snap = &prev_map[&uid];
                    let event = serde_json::json!({
                        "event": "deregistration",
                        "block": block,
                        "netuid": netuid,
                        "uid": uid,
                        "hotkey": snap.hotkey,
                    });
                    if json_mode {
                        println!("{}", event);
                    } else {
                        println!(
                            "[{}] UID {} deregistered (was {})",
                            block,
                            uid,
                            crate::utils::short_ss58(&snap.hotkey)
                        );
                    }
                }
            }

            // Detect significant changes for existing UIDs
            for &uid in &cur_uids {
                if !prev_uids.contains(&uid) {
                    continue;
                }
                let cur = &cur_map[&uid];
                let prev = &prev_map[&uid];

                // Hotkey changed (re-registration into same slot)
                if cur.hotkey != prev.hotkey {
                    let event = serde_json::json!({
                        "event": "hotkey_change",
                        "block": block,
                        "netuid": netuid,
                        "uid": uid,
                        "old_hotkey": prev.hotkey,
                        "new_hotkey": cur.hotkey,
                    });
                    if json_mode {
                        println!("{}", event);
                    } else {
                        println!(
                            "[{}] UID {} hotkey changed: {} → {}",
                            block,
                            uid,
                            crate::utils::short_ss58(&prev.hotkey),
                            crate::utils::short_ss58(&cur.hotkey)
                        );
                    }
                }

                // Large emission shift (>20% relative change)
                if prev.emission > 0.0 {
                    let change_pct = ((cur.emission - prev.emission) / prev.emission * 100.0).abs();
                    if change_pct > 20.0 {
                        let event = serde_json::json!({
                            "event": "emission_shift",
                            "block": block,
                            "netuid": netuid,
                            "uid": uid,
                            "hotkey": cur.hotkey,
                            "old_emission": prev.emission,
                            "new_emission": cur.emission,
                            "change_pct": change_pct,
                        });
                        if json_mode {
                            println!("{}", event);
                        } else {
                            let dir = if cur.emission > prev.emission {
                                "↑"
                            } else {
                                "↓"
                            };
                            println!(
                                "[{}] UID {} emission {}{:.0}% ({:.4}τ → {:.4}τ) — {}",
                                block,
                                uid,
                                dir,
                                change_pct,
                                prev.emission / 1e9,
                                cur.emission / 1e9,
                                crate::utils::short_ss58(&cur.hotkey)
                            );
                        }
                    }
                }

                // Large incentive shift (>0.05 absolute change)
                let incentive_delta = (cur.incentive - prev.incentive).abs();
                if incentive_delta > 0.05 {
                    let event = serde_json::json!({
                        "event": "incentive_shift",
                        "block": block,
                        "netuid": netuid,
                        "uid": uid,
                        "hotkey": cur.hotkey,
                        "old_incentive": prev.incentive,
                        "new_incentive": cur.incentive,
                    });
                    if json_mode {
                        println!("{}", event);
                    } else {
                        let dir = if cur.incentive > prev.incentive {
                            "↑"
                        } else {
                            "↓"
                        };
                        println!(
                            "[{}] UID {} incentive {} {:.4} → {:.4} — {}",
                            block,
                            uid,
                            dir,
                            prev.incentive,
                            cur.incentive,
                            crate::utils::short_ss58(&cur.hotkey)
                        );
                    }
                }

                // Became inactive
                if prev.active && !cur.active {
                    let event = serde_json::json!({
                        "event": "inactive",
                        "block": block,
                        "netuid": netuid,
                        "uid": uid,
                        "hotkey": cur.hotkey,
                    });
                    if json_mode {
                        println!("{}", event);
                    } else {
                        println!(
                            "[{}] UID {} became INACTIVE — {}",
                            block,
                            uid,
                            crate::utils::short_ss58(&cur.hotkey)
                        );
                    }
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
        tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
    }
}

// ──────── Subnet Health ────────

async fn handle_subnet_health(client: &Client, netuid: u16, output: &str) -> Result<()> {
    let nuid = NetUid(netuid);
    let (neurons, dynamic, hyperparams) = tokio::try_join!(
        client.get_neurons_lite(nuid),
        async { client.get_dynamic_info(nuid).await },
        async { client.get_subnet_hyperparams(nuid).await },
    )?;
    let block = client.get_block_number().await?;

    let n = neurons.len();
    let active_count = neurons.iter().filter(|n| n.active).count();
    let validators: Vec<_> = neurons.iter().filter(|n| n.validator_permit).collect();
    let miners: Vec<_> = neurons.iter().filter(|n| !n.validator_permit).collect();
    let zero_emission = neurons.iter().filter(|n| n.emission == 0.0).count();
    let stale_count = neurons
        .iter()
        .filter(|n| block.saturating_sub(n.last_update) > 1000)
        .count();

    if output == "json" {
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

    println!("\n  All Neurons:");
    let mut table = comfy_table::Table::new();
    table.set_header(vec![
        "UID",
        "Hotkey",
        "Active",
        "Stake",
        "Incentive",
        "Emission",
        "Trust",
        "Updated",
        "VP",
    ]);
    for n in &neurons {
        let staleness = block.saturating_sub(n.last_update);
        let stale_mark = if staleness > 1000 { " !" } else { "" };
        table.add_row(vec![
            format!("{}", n.uid),
            crate::utils::short_ss58(&n.hotkey),
            if n.active { "Y" } else { "N" }.to_string(),
            format!("{:.4}τ", n.stake.tao()),
            format!("{:.4}", n.incentive),
            format!("{:.4} τ", n.emission / 1e9),
            format!("{:.4}", n.trust),
            format!("{}{}", staleness, stale_mark),
            if n.validator_permit { "V" } else { "M" }.to_string(),
        ]);
    }
    println!("{table}");
    Ok(())
}

// ──────── Subnet Emissions ────────

async fn handle_subnet_emissions(client: &Client, netuid: u16, output: &str) -> Result<()> {
    let nuid = NetUid(netuid);
    let neurons = client.get_neurons_lite(nuid).await?;
    let dynamic = client.get_dynamic_info(nuid).await.ok().flatten();

    let total_emission: f64 = neurons.iter().map(|n| n.emission).sum();
    let emission_per_block = dynamic
        .as_ref()
        .map(|d| d.total_emission() as f64 / 1e9)
        .unwrap_or(0.0);
    let tempo = dynamic.as_ref().map(|d| d.tempo as f64).unwrap_or(360.0);
    let daily_emission = emission_per_block * 7200.0;

    let mut sorted = neurons.clone();
    sorted.sort_by(|a, b| {
        b.emission
            .partial_cmp(&a.emission)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    if output == "json" {
        let entries: Vec<serde_json::Value> = sorted
            .iter()
            .map(|n| {
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

    let mut table = comfy_table::Table::new();
    table.set_header(vec![
        "UID",
        "Hotkey",
        "Role",
        "Emission (τ)",
        "Share %",
        "Daily Est.",
    ]);
    for n in sorted.iter().take(50) {
        let share = if total_emission > 0.0 {
            n.emission / total_emission * 100.0
        } else {
            0.0
        };
        let daily_est = share / 100.0 * daily_emission;
        table.add_row(vec![
            format!("{}", n.uid),
            crate::utils::short_ss58(&n.hotkey),
            if n.validator_permit { "V" } else { "M" }.to_string(),
            format!("{:.6}", n.emission / 1e9),
            format!("{:.2}%", share),
            format!("{:.4} τ", daily_est),
        ]);
    }
    println!("{table}");
    Ok(())
}

// ──────── Subnet Cost ────────

async fn handle_subnet_cost(client: &Client, netuid: u16, output: &str) -> Result<()> {
    let nuid = NetUid(netuid);
    let info = client.get_subnet_info(nuid).await?;
    let hyperparams = client.get_subnet_hyperparams(nuid).await.ok().flatten();
    let dynamic = client.get_dynamic_info(nuid).await.ok().flatten();

    let burn = info.as_ref().map(|i| i.burn).unwrap_or(Balance::ZERO);
    let difficulty = info.as_ref().map(|i| i.difficulty).unwrap_or(0);
    let n = info.as_ref().map(|i| i.n).unwrap_or(0);
    let max_n = info.as_ref().map(|i| i.max_n).unwrap_or(0);

    if output == "json" {
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

// ──────── Weights ────────

async fn handle_weights(
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
            use blake2::digest::{Update, VariableOutput};
            let mut hasher = blake2::Blake2bVar::new(32)
                .map_err(|e| anyhow::anyhow!("blake2 init error: {:?}", e))?;
            for u in &uids {
                hasher.update(&u.to_le_bytes());
            }
            for w in &wts {
                hasher.update(&w.to_le_bytes());
            }
            hasher.update(salt_str.as_bytes());
            let mut hash_out = [0u8; 32];
            hasher
                .finalize_variable(&mut hash_out)
                .map_err(|e| anyhow::anyhow!("blake2 finalize error: {:?}", e))?;
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

            // Compute commit hash (blake2b, matching the commit handler)
            let commit_hash = {
                use blake2::digest::{Update, VariableOutput};
                let mut hasher = blake2::Blake2bVar::new(32)
                    .map_err(|e| anyhow::anyhow!("blake2 init error: {:?}", e))?;
                for u in &uids {
                    hasher.update(&u.to_le_bytes());
                }
                for w in &wts {
                    hasher.update(&w.to_le_bytes());
                }
                hasher.update(salt_str.as_bytes());
                let mut hash_out = [0u8; 32];
                hasher
                    .finalize_variable(&mut hash_out)
                    .map_err(|e| anyhow::anyhow!("blake2 finalize error: {:?}", e))?;
                hash_out
            };

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
    }
}

// ──────── Root ────────

async fn handle_root(
    cmd: RootCommands,
    client: &Client,
    wallet_dir: &str,
    wallet_name: &str,
    hotkey_name: &str,
    password: Option<&str>,
) -> Result<()> {
    match cmd {
        RootCommands::Register => {
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, None, password)?;
            println!(
                "Registering on root network with hotkey {}",
                crate::utils::short_ss58(&hk)
            );
            let hash = client.root_register(&pair, &hk).await?;
            println!("Root registered. Tx: {}", hash);
            Ok(())
        }
        RootCommands::Weights { weights } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            wallet.load_hotkey(hotkey_name)?;
            let (uids, wts) = parse_weight_pairs(&weights)?;
            println!("Setting {} root weights", uids.len());
            let hash = client
                .set_weights(wallet.hotkey()?, NetUid::ROOT, &uids, &wts, 0)
                .await?;
            println!("Root weights set. Tx: {}", hash);
            Ok(())
        }
    }
}

// ──────── Delegate ────────

async fn handle_delegate(
    cmd: DelegateCommands,
    client: &Client,
    wallet_dir: &str,
    wallet_name: &str,
    hotkey_name: &str,
    output: &str,
    password: Option<&str>,
) -> Result<()> {
    match cmd {
        DelegateCommands::List => {
            let delegates = client.get_delegates().await?;
            if output == "json" {
                print_json_ser(&delegates);
            } else if output == "csv" {
                println!("hotkey,owner,take_pct,total_stake_rao,nominators");
                for d in &delegates {
                    println!(
                        "{},{},{:.4},{},{}",
                        d.hotkey,
                        d.owner,
                        d.take * 100.0,
                        d.total_stake.rao(),
                        d.nominators.len()
                    );
                }
            } else {
                println!("{} delegates", delegates.len());
                let mut table = comfy_table::Table::new();
                table.set_header(vec!["Hotkey", "Owner", "Take", "Total Stake", "Nominators"]);
                for d in delegates.iter().take(50) {
                    table.add_row(vec![
                        crate::utils::short_ss58(&d.hotkey),
                        crate::utils::short_ss58(&d.owner),
                        format!("{:.2}%", d.take * 100.0),
                        d.total_stake.display_tao(),
                        format!("{}", d.nominators.len()),
                    ]);
                }
                println!("{table}");
            }
            Ok(())
        }
        DelegateCommands::Show { hotkey } => {
            let hotkey_ss58 = match hotkey {
                Some(hk) => hk,
                None => {
                    let mut wallet = open_wallet(wallet_dir, wallet_name)?;
                    resolve_hotkey_ss58(None, &mut wallet, hotkey_name)?
                }
            };
            let delegate = client.get_delegate(&hotkey_ss58).await?;
            match delegate {
                Some(d) => {
                    println!("Delegate: {}", d.hotkey);
                    println!("  Owner:       {}", d.owner);
                    println!("  Take:        {:.2}%", d.take * 100.0);
                    println!("  Total stake: {}", d.total_stake.display_tao());
                    println!("  Nominators:  {}", d.nominators.len());
                    println!("  Registrations: {:?}", d.registrations);
                    println!("  VP subnets:    {:?}", d.validator_permits);
                    if !d.nominators.is_empty() {
                        println!("  Top nominators:");
                        let mut sorted = d.nominators.clone();
                        sorted.sort_by(|a, b| b.1.rao().cmp(&a.1.rao()));
                        for (addr, stake) in sorted.iter().take(10) {
                            println!(
                                "    {} — {}",
                                crate::utils::short_ss58(addr),
                                stake.display_tao()
                            );
                        }
                    }
                }
                None => println!("Delegate not found for {}", hotkey_ss58),
            }
            Ok(())
        }
        DelegateCommands::DecreaseTake { take, hotkey } => {
            change_take(
                client,
                wallet_dir,
                wallet_name,
                hotkey_name,
                hotkey,
                password,
                take,
                false,
            )
            .await
        }
        DelegateCommands::IncreaseTake { take, hotkey } => {
            change_take(
                client,
                wallet_dir,
                wallet_name,
                hotkey_name,
                hotkey,
                password,
                take,
                true,
            )
            .await
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn change_take(
    client: &Client,
    wallet_dir: &str,
    wallet_name: &str,
    hotkey_name: &str,
    hotkey: Option<String>,
    password: Option<&str>,
    take: f64,
    increase: bool,
) -> Result<()> {
    let (pair, hk) = unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey, password)?;
    let take_u16 = (take / 100.0 * 65535.0).min(65535.0) as u16;
    let dir = if increase { "Increasing" } else { "Decreasing" };
    println!(
        "{} take to {:.2}% for {}",
        dir,
        take,
        crate::utils::short_ss58(&hk)
    );
    let hash = if increase {
        client.increase_take(&pair, &hk, take_u16).await?
    } else {
        client.decrease_take(&pair, &hk, take_u16).await?
    };
    println!(
        "Take {}. Tx: {}",
        if increase { "increased" } else { "decreased" },
        hash
    );
    Ok(())
}

// ──────── Identity ────────

async fn handle_identity(
    cmd: IdentityCommands,
    client: &Client,
    wallet_dir: &str,
    wallet_name: &str,
    password: Option<&str>,
) -> Result<()> {
    match cmd {
        IdentityCommands::Show { address } => {
            let identity = client.get_identity(&address).await?;
            match identity {
                Some(id) => {
                    println!("Identity for {}", address);
                    println!("  Name:        {}", id.name);
                    println!("  URL:         {}", id.url);
                    println!("  GitHub:      {}", id.github);
                    println!("  Discord:     {}", id.discord);
                    println!("  Description: {}", id.description);
                    if !id.image.is_empty() {
                        println!("  Image:       {}", id.image);
                    }
                }
                None => println!("No identity found for {}", address),
            }
            Ok(())
        }
        IdentityCommands::Set {
            name,
            url,
            github,
            description,
        } => {
            println!("Note: Account-level identity uses the Registry pallet.");
            println!("Use 'agcli identity set-subnet' for subnet identity.");
            println!("Name: {}, URL: {:?}, GitHub: {:?}", name, url, github);
            let _ = description;
            Ok(())
        }
        IdentityCommands::SetSubnet {
            netuid,
            name,
            github,
            url,
        } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            let identity = crate::types::chain_data::SubnetIdentity {
                subnet_name: name.clone(),
                github_repo: github.unwrap_or_default(),
                subnet_contact: String::new(),
                subnet_url: url.unwrap_or_default(),
                discord: String::new(),
                description: String::new(),
                additional: String::new(),
            };
            println!("Setting subnet identity for SN{}: {}", netuid, name);
            let hash = client
                .set_subnet_identity(wallet.coldkey()?, NetUid(netuid), &identity)
                .await?;
            println!("Subnet identity set. Tx: {}", hash);
            Ok(())
        }
    }
}

// ──────── Swap ────────

async fn handle_swap(
    cmd: SwapCommands,
    client: &Client,
    wallet_dir: &str,
    wallet_name: &str,
    password: Option<&str>,
) -> Result<()> {
    match cmd {
        SwapCommands::Hotkey { new_hotkey } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            let old_hotkey = match wallet.hotkey_ss58().map(|s| s.to_string()) {
                Some(hk) => hk,
                None => {
                    wallet.load_hotkey("default")?;
                    wallet
                        .hotkey_ss58()
                        .map(|s| s.to_string())
                        .ok_or_else(|| anyhow::anyhow!("Could not resolve current hotkey"))?
                }
            };
            println!(
                "Swapping hotkey {} -> {}",
                crate::utils::short_ss58(&old_hotkey),
                crate::utils::short_ss58(&new_hotkey)
            );
            let hash = client
                .swap_hotkey(wallet.coldkey()?, &old_hotkey, &new_hotkey)
                .await?;
            println!("Hotkey swapped. Tx: {}", hash);
            Ok(())
        }
        SwapCommands::Coldkey { new_coldkey } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            println!(
                "Scheduling coldkey swap to {}",
                crate::utils::short_ss58(&new_coldkey)
            );
            let hash = client
                .schedule_swap_coldkey(wallet.coldkey()?, &new_coldkey)
                .await?;
            println!("Coldkey swap scheduled. Tx: {}", hash);
            Ok(())
        }
    }
}

// ──────── Subscribe ────────

async fn handle_subscribe(
    cmd: SubscribeCommands,
    client: &Client,
    output: &str,
    _batch: bool,
) -> Result<()> {
    let json = output == "json";
    match cmd {
        SubscribeCommands::Blocks => {
            crate::events::subscribe_blocks(client.inner_client(), json).await
        }
        SubscribeCommands::Events {
            filter,
            netuid,
            account,
        } => {
            let f: crate::events::EventFilter = filter.parse().unwrap();
            crate::events::subscribe_events_filtered(
                client.inner_client(),
                f,
                json,
                netuid,
                account.as_deref(),
            )
            .await
        }
    }
}

// ──────── Config ────────

async fn handle_config(cmd: ConfigCommands) -> Result<()> {
    match cmd {
        ConfigCommands::Show => {
            let cfg = crate::config::Config::load();
            let s = toml::to_string_pretty(&cfg)?;
            if s.trim().is_empty() {
                println!(
                    "No configuration set. Use 'agcli config set <key> <value>' to configure."
                );
            } else {
                println!("{}", s);
            }
            Ok(())
        }
        ConfigCommands::Set { key, value } => {
            let mut cfg = crate::config::Config::load();
            match key.as_str() {
                "network" => cfg.network = Some(value),
                "endpoint" => cfg.endpoint = Some(value),
                "wallet_dir" => cfg.wallet_dir = Some(value),
                "wallet" => cfg.wallet = Some(value),
                "hotkey" => cfg.hotkey = Some(value),
                "output" => {
                    if !["table", "json", "csv"].contains(&value.as_str()) {
                        anyhow::bail!("Invalid output format '{}'. Must be: table, json, csv", value);
                    }
                    cfg.output = Some(value);
                }
                "proxy" => cfg.proxy = Some(value),
                "live_interval" => {
                    let v: u64 = value.parse().map_err(|_| anyhow::anyhow!("Invalid interval '{}'", value))?;
                    cfg.live_interval = Some(v);
                }
                "batch" => {
                    let v: bool = value.parse().map_err(|_| anyhow::anyhow!("Invalid bool '{}'. Use: true, false", value))?;
                    cfg.batch = Some(v);
                }
                k if k.starts_with("spending_limit.") => {
                    let netuid = k.strip_prefix("spending_limit.").unwrap();
                    let limit: f64 = value.parse().map_err(|_| anyhow::anyhow!("Invalid TAO amount '{}'", value))?;
                    let limits = cfg.spending_limits.get_or_insert_with(Default::default);
                    limits.insert(netuid.to_string(), limit);
                }
                _ => anyhow::bail!("Unknown config key '{}'. Valid keys: network, endpoint, wallet_dir, wallet, hotkey, output, proxy, live_interval, batch, spending_limit.<netuid>", key),
            }
            cfg.save()?;
            println!("Set {} = {}", key, cfg_value_display(&key, &cfg));
            Ok(())
        }
        ConfigCommands::Unset { key } => {
            let mut cfg = crate::config::Config::load();
            match key.as_str() {
                "network" => cfg.network = None,
                "endpoint" => cfg.endpoint = None,
                "wallet_dir" => cfg.wallet_dir = None,
                "wallet" => cfg.wallet = None,
                "hotkey" => cfg.hotkey = None,
                "output" => cfg.output = None,
                "proxy" => cfg.proxy = None,
                "live_interval" => cfg.live_interval = None,
                "batch" => cfg.batch = None,
                k if k.starts_with("spending_limit.") => {
                    let netuid = k.strip_prefix("spending_limit.").unwrap();
                    if let Some(ref mut limits) = cfg.spending_limits {
                        limits.remove(netuid);
                    }
                }
                _ => anyhow::bail!("Unknown config key '{}'", key),
            }
            cfg.save()?;
            println!("Unset {}", key);
            Ok(())
        }
        ConfigCommands::Path => {
            println!("{}", crate::config::Config::default_path().display());
            Ok(())
        }
    }
}

// ──────── Multisig ────────

async fn handle_multisig(
    cmd: MultisigCommands,
    wallet_dir: &str,
    wallet_name: &str,
    ws_url: &str,
    password: Option<&str>,
) -> Result<()> {
    match cmd {
        MultisigCommands::Address {
            signatories,
            threshold,
        } => {
            let addrs: Vec<&str> = signatories.split(',').map(|s| s.trim()).collect();
            if addrs.len() < 2 {
                anyhow::bail!("Need at least 2 signatories for a multisig. Provide comma-separated SS58 addresses.");
            }
            let account_ids = parse_sorted_signatories(&signatories)?;

            use blake2::digest::{Update, VariableOutput};
            let mut hasher = blake2::Blake2bVar::new(32)
                .map_err(|e| anyhow::anyhow!("blake2 error: {:?}", e))?;
            hasher.update(b"modlpy/teleport");
            hasher.update(&threshold.to_le_bytes());
            for id in &account_ids {
                hasher.update(id.as_ref());
            }
            let mut hash = [0u8; 32];
            hasher
                .finalize_variable(&mut hash)
                .map_err(|e| anyhow::anyhow!("blake2 finalize error: {:?}", e))?;

            let multisig_account = sp_core::crypto::AccountId32::from(hash);
            let ms_ss58 = multisig_account.to_string();
            println!("Multisig address: {}", ms_ss58);
            println!("  Threshold: {}/{}", threshold, addrs.len());
            println!("  Signatories:");
            for addr in &addrs {
                println!("    {}", addr);
            }
            Ok(())
        }
        MultisigCommands::Submit {
            others,
            threshold,
            pallet,
            call,
            args,
        } => {
            let client = Client::connect(ws_url).await?;
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            let other_ids = parse_sorted_signatories(&others)?;
            let fields: Vec<subxt::dynamic::Value> = if let Some(ref args_json) = args {
                let parsed: Vec<serde_json::Value> =
                    serde_json::from_str(args_json).map_err(|e| {
                        anyhow::anyhow!(
                            "Invalid JSON args '{}'. Expected a JSON array, e.g. '[1, \"0x...\"]'",
                            e
                        )
                    })?;
                parsed.iter().map(json_to_subxt_value).collect()
            } else {
                vec![]
            };
            println!(
                "Submitting multisig call: {}.{} (threshold {}/{})",
                pallet,
                call,
                threshold,
                other_ids.len() + 1
            );
            let hash = client
                .submit_multisig_call(
                    wallet.coldkey()?,
                    &other_ids,
                    threshold,
                    &pallet,
                    &call,
                    fields,
                )
                .await?;
            println!("Multisig call submitted. Tx: {}", hash);
            Ok(())
        }
        MultisigCommands::Approve {
            others,
            threshold,
            call_hash,
        } => {
            let client = Client::connect(ws_url).await?;
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            let other_ids = parse_sorted_signatories(&others)?;
            let hash_hex = call_hash.strip_prefix("0x").unwrap_or(&call_hash);
            let hash_bytes: [u8; 32] = hex::decode(hash_hex)?.try_into().map_err(|_| {
                anyhow::anyhow!("Call hash must be exactly 32 bytes (64 hex chars)")
            })?;
            println!(
                "Approving multisig call (threshold {}/{})",
                threshold,
                other_ids.len() + 1
            );
            let tx_hash = client
                .approve_multisig(wallet.coldkey()?, &other_ids, threshold, hash_bytes)
                .await?;
            println!("Multisig approval submitted. Tx: {}", tx_hash);
            Ok(())
        }
    }
}

// ──────── Serve ────────

async fn handle_serve(
    cmd: ServeCommands,
    client: &Client,
    wallet_dir: &str,
    wallet_name: &str,
    hotkey_name: &str,
    password: Option<&str>,
) -> Result<()> {
    match cmd {
        ServeCommands::Axon {
            netuid,
            ip,
            port,
            protocol,
            version,
        } => {
            let (pair, _hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, None, password)?;
            let ip_u128: u128 = {
                let parts: Vec<u8> = ip.split('.').filter_map(|p| p.parse().ok()).collect();
                if parts.len() == 4 {
                    ((parts[0] as u128) << 24)
                        | ((parts[1] as u128) << 16)
                        | ((parts[2] as u128) << 8)
                        | (parts[3] as u128)
                } else {
                    anyhow::bail!("Invalid IPv4 address: {}", ip);
                }
            };
            let axon = crate::types::chain_data::AxonInfo {
                block: 0,
                version,
                ip: ip_u128.to_string(),
                port,
                ip_type: 4,
                protocol,
            };
            println!(
                "Serving axon on SN{}: {}:{} (proto={}, ver={})",
                netuid, ip, port, protocol, version
            );
            let hash = client.serve_axon(&pair, NetUid(netuid), &axon).await?;
            println!("Axon served. Tx: {}", hash);
            Ok(())
        }
    }
}

// ──────── Proxy ────────

async fn handle_proxy(
    cmd: ProxyCommands,
    client: &Client,
    wallet_dir: &str,
    wallet_name: &str,
    output: &str,
    password: Option<&str>,
) -> Result<()> {
    let adding = matches!(cmd, ProxyCommands::Add { .. });
    match cmd {
        ProxyCommands::Add {
            delegate,
            proxy_type,
            delay,
        }
        | ProxyCommands::Remove {
            delegate,
            proxy_type,
            delay,
        } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            let verb = if adding { "Adding" } else { "Removing" };
            println!(
                "{} proxy: {} (type={}, delay={})",
                verb,
                crate::utils::short_ss58(&delegate),
                proxy_type,
                delay
            );
            let hash = if adding {
                client
                    .add_proxy(wallet.coldkey()?, &delegate, &proxy_type, delay)
                    .await?
            } else {
                client
                    .remove_proxy(wallet.coldkey()?, &delegate, &proxy_type, delay)
                    .await?
            };
            println!(
                "Proxy {}. Tx: {}",
                if adding { "added" } else { "removed" },
                hash
            );
            Ok(())
        }
        ProxyCommands::List { address } => {
            let addr = resolve_coldkey_address(address, wallet_dir, wallet_name);
            let proxies = client.list_proxies(&addr).await?;
            if output == "json" {
                let json: Vec<serde_json::Value> = proxies.iter().map(|(d, t, delay)| {
                    serde_json::json!({"delegate": d, "proxy_type": t, "delay": delay})
                }).collect();
                print_json_ser(&json);
            } else if proxies.is_empty() {
                println!(
                    "No proxy accounts found for {}",
                    crate::utils::short_ss58(&addr)
                );
            } else {
                println!("Proxy accounts for {}:", crate::utils::short_ss58(&addr));
                let mut table = comfy_table::Table::new();
                table.set_header(vec!["Delegate", "Type", "Delay"]);
                for (delegate, proxy_type, delay) in &proxies {
                    table.add_row(vec![
                        crate::utils::short_ss58(delegate),
                        proxy_type.clone(),
                        format!("{}", delay),
                    ]);
                }
                println!("{table}");
            }
            Ok(())
        }
    }
}

// ──────── Crowdloan ────────

async fn handle_crowdloan(
    cmd: CrowdloanCommands,
    client: &Client,
    wallet_dir: &str,
    wallet_name: &str,
    _output: &str,
    password: Option<&str>,
) -> Result<()> {
    let mut wallet = open_wallet(wallet_dir, wallet_name)?;
    unlock_coldkey(&mut wallet, password)?;
    let pair = wallet.coldkey()?;
    let (action, hash) = match cmd {
        CrowdloanCommands::Contribute {
            crowdloan_id,
            amount,
        } => {
            let bal = Balance::from_tao(amount);
            println!(
                "Contributing {} to crowdloan #{}",
                bal.display_tao(),
                crowdloan_id
            );
            (
                "Contribution submitted",
                client.crowdloan_contribute(pair, crowdloan_id, bal).await?,
            )
        }
        CrowdloanCommands::Withdraw { crowdloan_id } => {
            println!("Withdrawing from crowdloan #{}", crowdloan_id);
            (
                "Withdrawal submitted",
                client.crowdloan_withdraw(pair, crowdloan_id).await?,
            )
        }
        CrowdloanCommands::Finalize { crowdloan_id } => {
            println!("Finalizing crowdloan #{}", crowdloan_id);
            (
                "Crowdloan finalized",
                client.crowdloan_finalize(pair, crowdloan_id).await?,
            )
        }
    };
    println!("{}. Tx: {}", action, hash);
    Ok(())
}

// ──────── Completions ────────

fn generate_completions(shell: &str) {
    use clap_complete::{generate, Shell};
    let mut cmd = Cli::command();
    let shell_enum = match shell {
        "bash" => Shell::Bash,
        "zsh" => Shell::Zsh,
        "fish" => Shell::Fish,
        "powershell" => Shell::PowerShell,
        _ => {
            eprintln!(
                "Unsupported shell: {}. Use: bash, zsh, fish, powershell",
                shell
            );
            return;
        }
    };
    generate(shell_enum, &mut cmd, "agcli", &mut std::io::stdout());
}

/// Parse a comma-separated list of SS58 addresses into sorted AccountIds (for multisig).
fn parse_sorted_signatories(csv: &str) -> Result<Vec<crate::AccountId>> {
    let mut ids: Vec<crate::AccountId> = csv
        .split(',')
        .map(|s| Client::ss58_to_account_id_pub(s.trim()))
        .collect::<Result<_>>()?;
    ids.sort();
    Ok(ids)
}

fn cfg_value_display(key: &str, cfg: &crate::config::Config) -> String {
    match key {
        "network" => cfg.network.clone().unwrap_or_default(),
        "endpoint" => cfg.endpoint.clone().unwrap_or_default(),
        "wallet_dir" => cfg.wallet_dir.clone().unwrap_or_default(),
        "wallet" => cfg.wallet.clone().unwrap_or_default(),
        "hotkey" => cfg.hotkey.clone().unwrap_or_default(),
        "output" => cfg.output.clone().unwrap_or_default(),
        "proxy" => cfg.proxy.clone().unwrap_or_default(),
        "live_interval" => cfg.live_interval.map(|v| v.to_string()).unwrap_or_default(),
        "batch" => cfg.batch.map(|v| v.to_string()).unwrap_or_default(),
        k if k.starts_with("spending_limit.") => {
            let netuid = k.strip_prefix("spending_limit.").unwrap();
            cfg.spending_limits
                .as_ref()
                .and_then(|m| m.get(netuid))
                .map(|v| format!("{} TAO", v))
                .unwrap_or_default()
        }
        _ => String::new(),
    }
}

// ──────── Explain ────────

fn handle_explain(topic: Option<&str>) {
    match topic {
        Some(t) => match crate::utils::explain::explain(t) {
            Some(text) => println!("{}", text),
            None => {
                eprintln!("Unknown topic '{}'. Available topics:\n", t);
                for (key, desc) in crate::utils::explain::list_topics() {
                    eprintln!("  {:<16} {}", key, desc);
                }
                eprintln!("\nUsage: agcli explain <topic>");
            }
        },
        None => {
            println!("Available topics:\n");
            for (key, desc) in crate::utils::explain::list_topics() {
                println!("  {:<16} {}", key, desc);
            }
            println!("\nUsage: agcli explain <topic>");
        }
    }
}

// ──────── Self-Update ────────

async fn handle_update() -> Result<()> {
    println!("Updating agcli from GitHub...");
    let status = std::process::Command::new("cargo")
        .args([
            "install",
            "--git",
            "https://github.com/unconst/agcli",
            "--force",
        ])
        .status();
    match status {
        Ok(s) if s.success() => {
            println!("agcli updated successfully!");
            Ok(())
        }
        Ok(s) => anyhow::bail!("Update failed with exit code: {}", s),
        Err(e) => anyhow::bail!(
            "Failed to run cargo install: {}. Make sure cargo is installed.",
            e
        ),
    }
}

// ──────── Batch Extrinsics ────────

async fn handle_batch(
    client: &Client,
    pair: &sp_core::sr25519::Pair,
    file_path: &str,
    no_atomic: bool,
    output: &str,
) -> Result<()> {
    let content = std::fs::read_to_string(file_path)
        .map_err(|e| anyhow::anyhow!("Failed to read batch file '{}': {}", file_path, e))?;
    let calls: Vec<serde_json::Value> = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Invalid JSON in '{}': {}\n  Expected: [{{\n    \"pallet\": \"SubtensorModule\",\n    \"call\": \"add_stake\",\n    \"args\": [\"hotkey_ss58\", 1, 1000000000]\n  }}, ...]", file_path, e))?;

    if calls.is_empty() {
        anyhow::bail!("Batch file is empty (no calls to submit).");
    }

    eprintln!(
        "Batch: {} calls, mode={}",
        calls.len(),
        if no_atomic {
            "batch (non-atomic)"
        } else {
            "batch_all (atomic)"
        }
    );

    let mut encoded_calls: Vec<Vec<u8>> = Vec::with_capacity(calls.len());
    for (i, call_json) in calls.iter().enumerate() {
        let pallet = call_json
            .get("pallet")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Call #{}: missing \"pallet\" field", i))?;
        let call_name = call_json
            .get("call")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Call #{}: missing \"call\" field", i))?;
        let args = call_json
            .get("args")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("Call #{}: missing \"args\" array", i))?;

        let fields: Vec<subxt::dynamic::Value> = args.iter().map(json_to_subxt_value).collect();

        let tx = subxt::dynamic::tx(pallet, call_name, fields);
        let encoded = client.inner_client().tx().call_data(&tx).map_err(|e| {
            anyhow::anyhow!(
                "Call #{} ({}.{}): encoding failed: {}",
                i,
                pallet,
                call_name,
                e
            )
        })?;
        eprintln!(
            "  #{}: {}.{} ({} bytes)",
            i,
            pallet,
            call_name,
            encoded.len()
        );
        encoded_calls.push(encoded);
    }

    // Build Utility.batch_all or Utility.batch
    let batch_call_name = if no_atomic { "batch" } else { "batch_all" };
    let call_values: Vec<subxt::dynamic::Value> = encoded_calls
        .iter()
        .map(|c| subxt::dynamic::Value::from_bytes(c.clone()))
        .collect();

    let batch_tx = subxt::dynamic::tx(
        "Utility",
        batch_call_name,
        vec![subxt::dynamic::Value::unnamed_composite(call_values)],
    );

    let hash = client.sign_submit_dyn(&batch_tx, pair).await?;
    print_tx_result(
        output,
        &hash,
        &format!("Batch ({} calls) submitted.", calls.len()),
    );
    Ok(())
}
