//! Shared CLI helper functions.

use crate::wallet::Wallet;
use anyhow::Result;

use crate::cli::OutputFormat;

/// Common context passed to all command handlers, reducing parameter sprawl.
///
/// Instead of passing 6-9 individual parameters to every handler,
/// handlers receive a single `&Ctx` reference.
pub struct Ctx<'a> {
    pub wallet_dir: &'a str,
    pub wallet_name: &'a str,
    pub hotkey_name: &'a str,
    pub output: OutputFormat,
    pub password: Option<&'a str>,
    pub yes: bool,
    pub mev: bool,
    pub live_interval: Option<u64>,
}

/// Escape a value for RFC 4180 CSV output.
/// If the value contains a comma, double-quote, or newline, wrap it in double-quotes
/// and escape any internal double-quotes by doubling them.
pub fn csv_escape(val: &str) -> String {
    if val.contains(',') || val.contains('"') || val.contains('\n') || val.contains('\r') {
        let escaped = val.replace('"', "\"\"");
        format!("\"{}\"", escaped)
    } else {
        val.to_string()
    }
}

/// Join CSV fields with commas, escaping each field per RFC 4180.
pub fn csv_row_from(fields: &[&str]) -> String {
    fields.iter().map(|f| csv_escape(f)).collect::<Vec<_>>().join(",")
}

/// Create a styled spinner with a message, returns the ProgressBar handle.
/// Caller should call `.finish_with_message()` or `.finish_and_clear()` when done.
pub fn spinner(msg: &str) -> indicatif::ProgressBar {
    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_style(
        indicatif::ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .expect("static spinner template is valid")
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb
}

pub fn open_wallet(wallet_dir: &str, wallet_name: &str) -> Result<Wallet> {
    let path = format!("{}/{}", wallet_dir, wallet_name);
    if !std::path::Path::new(&path).exists() {
        anyhow::bail!(
            "Wallet '{}' not found in {}.\n  Create one with: agcli wallet create --name {}\n  List existing:   agcli wallet list",
            wallet_name, wallet_dir, wallet_name
        );
    }
    Wallet::open(&path)
}

/// Unlock the coldkey. If `password` is provided, use it directly (non-interactive).
/// Otherwise, prompt interactively (unless batch mode).
pub fn unlock_coldkey(wallet: &mut Wallet, password: Option<&str>) -> Result<()> {
    let pw = match password {
        Some(p) => p.to_string(),
        None => {
            if is_batch_mode() {
                anyhow::bail!(
                    "Password required in batch mode. Pass --password <pw> or set AGCLI_PASSWORD."
                );
            }
            dialoguer::Password::new()
                .with_prompt("Coldkey password")
                .interact()?
        }
    };
    tracing::debug!("Unlocking coldkey");
    wallet.unlock_coldkey(&pw)
        .map_err(|e| {
            let msg = e.to_string();
            tracing::warn!(error = %msg, "Coldkey unlock failed");
            if msg.contains("wrong password") || msg.contains("Decryption failed") {
                anyhow::anyhow!("{}\n  Tip: pass --password <pw> or set AGCLI_PASSWORD env var for non-interactive use.", msg)
            } else {
                e
            }
        })
}

/// Check per-subnet spending limit from config.
/// Returns Ok if no limit set or amount is within limit, Err otherwise.
pub fn check_spending_limit(netuid: u16, tao_amount: f64) -> Result<()> {
    let cfg = crate::config::Config::load();
    if let Some(ref limits) = cfg.spending_limits {
        let key = netuid.to_string();
        if let Some(&limit) = limits.get(&key) {
            if tao_amount > limit {
                tracing::warn!(netuid = netuid, amount = tao_amount, limit = limit, "Per-subnet spending limit exceeded");
                anyhow::bail!(
                    "Spending limit exceeded for SN{}: trying {:.4}τ but limit is {:.4}τ.\n  Adjust with: agcli config set spending_limit.{} {}",
                    netuid, tao_amount, limit, netuid, tao_amount
                );
            }
        }
        // Also check wildcard "*" key for global limit
        if let Some(&limit) = limits.get("*") {
            if tao_amount > limit {
                tracing::warn!(amount = tao_amount, limit = limit, "Global spending limit exceeded");
                anyhow::bail!(
                    "Global spending limit exceeded: trying {:.4}τ but limit is {:.4}τ.\n  Adjust with: agcli config set spending_limit.* {}",
                    tao_amount, limit, tao_amount
                );
            }
        }
    }
    Ok(())
}

/// Print a JSON value to stdout. Respects the global pretty-print flag.
pub fn print_json(value: &serde_json::Value) {
    if is_pretty_mode() {
        match serde_json::to_string_pretty(value) {
            Ok(s) => println!("{}", s),
            Err(e) => eprintln!("Error: failed to serialize JSON: {}", e),
        }
    } else {
        println!("{}", value);
    }
}

/// Print a Serialize-able value as JSON. Respects global pretty-print flag.
pub fn print_json_ser<T: serde::Serialize>(value: &T) {
    let result = if is_pretty_mode() {
        serde_json::to_string_pretty(value)
    } else {
        serde_json::to_string(value)
    };
    match result {
        Ok(s) => println!("{}", s),
        Err(e) => eprintln!("Error: failed to serialize JSON: {}", e),
    }
}

/// Print a JSON value to stderr. Respects the global pretty-print flag.
pub fn eprint_json(value: &serde_json::Value) {
    if is_pretty_mode() {
        match serde_json::to_string_pretty(value) {
            Ok(s) => eprintln!("{}", s),
            Err(e) => eprintln!("Error: failed to serialize JSON: {}", e),
        }
    } else {
        eprintln!("{}", value);
    }
}

/// Print transaction result in both json and table modes.
pub fn print_tx_result(output: OutputFormat, hash: &str, label: &str) {
    if output.is_json() {
        print_json(&serde_json::json!({"tx_hash": hash}));
    } else {
        println!("{} Tx: {}", label, hash);
    }
}

/// Thread-local pretty mode flag.
static PRETTY_MODE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Set pretty mode globally.
pub fn set_pretty_mode(pretty: bool) {
    PRETTY_MODE.store(pretty, std::sync::atomic::Ordering::Relaxed);
}

/// Check if pretty mode is active.
pub fn is_pretty_mode() -> bool {
    PRETTY_MODE.load(std::sync::atomic::Ordering::Relaxed)
}

/// Thread-local batch mode flag (set by main before dispatch).
static BATCH_MODE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Set batch mode globally (called from execute()).
pub fn set_batch_mode(batch: bool) {
    BATCH_MODE.store(batch, std::sync::atomic::Ordering::Relaxed);
}

/// Check if batch mode is active.
pub fn is_batch_mode() -> bool {
    BATCH_MODE.load(std::sync::atomic::Ordering::Relaxed)
}

pub fn resolve_coldkey_address(
    address: Option<String>,
    wallet_dir: &str,
    wallet_name: &str,
) -> String {
    address.unwrap_or_else(|| {
        match open_wallet(wallet_dir, wallet_name) {
            Ok(w) => w.coldkey_ss58().map(|s| s.to_string()).unwrap_or_default(),
            Err(e) => {
                tracing::debug!(wallet = wallet_name, error = %e, "Could not open wallet to resolve coldkey");
                String::new()
            }
        }
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
        .ok_or_else(|| anyhow::anyhow!("Could not resolve hotkey address.\n  Tip: pass --hotkey <ss58_address> or create a hotkey with `agcli wallet create-hotkey`."))
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
            anyhow::bail!(
                "Invalid weight pair '{}'. Format: 'uid:weight' (e.g., '0:100,1:200,2:50')",
                pair
            );
        }
        uids.push(
            parts[0].trim().parse::<u16>().map_err(|_| {
                anyhow::anyhow!("Invalid UID '{}' — must be 0–65535", parts[0].trim())
            })?,
        );
        weights.push(parts[1].trim().parse::<u16>().map_err(|_| {
            anyhow::anyhow!("Invalid weight '{}' — must be 0–65535", parts[1].trim())
        })?);
    }
    Ok((uids, weights))
}

pub fn parse_children(children_str: &str) -> Result<Vec<(u64, String)>> {
    let mut result = Vec::new();
    for pair in children_str.split(',') {
        let parts: Vec<&str> = pair.trim().split(':').collect();
        if parts.len() != 2 {
            anyhow::bail!(
                "Invalid child pair '{}'. Format: 'proportion:hotkey_ss58' (e.g., '50000:5Cai...')",
                pair
            );
        }
        let proportion = parts[0].trim().parse::<u64>().map_err(|_| {
            anyhow::anyhow!(
                "Invalid proportion '{}' — must be a positive integer (u64)",
                parts[0].trim()
            )
        })?;
        let hotkey = parts[1].trim().to_string();
        result.push((proportion, hotkey));
    }
    Ok(result)
}

/// Render a slice in json, csv, or table format.
///
/// - `json`: Serializes `data` with `print_json_ser`.
/// - `csv`: Prints `csv_header` then calls `csv_row` per item.
/// - `table`: Prints optional `preamble`, then builds a comfy_table
///   with `table_headers` and `table_row` per item.
pub fn render_rows<T: serde::Serialize>(
    output: OutputFormat,
    data: &[T],
    csv_header: &str,
    csv_row: impl Fn(&T) -> String,
    table_headers: &[&str],
    table_row: impl Fn(&T) -> Vec<String>,
    preamble: Option<&str>,
) {
    if output.is_json() {
        print_json_ser(&data);
    } else if output.is_csv() {
        println!("{}", csv_header);
        for item in data {
            println!("{}", csv_row(item));
        }
    } else {
        if let Some(text) = preamble {
            println!("{}", text);
        }
        let mut table = comfy_table::Table::new();
        table.set_header(
            table_headers
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
        );
        for item in data {
            table.add_row(table_row(item));
        }
        println!("{table}");
    }
}

/// Build a HashMap of netuid → &DynamicInfo for quick lookups.
pub fn build_dynamic_map(
    dynamic: &[crate::types::chain_data::DynamicInfo],
) -> std::collections::HashMap<u16, &crate::types::chain_data::DynamicInfo> {
    dynamic.iter().map(|d| (d.netuid.0, d)).collect()
}

/// Require a mnemonic phrase: use `provided` if Some, else prompt interactively (or error in batch mode).
pub fn require_mnemonic(provided: Option<String>) -> Result<String> {
    match provided {
        Some(m) => Ok(m),
        None => {
            if is_batch_mode() {
                anyhow::bail!("Mnemonic required in batch mode. Pass --mnemonic <phrase>.");
            }
            dialoguer::Input::<String>::new()
                .with_prompt("Enter mnemonic phrase")
                .interact_text()
                .map_err(anyhow::Error::from)
        }
    }
}

/// Require a password: use `cmd_password` (command-level), `global_password` (global flag), or prompt.
/// If `confirm` is true, ask for password confirmation on interactive entry.
pub fn require_password(
    cmd_password: Option<String>,
    global_password: Option<&str>,
    confirm: bool,
) -> Result<String> {
    cmd_password
        .or_else(|| global_password.map(|s| s.to_string()))
        .map(Ok)
        .unwrap_or_else(|| {
            if is_batch_mode() {
                return Err(anyhow::anyhow!(
                    "Password required in batch mode. Pass --password <pw> or set AGCLI_PASSWORD."
                ));
            }
            if confirm {
                dialoguer::Password::new()
                    .with_prompt("Set password")
                    .with_confirmation("Confirm", "Mismatch")
                    .interact()
                    .map_err(anyhow::Error::from)
            } else {
                dialoguer::Password::new()
                    .with_prompt("Password")
                    .interact()
                    .map_err(anyhow::Error::from)
            }
        })
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
            if let Some(hex_str) = s.strip_prefix("0x") {
                if let Ok(bytes) = hex::decode(hex_str) {
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
