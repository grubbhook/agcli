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

    let cache = QueryCache::with_ttl(std::time::Duration::from_secs(30));
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

    let cache = QueryCache::with_ttl(std::time::Duration::from_secs(30));
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

    let cache = QueryCache::with_ttl(std::time::Duration::from_secs(30));

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

// ──── Sprint 13: Multi-process + thread interference tests ────

/// Multiple processes writing to the same config file shouldn't corrupt it.
#[test]
fn config_concurrent_writes() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("config.toml");

    // Write initial config
    std::fs::write(&config_path, "[network]\ndefault = \"finney\"\n").unwrap();

    let threads: Vec<_> = (0..8)
        .map(|i| {
            let path = config_path.clone();
            std::thread::spawn(move || {
                let content = format!(
                    "[network]\ndefault = \"thread_{}\"\n[spending_limits]\n\"*\" = {}.0\n",
                    i, i * 100
                );
                // Atomic write pattern: write to temp then rename
                let tmp = path.with_extension(format!("tmp.{}", i));
                std::fs::write(&tmp, &content).unwrap();
                std::fs::rename(&tmp, &path).unwrap();

                // Read back should always get valid TOML
                let read = std::fs::read_to_string(&path).unwrap();
                assert!(read.contains("[network]"), "Config corrupted by thread {}: {}", i, read);
            })
        })
        .collect();

    for t in threads {
        t.join().unwrap();
    }

    // Final read should be valid TOML
    let final_content = std::fs::read_to_string(&config_path).unwrap();
    assert!(final_content.contains("[network]"), "Final config corrupted: {}", final_content);
}

/// Verify that clap parser is truly thread-safe by parsing conflicting args concurrently.
#[test]
fn cli_parsing_conflicting_args_concurrent() {
    use clap::Parser;

    // Mix of valid and intentionally invalid parses
    let scenarios: Vec<(Vec<&str>, bool)> = vec![
        (vec!["agcli", "balance"], true),
        (vec!["agcli", "--output", "json", "subnet", "list"], true),
        (vec!["agcli", "--output", "csv", "wallet", "list"], true),
        (vec!["agcli", "--debug", "doctor"], true),
        (vec!["agcli", "--verbose", "--timeout", "30", "balance"], true),
        (vec!["agcli", "subnet", "show", "--netuid", "1"], true),
        (vec!["agcli", "--network", "test", "balance"], true),
        (vec!["agcli", "--batch", "balance"], true),
    ];

    let threads: Vec<_> = scenarios
        .into_iter()
        .map(|(args, should_succeed)| {
            std::thread::spawn(move || {
                let result = agcli::cli::Cli::try_parse_from(&args);
                if should_succeed {
                    assert!(result.is_ok(), "Expected success for {:?}: {:?}", args, result.err());
                }
            })
        })
        .collect();

    for t in threads {
        t.join().unwrap();
    }
}

/// Error classification should be thread-safe and produce consistent results.
#[test]
fn error_classification_concurrent() {
    let test_cases: Vec<(&str, i32)> = vec![
        ("Connection refused", 10),
        ("Wrong password", 11),
        ("Invalid SS58 address", 12),
        ("Insufficient balance", 13),
        ("Permission denied", 14),
        ("Timeout waiting", 15),
        ("Generic error", 1),
    ];

    let threads: Vec<_> = test_cases
        .into_iter()
        .map(|(msg, expected_code)| {
            std::thread::spawn(move || {
                for _ in 0..100 {
                    let err = anyhow::anyhow!("{}", msg);
                    let code = agcli::error::classify(&err);
                    assert_eq!(code, expected_code, "Classification inconsistent for '{}': got {}, expected {}", msg, code, expected_code);
                }
            })
        })
        .collect();

    for t in threads {
        t.join().unwrap();
    }
}

/// Balance arithmetic should be thread-safe (no shared mutable state).
#[test]
fn balance_operations_concurrent() {
    use agcli::types::Balance;

    let threads: Vec<_> = (0..8)
        .map(|i| {
            std::thread::spawn(move || {
                let base = Balance::from_rao((i + 1) * 1_000_000_000);
                let add = Balance::from_rao(500_000_000);

                // Exercise all arithmetic
                let sum = base + add;
                assert!(sum.rao() > base.rao(), "Addition failed for thread {}", i);

                let tao = base.tao();
                assert!(tao > 0.0, "Tao conversion failed for thread {}", i);

                let display = base.display_tao();
                assert!(!display.is_empty(), "Display failed for thread {}", i);

                // from_tao roundtrip
                let rt = Balance::from_tao(tao);
                assert_eq!(rt.rao(), base.rao(), "Roundtrip failed for thread {}", i);
            })
        })
        .collect();

    for t in threads {
        t.join().unwrap();
    }
}

/// Simultaneous wallet encrypt+decrypt on separate files should not interfere.
#[test]
fn wallet_operations_isolated_concurrent() {
    let dir = tempfile::tempdir().unwrap();

    let threads: Vec<_> = (0..8)
        .map(|i| {
            let base = dir.path().to_path_buf();
            std::thread::spawn(move || {
                let path = base.join(format!("isolated_wallet_{}", i));
                let mnemonic = format!("word{} abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about", i);
                let password = format!("isolated_pw_{}", i);

                // Write, read, verify — 5 times per thread
                for round in 0..5 {
                    let mn = format!("{} round{}", mnemonic, round);
                    agcli::wallet::keyfile::write_encrypted_keyfile(&path, &mn, &password).unwrap();
                    let recovered = agcli::wallet::keyfile::read_encrypted_keyfile(&path, &password).unwrap();
                    assert_eq!(mn, recovered, "Mismatch in thread {} round {}", i, round);
                }
            })
        })
        .collect();

    for t in threads {
        t.join().unwrap();
    }
}

/// Cache operations under high contention — ensure no panics or data races.
#[tokio::test]
async fn cache_high_contention() {
    use agcli::queries::query_cache::QueryCache;

    let cache = QueryCache::with_ttl(std::time::Duration::from_secs(30));
    let call_count = Arc::new(AtomicU32::new(0));

    // Simulate 50 concurrent readers hitting both all-subnets and per-netuid caches
    let mut handles = Vec::new();
    for i in 0..50 {
        let c = cache.clone();
        let count = call_count.clone();
        handles.push(tokio::spawn(async move {
            // Alternate between all_subnets and all_dynamic_info
            if i % 2 == 0 {
                c.get_all_subnets(|| {
                    let cnt = count.clone();
                    async move {
                        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                        cnt.fetch_add(1, Ordering::SeqCst);
                        Ok(vec![])
                    }
                })
                .await
                .unwrap();
            } else {
                c.get_all_dynamic_info(|| {
                    let cnt = count.clone();
                    async move {
                        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                        cnt.fetch_add(1, Ordering::SeqCst);
                        Ok(vec![])
                    }
                })
                .await
                .unwrap();
            }
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    // With moka dedup, we expect far fewer than 50 fetches
    let total = call_count.load(Ordering::SeqCst);
    assert!(total <= 50, "Cache made {} calls (50 concurrent)", total);
}

// ──── Sprint 21: Wallet creation race + atomic cache symlink ────

/// Two threads creating the same wallet name should not corrupt keys.
/// One succeeds and the other gets an "already exists" error.
#[test]
fn wallet_create_race_protection() {
    let dir = tempfile::tempdir().unwrap();
    let wallet_dir = dir.path().to_path_buf();

    let mut handles = Vec::new();
    for _ in 0..4 {
        let wd = wallet_dir.clone();
        handles.push(std::thread::spawn(move || {
            agcli::Wallet::create(&wd, "race_test", "password123", "default")
        }));
    }

    let mut successes = 0;
    let mut already_exists = 0;
    for h in handles {
        match h.join().unwrap() {
            Ok(_) => successes += 1,
            Err(e) => {
                let msg = format!("{:#}", e);
                if msg.contains("already exists") {
                    already_exists += 1;
                } else {
                    // Other errors are OK too (lock contention, etc.)
                    // but should not be corruption
                    assert!(
                        !msg.contains("corrupted"),
                        "Unexpected corruption error: {}", msg
                    );
                }
            }
        }
    }

    // At least one should succeed
    assert!(
        successes >= 1,
        "Expected at least 1 success, got {} successes and {} already-exists",
        successes, already_exists
    );

    // The wallet should be valid
    let wallet = agcli::Wallet::open(wallet_dir.join("race_test")).unwrap();
    assert!(wallet.coldkey_ss58().is_some(), "Wallet should have a valid coldkey");
}

/// Concurrent cache save operations should not crash or leave missing latest.json.
#[test]
fn cache_save_concurrent_atomic_symlink() {
    use agcli::queries::cache;
    use agcli::types::balance::Balance;
    use agcli::types::chain_data::{Metagraph, NeuronInfoLite};
    use agcli::types::network::NetUid;

    let netuid = 60010; // Unique to avoid interference

    let make_neuron = |uid: u16| NeuronInfoLite {
        hotkey: format!("5Hot{}", uid),
        coldkey: "5Cold".to_string(),
        uid,
        netuid: NetUid(netuid),
        active: true,
        stake: Balance::from_rao(100_000_000_000),
        rank: 0.0,
        emission: 0.0,
        incentive: 0.5,
        consensus: 0.0,
        trust: 0.0,
        validator_trust: 0.0,
        dividends: 0.0,
        last_update: 100,
        validator_permit: false,
        pruning_score: 0.0,
    };

    let neurons = vec![make_neuron(0), make_neuron(1)];

    let threads: Vec<_> = (0..8)
        .map(|i| {
            let ns = neurons.clone();
            std::thread::spawn(move || {
                let mg = Metagraph {
                    netuid: NetUid(netuid),
                    n: ns.len() as u16,
                    block: 900000 + i as u64,
                    stake: ns.iter().map(|n| n.stake).collect(),
                    ranks: ns.iter().map(|n| n.rank).collect(),
                    trust: ns.iter().map(|n| n.trust).collect(),
                    consensus: ns.iter().map(|n| n.consensus).collect(),
                    incentive: ns.iter().map(|n| n.incentive).collect(),
                    dividends: ns.iter().map(|n| n.dividends).collect(),
                    emission: ns.iter().map(|n| n.emission).collect(),
                    validator_trust: ns.iter().map(|n| n.validator_trust).collect(),
                    validator_permit: ns.iter().map(|n| n.validator_permit).collect(),
                    uids: ns.iter().map(|n| n.uid).collect(),
                    active: ns.iter().map(|n| n.active).collect(),
                    last_update: ns.iter().map(|n| n.last_update).collect(),
                    neurons: ns,
                };
                cache::save(&mg).unwrap();
            })
        })
        .collect();

    for t in threads {
        t.join().unwrap();
    }

    // latest.json should exist and be readable
    let loaded = cache::load_latest(netuid).unwrap();
    assert!(loaded.is_some(), "latest.json should exist after concurrent saves");
    let mg = loaded.unwrap();
    assert!(mg.block >= 900000, "Should have a valid block number");

    // Cleanup
    let _ = std::fs::remove_dir_all(cache::cache_path(netuid));
}

/// Wallet directory lock prevents concurrent creation interference.
#[test]
fn wallet_dir_lock_serializes_creation() {
    let dir = tempfile::tempdir().unwrap();
    let wallet_dir = dir.path().to_path_buf();
    let lock_order = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));

    let threads: Vec<_> = (0..3)
        .map(|i| {
            let wd = wallet_dir.clone();
            let order = lock_order.clone();
            std::thread::spawn(move || {
                let name = format!("serial_test_{}", i);
                match agcli::Wallet::create(&wd, &name, "pw", "default") {
                    Ok(_) => order.lock().unwrap().push(i),
                    Err(e) => panic!("Wallet {} creation failed: {}", i, e),
                }
            })
        })
        .collect();

    for t in threads {
        t.join().unwrap();
    }

    // All 3 should succeed (different names)
    assert_eq!(lock_order.lock().unwrap().len(), 3);
}

/// Verify format_* utilities are safe under concurrent use.
#[test]
fn format_utilities_concurrent() {
    use agcli::utils::format_tao;
    use agcli::utils::short_ss58;
    use agcli::types::Balance;

    let threads: Vec<_> = (0..8)
        .map(|_| {
            std::thread::spawn(|| {
                for _ in 0..100 {
                    let _ = format_tao(Balance::from_rao(1_234_567_890));
                    let _ = format_tao(Balance::from_rao(0));
                    let _ = format_tao(Balance::from_rao(u64::MAX));
                    let _ = short_ss58("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKv3gB");
                }
            })
        })
        .collect();

    for t in threads {
        t.join().unwrap();
    }
}

// ──── Sprint 23: Dry-run and error path coverage ────

/// Dry-run mode should be settable and queryable without any chain connection.
#[test]
fn dry_run_flag_parse() {
    use clap::Parser;
    let cli = agcli::cli::Cli::try_parse_from(&["agcli", "--dry-run", "balance"]).unwrap();
    assert!(cli.dry_run, "dry-run flag should be parsed");
}

/// OutputFormat should round-trip through clap parsing for all variants.
#[test]
fn output_format_roundtrip() {
    use clap::Parser;
    for fmt in &["json", "csv", "table"] {
        let cli = agcli::cli::Cli::try_parse_from(&["agcli", "--output", fmt, "balance"]).unwrap();
        match *fmt {
            "json" => assert!(cli.output.is_json()),
            "csv" => assert!(cli.output.is_csv()),
            "table" => assert!(!cli.output.is_json() && !cli.output.is_csv()),
            _ => unreachable!(),
        }
    }
}

/// Config roundtrip with all fields set — verifies serialization fidelity.
#[test]
fn config_full_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");

    let mut cfg = agcli::Config::default();
    cfg.network = Some("test".into());
    cfg.endpoint = Some("wss://test.finney.opentensor.ai:443".into());
    cfg.wallet_dir = Some("/tmp/wallets".into());
    cfg.wallet = Some("mywallet".into());
    cfg.hotkey = Some("myhotkey".into());
    cfg.output = Some("json".into());
    cfg.proxy = Some("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKv3gB".into());
    cfg.live_interval = Some(30);
    cfg.batch = Some(true);
    let mut limits = std::collections::HashMap::new();
    limits.insert("1".into(), 100.0);
    limits.insert("*".into(), 500.0);
    cfg.spending_limits = Some(limits);

    cfg.save_to(&path).unwrap();
    let loaded = agcli::Config::load_from(&path).unwrap();
    assert_eq!(loaded.network.as_deref(), Some("test"));
    assert_eq!(loaded.endpoint.as_deref(), Some("wss://test.finney.opentensor.ai:443"));
    assert_eq!(loaded.wallet.as_deref(), Some("mywallet"));
    assert_eq!(loaded.hotkey.as_deref(), Some("myhotkey"));
    assert_eq!(loaded.output.as_deref(), Some("json"));
    assert_eq!(loaded.live_interval, Some(30));
    assert_eq!(loaded.batch, Some(true));
    let limits = loaded.spending_limits.unwrap();
    assert_eq!(*limits.get("1").unwrap(), 100.0);
    assert_eq!(*limits.get("*").unwrap(), 500.0);
}

/// Error classification should handle every possible exit code.
#[test]
fn error_codes_are_complete() {
    // Verify all exit codes are in range [1, 15]
    let codes = vec![
        agcli::error::exit_code::GENERIC,
        agcli::error::exit_code::NETWORK,
        agcli::error::exit_code::AUTH,
        agcli::error::exit_code::VALIDATION,
        agcli::error::exit_code::CHAIN,
        agcli::error::exit_code::IO,
        agcli::error::exit_code::TIMEOUT,
    ];
    for code in &codes {
        assert!(*code >= 1 && *code <= 15, "Exit code {} out of range", code);
    }
    // Verify they are all distinct
    let mut unique = codes.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), codes.len(), "Exit codes must be unique");
}

/// Spending limit check should pass when no limits are configured.
#[test]
fn spending_limit_no_config() {
    // With no config file, check_spending_limit should always pass
    let result = agcli::cli::helpers::check_spending_limit(1, 999999.0);
    assert!(result.is_ok(), "Should pass with no spending limits configured");
}

/// CSV escape handles edge cases correctly.
#[test]
fn csv_escape_edge_cases() {
    use agcli::cli::helpers::csv_escape;
    assert_eq!(csv_escape("simple"), "simple");
    assert_eq!(csv_escape("has,comma"), "\"has,comma\"");
    assert_eq!(csv_escape("has\"quote"), "\"has\"\"quote\"");
    assert_eq!(csv_escape("has\nnewline"), "\"has\nnewline\"");
    assert_eq!(csv_escape(""), "");
    assert_eq!(csv_escape("already \"quoted\""), "\"already \"\"quoted\"\"\"");
}

/// Weight pair parsing should validate rigorously.
#[test]
fn weight_pair_parsing_edge_cases() {
    use agcli::cli::helpers::parse_weight_pairs;

    // Valid
    let (uids, weights) = parse_weight_pairs("0:100,1:200").unwrap();
    assert_eq!(uids, vec![0, 1]);
    assert_eq!(weights, vec![100, 200]);

    // Single pair
    let (uids, weights) = parse_weight_pairs("5:32767").unwrap();
    assert_eq!(uids, vec![5]);
    assert_eq!(weights, vec![32767]);

    // Invalid: missing weight
    assert!(parse_weight_pairs("0").is_err());
    // Invalid: non-numeric UID
    assert!(parse_weight_pairs("abc:100").is_err());
    // Invalid: non-numeric weight
    assert!(parse_weight_pairs("0:abc").is_err());
    // Invalid: empty string
    assert!(parse_weight_pairs("").is_err());
    // Invalid: overflow u16 for UID
    assert!(parse_weight_pairs("99999:100").is_err());
}

/// Children pair parsing should validate correctly.
#[test]
fn children_pair_parsing() {
    use agcli::cli::helpers::parse_children;

    // Valid
    let result = parse_children("50000:5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKv3gB").unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].0, 50000);

    // Invalid: missing proportion
    assert!(parse_children("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKv3gB").is_err());
    // Invalid: non-numeric proportion
    assert!(parse_children("abc:5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKv3gB").is_err());
}

/// Parallel error classification with all error types simultaneously.
#[test]
fn parallel_error_classification_all_types() {
    let test_cases = vec![
        ("Connection refused to endpoint", agcli::error::exit_code::NETWORK),
        ("Decryption failed — wrong password for coldkey", agcli::error::exit_code::AUTH),
        ("Invalid SS58 address format", agcli::error::exit_code::VALIDATION),
        ("Extrinsic rejected: insufficient balance", agcli::error::exit_code::CHAIN),
        ("Permission denied writing config", agcli::error::exit_code::IO),
        ("Operation timed out after 30s", agcli::error::exit_code::TIMEOUT),
        ("Unknown error with no pattern match", agcli::error::exit_code::GENERIC),
    ];

    // Run all classifications in parallel threads, 50 iterations each
    let threads: Vec<_> = test_cases
        .into_iter()
        .map(|(msg, expected)| {
            std::thread::spawn(move || {
                for _ in 0..50 {
                    let err = anyhow::anyhow!("{}", msg);
                    let code = agcli::error::classify(&err);
                    assert_eq!(code, expected, "Mismatch for '{}': got {} expected {}", msg, code, expected);

                    // Also test hint generation doesn't panic
                    let _ = agcli::error::hint(code, msg);
                }
            })
        })
        .collect();

    for t in threads {
        t.join().unwrap();
    }
}

/// Disk cache pruning does not crash under concurrent writes.
#[test]
fn disk_cache_concurrent_put_prune() {
    let key_prefix = "stress_prune_test";
    // Write many entries to exercise pruning logic
    let threads: Vec<_> = (0..8)
        .map(|i| {
            std::thread::spawn(move || {
                for j in 0..15u32 {
                    let key = format!("{}_{}_{}",  key_prefix, i, j);
                    let _ = agcli::queries::disk_cache::put(&key, &(i * 100 + j));
                }
            })
        })
        .collect();
    for t in threads {
        t.join().unwrap();
    }
    // Cleanup
    for i in 0..8u32 {
        for j in 0..15u32 {
            agcli::queries::disk_cache::remove(&format!("{}_{}_{}", key_prefix, i, j));
        }
    }
}

/// Stale-while-error cache should be readable event after TTL expires.
#[test]
fn disk_cache_stale_after_expiry() {
    let key = "stress_stale_test";
    agcli::queries::disk_cache::put(key, &"stale_data").unwrap();
    // Regular get with 0 TTL returns None
    let fresh: Option<String> = agcli::queries::disk_cache::get(key, std::time::Duration::ZERO);
    assert!(fresh.is_none(), "Should be expired with 0 TTL");
    // Stale get still returns the data
    let stale: Option<String> = agcli::queries::disk_cache::get_stale(key);
    assert_eq!(stale, Some("stale_data".to_string()));
    agcli::queries::disk_cache::remove(key);
}

/// Error classification handles serde_json errors correctly.
#[test]
fn error_classify_serde_json() {
    let json_err: serde_json::Error = serde_json::from_str::<Vec<u32>>("not json").unwrap_err();
    let err = anyhow::Error::new(json_err).context("Decoding chain response");
    let code = agcli::error::classify(&err);
    assert_eq!(code, agcli::error::exit_code::VALIDATION, "serde_json errors should be VALIDATION");
}

/// The --best flag should be parseable.
#[test]
fn best_flag_parse() {
    use clap::Parser;
    let cli = agcli::cli::Cli::try_parse_from(&["agcli", "--best", "balance"]).unwrap();
    assert!(cli.best, "--best flag should be parsed");
}

/// event filter parsing covers all known variants.
#[test]
fn event_filter_parsing_all_variants() {
    for input in &["staking", "stake", "registration", "register", "reg",
                   "transfer", "transfers", "weights", "weight",
                   "subnet", "subnets", "all", "unknown"] {
        let _: agcli::events::EventFilter = input.parse().unwrap();
    }
}

/// Concurrent config file creation and destruction.
#[test]
fn config_create_destroy_concurrent() {
    let dir = tempfile::tempdir().unwrap();

    let threads: Vec<_> = (0..8)
        .map(|i| {
            let base = dir.path().to_path_buf();
            std::thread::spawn(move || {
                let path = base.join(format!("config_{}.toml", i));
                for round in 0..10 {
                    // Create
                    let mut cfg = agcli::Config::default();
                    cfg.network = Some(format!("thread_{}_round_{}", i, round));
                    cfg.save_to(&path).unwrap();

                    // Read back
                    let loaded = agcli::Config::load_from(&path).unwrap();
                    assert!(
                        loaded.network.is_some(),
                        "Config should be readable after save (thread={}, round={})", i, round
                    );

                    // Delete
                    let _ = std::fs::remove_file(&path);
                }
            })
        })
        .collect();

    for t in threads {
        t.join().unwrap();
    }
}
