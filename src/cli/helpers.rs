//! Shared CLI helper functions.

use crate::wallet::Wallet;
use anyhow::Result;

pub fn open_wallet(wallet_dir: &str, wallet_name: &str) -> Result<Wallet> {
    Wallet::open(&format!("{}/{}", wallet_dir, wallet_name))
}

/// Unlock the coldkey. If `password` is provided, use it directly (non-interactive).
/// Otherwise, prompt interactively.
pub fn unlock_coldkey(wallet: &mut Wallet, password: Option<&str>) -> Result<()> {
    let pw = match password {
        Some(p) => p.to_string(),
        None => dialoguer::Password::new()
            .with_prompt("Coldkey password")
            .interact()?,
    };
    wallet.unlock_coldkey(&pw)
}

pub fn resolve_coldkey_address(address: Option<String>, wallet_dir: &str, wallet_name: &str) -> String {
    address.unwrap_or_else(|| {
        open_wallet(wallet_dir, wallet_name)
            .ok()
            .and_then(|w| w.coldkey_ss58().map(|s| s.to_string()))
            .unwrap_or_default()
    })
}

pub fn resolve_hotkey_ss58(
    hotkey_arg: Option<String>,
    wallet: &mut Wallet,
    hotkey_name: &str,
) -> Result<String> {
    if let Some(hk) = hotkey_arg {
        return Ok(hk);
    }
    wallet.load_hotkey(hotkey_name)?;
    wallet
        .hotkey_ss58()
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Could not resolve hotkey SS58 address"))
}

/// Shortcut: open wallet, unlock, resolve hotkey, return (pair, hotkey_ss58).
pub fn unlock_and_resolve(
    wallet_dir: &str,
    wallet_name: &str,
    hotkey_name: &str,
    hotkey_arg: Option<String>,
    password: Option<&str>,
) -> Result<(sp_core::sr25519::Pair, String)> {
    let mut wallet = open_wallet(wallet_dir, wallet_name)?;
    unlock_coldkey(&mut wallet, password)?;
    let hotkey_ss58 = resolve_hotkey_ss58(hotkey_arg, &mut wallet, hotkey_name)?;
    let pair = wallet.coldkey()?.clone();
    Ok((pair, hotkey_ss58))
}

pub fn parse_weight_pairs(weights_str: &str) -> Result<(Vec<u16>, Vec<u16>)> {
    let mut uids = Vec::new();
    let mut weights = Vec::new();
    for pair in weights_str.split(',') {
        let parts: Vec<&str> = pair.trim().split(':').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid weight pair '{}', expected 'uid:weight'", pair);
        }
        uids.push(parts[0].trim().parse::<u16>()?);
        weights.push(parts[1].trim().parse::<u16>()?);
    }
    Ok((uids, weights))
}

pub fn parse_children(children_str: &str) -> Result<Vec<(u64, String)>> {
    let mut result = Vec::new();
    for pair in children_str.split(',') {
        let parts: Vec<&str> = pair.trim().split(':').collect();
        if parts.len() != 2 {
            anyhow::bail!(
                "Invalid child pair '{}', expected 'proportion:hotkey_ss58'",
                pair
            );
        }
        let proportion = parts[0].trim().parse::<u64>()?;
        let hotkey = parts[1].trim().to_string();
        result.push((proportion, hotkey));
    }
    Ok(result)
}

/// Convert a serde_json::Value to a subxt dynamic Value for multisig call args.
pub fn json_to_subxt_value(v: &serde_json::Value) -> subxt::dynamic::Value {
    use subxt::dynamic::Value;
    match v {
        serde_json::Value::Number(n) => {
            if let Some(u) = n.as_u64() {
                Value::u128(u as u128)
            } else if let Some(i) = n.as_i64() {
                Value::i128(i as i128)
            } else {
                Value::string(n.to_string())
            }
        }
        serde_json::Value::String(s) => {
            if s.starts_with("0x") {
                if let Ok(bytes) = hex::decode(s.strip_prefix("0x").unwrap()) {
                    return Value::from_bytes(bytes);
                }
            }
            Value::string(s.clone())
        }
        serde_json::Value::Bool(b) => Value::bool(*b),
        serde_json::Value::Array(arr) => {
            Value::unnamed_composite(arr.iter().map(json_to_subxt_value))
        }
        _ => Value::string(v.to_string()),
    }
}
