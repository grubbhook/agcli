//! Typed error codes for scripting and automation.
//!
//! Each error category maps to a distinct exit code so callers can
//! distinguish between "retry later" (network) and "fix your input"
//! (validation) without parsing stderr.

/// Exit codes used by agcli.
///
/// Standard convention: 0 = success, 1 = generic, 2+ = categorized.
pub mod exit_code {
    /// Generic / uncategorized error.
    pub const GENERIC: i32 = 1;
    /// Network / connection error (server unreachable, timeout, DNS failure).
    pub const NETWORK: i32 = 10;
    /// Authentication / wallet error (wrong password, missing key, locked wallet).
    pub const AUTH: i32 = 11;
    /// Validation error (bad input, invalid address, parse failure).
    pub const VALIDATION: i32 = 12;
    /// Chain/runtime error (extrinsic rejected, insufficient balance, rate-limited).
    pub const CHAIN: i32 = 13;
    /// File I/O error (permission denied, missing file, disk full).
    pub const IO: i32 = 14;
    /// Timeout (operation exceeded deadline).
    pub const TIMEOUT: i32 = 15;
}

/// Classify an anyhow error chain into an exit code.
pub fn classify(err: &anyhow::Error) -> i32 {
    let msg = format!("{:#}", err).to_lowercase();

    // Walk the chain for typed errors
    for cause in err.chain() {
        // Network errors
        if let Some(re) = cause.downcast_ref::<reqwest::Error>() {
            if re.is_timeout() {
                return exit_code::TIMEOUT;
            }
            return exit_code::NETWORK;
        }
        if cause.downcast_ref::<serde_json::Error>().is_some() {
            return exit_code::VALIDATION;
        }
        if let Some(io) = cause.downcast_ref::<std::io::Error>() {
            match io.kind() {
                std::io::ErrorKind::NotFound
                | std::io::ErrorKind::PermissionDenied
                | std::io::ErrorKind::AlreadyExists => return exit_code::IO,
                std::io::ErrorKind::TimedOut => return exit_code::TIMEOUT,
                std::io::ErrorKind::ConnectionRefused
                | std::io::ErrorKind::ConnectionReset
                | std::io::ErrorKind::ConnectionAborted => return exit_code::NETWORK,
                _ => {}
            }
        }
    }

    // Heuristic classification from error messages
    if msg.contains("wrong password")
        || msg.contains("decryption failed")
        || msg.contains("no hotkey loaded")
        || msg.contains("unlock")
        || msg.contains("no coldkey")
        || msg.contains("keyfile")
    {
        return exit_code::AUTH;
    }

    if msg.contains("invalid ss58")
        || msg.contains("failed to parse")
        || msg.contains("invalid address")
        || msg.contains("not a valid")
        || msg.contains("must be ")
        || msg.contains("expected format")
    {
        return exit_code::VALIDATION;
    }

    if msg.contains("connect")
        || msg.contains("dns")
        || msg.contains("websocket")
        || msg.contains("endpoint")
        || msg.contains("unreachable")
    {
        return exit_code::NETWORK;
    }

    if msg.contains("timeout") || msg.contains("timed out") {
        return exit_code::TIMEOUT;
    }

    if msg.contains("insufficient")
        || msg.contains("rate limit")
        || msg.contains("extrinsic")
        || msg.contains("dispatch")
        || msg.contains("nonce")
    {
        return exit_code::CHAIN;
    }

    if msg.contains("permission denied")
        || msg.contains("no such file")
        || msg.contains("cannot read")
        || msg.contains("cannot write")
        || msg.contains("cannot create")
    {
        return exit_code::IO;
    }

    exit_code::GENERIC
}

/// Provide an actionable hint based on the error code and message.
pub fn hint(code: i32, msg: &str) -> Option<&'static str> {
    let lower = msg.to_lowercase();
    match code {
        exit_code::NETWORK => {
            if lower.contains("dns") {
                Some("Tip: Check your DNS settings or try a different endpoint with --endpoint <url>")
            } else if lower.contains("refused") || lower.contains("unreachable") {
                Some("Tip: The chain endpoint may be down. Try --endpoint wss://entrypoint-finney.opentensor.ai:443 or check your network connection")
            } else {
                Some("Tip: Check your internet connection, or try a different endpoint with --endpoint <url>")
            }
        }
        exit_code::AUTH => {
            if lower.contains("password") {
                Some("Tip: Verify your password. Use AGCLI_PASSWORD env var for non-interactive mode")
            } else if lower.contains("hotkey") {
                Some("Tip: Create a hotkey with `agcli wallet create-hotkey` or pass --hotkey <ss58>")
            } else {
                Some("Tip: Check wallet path with --wallet-dir and --wallet flags. List wallets: `agcli wallet list`")
            }
        }
        exit_code::TIMEOUT => {
            Some("Tip: Increase timeout with --timeout <seconds> (default: none). The chain may be congested")
        }
        exit_code::CHAIN => {
            if lower.contains("insufficient") {
                Some("Tip: Check your balance with `agcli balance`. Transaction fees require a small reserve")
            } else if lower.contains("rate limit") {
                Some("Tip: Wait a few blocks before retrying. Use `agcli block latest` to check block progress")
            } else if lower.contains("nonce") {
                Some("Tip: Another transaction may be pending. Wait for it to finalize before retrying")
            } else {
                Some("Tip: The chain rejected this operation. Check `agcli doctor` for diagnostic info")
            }
        }
        exit_code::IO => {
            Some("Tip: Check file permissions and paths. Use --wallet-dir to specify wallet location")
        }
        exit_code::VALIDATION => None, // Validation errors already contain specific hints
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_wrong_password() {
        let err = anyhow::anyhow!("Decryption failed — wrong password");
        assert_eq!(classify(&err), exit_code::AUTH);
    }

    #[test]
    fn classify_invalid_address() {
        let err = anyhow::anyhow!("Invalid SS58 address: bad checksum");
        assert_eq!(classify(&err), exit_code::VALIDATION);
    }

    #[test]
    fn classify_connection_error() {
        let err = anyhow::anyhow!("Failed to connect to endpoint wss://...");
        assert_eq!(classify(&err), exit_code::NETWORK);
    }

    #[test]
    fn classify_insufficient_balance() {
        let err = anyhow::anyhow!("Extrinsic failed: insufficient balance for transfer");
        assert_eq!(classify(&err), exit_code::CHAIN);
    }

    #[test]
    fn classify_timeout() {
        let err = anyhow::anyhow!("Operation timed out after 30s");
        assert_eq!(classify(&err), exit_code::TIMEOUT);
    }

    #[test]
    fn classify_io_error() {
        let err = anyhow::anyhow!("Permission denied writing to /etc/foo");
        assert_eq!(classify(&err), exit_code::IO);
    }

    #[test]
    fn classify_generic() {
        let err = anyhow::anyhow!("Something unexpected happened");
        assert_eq!(classify(&err), exit_code::GENERIC);
    }

    #[test]
    fn classify_chained_io_error() {
        use std::io;
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "cannot write keyfile");
        let err = anyhow::Error::new(io_err).context("Writing wallet coldkey");
        assert_eq!(classify(&err), exit_code::IO);
    }

    #[test]
    fn classify_chained_connection_error() {
        use std::io;
        let io_err = io::Error::new(io::ErrorKind::ConnectionRefused, "connection refused");
        let err = anyhow::Error::new(io_err).context("Connecting to finney");
        assert_eq!(classify(&err), exit_code::NETWORK);
    }

    #[test]
    fn classify_nonce_error() {
        let err = anyhow::anyhow!("Nonce already used for this account");
        assert_eq!(classify(&err), exit_code::CHAIN);
    }

    #[test]
    fn classify_dns_error() {
        let err = anyhow::anyhow!("DNS resolution failed for entrypoint-finney.opentensor.ai");
        assert_eq!(classify(&err), exit_code::NETWORK);
    }

    #[test]
    fn classify_rate_limit() {
        let err = anyhow::anyhow!("Rate limit exceeded: too many staking operations");
        assert_eq!(classify(&err), exit_code::CHAIN);
    }

    #[test]
    fn classify_no_such_file() {
        let err = anyhow::anyhow!("No such file or directory: ~/.bittensor/wallets/default");
        assert_eq!(classify(&err), exit_code::IO);
    }

    #[test]
    fn classify_websocket() {
        let err = anyhow::anyhow!("WebSocket connection dropped unexpectedly");
        assert_eq!(classify(&err), exit_code::NETWORK);
    }

    #[test]
    fn classify_empty_error() {
        let err = anyhow::anyhow!("");
        assert_eq!(classify(&err), exit_code::GENERIC);
    }

    #[test]
    fn classify_case_insensitive() {
        let err = anyhow::anyhow!("TIMEOUT waiting for block finalization");
        assert_eq!(classify(&err), exit_code::TIMEOUT);
    }

    // ──── hint() tests ────

    #[test]
    fn classify_serde_json_error() {
        let json_err: serde_json::Error = serde_json::from_str::<String>("not valid json").unwrap_err();
        let err = anyhow::Error::new(json_err).context("Failed to deserialize chain data");
        assert_eq!(classify(&err), exit_code::VALIDATION);
    }

    #[test]
    fn hint_network_dns() {
        let h = hint(exit_code::NETWORK, "DNS resolution failed");
        assert!(h.is_some());
        assert!(h.unwrap().contains("DNS"));
    }

    #[test]
    fn hint_network_refused() {
        let h = hint(exit_code::NETWORK, "Connection refused");
        assert!(h.is_some());
        assert!(h.unwrap().contains("endpoint"));
    }

    #[test]
    fn hint_auth_password() {
        let h = hint(exit_code::AUTH, "wrong password");
        assert!(h.is_some());
        assert!(h.unwrap().contains("password"));
    }

    #[test]
    fn hint_auth_hotkey() {
        let h = hint(exit_code::AUTH, "No hotkey loaded");
        assert!(h.is_some());
        assert!(h.unwrap().contains("hotkey"));
    }

    #[test]
    fn hint_timeout() {
        let h = hint(exit_code::TIMEOUT, "timed out");
        assert!(h.is_some());
        assert!(h.unwrap().contains("--timeout"));
    }

    #[test]
    fn hint_chain_insufficient() {
        let h = hint(exit_code::CHAIN, "insufficient balance");
        assert!(h.is_some());
        assert!(h.unwrap().contains("balance"));
    }

    #[test]
    fn hint_chain_nonce() {
        let h = hint(exit_code::CHAIN, "Nonce already used");
        assert!(h.is_some());
        assert!(h.unwrap().contains("pending"));
    }

    #[test]
    fn hint_io() {
        let h = hint(exit_code::IO, "permission denied");
        assert!(h.is_some());
    }

    #[test]
    fn hint_validation_none() {
        // Validation errors don't get hints (they already contain specific messages)
        let h = hint(exit_code::VALIDATION, "invalid input");
        assert!(h.is_none());
    }

    #[test]
    fn hint_generic_none() {
        let h = hint(exit_code::GENERIC, "unknown error");
        assert!(h.is_none());
    }

    // ──── Additional classify() coverage ────

    #[test]
    fn classify_dispatch_error() {
        let err = anyhow::anyhow!("Dispatch error: extrinsic failed on chain");
        assert_eq!(classify(&err), exit_code::CHAIN);
    }

    #[test]
    fn classify_cannot_create() {
        let err = anyhow::anyhow!("Cannot create directory: /foo/bar");
        assert_eq!(classify(&err), exit_code::IO);
    }

    #[test]
    fn classify_not_valid() {
        let err = anyhow::anyhow!("Input is not a valid netuid");
        assert_eq!(classify(&err), exit_code::VALIDATION);
    }

    #[test]
    fn classify_expected_format() {
        let err = anyhow::anyhow!("Expected format: ss58 address");
        assert_eq!(classify(&err), exit_code::VALIDATION);
    }

    #[test]
    fn classify_unreachable() {
        let err = anyhow::anyhow!("Host unreachable: entrypoint-finney.opentensor.ai");
        assert_eq!(classify(&err), exit_code::NETWORK);
    }

    #[test]
    fn classify_cannot_read() {
        // "Cannot read" matches IO, but "keyfile" matches AUTH first — AUTH takes priority
        let err = anyhow::anyhow!("Cannot read wallet keyfile");
        assert_eq!(classify(&err), exit_code::AUTH);
    }

    #[test]
    fn classify_cannot_read_generic_file() {
        let err = anyhow::anyhow!("Cannot read config file /etc/foo");
        assert_eq!(classify(&err), exit_code::IO);
    }

    #[test]
    fn hint_chain_rate_limit() {
        let h = hint(exit_code::CHAIN, "Rate limit exceeded");
        assert!(h.is_some());
        assert!(h.unwrap().contains("Wait"));
    }

    #[test]
    fn hint_network_generic() {
        let h = hint(exit_code::NETWORK, "some network error");
        assert!(h.is_some());
        assert!(h.unwrap().contains("internet connection"));
    }

    #[test]
    fn hint_auth_generic() {
        let h = hint(exit_code::AUTH, "wallet locked");
        assert!(h.is_some());
        assert!(h.unwrap().contains("wallet"));
    }
}
