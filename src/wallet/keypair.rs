//! SR25519 keypair utilities — generation, derivation, SS58 encoding.

use anyhow::{Context, Result};
use bip39::{Language, Mnemonic};
use rand::Rng;
use sp_core::{crypto::Ss58Codec, sr25519, Pair};

/// Generate a new mnemonic and derive the SR25519 keypair.
/// Returns (pair, mnemonic_phrase).
pub fn generate_mnemonic_keypair() -> Result<(sr25519::Pair, String)> {
    let mut entropy = [0u8; 16]; // 128 bits = 12 words
    rand::thread_rng().fill(&mut entropy);
    let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)
        .map_err(|e| anyhow::anyhow!("mnemonic generation failed: {:?}", e))?;
    let phrase = mnemonic.to_string();
    let pair = pair_from_mnemonic(&phrase)?;
    Ok((pair, phrase))
}

/// Derive SR25519 keypair from a BIP-39 mnemonic phrase.
pub fn pair_from_mnemonic(mnemonic: &str) -> Result<sr25519::Pair> {
    sr25519::Pair::from_phrase(mnemonic, None)
        .map(|(pair, _seed)| pair)
        .map_err(|e| anyhow::anyhow!("Invalid mnemonic: {:?}", e))
}

/// Derive SR25519 keypair from a seed hex string (0x-prefixed or plain).
pub fn pair_from_seed_hex(seed: &str) -> Result<sr25519::Pair> {
    let seed = seed.strip_prefix("0x").unwrap_or(seed);
    let bytes = hex::decode(seed).context("Invalid hex seed")?;
    let seed_arr: [u8; 32] = bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Seed must be 32 bytes"))?;
    Ok(sr25519::Pair::from_seed(&seed_arr))
}

/// Encode an SR25519 public key to SS58 format.
pub fn to_ss58(public: &sr25519::Public, prefix: u16) -> String {
    public.to_ss58check_with_version(sp_core::crypto::Ss58AddressFormat::custom(prefix))
}

/// Decode an SS58 address to an SR25519 public key.
pub fn from_ss58(address: &str) -> Result<sr25519::Public> {
    sr25519::Public::from_ss58check(address)
        .map_err(|_| {
            let trimmed = address.trim();
            if trimmed.is_empty() {
                anyhow::anyhow!("Empty address. Provide a valid Bittensor SS58 address (starts with '5').")
            } else if trimmed.len() < 10 {
                anyhow::anyhow!("Invalid SS58 address '{}' — too short. Bittensor addresses are 48 characters starting with '5'.", trimmed)
            } else {
                anyhow::anyhow!("Invalid SS58 address '{}'. Expected a 48-character Bittensor address starting with '5'.\n  Tip: verify the address on taostats.io or use `agcli wallet show` to get your address.", crate::utils::short_ss58(trimmed))
            }
        })
}

/// Verify that an SS58 address is valid.
pub fn is_valid_ss58(address: &str) -> bool {
    sr25519::Public::from_ss58check(address).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_and_roundtrip() {
        let (pair, mnemonic) = generate_mnemonic_keypair().unwrap();
        let pair2 = pair_from_mnemonic(&mnemonic).unwrap();
        assert_eq!(pair.public(), pair2.public());
    }

    #[test]
    fn ss58_roundtrip() {
        let (pair, _) = generate_mnemonic_keypair().unwrap();
        let addr = to_ss58(&pair.public(), 42);
        let pub2 = from_ss58(&addr).unwrap();
        assert_eq!(pair.public(), pub2);
    }
}
