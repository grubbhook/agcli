//! CLI command execution handlers.

use crate::cli::*;
use crate::chain::Client;
use crate::wallet::Wallet;
use crate::types::Balance;
use anyhow::Result;

/// Execute the parsed CLI command.
pub async fn execute(cli: Cli) -> Result<()> {
    let network = cli.resolve_network();

    match cli.command {
        Commands::Wallet(cmd) => handle_wallet(cmd, &cli.wallet_dir).await,
        Commands::Balance { address } => {
            let client = Client::connect(network.ws_url()).await?;
            let addr = address.unwrap_or_else(|| {
                let w = Wallet::open(&format!("{}/{}", cli.wallet_dir, cli.wallet)).ok();
                w.and_then(|w| w.coldkey_ss58().map(|s| s.to_string()))
                    .unwrap_or_default()
            });
            let balance = client.get_balance_ss58(&addr).await?;
            println!("Address: {}", addr);
            println!("Balance: {}", balance.display_tao());
            Ok(())
        }
        Commands::Transfer { dest, amount } => {
            let client = Client::connect(network.ws_url()).await?;
            let mut wallet = Wallet::open(&format!("{}/{}", cli.wallet_dir, cli.wallet))?;
            let password = dialoguer::Password::new()
                .with_prompt("Coldkey password")
                .interact()?;
            wallet.unlock_coldkey(&password)?;
            let balance = Balance::from_tao(amount);
            println!("Transferring {} to {}", balance.display_tao(), dest);
            let hash = client.transfer(wallet.coldkey()?, &dest, balance).await?;
            println!("Transaction submitted: {}", hash);
            Ok(())
        }
        Commands::Stake(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            handle_stake(cmd, &client, &cli.wallet_dir, &cli.wallet, &cli.hotkey).await
        }
        Commands::Subnet(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            handle_subnet(cmd, &client).await
        }
        Commands::Weights(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            handle_weights(cmd, &client, &cli.wallet_dir, &cli.wallet, &cli.hotkey).await
        }
        Commands::Root(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            handle_root(cmd, &client, &cli.wallet_dir, &cli.wallet, &cli.hotkey).await
        }
        Commands::Delegate(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            handle_delegate(cmd, &client).await
        }
        Commands::View(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            handle_view(cmd, &client, &cli.wallet_dir, &cli.wallet).await
        }
        Commands::Identity(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            handle_identity(cmd, &client, &cli.wallet_dir, &cli.wallet).await
        }
        Commands::Swap(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            handle_swap(cmd, &client, &cli.wallet_dir, &cli.wallet).await
        }
    }
}

async fn handle_wallet(cmd: WalletCommands, wallet_dir: &str) -> Result<()> {
    match cmd {
        WalletCommands::Create { name } => {
            let password = dialoguer::Password::new()
                .with_prompt("Set coldkey password")
                .with_confirmation("Confirm password", "Passwords don't match")
                .interact()?;
            let wallet = Wallet::create(wallet_dir, &name, &password, "default")?;
            println!("Wallet '{}' created.", name);
            if let Some(addr) = wallet.coldkey_ss58() {
                println!("Coldkey: {}", addr);
            }
            if let Some(addr) = wallet.hotkey_ss58() {
                println!("Hotkey:  {}", addr);
            }
            Ok(())
        }
        WalletCommands::List => {
            let wallets = Wallet::list_wallets(wallet_dir)?;
            if wallets.is_empty() {
                println!("No wallets found in {}", wallet_dir);
            } else {
                println!("Wallets in {}:", wallet_dir);
                for name in wallets {
                    let w = Wallet::open(&format!("{}/{}", wallet_dir, name)).ok();
                    let addr = w
                        .as_ref()
                        .and_then(|w| w.coldkey_ss58().map(|s| s.to_string()))
                        .unwrap_or_else(|| "?".to_string());
                    println!("  {} ({})", name, crate::utils::short_ss58(&addr));
                }
            }
            Ok(())
        }
        WalletCommands::Show { all } => {
            println!("TODO: show wallet details (all={})", all);
            Ok(())
        }
        WalletCommands::Import { name } => {
            let mnemonic = dialoguer::Input::<String>::new()
                .with_prompt("Enter mnemonic phrase")
                .interact_text()?;
            let password = dialoguer::Password::new()
                .with_prompt("Set password")
                .with_confirmation("Confirm", "Mismatch")
                .interact()?;
            let wallet = Wallet::import_from_mnemonic(wallet_dir, &name, &mnemonic, &password)?;
            println!("Wallet '{}' imported.", name);
            if let Some(addr) = wallet.coldkey_ss58() {
                println!("Coldkey: {}", addr);
            }
            Ok(())
        }
        _ => {
            println!("Command not yet implemented");
            Ok(())
        }
    }
}

async fn handle_stake(
    cmd: StakeCommands,
    client: &Client,
    wallet_dir: &str,
    wallet_name: &str,
    _hotkey_name: &str,
) -> Result<()> {
    match cmd {
        StakeCommands::List { address } => {
            let addr = address.unwrap_or_else(|| {
                Wallet::open(&format!("{}/{}", wallet_dir, wallet_name))
                    .ok()
                    .and_then(|w| w.coldkey_ss58().map(|s| s.to_string()))
                    .unwrap_or_default()
            });
            let stakes = client.get_stake_for_coldkey(&addr).await?;
            if stakes.is_empty() {
                println!("No stakes found for {}", crate::utils::short_ss58(&addr));
            } else {
                println!("Stakes for {}:", crate::utils::short_ss58(&addr));
                let mut table = comfy_table::Table::new();
                table.set_header(vec!["Subnet", "Hotkey", "Stake (τ)"]);
                for s in &stakes {
                    table.add_row(vec![
                        format!("SN{}", s.netuid),
                        crate::utils::short_ss58(&s.hotkey),
                        s.stake.display_tao(),
                    ]);
                }
                println!("{table}");
            }
            Ok(())
        }
        _ => {
            println!("Stake command not yet fully implemented");
            Ok(())
        }
    }
}

async fn handle_subnet(cmd: SubnetCommands, client: &Client) -> Result<()> {
    match cmd {
        SubnetCommands::List => {
            let subnets = client.get_all_subnets().await?;
            if subnets.is_empty() {
                println!("No subnets found.");
            } else {
                let mut table = comfy_table::Table::new();
                table.set_header(vec!["NetUID", "N", "Max", "Tempo", "Emission", "Burn", "Owner"]);
                for s in &subnets {
                    table.add_row(vec![
                        format!("{}", s.netuid),
                        format!("{}", s.n),
                        format!("{}", s.max_n),
                        format!("{}", s.tempo),
                        format!("{}", s.emission_value),
                        s.burn.display_tao(),
                        crate::utils::short_ss58(&s.owner),
                    ]);
                }
                println!("{table}");
            }
            Ok(())
        }
        SubnetCommands::Metagraph { netuid } => {
            let mg = crate::queries::fetch_metagraph(client, netuid.into()).await?;
            println!("Metagraph SN{} — {} neurons, block {}", netuid, mg.n, mg.block);
            let mut table = comfy_table::Table::new();
            table.set_header(vec!["UID", "Hotkey", "Stake", "Rank", "Trust", "Incentive", "Emission", "VP"]);
            for n in &mg.neurons {
                table.add_row(vec![
                    format!("{}", n.uid),
                    crate::utils::short_ss58(&n.hotkey),
                    format!("{:.4}τ", n.stake.tao()),
                    format!("{:.4}", n.rank),
                    format!("{:.4}", n.trust),
                    format!("{:.4}", n.incentive),
                    format!("{:.0}", n.emission),
                    if n.validator_permit { "✓" } else { "" }.to_string(),
                ]);
            }
            println!("{table}");
            Ok(())
        }
        _ => {
            println!("Subnet command not yet fully implemented");
            Ok(())
        }
    }
}

async fn handle_weights(
    _cmd: WeightCommands,
    _client: &Client,
    _wallet_dir: &str,
    _wallet_name: &str,
    _hotkey_name: &str,
) -> Result<()> {
    println!("Weight commands not yet fully implemented");
    Ok(())
}

async fn handle_root(
    _cmd: RootCommands,
    _client: &Client,
    _wallet_dir: &str,
    _wallet_name: &str,
    _hotkey_name: &str,
) -> Result<()> {
    println!("Root commands not yet fully implemented");
    Ok(())
}

async fn handle_delegate(cmd: DelegateCommands, client: &Client) -> Result<()> {
    match cmd {
        DelegateCommands::List => {
            let delegates = client.get_delegates().await?;
            println!("{} delegates", delegates.len());
            let mut table = comfy_table::Table::new();
            table.set_header(vec!["Hotkey", "Owner", "Take", "Total Stake", "Nominators"]);
            for d in delegates.iter().take(20) {
                table.add_row(vec![
                    crate::utils::short_ss58(&d.hotkey),
                    crate::utils::short_ss58(&d.owner),
                    format!("{:.2}%", d.take * 100.0),
                    d.total_stake.display_tao(),
                    format!("{}", d.nominators.len()),
                ]);
            }
            println!("{table}");
            Ok(())
        }
        _ => {
            println!("Delegate command not yet fully implemented");
            Ok(())
        }
    }
}

async fn handle_view(
    cmd: ViewCommands,
    client: &Client,
    wallet_dir: &str,
    wallet_name: &str,
) -> Result<()> {
    match cmd {
        ViewCommands::Portfolio { address } => {
            let addr = address.unwrap_or_else(|| {
                Wallet::open(&format!("{}/{}", wallet_dir, wallet_name))
                    .ok()
                    .and_then(|w| w.coldkey_ss58().map(|s| s.to_string()))
                    .unwrap_or_default()
            });
            let portfolio = crate::queries::portfolio::fetch_portfolio(client, &addr).await?;
            println!("Portfolio for {}", crate::utils::short_ss58(&addr));
            println!("  Free: {}", portfolio.free_balance.display_tao());
            println!("  Staked: {}", portfolio.total_staked.display_tao());
            for p in &portfolio.positions {
                println!(
                    "    SN{}: {} α = {} on {}",
                    p.netuid,
                    p.alpha_stake,
                    p.tao_equivalent,
                    crate::utils::short_ss58(&p.hotkey_ss58)
                );
            }
            Ok(())
        }
        ViewCommands::Network => {
            let block = client.get_block_number().await?;
            let total_stake = client.get_total_stake().await?;
            let total_networks = client.get_total_networks().await?;
            let total_issuance = client.get_total_issuance().await?;
            let emission = client.get_block_emission().await?;
            println!("Network Overview");
            println!("  Block:        {}", block);
            println!("  Subnets:      {}", total_networks);
            println!("  Total issued: {}", total_issuance.display_tao());
            println!("  Total staked: {}", total_stake.display_tao());
            println!("  Emission/blk: {}", emission.display_tao());
            let staking_ratio = if total_issuance.rao() > 0 {
                total_stake.tao() / total_issuance.tao() * 100.0
            } else {
                0.0
            };
            println!("  Staking ratio: {:.1}%", staking_ratio);
            Ok(())
        }
        _ => {
            println!("View command not yet fully implemented");
            Ok(())
        }
    }
}

async fn handle_identity(
    _cmd: IdentityCommands,
    _client: &Client,
    _wallet_dir: &str,
    _wallet_name: &str,
) -> Result<()> {
    println!("Identity commands not yet fully implemented");
    Ok(())
}

async fn handle_swap(
    _cmd: SwapCommands,
    _client: &Client,
    _wallet_dir: &str,
    _wallet_name: &str,
) -> Result<()> {
    println!("Swap commands not yet fully implemented");
    Ok(())
}
