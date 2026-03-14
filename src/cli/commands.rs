//! CLI command execution — thin dispatcher to focused handler modules.

use crate::cli::*;
use crate::cli::helpers::*;
use crate::chain::Client;
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

    match cli.command {
        Commands::Wallet(cmd) => wallet_cmds::handle_wallet(cmd, &cli.wallet_dir, password.as_deref()).await,
        Commands::Balance { address } => {
            let client = Client::connect(network.ws_url()).await?;
            let addr = resolve_coldkey_address(address, &cli.wallet_dir, &cli.wallet);
            let balance = client.get_balance_ss58(&addr).await?;
            if output == "json" {
                println!("{}", serde_json::json!({"address": addr, "balance_rao": balance.rao(), "balance_tao": balance.tao()}));
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
            println!("Transferring {} to {}", balance.display_tao(), dest);
            let hash = client.transfer(wallet.coldkey()?, &dest, balance).await?;
            if output == "json" {
                println!("{}", serde_json::json!({"tx_hash": hash}));
            } else {
                println!("Transaction submitted: {}", hash);
            }
            Ok(())
        }
        Commands::TransferAll { dest, keep_alive } => {
            let client = Client::connect(network.ws_url()).await?;
            let mut wallet = open_wallet(&cli.wallet_dir, &cli.wallet)?;
            unlock_coldkey(&mut wallet, password.as_deref())?;
            println!("Transferring all balance to {} (keep_alive={})", dest, keep_alive);
            let hash = client.transfer_all(wallet.coldkey()?, &dest, keep_alive).await?;
            if output == "json" {
                println!("{}", serde_json::json!({"tx_hash": hash}));
            } else {
                println!("All balance transferred. Tx: {}", hash);
            }
            Ok(())
        }
        Commands::Stake(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            stake_cmds::handle_stake(cmd, &client, &cli.wallet_dir, &cli.wallet, &cli.hotkey, output, password.as_deref(), yes).await
        }
        Commands::Subnet(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            handle_subnet(cmd, &client, &cli.wallet_dir, &cli.wallet, &cli.hotkey, output, live_interval, password.as_deref()).await
        }
        Commands::Weights(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            handle_weights(cmd, &client, &cli.wallet_dir, &cli.wallet, &cli.hotkey, password.as_deref()).await
        }
        Commands::Root(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            handle_root(cmd, &client, &cli.wallet_dir, &cli.wallet, &cli.hotkey, password.as_deref()).await
        }
        Commands::Delegate(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            handle_delegate(cmd, &client, &cli.wallet_dir, &cli.wallet, &cli.hotkey, output, password.as_deref()).await
        }
        Commands::View(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            view_cmds::handle_view(cmd, &client, &cli.wallet_dir, &cli.wallet, output, live_interval).await
        }
        Commands::Identity(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            handle_identity(cmd, &client, &cli.wallet_dir, &cli.wallet, password.as_deref()).await
        }
        Commands::Serve(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            handle_serve(cmd, &client, &cli.wallet_dir, &cli.wallet, &cli.hotkey, password.as_deref()).await
        }
        Commands::Proxy(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            handle_proxy(cmd, &client, &cli.wallet_dir, &cli.wallet, output, password.as_deref()).await
        }
        Commands::Crowdloan(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            handle_crowdloan(cmd, &client, &cli.wallet_dir, &cli.wallet, output, password.as_deref()).await
        }
        Commands::Swap(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            handle_swap(cmd, &client, &cli.wallet_dir, &cli.wallet, password.as_deref()).await
        }
        Commands::Subscribe(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            handle_subscribe(cmd, &client, output).await
        }
        Commands::Multisig(cmd) => {
            handle_multisig(cmd, &cli.wallet_dir, &cli.wallet, network.ws_url(), password.as_deref()).await
        }
        Commands::Config(cmd) => handle_config(cmd).await,
        Commands::Completions { shell } => {
            generate_completions(&shell);
            Ok(())
        }
        Commands::Update => {
            handle_update().await
        }
    }
}

// ──────── Subnet ────────

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
                println!("{}", serde_json::to_string_pretty(&subnets)?);
            } else if output == "csv" {
                println!("netuid,name,n,max_n,tempo,emission,burn_rao,owner");
                for s in &subnets {
                    println!("{},{},{},{},{},{},{},{}", s.netuid, s.name, s.n, s.max_n, s.tempo, s.emission_value, s.burn.rao(), s.owner);
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
                        if !di.name.is_empty() { s.name = di.name.clone(); }
                    }
                    if output == "json" {
                        println!("{}", serde_json::to_string_pretty(&s)?);
                    } else {
                        println!("Subnet {} ({})", s.netuid, s.name);
                        println!("  Symbol:        {}", s.symbol);
                        println!("  Neurons:       {}/{}", s.n, s.max_n);
                        println!("  Tempo:         {}", s.tempo);
                        println!("  Emission:      {}", s.emission_value);
                        println!("  Burn:          {}", s.burn.display_tao());
                        println!("  Difficulty:    {}", s.difficulty);
                        println!("  Immunity:      {} blocks", s.immunity_period);
                        println!("  Owner:         {}", s.owner);
                        println!("  Registration:  {}", if s.registration_allowed { "open" } else { "closed" });
                        if let Some(ref di) = dynamic {
                            println!("  Price:         {:.6} τ/α", di.price);
                            println!("  TAO in pool:   {}", di.tao_in.display_tao());
                            println!("  Alpha in:      {}", di.alpha_in);
                            println!("  Alpha out:     {}", di.alpha_out);
                            println!("  Volume:        {}", di.subnet_volume);
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
                        ("registration_allowed", format!("{}", h.registration_allowed)),
                        ("target_regs_per_interval", format!("{}", h.target_regs_per_interval)),
                        ("min_burn", h.min_burn.display_tao()),
                        ("max_burn", h.max_burn.display_tao()),
                        ("bonds_moving_avg", format!("{}", h.bonds_moving_avg)),
                        ("max_regs_per_block", format!("{}", h.max_regs_per_block)),
                        ("serving_rate_limit", format!("{}", h.serving_rate_limit)),
                        ("max_validators", format!("{}", h.max_validators)),
                        ("adjustment_alpha", format!("{}", h.adjustment_alpha)),
                        ("difficulty", format!("{}", h.difficulty)),
                        ("commit_reveal_weights", format!("{}", h.commit_reveal_weights_enabled)),
                        ("commit_reveal_interval", format!("{}", h.commit_reveal_weights_interval)),
                        ("liquid_alpha_enabled", format!("{}", h.liquid_alpha_enabled)),
                    ];
                    for (k, v) in &rows { table.add_row(vec![k, v.as_str()]); }
                    println!("{table}");
                }
                None => println!("Hyperparameters not found for SN{}.", netuid),
            }
            Ok(())
        }
        SubnetCommands::Metagraph { netuid } => {
            if let Some(interval) = live_interval {
                return crate::live::live_metagraph(client, netuid.into(), interval).await;
            }
            let mg = crate::queries::fetch_metagraph(client, netuid.into()).await?;
            if output == "json" {
                println!("{}", serde_json::to_string_pretty(&mg)?);
            } else if output == "csv" {
                println!("uid,hotkey,coldkey,stake_rao,rank,trust,consensus,incentive,dividends,emission,validator_permit");
                for n in &mg.neurons {
                    println!("{},{},{},{},{:.6},{:.6},{:.6},{:.6},{:.6},{:.0},{}", n.uid, n.hotkey, n.coldkey, n.stake.rao(), n.rank, n.trust, n.consensus, n.incentive, n.dividends, n.emission, n.validator_permit);
                }
            } else {
                println!("Metagraph SN{} — {} neurons, block {}", netuid, mg.n, mg.block);
                let mut table = comfy_table::Table::new();
                table.set_header(vec![
                    "UID", "Hotkey", "Stake", "Rank", "Trust", "Incentive", "Emission", "VP",
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
                        if n.validator_permit { "Y" } else { "" }.to_string(),
                    ]);
                }
                println!("{table}");
            }
            Ok(())
        }
        SubnetCommands::Register => {
            let (pair, hk) = unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, None, password)?;
            println!("Registering new subnet...");
            let hash = client.register_network(&pair, &hk).await?;
            println!("Subnet registered. Tx: {}", hash);
            Ok(())
        }
        SubnetCommands::RegisterNeuron { netuid } => {
            let (pair, hk) = unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, None, password)?;
            println!("Burn-registering on SN{} with hotkey {}", netuid, crate::utils::short_ss58(&hk));
            let hash = client.burned_register(&pair, NetUid(netuid), &hk).await?;
            println!("Neuron registered. Tx: {}", hash);
            Ok(())
        }
        SubnetCommands::Pow { netuid, threads } => {
            let (pair, hk) = unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, None, password)?;
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
                    crate::utils::pow::solve_pow_range(&bh, &hk_bytes, diff, offset, attempts_per_thread)
                }));
            }
            let result = handles.into_iter()
                .find_map(|h| h.join().ok().flatten());
            match result {
                Some((nonce, work)) => {
                    println!("POW solved! Nonce: {}", nonce);
                    let hash = client.pow_register(&pair, NetUid(netuid), &hk, block_number, nonce, work).await?;
                    println!("POW registered. Tx: {}", hash);
                }
                None => println!("POW not found after {} attempts/thread. Try burn registration.", attempts_per_thread),
            }
            Ok(())
        }
        SubnetCommands::Dissolve { netuid } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            println!("Dissolving subnet SN{} (owner only)", netuid);
            let hash = client.dissolve_network(wallet.coldkey()?, NetUid(netuid)).await?;
            println!("Subnet dissolved. Tx: {}", hash);
            Ok(())
        }
    }
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
        WeightCommands::Set { netuid, weights, version_key } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            wallet.load_hotkey(hotkey_name)?;
            let (uids, wts) = parse_weight_pairs(&weights)?;
            println!("Setting {} weights on SN{} (version_key={})", uids.len(), netuid, version_key);
            let hash = client.set_weights(wallet.hotkey()?, NetUid(netuid), &uids, &wts, version_key).await?;
            println!("Weights set. Tx: {}", hash);
            Ok(())
        }
        WeightCommands::Commit { netuid, weights, salt } => {
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
            for u in &uids { hasher.update(&u.to_le_bytes()); }
            for w in &wts { hasher.update(&w.to_le_bytes()); }
            hasher.update(salt_str.as_bytes());
            let mut hash_out = [0u8; 32];
            hasher.finalize_variable(&mut hash_out)
                .map_err(|e| anyhow::anyhow!("blake2 finalize error: {:?}", e))?;
            println!("Committing weights on SN{}", netuid);
            println!("  Commit hash: 0x{}", hex::encode(hash_out));
            println!("  Save this salt for reveal: {}", salt_str);
            let hash = client.commit_weights(wallet.hotkey()?, NetUid(netuid), hash_out).await?;
            println!("Weights committed. Tx: {}", hash);
            Ok(())
        }
        WeightCommands::Reveal { netuid, weights, salt, version_key } => {
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
            println!("Revealing {} weights on SN{} (version_key={})", uids.len(), netuid, version_key);
            let hash = client.reveal_weights(wallet.hotkey()?, NetUid(netuid), &uids, &wts, &salt_u16, version_key).await?;
            println!("Weights revealed. Tx: {}", hash);
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
            let (pair, hk) = unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, None, password)?;
            println!("Registering on root network with hotkey {}", crate::utils::short_ss58(&hk));
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
            let hash = client.set_weights(wallet.hotkey()?, NetUid::ROOT, &uids, &wts, 0).await?;
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
                println!("{}", serde_json::to_string_pretty(&delegates)?);
            } else if output == "csv" {
                println!("hotkey,owner,take_pct,total_stake_rao,nominators");
                for d in &delegates {
                    println!("{},{},{:.4},{},{}", d.hotkey, d.owner, d.take * 100.0, d.total_stake.rao(), d.nominators.len());
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
                            println!("    {} — {}", crate::utils::short_ss58(addr), stake.display_tao());
                        }
                    }
                }
                None => println!("Delegate not found for {}", hotkey_ss58),
            }
            Ok(())
        }
        DelegateCommands::DecreaseTake { take, hotkey } => {
            let (pair, hk) = unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey, password)?;
            let take_u16 = (take / 100.0 * 65535.0).min(65535.0) as u16;
            println!("Decreasing take to {:.2}% for {}", take, crate::utils::short_ss58(&hk));
            let hash = client.decrease_take(&pair, &hk, take_u16).await?;
            println!("Take decreased. Tx: {}", hash);
            Ok(())
        }
        DelegateCommands::IncreaseTake { take, hotkey } => {
            let (pair, hk) = unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey, password)?;
            let take_u16 = (take / 100.0 * 65535.0).min(65535.0) as u16;
            println!("Increasing take to {:.2}% for {}", take, crate::utils::short_ss58(&hk));
            let hash = client.increase_take(&pair, &hk, take_u16).await?;
            println!("Take increased. Tx: {}", hash);
            Ok(())
        }
    }
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
        IdentityCommands::Set { name, url, github, description } => {
            println!("Note: Account-level identity uses the Registry pallet.");
            println!("Use 'agcli identity set-subnet' for subnet identity.");
            println!("Name: {}, URL: {:?}, GitHub: {:?}", name, url, github);
            let _ = description;
            Ok(())
        }
        IdentityCommands::SetSubnet { netuid, name, github, url } => {
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
            let hash = client.set_subnet_identity(wallet.coldkey()?, NetUid(netuid), &identity).await?;
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
                    wallet.hotkey_ss58().map(|s| s.to_string())
                        .ok_or_else(|| anyhow::anyhow!("Could not resolve current hotkey"))?
                }
            };
            println!("Swapping hotkey {} -> {}", crate::utils::short_ss58(&old_hotkey), crate::utils::short_ss58(&new_hotkey));
            let hash = client.swap_hotkey(wallet.coldkey()?, &old_hotkey, &new_hotkey).await?;
            println!("Hotkey swapped. Tx: {}", hash);
            Ok(())
        }
        SwapCommands::Coldkey { new_coldkey } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            println!("Scheduling coldkey swap to {}", crate::utils::short_ss58(&new_coldkey));
            let hash = client.schedule_swap_coldkey(wallet.coldkey()?, &new_coldkey).await?;
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
) -> Result<()> {
    let json = output == "json";
    match cmd {
        SubscribeCommands::Blocks => {
            crate::events::subscribe_blocks(client.inner_client(), json).await
        }
        SubscribeCommands::Events { filter } => {
            let f = crate::events::EventFilter::from_str(&filter);
            crate::events::subscribe_events(client.inner_client(), f, json).await
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
                println!("No configuration set. Use 'agcli config set <key> <value>' to configure.");
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
                _ => anyhow::bail!("Unknown config key '{}'. Valid keys: network, endpoint, wallet_dir, wallet, hotkey, output, proxy, live_interval", key),
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
        MultisigCommands::Address { signatories, threshold } => {
            let addrs: Vec<&str> = signatories.split(',').map(|s| s.trim()).collect();
            if addrs.len() < 2 {
                anyhow::bail!("Need at least 2 signatories for a multisig");
            }
            let mut account_ids: Vec<crate::AccountId> = addrs.iter()
                .map(|s| Client::ss58_to_account_id_pub(s))
                .collect::<Result<_>>()?;
            account_ids.sort();

            use blake2::digest::{Update, VariableOutput};
            let mut hasher = blake2::Blake2bVar::new(32)
                .map_err(|e| anyhow::anyhow!("blake2 error: {:?}", e))?;
            hasher.update(b"modlpy/teleport");
            hasher.update(&(threshold as u16).to_le_bytes());
            for id in &account_ids { hasher.update(id.as_ref()); }
            let mut hash = [0u8; 32];
            hasher.finalize_variable(&mut hash)
                .map_err(|e| anyhow::anyhow!("blake2 finalize error: {:?}", e))?;

            let multisig_account = sp_core::crypto::AccountId32::from(hash);
            let ms_ss58 = multisig_account.to_string();
            println!("Multisig address: {}", ms_ss58);
            println!("  Threshold: {}/{}", threshold, addrs.len());
            println!("  Signatories:");
            for addr in &addrs { println!("    {}", addr); }
            Ok(())
        }
        MultisigCommands::Submit { others, threshold, pallet, call, args } => {
            let client = Client::connect(ws_url).await?;
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;

            let other_addrs: Vec<&str> = others.split(',').map(|s| s.trim()).collect();
            let mut other_ids: Vec<crate::AccountId> = other_addrs.iter()
                .map(|s| Client::ss58_to_account_id_pub(s))
                .collect::<Result<_>>()?;
            other_ids.sort();

            let fields: Vec<subxt::dynamic::Value> = if let Some(ref args_json) = args {
                let parsed: Vec<serde_json::Value> = serde_json::from_str(args_json)
                    .map_err(|e| anyhow::anyhow!("Invalid JSON args: {}", e))?;
                parsed.iter().map(json_to_subxt_value).collect()
            } else {
                vec![]
            };

            println!("Submitting multisig call: {}.{} (threshold {}/{})",
                pallet, call, threshold, other_ids.len() + 1);
            let hash = client.submit_multisig_call(
                wallet.coldkey()?, &other_ids, threshold, &pallet, &call, fields,
            ).await?;
            println!("Multisig call submitted. Tx: {}", hash);
            Ok(())
        }
        MultisigCommands::Approve { others, threshold, call_hash } => {
            let client = Client::connect(ws_url).await?;
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;

            let other_addrs: Vec<&str> = others.split(',').map(|s| s.trim()).collect();
            let mut other_ids: Vec<crate::AccountId> = other_addrs.iter()
                .map(|s| Client::ss58_to_account_id_pub(s))
                .collect::<Result<_>>()?;
            other_ids.sort();

            let hash_hex = call_hash.strip_prefix("0x").unwrap_or(&call_hash);
            let hash_bytes: [u8; 32] = hex::decode(hash_hex)?
                .try_into()
                .map_err(|_| anyhow::anyhow!("Call hash must be 32 bytes"))?;

            println!("Approving multisig call (threshold {}/{})", threshold, other_ids.len() + 1);
            let tx_hash = client.approve_multisig(
                wallet.coldkey()?, &other_ids, threshold, hash_bytes,
            ).await?;
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
        ServeCommands::Axon { netuid, ip, port, protocol, version } => {
            let (pair, _hk) = unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, None, password)?;
            let ip_u128: u128 = {
                let parts: Vec<u8> = ip.split('.').filter_map(|p| p.parse().ok()).collect();
                if parts.len() == 4 {
                    ((parts[0] as u128) << 24) | ((parts[1] as u128) << 16)
                        | ((parts[2] as u128) << 8) | (parts[3] as u128)
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
            println!("Serving axon on SN{}: {}:{} (proto={}, ver={})", netuid, ip, port, protocol, version);
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
    match cmd {
        ProxyCommands::Add { delegate, proxy_type, delay } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            println!("Adding proxy: {} (type={}, delay={})", crate::utils::short_ss58(&delegate), proxy_type, delay);
            let hash = client.add_proxy(wallet.coldkey()?, &delegate, &proxy_type, delay).await?;
            println!("Proxy added. Tx: {}", hash);
            Ok(())
        }
        ProxyCommands::Remove { delegate, proxy_type, delay } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            println!("Removing proxy: {} (type={}, delay={})", crate::utils::short_ss58(&delegate), proxy_type, delay);
            let hash = client.remove_proxy(wallet.coldkey()?, &delegate, &proxy_type, delay).await?;
            println!("Proxy removed. Tx: {}", hash);
            Ok(())
        }
        ProxyCommands::List { address } => {
            let addr = resolve_coldkey_address(address, wallet_dir, wallet_name);
            let proxies = client.list_proxies(&addr).await?;
            if output == "json" {
                let json: Vec<serde_json::Value> = proxies.iter().map(|(d, t, delay)| {
                    serde_json::json!({"delegate": d, "proxy_type": t, "delay": delay})
                }).collect();
                println!("{}", serde_json::to_string_pretty(&json)?);
            } else if proxies.is_empty() {
                println!("No proxy accounts found for {}", crate::utils::short_ss58(&addr));
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
    output: &str,
    password: Option<&str>,
) -> Result<()> {
    let _ = output;
    match cmd {
        CrowdloanCommands::Contribute { crowdloan_id, amount } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            let bal = Balance::from_tao(amount);
            println!("Contributing {} to crowdloan #{}", bal.display_tao(), crowdloan_id);
            let hash = client.crowdloan_contribute(wallet.coldkey()?, crowdloan_id, bal).await?;
            println!("Contribution submitted. Tx: {}", hash);
            Ok(())
        }
        CrowdloanCommands::Withdraw { crowdloan_id } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            println!("Withdrawing from crowdloan #{}", crowdloan_id);
            let hash = client.crowdloan_withdraw(wallet.coldkey()?, crowdloan_id).await?;
            println!("Withdrawal submitted. Tx: {}", hash);
            Ok(())
        }
        CrowdloanCommands::Finalize { crowdloan_id } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            println!("Finalizing crowdloan #{}", crowdloan_id);
            let hash = client.crowdloan_finalize(wallet.coldkey()?, crowdloan_id).await?;
            println!("Crowdloan finalized. Tx: {}", hash);
            Ok(())
        }
    }
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
            eprintln!("Unsupported shell: {}. Use: bash, zsh, fish, powershell", shell);
            return;
        }
    };
    generate(shell_enum, &mut cmd, "agcli", &mut std::io::stdout());
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
        _ => String::new(),
    }
}

// ──────── Self-Update ────────

async fn handle_update() -> Result<()> {
    println!("Updating agcli from GitHub...");
    let status = std::process::Command::new("cargo")
        .args(["install", "--git", "https://github.com/unconst/agcli", "--force"])
        .status();
    match status {
        Ok(s) if s.success() => {
            println!("agcli updated successfully!");
            Ok(())
        }
        Ok(s) => anyhow::bail!("Update failed with exit code: {}", s),
        Err(e) => anyhow::bail!("Failed to run cargo install: {}. Make sure cargo is installed.", e),
    }
}
