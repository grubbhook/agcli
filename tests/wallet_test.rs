//! Wallet creation, import and key operations tests.
//! Run with: cargo test --test wallet_test

use agcli::Wallet;

#[test]
fn create_wallet_and_read_keys() {
    let dir = tempfile::tempdir().unwrap();
    let wallet = Wallet::create(dir.path().to_str().unwrap(), "test_wallet", "password123", "default").unwrap();
    assert!(wallet.coldkey_ss58().is_some());
    assert!(wallet.hotkey_ss58().is_some());
    // Address should be valid SS58
    let addr = wallet.coldkey_ss58().unwrap();
    assert!(addr.starts_with("5"), "should be a substrate SS58 address: {}", addr);
    assert!(addr.len() > 40, "SS58 address should be ~48 chars: {}", addr);
}

#[test]
fn open_wallet_and_read_public_key() {
    let dir = tempfile::tempdir().unwrap();
    let wallet = Wallet::create(dir.path().to_str().unwrap(), "w1", "pass", "default").unwrap();
    let addr = wallet.coldkey_ss58().unwrap().to_string();

    // Open and verify the SS58 is the same
    let opened = Wallet::open(&format!("{}/w1", dir.path().to_str().unwrap())).unwrap();
    assert_eq!(opened.coldkey_ss58().unwrap(), addr);
}

#[test]
fn unlock_coldkey_correct_password() {
    let dir = tempfile::tempdir().unwrap();
    Wallet::create(dir.path().to_str().unwrap(), "w2", "secret", "default").unwrap();
    let mut opened = Wallet::open(&format!("{}/w2", dir.path().to_str().unwrap())).unwrap();
    assert!(opened.unlock_coldkey("secret").is_ok());
    assert!(opened.coldkey().is_ok());
}

#[test]
fn unlock_coldkey_wrong_password() {
    let dir = tempfile::tempdir().unwrap();
    Wallet::create(dir.path().to_str().unwrap(), "w3", "correct", "default").unwrap();
    let mut opened = Wallet::open(&format!("{}/w3", dir.path().to_str().unwrap())).unwrap();
    assert!(opened.unlock_coldkey("wrong").is_err());
}

#[test]
fn import_from_mnemonic_and_verify() {
    let dir = tempfile::tempdir().unwrap();
    let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let wallet = Wallet::import_from_mnemonic(dir.path().to_str().unwrap(), "imported", mnemonic, "pass").unwrap();
    let addr = wallet.coldkey_ss58().unwrap().to_string();

    // Reimporting the same mnemonic should produce the same address
    let dir2 = tempfile::tempdir().unwrap();
    let wallet2 = Wallet::import_from_mnemonic(dir2.path().to_str().unwrap(), "imported2", mnemonic, "other_pass").unwrap();
    assert_eq!(wallet2.coldkey_ss58().unwrap(), addr);
}

#[test]
fn list_wallets() {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().to_str().unwrap();
    Wallet::create(base, "alice", "pass", "default").unwrap();
    Wallet::create(base, "bob", "pass", "default").unwrap();
    let wallets = Wallet::list_wallets(base).unwrap();
    assert!(wallets.contains(&"alice".to_string()));
    assert!(wallets.contains(&"bob".to_string()));
    assert_eq!(wallets.len(), 2);
}

#[test]
fn list_hotkeys() {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().to_str().unwrap();
    let wallet = Wallet::create(base, "hk_test", "pass", "default").unwrap();
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
            assert!(w.coldkey_ss58().is_none(), "nonexistent wallet should have no coldkey");
        }
    }
}

#[test]
fn wrong_password_error_message_is_helpful() {
    let dir = tempfile::tempdir().unwrap();
    Wallet::create(dir.path().to_str().unwrap(), "err_test", "correct", "default").unwrap();
    let mut wallet = Wallet::open(&format!("{}/err_test", dir.path().to_str().unwrap())).unwrap();
    let err = wallet.unlock_coldkey("wrong").unwrap_err();
    // The error chain includes "Failed to decrypt coldkey" context and inner "wrong password" cause
    let full = format!("{:#}", err);
    assert!(full.contains("decrypt") || full.contains("wrong password"),
        "Error chain should mention decryption failure, got: {}", full);
}

#[test]
fn ss58_validation_errors_are_helpful() {
    use agcli::wallet::keypair::from_ss58;
    // Empty address
    let err = from_ss58("").unwrap_err();
    assert!(err.to_string().contains("Empty address"), "Expected empty address hint, got: {}", err);

    // Too short
    let err = from_ss58("5abc").unwrap_err();
    assert!(err.to_string().contains("too short"), "Expected short address hint, got: {}", err);

    // Invalid characters
    let err = from_ss58("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQYxxxxxxinvalid").unwrap_err();
    assert!(err.to_string().contains("Invalid SS58"), "Expected invalid SS58 error, got: {}", err);
}
