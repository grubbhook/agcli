//! Keyfile I/O — encrypted coldkeys, plaintext hotkeys.
//!
//! Coldkeys are encrypted with AES-256-GCM using a key derived from
//! Argon2id (matching the Python bittensor-wallet encryption scheme).

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{Context, Result};
use argon2::Argon2;
use rand::RngCore;
use sp_core::sr25519;
use std::fs;
use std::path::Path;

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;

/// Write mnemonic encrypted with password.
pub fn write_encrypted_keyfile(path: &Path, mnemonic: &str, password: &str) -> Result<()> {
    let mut salt = [0u8; SALT_LEN];
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut salt);
    rand::thread_rng().fill_bytes(&mut nonce_bytes);

    let key = derive_key(password, &salt)?;
    let cipher =
        Aes256Gcm::new_from_slice(&key).map_err(|e| anyhow::anyhow!("cipher init: {}", e))?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, mnemonic.as_bytes())
        .map_err(|e| anyhow::anyhow!("encryption failed: {}", e))?;

    // Format: salt || nonce || ciphertext
    let mut data = Vec::with_capacity(SALT_LEN + NONCE_LEN + ciphertext.len());
    data.extend_from_slice(&salt);
    data.extend_from_slice(&nonce_bytes);
    data.extend_from_slice(&ciphertext);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, data).context("write keyfile")?;
    Ok(())
}

/// Read and decrypt an encrypted keyfile, returning the mnemonic.
pub fn read_encrypted_keyfile(path: &Path, password: &str) -> Result<String> {
    let data =
        fs::read(path).with_context(|| format!("Cannot read keyfile at '{}'", path.display()))?;
    if data.len() < SALT_LEN + NONCE_LEN {
        anyhow::bail!("Keyfile '{}' is corrupted (too short). Re-create your wallet with `agcli wallet create`.", path.display());
    }

    let (salt, rest) = data.split_at(SALT_LEN);
    let (nonce_bytes, ciphertext) = rest.split_at(NONCE_LEN);

    let key = derive_key(password, salt)?;
    let cipher =
        Aes256Gcm::new_from_slice(&key).map_err(|e| anyhow::anyhow!("cipher init: {}", e))?;
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| anyhow::anyhow!("Decryption failed — wrong password. If you forgot your password, restore from your mnemonic with `agcli wallet regen-coldkey`."))?;

    String::from_utf8(plaintext).context("mnemonic is not valid UTF-8")
}

/// Write a plaintext keyfile (for hotkeys).
pub fn write_keyfile(path: &Path, mnemonic: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, mnemonic).context("write keyfile")?;
    Ok(())
}

/// Read a plaintext keyfile.
pub fn read_keyfile(path: &Path) -> Result<String> {
    fs::read_to_string(path).context("read keyfile")
}

/// Write public key to a file (hex-encoded).
pub fn write_public_key(path: &Path, public: &sr25519::Public) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, hex::encode(public.0)).context("write public key")?;
    Ok(())
}

/// Read a public key from file.
pub fn read_public_key(path: &Path) -> Result<sr25519::Public> {
    let hex_str = fs::read_to_string(path).context("read public key file")?;
    let hex_str = hex_str.trim().strip_prefix("0x").unwrap_or(hex_str.trim());
    let bytes = hex::decode(hex_str).context("invalid hex")?;
    if bytes.len() != 32 {
        anyhow::bail!("public key must be 32 bytes, got {}", bytes.len());
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(sr25519::Public::from_raw(arr))
}

fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; KEY_LEN]> {
    let mut key = [0u8; KEY_LEN];
    Argon2::default()
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| anyhow::anyhow!("key derivation failed: {}", e))?;
    Ok(key)
}

// ──────── Python bittensor-wallet compatibility ────────

/// Magic prefix for NaCl-encrypted keyfiles (Python bittensor-wallet format).
const NACL_PREFIX: &[u8] = b"$NACL";

/// Fixed salt used by the Python bittensor-wallet for Argon2i KDF.
const NACL_SALT: [u8; 16] = [
    0x13, 0x71, 0x83, 0xdf, 0xf1, 0x5a, 0x09, 0xbc, 0x9c, 0x90, 0xb5, 0x51, 0x87, 0x39, 0xe9, 0xb1,
];

/// Check if keyfile data is in the Python NaCl format.
pub fn is_nacl_encrypted(data: &[u8]) -> bool {
    data.starts_with(NACL_PREFIX)
}

/// Read and decrypt a Python bittensor-wallet NaCl-encrypted keyfile.
/// Format: "$NACL" prefix + SecretBox encrypted data (nonce + ciphertext + MAC).
/// KDF: Argon2i with opslimit=4, memlimit=1GiB, fixed NACL_SALT.
pub fn read_python_keyfile(path: &Path, password: &str) -> Result<String> {
    let data = fs::read(path).context("read keyfile")?;
    decrypt_nacl_keyfile_data(&data, password)
}

/// Decrypt NaCl keyfile data (with or without $NACL prefix).
pub fn decrypt_nacl_keyfile_data(data: &[u8], password: &str) -> Result<String> {
    let encrypted = if data.starts_with(NACL_PREFIX) {
        &data[NACL_PREFIX.len()..]
    } else {
        data
    };

    // Derive key using Argon2i (matching Python: opslimit=4, memlimit=1GiB)
    let argon2_params = argon2::Params::new(
        1_048_576, // 1 GiB in KiB (1024*1024)
        4,         // t_cost (opslimit)
        1,         // p_cost (parallelism)
        Some(KEY_LEN),
    )
    .map_err(|e| anyhow::anyhow!("argon2 params error: {}", e))?;
    let argon2 = argon2::Argon2::new(
        argon2::Algorithm::Argon2i,
        argon2::Version::V0x13,
        argon2_params,
    );
    let mut key = [0u8; KEY_LEN];
    argon2
        .hash_password_into(password.as_bytes(), &NACL_SALT, &mut key)
        .map_err(|e| anyhow::anyhow!("key derivation failed: {}", e))?;

    // Decrypt using XSalsa20-Poly1305 (NaCl SecretBox)
    // PyNaCl SecretBox format: nonce (24 bytes) + ciphertext (with MAC)
    use crypto_secretbox::{
        aead::{Aead, KeyInit},
        XSalsa20Poly1305,
    };
    if encrypted.len() < 24 {
        anyhow::bail!("NaCl keyfile too short");
    }
    let (nonce_bytes, ciphertext) = encrypted.split_at(24);
    let cipher = XSalsa20Poly1305::new_from_slice(&key)
        .map_err(|e| anyhow::anyhow!("cipher init: {}", e))?;
    let nonce = crypto_secretbox::Nonce::from_slice(nonce_bytes);
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|_| anyhow::anyhow!("Decryption failed — wrong password for Python wallet. If you forgot your password, restore from your mnemonic with `agcli wallet regen-coldkey`."))?;

    String::from_utf8(plaintext).context("decrypted data is not valid UTF-8")
}

/// Detect keyfile format and decrypt accordingly.
/// Supports both agcli's AES-256-GCM format and Python's NaCl SecretBox format.
pub fn read_any_encrypted_keyfile(path: &Path, password: &str) -> Result<String> {
    let data = fs::read(path).context("read keyfile")?;
    if is_nacl_encrypted(&data) {
        decrypt_nacl_keyfile_data(&data, password)
    } else {
        // Try our AES-256-GCM format
        read_encrypted_keyfile(path, password)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test_coldkey");
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let password = "test_password_123";

        write_encrypted_keyfile(&path, mnemonic, password).unwrap();
        let recovered = read_encrypted_keyfile(&path, password).unwrap();
        assert_eq!(mnemonic, recovered);
    }

    #[test]
    fn wrong_password_fails() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test_coldkey");
        let mnemonic = "test mnemonic phrase";
        write_encrypted_keyfile(&path, mnemonic, "correct").unwrap();
        assert!(read_encrypted_keyfile(&path, "wrong").is_err());
    }
}
