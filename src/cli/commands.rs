//! CLI command execution — thin dispatcher to focused handler modules.

use crate::chain::Client;
use crate::cli::helpers::*;
use crate::cli::*;
use crate::types::Balance;
use anyhow::Result;

/// Connect to the network and apply dry-run flag.
async fn connect(network: &crate::types::Network, dry_run: bool) -> Result<Client> {
    let mut client = Client::connect_network(network).await?;
    client.set_dry_run(dry_run);
    Ok(client)
}

/// Execute the parsed CLI command.
pub async fn execute(cli: Cli) -> Result<()> {
    let network = cli.resolve_network();
    let password = cli.password.clone();
    let batch = cli.batch;
    let dry_run = cli.dry_run;

    // Set global mode flags so helpers can check them
    set_batch_mode(batch || cli.yes);
    set_pretty_mode(cli.pretty);

    // Build shared command context — eliminates 6-9 repeated parameters across handlers
    let ctx = Ctx {
        wallet_dir: &cli.wallet_dir,
        wallet_name: &cli.wallet,
        hotkey_name: &cli.hotkey,
        output: cli.output,
        password: password.as_deref(),
        yes: cli.yes,
        mev: cli.mev,
        live_interval: cli.live_interval(),
    };

    match cli.command {
        Commands::Wallet(WalletCommands::AssociateHotkey { hotkey }) => {
            let client = connect(&network, dry_run).await?;
            let (pair, hk) = unlock_and_resolve(
                ctx.wallet_dir, ctx.wallet_name, ctx.hotkey_name,
                hotkey, ctx.password,
            )?;
            println!("Associating hotkey {} on-chain", crate::utils::short_ss58(&hk));
            let hash = client.try_associate_hotkey(&pair, &hk).await?;
            print_tx_result(ctx.output, &hash, "Hotkey associated.");
            Ok(())
        }
        Commands::Wallet(WalletCommands::CheckSwap { address }) => {
            let client = connect(&network, dry_run).await?;
            let addr = resolve_coldkey_address(address, ctx.wallet_dir, ctx.wallet_name);
            let swap = client.get_coldkey_swap_scheduled(&addr).await?;
            match swap {
                Some((block, new_ss58)) => {
                    if ctx.output.is_json() {
                        print_json(&serde_json::json!({
                            "address": addr,
                            "swap_scheduled": true,
                            "execution_block": block,
                            "new_coldkey": new_ss58,
                        }));
                    } else {
                        println!("Coldkey swap scheduled for {}", addr);
                        println!("  Execution block: {}", block);
                        println!("  New coldkey:     {}", new_ss58);
                    }
                }
                None => {
                    if ctx.output.is_json() {
                        print_json(&serde_json::json!({
                            "address": addr,
                            "swap_scheduled": false,
                        }));
                    } else {
                        println!("No coldkey swap scheduled for {}", addr);
                    }
                }
            }
            Ok(())
        }
        Commands::Wallet(cmd) => {
            wallet_cmds::handle_wallet(
                cmd,
                ctx.wallet_dir,
                ctx.wallet_name,
                ctx.password,
                ctx.output,
            )
            .await
        }
        Commands::Balance {
            address,
            watch,
            threshold,
            at_block,
        } => {
            let client = connect(&network, dry_run).await?;
            let addr = resolve_coldkey_address(address, ctx.wallet_dir, ctx.wallet_name);

            // Historical wayback mode
            if let Some(block_num) = at_block {
                let block_hash = client.get_block_hash(block_num).await?;
                let balance = client.get_balance_at_block(&addr, block_hash).await?;
                if ctx.output.is_json() {
                    print_json(
                        &serde_json::json!({"address": addr, "block": block_num, "block_hash": format!("{:?}", block_hash), "balance_rao": balance.rao(), "balance_tao": balance.tao()}),
                    );
                } else {
                    println!("Address: {}", addr);
                    println!("Block:   {} ({:?})", block_num, block_hash);
                    println!("Balance: {}", balance.display_tao());
                }
                return Ok(());
            }

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
                tracing::info!(
                    address = %crate::utils::short_ss58(&addr),
                    interval_secs = interval,
                    "Starting balance watch mode"
                );
                loop {
                    let balance = match client.get_balance_ss58(&addr).await {
                        Ok(b) => b,
                        Err(e) => {
                            eprintln!("[{}] Warning: balance query failed: {}", chrono::Local::now().format("%H:%M:%S"), e);
                            tracing::warn!(error = %e, "Balance query failed during watch mode");
                            tokio::select! {
                                _ = tokio::time::sleep(tokio::time::Duration::from_secs(interval)) => {},
                                _ = tokio::signal::ctrl_c() => {
                                    println!("\nStopping balance watch (received Ctrl+C)");
                                    return Ok(());
                                }
                            }
                            continue;
                        }
                    };
                    let below = threshold_rao
                        .as_ref()
                        .map(|t| balance.rao() < t.rao())
                        .unwrap_or(false);
                    if ctx.output.is_json() {
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
                    tokio::select! {
                        _ = tokio::time::sleep(tokio::time::Duration::from_secs(interval)) => {},
                        _ = tokio::signal::ctrl_c() => {
                            println!("\nStopping balance watch (received Ctrl+C)");
                            return Ok(());
                        }
                    }
                }
            }

            let balance = client.get_balance_ss58(&addr).await?;
            if ctx.output.is_json() {
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
            let client = connect(&network, dry_run).await?;
            let mut wallet = open_wallet(ctx.wallet_dir, ctx.wallet_name)?;
            unlock_coldkey(&mut wallet, ctx.password)?;
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
            println!("Transferring {} to {}", balance.display_tao(), crate::utils::short_ss58(&dest));
            if !is_batch_mode() {
                let proceed = dialoguer::Confirm::new()
                    .with_prompt("Proceed?")
                    .default(true)
                    .interact()?;
                if !proceed {
                    println!("Cancelled.");
                    return Ok(());
                }
            }
            let hash = client.transfer(wallet.coldkey()?, &dest, balance).await?;
            print_tx_result(ctx.output, &hash, "Transaction submitted.");
            Ok(())
        }
        Commands::TransferAll { dest, keep_alive } => {
            let client = connect(&network, dry_run).await?;
            let mut wallet = open_wallet(ctx.wallet_dir, ctx.wallet_name)?;
            unlock_coldkey(&mut wallet, ctx.password)?;
            println!(
                "Transferring all balance to {} (keep_alive={})",
                crate::utils::short_ss58(&dest), keep_alive
            );
            if !is_batch_mode() {
                let proceed = dialoguer::Confirm::new()
                    .with_prompt("Transfer ALL funds? This will empty your account.")
                    .default(false)
                    .interact()?;
                if !proceed {
                    println!("Cancelled.");
                    return Ok(());
                }
            }
            let hash = client
                .transfer_all(wallet.coldkey()?, &dest, keep_alive)
                .await?;
            print_tx_result(ctx.output, &hash, "All balance transferred.");
            Ok(())
        }
        Commands::Stake(cmd) => {
            let client = connect(&network, dry_run).await?;
            stake_cmds::handle_stake(cmd, &client, &ctx).await
        }
        Commands::Subnet(cmd) => {
            let client = connect(&network, dry_run).await?;
            subnet_cmds::handle_subnet(cmd, &client, &ctx).await
        }
        Commands::Weights(cmd) => {
            let client = connect(&network, dry_run).await?;
            weights_cmds::handle_weights(cmd, &client, &ctx).await
        }
        Commands::Root(cmd) => {
            let client = connect(&network, dry_run).await?;
            network_cmds::handle_root(cmd, &client, &ctx).await
        }
        Commands::Delegate(cmd) => {
            let client = connect(&network, dry_run).await?;
            network_cmds::handle_delegate(cmd, &client, &ctx).await
        }
        Commands::View(cmd) => {
            let client = connect(&network, dry_run).await?;
            view_cmds::handle_view(cmd, &client, &ctx).await
        }
        Commands::Identity(cmd) => {
            let client = connect(&network, dry_run).await?;
            network_cmds::handle_identity(cmd, &client, &ctx).await
        }
        Commands::Serve(cmd) => {
            let client = connect(&network, dry_run).await?;
            network_cmds::handle_serve(cmd, &client, &ctx).await
        }
        Commands::Proxy(cmd) => {
            let client = connect(&network, dry_run).await?;
            network_cmds::handle_proxy(cmd, &client, &ctx).await
        }
        Commands::Crowdloan(cmd) => {
            let client = connect(&network, dry_run).await?;
            network_cmds::handle_crowdloan(cmd, &client, &ctx).await
        }
        Commands::Liquidity(cmd) => {
            let client = connect(&network, dry_run).await?;
            network_cmds::handle_liquidity(cmd, &client, &ctx).await
        }
        Commands::Swap(cmd) => {
            let client = connect(&network, dry_run).await?;
            network_cmds::handle_swap(cmd, &client, &ctx).await
        }
        Commands::Subscribe(cmd) => {
            let client = connect(&network, dry_run).await?;
            network_cmds::handle_subscribe(cmd, &client, ctx.output, batch).await
        }
        Commands::Multisig(cmd) => {
            network_cmds::handle_multisig(
                cmd,
                ctx.wallet_dir,
                ctx.wallet_name,
                &network,
                ctx.password,
                dry_run,
            )
            .await
        }
        Commands::Utils(cmd) => {
            system_cmds::handle_utils(cmd, &network, ctx.output).await
        }
        Commands::Config(cmd) => system_cmds::handle_config(cmd).await,
        Commands::Completions { shell } => {
            system_cmds::generate_completions(&shell);
            Ok(())
        }
        Commands::Update => system_cmds::handle_update().await,
        Commands::Doctor => {
            system_cmds::handle_doctor(&network, ctx.wallet_dir, ctx.wallet_name, ctx.output).await
        }
        Commands::Explain { topic } => {
            system_cmds::handle_explain(topic.as_deref(), ctx.output)
        }
        Commands::Audit { address } => {
            let client = connect(&network, dry_run).await?;
            let addr = resolve_coldkey_address(address, ctx.wallet_dir, ctx.wallet_name);
            view_cmds::handle_audit(&client, &addr, ctx.output).await
        }
        Commands::Block(cmd) => {
            let client = connect(&network, dry_run).await?;
            block_cmds::handle_block(cmd, &client, ctx.output).await
        }
        Commands::Diff(cmd) => {
            let client = connect(&network, dry_run).await?;
            block_cmds::handle_diff(cmd, &client, ctx.output, ctx.wallet_dir, ctx.wallet_name).await
        }
        Commands::Batch { file, no_atomic } => {
            let client = connect(&network, dry_run).await?;
            let mut wallet = open_wallet(ctx.wallet_dir, ctx.wallet_name)?;
            unlock_coldkey(&mut wallet, ctx.password)?;
            system_cmds::handle_batch(&client, wallet.coldkey()?, &file, no_atomic, ctx.output).await
        }
    }
}
