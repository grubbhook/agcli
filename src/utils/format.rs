//! Formatting helpers for CLI display.

use crate::types::Balance;

/// Truncate an SS58 address for display: "5Gx...abc"
pub fn short_ss58(addr: &str) -> String {
    if addr.len() <= 10 {
        return addr.to_string();
    }
    format!("{}...{}", &addr[..4], &addr[addr.len() - 4..])
}

/// Format TAO balance with commas: "1,234.5678"
pub fn format_tao(balance: Balance) -> String {
    let tao = balance.tao();
    if tao >= 1_000.0 {
        let whole = tao as u64;
        let frac = tao - whole as f64;
        format!(
            "{},{:03}.{:04}",
            whole / 1000,
            whole % 1000,
            (frac * 10000.0) as u64
        )
    } else {
        format!("{:.4}", tao)
    }
}

/// Truncate a string to `max` chars, appending ellipsis if needed.
pub fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max - 1])
    }
}

/// Normalize u16 weight to f64 in [0, 1].
pub fn u16_to_float(val: u16) -> f64 {
    val as f64 / 65535.0
}

/// Convert f64 in [0, 1] to u16 weight.
pub fn float_to_u16(val: f64) -> u16 {
    (val.clamp(0.0, 1.0) * 65535.0) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_address() {
        let addr = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
        assert_eq!(short_ss58(addr), "5Grw...utQY");
    }

    #[test]
    fn short_address_short_input() {
        assert_eq!(short_ss58("abcde"), "abcde");
        assert_eq!(short_ss58(""), "");
        assert_eq!(short_ss58("1234567890"), "1234567890");
    }

    #[test]
    fn weight_roundtrip() {
        let f = 0.5;
        let u = float_to_u16(f);
        let back = u16_to_float(u);
        assert!((back - f).abs() < 0.001);
    }

    #[test]
    fn weight_boundaries() {
        assert_eq!(float_to_u16(0.0), 0);
        assert_eq!(float_to_u16(1.0), 65535);
        assert_eq!(u16_to_float(0), 0.0);
        assert!((u16_to_float(65535) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn weight_clamp() {
        assert_eq!(float_to_u16(-1.0), 0);
        assert_eq!(float_to_u16(2.0), 65535);
    }

    #[test]
    fn format_tao_small() {
        let b = Balance::from_tao(0.5);
        let s = format_tao(b);
        assert!(s.starts_with("0."));
    }

    #[test]
    fn format_tao_large() {
        let b = Balance::from_tao(12345.6789);
        let s = format_tao(b);
        assert!(s.contains(","));
    }
}
