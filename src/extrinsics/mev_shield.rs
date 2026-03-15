//! MEV Shield: encrypt extrinsics with ML-KEM-768 + XChaCha20-Poly1305.
//!
//! The MevShield pallet uses post-quantum ML-KEM-768 key encapsulation to protect
//! extrinsics from front-running. The block producer holds the decapsulation key
//! and decrypts after inclusion.
//!
//! Ciphertext format: `[u16 kem_ct_len LE][kem_ct (1088 bytes)][nonce (24 bytes)][aead_ct]`
//! Commitment: `Blake2s-256(plaintext)`

use anyhow::{Context, Result};
use blake2::{Blake2s256, Digest};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    XChaCha20Poly1305, XNonce,
};
use ml_kem::{EncodedSizeUser, KemCore, MlKem768};

/// Encrypt a plaintext extrinsic for MEV shield submission.
///
/// Returns `(commitment_h256, ciphertext)` where:
/// - `commitment_h256`: 32-byte Blake2s-256 hash of the plaintext (used as on-chain commitment)
/// - `ciphertext`: encrypted blob in the format expected by `MevShield.submit_encrypted`
pub fn encrypt_for_mev_shield(
    ml_kem_public_key: &[u8],
    plaintext: &[u8],
) -> Result<([u8; 32], Vec<u8>)> {
    use kem::Encapsulate;

    // 1. Commitment: Blake2s-256 of plaintext
    let mut hasher = Blake2s256::new();
    hasher.update(plaintext);
    let commitment: [u8; 32] = hasher.finalize().into();

    // 2. Deserialize the ML-KEM-768 encapsulation key from bytes
    let ek_encoded: &ml_kem::Encoded<
        <MlKem768 as KemCore>::EncapsulationKey,
    > = ml_kem_public_key.try_into().map_err(|_| {
        anyhow::anyhow!(
            "Invalid ML-KEM-768 public key: expected {} bytes, got {}",
            std::mem::size_of::<ml_kem::Encoded<<MlKem768 as KemCore>::EncapsulationKey>>(),
            ml_kem_public_key.len()
        )
    })?;
    let ek = <MlKem768 as KemCore>::EncapsulationKey::from_bytes(ek_encoded);

    // 3. ML-KEM-768 encapsulation: derive shared secret + KEM ciphertext
    let mut rng = rand::thread_rng();
    let (kem_ct, shared_secret) = ek
        .encapsulate(&mut rng)
        .map_err(|e| anyhow::anyhow!("ML-KEM encapsulation failed: {:?}", e))?;

    // 4. Derive AEAD key from shared secret (32 bytes)
    let aead_key: &[u8; 32] = shared_secret
        .as_slice()
        .first_chunk::<32>()
        .context("Shared secret too short for AEAD key")?;
    let cipher = XChaCha20Poly1305::new_from_slice(aead_key)
        .map_err(|e| anyhow::anyhow!("AEAD key init failed: {}", e))?;

    // 5. Generate random 24-byte nonce for XChaCha20
    let mut nonce_bytes = [0u8; 24];
    rand::RngCore::fill_bytes(&mut rng, &mut nonce_bytes);
    let nonce = XNonce::from_slice(&nonce_bytes);

    // 6. Encrypt plaintext with XChaCha20-Poly1305
    let aead_ct = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| anyhow::anyhow!("AEAD encryption failed: {}", e))?;

    // 7. Assemble ciphertext: [u16 kem_ct_len LE][kem_ct][nonce24][aead_ct]
    let kem_ct_bytes: &[u8] = kem_ct.as_slice();
    let kem_ct_len = kem_ct_bytes.len() as u16;
    let mut output = Vec::with_capacity(2 + kem_ct_bytes.len() + 24 + aead_ct.len());
    output.extend_from_slice(&kem_ct_len.to_le_bytes());
    output.extend_from_slice(kem_ct_bytes);
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&aead_ct);

    Ok((commitment, output))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_roundtrip_structure() {
        // Generate a test keypair
        let mut rng = rand::thread_rng();
        let (_dk, ek) = MlKem768::generate(&mut rng);
        let ek_bytes = ek.as_bytes();
        let plaintext = b"test extrinsic data for MEV shield";

        let (commitment1, ct1) = encrypt_for_mev_shield(ek_bytes.as_slice(), plaintext).unwrap();
        let (commitment2, ct2) = encrypt_for_mev_shield(ek_bytes.as_slice(), plaintext).unwrap();

        // Commitment is deterministic (same plaintext → same hash)
        assert_eq!(commitment1, commitment2);
        // Ciphertext is different each time (random nonce + random KEM)
        assert_ne!(ct1, ct2);

        // Verify structure: first 2 bytes are KEM CT length (1088 for ML-KEM-768)
        let kem_len = u16::from_le_bytes([ct1[0], ct1[1]]);
        assert_eq!(kem_len, 1088);
        // Total: 2 + 1088 + 24 + (plaintext.len() + 16 poly1305 tag)
        assert_eq!(ct1.len(), 2 + 1088 + 24 + plaintext.len() + 16);
    }

    #[test]
    fn encrypt_rejects_wrong_key_size() {
        let bad_key = vec![0u8; 100];
        let result = encrypt_for_mev_shield(&bad_key, b"test");
        assert!(result.is_err());
    }

    #[test]
    fn encrypt_decrypt_roundtrip() {
        use kem::Decapsulate;

        let mut rng = rand::thread_rng();
        let (dk, ek) = MlKem768::generate(&mut rng);
        let ek_bytes = ek.as_bytes();
        let plaintext = b"secret extrinsic payload for MEV shield testing";

        let (commitment, ciphertext) =
            encrypt_for_mev_shield(ek_bytes.as_slice(), plaintext).unwrap();

        // Verify commitment
        let mut hasher = Blake2s256::new();
        hasher.update(plaintext);
        let expected_commitment: [u8; 32] = hasher.finalize().into();
        assert_eq!(commitment, expected_commitment);

        // Parse ciphertext structure
        let kem_ct_len = u16::from_le_bytes([ciphertext[0], ciphertext[1]]) as usize;
        assert_eq!(kem_ct_len, 1088);
        let kem_ct_bytes = &ciphertext[2..2 + kem_ct_len];
        let nonce_bytes = &ciphertext[2 + kem_ct_len..2 + kem_ct_len + 24];
        let aead_ct = &ciphertext[2 + kem_ct_len + 24..];

        // Decapsulate shared secret
        let kem_ct: &ml_kem::Ciphertext<MlKem768> = kem_ct_bytes.try_into().unwrap();
        let shared_secret = dk.decapsulate(kem_ct).unwrap();

        // Decrypt
        let aead_key: &[u8; 32] = shared_secret.as_slice().first_chunk::<32>().unwrap();
        let cipher = XChaCha20Poly1305::new_from_slice(aead_key).unwrap();
        let nonce = XNonce::from_slice(nonce_bytes);
        let decrypted = cipher.decrypt(nonce, aead_ct).unwrap();

        assert_eq!(decrypted, plaintext);
    }
}
