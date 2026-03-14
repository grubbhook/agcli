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
}
