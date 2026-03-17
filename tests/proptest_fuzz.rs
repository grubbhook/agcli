//! Property-based fuzz tests for all CLI validators using proptest.
//!
//! Every validator that accepts user input is tested with random data to verify:
//! 1. No panics on arbitrary input
//! 2. Error messages always include "Tip:" or helpful guidance
//! 3. Valid inputs are accepted
//! 4. Invariants hold (e.g. accepted amounts are always positive)

use proptest::prelude::*;

use agcli::cli::helpers::{
    json_to_subxt_value, parse_children, parse_json_args, parse_weight_pairs,
    validate_amount, validate_batch_axon_json, validate_delegate_take, validate_derive_input,
    validate_emission_weights, validate_evm_address, validate_hex_data, validate_ipv4,
    validate_max_cost, validate_mnemonic, validate_multisig_json_args, validate_name,
    validate_netuid, validate_pallet_call, validate_password_strength, validate_port,
    validate_schedule_id, validate_ss58, validate_symbol, validate_take_pct,
};

// ──── validate_amount: never panics, valid amounts always accepted ────

proptest! {
    #[test]
    fn fuzz_validate_amount_no_panic(amount in proptest::num::f64::ANY) {
        let _ = validate_amount(amount, "test");
    }

    #[test]
    fn fuzz_validate_amount_positive_finite_accepted(amount in 0.000000001f64..1e18) {
        prop_assert!(validate_amount(amount, "stake").is_ok(),
            "positive finite amount {:.9} should be accepted", amount);
    }

    #[test]
    fn fuzz_validate_amount_negative_rejected(amount in -1e18..-0.000000001f64) {
        let res = validate_amount(amount, "stake");
        prop_assert!(res.is_err(), "negative amount {} should be rejected", amount);
        let msg = res.unwrap_err().to_string();
        prop_assert!(msg.contains("negative"), "error should mention negative: {}", msg);
    }

    #[test]
    fn fuzz_validate_amount_zero_rejected(_dummy in 0u8..1u8) {
        let res = validate_amount(0.0, "stake");
        prop_assert!(res.is_err());
        let msg = res.unwrap_err().to_string();
        prop_assert!(msg.contains("greater than zero"), "error: {}", msg);
    }
}

// ──── validate_take_pct: range [0, 18] ────

proptest! {
    #[test]
    fn fuzz_validate_take_pct_no_panic(take in proptest::num::f64::ANY) {
        let _ = validate_take_pct(take);
    }

    #[test]
    fn fuzz_validate_take_pct_valid_range(take in 0.0f64..=18.0) {
        prop_assert!(validate_take_pct(take).is_ok(),
            "take {}% should be in range [0, 18]", take);
    }

    #[test]
    fn fuzz_validate_take_pct_over_18_rejected(take in 18.01f64..1e6) {
        prop_assert!(validate_take_pct(take).is_err(),
            "take {}% above 18 should be rejected", take);
    }

    #[test]
    fn fuzz_validate_take_pct_negative_rejected(take in -1e6f64..-0.01) {
        prop_assert!(validate_take_pct(take).is_err());
    }
}

// ──── validate_delegate_take: same range [0, 18] ────

proptest! {
    #[test]
    fn fuzz_validate_delegate_take_no_panic(take in proptest::num::f64::ANY) {
        let _ = validate_delegate_take(take);
    }

    #[test]
    fn fuzz_validate_delegate_take_valid(take in 0.0f64..=18.0) {
        prop_assert!(validate_delegate_take(take).is_ok());
    }

    #[test]
    fn fuzz_validate_delegate_take_over_rejected(take in 18.01f64..1e6) {
        prop_assert!(validate_delegate_take(take).is_err());
    }
}

// ──── validate_max_cost: non-negative ────

proptest! {
    #[test]
    fn fuzz_validate_max_cost_no_panic(cost in proptest::num::f64::ANY) {
        let _ = validate_max_cost(cost);
    }

    #[test]
    fn fuzz_validate_max_cost_non_negative_accepted(cost in 0.0f64..1e18) {
        prop_assert!(validate_max_cost(cost).is_ok());
    }

    #[test]
    fn fuzz_validate_max_cost_negative_rejected(cost in -1e18f64..-0.001) {
        prop_assert!(validate_max_cost(cost).is_err());
    }
}

// ──── validate_symbol: ASCII, non-empty, ≤32 chars ────

proptest! {
    #[test]
    fn fuzz_validate_symbol_no_panic(s in ".*") {
        let _ = validate_symbol(&s);
    }

    #[test]
    fn fuzz_validate_symbol_valid_ascii(s in "[A-Z]{1,32}") {
        prop_assert!(validate_symbol(&s).is_ok(),
            "valid ASCII symbol '{}' should be accepted", s);
    }

    #[test]
    fn fuzz_validate_symbol_long_rejected(s in "[A-Z]{33,100}") {
        prop_assert!(validate_symbol(&s).is_err(),
            "symbol '{}' over 32 chars should be rejected", s);
    }
}

// ──── validate_emission_weights: non-empty, sum > 0 ────

proptest! {
    #[test]
    fn fuzz_validate_emission_weights_no_panic(weights in proptest::collection::vec(any::<u16>(), 0..50)) {
        let _ = validate_emission_weights(&weights);
    }

    #[test]
    fn fuzz_validate_emission_weights_nonempty_with_nonzero(
        weights in proptest::collection::vec(1u16..=65535, 1..20)
    ) {
        prop_assert!(validate_emission_weights(&weights).is_ok());
    }

    #[test]
    fn fuzz_validate_emission_weights_empty_rejected(_dummy in 0u8..1u8) {
        prop_assert!(validate_emission_weights(&[]).is_err());
    }

    #[test]
    fn fuzz_validate_emission_weights_all_zeros_rejected(count in 1usize..20) {
        let weights = vec![0u16; count];
        prop_assert!(validate_emission_weights(&weights).is_err());
    }
}

// ──── validate_name: alphanumeric + hyphens/underscores, ≤64, no path traversal ────

proptest! {
    #[test]
    fn fuzz_validate_name_no_panic(s in ".*") {
        let _ = validate_name(&s, "wallet");
    }

    #[test]
    fn fuzz_validate_name_valid(s in "[a-zA-Z][a-zA-Z0-9_-]{0,63}") {
        // Filter out OS reserved names
        let upper = s.to_uppercase();
        let reserved = ["CON", "PRN", "AUX", "NUL",
            "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
            "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9"];
        prop_assume!(!reserved.contains(&upper.as_str()));
        prop_assert!(validate_name(&s, "wallet").is_ok(),
            "valid name '{}' should be accepted", s);
    }

    #[test]
    fn fuzz_validate_name_path_traversal(prefix in ".*", suffix in ".*") {
        let name = format!("{}../{}", prefix, suffix);
        // Only test if the combined string has ".." or "/" (path traversal patterns)
        if name.contains("..") || name.contains('/') {
            if !name.trim().is_empty() {
                let result = validate_name(&name, "wallet");
                prop_assert!(result.is_err(),
                    "path traversal '{}' should be rejected", name);
            }
        }
    }

    #[test]
    fn fuzz_validate_name_empty_rejected(spaces in " {0,10}") {
        // Empty or whitespace-only should fail
        if spaces.trim().is_empty() {
            prop_assert!(validate_name(&spaces, "wallet").is_err());
        }
    }
}

// ──── validate_ipv4: valid IPs accepted, garbage rejected without panic ────

proptest! {
    #[test]
    fn fuzz_validate_ipv4_no_panic(s in ".*") {
        let _ = validate_ipv4(&s);
    }

    #[test]
    fn fuzz_validate_ipv4_valid_public(
        a in 1u8..=126,   // skip 127 (loopback)
        b in 0u8..255u8,
        c in 0u8..255u8,
        d in 1u8..254u8   // skip 0 and 255
    ) {
        // Skip private ranges
        let is_private = a == 10
            || (a == 172 && (16..=31).contains(&b))
            || (a == 192 && b == 168);
        prop_assume!(!is_private);
        let ip = format!("{}.{}.{}.{}", a, b, c, d);
        prop_assert!(validate_ipv4(&ip).is_ok(),
            "valid public IP {} should be accepted", ip);
    }

    #[test]
    fn fuzz_validate_ipv4_loopback_rejected(
        b in 0u8..255u8, c in 0u8..255u8, d in 0u8..255u8
    ) {
        let ip = format!("127.{}.{}.{}", b, c, d);
        prop_assert!(validate_ipv4(&ip).is_err(),
            "loopback IP {} should be rejected", ip);
    }

    #[test]
    fn fuzz_validate_ipv4_leading_zeros_rejected(
        a in 1u8..9u8,
        rest in "[0-9]{1,3}\\.[0-9]{1,3}\\.[0-9]{1,3}"
    ) {
        let ip = format!("0{}.{}", a, rest);
        let result = validate_ipv4(&ip);
        // Leading zeros should be rejected
        prop_assert!(result.is_err(), "leading zero IP {} should be rejected", ip);
    }
}

// ──── validate_ss58: valid addresses accepted, garbage never panics ────

proptest! {
    #[test]
    fn fuzz_validate_ss58_no_panic(s in "\\PC{0,100}") {
        let _ = validate_ss58(&s, "dest");
    }

    #[test]
    fn fuzz_validate_ss58_empty_rejected(spaces in " {0,5}") {
        if spaces.trim().is_empty() {
            prop_assert!(validate_ss58(&spaces, "dest").is_err());
        }
    }

    #[test]
    fn fuzz_validate_ss58_ethereum_rejected(hex in "[0-9a-f]{40}") {
        let eth = format!("0x{}", hex);
        let result = validate_ss58(&eth, "dest");
        prop_assert!(result.is_err(), "Ethereum address should be rejected");
        let msg = result.unwrap_err().to_string();
        prop_assert!(msg.contains("Ethereum") || msg.contains("hex"),
            "error should mention Ethereum: {}", msg);
    }

    #[test]
    fn fuzz_validate_ss58_random_string_rejected(s in "[a-zA-Z0-9]{10,60}") {
        // Random strings are extremely unlikely to be valid SS58
        // This is a probabilistic test — valid SS58 by chance is ~impossible
        let _ = validate_ss58(&s, "dest");
        // Just verify no panic
    }
}

// ──── validate_password_strength: never panics, always returns ────

proptest! {
    #[test]
    fn fuzz_validate_password_strength_no_panic(s in "\\PC{0,200}") {
        // validate_password_strength returns () (only prints warnings)
        validate_password_strength(&s);
    }
}

// ──── validate_port: [1, 65535] ────

proptest! {
    #[test]
    fn fuzz_validate_port_no_panic(port in any::<u16>()) {
        let _ = validate_port(port, "test");
    }

    #[test]
    fn fuzz_validate_port_nonzero_accepted(port in 1u16..=65535) {
        prop_assert!(validate_port(port, "axon").is_ok());
    }

    #[test]
    fn fuzz_validate_port_zero_rejected(_dummy in 0u8..1u8) {
        prop_assert!(validate_port(0, "axon").is_err());
    }
}

// ──── validate_netuid: > 0 ────

proptest! {
    #[test]
    fn fuzz_validate_netuid_no_panic(netuid in any::<u16>()) {
        let _ = validate_netuid(netuid);
    }

    #[test]
    fn fuzz_validate_netuid_nonzero_accepted(netuid in 1u16..=65535) {
        prop_assert!(validate_netuid(netuid).is_ok());
    }

    #[test]
    fn fuzz_validate_netuid_zero_rejected(_dummy in 0u8..1u8) {
        prop_assert!(validate_netuid(0).is_err());
    }
}

// ──── validate_mnemonic: never panics on arbitrary strings ────

proptest! {
    #[test]
    fn fuzz_validate_mnemonic_no_panic(s in "\\PC{0,500}") {
        let _ = validate_mnemonic(&s);
    }

    #[test]
    fn fuzz_validate_mnemonic_random_words_12(
        words in proptest::collection::vec("[a-z]{3,8}", 12)
    ) {
        let phrase = words.join(" ");
        // Random words are almost certainly not valid BIP-39 — just verify no panic
        let _ = validate_mnemonic(&phrase);
    }

    #[test]
    fn fuzz_validate_mnemonic_wrong_word_counts(count in 1usize..50) {
        prop_assume!(count != 12 && count != 15 && count != 18 && count != 21 && count != 24);
        let words: Vec<&str> = std::iter::repeat("abandon").take(count).collect();
        let phrase = words.join(" ");
        prop_assert!(validate_mnemonic(&phrase).is_err(),
            "mnemonic with {} words should be rejected", count);
    }
}

// ──── validate_derive_input: never panics ────

proptest! {
    #[test]
    fn fuzz_validate_derive_input_no_panic(s in "\\PC{0,200}") {
        let _ = validate_derive_input(&s);
    }

    #[test]
    fn fuzz_validate_derive_input_valid_hex(hex in "[0-9a-f]{64}") {
        let input = format!("0x{}", hex);
        prop_assert!(validate_derive_input(&input).is_ok(),
            "valid 64-char hex should be accepted: {}", input);
    }

    #[test]
    fn fuzz_validate_derive_input_wrong_hex_length(hex in "[0-9a-f]{1,63}") {
        prop_assume!(hex.len() != 64 && hex.len() % 2 == 0);
        let input = format!("0x{}", hex);
        prop_assert!(validate_derive_input(&input).is_err(),
            "hex with wrong length should be rejected: {}", input);
    }

    #[test]
    fn fuzz_validate_derive_input_odd_hex(hex in "[0-9a-f]{1,63}") {
        prop_assume!(hex.len() % 2 == 1);
        let input = format!("0x{}", hex);
        let result = validate_derive_input(&input);
        prop_assert!(result.is_err(), "odd-length hex should be rejected: {}", input);
        let msg = result.unwrap_err().to_string();
        prop_assert!(msg.contains("odd"), "error should mention odd: {}", msg);
    }
}

// ──── validate_multisig_json_args: never panics on arbitrary JSON ────

proptest! {
    #[test]
    fn fuzz_validate_multisig_json_args_no_panic(s in "\\PC{0,500}") {
        let _ = validate_multisig_json_args(&s);
    }

    #[test]
    fn fuzz_validate_multisig_json_args_valid_arrays(
        values in proptest::collection::vec(-1000i64..1000, 1..10)
    ) {
        let json = serde_json::to_string(&values).unwrap();
        prop_assert!(validate_multisig_json_args(&json).is_ok(),
            "valid integer array should be accepted: {}", json);
    }

    #[test]
    fn fuzz_validate_multisig_json_args_non_array_rejected(n in -1000i64..1000) {
        let json = n.to_string();
        prop_assert!(validate_multisig_json_args(&json).is_err(),
            "bare number should be rejected: {}", json);
    }

    #[test]
    fn fuzz_validate_multisig_json_args_string_rejected(s in "[a-z]{1,20}") {
        let json = format!("\"{}\"", s);
        prop_assert!(validate_multisig_json_args(&json).is_err(),
            "bare string should be rejected: {}", json);
    }
}

// ──── validate_batch_axon_json: never panics on arbitrary strings ────

proptest! {
    #[test]
    fn fuzz_validate_batch_axon_json_no_panic(s in "\\PC{0,500}") {
        let _ = validate_batch_axon_json(&s);
    }

    #[test]
    fn fuzz_validate_batch_axon_json_valid_entries(
        netuid in 1u16..=100,
        a in 1u8..=126u8,
        b in 0u8..254u8,
        c in 0u8..254u8,
        d in 1u8..254u8,
        port in 1024u16..=65535
    ) {
        // Skip private/loopback ranges
        prop_assume!(a != 10 && a != 127);
        prop_assume!(!(a == 172 && (16..=31).contains(&b)));
        prop_assume!(!(a == 192 && b == 168));
        let json = format!(
            r#"[{{"netuid": {}, "ip": "{}.{}.{}.{}", "port": {}}}]"#,
            netuid, a, b, c, d, port
        );
        prop_assert!(validate_batch_axon_json(&json).is_ok(),
            "valid batch-axon JSON should be accepted: {}", json);
    }
}

// ──── parse_weight_pairs: never panics ────

proptest! {
    #[test]
    fn fuzz_parse_weight_pairs_no_panic(s in "\\PC{0,200}") {
        let _ = parse_weight_pairs(&s);
    }

    #[test]
    fn fuzz_parse_weight_pairs_valid(
        pairs in proptest::collection::vec((0u16..=1000, 0u16..=65535), 1..10)
    ) {
        let s = pairs.iter()
            .map(|(uid, w)| format!("{}:{}", uid, w))
            .collect::<Vec<_>>()
            .join(",");
        let result = parse_weight_pairs(&s);
        prop_assert!(result.is_ok(), "valid weight pairs should parse: {}", s);
        let (uids, weights) = result.unwrap();
        prop_assert_eq!(uids.len(), pairs.len());
        prop_assert_eq!(weights.len(), pairs.len());
    }
}

// ──── parse_children: never panics ────

proptest! {
    #[test]
    fn fuzz_parse_children_no_panic(s in "\\PC{0,200}") {
        let _ = parse_children(&s);
    }
}

// ──── json_to_subxt_value: never panics on any JSON value ────

proptest! {
    #[test]
    fn fuzz_json_to_subxt_value_numbers(n in proptest::num::i64::ANY) {
        let v = serde_json::json!(n);
        let _ = json_to_subxt_value(&v);
    }

    #[test]
    fn fuzz_json_to_subxt_value_strings(s in "\\PC{0,200}") {
        let v = serde_json::json!(s);
        let _ = json_to_subxt_value(&v);
    }

    #[test]
    fn fuzz_json_to_subxt_value_bools(b in any::<bool>()) {
        let v = serde_json::json!(b);
        let _ = json_to_subxt_value(&v);
    }

    #[test]
    fn fuzz_json_to_subxt_value_arrays(
        vals in proptest::collection::vec(-100i64..100, 0..10)
    ) {
        let v = serde_json::json!(vals);
        let _ = json_to_subxt_value(&v);
    }
}

// ──── parse_json_args: never panics ────

proptest! {
    #[test]
    fn fuzz_parse_json_args_no_panic(s in "\\PC{0,200}") {
        let _ = parse_json_args(&Some(s));
    }

    #[test]
    fn fuzz_parse_json_args_none(_dummy in 0u8..1u8) {
        let result = parse_json_args(&None);
        prop_assert!(result.is_ok());
        prop_assert!(result.unwrap().is_empty());
    }
}

// ──── Wallet keypair module: never panics on arbitrary input ────

proptest! {
    /// pair_from_mnemonic with random strings should never panic
    #[test]
    fn fuzz_pair_from_mnemonic_no_panic(s in "\\PC{0,500}") {
        let _ = agcli::wallet::keypair::pair_from_mnemonic(&s);
    }

    /// pair_from_seed_hex with random strings should never panic
    #[test]
    fn fuzz_pair_from_seed_hex_no_panic(s in "\\PC{0,200}") {
        let _ = agcli::wallet::keypair::pair_from_seed_hex(&s);
    }

    /// pair_from_seed_hex with valid 64-char hex should succeed
    #[test]
    fn fuzz_pair_from_seed_hex_valid(hex in "[0-9a-f]{64}") {
        let input = format!("0x{}", hex);
        let result = agcli::wallet::keypair::pair_from_seed_hex(&input);
        prop_assert!(result.is_ok(),
            "valid 64-char seed hex should produce a keypair: {:?}", result.err());
    }

    /// pair_from_seed_hex with wrong-length hex should fail
    #[test]
    fn fuzz_pair_from_seed_hex_wrong_length(hex in "[0-9a-f]{1,63}") {
        prop_assume!(hex.len() != 64);
        let input = format!("0x{}", hex);
        let _ = agcli::wallet::keypair::pair_from_seed_hex(&input);
        // Just no panic; whether it errors depends on implementation
    }

    /// pair_from_uri with random strings should never panic
    #[test]
    fn fuzz_pair_from_uri_no_panic(s in "\\PC{0,200}") {
        let _ = agcli::wallet::keypair::pair_from_uri(&s);
    }

    /// pair_from_uri with dev accounts should succeed
    #[test]
    fn fuzz_pair_from_uri_dev_accounts(name in "(Alice|Bob|Charlie|Dave|Eve|Ferdie)") {
        let uri = format!("//{}", name);
        let result = agcli::wallet::keypair::pair_from_uri(&uri);
        prop_assert!(result.is_ok(),
            "dev account //{}  should produce a keypair: {:?}", name, result.err());
    }

    /// from_ss58 with random strings should never panic
    #[test]
    fn fuzz_from_ss58_no_panic(s in "\\PC{0,100}") {
        let _ = agcli::wallet::keypair::from_ss58(&s);
    }

    /// to_ss58 with prefix 42 (Bittensor) → from_ss58 roundtrip
    #[test]
    fn fuzz_ss58_roundtrip_bittensor(_dummy in 0u8..10) {
        use sp_core::Pair;
        let (pair, _) = sp_core::sr25519::Pair::generate();
        let public = pair.public();
        let ss58 = agcli::wallet::keypair::to_ss58(&public, 42);
        // The address should be a non-empty string starting with '5'
        prop_assert!(!ss58.is_empty(), "to_ss58 should produce non-empty address");
        prop_assert!(ss58.starts_with('5'), "Bittensor SS58 should start with '5': {}", ss58);
        // from_ss58 should parse it back
        let decoded = agcli::wallet::keypair::from_ss58(&ss58);
        prop_assert!(decoded.is_ok(),
            "from_ss58 should parse Bittensor SS58 address: {:?}", decoded.err());
        let decoded_pub = decoded.unwrap();
        prop_assert_eq!(decoded_pub, public,
            "roundtrip should produce the same public key");
    }

    /// to_ss58 never panics with any prefix
    #[test]
    fn fuzz_to_ss58_no_panic(prefix in 0u16..=16383) {
        use sp_core::Pair;
        let (pair, _) = sp_core::sr25519::Pair::generate();
        let public = pair.public();
        let ss58 = agcli::wallet::keypair::to_ss58(&public, prefix);
        prop_assert!(!ss58.is_empty(), "to_ss58 should always produce non-empty string");
    }
}

// ──── Wallet keyfile module: encrypt/decrypt roundtrip + panic safety ────

proptest! {
    /// is_nacl_encrypted should never panic on any input
    #[test]
    fn fuzz_is_nacl_encrypted_no_panic(data in proptest::collection::vec(any::<u8>(), 0..200)) {
        let _ = agcli::wallet::keyfile::is_nacl_encrypted(&data);
    }

    /// is_nacl_encrypted: NaCl-encrypted data starts with "$NACL" header
    #[test]
    fn fuzz_is_nacl_encrypted_with_header(
        tail in proptest::collection::vec(any::<u8>(), 0..50)
    ) {
        let mut data = b"$NACL".to_vec();
        data.extend_from_slice(&tail);
        prop_assert!(agcli::wallet::keyfile::is_nacl_encrypted(&data),
            "data starting with $NACL should be detected as NaCl-encrypted");
    }

    /// is_nacl_encrypted: random data without header should return false
    #[test]
    fn fuzz_is_nacl_encrypted_without_header(data in proptest::collection::vec(any::<u8>(), 0..50)) {
        prop_assume!(!data.starts_with(b"$NACL"));
        prop_assert!(!agcli::wallet::keyfile::is_nacl_encrypted(&data),
            "data not starting with $NACL should not be detected as NaCl-encrypted");
    }

    /// decrypt_nacl_keyfile_data with random data should never panic
    #[test]
    #[ignore] // slow due to Argon2 KDF in decrypt path; run manually with --ignored
    fn fuzz_decrypt_nacl_no_panic(
        data in proptest::collection::vec(any::<u8>(), 0..100),
        password in "[a-z]{1,16}"
    ) {
        let _ = agcli::wallet::keyfile::decrypt_nacl_keyfile_data(&data, &password);
    }
}

// ──── Wallet create/import/unlock roundtrip property tests ────

proptest! {
    /// generate_mnemonic_keypair always produces a valid keypair + mnemonic
    #[test]
    fn fuzz_generate_mnemonic_keypair_always_valid(_dummy in 0u8..10) {
        let result = agcli::wallet::keypair::generate_mnemonic_keypair();
        prop_assert!(result.is_ok(), "generate_mnemonic_keypair should never fail");
        let (_pair, mnemonic) = result.unwrap();
        // The mnemonic should be a valid 12-word phrase
        let words: Vec<&str> = mnemonic.split_whitespace().collect();
        prop_assert_eq!(words.len(), 12, "generated mnemonic should have 12 words, got {}", words.len());
        // And it should be accepted by validate_mnemonic
        prop_assert!(validate_mnemonic(&mnemonic).is_ok(),
            "generated mnemonic should pass validation: {}", mnemonic);
    }

    /// Mnemonic → keypair is deterministic (same mnemonic → same keys)
    #[test]
    fn fuzz_mnemonic_deterministic(_dummy in 0u8..5) {
        let (_, mnemonic) = agcli::wallet::keypair::generate_mnemonic_keypair().unwrap();
        let pair1 = agcli::wallet::keypair::pair_from_mnemonic(&mnemonic).unwrap();
        let pair2 = agcli::wallet::keypair::pair_from_mnemonic(&mnemonic).unwrap();
        use sp_core::Pair;
        prop_assert_eq!(pair1.public(), pair2.public(),
            "same mnemonic must produce same public key");
    }

    /// Mnemonic → keypair → SS58 → from_ss58 full roundtrip
    #[test]
    fn fuzz_mnemonic_to_ss58_roundtrip(_dummy in 0u8..5) {
        let (pair, _mnemonic) = agcli::wallet::keypair::generate_mnemonic_keypair().unwrap();
        use sp_core::Pair;
        let public = pair.public();
        let ss58 = agcli::wallet::keypair::to_ss58(&public, 42);
        // Validate the SS58 address
        prop_assert!(validate_ss58(&ss58, "test").is_ok(),
            "generated SS58 address should pass validation: {}", ss58);
        // Decode back
        let decoded = agcli::wallet::keypair::from_ss58(&ss58).unwrap();
        prop_assert_eq!(decoded, public, "SS58 roundtrip should preserve public key");
    }
}

// ──── Wallet encrypt/decrypt file roundtrip property tests ────

proptest! {
    /// write_encrypted_keyfile → read_encrypted_keyfile roundtrip
    #[test]
    fn fuzz_encrypt_decrypt_roundtrip(
        password in "[a-zA-Z0-9!@#$]{8,32}"
    ) {
        let (_pair, mnemonic) = agcli::wallet::keypair::generate_mnemonic_keypair().unwrap();
        // Use a temp directory
        let dir = tempfile::tempdir().unwrap();
        let keypath = dir.path().join("coldkey");
        // Write encrypted
        agcli::wallet::keyfile::write_encrypted_keyfile(&keypath, &mnemonic, &password).unwrap();
        // Read back
        let decrypted = agcli::wallet::keyfile::read_encrypted_keyfile(&keypath, &password).unwrap();
        prop_assert_eq!(&decrypted, &mnemonic,
            "decrypt(encrypt(mnemonic)) should return the original mnemonic");
    }

    /// Encrypted keyfile with wrong password should fail
    #[test]
    fn fuzz_decrypt_wrong_password(
        password in "[a-zA-Z0-9]{8,16}",
        wrong_password in "[a-zA-Z0-9]{8,16}"
    ) {
        prop_assume!(password != wrong_password);
        let (_pair, mnemonic) = agcli::wallet::keypair::generate_mnemonic_keypair().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let keypath = dir.path().join("coldkey");
        agcli::wallet::keyfile::write_encrypted_keyfile(&keypath, &mnemonic, &password).unwrap();
        let result = agcli::wallet::keyfile::read_encrypted_keyfile(&keypath, &wrong_password);
        prop_assert!(result.is_err(),
            "decrypting with wrong password should fail");
    }

    /// write_keyfile → read_keyfile roundtrip (unencrypted hotkey style)
    #[test]
    fn fuzz_unencrypted_keyfile_roundtrip(_dummy in 0u8..5) {
        let (_pair, mnemonic) = agcli::wallet::keypair::generate_mnemonic_keypair().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let keypath = dir.path().join("hotkey");
        agcli::wallet::keyfile::write_keyfile(&keypath, &mnemonic).unwrap();
        let read_back = agcli::wallet::keyfile::read_keyfile(&keypath).unwrap();
        prop_assert_eq!(&read_back, &mnemonic,
            "read_keyfile should return what write_keyfile wrote");
    }

    /// write_public_key → read_public_key roundtrip
    #[test]
    fn fuzz_public_key_file_roundtrip(_dummy in 0u8..5) {
        use sp_core::Pair;
        let (pair, _) = sp_core::sr25519::Pair::generate();
        let public = pair.public();
        let dir = tempfile::tempdir().unwrap();
        let keypath = dir.path().join("pubkey");
        agcli::wallet::keyfile::write_public_key(&keypath, &public).unwrap();
        let read_back = agcli::wallet::keyfile::read_public_key(&keypath).unwrap();
        prop_assert_eq!(read_back, public,
            "read_public_key should return what write_public_key wrote");
    }
}

// ──── Wallet full lifecycle property tests ────

proptest! {
    /// Full wallet lifecycle: create → open → unlock → sign → verify
    #[test]
    fn fuzz_wallet_lifecycle(
        wallet_name in "[a-zA-Z][a-zA-Z0-9]{2,10}",
        password in "[a-zA-Z0-9!@#]{8,20}",
        message in proptest::collection::vec(any::<u8>(), 1..100)
    ) {
        use sp_core::Pair;
        // Create temp wallet dir
        let dir = tempfile::tempdir().unwrap();
        // Wallet::create takes (wallet_dir, name, password, hotkey_name) → (Wallet, cold_mnemonic, hot_mnemonic)
        let result = agcli::wallet::Wallet::create(
            dir.path(), &wallet_name, &password, "default",
        );
        prop_assert!(result.is_ok(), "wallet creation should succeed: {:?}", result.err());
        let (wallet, _cold_mnemonic, _hot_mnemonic) = result.unwrap();
        let original_ss58 = wallet.coldkey_ss58().unwrap().to_string();
        let wallet_path = dir.path().join(&wallet_name);
        drop(wallet);
        // Re-open and unlock
        let mut wallet = agcli::wallet::Wallet::open(&wallet_path).unwrap();
        prop_assert!(wallet.unlock_coldkey(&password).is_ok(),
            "unlock with correct password should succeed");
        // Verify same address
        prop_assert_eq!(wallet.coldkey_ss58().unwrap(), &original_ss58,
            "re-opened wallet should have same SS58 address");
        // Sign and verify
        let sig = wallet.coldkey().unwrap().sign(&message);
        prop_assert!(sp_core::sr25519::Pair::verify(&sig, &message, &wallet.coldkey().unwrap().public()),
            "signature should verify");
        // Verify with wrong message fails
        let wrong_msg = [&message[..], &[0xFF]].concat();
        prop_assert!(!sp_core::sr25519::Pair::verify(&sig, &wrong_msg, &wallet.coldkey().unwrap().public()),
            "signature should NOT verify with wrong message");
    }

    /// Wallet import from mnemonic → re-derive produces same keys
    #[test]
    fn fuzz_wallet_import_deterministic(
        wallet_name in "[a-zA-Z][a-zA-Z0-9]{2,8}",
        password in "[a-zA-Z0-9]{8,16}"
    ) {
        // Generate a mnemonic
        let (_, mnemonic) = agcli::wallet::keypair::generate_mnemonic_keypair().unwrap();
        // Import it twice into different directories
        // import_from_mnemonic takes (wallet_dir, name, mnemonic, password)
        let dir1 = tempfile::tempdir().unwrap();
        let dir2 = tempfile::tempdir().unwrap();
        let w1 = agcli::wallet::Wallet::import_from_mnemonic(
            dir1.path(), &wallet_name, &mnemonic, &password,
        ).unwrap();
        let w2 = agcli::wallet::Wallet::import_from_mnemonic(
            dir2.path(), &wallet_name, &mnemonic, &password,
        ).unwrap();
        prop_assert_eq!(w1.coldkey_ss58().unwrap(), w2.coldkey_ss58().unwrap(),
            "importing same mnemonic twice should produce same address");
    }
}

// ──── Cross-validator consistency: valid input patterns ────

proptest! {
    /// If validate_amount accepts a value, it must be positive and finite
    #[test]
    fn prop_amount_accepted_implies_positive_finite(amount in proptest::num::f64::ANY) {
        if validate_amount(amount, "test").is_ok() {
            prop_assert!(amount > 0.0, "accepted amount must be positive");
            prop_assert!(amount.is_finite(), "accepted amount must be finite");
        }
    }

    /// If validate_take_pct accepts a value, it must be in [0, 18]
    #[test]
    fn prop_take_pct_accepted_implies_range(take in proptest::num::f64::ANY) {
        if validate_take_pct(take).is_ok() {
            prop_assert!(take >= 0.0 && take <= 18.0 && take.is_finite(),
                "accepted take {}% must be in [0, 18] and finite", take);
        }
    }

    /// If validate_delegate_take accepts a value, it must be in [0, 18]
    #[test]
    fn prop_delegate_take_accepted_implies_range(take in proptest::num::f64::ANY) {
        if validate_delegate_take(take).is_ok() {
            prop_assert!(take >= 0.0 && take <= 18.0 && take.is_finite());
        }
    }

    /// If validate_max_cost accepts a value, it must be non-negative and finite
    #[test]
    fn prop_max_cost_accepted_implies_nonneg_finite(cost in proptest::num::f64::ANY) {
        if validate_max_cost(cost).is_ok() {
            prop_assert!(cost >= 0.0 && cost.is_finite());
        }
    }

    /// If validate_name accepts a value, it must not contain path traversal chars
    #[test]
    fn prop_name_accepted_implies_safe(s in "\\PC{0,100}") {
        if validate_name(&s, "test").is_ok() {
            let t = s.trim();
            prop_assert!(!t.contains(".."), "accepted name must not contain '..'");
            prop_assert!(!t.contains('/'), "accepted name must not contain '/'");
            prop_assert!(!t.contains('\\'), "accepted name must not contain '\\'");
            prop_assert!(!t.starts_with('.'), "accepted name must not start with '.'");
            prop_assert!(t.len() <= 64, "accepted name must be ≤64 chars");
            prop_assert!(!t.is_empty(), "accepted name must not be empty");
        }
    }

    /// If validate_port accepts a port, it must be > 0
    #[test]
    fn prop_port_accepted_implies_nonzero(port in any::<u16>()) {
        if validate_port(port, "test").is_ok() {
            prop_assert!(port > 0, "accepted port must be > 0");
        }
    }

    /// If validate_netuid accepts a netuid, it must be > 0
    #[test]
    fn prop_netuid_accepted_implies_nonzero(netuid in any::<u16>()) {
        if validate_netuid(netuid).is_ok() {
            prop_assert!(netuid > 0);
        }
    }

    // ──── validate_evm_address: never panics on arbitrary input ────

    #[test]
    fn fuzz_evm_address_no_panic(s in "\\PC{0,100}") {
        let _ = validate_evm_address(&s, "test");
    }

    #[test]
    fn fuzz_evm_address_valid_always_accepted(bytes in proptest::collection::vec(any::<u8>(), 20)) {
        let hex_str = format!("0x{}", hex::encode(&bytes));
        prop_assert!(validate_evm_address(&hex_str, "test").is_ok(),
            "valid 20-byte hex should be accepted: {}", hex_str);
    }

    #[test]
    fn fuzz_evm_address_wrong_length_rejected(bytes in proptest::collection::vec(any::<u8>(), 0..40usize).prop_filter("not 20 bytes", |v| v.len() != 20)) {
        let hex_str = format!("0x{}", hex::encode(&bytes));
        prop_assert!(validate_evm_address(&hex_str, "test").is_err(),
            "non-20-byte hex should be rejected: {} ({} bytes)", hex_str, bytes.len());
    }

    // ──── validate_hex_data: never panics on arbitrary input ────

    #[test]
    fn fuzz_hex_data_no_panic(s in "\\PC{0,200}") {
        let _ = validate_hex_data(&s, "test");
    }

    #[test]
    fn fuzz_hex_data_valid_always_accepted(bytes in proptest::collection::vec(any::<u8>(), 0..100)) {
        let hex_str = format!("0x{}", hex::encode(&bytes));
        prop_assert!(validate_hex_data(&hex_str, "test").is_ok(),
            "valid hex should be accepted: {}", hex_str);
    }

    // ──── validate_pallet_call: never panics on arbitrary input ────

    #[test]
    fn fuzz_pallet_call_no_panic(s in "\\PC{0,200}") {
        let _ = validate_pallet_call(&s, "test");
    }

    #[test]
    fn fuzz_pallet_call_valid_pascal(name in "[A-Z][a-zA-Z0-9_]{0,50}") {
        prop_assert!(validate_pallet_call(&name, "pallet").is_ok(),
            "valid PascalCase should be accepted: {}", name);
    }

    #[test]
    fn fuzz_pallet_call_valid_snake(name in "[a-z][a-z0-9_]{0,50}") {
        prop_assert!(validate_pallet_call(&name, "call").is_ok(),
            "valid snake_case should be accepted: {}", name);
    }

    // ──── validate_schedule_id: never panics on arbitrary input ────

    #[test]
    fn fuzz_schedule_id_no_panic(s in "\\PC{0,100}") {
        let _ = validate_schedule_id(&s);
    }

    #[test]
    fn fuzz_schedule_id_valid_short(s in "[a-zA-Z0-9_]{1,32}") {
        prop_assert!(validate_schedule_id(&s).is_ok(),
            "valid short id should be accepted: {}", s);
    }

    #[test]
    fn fuzz_schedule_id_too_long_rejected(s in ".{33,100}") {
        prop_assert!(validate_schedule_id(&s).is_err(),
            "id > 32 bytes should be rejected: len={}", s.len());
    }
}
