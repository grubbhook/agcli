//! Wallet management — create, open, import, encrypt/decrypt keypairs.
//!
//! Bittensor wallets consist of:
//! - **Coldkey**: Signing key for high-value operations (staking, transfers).
//!   Stored encrypted on disk.
//! - **Hotkey**: Signing key for low-value operations (weights, serving).
//!   Stored unencrypted for automated use.
//!
//! Keys are SR25519 keypairs (Substrate default).

pub mod keyfile;
pub mod keypair;

use anyhow::{Context, Result};
use sp_core::{sr25519, Pair as _};
use std::path::{Path, PathBuf};

/// A Bittensor wallet containing coldkey and hotkey.
pub struct Wallet {
    /// Display name.
    pub name: String,
    /// Path to the wallet directory.
    pub path: PathBuf,
    /// Decrypted coldkey (loaded lazily).
    coldkey: Option<sr25519::Pair>,
    /// Hotkey pair.
    hotkey: Option<sr25519::Pair>,
    /// Coldkey SS58 address (always available if public key is known).
    coldkey_ss58: Option<String>,
    /// Hotkey SS58 address.
    hotkey_ss58: Option<String>,
}

impl std::fmt::Debug for Wallet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Wallet")
            .field("name", &self.name)
            .field("path", &self.path)
            .field("coldkey_ss58", &self.coldkey_ss58)
            .field("hotkey_ss58", &self.hotkey_ss58)
            .finish()
    }
}

impl Wallet {
    /// Open an existing wallet from disk.
    ///
    /// ```rust,no_run
    /// let w = agcli::Wallet::open("~/.bittensor/wallets/default").unwrap();
    /// ```
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = expand_tilde(path.as_ref());
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "default".to_string());

        let coldkey_ss58 = match keyfile::read_public_key(&path.join("coldkeypub.txt")) {
            Ok(pk) => Some(keypair::to_ss58(&pk, 42)),
            Err(e) => {
                tracing::debug!(wallet = %name, error = %e, "Could not read coldkeypub.txt (not yet created or corrupted)");
                None
            }
        };
        let hotkey_ss58 = match keyfile::read_public_key(&path.join("hotkeys").join("default")) {
            Ok(pk) => Some(keypair::to_ss58(&pk, 42)),
            Err(e) => {
                tracing::debug!(wallet = %name, error = %e, "Could not read default hotkey public key");
                None
            }
        };

        Ok(Self {
            name,
            path,
            coldkey: None,
            hotkey: None,
            coldkey_ss58,
            hotkey_ss58,
        })
    }

    /// Create a new wallet with fresh keys.
    pub fn create(
        wallet_dir: impl AsRef<Path>,
        name: &str,
        password: &str,
        hotkey_name: &str,
    ) -> Result<Self> {
        let dir = expand_tilde(wallet_dir.as_ref()).join(name);
        std::fs::create_dir_all(dir.join("hotkeys"))?;

        let coldkey = keypair::generate_mnemonic_keypair()?;
        let hotkey = keypair::generate_mnemonic_keypair()?;

        let coldkey_ss58 = keypair::to_ss58(&coldkey.0.public(), 42);
        let hotkey_ss58 = keypair::to_ss58(&hotkey.0.public(), 42);

        // Save encrypted coldkey
        keyfile::write_encrypted_keyfile(
            &dir.join("coldkey"),
            &coldkey.1, // mnemonic
            password,
        )?;

        // Save coldkey public key
        keyfile::write_public_key(&dir.join("coldkeypub.txt"), &coldkey.0.public())?;

        // Save hotkey (unencrypted)
        keyfile::write_keyfile(&dir.join("hotkeys").join(hotkey_name), &hotkey.1)?;

        Ok(Self {
            name: name.to_string(),
            path: dir,
            coldkey: Some(coldkey.0),
            hotkey: Some(hotkey.0),
            coldkey_ss58: Some(coldkey_ss58),
            hotkey_ss58: Some(hotkey_ss58),
        })
    }

    /// Import wallet from mnemonic.
    pub fn import_from_mnemonic(
        wallet_dir: impl AsRef<Path>,
        name: &str,
        mnemonic: &str,
        password: &str,
    ) -> Result<Self> {
        let dir = expand_tilde(wallet_dir.as_ref()).join(name);
        std::fs::create_dir_all(dir.join("hotkeys"))?;

        let pair = keypair::pair_from_mnemonic(mnemonic)?;
        let ss58 = keypair::to_ss58(&pair.public(), 42);

        keyfile::write_encrypted_keyfile(&dir.join("coldkey"), mnemonic, password)?;
        keyfile::write_public_key(&dir.join("coldkeypub.txt"), &pair.public())?;

        Ok(Self {
            name: name.to_string(),
            path: dir,
            coldkey: Some(pair),
            hotkey: None,
            coldkey_ss58: Some(ss58),
            hotkey_ss58: None,
        })
    }

    /// Unlock the coldkey with a password.
    /// Auto-detects keyfile format (agcli AES-256-GCM or Python NaCl SecretBox).
    pub fn unlock_coldkey(&mut self, password: &str) -> Result<()> {
        let data = keyfile::read_any_encrypted_keyfile(&self.path.join("coldkey"), password)
            .context("Failed to decrypt coldkey")?;
        // The decrypted data may be a mnemonic or a JSON keypair (Python format)
        let pair = if data.trim().starts_with('{') {
            // Python bittensor-wallet stores JSON: {"secretSeed": "0x...", ...} or
            // {"ss58Address": "...", "secretPhrase": "...", ...}
            let v: serde_json::Value =
                serde_json::from_str(data.trim()).context("Failed to parse Python keyfile JSON")?;
            if let Some(seed) = v.get("secretSeed").and_then(|s| s.as_str()) {
                keypair::pair_from_seed_hex(seed)?
            } else if let Some(phrase) = v.get("secretPhrase").and_then(|s| s.as_str()) {
                keypair::pair_from_mnemonic(phrase)?
            } else {
                anyhow::bail!("Python keyfile JSON has no secretSeed or secretPhrase");
            }
        } else {
            keypair::pair_from_mnemonic(data.trim())?
        };
        self.coldkey_ss58 = Some(keypair::to_ss58(&pair.public(), 42));
        self.coldkey = Some(pair);
        Ok(())
    }

    /// Load the hotkey (unencrypted).
    /// Supports both mnemonic plaintext and Python JSON keypair format.
    pub fn load_hotkey(&mut self, hotkey_name: &str) -> Result<()> {
        let data = keyfile::read_keyfile(&self.path.join("hotkeys").join(hotkey_name))?;
        let pair = if data.trim().starts_with('{') {
            let v: serde_json::Value =
                serde_json::from_str(data.trim()).context("Failed to parse hotkey JSON")?;
            if let Some(seed) = v.get("secretSeed").and_then(|s| s.as_str()) {
                keypair::pair_from_seed_hex(seed)?
            } else if let Some(phrase) = v.get("secretPhrase").and_then(|s| s.as_str()) {
                keypair::pair_from_mnemonic(phrase)?
            } else {
                anyhow::bail!("Hotkey JSON has no secretSeed or secretPhrase");
            }
        } else {
            keypair::pair_from_mnemonic(data.trim())?
        };
        self.hotkey_ss58 = Some(keypair::to_ss58(&pair.public(), 42));
        self.hotkey = Some(pair);
        Ok(())
    }

    /// Get the coldkey pair (must be unlocked).
    pub fn coldkey(&self) -> Result<&sr25519::Pair> {
        self.coldkey
            .as_ref()
            .context("Coldkey not unlocked. Call unlock_coldkey() first.")
    }

    /// Get the hotkey pair (must be loaded).
    pub fn hotkey(&self) -> Result<&sr25519::Pair> {
        self.hotkey
            .as_ref()
            .context("Hotkey not loaded. Call load_hotkey() first.")
    }

    /// Coldkey SS58 address.
    pub fn coldkey_ss58(&self) -> Option<&str> {
        self.coldkey_ss58.as_deref()
    }

    /// Hotkey SS58 address.
    pub fn hotkey_ss58(&self) -> Option<&str> {
        self.hotkey_ss58.as_deref()
    }

    /// Get the coldkey public key bytes.
    pub fn coldkey_public(&self) -> sp_core::sr25519::Public {
        self.coldkey
            .as_ref()
            .map(|p| p.public())
            .unwrap_or_default()
    }

    /// List all hotkeys in the wallet.
    pub fn list_hotkeys(&self) -> Result<Vec<String>> {
        let hotkey_dir = self.path.join("hotkeys");
        if !hotkey_dir.exists() {
            return Ok(vec![]);
        }
        let mut names = Vec::new();
        for entry in std::fs::read_dir(hotkey_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                if let Some(name) = entry.file_name().to_str() {
                    names.push(name.to_string());
                }
            }
        }
        names.sort();
        Ok(names)
    }

    /// List all wallets in a directory.
    pub fn list_wallets(wallet_dir: impl AsRef<Path>) -> Result<Vec<String>> {
        let dir = expand_tilde(wallet_dir.as_ref());
        let mut names = Vec::new();
        if !dir.exists() {
            return Ok(names);
        }
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    names.push(name.to_string());
                }
            }
        }
        names.sort();
        Ok(names)
    }
}

fn expand_tilde(path: &Path) -> PathBuf {
    if let Ok(stripped) = path.strip_prefix("~") {
        if let Some(home) = dirs::home_dir() {
            return home.join(stripped);
        }
    }
    path.to_path_buf()
}
