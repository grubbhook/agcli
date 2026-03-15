//! Persistent configuration file (~/.agcli/config.toml).
//!
//! Stores user preferences: default network, wallet, hotkey, endpoint, output format.
//! CLI flags override config file values.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Configuration loaded from ~/.agcli/config.toml.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Default network (finney, test, local, or custom URL).
    pub network: Option<String>,
    /// Custom chain endpoint (overrides network).
    pub endpoint: Option<String>,
    /// Wallet directory.
    pub wallet_dir: Option<String>,
    /// Default wallet name.
    pub wallet: Option<String>,
    /// Default hotkey name.
    pub hotkey: Option<String>,
    /// Default output format (table, json, csv).
    pub output: Option<String>,
    /// Proxy account SS58 (if set, wraps all extrinsics in Proxy.proxy).
    pub proxy: Option<String>,
    /// Default live polling interval in seconds.
    pub live_interval: Option<u64>,
    /// Batch mode default (never prompt for input).
    pub batch: Option<bool>,
    /// Per-subnet spending limits in TAO (key = netuid as string).
    pub spending_limits: Option<std::collections::HashMap<String, f64>>,
}

impl Config {
    /// Default config file path.
    pub fn default_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".agcli")
            .join("config.toml")
    }

    /// Load config from the default path. Returns default if file doesn't exist.
    pub fn load() -> Self {
        Self::load_from(&Self::default_path()).unwrap_or_default()
    }

    /// Load config from a specific path.
    pub fn load_from(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save config to the default path.
    pub fn save(&self) -> Result<()> {
        self.save_to(&Self::default_path())
    }

    /// Save config to a specific path.
    pub fn save_to(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_roundtrip() {
        let cfg = Config {
            network: Some("finney".to_string()),
            wallet: Some("mywallet".to_string()),
            hotkey: Some("default".to_string()),
            ..Default::default()
        };
        let s = toml::to_string_pretty(&cfg).unwrap();
        let parsed: Config = toml::from_str(&s).unwrap();
        assert_eq!(parsed.network.as_deref(), Some("finney"));
        assert_eq!(parsed.wallet.as_deref(), Some("mywallet"));
    }

    #[test]
    fn missing_file_returns_default() {
        let cfg = Config::load_from(Path::new("/nonexistent/path/config.toml")).unwrap();
        assert!(cfg.network.is_none());
    }

    /// Multiple concurrent writers to the same config file should all succeed.
    #[test]
    fn concurrent_config_writes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");

        let mut handles = Vec::new();
        for i in 0..10u32 {
            let p = path.clone();
            handles.push(std::thread::spawn(move || {
                let cfg = Config {
                    network: Some(format!("net-{}", i)),
                    wallet: Some(format!("wallet-{}", i)),
                    ..Default::default()
                };
                cfg.save_to(&p)
            }));
        }

        let mut errors = 0;
        for h in handles {
            if h.join().unwrap().is_err() {
                errors += 1;
            }
        }
        // All writes should succeed (even if they overwrite each other)
        assert_eq!(errors, 0);

        // File should be parseable TOML afterward
        let final_cfg = Config::load_from(&path).unwrap();
        assert!(final_cfg.network.is_some());
    }

    /// Config with all fields populated roundtrips correctly.
    #[test]
    fn full_config_roundtrip() {
        let mut limits = std::collections::HashMap::new();
        limits.insert("1".to_string(), 100.0);
        limits.insert("18".to_string(), 50.0);

        let cfg = Config {
            network: Some("finney".into()),
            endpoint: Some("wss://custom:443".into()),
            wallet_dir: Some("/home/user/.bt".into()),
            wallet: Some("mywallet".into()),
            hotkey: Some("hk1".into()),
            output: Some("json".into()),
            proxy: Some("5GrwvaEF...".into()),
            live_interval: Some(30),
            batch: Some(true),
            spending_limits: Some(limits),
        };
        let s = toml::to_string_pretty(&cfg).unwrap();
        let parsed: Config = toml::from_str(&s).unwrap();
        assert_eq!(parsed.endpoint.as_deref(), Some("wss://custom:443"));
        assert_eq!(parsed.live_interval, Some(30));
        assert_eq!(parsed.batch, Some(true));
        assert!(parsed.spending_limits.as_ref().unwrap().contains_key("18"));
    }
}
