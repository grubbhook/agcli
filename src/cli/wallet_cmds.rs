//! Wallet command handlers.

use crate::cli::WalletCommands;
use crate::wallet::Wallet;
use anyhow::Result;
use sp_core::Pair as _;

pub async fn handle_wallet(cmd: WalletCommands, wallet_dir: &str, global_password: Option<&str>) -> Result<()> {
    match cmd {
        WalletCommands::Create { name, password: cmd_password } => {
            let password = cmd_password
                .or_else(|| global_password.map(|s| s.to_string()))
                .map(Ok)
                .unwrap_or_else(|| {
                    dialoguer::Password::new()
                        .with_prompt("Set coldkey password")
                        .with_confirmation("Confirm password", "Passwords don't match")
                        .interact()
                        .map_err(anyhow::Error::from)
                })?;
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
            let wallets = Wallet::list_wallets(wallet_dir)?;
            for name in &wallets {
                let w = Wallet::open(&format!("{}/{}", wallet_dir, name));
                if let Ok(w) = w {
                    println!("Wallet: {}", name);
                    if let Some(addr) = w.coldkey_ss58() {
                        println!("  Coldkey: {}", addr);
                    }
                    if all {
                        if let Ok(hotkeys) = w.list_hotkeys() {
                            for hk_name in &hotkeys {
                                let mut w2 =
                                    Wallet::open(&format!("{}/{}", wallet_dir, name)).unwrap();
                                if w2.load_hotkey(hk_name).is_ok() {
                                    if let Some(hk_addr) = w2.hotkey_ss58() {
                                        println!("  Hotkey '{}': {}", hk_name, hk_addr);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Ok(())
        }
        WalletCommands::Import { name, mnemonic: cmd_mnemonic, password: cmd_password } => {
            let mnemonic = match cmd_mnemonic {
                Some(m) => m,
                None => dialoguer::Input::<String>::new()
                    .with_prompt("Enter mnemonic phrase")
                    .interact_text()?,
            };
            let password = cmd_password
                .or_else(|| global_password.map(|s| s.to_string()))
                .map(Ok)
                .unwrap_or_else(|| {
                    dialoguer::Password::new()
                        .with_prompt("Set password")
                        .with_confirmation("Confirm", "Mismatch")
                        .interact()
                        .map_err(anyhow::Error::from)
                })?;
            let wallet = Wallet::import_from_mnemonic(wallet_dir, &name, &mnemonic, &password)?;
            println!("Wallet '{}' imported.", name);
            if let Some(addr) = wallet.coldkey_ss58() {
                println!("Coldkey: {}", addr);
            }
            Ok(())
        }
        WalletCommands::RegenColdkey { mnemonic: cmd_mnemonic, password: cmd_password } => {
            println!("Regenerating coldkey from mnemonic...");
            let mnemonic = match cmd_mnemonic {
                Some(m) => m,
                None => dialoguer::Input::<String>::new()
                    .with_prompt("Enter mnemonic phrase")
                    .interact_text()?,
            };
            let password = cmd_password
                .or_else(|| global_password.map(|s| s.to_string()))
                .map(Ok)
                .unwrap_or_else(|| {
                    dialoguer::Password::new()
                        .with_prompt("Set password")
                        .with_confirmation("Confirm", "Mismatch")
                        .interact()
                        .map_err(anyhow::Error::from)
                })?;
            let wallet =
                Wallet::import_from_mnemonic(wallet_dir, "default", &mnemonic, &password)?;
            println!("Coldkey regenerated.");
            if let Some(addr) = wallet.coldkey_ss58() {
                println!("Coldkey: {}", addr);
            }
            Ok(())
        }
        WalletCommands::RegenHotkey { name, mnemonic: cmd_mnemonic } => {
            println!("Regenerating hotkey '{}' from mnemonic...", name);
            let mnemonic = match cmd_mnemonic {
                Some(m) => m,
                None => dialoguer::Input::<String>::new()
                    .with_prompt("Enter hotkey mnemonic phrase")
                    .interact_text()?,
            };
            let pair = crate::wallet::keypair::pair_from_mnemonic(&mnemonic)?;
            let ss58 = crate::wallet::keypair::to_ss58(&pair.public(), 42);
            let hotkey_path =
                std::path::PathBuf::from(wallet_dir).join("default").join("hotkeys").join(&name);
            std::fs::create_dir_all(hotkey_path.parent().unwrap())?;
            crate::wallet::keyfile::write_keyfile(&hotkey_path, &mnemonic)?;
            println!("Hotkey '{}' regenerated: {}", name, ss58);
            Ok(())
        }
        WalletCommands::NewHotkey { name } => {
            let (pair, mnemonic) = crate::wallet::keypair::generate_mnemonic_keypair()?;
            let ss58 = crate::wallet::keypair::to_ss58(&pair.public(), 42);
            let hotkey_path =
                std::path::PathBuf::from(wallet_dir).join("default").join("hotkeys").join(&name);
            std::fs::create_dir_all(hotkey_path.parent().unwrap())?;
            crate::wallet::keyfile::write_keyfile(&hotkey_path, &mnemonic)?;
            println!("New hotkey '{}' created: {}", name, ss58);
            Ok(())
        }
    }
}
