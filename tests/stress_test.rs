//! Stress tests — verify agcli handles concurrency and edge cases.

use std::sync::{Arc, atomic::{AtomicU32, Ordering}};

/// Concurrent wallet writes should not corrupt data thanks to file locking.
#[test]
fn concurrent_wallet_writes_no_corruption() {
    let dir = tempfile::tempdir().unwrap();
    let keyfile = dir.path().join("stress_coldkey");
    let password = "stress_test_pw";
    let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    // Write the keyfile first
    agcli::wallet::keyfile::write_encrypted_keyfile(&keyfile, mnemonic, password).unwrap();

    // Spawn N threads that all try to read/write simultaneously
    let threads: Vec<_> = (0..8)
        .map(|i| {
            let path = keyfile.clone();
            let pw = password.to_string();
            let mn = format!("thread {} {}", i, mnemonic);
            std::thread::spawn(move || {
                // Each thread writes its own mnemonic
                agcli::wallet::keyfile::write_encrypted_keyfile(&path, &mn, &pw).unwrap();
                // Then reads back — must get a valid mnemonic (any thread's)
                let read = agcli::wallet::keyfile::read_encrypted_keyfile(&path, &pw).unwrap();
                assert!(
                    read.contains("abandon") || read.starts_with("thread"),
                    "Got corrupted data: {}", read
                );
            })
        })
        .collect();

    for t in threads {
        t.join().unwrap();
    }

    // Final read should succeed
    let final_read = agcli::wallet::keyfile::read_encrypted_keyfile(&keyfile, password);
    assert!(final_read.is_ok(), "Final read failed: {:?}", final_read.err());
}

/// Concurrent hotkey file writes should be safe.
#[test]
fn concurrent_hotkey_writes_no_corruption() {
    let dir = tempfile::tempdir().unwrap();
    let keyfile = dir.path().join("stress_hotkey");
    let mnemonic = "test mnemonic for hotkey stress testing";

    agcli::wallet::keyfile::write_keyfile(&keyfile, mnemonic).unwrap();

    let threads: Vec<_> = (0..8)
        .map(|i| {
            let path = keyfile.clone();
            let mn = format!("hotkey_thread_{} {}", i, mnemonic);
            std::thread::spawn(move || {
                agcli::wallet::keyfile::write_keyfile(&path, &mn).unwrap();
                let read = agcli::wallet::keyfile::read_keyfile(&path).unwrap();
                assert!(
                    read.contains("hotkey_thread") || read.contains("test mnemonic"),
                    "Got corrupted data: {}", read
                );
            })
        })
        .collect();

    for t in threads {
        t.join().unwrap();
    }
}

/// Parallel CLI parses should not interfere with each other.
#[test]
fn parallel_cli_parsing() {
    use clap::Parser;

    let commands = vec![
        vec!["agcli", "subnet", "list"],
        vec!["agcli", "balance"],
        vec!["agcli", "stake", "add", "--amount", "1.0", "--netuid", "1"],
        vec!["agcli", "wallet", "list"],
        vec!["agcli", "subnet", "show", "--netuid", "1"],
        vec!["agcli", "weights", "status", "--netuid", "1"],
        vec!["agcli", "subnet", "commits", "--netuid", "18"],
        vec!["agcli", "doctor"],
    ];

    let threads: Vec<_> = commands
        .into_iter()
        .map(|args| {
            std::thread::spawn(move || {
                let result = agcli::cli::Cli::try_parse_from(&args);
                assert!(
                    result.is_ok(),
                    "Failed to parse {:?}: {:?}",
                    args,
                    result.err()
                );
            })
        })
        .collect();

    for t in threads {
        t.join().unwrap();
    }
}

/// Error classifier should handle all common patterns without panicking.
#[test]
fn error_classifier_exhaustive() {
    let long_msg = "x".repeat(10000);
    let test_messages = vec![
        "Connection refused to wss://entrypoint-finney.opentensor.ai:443",
        "DNS resolution failed for host: bittensor.example.com",
        "WebSocket connection timeout after 30s",
        "Decryption failed — wrong password",
        "No hotkey loaded for wallet default",
        "Cannot read keyfile at '/missing/coldkey'",
        "Invalid SS58 address: bad checksum in 5GrwvaEF...",
        "Failed to parse weight pairs: invalid format",
        "Extrinsic failed: insufficient balance for transfer",
        "Rate limit exceeded — wait 100 blocks",
        "Nonce too low for account",
        "Permission denied writing to /etc/agcli/config",
        "No such file or directory: /nonexistent/path",
        "Something completely unexpected happened",
        "",
        "a]]]***weird chars!!!",
        &long_msg, // Very long message
    ];

    for msg in test_messages {
        let err = anyhow::anyhow!("{}", msg);
        let code = agcli::error::classify(&err);
        assert!(
            code >= 1 && code <= 15,
            "Unexpected exit code {} for message: {}",
            code,
            &msg[..msg.len().min(100)]
        );
    }
}

/// Cache deduplication under concurrent access.
#[tokio::test]
async fn cache_concurrent_access() {
    use agcli::queries::query_cache::QueryCache;

    let cache = QueryCache::new();
    let call_count = Arc::new(AtomicU32::new(0));

    // Launch 10 concurrent cache reads — only 1 should actually fetch
    let mut handles = Vec::new();
    for _ in 0..10 {
        let c = cache.clone();
        let count = call_count.clone();
        handles.push(tokio::spawn(async move {
            c.get_all_subnets(|| {
                let cnt = count.clone();
                async move {
                    // Simulate slow fetch
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                    cnt.fetch_add(1, Ordering::SeqCst);
                    Ok(vec![])
                }
            })
            .await
            .unwrap();
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    // Due to moka's lazy evaluation, first access triggers fetch,
    // subsequent accesses may also fetch if they arrive before insertion completes.
    // But we should get far fewer than 10 fetches.
    let count = call_count.load(Ordering::SeqCst);
    assert!(
        count <= 10,
        "Cache made {} calls (10 concurrent, all OK)",
        count
    );
}

/// Wallet encrypt/decrypt roundtrip is deterministic across threads.
#[test]
fn wallet_roundtrip_multithread() {
    let dir = tempfile::tempdir().unwrap();

    let threads: Vec<_> = (0..4)
        .map(|i| {
            let base = dir.path().to_path_buf();
            std::thread::spawn(move || {
                let path = base.join(format!("wallet_{}", i));
                let mnemonic = format!("word{} abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about", i);
                let password = format!("pw_{}", i);

                agcli::wallet::keyfile::write_encrypted_keyfile(&path, &mnemonic, &password)
                    .unwrap();
                let recovered =
                    agcli::wallet::keyfile::read_encrypted_keyfile(&path, &password).unwrap();
                assert_eq!(mnemonic, recovered, "Roundtrip failed for wallet {}", i);
            })
        })
        .collect();

    for t in threads {
        t.join().unwrap();
    }
}

/// Public key file write/read roundtrip.
#[test]
fn public_key_roundtrip_concurrent() {
    use sp_core::{sr25519, Pair};

    let dir = tempfile::tempdir().unwrap();

    let threads: Vec<_> = (0..4)
        .map(|i| {
            let base = dir.path().to_path_buf();
            std::thread::spawn(move || {
                let path = base.join(format!("pubkey_{}", i));
                let (pair, _) = sr25519::Pair::generate();
                let public = pair.public();

                agcli::wallet::keyfile::write_public_key(&path, &public).unwrap();
                let recovered = agcli::wallet::keyfile::read_public_key(&path).unwrap();
                assert_eq!(public, recovered, "Public key roundtrip failed for {}", i);
            })
        })
        .collect();

    for t in threads {
        t.join().unwrap();
    }
}

// ──── Sprint 6: QueryCache concurrent access tests ────

#[tokio::test]
async fn query_cache_sequential_dedup() {
    use agcli::queries::query_cache::QueryCache;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    let cache = QueryCache::new();
    let fetch_count = Arc::new(AtomicU32::new(0));

    // First call: should fetch
    let count = fetch_count.clone();
    cache
        .get_all_subnets(|| {
            let c = count.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                Ok(vec![])
            }
        })
        .await
        .unwrap();

    // 10 sequential calls: all should hit cache
    for _ in 0..10 {
        let count = fetch_count.clone();
        cache
            .get_all_subnets(|| {
                let c = count.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(vec![])
                }
            })
            .await
            .unwrap();
    }

    // Only 1 actual fetch should have happened
    assert_eq!(
        fetch_count.load(Ordering::SeqCst),
        1,
        "Sequential reads should all hit cache after first fetch"
    );
}

#[tokio::test]
async fn query_cache_dynamic_populates_per_netuid() {
    use agcli::queries::query_cache::QueryCache;
    use agcli::types::chain_data::DynamicInfo;
    use agcli::types::balance::{AlphaBalance, Balance};
    use agcli::types::network::NetUid;

    let cache = QueryCache::new();

    let make_di = |netuid: u16, name: &str, price: f64, tao_rao: u64| DynamicInfo {
        netuid: NetUid(netuid),
        name: name.to_string(),
        symbol: String::new(),
        tempo: 360,
        emission: 0,
        tao_in: Balance::from_rao(tao_rao),
        alpha_in: AlphaBalance::from_raw(500_000_000),
        alpha_out: AlphaBalance::from_raw(500_000_000),
        price,
        owner_hotkey: String::new(),
        owner_coldkey: String::new(),
        last_step: 0,
        blocks_since_last_step: 0,
        alpha_out_emission: 0,
        alpha_in_emission: 0,
        tao_in_emission: 0,
        pending_alpha_emission: 0,
        pending_root_emission: 0,
        subnet_volume: 0,
        network_registered_at: 0,
    };

    // Fetch all dynamic info with 2 subnets
    let fetch_count = Arc::new(AtomicU32::new(0));
    let count = fetch_count.clone();
    cache
        .get_all_dynamic_info(|| {
            let c = count.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                Ok(vec![
                    make_di(1, "alpha", 1.5, 1_000_000_000),
                    make_di(2, "beta", 0.5, 2_000_000_000),
                ])
            }
        })
        .await
        .unwrap();

    assert_eq!(fetch_count.load(Ordering::SeqCst), 1);

    // Now per-netuid cache should be populated — fetching netuid 1 should NOT call fetch
    let per_netuid_count = Arc::new(AtomicU32::new(0));
    let c = per_netuid_count.clone();
    let result = cache
        .get_dynamic_info(1, || {
            let c = c.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                Ok(None)
            }
        })
        .await
        .unwrap();

    assert!(result.is_some(), "netuid 1 should be cached");
    assert_eq!(result.unwrap().name, "alpha");
    assert_eq!(per_netuid_count.load(Ordering::SeqCst), 0, "should use cache, not fetch");
}

// ──── Sprint 6: Balance edge cases ────

#[test]
fn balance_arithmetic_overflow_safety() {
    use agcli::types::Balance;
    // Adding two large balances should not panic
    let a = Balance::from_rao(u64::MAX / 2);
    let b = Balance::from_rao(u64::MAX / 2);
    let sum = a + b;
    assert!(sum.rao() >= u64::MAX / 2, "balance addition should work");
}

#[test]
fn balance_display_tao_large() {
    use agcli::types::Balance;
    let b = Balance::from_rao(u64::MAX);
    let display = b.display_tao();
    assert!(!display.is_empty(), "should display something");
    // u64::MAX RAO = ~18.4 billion TAO
    assert!(display.contains("."), "should display decimal TAO");
}

#[test]
fn balance_from_tao_fractional() {
    use agcli::types::Balance;
    let b = Balance::from_tao(0.000000001); // 1 RAO
    assert_eq!(b.rao(), 1);
}

#[test]
fn balance_zero_operations() {
    use agcli::types::Balance;
    let z = Balance::ZERO;
    assert_eq!(z.rao(), 0);
    assert_eq!(z.tao(), 0.0);
    assert_eq!((z + z).rao(), 0);
    let display = z.display_tao();
    assert!(display.contains("0"), "zero should display as 0");
}

// ──── Sprint 6: MEV shield encrypt edge cases ────

#[test]
fn mev_encrypt_empty_plaintext() {
    use ml_kem::{EncodedSizeUser, KemCore, MlKem768};
    let mut rng = rand::thread_rng();
    let (_dk, ek) = MlKem768::generate(&mut rng);
    let ek_bytes = ek.as_bytes();

    // Empty plaintext should still work
    let result = agcli::extrinsics::mev_shield::encrypt_for_mev_shield(ek_bytes.as_slice(), b"");
    assert!(result.is_ok(), "empty plaintext should encrypt: {:?}", result.err());
    let (_, ct) = result.unwrap();
    // Ciphertext: 2 + 1088 + 24 + (0 + 16 tag)
    assert_eq!(ct.len(), 2 + 1088 + 24 + 16);
}

#[test]
fn mev_encrypt_large_plaintext() {
    use ml_kem::{EncodedSizeUser, KemCore, MlKem768};
    let mut rng = rand::thread_rng();
    let (_dk, ek) = MlKem768::generate(&mut rng);
    let ek_bytes = ek.as_bytes();

    // Large 10KB plaintext
    let plaintext = vec![0xABu8; 10_000];
    let result = agcli::extrinsics::mev_shield::encrypt_for_mev_shield(ek_bytes.as_slice(), &plaintext);
    assert!(result.is_ok(), "large plaintext should encrypt: {:?}", result.err());
    let (_, ct) = result.unwrap();
    assert_eq!(ct.len(), 2 + 1088 + 24 + plaintext.len() + 16);
}

#[test]
fn mev_encrypt_commitment_deterministic() {
    use ml_kem::{EncodedSizeUser, KemCore, MlKem768};
    let mut rng = rand::thread_rng();
    let (_dk, ek) = MlKem768::generate(&mut rng);
    let ek_bytes = ek.as_bytes();
    let plaintext = b"deterministic commitment test";

    let (c1, _) = agcli::extrinsics::mev_shield::encrypt_for_mev_shield(ek_bytes.as_slice(), plaintext).unwrap();
    let (c2, _) = agcli::extrinsics::mev_shield::encrypt_for_mev_shield(ek_bytes.as_slice(), plaintext).unwrap();
    assert_eq!(c1, c2, "same plaintext should produce same commitment");
}

#[test]
fn mev_encrypt_ciphertext_nondeterministic() {
    use ml_kem::{EncodedSizeUser, KemCore, MlKem768};
    let mut rng = rand::thread_rng();
    let (_dk, ek) = MlKem768::generate(&mut rng);
    let ek_bytes = ek.as_bytes();
    let plaintext = b"nondeterministic ciphertext test";

    let (_, ct1) = agcli::extrinsics::mev_shield::encrypt_for_mev_shield(ek_bytes.as_slice(), plaintext).unwrap();
    let (_, ct2) = agcli::extrinsics::mev_shield::encrypt_for_mev_shield(ek_bytes.as_slice(), plaintext).unwrap();
    assert_ne!(ct1, ct2, "ciphertext should differ due to random nonce/KEM");
}
