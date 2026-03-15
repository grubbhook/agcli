//! Weight-setting extrinsics and commit-reveal hashing.
//!
//! Extrinsics implemented on `Client` in `chain/mod.rs`:
//! - `set_weights(netuid, uids, weights, version_key)`
//! - `commit_weights(netuid, commit_hash)`
//! - `reveal_weights(netuid, uids, values, salt, version_key)`

/// Compute the commit hash for weight commit-reveal (Blake2b-256).
///
/// This matches the on-chain verification algorithm. The hash is computed
/// over little-endian encoded uids, weights, and the salt as raw bytes.
pub fn compute_weight_commit_hash(
    uids: &[u16],
    values: &[u16],
    salt: &[u8],
) -> Result<[u8; 32], blake2::digest::InvalidOutputSize> {
    use blake2::digest::{Update, VariableOutput};
    let mut hasher = blake2::Blake2bVar::new(32)?;
    for uid in uids {
        hasher.update(&uid.to_le_bytes());
    }
    for val in values {
        hasher.update(&val.to_le_bytes());
    }
    hasher.update(salt);
    let mut hash = [0u8; 32];
    hasher.finalize_variable(&mut hash).map_err(|_| {
        // finalize_variable with a correctly-sized buffer can't fail,
        // but we propagate for correctness
        blake2::digest::InvalidOutputSize
    })?;
    Ok(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commit_hash_deterministic() {
        let h1 = compute_weight_commit_hash(&[1, 2], &[100, 200], b"salt42").unwrap();
        let h2 = compute_weight_commit_hash(&[1, 2], &[100, 200], b"salt42").unwrap();
        assert_eq!(h1, h2);
    }

    #[test]
    fn commit_hash_changes_with_salt() {
        let h1 = compute_weight_commit_hash(&[1, 2], &[100, 200], b"salt42").unwrap();
        let h2 = compute_weight_commit_hash(&[1, 2], &[100, 200], b"salt99").unwrap();
        assert_ne!(h1, h2);
    }

    #[test]
    fn commit_hash_matches_inline_blake2b() {
        // Verify our function produces the same result as the inline Blake2b
        // code that was previously duplicated in weights_cmds.rs
        use blake2::digest::{Update, VariableOutput};
        let uids: Vec<u16> = vec![1, 2, 3];
        let wts: Vec<u16> = vec![100, 200, 300];
        let salt = b"testSalt123";

        let mut hasher = blake2::Blake2bVar::new(32).unwrap();
        for u in &uids {
            hasher.update(&u.to_le_bytes());
        }
        for w in &wts {
            hasher.update(&w.to_le_bytes());
        }
        hasher.update(salt);
        let mut expected = [0u8; 32];
        hasher.finalize_variable(&mut expected).unwrap();

        let actual = compute_weight_commit_hash(&uids, &wts, salt).unwrap();
        assert_eq!(actual, expected);
    }
}
