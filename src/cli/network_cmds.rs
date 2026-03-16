//! Network operation handlers (root, delegate, identity, swap, subscribe, serve, proxy, crowdloan, liquidity).

use crate::chain::Client;
use crate::cli::helpers::*;
use crate::cli::*;
use crate::types::{Balance, NetUid};
use anyhow::Result;

// ──────── Root ────────

pub(super) async fn handle_root(cmd: RootCommands, client: &Client, ctx: &Ctx<'_>) -> Result<()> {
    let (wallet_dir, wallet_name, hotkey_name, password) = (
        ctx.wallet_dir,
        ctx.wallet_name,
        ctx.hotkey_name,
        ctx.password,
    );
    match cmd {
        RootCommands::Register => {
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, None, password)?;
            tracing::info!(hotkey = %crate::utils::short_ss58(&hk), "Registering on root network");
            println!(
                "Registering on root network with hotkey {}",
                crate::utils::short_ss58(&hk)
            );
            let hash = client.root_register(&pair, &hk).await?;
            tracing::info!(tx = %hash, "Root registration complete");
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

pub(super) async fn handle_delegate(
    cmd: DelegateCommands,
    client: &Client,
    ctx: &Ctx<'_>,
) -> Result<()> {
    match cmd {
        DelegateCommands::List => {
            let delegates = client.get_delegates().await?;
            let top: Vec<_> = delegates.into_iter().take(50).collect();
            render_rows(
                ctx.output,
                &top,
                "hotkey,owner,take_pct,total_stake_rao,nominators",
                |d| {
                    format!(
                        "{},{},{:.4},{},{}",
                        d.hotkey,
                        d.owner,
                        d.take * 100.0,
                        d.total_stake.rao(),
                        d.nominators.len()
                    )
                },
                &["Hotkey", "Owner", "Take", "Total Stake", "Nominators"],
                |d| {
                    vec![
                        crate::utils::short_ss58(&d.hotkey),
                        crate::utils::short_ss58(&d.owner),
                        format!("{:.2}%", d.take * 100.0),
                        d.total_stake.display_tao(),
                        format!("{}", d.nominators.len()),
                    ]
                },
                Some(&format!("{} delegates", top.len())),
            );
            Ok(())
        }
        DelegateCommands::Show { hotkey } => {
            let hotkey_ss58 = match hotkey {
                Some(hk) => hk,
                None => {
                    let mut wallet = open_wallet(ctx.wallet_dir, ctx.wallet_name)?;
                    resolve_hotkey_ss58(None, &mut wallet, ctx.hotkey_name)?
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
                        // Sort indices to avoid cloning the full nominators list
                        let mut indices: Vec<usize> = (0..d.nominators.len()).collect();
                        indices.sort_unstable_by(|&a, &b| {
                            d.nominators[b].1.rao().cmp(&d.nominators[a].1.rao())
                        });
                        for &i in indices.iter().take(10) {
                            let (addr, stake) = &d.nominators[i];
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
            change_take(client, ctx, hotkey, take, false).await
        }
        DelegateCommands::IncreaseTake { take, hotkey } => {
            change_take(client, ctx, hotkey, take, true).await
        }
    }
}

async fn change_take(
    client: &Client,
    ctx: &Ctx<'_>,
    hotkey: Option<String>,
    take: f64,
    increase: bool,
) -> Result<()> {
    let (pair, hk) = unlock_and_resolve(
        ctx.wallet_dir,
        ctx.wallet_name,
        ctx.hotkey_name,
        hotkey,
        ctx.password,
    )?;
    let take_u16 = (take / 100.0 * 65535.0).min(65535.0) as u16;
    let dir = if increase { "Increasing" } else { "Decreasing" };
    tracing::info!(hotkey = %crate::utils::short_ss58(&hk), take_pct = take, direction = dir, "Changing delegate take");
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
    tracing::info!(tx = %hash, "Delegate take changed");
    println!(
        "Take {}. Tx: {}",
        if increase { "increased" } else { "decreased" },
        hash
    );
    Ok(())
}

// ──────── Identity ────────

pub(super) async fn handle_identity(
    cmd: IdentityCommands,
    client: &Client,
    ctx: &Ctx<'_>,
) -> Result<()> {
    let (wallet_dir, wallet_name, password) = (ctx.wallet_dir, ctx.wallet_name, ctx.password);
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
            let _ = (&name, &url, &github, &description);
            anyhow::bail!(
                "Account-level identity (Registry pallet) is not yet supported.\n\
                 Use 'agcli identity set-subnet' to set subnet identity instead."
            );
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
            tracing::info!(netuid = netuid, name = %name, "Setting subnet identity");
            println!("Setting subnet identity for SN{}: {}", netuid, name);
            let hash = client
                .set_subnet_identity(wallet.coldkey()?, NetUid(netuid), &identity)
                .await?;
            tracing::info!(tx = %hash, netuid = netuid, "Subnet identity set");
            println!("Subnet identity set. Tx: {}", hash);
            Ok(())
        }
    }
}

// ──────── Swap ────────

pub(super) async fn handle_swap(cmd: SwapCommands, client: &Client, ctx: &Ctx<'_>) -> Result<()> {
    let (wallet_dir, wallet_name, password) = (ctx.wallet_dir, ctx.wallet_name, ctx.password);
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
            tracing::info!(old = %crate::utils::short_ss58(&old_hotkey), new = %crate::utils::short_ss58(&new_hotkey), "Swapping hotkey");
            println!(
                "Swapping hotkey {} -> {}",
                crate::utils::short_ss58(&old_hotkey),
                crate::utils::short_ss58(&new_hotkey)
            );
            let hash = client
                .swap_hotkey(wallet.coldkey()?, &old_hotkey, &new_hotkey)
                .await?;
            tracing::info!(tx = %hash, "Hotkey swapped");
            println!("Hotkey swapped. Tx: {}", hash);
            Ok(())
        }
        SwapCommands::Coldkey { new_coldkey } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            tracing::info!(new_coldkey = %crate::utils::short_ss58(&new_coldkey), "Scheduling coldkey swap");
            println!(
                "Scheduling coldkey swap to {}",
                crate::utils::short_ss58(&new_coldkey)
            );
            let hash = client
                .schedule_swap_coldkey(wallet.coldkey()?, &new_coldkey)
                .await?;
            tracing::info!(tx = %hash, "Coldkey swap scheduled");
            println!("Coldkey swap scheduled. Tx: {}", hash);
            Ok(())
        }
    }
}

// ──────── Subscribe ────────

pub(super) async fn handle_subscribe(
    cmd: SubscribeCommands,
    client: &Client,
    output: OutputFormat,
    _batch: bool,
) -> Result<()> {
    let json = output.is_json();
    match cmd {
        SubscribeCommands::Blocks => crate::events::subscribe_blocks(client.subxt(), json).await,
        SubscribeCommands::Events {
            filter,
            netuid,
            account,
        } => {
            let f: crate::events::EventFilter = filter
                .parse()
                .map_err(|e| anyhow::anyhow!("Invalid event filter '{}': {}", filter, e))?;
            crate::events::subscribe_events_filtered(
                client.subxt(),
                f,
                json,
                netuid,
                account.as_deref(),
            )
            .await
        }
    }
}

// ──────── Multisig ────────

pub(super) async fn handle_multisig(
    cmd: MultisigCommands,
    wallet_dir: &str,
    wallet_name: &str,
    network: &crate::types::Network,
    password: Option<&str>,
    dry_run: bool,
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
            let mut client = Client::connect_network(network).await?;
            client.set_dry_run(dry_run);
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
            let mut client = Client::connect_network(network).await?;
            client.set_dry_run(dry_run);
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

pub(super) async fn handle_serve(cmd: ServeCommands, client: &Client, ctx: &Ctx<'_>) -> Result<()> {
    let (wallet_dir, wallet_name, hotkey_name, password) = (
        ctx.wallet_dir,
        ctx.wallet_name,
        ctx.hotkey_name,
        ctx.password,
    );
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
        ServeCommands::Reset { netuid } => {
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, None, password)?;
            println!(
                "Resetting axon info for hotkey {} on SN{}",
                crate::utils::short_ss58(&hk),
                netuid
            );
            // Reset axon by setting all fields to zero
            let axon = crate::types::chain_data::AxonInfo {
                block: 0,
                version: 0,
                ip: "0".to_string(),
                port: 0,
                ip_type: 4,
                protocol: 0,
            };
            let hash = client.serve_axon(&pair, NetUid(netuid), &axon).await?;
            println!("Axon reset. Tx: {}", hash);
            Ok(())
        }
    }
}

// ──────── Proxy ────────

pub(super) async fn handle_proxy(cmd: ProxyCommands, client: &Client, ctx: &Ctx<'_>) -> Result<()> {
    let (wallet_dir, wallet_name, output, password) =
        (ctx.wallet_dir, ctx.wallet_name, ctx.output, ctx.password);
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
        ProxyCommands::CreatePure {
            proxy_type,
            delay,
            index,
        } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            println!(
                "Creating pure proxy (type={}, delay={}, index={})",
                proxy_type, delay, index
            );
            let hash = client
                .create_pure_proxy(wallet.coldkey()?, &proxy_type, delay, index)
                .await?;
            println!("Pure proxy created. Tx: {}", hash);
            println!("Check proxy list to find the new pure proxy address.");
            Ok(())
        }
        ProxyCommands::KillPure {
            spawner,
            proxy_type,
            index,
            height,
            ext_index,
        } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            println!(
                "WARNING: Killing pure proxy will make all funds in it permanently inaccessible!"
            );
            let hash = client
                .kill_pure_proxy(
                    wallet.coldkey()?,
                    &spawner,
                    &proxy_type,
                    index,
                    height,
                    ext_index,
                )
                .await?;
            println!("Pure proxy killed. Tx: {}", hash);
            Ok(())
        }
        ProxyCommands::List { address } => {
            let addr = resolve_coldkey_address(address, wallet_dir, wallet_name);
            let proxies = client.list_proxies(&addr).await?;
            if output.is_json() {
                let json: Vec<serde_json::Value> = proxies.iter().map(|(d, t, delay)| {
                    serde_json::json!({"delegate": d, "proxy_type": t, "delay": delay})
                }).collect();
                print_json_ser(&json);
            } else {
                render_rows(
                    OutputFormat::Table,
                    &proxies,
                    "",
                    |_| String::new(),
                    &["Delegate", "Type", "Delay"],
                    |(delegate, proxy_type, delay)| {
                        vec![
                            crate::utils::short_ss58(delegate),
                            proxy_type.clone(),
                            format!("{}", delay),
                        ]
                    },
                    Some(&format!(
                        "Proxy accounts for {}:",
                        crate::utils::short_ss58(&addr)
                    )),
                );
            }
            Ok(())
        }
    }
}

// ──────── Crowdloan ────────

pub(super) async fn handle_crowdloan(
    cmd: CrowdloanCommands,
    client: &Client,
    ctx: &Ctx<'_>,
) -> Result<()> {
    let (wallet_dir, wallet_name, password, output) =
        (ctx.wallet_dir, ctx.wallet_name, ctx.password, ctx.output);
    // Read-only query commands (no wallet needed)
    match &cmd {
        CrowdloanCommands::List => {
            let crowdloans = client.list_crowdloans().await?;
            if crowdloans.is_empty() {
                println!("No crowdloans found.");
            } else {
                render_rows(
                    output,
                    &crowdloans,
                    "id,creator,deposit,raised,cap,end_block,finalized",
                    |(id, creator, deposit, raised, cap, end_block, finalized)| {
                        format!(
                            "{},{},{},{},{},{},{}",
                            id,
                            creator,
                            Balance::from_rao(*deposit).display_tao(),
                            Balance::from_rao(*raised).display_tao(),
                            Balance::from_rao(*cap).display_tao(),
                            end_block,
                            finalized,
                        )
                    },
                    &[
                        "ID",
                        "Creator",
                        "Deposit",
                        "Raised",
                        "Cap",
                        "End Block",
                        "Done",
                    ],
                    |(id, creator, deposit, raised, cap, end_block, finalized)| {
                        vec![
                            format!("{}", id),
                            crate::utils::short_ss58(creator),
                            Balance::from_rao(*deposit).display_tao(),
                            Balance::from_rao(*raised).display_tao(),
                            Balance::from_rao(*cap).display_tao(),
                            format!("{}", end_block),
                            if *finalized { "Yes" } else { "No" }.to_string(),
                        ]
                    },
                    Some(&format!("{} crowdloans", crowdloans.len())),
                );
            }
            return Ok(());
        }
        CrowdloanCommands::Info { crowdloan_id } => {
            let info = client.get_crowdloan_info(*crowdloan_id).await?;
            match info {
                Some((
                    creator,
                    deposit,
                    raised,
                    cap,
                    end_block,
                    min_contrib,
                    finalized,
                    target,
                )) => {
                    println!("Crowdloan #{}", crowdloan_id);
                    println!("  Creator:          {}", creator);
                    println!(
                        "  Deposit:          {}",
                        Balance::from_rao(deposit).display_tao()
                    );
                    println!(
                        "  Raised:           {}",
                        Balance::from_rao(raised).display_tao()
                    );
                    println!(
                        "  Cap:              {}",
                        Balance::from_rao(cap).display_tao()
                    );
                    println!(
                        "  Progress:         {:.1}%",
                        if cap > 0 {
                            raised as f64 / cap as f64 * 100.0
                        } else {
                            0.0
                        }
                    );
                    println!("  End block:        {}", end_block);
                    println!(
                        "  Min contribution: {}",
                        Balance::from_rao(min_contrib).display_tao()
                    );
                    println!(
                        "  Finalized:        {}",
                        if finalized { "Yes" } else { "No" }
                    );
                    if let Some(t) = target {
                        println!("  Target address:   {}", t);
                    }
                }
                None => println!("Crowdloan #{} not found.", crowdloan_id),
            }
            return Ok(());
        }
        CrowdloanCommands::Contributors { crowdloan_id } => {
            let contributors = client.get_crowdloan_contributors(*crowdloan_id).await?;
            if contributors.is_empty() {
                println!("No contributors for crowdloan #{}.", crowdloan_id);
            } else {
                let total: u64 = contributors.iter().map(|(_, amount)| amount).sum();
                render_rows(
                    output,
                    &contributors,
                    "address,amount_rao,pct",
                    |(addr, amount)| {
                        format!(
                            "{},{},{:.2}",
                            addr,
                            amount,
                            if total > 0 {
                                *amount as f64 / total as f64 * 100.0
                            } else {
                                0.0
                            },
                        )
                    },
                    &["Address", "Amount", "%"],
                    |(addr, amount)| {
                        vec![
                            crate::utils::short_ss58(addr),
                            Balance::from_rao(*amount).display_tao(),
                            format!(
                                "{:.1}%",
                                if total > 0 {
                                    *amount as f64 / total as f64 * 100.0
                                } else {
                                    0.0
                                }
                            ),
                        ]
                    },
                    Some(&format!(
                        "{} contributors, total {}",
                        contributors.len(),
                        Balance::from_rao(total).display_tao()
                    )),
                );
            }
            return Ok(());
        }
        _ => {} // Fall through to write commands that need wallet
    }

    // Write commands need wallet
    let mut wallet = open_wallet(wallet_dir, wallet_name)?;
    unlock_coldkey(&mut wallet, password)?;
    let pair = wallet.coldkey()?;
    let (action, hash) = match cmd {
        CrowdloanCommands::Create {
            deposit,
            min_contribution,
            cap,
            end_block,
            target,
        } => {
            let dep = Balance::from_tao(deposit);
            let min = Balance::from_tao(min_contribution);
            let cap_b = Balance::from_tao(cap);
            println!(
                "Creating crowdloan: deposit={}, min={}, cap={}, end_block={}",
                dep.display_tao(),
                min.display_tao(),
                cap_b.display_tao(),
                end_block
            );
            (
                "Crowdloan created",
                client
                    .crowdloan_create(
                        pair,
                        dep.rao(),
                        min.rao(),
                        cap_b.rao(),
                        end_block,
                        target.as_deref(),
                    )
                    .await?,
            )
        }
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
        CrowdloanCommands::Refund { crowdloan_id } => {
            println!("Refunding contributors of crowdloan #{}", crowdloan_id);
            (
                "Refund submitted",
                client.crowdloan_refund(pair, crowdloan_id).await?,
            )
        }
        CrowdloanCommands::Dissolve { crowdloan_id } => {
            println!("Dissolving crowdloan #{}", crowdloan_id);
            (
                "Crowdloan dissolved",
                client.crowdloan_dissolve(pair, crowdloan_id).await?,
            )
        }
        CrowdloanCommands::UpdateCap { crowdloan_id, cap } => {
            let cap_b = Balance::from_tao(cap);
            println!(
                "Updating cap of crowdloan #{} to {}",
                crowdloan_id,
                cap_b.display_tao()
            );
            (
                "Cap updated",
                client
                    .crowdloan_update_cap(pair, crowdloan_id, cap_b.rao())
                    .await?,
            )
        }
        CrowdloanCommands::UpdateEnd {
            crowdloan_id,
            end_block,
        } => {
            println!(
                "Updating end block of crowdloan #{} to {}",
                crowdloan_id, end_block
            );
            (
                "End block updated",
                client
                    .crowdloan_update_end(pair, crowdloan_id, end_block)
                    .await?,
            )
        }
        CrowdloanCommands::UpdateMinContribution {
            crowdloan_id,
            min_contribution,
        } => {
            let min = Balance::from_tao(min_contribution);
            println!(
                "Updating min contribution of crowdloan #{} to {}",
                crowdloan_id,
                min.display_tao()
            );
            (
                "Min contribution updated",
                client
                    .crowdloan_update_min_contribution(pair, crowdloan_id, min.rao())
                    .await?,
            )
        }
        // Read-only variants already handled above
        CrowdloanCommands::List
        | CrowdloanCommands::Info { .. }
        | CrowdloanCommands::Contributors { .. } => unreachable!(),
    };
    println!("{}. Tx: {}", action, hash);
    Ok(())
}

// ──────── Liquidity ────────

/// Convert a price (TAO per Alpha) to a Uniswap V3-style tick index.
/// tick = log(price) / log(1.0001), clamped to [-887272, 887272].
fn price_to_tick(price: f64) -> i32 {
    const MIN_TICK: i32 = -887272;
    const MAX_TICK: i32 = 887272;
    if price <= 0.0 {
        return MIN_TICK;
    }
    let tick = (price.ln() / 1.0001_f64.ln()) as i32;
    tick.clamp(MIN_TICK, MAX_TICK)
}

pub(super) async fn handle_liquidity(
    cmd: LiquidityCommands,
    client: &Client,
    ctx: &Ctx<'_>,
) -> Result<()> {
    let hotkey_name = ctx.hotkey_name;
    let mut wallet = open_wallet(ctx.wallet_dir, ctx.wallet_name)?;
    unlock_coldkey(&mut wallet, ctx.password)?;

    match cmd {
        LiquidityCommands::Add {
            netuid,
            price_low,
            price_high,
            amount,
            hotkey,
        } => {
            let hk = resolve_hotkey_ss58(hotkey, &mut wallet, hotkey_name)?;
            let pair = wallet.coldkey()?;
            let tick_low = price_to_tick(price_low);
            let tick_high = price_to_tick(price_high);
            if tick_low >= tick_high {
                anyhow::bail!(
                    "price_low ({:.6}) must be less than price_high ({:.6})",
                    price_low,
                    price_high
                );
            }
            println!(
                "Adding liquidity on SN{}: range [{:.6}, {:.6}] (ticks [{}, {}]), amount={} RAO",
                netuid, price_low, price_high, tick_low, tick_high, amount
            );
            let hash = client
                .add_liquidity(pair, &hk, NetUid(netuid), tick_low, tick_high, amount)
                .await?;
            println!("Liquidity added. Tx: {}", hash);
        }
        LiquidityCommands::Remove {
            netuid,
            position_id,
            hotkey,
        } => {
            let hk = resolve_hotkey_ss58(hotkey, &mut wallet, hotkey_name)?;
            let pair = wallet.coldkey()?;
            println!(
                "Removing liquidity position {} on SN{}",
                position_id, netuid
            );
            let hash = client
                .remove_liquidity(pair, &hk, NetUid(netuid), position_id)
                .await?;
            println!("Position removed. Tx: {}", hash);
        }
        LiquidityCommands::Modify {
            netuid,
            position_id,
            delta,
            hotkey,
        } => {
            let hk = resolve_hotkey_ss58(hotkey, &mut wallet, hotkey_name)?;
            let pair = wallet.coldkey()?;
            let action = if delta > 0 { "Adding" } else { "Removing" };
            println!(
                "{} {} RAO liquidity on position {} (SN{})",
                action,
                delta.unsigned_abs(),
                position_id,
                netuid
            );
            let hash = client
                .modify_liquidity(pair, &hk, NetUid(netuid), position_id, delta)
                .await?;
            println!("Position modified. Tx: {}", hash);
        }
        LiquidityCommands::Toggle { netuid, enable } => {
            let pair = wallet.coldkey()?;
            let action = if enable { "Enabling" } else { "Disabling" };
            println!(
                "{} user liquidity on SN{} (subnet owner only)",
                action, netuid
            );
            let hash = client
                .toggle_user_liquidity(pair, NetUid(netuid), enable)
                .await?;
            println!(
                "User liquidity {}. Tx: {}",
                if enable { "enabled" } else { "disabled" },
                hash
            );
        }
    }
    Ok(())
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
