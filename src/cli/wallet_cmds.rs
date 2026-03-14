//! Wallet command handlers.

use crate::cli::WalletCommands;
use crate::wallet::Wallet;
use anyhow::Result;
use sp_core::Pair as _;

pub async fn handle_wallet(cmd: WalletCommands, wallet_dir: &str, wallet_name: &str, global_password: Option<&str>) -> Result<()> {
    match cmd {
        WalletCommands::Create { name, password: cmd_password } => {
            let password = cmd_password
                .or_else(|| global_password.map(|s| s.to_string()))
                .map(Ok)
                .unwrap_or_else(|| {
                    if crate::cli::helpers::is_batch_mode() {
                        return Err(anyhow::anyhow!("Password required in batch mode. Pass --password <pw> or set AGCLI_PASSWORD."));
                    }
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
            let all_wallets = Wallet::list_wallets(wallet_dir)?;
            // If a specific wallet was requested via -w/--wallet, filter to it
            let wallets: Vec<String> = if wallet_name != "default" && all_wallets.contains(&wallet_name.to_string()) {
                vec![wallet_name.to_string()]
            } else if wallet_name != "default" && !all_wallets.contains(&wallet_name.to_string()) {
                anyhow::bail!("Wallet '{}' not found in {}", wallet_name, wallet_dir);
            } else {
                all_wallets
            };
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
                None => {
                    if crate::cli::helpers::is_batch_mode() {
                        anyhow::bail!("Mnemonic required in batch mode. Pass --mnemonic <phrase>.");
                    }
                    dialoguer::Input::<String>::new()
                        .with_prompt("Enter mnemonic phrase")
                        .interact_text()?
                }
            };
            let password = cmd_password
                .or_else(|| global_password.map(|s| s.to_string()))
                .map(Ok)
                .unwrap_or_else(|| {
                    if crate::cli::helpers::is_batch_mode() {
                        return Err(anyhow::anyhow!("Password required in batch mode. Pass --password <pw> or set AGCLI_PASSWORD."));
                    }
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
                None => {
                    if crate::cli::helpers::is_batch_mode() {
                        anyhow::bail!("Mnemonic required in batch mode. Pass --mnemonic <phrase>.");
                    }
                    dialoguer::Input::<String>::new()
                        .with_prompt("Enter mnemonic phrase")
                        .interact_text()?
                }
            };
            let password = cmd_password
                .or_else(|| global_password.map(|s| s.to_string()))
                .map(Ok)
                .unwrap_or_else(|| {
                    if crate::cli::helpers::is_batch_mode() {
                        return Err(anyhow::anyhow!("Password required in batch mode. Pass --password <pw> or set AGCLI_PASSWORD."));
                    }
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
                None => {
                    if crate::cli::helpers::is_batch_mode() {
                        anyhow::bail!("Mnemonic required in batch mode. Pass --mnemonic <phrase>.");
                    }
                    dialoguer::Input::<String>::new()
                        .with_prompt("Enter hotkey mnemonic phrase")
                        .interact_text()?
                }
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
        WalletCommands::Sign { message } => {
            let mut wallet = crate::cli::helpers::open_wallet(wallet_dir, wallet_name)?;
            crate::cli::helpers::unlock_coldkey(&mut wallet, global_password)?;
            let pair = wallet.coldkey()?;
            let msg_bytes = if message.starts_with("0x") {
                hex::decode(message.strip_prefix("0x").unwrap())
                    .map_err(|e| anyhow::anyhow!("Invalid hex message: {}", e))?
            } else {
                message.as_bytes().to_vec()
            };
            let signature = pair.sign(&msg_bytes);
            println!("{}", serde_json::json!({
                "signer": wallet.coldkey_ss58().unwrap_or(""),
                "message": message,
                "signature": format!("0x{}", hex::encode(signature.0)),
            }));
            Ok(())
        }
        WalletCommands::Verify { message, signature, signer } => {
            let signer_ss58 = match signer {
                Some(s) => s,
                None => {
                    let wallet = crate::cli::helpers::open_wallet(wallet_dir, wallet_name)?;
                    wallet.coldkey_ss58().map(|s| s.to_string())
                        .ok_or_else(|| anyhow::anyhow!("No coldkey found. Pass --signer <ss58>."))?
                }
            };
            let msg_bytes = if message.starts_with("0x") {
                hex::decode(message.strip_prefix("0x").unwrap())
                    .map_err(|e| anyhow::anyhow!("Invalid hex message: {}", e))?
            } else {
                message.as_bytes().to_vec()
            };
            let sig_hex = signature.strip_prefix("0x").unwrap_or(&signature);
            let sig_bytes = hex::decode(sig_hex)
                .map_err(|e| anyhow::anyhow!("Invalid hex signature: {}", e))?;
            if sig_bytes.len() != 64 {
                anyhow::bail!("Signature must be 64 bytes (128 hex chars), got {}", sig_bytes.len());
            }
            let public = crate::wallet::keypair::from_ss58(&signer_ss58)?;
            let sig = sp_core::sr25519::Signature::from_raw(sig_bytes.try_into().unwrap());
            let valid = sp_core::sr25519::Pair::verify(&sig, &msg_bytes, &public);
            println!("{}", serde_json::json!({
                "signer": signer_ss58,
                "valid": valid,
            }));
            if !valid {
                std::process::exit(1);
            }
            Ok(())
        }
        WalletCommands::Derive { input } => {
            if input.starts_with("0x") {
                // Public key hex
                let bytes = hex::decode(input.strip_prefix("0x").unwrap())
                    .map_err(|e| anyhow::anyhow!("Invalid hex: {}", e))?;
                if bytes.len() != 32 {
                    anyhow::bail!("Public key must be 32 bytes, got {}", bytes.len());
                }
                let public = sp_core::sr25519::Public::from_raw(bytes.try_into().unwrap());
                let ss58 = crate::wallet::keypair::to_ss58(&public, 42);
                println!("{}", serde_json::json!({
                    "public_key": format!("0x{}", hex::encode(public.0)),
                    "ss58": ss58,
                }));
            } else {
                // Mnemonic phrase — derive public key only (never print secret)
                let pair = crate::wallet::keypair::pair_from_mnemonic(&input)?;
                let ss58 = crate::wallet::keypair::to_ss58(&pair.public(), 42);
                println!("{}", serde_json::json!({
                    "public_key": format!("0x{}", hex::encode(pair.public().0)),
                    "ss58": ss58,
                }));
            }
            Ok(())
        }
    }
}
