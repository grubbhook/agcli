//! Wallet command handlers.

use crate::cli::WalletCommands;
use crate::wallet::Wallet;
use anyhow::Result;
use sp_core::Pair as _;

pub async fn handle_wallet(
    cmd: WalletCommands,
    wallet_dir: &str,
    wallet_name: &str,
    global_password: Option<&str>,
    output: &str,
) -> Result<()> {
    match cmd {
        WalletCommands::Create {
            name,
            password: cmd_password,
        } => {
            let password =
                crate::cli::helpers::require_password(cmd_password, global_password, true)?;
            let wallet = Wallet::create(wallet_dir, &name, &password, "default")?;
            if output == "json" {
                crate::cli::helpers::print_json(&serde_json::json!({
                    "name": name,
                    "coldkey": wallet.coldkey_ss58().unwrap_or(""),
                    "hotkey": wallet.hotkey_ss58().unwrap_or(""),
                }));
            } else {
                println!("Wallet '{}' created.", name);
                if let Some(addr) = wallet.coldkey_ss58() {
                    println!("Coldkey: {}", addr);
                }
                if let Some(addr) = wallet.hotkey_ss58() {
                    println!("Hotkey:  {}", addr);
                }
            }
            Ok(())
        }
        WalletCommands::List => {
            let wallets = Wallet::list_wallets(wallet_dir)?;
            // Collect name+address pairs once
            let entries: Vec<(String, String)> = wallets
                .iter()
                .map(|name| {
                    let addr = Wallet::open(format!("{}/{}", wallet_dir, name))
                        .ok()
                        .and_then(|w| w.coldkey_ss58().map(|s| s.to_string()))
                        .unwrap_or_default();
                    (name.clone(), addr)
                })
                .collect();
            if output == "json" {
                let items: Vec<serde_json::Value> = entries
                    .iter()
                    .map(|(n, a)| serde_json::json!({"name": n, "coldkey": a}))
                    .collect();
                crate::cli::helpers::print_json(&serde_json::json!(items));
            } else if output == "csv" {
                println!("name,coldkey");
                for (name, addr) in &entries {
                    println!("{},{}", crate::cli::helpers::csv_escape(name), addr);
                }
            } else if entries.is_empty() {
                println!("No wallets found in {}", wallet_dir);
            } else {
                println!("Wallets in {}:", wallet_dir);
                for (name, addr) in &entries {
                    println!("  {} ({})", name, crate::utils::short_ss58(addr));
                }
            }
            Ok(())
        }
        WalletCommands::Show { all } => {
            let all_wallets = Wallet::list_wallets(wallet_dir)?;
            // If a specific wallet was requested via -w/--wallet, filter to it
            let wallets: Vec<String> = if wallet_name != "default"
                && all_wallets.contains(&wallet_name.to_string())
            {
                vec![wallet_name.to_string()]
            } else if wallet_name != "default" && !all_wallets.contains(&wallet_name.to_string()) {
                anyhow::bail!("Wallet '{}' not found in {}", wallet_name, wallet_dir);
            } else {
                all_wallets
            };
            // Collect wallet data once, then format for the chosen output
            struct WalletEntry {
                name: String,
                coldkey: String,
                hotkeys: Vec<(String, String)>, // (name, address)
            }
            let mut entries: Vec<WalletEntry> = Vec::new();
            for name in &wallets {
                if let Ok(w) = Wallet::open(format!("{}/{}", wallet_dir, name)) {
                    let coldkey = w.coldkey_ss58().map(|s| s.to_string()).unwrap_or_default();
                    let mut hotkeys = Vec::new();
                    if all {
                        if let Ok(hk_names) = w.list_hotkeys() {
                            for hk_name in &hk_names {
                                let mut w2 =
                                    match Wallet::open(format!("{}/{}", wallet_dir, name)) {
                                    Ok(w) => w,
                                    Err(_) => continue,
                                };
                                if w2.load_hotkey(hk_name).is_ok() {
                                    if let Some(hk_addr) = w2.hotkey_ss58() {
                                        hotkeys.push((hk_name.clone(), hk_addr.to_string()));
                                    }
                                }
                            }
                        }
                    }
                    entries.push(WalletEntry {
                        name: name.clone(),
                        coldkey,
                        hotkeys,
                    });
                }
            }
            if output == "json" {
                let items: Vec<serde_json::Value> = entries
                    .iter()
                    .map(|e| {
                        let mut obj = serde_json::json!({"name": e.name, "coldkey": e.coldkey});
                        if all {
                            obj["hotkeys"] = serde_json::json!(e
                                .hotkeys
                                .iter()
                                .map(|(n, a)| serde_json::json!({"name": n, "address": a}))
                                .collect::<Vec<_>>());
                        }
                        obj
                    })
                    .collect();
                crate::cli::helpers::print_json(&serde_json::json!(items));
            } else if output == "csv" {
                if all {
                    println!("wallet,coldkey,hotkey_name,hotkey_address");
                    for e in &entries {
                        let esc_name = crate::cli::helpers::csv_escape(&e.name);
                        if e.hotkeys.is_empty() {
                            println!("{},{},,", esc_name, e.coldkey);
                        } else {
                            for (hk_name, hk_addr) in &e.hotkeys {
                                println!("{},{},{},{}", esc_name, e.coldkey,
                                    crate::cli::helpers::csv_escape(hk_name), hk_addr);
                            }
                        }
                    }
                } else {
                    println!("name,coldkey");
                    for e in &entries {
                        println!("{},{}", crate::cli::helpers::csv_escape(&e.name), e.coldkey);
                    }
                }
            } else {
                for e in &entries {
                    println!("Wallet: {}", e.name);
                    println!("  Coldkey: {}", e.coldkey);
                    for (hk_name, hk_addr) in &e.hotkeys {
                        println!("  Hotkey '{}': {}", hk_name, hk_addr);
                    }
                }
            }
            Ok(())
        }
        WalletCommands::Import {
            name,
            mnemonic: cmd_mnemonic,
            password: cmd_password,
        } => {
            let mnemonic = crate::cli::helpers::require_mnemonic(cmd_mnemonic)?;
            let password =
                crate::cli::helpers::require_password(cmd_password, global_password, true)?;
            let wallet = Wallet::import_from_mnemonic(wallet_dir, &name, &mnemonic, &password)?;
            if output == "json" {
                crate::cli::helpers::print_json(&serde_json::json!({
                    "name": name,
                    "coldkey": wallet.coldkey_ss58().unwrap_or(""),
                }));
            } else {
                println!("Wallet '{}' imported.", name);
                if let Some(addr) = wallet.coldkey_ss58() {
                    println!("Coldkey: {}", addr);
                }
            }
            Ok(())
        }
        WalletCommands::RegenColdkey {
            mnemonic: cmd_mnemonic,
            password: cmd_password,
        } => {
            let mnemonic = crate::cli::helpers::require_mnemonic(cmd_mnemonic)?;
            let password =
                crate::cli::helpers::require_password(cmd_password, global_password, true)?;
            let wallet = Wallet::import_from_mnemonic(wallet_dir, "default", &mnemonic, &password)?;
            if output == "json" {
                crate::cli::helpers::print_json(&serde_json::json!({
                    "coldkey": wallet.coldkey_ss58().unwrap_or(""),
                }));
            } else {
                println!("Coldkey regenerated.");
                if let Some(addr) = wallet.coldkey_ss58() {
                    println!("Coldkey: {}", addr);
                }
            }
            Ok(())
        }
        WalletCommands::RegenHotkey {
            name,
            mnemonic: cmd_mnemonic,
        } => {
            let mnemonic = crate::cli::helpers::require_mnemonic(cmd_mnemonic)?;
            let pair = crate::wallet::keypair::pair_from_mnemonic(&mnemonic)?;
            let ss58 = crate::wallet::keypair::to_ss58(&pair.public(), 42);
            let hotkey_path = std::path::PathBuf::from(wallet_dir)
                .join("default")
                .join("hotkeys")
                .join(&name);
            if let Some(parent) = hotkey_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            crate::wallet::keyfile::write_keyfile(&hotkey_path, &mnemonic)?;
            if output == "json" {
                crate::cli::helpers::print_json(&serde_json::json!({
                    "name": name,
                    "hotkey": ss58,
                }));
            } else {
                println!("Hotkey '{}' regenerated: {}", name, ss58);
            }
            Ok(())
        }
        WalletCommands::NewHotkey { name } => {
            let (pair, mnemonic) = crate::wallet::keypair::generate_mnemonic_keypair()?;
            let ss58 = crate::wallet::keypair::to_ss58(&pair.public(), 42);
            let hotkey_path = std::path::PathBuf::from(wallet_dir)
                .join("default")
                .join("hotkeys")
                .join(&name);
            if let Some(parent) = hotkey_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            crate::wallet::keyfile::write_keyfile(&hotkey_path, &mnemonic)?;
            if output == "json" {
                crate::cli::helpers::print_json(&serde_json::json!({
                    "name": name,
                    "hotkey": ss58,
                }));
            } else {
                println!("New hotkey '{}' created: {}", name, ss58);
            }
            Ok(())
        }
        WalletCommands::Sign { message } => {
            let mut wallet = crate::cli::helpers::open_wallet(wallet_dir, wallet_name)?;
            crate::cli::helpers::unlock_coldkey(&mut wallet, global_password)?;
            let pair = wallet.coldkey()?;
            let msg_bytes = if let Some(hex_str) = message.strip_prefix("0x") {
                hex::decode(hex_str)
                    .map_err(|e| anyhow::anyhow!("Invalid hex message: {}", e))?
            } else {
                message.as_bytes().to_vec()
            };
            let signature = pair.sign(&msg_bytes);
            crate::cli::helpers::print_json(&serde_json::json!({
                "signer": wallet.coldkey_ss58().unwrap_or(""),
                "message": message,
                "signature": format!("0x{}", hex::encode(signature.0)),
            }));
            Ok(())
        }
        WalletCommands::Verify {
            message,
            signature,
            signer,
        } => {
            let signer_ss58 = match signer {
                Some(s) => s,
                None => {
                    let wallet = crate::cli::helpers::open_wallet(wallet_dir, wallet_name)?;
                    wallet
                        .coldkey_ss58()
                        .map(|s| s.to_string())
                        .ok_or_else(|| anyhow::anyhow!("No coldkey found. Pass --signer <ss58>."))?
                }
            };
            let msg_bytes = if let Some(hex_str) = message.strip_prefix("0x") {
                hex::decode(hex_str)
                    .map_err(|e| anyhow::anyhow!("Invalid hex message: {}", e))?
            } else {
                message.as_bytes().to_vec()
            };
            let sig_hex = signature.strip_prefix("0x").unwrap_or(&signature);
            let sig_bytes = hex::decode(sig_hex)
                .map_err(|e| anyhow::anyhow!("Invalid hex signature: {}", e))?;
            if sig_bytes.len() != 64 {
                anyhow::bail!(
                    "Signature must be 64 bytes (128 hex chars), got {}",
                    sig_bytes.len()
                );
            }
            let public = crate::wallet::keypair::from_ss58(&signer_ss58)?;
            let sig_arr: [u8; 64] = sig_bytes.try_into()
                .map_err(|_| anyhow::anyhow!("Signature bytes conversion failed"))?;
            let sig = sp_core::sr25519::Signature::from_raw(sig_arr);
            let valid = sp_core::sr25519::Pair::verify(&sig, &msg_bytes, &public);
            crate::cli::helpers::print_json(&serde_json::json!({
                "signer": signer_ss58,
                "valid": valid,
            }));
            if !valid {
                anyhow::bail!("Signature verification failed");
            }
            Ok(())
        }
        WalletCommands::Derive { input } => {
            if input.starts_with("0x") {
                // Public key hex
                let hex_str = input.strip_prefix("0x")
                    .ok_or_else(|| anyhow::anyhow!("Expected 0x-prefixed hex"))?;
                let bytes = hex::decode(hex_str)
                    .map_err(|e| anyhow::anyhow!("Invalid hex: {}", e))?;
                if bytes.len() != 32 {
                    anyhow::bail!("Public key must be 32 bytes, got {}", bytes.len());
                }
                let arr: [u8; 32] = bytes.try_into()
                    .map_err(|_| anyhow::anyhow!("Public key must be exactly 32 bytes"))?;
                let public = sp_core::sr25519::Public::from_raw(arr);
                let ss58 = crate::wallet::keypair::to_ss58(&public, 42);
                crate::cli::helpers::print_json(&serde_json::json!({
                    "public_key": format!("0x{}", hex::encode(public.0)),
                    "ss58": ss58,
                }));
            } else {
                // Mnemonic phrase — derive public key only (never print secret)
                let pair = crate::wallet::keypair::pair_from_mnemonic(&input)?;
                let ss58 = crate::wallet::keypair::to_ss58(&pair.public(), 42);
                crate::cli::helpers::print_json(&serde_json::json!({
                    "public_key": format!("0x{}", hex::encode(pair.public().0)),
                    "ss58": ss58,
                }));
            }
            Ok(())
        }
    }
}
