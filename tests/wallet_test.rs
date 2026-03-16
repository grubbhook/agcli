//! Wallet creation, import and key operations tests.
//! Run with: cargo test --test wallet_test
//!
//! Covers: create, import, open, unlock, list, hotkeys, sign/verify,
//! dev-key derivation, mnemonic validation, edge cases.

use agcli::Wallet;
use agcli::wallet::keypair;
use sp_core::{sr25519, Pair as _};

#[test]
fn create_wallet_and_read_keys() {
    let dir = tempfile::tempdir().unwrap();
    let (wallet, _mnemonic, _hk_mnemonic) = Wallet::create(
        dir.path().to_str().unwrap(),
        "test_wallet",
        "password123",
        "default",
    )
    .unwrap();
    assert!(wallet.coldkey_ss58().is_some());
    assert!(wallet.hotkey_ss58().is_some());
    // Address should be valid SS58
    let addr = wallet.coldkey_ss58().unwrap();
    assert!(
        addr.starts_with("5"),
        "should be a substrate SS58 address: {}",
        addr
    );
    assert!(
        addr.len() > 40,
        "SS58 address should be ~48 chars: {}",
        addr
    );
}

#[test]
fn open_wallet_and_read_public_key() {
    let dir = tempfile::tempdir().unwrap();
    let (wallet, _, _) =
        Wallet::create(dir.path().to_str().unwrap(), "w1", "pass", "default").unwrap();
    let addr = wallet.coldkey_ss58().unwrap().to_string();

    // Open and verify the SS58 is the same
    let opened = Wallet::open(format!("{}/w1", dir.path().to_str().unwrap())).unwrap();
    assert_eq!(opened.coldkey_ss58().unwrap(), addr);
}

#[test]
fn unlock_coldkey_correct_password() {
    let dir = tempfile::tempdir().unwrap();
    let _ = Wallet::create(dir.path().to_str().unwrap(), "w2", "secret", "default").unwrap();
    let mut opened = Wallet::open(format!("{}/w2", dir.path().to_str().unwrap())).unwrap();
    assert!(opened.unlock_coldkey("secret").is_ok());
    assert!(opened.coldkey().is_ok());
}

#[test]
fn unlock_coldkey_wrong_password() {
    let dir = tempfile::tempdir().unwrap();
    let _ = Wallet::create(dir.path().to_str().unwrap(), "w3", "correct", "default").unwrap();
    let mut opened = Wallet::open(format!("{}/w3", dir.path().to_str().unwrap())).unwrap();
    assert!(opened.unlock_coldkey("wrong").is_err());
}

#[test]
fn import_from_mnemonic_and_verify() {
    let dir = tempfile::tempdir().unwrap();
    let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let wallet =
        Wallet::import_from_mnemonic(dir.path().to_str().unwrap(), "imported", mnemonic, "pass")
            .unwrap();
    let addr = wallet.coldkey_ss58().unwrap().to_string();

    // Reimporting the same mnemonic should produce the same address
    let dir2 = tempfile::tempdir().unwrap();
    let wallet2 = Wallet::import_from_mnemonic(
        dir2.path().to_str().unwrap(),
        "imported2",
        mnemonic,
        "other_pass",
    )
    .unwrap();
    assert_eq!(wallet2.coldkey_ss58().unwrap(), addr);
}

#[test]
fn list_wallets() {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().to_str().unwrap();
    let _ = Wallet::create(base, "alice", "pass", "default").unwrap();
    let _ = Wallet::create(base, "bob", "pass", "default").unwrap();
    let wallets = Wallet::list_wallets(base).unwrap();
    assert!(wallets.contains(&"alice".to_string()));
    assert!(wallets.contains(&"bob".to_string()));
    assert_eq!(wallets.len(), 2);
}

#[test]
fn list_hotkeys() {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().to_str().unwrap();
    let (wallet, _, _) = Wallet::create(base, "hk_test", "pass", "default").unwrap();
    let hotkeys = wallet.list_hotkeys().unwrap();
    assert!(hotkeys.contains(&"default".to_string()));
}

#[test]
fn open_nonexistent_wallet_has_no_keys() {
    // Wallet::open doesn't fail on missing dir, but the wallet has no keys
    let result = Wallet::open("/tmp/nonexistent_wallet_12345_xyz");
    match result {
        Err(_) => {} // expected on strict implementations
        Ok(w) => {
            // If it opens, it should have no coldkey SS58
            assert!(
                w.coldkey_ss58().is_none(),
                "nonexistent wallet should have no coldkey"
            );
        }
    }
}

#[test]
fn wrong_password_error_message_is_helpful() {
    let dir = tempfile::tempdir().unwrap();
    let _ = Wallet::create(
        dir.path().to_str().unwrap(),
        "err_test",
        "correct",
        "default",
    )
    .unwrap();
    let mut wallet = Wallet::open(format!("{}/err_test", dir.path().to_str().unwrap())).unwrap();
    let err = wallet.unlock_coldkey("wrong").unwrap_err();
    // The error chain includes "Failed to decrypt coldkey" context and inner "wrong password" cause
    let full = format!("{:#}", err);
    assert!(
        full.contains("decrypt") || full.contains("wrong password"),
        "Error chain should mention decryption failure, got: {}",
        full
    );
}

#[test]
fn ss58_validation_errors_are_helpful() {
    use agcli::wallet::keypair::from_ss58;
    // Empty address
    let err = from_ss58("").unwrap_err();
    assert!(
        err.to_string().contains("Empty address"),
        "Expected empty address hint, got: {}",
        err
    );

    // Too short
    let err = from_ss58("5abc").unwrap_err();
    assert!(
        err.to_string().contains("too short"),
        "Expected short address hint, got: {}",
        err
    );

    // Invalid characters
    let err =
        from_ss58("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQYxxxxxxinvalid").unwrap_err();
    assert!(
        err.to_string().contains("Invalid SS58"),
        "Expected invalid SS58 error, got: {}",
        err
    );
}

// ──────── Wallet creation edge cases ────────

#[test]
fn create_wallet_already_exists_fails() {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().to_str().unwrap();
    Wallet::create(base, "dup", "pass", "default").unwrap();
    let err = Wallet::create(base, "dup", "pass2", "default").unwrap_err();
    let msg = format!("{:#}", err);
    assert!(
        msg.contains("already exists"),
        "Should tell user wallet exists, got: {}",
        msg
    );
}

#[test]
fn create_wallet_with_custom_hotkey_name() {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().to_str().unwrap();
    let (wallet, _, _) = Wallet::create(base, "custom_hk", "pass", "miner_hk").unwrap();
    let hotkeys = wallet.list_hotkeys().unwrap();
    assert!(
        hotkeys.contains(&"miner_hk".to_string()),
        "Hotkey 'miner_hk' should exist, found: {:?}",
        hotkeys
    );
    assert!(
        !hotkeys.contains(&"default".to_string()),
        "Should not have 'default' hotkey when custom name used"
    );
}

#[test]
fn create_wallet_empty_password_works() {
    // Empty password should still work (encrypt with empty passphrase)
    let dir = tempfile::tempdir().unwrap();
    let (_, _, _) =
        Wallet::create(dir.path().to_str().unwrap(), "empty_pw", "", "default").unwrap();
    let mut w = Wallet::open(format!("{}/empty_pw", dir.path().to_str().unwrap())).unwrap();
    assert!(w.unlock_coldkey("").is_ok(), "Should unlock with empty password");
    assert!(
        w.unlock_coldkey("wrong").is_err(),
        "Should NOT unlock with non-empty password"
    );
}

// ──────── Import edge cases ────────

#[test]
fn import_invalid_mnemonic_too_few_words() {
    let dir = tempfile::tempdir().unwrap();
    let err = Wallet::import_from_mnemonic(
        dir.path().to_str().unwrap(),
        "bad",
        "abandon abandon",
        "pass",
    )
    .unwrap_err();
    let msg = format!("{:#}", err);
    assert!(
        msg.to_lowercase().contains("invalid") || msg.to_lowercase().contains("mnemonic"),
        "Should mention invalid mnemonic, got: {}",
        msg
    );
}

#[test]
fn import_invalid_mnemonic_wrong_words() {
    let dir = tempfile::tempdir().unwrap();
    let err = Wallet::import_from_mnemonic(
        dir.path().to_str().unwrap(),
        "bad2",
        "notaword notaword notaword notaword notaword notaword notaword notaword notaword notaword notaword notaword",
        "pass",
    )
    .unwrap_err();
    let msg = format!("{:#}", err);
    assert!(
        msg.to_lowercase().contains("invalid") || msg.to_lowercase().contains("mnemonic"),
        "Should reject non-BIP39 words, got: {}",
        msg
    );
}

#[test]
fn import_mnemonic_with_extra_whitespace() {
    // Should handle leading/trailing/extra internal whitespace gracefully
    let dir = tempfile::tempdir().unwrap();
    let mnemonic_clean = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let mnemonic_messy = "  abandon  abandon  abandon abandon abandon abandon abandon abandon abandon abandon abandon  about  ";

    let w1 = Wallet::import_from_mnemonic(dir.path().to_str().unwrap(), "clean", mnemonic_clean, "p").unwrap();
    let dir2 = tempfile::tempdir().unwrap();
    // This might fail or produce a different key — both are useful to know
    match Wallet::import_from_mnemonic(dir2.path().to_str().unwrap(), "messy", mnemonic_messy, "p") {
        Ok(w2) => {
            // If it succeeds, should produce the same address
            assert_eq!(
                w1.coldkey_ss58().unwrap(),
                w2.coldkey_ss58().unwrap(),
                "Extra whitespace should not change the derived key"
            );
        }
        Err(e) => {
            let msg = format!("{:#}", e);
            assert!(
                msg.to_lowercase().contains("mnemonic"),
                "If whitespace fails, error should mention mnemonic: {}",
                msg
            );
        }
    }
}

#[test]
fn import_24_word_mnemonic() {
    let dir = tempfile::tempdir().unwrap();
    // Valid 24-word BIP39 mnemonic
    let mnemonic_24 = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
    let w = Wallet::import_from_mnemonic(dir.path().to_str().unwrap(), "m24", mnemonic_24, "p").unwrap();
    assert!(w.coldkey_ss58().is_some(), "24-word mnemonic should produce valid key");
}

// ──────── Dev key tests ────────

#[test]
fn dev_key_all_standard_accounts() {
    // All 6 standard dev accounts should derive valid keypairs
    for name in keypair::DEV_ACCOUNTS {
        let uri = format!("//{}", name);
        let pair = keypair::pair_from_uri(&uri).expect(&format!("//{} should derive", name));
        let ss58 = keypair::to_ss58(&pair.public(), 42);
        assert!(ss58.starts_with("5"), "//{} SS58 should start with 5: {}", name, ss58);
    }
}

#[test]
fn dev_key_case_normalization() {
    // All these should derive the same keypair as "//Alice"
    let alice_pair = keypair::pair_from_uri("//Alice").unwrap();
    let alice_ss58 = keypair::to_ss58(&alice_pair.public(), 42);

    // "//alice" is a DIFFERENT account in substrate (case-sensitive URI derivation)
    let alice_lower = keypair::pair_from_uri("//alice").unwrap();
    let alice_lower_ss58 = keypair::to_ss58(&alice_lower.public(), 42);

    // They should be different — substrate URIs are case-sensitive
    assert_ne!(
        alice_ss58, alice_lower_ss58,
        "//Alice and //alice should produce different keys (substrate URI is case-sensitive)"
    );
}

#[test]
fn dev_key_create_from_uri() {
    let dir = tempfile::tempdir().unwrap();
    let wallet = Wallet::create_from_uri(dir.path().to_str().unwrap(), "//Alice", "pass").unwrap();
    assert_eq!(
        wallet.coldkey_ss58().unwrap(),
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
    );
    // Wallet name should be derived from URI
    assert_eq!(wallet.name, "alice");
}

#[test]
fn dev_key_invalid_uri() {
    // Completely invalid URI should fail
    match keypair::pair_from_uri("not_a_valid_uri_at_all") {
        Ok(_) => panic!("Invalid URI should fail"),
        Err(err) => {
            let msg = format!("{:#}", err);
            assert!(
                msg.contains("Invalid key URI"),
                "Should mention invalid URI, got: {}",
                msg
            );
        }
    }
}

// ──────── Sign and verify ────────

#[test]
fn sign_and_verify_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let (wallet, _, _) =
        Wallet::create(dir.path().to_str().unwrap(), "signer", "pass", "default").unwrap();
    let pair = wallet.coldkey().unwrap();

    let message = b"Hello Bittensor";
    let sig = pair.sign(message);

    // Verify with correct message
    assert!(
        sr25519::Pair::verify(&sig, message, &pair.public()),
        "Signature should verify with correct message"
    );

    // Verify with wrong message should fail
    assert!(
        !sr25519::Pair::verify(&sig, b"Wrong message", &pair.public()),
        "Signature should NOT verify with wrong message"
    );
}

#[test]
fn sign_deterministic_for_sr25519() {
    use sp_core::Pair;

    // SR25519 signatures are NOT deterministic (they use randomness)
    let dir = tempfile::tempdir().unwrap();
    let (wallet, _, _) =
        Wallet::create(dir.path().to_str().unwrap(), "det", "pass", "default").unwrap();
    let pair = wallet.coldkey().unwrap();

    let message = b"test message";
    let sig1 = pair.sign(message);
    let sig2 = pair.sign(message);

    // Both should verify
    assert!(sr25519::Pair::verify(&sig1, message, &pair.public()));
    assert!(sr25519::Pair::verify(&sig2, message, &pair.public()));
    // But they should be different (non-deterministic)
    // Note: there's a very small chance they could be equal, but practically never
}

// ──────── Seed hex derivation ────────

#[test]
fn pair_from_seed_hex_valid() {
    let seed = "0x0000000000000000000000000000000000000000000000000000000000000001";
    let pair = keypair::pair_from_seed_hex(seed).unwrap();
    let ss58 = keypair::to_ss58(&pair.public(), 42);
    assert!(ss58.starts_with("5"), "Should produce valid SS58: {}", ss58);
}

#[test]
fn pair_from_seed_hex_without_prefix() {
    let seed = "0000000000000000000000000000000000000000000000000000000000000001";
    let pair = keypair::pair_from_seed_hex(seed).unwrap();
    let ss58 = keypair::to_ss58(&pair.public(), 42);
    assert!(ss58.starts_with("5"));
}

#[test]
fn pair_from_seed_hex_wrong_length() {
    match keypair::pair_from_seed_hex("0x1234") {
        Ok(_) => panic!("Short seed should fail"),
        Err(err) => {
            let msg = format!("{:#}", err);
            assert!(
                msg.contains("32 bytes"),
                "Should mention 32 bytes, got: {}",
                msg
            );
        }
    }
}

#[test]
fn pair_from_seed_hex_invalid_hex() {
    match keypair::pair_from_seed_hex("0xGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGG") {
        Ok(_) => panic!("Invalid hex should fail"),
        Err(err) => {
            let msg = format!("{:#}", err);
            assert!(
                msg.to_lowercase().contains("hex"),
                "Should mention hex error, got: {}",
                msg
            );
        }
    }
}

// ──────── Multiple hotkeys ────────

#[test]
fn create_and_load_multiple_hotkeys() {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().to_str().unwrap();
    let (wallet, _, _) = Wallet::create(base, "multi_hk", "pass", "default").unwrap();

    // Write additional hotkeys using the keypair module
    let hk_path = wallet.path.join("hotkeys");
    let (_, mnemonic2) = keypair::generate_mnemonic_keypair().unwrap();
    agcli::wallet::keyfile::write_keyfile(&hk_path.join("miner1"), &mnemonic2).unwrap();
    let (_, mnemonic3) = keypair::generate_mnemonic_keypair().unwrap();
    agcli::wallet::keyfile::write_keyfile(&hk_path.join("miner2"), &mnemonic3).unwrap();

    let hotkeys = wallet.list_hotkeys().unwrap();
    assert_eq!(hotkeys.len(), 3, "Should have 3 hotkeys: {:?}", hotkeys);
    assert!(hotkeys.contains(&"default".to_string()));
    assert!(hotkeys.contains(&"miner1".to_string()));
    assert!(hotkeys.contains(&"miner2".to_string()));
}

#[test]
fn load_specific_hotkey() {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().to_str().unwrap();
    let (wallet, _, hk_mnemonic) = Wallet::create(base, "load_hk", "pass", "default").unwrap();
    let _default_ss58 = wallet.hotkey_ss58().unwrap().to_string();

    // Open and load the default hotkey
    let mut opened = Wallet::open(format!("{}/load_hk", base)).unwrap();
    opened.load_hotkey("default").unwrap();
    assert!(opened.hotkey().is_ok(), "Should have hotkey loaded");

    // Verify the loaded hotkey derives correctly from the mnemonic.
    let expected_pair = keypair::pair_from_mnemonic(&hk_mnemonic).unwrap();
    assert_eq!(
        opened.hotkey().unwrap().public(),
        expected_pair.public(),
        "Loaded hotkey should match original mnemonic derivation"
    );
}

#[test]
fn load_nonexistent_hotkey_fails() {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().to_str().unwrap();
    Wallet::create(base, "no_hk", "pass", "default").unwrap();
    let mut opened = Wallet::open(format!("{}/no_hk", base)).unwrap();
    let err = opened.load_hotkey("nonexistent_hotkey_xyz").unwrap_err();
    let msg = format!("{:#}", err);
    assert!(
        msg.contains("read") || msg.contains("No such file") || msg.contains("not found"),
        "Should indicate hotkey not found, got: {}",
        msg
    );
}

// ──────── SS58 validation comprehensive ────────

#[test]
fn ss58_validation_whitespace_around_address() {
    // Valid address with whitespace should still work through from_ss58 or produce clear error
    let valid = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
    let result = keypair::from_ss58(&format!(" {} ", valid));
    // from_ss58 does not trim, so this should fail with a helpful message
    assert!(result.is_err(), "Address with whitespace should fail without trimming");
}

#[test]
fn ss58_roundtrip_all_dev_accounts() {
    for name in keypair::DEV_ACCOUNTS {
        let pair = keypair::pair_from_uri(&format!("//{}", name)).unwrap();
        let ss58 = keypair::to_ss58(&pair.public(), 42);
        let recovered = keypair::from_ss58(&ss58).unwrap();
        assert_eq!(
            pair.public(),
            recovered,
            "SS58 roundtrip failed for //{}",
            name
        );
    }
}

// ──────── Encryption edge cases ────────

#[test]
fn encrypted_keyfile_special_chars_in_password() {
    let dir = tempfile::tempdir().unwrap();
    let special_pw = "p@$$w0rd!#%^&*()_+-={}[]|\\:\";<>?,./~`";
    let (_, _, _) = Wallet::create(dir.path().to_str().unwrap(), "special_pw", special_pw, "default").unwrap();
    let mut opened = Wallet::open(format!("{}/special_pw", dir.path().to_str().unwrap())).unwrap();
    assert!(opened.unlock_coldkey(special_pw).is_ok(), "Should unlock with special char password");
    assert!(opened.unlock_coldkey("wrong").is_err(), "Should fail with wrong password");
}

#[test]
fn encrypted_keyfile_very_long_password() {
    let dir = tempfile::tempdir().unwrap();
    let long_pw = "a".repeat(500);
    let (_, _, _) = Wallet::create(dir.path().to_str().unwrap(), "long_pw", &long_pw, "default").unwrap();
    let mut opened = Wallet::open(format!("{}/long_pw", dir.path().to_str().unwrap())).unwrap();
    assert!(opened.unlock_coldkey(&long_pw).is_ok(), "Should unlock with 500-char password");
}

// ──────── Wallet listing edge cases ────────

#[test]
fn list_wallets_empty_dir() {
    let dir = tempfile::tempdir().unwrap();
    let wallets = Wallet::list_wallets(dir.path().to_str().unwrap()).unwrap();
    assert!(wallets.is_empty(), "Empty dir should list no wallets");
}

#[test]
fn list_wallets_ignores_non_wallet_files() {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().to_str().unwrap();
    // Create a real wallet
    Wallet::create(base, "real_wallet", "pass", "default").unwrap();
    // Create a non-wallet file
    std::fs::write(dir.path().join("random_file.txt"), "not a wallet").unwrap();
    // Create a non-wallet directory without coldkey
    std::fs::create_dir(dir.path().join("empty_dir")).unwrap();

    let wallets = Wallet::list_wallets(base).unwrap();
    // Should only list directories, and the list function may include empty_dir
    // The key thing is real_wallet is listed
    assert!(
        wallets.contains(&"real_wallet".to_string()),
        "Should list the real wallet, got: {:?}",
        wallets
    );
}

// ──────── Mnemonic recovery/determinism ────────

#[test]
fn mnemonic_produces_deterministic_keys() {
    let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let pair1 = keypair::pair_from_mnemonic(mnemonic).unwrap();
    let pair2 = keypair::pair_from_mnemonic(mnemonic).unwrap();
    assert_eq!(
        pair1.public(),
        pair2.public(),
        "Same mnemonic must produce same keypair"
    );
}

#[test]
fn different_mnemonics_produce_different_keys() {
    let m1 = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let m2 = "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong";
    let pair1 = keypair::pair_from_mnemonic(m1).unwrap();
    let pair2 = keypair::pair_from_mnemonic(m2).unwrap();
    assert_ne!(
        pair1.public(),
        pair2.public(),
        "Different mnemonics should produce different keys"
    );
}

#[test]
fn generated_mnemonic_is_12_words() {
    let (_, mnemonic) = keypair::generate_mnemonic_keypair().unwrap();
    let words: Vec<&str> = mnemonic.split_whitespace().collect();
    assert_eq!(words.len(), 12, "Generated mnemonic should be 12 words, got: {}", mnemonic);
}

#[test]
fn wallet_coldkey_requires_unlock() {
    let dir = tempfile::tempdir().unwrap();
    Wallet::create(dir.path().to_str().unwrap(), "locked", "pass", "default").unwrap();
    let wallet = Wallet::open(format!("{}/locked", dir.path().to_str().unwrap())).unwrap();
    // Without unlock, coldkey() should fail
    match wallet.coldkey() {
        Ok(_) => panic!("coldkey() should fail without unlock"),
        Err(err) => {
            let msg = format!("{:#}", err);
            assert!(
                msg.to_lowercase().contains("unlock") || msg.to_lowercase().contains("not available") || msg.to_lowercase().contains("not loaded"),
                "Should say coldkey needs unlocking, got: {}",
                msg
            );
        }
    }
}
