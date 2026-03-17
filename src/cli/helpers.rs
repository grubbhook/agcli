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
    fields
        .iter()
        .map(|f| csv_escape(f))
        .collect::<Vec<_>>()
        .join(",")
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
    validate_name(wallet_name, "wallet")?;
    let raw = format!("{}/{}", wallet_dir, wallet_name);
    // Expand ~ so the existence check works outside a shell context.
    let path = if let Some(rest) = raw.strip_prefix("~/") {
        dirs::home_dir()
            .map(|h| h.join(rest).to_string_lossy().into_owned())
            .unwrap_or(raw)
    } else {
        raw
    };
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

/// Validate that an amount is positive and non-zero.
/// Returns a human-friendly error if the amount is invalid.
pub fn validate_amount(amount: f64, label: &str) -> Result<()> {
    if amount < 0.0 {
        anyhow::bail!(
            "Invalid {}: {:.9}. Amount cannot be negative.",
            label, amount
        );
    }
    if amount == 0.0 {
        anyhow::bail!(
            "Invalid {}: amount must be greater than zero.\n  Tip: minimum stake is 1 RAO (0.000000001 τ).",
            label
        );
    }
    if !amount.is_finite() {
        anyhow::bail!(
            "Invalid {}: amount must be a finite number (got {}).",
            label, amount
        );
    }
    Ok(())
}

/// Validate childkey take percentage is in the allowed range [0, 18].
pub fn validate_take_pct(take: f64) -> Result<()> {
    if take < 0.0 {
        anyhow::bail!(
            "Invalid childkey take: {:.2}%. Take cannot be negative.",
            take
        );
    }
    if take > 18.0 {
        anyhow::bail!(
            "Invalid childkey take: {:.2}%. Maximum allowed is 18%.\n  Tip: use --take 18 for maximum take.",
            take
        );
    }
    if !take.is_finite() {
        anyhow::bail!(
            "Invalid childkey take: must be a finite number (got {}).",
            take
        );
    }
    Ok(())
}

/// Validate a token symbol string (non-empty, reasonable length, ASCII).
pub fn validate_symbol(symbol: &str) -> Result<()> {
    let trimmed = symbol.trim();
    if trimmed.is_empty() {
        anyhow::bail!(
            "Invalid symbol: cannot be empty.\n  Tip: use a short, uppercase token symbol like \"ALPHA\" or \"SN1\"."
        );
    }
    if trimmed.len() > 32 {
        anyhow::bail!(
            "Invalid symbol: \"{}\" is too long ({} chars, max 32).\n  Tip: token symbols should be short, like \"ALPHA\".",
            trimmed, trimmed.len()
        );
    }
    if !trimmed.is_ascii() {
        anyhow::bail!(
            "Invalid symbol: \"{}\" contains non-ASCII characters. Use only ASCII letters/numbers.",
            trimmed
        );
    }
    Ok(())
}

/// Validate emission split weights (non-empty, no zeros in individual weights unless intentional).
pub fn validate_emission_weights(weights: &[u16]) -> Result<()> {
    if weights.is_empty() {
        anyhow::bail!("At least one emission weight is required.");
    }
    let total: u64 = weights.iter().map(|w| *w as u64).sum();
    if total == 0 {
        anyhow::bail!(
            "Invalid emission weights: total is zero. At least one weight must be non-zero."
        );
    }
    Ok(())
}

/// Validate snipe max-cost is non-negative.
pub fn validate_max_cost(max_cost: f64) -> Result<()> {
    if max_cost < 0.0 {
        anyhow::bail!(
            "Invalid --max-cost: {:.9}. Cost limit cannot be negative.",
            max_cost
        );
    }
    if !max_cost.is_finite() {
        anyhow::bail!(
            "Invalid --max-cost: must be a finite number (got {}).",
            max_cost
        );
    }
    Ok(())
}

/// Validate a wallet or hotkey name. Rejects path traversal, special characters,
/// and names that would be unsafe as directory/file names.
pub fn validate_name(name: &str, label: &str) -> Result<()> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        anyhow::bail!(
            "Invalid {} name: cannot be empty.\n  Tip: use a simple alphanumeric name like \"default\" or \"mywallet\".",
            label
        );
    }
    if trimmed.len() > 64 {
        anyhow::bail!(
            "Invalid {} name: \"{}\" is too long ({} chars, max 64).",
            label, trimmed, trimmed.len()
        );
    }
    // Path traversal
    if trimmed.contains("..") || trimmed.contains('/') || trimmed.contains('\\') {
        anyhow::bail!(
            "Invalid {} name: \"{}\" contains path separators or traversal sequences.\n  Tip: use a simple name without '/', '\\', or '..'.",
            label, trimmed
        );
    }
    // Absolute paths (Unix or Windows)
    if trimmed.starts_with('/') || trimmed.starts_with('\\') || (trimmed.len() >= 2 && trimmed.as_bytes()[1] == b':') {
        anyhow::bail!(
            "Invalid {} name: \"{}\" looks like an absolute path. Use a simple name.",
            label, trimmed
        );
    }
    // Reserved or hidden names
    if trimmed.starts_with('.') {
        anyhow::bail!(
            "Invalid {} name: \"{}\" starts with a dot (hidden file).\n  Tip: use a name that starts with a letter or number.",
            label, trimmed
        );
    }
    // Only allow alphanumeric, hyphens, underscores
    if !trimmed.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        anyhow::bail!(
            "Invalid {} name: \"{}\" contains invalid characters.\n  Tip: use only letters, numbers, hyphens, and underscores.",
            label, trimmed
        );
    }
    // OS reserved names (Windows)
    let upper = trimmed.to_uppercase();
    let reserved = ["CON", "PRN", "AUX", "NUL",
        "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
        "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9"];
    if reserved.contains(&upper.as_str()) {
        anyhow::bail!(
            "Invalid {} name: \"{}\" is a reserved system name.",
            label, trimmed
        );
    }
    Ok(())
}

/// Validate an IPv4 address string and return the numeric representation.
/// Rejects broadcast (255.255.255.255), unspecified (0.0.0.0), and warns on private ranges.
pub fn validate_ipv4(ip: &str) -> Result<u128> {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() != 4 {
        anyhow::bail!(
            "Invalid IPv4 address: \"{}\". Expected format: A.B.C.D (e.g., 1.2.3.4).",
            ip
        );
    }
    let mut octets = [0u8; 4];
    for (i, part) in parts.iter().enumerate() {
        // Reject leading zeros (ambiguous: octal vs decimal)
        if part.len() > 1 && part.starts_with('0') {
            anyhow::bail!(
                "Invalid IPv4 address: \"{}\" — octet {} has leading zeros. Use {} instead.",
                ip, i + 1, part.trim_start_matches('0')
            );
        }
        octets[i] = part.parse::<u8>().map_err(|_| {
            anyhow::anyhow!(
                "Invalid IPv4 address: \"{}\" — octet {} ('{}') is not a valid number (0–255).",
                ip, i + 1, part
            )
        })?;
    }
    // Reject all-zeros
    if octets == [0, 0, 0, 0] {
        anyhow::bail!(
            "Invalid IP address: 0.0.0.0 (unspecified). Use your actual public IP address."
        );
    }
    // Reject broadcast
    if octets == [255, 255, 255, 255] {
        anyhow::bail!(
            "Invalid IP address: 255.255.255.255 (broadcast). Use your actual public IP address."
        );
    }
    // Reject loopback
    if octets[0] == 127 {
        anyhow::bail!(
            "Invalid IP address: {} (loopback). Use your public IP address for serving on the network.",
            ip
        );
    }
    // Warn on private ranges (print warning but allow)
    let is_private = matches!(
        (octets[0], octets[1]),
        (10, _) | (172, 16..=31) | (192, 168)
    );
    if is_private {
        eprintln!(
            "Warning: {} is a private IP address. Other nodes on the public network won't be able to reach you.\n  Tip: use your public IP address for serving.",
            ip
        );
    }
    let ip_u128 = ((octets[0] as u128) << 24)
        | ((octets[1] as u128) << 16)
        | ((octets[2] as u128) << 8)
        | (octets[3] as u128);
    Ok(ip_u128)
}

/// Validate a delegate take percentage is in the allowed range [0, 18].
pub fn validate_delegate_take(take: f64) -> Result<()> {
    if take < 0.0 {
        anyhow::bail!(
            "Invalid delegate take: {:.2}%. Take cannot be negative.",
            take
        );
    }
    if take > 18.0 {
        anyhow::bail!(
            "Invalid delegate take: {:.2}%. Maximum allowed is 18%.\n  Tip: use --take 18 for maximum.",
            take
        );
    }
    if !take.is_finite() {
        anyhow::bail!(
            "Invalid delegate take: must be a finite number (got {}).",
            take
        );
    }
    Ok(())
}

/// Validate an SS58 address string. Returns Ok(()) if valid, or a helpful error message.
/// Use this to validate user-supplied addresses (--dest, --delegate, --hotkey, --spawner, etc.)
/// before submitting them to the chain.
pub fn validate_ss58(address: &str, label: &str) -> Result<()> {
    let trimmed = address.trim();
    if trimmed.is_empty() {
        anyhow::bail!(
            "Invalid {}: address cannot be empty.\n  Tip: provide a valid Bittensor SS58 address (48 characters, starts with '5').",
            label
        );
    }
    // Quick sanity checks before the expensive crypto verification
    let char_len = trimmed.chars().count();
    if char_len < 10 {
        anyhow::bail!(
            "Invalid {} address '{}' — too short. Bittensor SS58 addresses are 48 characters starting with '5'.",
            label, trimmed
        );
    }
    if char_len > 60 {
        let preview: String = trimmed.chars().take(20).collect();
        anyhow::bail!(
            "Invalid {} address '{}' — too long ({} chars). Bittensor SS58 addresses are 48 characters.",
            label, preview, char_len
        );
    }
    // Check for common mistakes: 0x prefix (Ethereum address), spaces, non-base58 chars
    if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
        anyhow::bail!(
            "Invalid {} address: '{}' looks like an Ethereum/hex address.\n  Tip: Bittensor uses SS58 addresses (start with '5'). Convert at https://ss58.org or use `agcli wallet show`.",
            label, trimmed
        );
    }
    if trimmed.contains(' ') || trimmed.contains('\t') {
        anyhow::bail!(
            "Invalid {} address: contains whitespace. Remove any spaces or tabs from the address.",
            label
        );
    }
    // Base58 character set check (1-9, A-H, J-N, P-Z, a-k, m-z — no 0, I, O, l)
    if let Some(bad) = trimmed.chars().find(|c| {
        !matches!(c, '1'..='9' | 'A'..='H' | 'J'..='N' | 'P'..='Z' | 'a'..='k' | 'm'..='z')
    }) {
        anyhow::bail!(
            "Invalid {} address '{}': character '{}' is not valid Base58.\n  Tip: SS58 addresses use Base58 encoding (no 0, I, O, or l).",
            label, crate::utils::short_ss58(trimmed), bad
        );
    }
    // Full cryptographic verification via sp_core
    use sp_core::{crypto::Ss58Codec, sr25519};
    sr25519::Public::from_ss58check(trimmed).map_err(|_| {
        anyhow::anyhow!(
            "Invalid {} address '{}': checksum verification failed.\n  Tip: double-check the address. Use `agcli wallet show` to get your correct address.",
            label, crate::utils::short_ss58(trimmed)
        )
    })?;
    Ok(())
}

/// Validate password strength for wallet creation. Returns Ok(()) always, but prints
/// warnings to stderr for weak passwords. Does NOT reject weak passwords — just warns.
pub fn validate_password_strength(password: &str) {
    if password.len() < 8 {
        eprintln!(
            "Warning: password is only {} characters. Consider using at least 8 characters for better security.",
            password.len()
        );
    }
    if password.len() < 4 {
        eprintln!(
            "Warning: password is very short ({} chars). Your wallet encryption may be easily brute-forced.",
            password.len()
        );
    }
    let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
    let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| !c.is_ascii_alphanumeric());
    let variety = [has_upper, has_lower, has_digit, has_special]
        .iter()
        .filter(|&&b| b)
        .count();
    if variety < 2 && password.len() >= 4 {
        eprintln!(
            "Warning: password uses only one character type. Mix uppercase, lowercase, numbers, and symbols for stronger security."
        );
    }
    // Check for common weak passwords
    let common = [
        "password", "12345678", "123456789", "1234567890", "qwerty",
        "abc123", "letmein", "welcome", "monkey", "dragon",
        "master", "login", "princess", "football", "shadow",
    ];
    if common.contains(&password.to_lowercase().as_str()) {
        eprintln!(
            "Warning: this is a commonly used password. Choose something unique to protect your wallet."
        );
    }
}

/// Validate a port number is in the valid range [1, 65535].
pub fn validate_port(port: u16, label: &str) -> Result<()> {
    if port == 0 {
        anyhow::bail!(
            "Invalid {} port: 0. Port must be between 1 and 65535.\n  Tip: common ports are 8091 (axon) and 443 (HTTPS).",
            label
        );
    }
    if port < 1024 {
        eprintln!(
            "Warning: {} port {} is a privileged port (< 1024). You may need root access to bind to it.",
            label, port
        );
    }
    Ok(())
}

/// Validate a netuid is in a reasonable range for the Bittensor network.
pub fn validate_netuid(netuid: u16) -> Result<()> {
    if netuid == 0 {
        anyhow::bail!(
            "Invalid netuid: 0. Root network (netuid 0) is not a user subnet.\n  Tip: user subnets start at netuid 1."
        );
    }
    Ok(())
}

/// Validate a batch-axon JSON file structure. Returns a vec of errors found.
/// Each entry should have: netuid (u16), ip (valid IPv4), port (u16).
/// Optional fields: protocol (u8, default 4), version (u32, default 0).
pub fn validate_batch_axon_json(json_str: &str) -> Result<Vec<serde_json::Value>> {
    let entries: Vec<serde_json::Value> = serde_json::from_str(json_str).map_err(|e| {
        anyhow::anyhow!(
            "Invalid batch-axon JSON: {}.\n  Expected format: [{{\"netuid\": 1, \"ip\": \"1.2.3.4\", \"port\": 8091}}]",
            e
        )
    })?;
    if entries.is_empty() {
        anyhow::bail!(
            "Batch-axon JSON is empty. Provide at least one entry.\n  Format: [{{\"netuid\": 1, \"ip\": \"1.2.3.4\", \"port\": 8091}}]"
        );
    }
    for (i, entry) in entries.iter().enumerate() {
        let obj = entry.as_object().ok_or_else(|| {
            anyhow::anyhow!("Batch-axon entry {} is not a JSON object. Each entry must be {{\"netuid\": N, \"ip\": \"...\", \"port\": N}}.", i)
        })?;
        // Required: netuid
        let netuid = obj.get("netuid").ok_or_else(|| {
            anyhow::anyhow!("Batch-axon entry {}: missing required field 'netuid'.", i)
        })?;
        let netuid_val = netuid.as_u64().ok_or_else(|| {
            anyhow::anyhow!("Batch-axon entry {}: 'netuid' must be a positive integer (got {}).", i, netuid)
        })?;
        if netuid_val > 65535 {
            anyhow::bail!("Batch-axon entry {}: 'netuid' {} exceeds maximum (65535).", i, netuid_val);
        }
        // Required: ip
        let ip = obj.get("ip").ok_or_else(|| {
            anyhow::anyhow!("Batch-axon entry {}: missing required field 'ip'.", i)
        })?;
        let ip_str = ip.as_str().ok_or_else(|| {
            anyhow::anyhow!("Batch-axon entry {}: 'ip' must be a string (got {}).", i, ip)
        })?;
        validate_ipv4(ip_str).map_err(|e| {
            anyhow::anyhow!("Batch-axon entry {}: {}", i, e)
        })?;
        // Required: port
        let port = obj.get("port").ok_or_else(|| {
            anyhow::anyhow!("Batch-axon entry {}: missing required field 'port'.", i)
        })?;
        let port_val = port.as_u64().ok_or_else(|| {
            anyhow::anyhow!("Batch-axon entry {}: 'port' must be a positive integer (got {}).", i, port)
        })?;
        if port_val == 0 || port_val > 65535 {
            anyhow::bail!("Batch-axon entry {}: 'port' {} is out of range (1–65535).", i, port_val);
        }
        // Optional: protocol (u8, default 4)
        if let Some(proto) = obj.get("protocol") {
            let proto_val = proto.as_u64().ok_or_else(|| {
                anyhow::anyhow!("Batch-axon entry {}: 'protocol' must be a number (got {}).", i, proto)
            })?;
            if proto_val > 255 {
                anyhow::bail!("Batch-axon entry {}: 'protocol' {} exceeds maximum (255).", i, proto_val);
            }
        }
        // Optional: version (u32)
        if let Some(ver) = obj.get("version") {
            ver.as_u64().ok_or_else(|| {
                anyhow::anyhow!("Batch-axon entry {}: 'version' must be a number (got {}).", i, ver)
            })?;
        }
        // Warn on unknown fields
        let known = ["netuid", "ip", "port", "protocol", "version"];
        for key in obj.keys() {
            if !known.contains(&key.as_str()) {
                eprintln!("Warning: batch-axon entry {}: unknown field '{}' (ignored).", i, key);
            }
        }
    }
    Ok(entries)
}

/// Check per-subnet spending limit from config.
/// Returns Ok if no limit set or amount is within limit, Err otherwise.
pub fn check_spending_limit(netuid: u16, tao_amount: f64) -> Result<()> {
    let cfg = crate::config::Config::load();
    if let Some(ref limits) = cfg.spending_limits {
        let key = netuid.to_string();
        if let Some(&limit) = limits.get(&key) {
            if tao_amount > limit {
                tracing::warn!(
                    netuid = netuid,
                    amount = tao_amount,
                    limit = limit,
                    "Per-subnet spending limit exceeded"
                );
                anyhow::bail!(
                    "Spending limit exceeded for SN{}: trying {:.4}τ but limit is {:.4}τ.\n  Adjust with: agcli config set spending_limit.{} {}",
                    netuid, tao_amount, limit, netuid, tao_amount
                );
            }
        }
        // Also check wildcard "*" key for global limit
        if let Some(&limit) = limits.get("*") {
            if tao_amount > limit {
                tracing::warn!(
                    amount = tao_amount,
                    limit = limit,
                    "Global spending limit exceeded"
                );
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
        validate_ss58(&hk, "hotkey")?;
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
    let trimmed = children_str.trim();
    if trimmed.is_empty() {
        anyhow::bail!(
            "Children list cannot be empty.\n  Format: 'proportion:hotkey_ss58' (e.g., '50000:5Cai...,50000:5Dqw...')"
        );
    }
    let mut result = Vec::new();
    for pair in trimmed.split(',') {
        let pair_trimmed = pair.trim();
        if pair_trimmed.is_empty() {
            continue; // skip trailing commas
        }
        // SS58 addresses contain colons in some edge cases, so split on first colon only
        let colon_pos = pair_trimmed.find(':').ok_or_else(|| {
            anyhow::anyhow!(
                "Invalid child pair '{}'. Format: 'proportion:hotkey_ss58' (e.g., '50000:5Cai...')",
                pair_trimmed
            )
        })?;
        let proportion_str = &pair_trimmed[..colon_pos].trim();
        let hotkey_str = &pair_trimmed[colon_pos + 1..].trim();
        let proportion = proportion_str.parse::<u64>().map_err(|_| {
            anyhow::anyhow!(
                "Invalid proportion '{}' — must be a positive integer (u64)",
                proportion_str
            )
        })?;
        if proportion == 0 {
            anyhow::bail!(
                "Invalid proportion: 0. Each child must have a non-zero proportion."
            );
        }
        // Validate the hotkey is a valid SS58 address
        validate_ss58(hotkey_str, "child hotkey")?;
        result.push((proportion, hotkey_str.to_string()));
    }
    if result.is_empty() {
        anyhow::bail!(
            "No valid children provided.\n  Format: 'proportion:hotkey_ss58' (e.g., '50000:5Cai...')"
        );
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

/// Validate a mnemonic phrase. Checks word count (12, 15, 18, 21, or 24 words)
/// and verifies all words are in the BIP-39 English dictionary with checksum validation.
/// Returns Ok(()) on success, or a helpful error message.
pub fn validate_mnemonic(mnemonic: &str) -> Result<()> {
    let trimmed = mnemonic.trim();
    if trimmed.is_empty() {
        anyhow::bail!(
            "Mnemonic phrase cannot be empty.\n  Tip: a BIP-39 mnemonic is 12, 15, 18, 21, or 24 English words."
        );
    }
    let words: Vec<&str> = trimmed.split_whitespace().collect();
    let valid_counts = [12, 15, 18, 21, 24];
    if !valid_counts.contains(&words.len()) {
        anyhow::bail!(
            "Invalid mnemonic: {} words found, expected 12, 15, 18, 21, or 24.\n  Tip: check for missing or extra words. Most wallets use 12-word mnemonics.",
            words.len()
        );
    }
    // Check each word is in the BIP-39 English wordlist
    let wordlist = bip39::Language::English.word_list();
    for (i, word) in words.iter().enumerate() {
        if wordlist.binary_search(word).is_err() {
            // Try to suggest a close match
            let suggestion = wordlist
                .iter()
                .filter(|w| w.starts_with(&word[..word.len().min(3)]))
                .next();
            let tip = if let Some(s) = suggestion {
                format!("  Did you mean \"{}\"?", s)
            } else {
                String::from("  Check spelling — all words must be from the BIP-39 English wordlist.")
            };
            anyhow::bail!(
                "Invalid mnemonic: word {} (\"{}\") is not in the BIP-39 dictionary.\n{}",
                i + 1, word, tip
            );
        }
    }
    // Full checksum validation via bip39 crate
    use bip39::Mnemonic;
    Mnemonic::parse_in(bip39::Language::English, trimmed).map_err(|e| {
        anyhow::anyhow!(
            "Invalid mnemonic checksum: {}.\n  Tip: the last word encodes a checksum. Make sure all words are correct and in the right order.",
            e
        )
    })?;
    Ok(())
}

/// Validate input for the `wallet derive` command.
/// Accepts either a 0x-prefixed 32-byte hex public key or a BIP-39 mnemonic phrase.
/// Rejects: empty input, odd-length hex, wrong-length hex keys, invalid hex chars,
/// ambiguous inputs that look neither like hex nor like a mnemonic.
pub fn validate_derive_input(input: &str) -> Result<()> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        anyhow::bail!(
            "Derive input cannot be empty.\n  Tip: pass a 0x-prefixed hex public key (64 hex chars) or a BIP-39 mnemonic phrase."
        );
    }
    if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
        // Hex public key path
        let hex_str = &trimmed[2..];
        if hex_str.is_empty() {
            anyhow::bail!(
                "Hex public key is empty after '0x' prefix.\n  Tip: provide 64 hex characters, e.g. '0x0123...abcd'."
            );
        }
        if hex_str.len() % 2 != 0 {
            anyhow::bail!(
                "Hex has odd length ({} chars). Hex bytes come in pairs.\n  Tip: check for a missing or extra character.",
                hex_str.len()
            );
        }
        // Validate chars before decode
        if let Some(pos) = hex_str.find(|c: char| !c.is_ascii_hexdigit()) {
            let bad_char = hex_str.chars().nth(pos).unwrap();
            anyhow::bail!(
                "Invalid hex character '{}' at position {}.\n  Tip: hex values use only 0-9 and a-f.",
                bad_char, pos + 2
            );
        }
        let byte_len = hex_str.len() / 2;
        if byte_len != 32 {
            anyhow::bail!(
                "Public key must be 32 bytes (64 hex chars), got {} bytes ({} hex chars).\n  Tip: an SR25519 public key is exactly 32 bytes.",
                byte_len, hex_str.len()
            );
        }
    } else {
        // Treat as mnemonic
        validate_mnemonic(trimmed)?;
    }
    Ok(())
}

/// Require a mnemonic phrase: use `provided` if Some, else prompt interactively (or error in batch mode).
/// Validates the mnemonic format and dictionary words before returning.
pub fn require_mnemonic(provided: Option<String>) -> Result<String> {
    let mnemonic = match provided {
        Some(m) => m,
        None => {
            if is_batch_mode() {
                anyhow::bail!("Mnemonic required in batch mode. Pass --mnemonic <phrase>.");
            }
            dialoguer::Input::<String>::new()
                .with_prompt("Enter mnemonic phrase")
                .interact_text()
                .map_err(anyhow::Error::from)?
        }
    };
    validate_mnemonic(&mnemonic)?;
    Ok(mnemonic)
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

/// Validate JSON args for multisig batch extrinsics.
/// Rejects: non-array JSON, null elements, deeply nested structures (depth > 4),
/// excessively long strings (> 1024 chars), NaN/Infinity floats.
/// Returns the parsed JSON array on success.
pub fn validate_multisig_json_args(json_str: &str) -> Result<Vec<serde_json::Value>> {
    let trimmed = json_str.trim();
    if trimmed.is_empty() {
        anyhow::bail!(
            "Empty JSON args.\n  Tip: pass a JSON array, e.g. '[1, \"0x...\"]'."
        );
    }
    let parsed: serde_json::Value = serde_json::from_str(trimmed).map_err(|e| {
        anyhow::anyhow!(
            "Invalid JSON: {}.\n  Tip: args must be a valid JSON array, e.g. '[1, \"0x...\"]'.",
            e
        )
    })?;
    let arr = match parsed {
        serde_json::Value::Array(a) => a,
        other => {
            anyhow::bail!(
                "Expected a JSON array, got {}.\n  Tip: wrap your args in square brackets, e.g. '[{}]'.",
                match &other {
                    serde_json::Value::Object(_) => "an object",
                    serde_json::Value::String(_) => "a string",
                    serde_json::Value::Number(_) => "a number",
                    serde_json::Value::Bool(_) => "a boolean",
                    serde_json::Value::Null => "null",
                    _ => "unknown type",
                },
                other
            );
        }
    };
    // Validate each element
    fn check_depth(v: &serde_json::Value, depth: usize) -> Result<()> {
        if depth > 4 {
            anyhow::bail!(
                "JSON nesting too deep (>4 levels).\n  Tip: flatten your args structure."
            );
        }
        match v {
            serde_json::Value::Null => {
                anyhow::bail!(
                    "null values not allowed in multisig args.\n  Tip: use 0 or \"\" instead of null."
                );
            }
            serde_json::Value::String(s) if s.len() > 1024 => {
                anyhow::bail!(
                    "String arg too long ({} chars, max 1024).\n  Tip: shorten the value or use a different encoding.",
                    s.len()
                );
            }
            serde_json::Value::Number(n) => {
                if n.as_f64().map_or(false, |f| f.is_nan() || f.is_infinite()) {
                    anyhow::bail!(
                        "NaN/Infinity not allowed in args.\n  Tip: use a finite number."
                    );
                }
            }
            serde_json::Value::Array(inner) => {
                for item in inner {
                    check_depth(item, depth + 1)?;
                }
            }
            serde_json::Value::Object(map) => {
                for (_k, val) in map {
                    check_depth(val, depth + 1)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    for (i, elem) in arr.iter().enumerate() {
        check_depth(elem, 0).map_err(|e| {
            anyhow::anyhow!("Invalid arg at index {}: {}", i, e)
        })?;
    }
    Ok(arr)
}

/// Validate an EVM address string (hex, 20 bytes). Accepts optional 0x prefix.
/// Returns a helpful error for common mistakes.
pub fn validate_evm_address(address: &str, label: &str) -> Result<()> {
    let trimmed = address.trim();
    if trimmed.is_empty() {
        anyhow::bail!(
            "Invalid {} EVM address: cannot be empty.\n  Tip: provide a 0x-prefixed 40 hex char address, e.g. '0x1234...abcd'.",
            label
        );
    }
    let hex_str = trimmed.strip_prefix("0x").or_else(|| trimmed.strip_prefix("0X")).unwrap_or(trimmed);
    if hex_str.is_empty() {
        anyhow::bail!(
            "Invalid {} EVM address: empty after '0x' prefix.\n  Tip: provide 40 hex characters after '0x'.",
            label
        );
    }
    if hex_str.len() % 2 != 0 {
        anyhow::bail!(
            "Invalid {} EVM address: odd hex length ({} chars). Hex bytes come in pairs.\n  Tip: check for a missing or extra character.",
            label, hex_str.len()
        );
    }
    if let Some(pos) = hex_str.find(|c: char| !c.is_ascii_hexdigit()) {
        let bad_char = hex_str.chars().nth(pos).unwrap();
        anyhow::bail!(
            "Invalid {} EVM address: character '{}' at position {} is not valid hex.\n  Tip: use only 0-9 and a-f.",
            label, bad_char, pos
        );
    }
    let byte_len = hex_str.len() / 2;
    if byte_len != 20 {
        anyhow::bail!(
            "Invalid {} EVM address: must be 20 bytes (40 hex chars), got {} bytes ({} hex chars).\n  Tip: Ethereum/EVM addresses are exactly 20 bytes.",
            label, byte_len, hex_str.len()
        );
    }
    Ok(())
}

/// Validate a hex data string. Accepts optional 0x prefix.
/// Rejects odd-length and non-hex characters.
pub fn validate_hex_data(data: &str, label: &str) -> Result<()> {
    let trimmed = data.trim();
    if trimmed.is_empty() {
        anyhow::bail!(
            "Invalid {} hex data: cannot be empty.\n  Tip: use '0x' for empty data.",
            label
        );
    }
    let hex_str = trimmed.strip_prefix("0x").or_else(|| trimmed.strip_prefix("0X")).unwrap_or(trimmed);
    // "0x" alone is valid (empty data)
    if hex_str.is_empty() {
        return Ok(());
    }
    if hex_str.len() % 2 != 0 {
        anyhow::bail!(
            "Invalid {} hex data: odd length ({} chars). Hex bytes come in pairs.\n  Tip: check for a missing or extra character.",
            label, hex_str.len()
        );
    }
    if let Some(pos) = hex_str.find(|c: char| !c.is_ascii_hexdigit()) {
        let bad_char = hex_str.chars().nth(pos).unwrap();
        anyhow::bail!(
            "Invalid {} hex data: character '{}' at position {} is not valid hex.\n  Tip: use only 0-9 and a-f.",
            label, bad_char, pos
        );
    }
    Ok(())
}

/// Validate a pallet or call name for scheduler/preimage commands.
/// Must be non-empty, reasonable length, and contain only valid Rust identifier characters.
pub fn validate_pallet_call(name: &str, label: &str) -> Result<()> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        anyhow::bail!(
            "Invalid {}: cannot be empty.\n  Tip: use the pallet name exactly as in the runtime, e.g. 'System', 'Balances', 'SubtensorModule'.",
            label
        );
    }
    if trimmed.len() > 128 {
        let preview: String = trimmed.chars().take(32).collect();
        anyhow::bail!(
            "Invalid {}: '{}' is too long ({} chars, max 128).",
            label, preview, trimmed.len()
        );
    }
    // Must start with a letter (PascalCase for pallets, snake_case for calls)
    if !trimmed.chars().next().unwrap().is_ascii_alphabetic() {
        anyhow::bail!(
            "Invalid {}: '{}' must start with a letter.\n  Tip: pallet names are PascalCase (e.g. 'System'), call names are snake_case (e.g. 'remark').",
            label, trimmed
        );
    }
    // Only allow alphanumeric and underscore (covers PascalCase + snake_case)
    if let Some(bad) = trimmed.chars().find(|c| !c.is_ascii_alphanumeric() && *c != '_') {
        anyhow::bail!(
            "Invalid {}: character '{}' is not allowed.\n  Tip: use only letters, numbers, and underscores.",
            label, bad
        );
    }
    Ok(())
}

/// Validate a scheduler task ID (for schedule-named / cancel-named).
/// Must be non-empty and ≤ 32 bytes (on-chain limit).
pub fn validate_schedule_id(id: &str) -> Result<()> {
    if id.is_empty() {
        anyhow::bail!(
            "Schedule ID cannot be empty.\n  Tip: provide a short, descriptive name for your scheduled task."
        );
    }
    if id.len() > 32 {
        anyhow::bail!(
            "Schedule ID too long: {} bytes (max 32).\n  Tip: use a shorter ID.",
            id.len()
        );
    }
    Ok(())
}

/// Parse an optional JSON string into a vec of subxt dynamic Values.
/// Validates the JSON structure before converting.
pub fn parse_json_args(args: &Option<String>) -> anyhow::Result<Vec<subxt::dynamic::Value>> {
    if let Some(ref args_json) = args {
        let validated = validate_multisig_json_args(args_json)?;
        Ok(validated.iter().map(json_to_subxt_value).collect())
    } else {
        Ok(vec![])
    }
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
