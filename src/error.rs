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
}
