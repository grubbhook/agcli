//! Tests for CLI helper functions.
//! Run with: cargo test --test helpers_test

use agcli::cli::helpers::{
    json_to_subxt_value, parse_children, parse_weight_pairs, validate_amount,
    validate_delegate_take, validate_derive_input, validate_emission_weights,
    validate_evm_address, validate_hex_data, validate_ipv4, validate_max_cost,
    validate_mnemonic, validate_multisig_json_args, validate_name, validate_pallet_call,
    validate_schedule_id, validate_symbol, validate_take_pct,
};
use agcli::utils::explain;

#[test]
fn parse_weight_pairs_basic() {
    let (uids, weights) = parse_weight_pairs("0:100,1:200,2:300").unwrap();
    assert_eq!(uids, vec![0, 1, 2]);
    assert_eq!(weights, vec![100, 200, 300]);
}

#[test]
fn parse_weight_pairs_with_spaces() {
    let (uids, weights) = parse_weight_pairs("0: 100, 1: 200").unwrap();
    assert_eq!(uids, vec![0, 1]);
    assert_eq!(weights, vec![100, 200]);
}

#[test]
fn parse_weight_pairs_single() {
    let (uids, weights) = parse_weight_pairs("5:65535").unwrap();
    assert_eq!(uids, vec![5]);
    assert_eq!(weights, vec![65535]);
}

#[test]
fn parse_weight_pairs_invalid() {
    assert!(parse_weight_pairs("0").is_err());
    assert!(parse_weight_pairs("abc:def").is_err());
    assert!(parse_weight_pairs("").is_err());
}

#[test]
fn parse_children_basic() {
    let result = parse_children("1000:5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].0, 1000);
    assert_eq!(
        result[0].1,
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
    );
}

#[test]
fn parse_children_multiple() {
    let result = parse_children("500:5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY,500:5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty").unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].0, 500);
    assert_eq!(result[1].0, 500);
}

#[test]
fn parse_children_invalid() {
    assert!(parse_children("invalid").is_err());
    assert!(parse_children("").is_err());
}

#[test]
fn parse_weight_pairs_overflow_uid() {
    // UID > 65535 should fail
    let result = parse_weight_pairs("70000:100");
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("Invalid UID"),
        "Expected helpful UID error, got: {}",
        msg
    );
}

#[test]
fn parse_weight_pairs_overflow_weight() {
    // Weight > 65535 should fail
    let result = parse_weight_pairs("0:70000");
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("Invalid weight"),
        "Expected helpful weight error, got: {}",
        msg
    );
}

#[test]
fn parse_children_bad_proportion() {
    let result = parse_children("abc:5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY");
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("Invalid proportion"),
        "Expected helpful proportion error, got: {}",
        msg
    );
}

// ──── Explain tests ────

#[test]
fn explain_known_topics() {
    assert!(explain::explain("tempo").is_some());
    assert!(explain::explain("commit-reveal").is_some());
    assert!(explain::explain("commitreveal").is_some());
    assert!(explain::explain("yuma").is_some());
    assert!(explain::explain("amm").is_some());
    assert!(explain::explain("bootstrap").is_some());
    assert!(explain::explain("rate-limits").is_some());
    assert!(explain::explain("stake-weight").is_some());
    assert!(explain::explain("alpha").is_some());
    assert!(explain::explain("emission").is_some());
}

#[test]
fn explain_case_insensitive() {
    assert!(explain::explain("TEMPO").is_some());
    assert!(explain::explain("Commit-Reveal").is_some());
    assert!(explain::explain("AMM").is_some());
}

#[test]
fn explain_unknown_topic() {
    assert!(explain::explain("nonexistent_topic_xyz").is_none());
}

#[test]
fn explain_list_topics_not_empty() {
    let topics = explain::list_topics();
    assert!(
        topics.len() >= 10,
        "Expected at least 10 topics, got {}",
        topics.len()
    );
    for (key, desc) in &topics {
        assert!(!key.is_empty());
        assert!(!desc.is_empty());
    }
}

#[test]
fn explain_content_has_substance() {
    // Each explanation should be non-trivial
    let text = explain::explain("tempo").unwrap();
    assert!(
        text.len() > 100,
        "Explanation too short: {} chars",
        text.len()
    );
    assert!(
        text.contains("blocks"),
        "Tempo explanation should mention blocks"
    );
}

#[test]
fn explain_aliases_work() {
    // "cr" should resolve to commit-reveal
    assert!(explain::explain("cr").is_some());
    // "dtao" should resolve to AMM
    assert!(explain::explain("dtao").is_some());
    // "1000" should resolve to stake-weight
    assert!(explain::explain("1000").is_some());
}

// ──── json_to_subxt_value tests ────

#[test]
fn json_to_subxt_value_number() {
    use agcli::cli::helpers::json_to_subxt_value;
    let val = json_to_subxt_value(&serde_json::json!(42));
    // Should produce a u128 value
    assert_eq!(
        format!("{:?}", val),
        format!("{:?}", subxt::dynamic::Value::u128(42))
    );
}

#[test]
fn json_to_subxt_value_string() {
    use agcli::cli::helpers::json_to_subxt_value;
    let val = json_to_subxt_value(&serde_json::json!("hello"));
    assert_eq!(
        format!("{:?}", val),
        format!("{:?}", subxt::dynamic::Value::string("hello".to_string()))
    );
}

#[test]
fn json_to_subxt_value_bool() {
    use agcli::cli::helpers::json_to_subxt_value;
    let val = json_to_subxt_value(&serde_json::json!(true));
    assert_eq!(
        format!("{:?}", val),
        format!("{:?}", subxt::dynamic::Value::bool(true))
    );
}

#[test]
fn json_to_subxt_value_hex_bytes() {
    use agcli::cli::helpers::json_to_subxt_value;
    let val = json_to_subxt_value(&serde_json::json!("0xdeadbeef"));
    // Should decode as bytes
    let expected = subxt::dynamic::Value::from_bytes(vec![0xde, 0xad, 0xbe, 0xef]);
    assert_eq!(format!("{:?}", val), format!("{:?}", expected));
}

#[test]
fn json_to_subxt_value_array() {
    use agcli::cli::helpers::json_to_subxt_value;
    let val = json_to_subxt_value(&serde_json::json!([1, 2, 3]));
    // Should produce an unnamed composite
    let _formatted = format!("{:?}", val); // Just check it doesn't panic
}

// ──── Pretty mode tests ────

#[test]
fn pretty_mode_flag_toggles() {
    use agcli::cli::helpers::{is_pretty_mode, set_pretty_mode};
    set_pretty_mode(true);
    assert!(is_pretty_mode());
    set_pretty_mode(false);
    assert!(!is_pretty_mode());
}

// ──── Step 18: Batch mode & spending limits tests ────

#[test]
fn batch_mode_flag_sets_global() {
    use agcli::cli::helpers::{is_batch_mode, set_batch_mode};
    set_batch_mode(true);
    assert!(is_batch_mode());
    set_batch_mode(false);
    assert!(!is_batch_mode());
}

#[test]
fn spending_limit_no_config_passes() {
    // No config file → should pass for any amount
    let result = agcli::cli::helpers::check_spending_limit(97, 100.0);
    assert!(
        result.is_ok(),
        "No config should always pass: {:?}",
        result.err()
    );
}

// ──── Step 26: Edge case tests ────

#[test]
fn parse_weight_pairs_empty_string() {
    let result = parse_weight_pairs("");
    assert!(result.is_err(), "empty string should fail");
}

#[test]
fn parse_weight_pairs_only_commas() {
    let result = parse_weight_pairs(",,,");
    assert!(result.is_err(), "commas-only should fail");
}

#[test]
fn parse_weight_pairs_negative_uid() {
    // Negative values can't parse as u16
    let result = parse_weight_pairs("-1:100");
    assert!(result.is_err(), "negative UID should fail");
}

#[test]
fn parse_weight_pairs_extra_colon() {
    let result = parse_weight_pairs("0:100:extra");
    assert!(result.is_err(), "extra colon should fail");
}

#[test]
fn parse_weight_pairs_max_u16_values() {
    // Max u16 values should succeed
    let (uids, weights) = parse_weight_pairs("65535:65535").unwrap();
    assert_eq!(uids, vec![65535]);
    assert_eq!(weights, vec![65535]);
}

#[test]
fn parse_children_empty_hotkey() {
    let result = parse_children("1000:");
    // Empty hotkey should now fail SS58 validation
    assert!(result.is_err(), "empty hotkey should fail SS58 validation");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("empty") || msg.contains("address"), "error msg: {}", msg);
}

#[test]
fn parse_children_zero_proportion() {
    let result = parse_children("0:5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY");
    assert!(result.is_err(), "zero proportion should be rejected");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("non-zero"), "error msg: {}", msg);
}

#[test]
fn json_to_subxt_value_null() {
    use agcli::cli::helpers::json_to_subxt_value;
    let val = json_to_subxt_value(&serde_json::json!(null));
    // Null should not panic — produces a string representation
    let _formatted = format!("{:?}", val);
}

#[test]
fn json_to_subxt_value_object() {
    use agcli::cli::helpers::json_to_subxt_value;
    let val = json_to_subxt_value(&serde_json::json!({"key": "value"}));
    // Objects should not panic — produces a string representation
    let _formatted = format!("{:?}", val);
}

#[test]
fn json_to_subxt_value_negative_number() {
    use agcli::cli::helpers::json_to_subxt_value;
    let val = json_to_subxt_value(&serde_json::json!(-42));
    let _formatted = format!("{:?}", val);
}

#[test]
fn json_to_subxt_value_large_number() {
    use agcli::cli::helpers::json_to_subxt_value;
    let val = json_to_subxt_value(&serde_json::json!(u64::MAX));
    let _formatted = format!("{:?}", val);
}

#[test]
fn json_to_subxt_value_float_as_string() {
    use agcli::cli::helpers::json_to_subxt_value;
    // JSON floats become strings since we can't represent them as u128/i128
    let val = json_to_subxt_value(&serde_json::json!(3.15));
    let _formatted = format!("{:?}", val);
}

#[test]
fn json_to_subxt_value_invalid_hex() {
    use agcli::cli::helpers::json_to_subxt_value;
    // "0x" prefix but invalid hex should fall back to string
    let val = json_to_subxt_value(&serde_json::json!("0xzzzz"));
    let _formatted = format!("{:?}", val);
}

#[test]
fn json_to_subxt_value_empty_array() {
    use agcli::cli::helpers::json_to_subxt_value;
    let val = json_to_subxt_value(&serde_json::json!([]));
    let _formatted = format!("{:?}", val);
}

#[test]
fn ss58_validation_invalid_address() {
    use agcli::wallet::keypair;
    let result = keypair::from_ss58("not_an_address");
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("Invalid SS58"),
        "Expected SS58 error, got: {}",
        msg
    );
}

#[test]
fn ss58_validation_empty_address() {
    use agcli::wallet::keypair;
    let result = keypair::from_ss58("");
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("Empty address"),
        "Expected empty address error, got: {}",
        msg
    );
}

#[test]
fn ss58_validation_short_address() {
    use agcli::wallet::keypair;
    let result = keypair::from_ss58("5abc");
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("too short"),
        "Expected 'too short' error, got: {}",
        msg
    );
}

#[test]
fn ss58_validation_valid_address() {
    use agcli::wallet::keypair;
    assert!(keypair::from_ss58("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").is_ok());
    assert!(keypair::from_ss58("invalid").is_err());
    assert!(keypair::from_ss58("").is_err());
}

#[test]
fn explain_all_topics_have_content() {
    let topics = explain::list_topics();
    for (key, _desc) in &topics {
        let content = explain::explain(key);
        assert!(
            content.is_some(),
            "Topic '{}' listed but has no content",
            key
        );
        assert!(
            content.unwrap().len() > 50,
            "Topic '{}' content is too short",
            key
        );
    }
}

#[test]
fn balance_from_tao_negative() {
    // Negative TAO should produce 0 (wraps due to as u64)
    use agcli::types::Balance;
    let b = Balance::from_tao(-1.0);
    // This is an edge case — negative f64 cast to u64 wraps. Document behavior.
    // Just verify it doesn't panic.
    let _ = b.rao();
}

#[test]
fn balance_from_tao_very_large() {
    use agcli::types::Balance;
    // Very large TAO — near u64 max
    let b = Balance::from_tao(18.0); // 18 TAO = 18_000_000_000 RAO, well within u64
    assert_eq!(b.rao(), 18_000_000_000);
}

#[test]
fn balance_display_tao_precision() {
    use agcli::types::Balance;
    let b = Balance::from_rao(1); // 1 RAO = 0.000000001 TAO
    let s = b.display_tao();
    assert!(
        s.contains("0.000000001"),
        "Expected full precision, got: {}",
        s
    );
}

#[test]
fn format_tao_zero() {
    use agcli::types::Balance;
    use agcli::utils::format::format_tao;
    let s = format_tao(Balance::ZERO);
    assert!(s.contains("0.0"), "Expected zero TAO display, got: {}", s);
}

#[test]
fn explain_coldkey_swap_topic() {
    let content = explain::explain("coldkey-swap");
    assert!(content.is_some(), "coldkey-swap topic should exist");
    let text = content.unwrap();
    assert!(
        text.len() > 200,
        "coldkey-swap explanation should be substantial"
    );
    assert!(text.contains("schedule"), "should mention scheduling");
    assert!(text.contains("security"), "should mention security");
}

#[test]
fn explain_coldkey_swap_aliases() {
    // All aliases should resolve
    assert!(explain::explain("coldkey").is_some());
    assert!(explain::explain("ckswap").is_some());
    assert!(explain::explain("coldkeyswap").is_some());
    assert!(explain::explain("COLDKEY-SWAP").is_some());
}

#[test]
fn explain_topic_count_includes_coldkey_swap() {
    let topics = explain::list_topics();
    let has_ck = topics.iter().any(|(k, _)| *k == "coldkey-swap");
    assert!(has_ck, "coldkey-swap should be in list_topics()");
    assert!(
        topics.len() >= 22,
        "Expected at least 22 topics (19 + governance, senate, mev-shield), got {}",
        topics.len()
    );
}

// ──── Step 32: governance, senate, mev-shield explain topics ────

#[test]
fn explain_governance_topic() {
    let content = explain::explain("governance");
    assert!(content.is_some(), "governance topic should exist");
    let text = content.unwrap();
    assert!(text.contains("GOVERNANCE"), "should have title");
    assert!(text.contains("proposal"), "should mention proposals");
}

#[test]
fn explain_governance_aliases() {
    assert!(explain::explain("gov").is_some());
    assert!(explain::explain("proposals").is_some());
    assert!(explain::explain("GOV").is_some());
}

#[test]
fn explain_senate_topic() {
    let content = explain::explain("senate");
    assert!(content.is_some(), "senate topic should exist");
    let text = content.unwrap();
    assert!(text.contains("SENATE"), "should have title");
    assert!(text.contains("triumvirate") || text.contains("Triumvirate"));
}

#[test]
fn explain_senate_aliases() {
    assert!(explain::explain("triumvirate").is_some());
    assert!(explain::explain("SENATE").is_some());
}

#[test]
fn explain_mev_shield_topic() {
    let content = explain::explain("mev-shield");
    assert!(content.is_some(), "mev-shield topic should exist");
    let text = content.unwrap();
    assert!(text.contains("MEV"), "should contain MEV");
    assert!(text.contains("protection") || text.contains("shield") || text.contains("Shield"));
}

#[test]
fn explain_mev_aliases() {
    assert!(explain::explain("mev").is_some());
    assert!(explain::explain("mevshield").is_some());
    assert!(explain::explain("mevprotection").is_some());
}

// ──── Step 33 — explain limits, hyperparams, axon ────

#[test]
fn explain_limits_topic() {
    let content = explain::explain("limits");
    assert!(content.is_some(), "limits topic should exist");
    let text = content.unwrap();
    assert!(text.contains("weight"), "should mention weight limits");
    assert!(text.contains("registration") || text.contains("Registration"));
}

#[test]
fn explain_limits_aliases() {
    assert!(explain::explain("networklimits").is_some());
    assert!(explain::explain("chainlimits").is_some());
    assert!(explain::explain("LIMITS").is_some());
}

#[test]
fn explain_hyperparams_topic() {
    let content = explain::explain("hyperparams");
    assert!(content.is_some(), "hyperparams topic should exist");
    let text = content.unwrap();
    assert!(text.contains("tempo"), "should mention tempo");
    assert!(text.contains("rho") || text.contains("kappa"));
}

#[test]
fn explain_hyperparams_aliases() {
    assert!(explain::explain("hyperparameters").is_some());
    assert!(explain::explain("params").is_some());
}

#[test]
fn explain_axon_topic() {
    let content = explain::explain("axon");
    assert!(content.is_some(), "axon topic should exist");
    let text = content.unwrap();
    assert!(text.contains("endpoint") || text.contains("IP"));
    assert!(text.contains("miner") || text.contains("Miner"));
}

#[test]
fn explain_axon_aliases() {
    assert!(explain::explain("axoninfo").is_some());
    assert!(explain::explain("serving").is_some());
}

#[test]
fn explain_topic_count_28() {
    let topics = explain::list_topics();
    assert!(
        topics.len() >= 28,
        "Expected at least 28 topics, got {}",
        topics.len()
    );
}

#[test]
fn explain_take_topic() {
    let content = explain::explain("take");
    assert!(content.is_some(), "take topic should exist");
    let text = content.unwrap();
    assert!(text.contains("percentage") || text.contains("dividends"));
    assert!(text.contains("delegate") || text.contains("Delegate"));
}

#[test]
fn explain_take_aliases() {
    assert!(explain::explain("delegate-take").is_some());
    assert!(explain::explain("validator_take").is_some());
}

#[test]
fn explain_recycle_topic() {
    let content = explain::explain("recycle");
    assert!(content.is_some(), "recycle topic should exist");
    let text = content.unwrap();
    assert!(text.contains("alpha") || text.contains("Alpha"));
    assert!(text.contains("burn") || text.contains("Burn"));
}

#[test]
fn explain_recycle_aliases() {
    assert!(explain::explain("burn-alpha").is_some());
    assert!(explain::explain("recyclealpha").is_some());
}

#[test]
fn explain_pow_topic() {
    let content = explain::explain("pow");
    assert!(content.is_some(), "pow topic should exist");
    let text = content.unwrap();
    assert!(text.contains("difficulty") || text.contains("nonce"));
    assert!(text.contains("registration") || text.contains("Registration"));
}

#[test]
fn explain_pow_aliases() {
    assert!(explain::explain("pow-registration").is_some());
    assert!(explain::explain("proof-of-work").is_some());
}

// ──── Step 34 — archive explain topic ────

#[test]
fn explain_archive_topic() {
    let content = explain::explain("archive");
    assert!(content.is_some(), "archive topic should exist");
    let text = content.unwrap();
    assert!(text.contains("historical") || text.contains("Historical"));
    assert!(text.contains("--at-block") || text.contains("at_block"));
    assert!(text.contains("--network archive"));
}

#[test]
fn explain_archive_aliases() {
    assert!(explain::explain("archive-node").is_some());
    assert!(explain::explain("historical").is_some());
    assert!(explain::explain("wayback").is_some());
    assert!(explain::explain("ARCHIVE").is_some());
}

#[test]
fn explain_topic_count_29() {
    let topics = explain::list_topics();
    assert!(
        topics.len() >= 29,
        "Expected at least 29 topics, got {}",
        topics.len()
    );
}

// ──── Step 34 — Network::Archive ────

#[test]
fn network_archive_url() {
    use agcli::types::network::Network;
    let net = Network::Archive;
    assert!(net.ws_url().starts_with("wss://"));
    assert!(matches!(net, Network::Archive));
    assert_eq!(format!("{}", net), "archive");
}

#[test]
fn network_finney_not_archive() {
    use agcli::types::network::Network;
    assert!(!matches!(Network::Finney, Network::Archive));
    assert!(!matches!(Network::Test, Network::Archive));
    assert!(!matches!(Network::Local, Network::Archive));
}

// ──── Amount validation tests ────

#[test]
fn validate_amount_positive() {
    assert!(validate_amount(1.0, "test").is_ok());
    assert!(validate_amount(0.000000001, "test").is_ok());
    assert!(validate_amount(1000000.0, "test").is_ok());
}

#[test]
fn validate_amount_zero_rejects() {
    let result = validate_amount(0.0, "test");
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("greater than zero"), "error msg: {}", msg);
}

#[test]
fn validate_amount_negative_rejects() {
    let result = validate_amount(-1.0, "stake amount");
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("negative"), "error msg: {}", msg);
    assert!(msg.contains("stake amount"), "should include label: {}", msg);
}

#[test]
fn validate_amount_nan_rejects() {
    let result = validate_amount(f64::NAN, "test");
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("finite"), "error msg: {}", msg);
}

#[test]
fn validate_amount_infinity_rejects() {
    let result = validate_amount(f64::INFINITY, "test");
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("finite"), "error msg: {}", msg);
}

#[test]
fn validate_amount_neg_infinity_rejects() {
    let result = validate_amount(f64::NEG_INFINITY, "test");
    assert!(result.is_err());
}

// ──── Take percentage validation tests ────

#[test]
fn validate_take_valid_range() {
    assert!(validate_take_pct(0.0).is_ok());
    assert!(validate_take_pct(9.0).is_ok());
    assert!(validate_take_pct(18.0).is_ok());
}

#[test]
fn validate_take_over_max_rejects() {
    let result = validate_take_pct(18.01);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("18%"), "should mention max: {}", msg);
}

#[test]
fn validate_take_negative_rejects() {
    let result = validate_take_pct(-1.0);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("negative"), "error msg: {}", msg);
}

#[test]
fn validate_take_nan_rejects() {
    let result = validate_take_pct(f64::NAN);
    assert!(result.is_err());
}

#[test]
fn validate_take_very_large_rejects() {
    let result = validate_take_pct(100.0);
    assert!(result.is_err());
}

// ──── Comprehensive parse_children tests ────

#[test]
fn parse_children_whitespace_around_colons() {
    let result = parse_children("1000 : 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY");
    assert!(result.is_ok(), "whitespace around colon: {:?}", result.err());
    let children = result.unwrap();
    assert_eq!(children[0].0, 1000);
}

#[test]
fn parse_children_whitespace_around_commas() {
    let alice = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
    let bob = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty";
    let result = parse_children(&format!("500:{} , 500:{}", alice, bob));
    assert!(result.is_ok(), "whitespace around commas: {:?}", result.err());
    assert_eq!(result.unwrap().len(), 2);
}

#[test]
fn parse_children_large_proportion() {
    let alice = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
    let result = parse_children(&format!("{}:{}", u64::MAX, alice));
    assert!(result.is_ok(), "u64 max proportion: {:?}", result.err());
    assert_eq!(result.unwrap()[0].0, u64::MAX);
}

#[test]
fn parse_children_negative_proportion() {
    let alice = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
    let result = parse_children(&format!("-1:{}", alice));
    assert!(result.is_err(), "negative proportion should fail");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("Invalid proportion"), "error msg: {}", msg);
}

#[test]
fn parse_children_float_proportion() {
    let alice = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
    let result = parse_children(&format!("1.5:{}", alice));
    assert!(result.is_err(), "float proportion should fail");
}

#[test]
fn parse_children_multiple_colons() {
    // With first-colon split, "1000:5Abc:extra" parses proportion=1000, hotkey="5Abc:extra"
    // The hotkey "5Abc:extra" should fail SS58 validation
    let result = parse_children("1000:5Abc:extra");
    assert!(result.is_err(), "garbage hotkey should fail SS58 validation");
}

#[test]
fn parse_children_only_commas() {
    let result = parse_children(",,,");
    assert!(result.is_err(), "comma-only input should fail");
}

#[test]
fn parse_children_single_colon_only() {
    let result = parse_children(":");
    // proportion is empty → parse error
    assert!(result.is_err(), "colon-only should fail");
}

#[test]
fn parse_children_invalid_ss58_hotkey() {
    let result = parse_children("1000:5NotAValidAddress");
    assert!(result.is_err(), "invalid SS58 hotkey should be rejected");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("child hotkey") || msg.contains("SS58") || msg.contains("checksum"),
        "error should mention hotkey validation: {}", msg);
}

#[test]
fn parse_children_ethereum_address_as_hotkey() {
    let result = parse_children("1000:0x742d35Cc6634C0532925a3b844BcEfe0390a94e0");
    assert!(result.is_err(), "Ethereum address should be rejected");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("Ethereum") || msg.contains("0x"),
        "error should hint about Ethereum address: {}", msg);
}

// ──── Balance edge case tests ────

#[test]
fn balance_from_tao_zero() {
    use agcli::types::Balance;
    let b = Balance::from_tao(0.0);
    assert_eq!(b.rao(), 0);
    assert_eq!(b.tao(), 0.0);
}

#[test]
fn balance_from_rao_zero() {
    use agcli::types::Balance;
    let b = Balance::from_rao(0);
    assert_eq!(b.rao(), 0);
    assert_eq!(b.tao(), 0.0);
}

#[test]
fn balance_from_tao_fractional() {
    use agcli::types::Balance;
    let b = Balance::from_tao(1.5);
    assert_eq!(b.rao(), 1_500_000_000);
}

#[test]
fn balance_from_tao_one_rao() {
    use agcli::types::Balance;
    // 0.000000001 TAO = 1 RAO
    let b = Balance::from_tao(0.000000001);
    assert_eq!(b.rao(), 1);
}

#[test]
fn balance_roundtrip() {
    use agcli::types::Balance;
    let original_rao = 12_345_678_901u64;
    let b = Balance::from_rao(original_rao);
    let tao = b.tao();
    let b2 = Balance::from_tao(tao);
    // May lose precision due to f64, but should be close
    let diff = (b2.rao() as i64 - original_rao as i64).unsigned_abs();
    assert!(diff <= 1, "roundtrip error too large: {} vs {}", b2.rao(), original_rao);
}

#[test]
fn balance_display_tao_whole_number() {
    use agcli::types::Balance;
    let b = Balance::from_tao(100.0);
    let s = b.display_tao();
    assert!(s.contains("100"), "display should contain 100: {}", s);
}

// ──── Sprint 14: CSV escaping ────

#[test]
fn csv_escape_plain_string() {
    use agcli::cli::helpers::csv_escape;
    assert_eq!(csv_escape("hello"), "hello");
    assert_eq!(csv_escape("simple_name"), "simple_name");
    assert_eq!(csv_escape("42"), "42");
}

#[test]
fn csv_escape_with_comma() {
    use agcli::cli::helpers::csv_escape;
    assert_eq!(csv_escape("Subnet, Inc."), "\"Subnet, Inc.\"");
}

#[test]
fn csv_escape_with_quotes() {
    use agcli::cli::helpers::csv_escape;
    assert_eq!(csv_escape("say \"hello\""), "\"say \"\"hello\"\"\"");
}

#[test]
fn csv_escape_with_newline() {
    use agcli::cli::helpers::csv_escape;
    assert_eq!(csv_escape("line1\nline2"), "\"line1\nline2\"");
}

#[test]
fn csv_escape_with_carriage_return() {
    use agcli::cli::helpers::csv_escape;
    assert_eq!(csv_escape("line1\rline2"), "\"line1\rline2\"");
}

#[test]
fn csv_escape_with_all_special_chars() {
    use agcli::cli::helpers::csv_escape;
    let val = csv_escape("a,b\"c\nd");
    assert!(val.starts_with('"') && val.ends_with('"'));
    assert!(val.contains("\"\""));
}

#[test]
fn csv_escape_empty_string() {
    use agcli::cli::helpers::csv_escape;
    assert_eq!(csv_escape(""), "");
}

#[test]
fn csv_row_from_basic() {
    use agcli::cli::helpers::csv_row_from;
    assert_eq!(csv_row_from(&["a", "b", "c"]), "a,b,c");
    assert_eq!(csv_row_from(&["Subnet, Inc.", "42"]), "\"Subnet, Inc.\",42");
}

// ──── Sprint 14: owner-workflow explain topic ────

#[test]
fn explain_owner_workflow_topic() {
    let content = explain::explain("owner-workflow");
    assert!(content.is_some(), "owner-workflow topic should exist");
    let text = content.unwrap();
    assert!(text.contains("SUBNET OWNER WORKFLOW"), "should have title");
    assert!(text.contains("register"), "should mention registration");
    assert!(text.contains("set-param"), "should mention set-param");
    assert!(text.contains("monitor"), "should mention monitoring");
}

#[test]
fn explain_owner_workflow_aliases() {
    assert!(explain::explain("ow").is_some());
    assert!(explain::explain("subnet-owner").is_some());
    assert!(explain::explain("owner-guide").is_some());
}

#[test]
fn explain_topic_count_31() {
    let topics = explain::list_topics();
    assert!(
        topics.len() >= 31,
        "Expected at least 31 topics, got {}",
        topics.len()
    );
}

// ══════════════════════════════════════════════════════════════════════
// validate_symbol tests
// ══════════════════════════════════════════════════════════════════════

#[test]
fn validate_symbol_basic() {
    assert!(validate_symbol("ALPHA").is_ok());
    assert!(validate_symbol("SN1").is_ok());
    assert!(validate_symbol("TAO").is_ok());
    assert!(validate_symbol("X").is_ok());
}

#[test]
fn validate_symbol_empty_rejects() {
    let result = validate_symbol("");
    assert!(result.is_err(), "empty symbol should fail");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("cannot be empty"), "message: {}", msg);
}

#[test]
fn validate_symbol_whitespace_only_rejects() {
    let result = validate_symbol("   ");
    assert!(result.is_err(), "whitespace-only symbol should fail");
}

#[test]
fn validate_symbol_too_long_rejects() {
    let long = "A".repeat(33);
    let result = validate_symbol(&long);
    assert!(result.is_err(), "33-char symbol should fail");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("too long"), "message: {}", msg);
}

#[test]
fn validate_symbol_max_length_ok() {
    let at_limit = "A".repeat(32);
    assert!(validate_symbol(&at_limit).is_ok());
}

#[test]
fn validate_symbol_non_ascii_rejects() {
    let result = validate_symbol("ΑΛΦΑ");  // Greek letters
    assert!(result.is_err(), "non-ASCII should fail");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("non-ASCII"), "message: {}", msg);
}

#[test]
fn validate_symbol_with_spaces() {
    // Leading/trailing spaces: the trim handles it, but space inside is ok
    assert!(validate_symbol("  ALPHA  ").is_ok(), "padded symbol should be ok after trim");
}

// ══════════════════════════════════════════════════════════════════════
// validate_emission_weights tests
// ══════════════════════════════════════════════════════════════════════

#[test]
fn validate_emission_weights_basic() {
    assert!(validate_emission_weights(&[50, 50]).is_ok());
    assert!(validate_emission_weights(&[100]).is_ok());
    assert!(validate_emission_weights(&[33, 33, 34]).is_ok());
}

#[test]
fn validate_emission_weights_empty_rejects() {
    let result = validate_emission_weights(&[]);
    assert!(result.is_err(), "empty weights should fail");
}

#[test]
fn validate_emission_weights_all_zeros_rejects() {
    let result = validate_emission_weights(&[0, 0, 0]);
    assert!(result.is_err(), "all-zero weights should fail");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("total is zero"), "message: {}", msg);
}

#[test]
fn validate_emission_weights_single_zero_rejects() {
    let result = validate_emission_weights(&[0]);
    assert!(result.is_err(), "single zero should fail");
}

#[test]
fn validate_emission_weights_mixed_with_zero_ok() {
    assert!(validate_emission_weights(&[100, 0]).is_ok());
    assert!(validate_emission_weights(&[0, 50, 50]).is_ok());
}

#[test]
fn validate_emission_weights_max_u16() {
    assert!(validate_emission_weights(&[u16::MAX, u16::MAX]).is_ok());
}

// ══════════════════════════════════════════════════════════════════════
// validate_max_cost tests
// ══════════════════════════════════════════════════════════════════════

#[test]
fn validate_max_cost_positive() {
    assert!(validate_max_cost(1.0).is_ok());
    assert!(validate_max_cost(0.01).is_ok());
    assert!(validate_max_cost(1000.0).is_ok());
}

#[test]
fn validate_max_cost_zero_ok() {
    assert!(validate_max_cost(0.0).is_ok());
}

#[test]
fn validate_max_cost_negative_rejects() {
    let result = validate_max_cost(-1.0);
    assert!(result.is_err(), "negative cost should fail");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("cannot be negative"), "message: {}", msg);
}

#[test]
fn validate_max_cost_nan_rejects() {
    let result = validate_max_cost(f64::NAN);
    assert!(result.is_err(), "NaN cost should fail");
}

#[test]
fn validate_max_cost_infinity_rejects() {
    let result = validate_max_cost(f64::INFINITY);
    assert!(result.is_err(), "infinity cost should fail");
}

#[test]
fn validate_max_cost_neg_infinity_rejects() {
    let result = validate_max_cost(f64::NEG_INFINITY);
    assert!(result.is_err(), "negative infinity cost should fail");
}

// ══════════════════════════════════════════════════════════════════════
// validate_delegate_take tests
// ══════════════════════════════════════════════════════════════════════

#[test]
fn validate_delegate_take_valid() {
    assert!(validate_delegate_take(0.0).is_ok());
    assert!(validate_delegate_take(9.0).is_ok());
    assert!(validate_delegate_take(18.0).is_ok());
}

#[test]
fn validate_delegate_take_over_max_rejects() {
    let result = validate_delegate_take(18.01);
    assert!(result.is_err(), "take > 18% should fail");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("Maximum allowed is 18%"), "message: {}", msg);
}

#[test]
fn validate_delegate_take_negative_rejects() {
    let result = validate_delegate_take(-1.0);
    assert!(result.is_err(), "negative take should fail");
}

#[test]
fn validate_delegate_take_nan_rejects() {
    let result = validate_delegate_take(f64::NAN);
    assert!(result.is_err(), "NaN take should fail");
}

#[test]
fn validate_delegate_take_way_over_rejects() {
    let result = validate_delegate_take(100.0);
    assert!(result.is_err(), "100% take should fail");
}

// ══════════════════════════════════════════════════════════════════════
// validate_name tests (wallet/hotkey name validation)
// ══════════════════════════════════════════════════════════════════════

#[test]
fn validate_name_valid_simple() {
    assert!(validate_name("default", "wallet").is_ok());
    assert!(validate_name("my_wallet", "wallet").is_ok());
    assert!(validate_name("wallet-1", "wallet").is_ok());
    assert!(validate_name("Alice", "hotkey").is_ok());
    assert!(validate_name("test123", "wallet").is_ok());
}

#[test]
fn validate_name_valid_boundary() {
    // Max length (64 chars)
    let name = "a".repeat(64);
    assert!(validate_name(&name, "wallet").is_ok());
}

#[test]
fn validate_name_empty_rejects() {
    let result = validate_name("", "wallet");
    assert!(result.is_err(), "empty name should fail");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("cannot be empty"), "msg: {}", msg);
}

#[test]
fn validate_name_whitespace_only_rejects() {
    let result = validate_name("   ", "wallet");
    assert!(result.is_err(), "whitespace-only should fail");
}

#[test]
fn validate_name_too_long_rejects() {
    let name = "a".repeat(65);
    let result = validate_name(&name, "wallet");
    assert!(result.is_err(), "65-char name should fail");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("too long"), "msg: {}", msg);
}

#[test]
fn validate_name_path_traversal_dotdot_rejects() {
    let result = validate_name("../etc/passwd", "wallet");
    assert!(result.is_err(), "path traversal should fail");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("path separators"), "msg: {}", msg);
}

#[test]
fn validate_name_path_traversal_slash_rejects() {
    assert!(validate_name("foo/bar", "wallet").is_err());
    assert!(validate_name("foo\\bar", "wallet").is_err());
}

#[test]
fn validate_name_absolute_path_rejects() {
    assert!(validate_name("/etc/passwd", "wallet").is_err());
    assert!(validate_name("\\\\server\\share", "wallet").is_err());
}

#[test]
fn validate_name_hidden_file_rejects() {
    let result = validate_name(".hidden", "wallet");
    assert!(result.is_err(), "hidden name should fail");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("starts with a dot"), "msg: {}", msg);
}

#[test]
fn validate_name_special_chars_rejects() {
    assert!(validate_name("wallet name", "wallet").is_err(), "spaces should fail");
    assert!(validate_name("wallet@home", "wallet").is_err(), "@ should fail");
    assert!(validate_name("wallet#1", "wallet").is_err(), "# should fail");
    assert!(validate_name("wallet$", "wallet").is_err(), "$ should fail");
    assert!(validate_name("wallet!", "wallet").is_err(), "! should fail");
    assert!(validate_name("wallet&more", "wallet").is_err(), "& should fail");
    assert!(validate_name("wallet;rm -rf /", "wallet").is_err(), "; injection should fail");
}

#[test]
fn validate_name_unicode_rejects() {
    assert!(validate_name("wället", "wallet").is_err(), "umlaut should fail");
    assert!(validate_name("钱包", "wallet").is_err(), "CJK should fail");
    assert!(validate_name("wallet🔑", "wallet").is_err(), "emoji should fail");
}

#[test]
fn validate_name_reserved_windows_rejects() {
    assert!(validate_name("CON", "wallet").is_err(), "CON reserved");
    assert!(validate_name("con", "wallet").is_err(), "con reserved (case-insensitive)");
    assert!(validate_name("PRN", "wallet").is_err(), "PRN reserved");
    assert!(validate_name("NUL", "wallet").is_err(), "NUL reserved");
    assert!(validate_name("COM1", "wallet").is_err(), "COM1 reserved");
    assert!(validate_name("LPT1", "wallet").is_err(), "LPT1 reserved");
    assert!(validate_name("AUX", "wallet").is_err(), "AUX reserved");
}

#[test]
fn validate_name_hyphens_underscores_ok() {
    assert!(validate_name("my-wallet", "wallet").is_ok());
    assert!(validate_name("my_wallet", "wallet").is_ok());
    assert!(validate_name("my-wallet_2", "wallet").is_ok());
    assert!(validate_name("_leading", "wallet").is_ok());
    assert!(validate_name("-leading", "wallet").is_ok());
}

#[test]
fn validate_name_numbers_ok() {
    assert!(validate_name("123", "wallet").is_ok());
    assert!(validate_name("0", "wallet").is_ok());
    assert!(validate_name("wallet99", "wallet").is_ok());
}

#[test]
fn validate_name_label_in_error() {
    let result = validate_name("../bad", "hotkey");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("hotkey"), "error should mention 'hotkey': {}", msg);

    let result = validate_name("../bad", "wallet");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("wallet"), "error should mention 'wallet': {}", msg);
}

// ══════════════════════════════════════════════════════════════════════
// validate_ipv4 tests
// ══════════════════════════════════════════════════════════════════════

#[test]
fn validate_ipv4_valid_public() {
    assert!(validate_ipv4("1.2.3.4").is_ok());
    assert!(validate_ipv4("8.8.8.8").is_ok());
    assert!(validate_ipv4("203.0.113.1").is_ok());
    assert!(validate_ipv4("100.64.0.1").is_ok());
}

#[test]
fn validate_ipv4_returns_correct_u128() {
    let result = validate_ipv4("1.2.3.4").unwrap();
    let expected: u128 = (1 << 24) | (2 << 16) | (3 << 8) | 4;
    assert_eq!(result, expected);
}

#[test]
fn validate_ipv4_max_octets() {
    let result = validate_ipv4("254.254.254.254").unwrap();
    assert!(result > 0);
}

#[test]
fn validate_ipv4_rejects_zeroes() {
    let result = validate_ipv4("0.0.0.0");
    assert!(result.is_err(), "all-zeros should fail");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("unspecified"), "msg: {}", msg);
}

#[test]
fn validate_ipv4_rejects_broadcast() {
    let result = validate_ipv4("255.255.255.255");
    assert!(result.is_err(), "broadcast should fail");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("broadcast"), "msg: {}", msg);
}

#[test]
fn validate_ipv4_rejects_loopback() {
    let result = validate_ipv4("127.0.0.1");
    assert!(result.is_err(), "loopback should fail");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("loopback"), "msg: {}", msg);
}

#[test]
fn validate_ipv4_rejects_loopback_range() {
    assert!(validate_ipv4("127.0.0.1").is_err());
    assert!(validate_ipv4("127.255.255.254").is_err());
    assert!(validate_ipv4("127.1.2.3").is_err());
}

#[test]
fn validate_ipv4_warns_private_but_allows() {
    // Private ranges should succeed (just warn to stderr)
    assert!(validate_ipv4("10.0.0.1").is_ok());
    assert!(validate_ipv4("10.255.255.255").is_ok());
    assert!(validate_ipv4("172.16.0.1").is_ok());
    assert!(validate_ipv4("172.31.255.255").is_ok());
    assert!(validate_ipv4("192.168.0.1").is_ok());
    assert!(validate_ipv4("192.168.255.255").is_ok());
}

#[test]
fn validate_ipv4_rejects_too_few_octets() {
    assert!(validate_ipv4("1.2.3").is_err());
    assert!(validate_ipv4("1.2").is_err());
    assert!(validate_ipv4("1").is_err());
}

#[test]
fn validate_ipv4_rejects_too_many_octets() {
    assert!(validate_ipv4("1.2.3.4.5").is_err());
}

#[test]
fn validate_ipv4_rejects_octet_overflow() {
    assert!(validate_ipv4("256.0.0.1").is_err());
    assert!(validate_ipv4("1.2.3.999").is_err());
}

#[test]
fn validate_ipv4_rejects_non_numeric() {
    assert!(validate_ipv4("abc.def.ghi.jkl").is_err());
    assert!(validate_ipv4("1.2.3.x").is_err());
}

#[test]
fn validate_ipv4_rejects_empty() {
    assert!(validate_ipv4("").is_err());
}

#[test]
fn validate_ipv4_rejects_leading_zeros() {
    let result = validate_ipv4("01.02.03.04");
    assert!(result.is_err(), "leading zeros should fail");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("leading zeros"), "msg: {}", msg);
}

#[test]
fn validate_ipv4_rejects_negative() {
    assert!(validate_ipv4("-1.0.0.1").is_err());
}

#[test]
fn validate_ipv4_rejects_spaces() {
    assert!(validate_ipv4("1.2.3. 4").is_err());
    assert!(validate_ipv4(" 1.2.3.4").is_err());
}

#[test]
fn validate_ipv4_rejects_hostname() {
    assert!(validate_ipv4("example.com").is_err());
    assert!(validate_ipv4("localhost").is_err());
}

// ── validate_ss58 ──

use agcli::cli::helpers::validate_ss58;

#[test]
fn validate_ss58_valid_alice() {
    assert!(validate_ss58("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", "test").is_ok());
}

#[test]
fn validate_ss58_valid_bob() {
    assert!(validate_ss58("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty", "test").is_ok());
}

#[test]
fn validate_ss58_empty_rejects() {
    let err = validate_ss58("", "destination").unwrap_err().to_string();
    assert!(err.contains("empty"), "msg: {}", err);
    assert!(err.contains("destination"), "should include label: {}", err);
}

#[test]
fn validate_ss58_whitespace_only_rejects() {
    assert!(validate_ss58("   ", "dest").is_err());
}

#[test]
fn validate_ss58_too_short_rejects() {
    let err = validate_ss58("5Grw", "hotkey").unwrap_err().to_string();
    assert!(err.contains("too short"), "msg: {}", err);
}

#[test]
fn validate_ss58_too_long_rejects() {
    let long = "5".to_string() + &"a".repeat(60);
    let err = validate_ss58(&long, "test").unwrap_err().to_string();
    assert!(err.contains("too long"), "msg: {}", err);
}

#[test]
fn validate_ss58_ethereum_address_rejects() {
    let err = validate_ss58("0x742d35Cc6634C0532925a3b844Bc9e7595f2bD18", "destination").unwrap_err().to_string();
    assert!(err.contains("Ethereum") || err.contains("hex"), "should detect 0x prefix: {}", err);
}

#[test]
fn validate_ss58_uppercase_0x_rejects() {
    let err = validate_ss58("0X742d35Cc6634C0532925a3b844Bc9e7595f2bD18", "test").unwrap_err().to_string();
    assert!(err.contains("Ethereum") || err.contains("hex"), "msg: {}", err);
}

#[test]
fn validate_ss58_contains_spaces_rejects() {
    let err = validate_ss58("5Grwva EF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", "test").unwrap_err().to_string();
    assert!(err.contains("whitespace"), "msg: {}", err);
}

#[test]
fn validate_ss58_tabs_rejects() {
    assert!(validate_ss58("5Grwva\tEF5z", "test").is_err());
}

#[test]
fn validate_ss58_invalid_base58_chars_rejects() {
    // 'O' is not in Base58 (0, I, O, l are excluded)
    let err = validate_ss58("5GrwvaOF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", "test").unwrap_err().to_string();
    assert!(err.contains("Base58") || err.contains("'O'"), "msg: {}", err);
}

#[test]
fn validate_ss58_zero_char_rejects() {
    // '0' is not valid Base58
    let err = validate_ss58("50rwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", "test").unwrap_err().to_string();
    assert!(err.contains("Base58") || err.contains("'0'"), "msg: {}", err);
}

#[test]
fn validate_ss58_lowercase_l_rejects() {
    // 'l' is not valid Base58
    let err = validate_ss58("5GrwvalF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", "test").unwrap_err().to_string();
    assert!(err.contains("Base58") || err.contains("'l'"), "msg: {}", err);
}

#[test]
fn validate_ss58_bad_checksum_rejects() {
    // Change last char to invalidate checksum
    let err = validate_ss58("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQZ", "test").unwrap_err().to_string();
    assert!(err.contains("checksum"), "msg: {}", err);
}

#[test]
fn validate_ss58_random_string_rejects() {
    assert!(validate_ss58("notanaddressatall12345678901234567890123456", "test").is_err());
}

#[test]
fn validate_ss58_label_in_error() {
    let err = validate_ss58("", "my-delegate").unwrap_err().to_string();
    assert!(err.contains("my-delegate"), "error should include label: {}", err);
}

#[test]
fn validate_ss58_leading_trailing_whitespace_trimmed() {
    // Trimmed version of Alice should be valid
    assert!(validate_ss58(" 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY ", "test").is_ok());
}

// ── validate_password_strength ──

use agcli::cli::helpers::validate_password_strength;

#[test]
fn validate_password_strength_strong_no_panic() {
    // Should not panic — function only prints warnings
    validate_password_strength("Str0ng!Pass#2024");
}

#[test]
fn validate_password_strength_short_no_panic() {
    validate_password_strength("ab");
}

#[test]
fn validate_password_strength_empty_no_panic() {
    validate_password_strength("");
}

#[test]
fn validate_password_strength_common_no_panic() {
    validate_password_strength("password");
    validate_password_strength("12345678");
    validate_password_strength("qwerty");
}

#[test]
fn validate_password_strength_single_type_no_panic() {
    validate_password_strength("abcdefgh");
    validate_password_strength("12345678");
    validate_password_strength("ABCDEFGH");
}

#[test]
fn validate_password_strength_mixed_no_panic() {
    validate_password_strength("aB1!");
}

// ── validate_port ──

use agcli::cli::helpers::validate_port;

#[test]
fn validate_port_normal_ok() {
    assert!(validate_port(8091, "axon").is_ok());
    assert!(validate_port(443, "https").is_ok());
    assert!(validate_port(65535, "max").is_ok());
    assert!(validate_port(1024, "user").is_ok());
}

#[test]
fn validate_port_zero_rejects() {
    let err = validate_port(0, "axon").unwrap_err().to_string();
    assert!(err.contains("0"), "msg: {}", err);
    assert!(err.contains("axon"), "should include label: {}", err);
}

#[test]
fn validate_port_privileged_warns_but_ok() {
    // Ports < 1024 should succeed but print a warning
    assert!(validate_port(80, "http").is_ok());
    assert!(validate_port(1, "min").is_ok());
    assert!(validate_port(22, "ssh").is_ok());
}

#[test]
fn validate_port_one_ok() {
    assert!(validate_port(1, "test").is_ok());
}

// ── validate_netuid ──

use agcli::cli::helpers::validate_netuid;

#[test]
fn validate_netuid_normal_ok() {
    assert!(validate_netuid(1).is_ok());
    assert!(validate_netuid(100).is_ok());
    assert!(validate_netuid(65535).is_ok());
}

#[test]
fn validate_netuid_zero_rejects() {
    let err = validate_netuid(0).unwrap_err().to_string();
    assert!(err.contains("0") || err.contains("Root"), "msg: {}", err);
}

// ── validate_batch_axon_json ──

use agcli::cli::helpers::validate_batch_axon_json;

#[test]
fn validate_batch_axon_json_valid_single() {
    let json = r#"[{"netuid": 1, "ip": "1.2.3.4", "port": 8091}]"#;
    let entries = validate_batch_axon_json(json).unwrap();
    assert_eq!(entries.len(), 1);
}

#[test]
fn validate_batch_axon_json_valid_multiple() {
    let json = r#"[
        {"netuid": 1, "ip": "1.2.3.4", "port": 8091},
        {"netuid": 2, "ip": "5.6.7.8", "port": 9092, "protocol": 4, "version": 1}
    ]"#;
    let entries = validate_batch_axon_json(json).unwrap();
    assert_eq!(entries.len(), 2);
}

#[test]
fn validate_batch_axon_json_valid_with_all_fields() {
    let json = r#"[{"netuid": 1, "ip": "1.2.3.4", "port": 8091, "protocol": 6, "version": 42}]"#;
    assert!(validate_batch_axon_json(json).is_ok());
}

#[test]
fn validate_batch_axon_json_empty_array_rejects() {
    let err = validate_batch_axon_json("[]").unwrap_err().to_string();
    assert!(err.contains("empty"), "msg: {}", err);
}

#[test]
fn validate_batch_axon_json_not_array_rejects() {
    assert!(validate_batch_axon_json(r#"{"netuid": 1}"#).is_err());
}

#[test]
fn validate_batch_axon_json_invalid_json_rejects() {
    let err = validate_batch_axon_json("not json").unwrap_err().to_string();
    assert!(err.contains("Invalid"), "msg: {}", err);
}

#[test]
fn validate_batch_axon_json_missing_netuid_rejects() {
    let err = validate_batch_axon_json(r#"[{"ip": "1.2.3.4", "port": 8091}]"#)
        .unwrap_err().to_string();
    assert!(err.contains("netuid"), "msg: {}", err);
}

#[test]
fn validate_batch_axon_json_missing_ip_rejects() {
    let err = validate_batch_axon_json(r#"[{"netuid": 1, "port": 8091}]"#)
        .unwrap_err().to_string();
    assert!(err.contains("ip"), "msg: {}", err);
}

#[test]
fn validate_batch_axon_json_missing_port_rejects() {
    let err = validate_batch_axon_json(r#"[{"netuid": 1, "ip": "1.2.3.4"}]"#)
        .unwrap_err().to_string();
    assert!(err.contains("port"), "msg: {}", err);
}

#[test]
fn validate_batch_axon_json_netuid_not_number_rejects() {
    let err = validate_batch_axon_json(r#"[{"netuid": "one", "ip": "1.2.3.4", "port": 8091}]"#)
        .unwrap_err().to_string();
    assert!(err.contains("netuid"), "msg: {}", err);
}

#[test]
fn validate_batch_axon_json_ip_not_string_rejects() {
    let err = validate_batch_axon_json(r#"[{"netuid": 1, "ip": 123, "port": 8091}]"#)
        .unwrap_err().to_string();
    assert!(err.contains("ip"), "msg: {}", err);
}

#[test]
fn validate_batch_axon_json_port_zero_rejects() {
    let err = validate_batch_axon_json(r#"[{"netuid": 1, "ip": "1.2.3.4", "port": 0}]"#)
        .unwrap_err().to_string();
    assert!(err.contains("port"), "msg: {}", err);
}

#[test]
fn validate_batch_axon_json_port_too_large_rejects() {
    let err = validate_batch_axon_json(r#"[{"netuid": 1, "ip": "1.2.3.4", "port": 70000}]"#)
        .unwrap_err().to_string();
    assert!(err.contains("port"), "msg: {}", err);
}

#[test]
fn validate_batch_axon_json_netuid_too_large_rejects() {
    let err = validate_batch_axon_json(r#"[{"netuid": 100000, "ip": "1.2.3.4", "port": 8091}]"#)
        .unwrap_err().to_string();
    assert!(err.contains("netuid"), "msg: {}", err);
}

#[test]
fn validate_batch_axon_json_protocol_overflow_rejects() {
    let err = validate_batch_axon_json(r#"[{"netuid": 1, "ip": "1.2.3.4", "port": 8091, "protocol": 256}]"#)
        .unwrap_err().to_string();
    assert!(err.contains("protocol"), "msg: {}", err);
}

#[test]
fn validate_batch_axon_json_invalid_ip_rejects() {
    let err = validate_batch_axon_json(r#"[{"netuid": 1, "ip": "127.0.0.1", "port": 8091}]"#)
        .unwrap_err().to_string();
    assert!(err.contains("loopback") || err.contains("IP") || err.contains("ip"), "msg: {}", err);
}

#[test]
fn validate_batch_axon_json_entry_not_object_rejects() {
    let err = validate_batch_axon_json(r#"[42]"#).unwrap_err().to_string();
    assert!(err.contains("not a JSON object"), "msg: {}", err);
}

#[test]
fn validate_batch_axon_json_string_entry_rejects() {
    let err = validate_batch_axon_json(r#"["hello"]"#).unwrap_err().to_string();
    assert!(err.contains("not a JSON object"), "msg: {}", err);
}

// ──── validate_mnemonic tests ────

#[test]
fn validate_mnemonic_valid_12_words() {
    // Generate a valid 12-word mnemonic using bip39 crate
    let mnemonic = bip39::Mnemonic::from_entropy_in(bip39::Language::English, &[0u8; 16]).unwrap();
    assert!(validate_mnemonic(&mnemonic.to_string()).is_ok());
}

#[test]
fn validate_mnemonic_valid_24_words() {
    let mnemonic = bip39::Mnemonic::from_entropy_in(bip39::Language::English, &[0u8; 32]).unwrap();
    let phrase = mnemonic.to_string();
    let words: Vec<&str> = phrase.split_whitespace().collect();
    assert_eq!(words.len(), 24);
    assert!(validate_mnemonic(&phrase).is_ok());
}

#[test]
fn validate_mnemonic_valid_15_words() {
    let mnemonic = bip39::Mnemonic::from_entropy_in(bip39::Language::English, &[0u8; 20]).unwrap();
    let phrase = mnemonic.to_string();
    let words: Vec<&str> = phrase.split_whitespace().collect();
    assert_eq!(words.len(), 15);
    assert!(validate_mnemonic(&phrase).is_ok());
}

#[test]
fn validate_mnemonic_empty_rejects() {
    let err = validate_mnemonic("").unwrap_err().to_string();
    assert!(err.contains("empty"), "msg: {}", err);
}

#[test]
fn validate_mnemonic_whitespace_only_rejects() {
    let err = validate_mnemonic("   \t  ").unwrap_err().to_string();
    assert!(err.contains("empty"), "msg: {}", err);
}

#[test]
fn validate_mnemonic_wrong_word_count_rejects() {
    let err = validate_mnemonic("abandon abandon abandon").unwrap_err().to_string();
    assert!(err.contains("3 words"), "msg: {}", err);
    assert!(err.contains("12, 15, 18, 21, or 24"), "msg: {}", err);
}

#[test]
fn validate_mnemonic_11_words_rejects() {
    let err = validate_mnemonic("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon").unwrap_err().to_string();
    assert!(err.contains("11 words"), "msg: {}", err);
}

#[test]
fn validate_mnemonic_13_words_rejects() {
    let err = validate_mnemonic("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about").unwrap_err().to_string();
    assert!(err.contains("13 words"), "msg: {}", err);
}

#[test]
fn validate_mnemonic_invalid_word_rejects() {
    // 12 words but one is not BIP-39
    let err = validate_mnemonic("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon xylophone").unwrap_err().to_string();
    assert!(err.contains("xylophone"), "should mention bad word: {}", err);
    assert!(err.contains("BIP-39"), "should mention BIP-39: {}", err);
}

#[test]
fn validate_mnemonic_bad_checksum_rejects() {
    // 12 valid BIP-39 words but wrong checksum
    let err = validate_mnemonic("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon").unwrap_err().to_string();
    assert!(err.contains("checksum"), "should mention checksum: {}", err);
}

#[test]
fn validate_mnemonic_extra_spaces_ok() {
    // Valid mnemonic with extra whitespace should be accepted
    let mnemonic = bip39::Mnemonic::from_entropy_in(bip39::Language::English, &[0u8; 16]).unwrap();
    let phrase = mnemonic.to_string();
    let with_spaces = format!("  {}  ", phrase.replace(' ', "  "));
    assert!(validate_mnemonic(&with_spaces).is_ok(), "extra whitespace should be tolerated");
}

#[test]
fn validate_mnemonic_misspelled_word_suggests() {
    // "abandn" is close to "abandon"
    let err = validate_mnemonic("abandn abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about").unwrap_err().to_string();
    assert!(err.contains("abandn"), "should mention misspelled word: {}", err);
    assert!(err.contains("BIP-39"), "should mention BIP-39: {}", err);
}

#[test]
fn validate_mnemonic_numbers_reject() {
    let err = validate_mnemonic("1 2 3 4 5 6 7 8 9 10 11 12").unwrap_err().to_string();
    assert!(err.contains("BIP-39") || err.contains("not in"), "msg: {}", err);
}

#[test]
fn validate_mnemonic_single_word_rejects() {
    let err = validate_mnemonic("abandon").unwrap_err().to_string();
    assert!(err.contains("1 words") || err.contains("1 word"), "msg: {}", err);
}

#[test]
fn validate_mnemonic_passphrase_not_mnemonic() {
    // Common mistake: entering a password instead of mnemonic
    let err = validate_mnemonic("MySecretPassword123!").unwrap_err().to_string();
    assert!(err.contains("1 word") || err.contains("expected"), "msg: {}", err);
}

#[test]
fn validate_mnemonic_25_words_rejects() {
    // More than 24 words
    let mnemonic24 = bip39::Mnemonic::from_entropy_in(bip39::Language::English, &[0u8; 32]).unwrap();
    let phrase = format!("{} abandon", mnemonic24);
    let err = validate_mnemonic(&phrase).unwrap_err().to_string();
    assert!(err.contains("25 words"), "msg: {}", err);
}

// ──── Error message quality tests ────
// Verify that all user-facing errors contain actionable tips

#[test]
fn error_quality_validate_amount_zero_has_tip() {
    let err = validate_amount(0.0, "stake").unwrap_err().to_string();
    assert!(err.contains("Tip:"), "zero amount error should have tip: {}", err);
    assert!(err.contains("RAO"), "should mention RAO minimum: {}", err);
}

#[test]
fn error_quality_validate_name_empty_has_tip() {
    let err = validate_name("", "wallet").unwrap_err().to_string();
    assert!(err.contains("Tip:"), "empty name error should have tip: {}", err);
    assert!(err.contains("alphanumeric") || err.contains("mywallet"), "should suggest valid name: {}", err);
}

#[test]
fn error_quality_validate_name_path_traversal_has_tip() {
    let err = validate_name("../../../etc/passwd", "wallet").unwrap_err().to_string();
    assert!(err.contains("Tip:"), "path traversal error should have tip: {}", err);
}

#[test]
fn error_quality_validate_ss58_empty_has_tip() {
    let err = agcli::cli::helpers::validate_ss58("", "destination").unwrap_err().to_string();
    assert!(err.contains("Tip:"), "empty SS58 error should have tip: {}", err);
    assert!(err.contains("48"), "should mention address length: {}", err);
}

#[test]
fn error_quality_validate_ss58_ethereum_has_tip() {
    let err = agcli::cli::helpers::validate_ss58("0x742d35Cc6634C0532925a3b844BcEfe0390a94e0", "destination").unwrap_err().to_string();
    assert!(err.contains("Tip:"), "Ethereum address error should have tip: {}", err);
    assert!(err.contains("Ethereum") || err.contains("SS58"), "should explain format: {}", err);
}

#[test]
fn error_quality_validate_ipv4_loopback_has_tip() {
    let err = validate_ipv4("127.0.0.1").unwrap_err().to_string();
    assert!(err.contains("public"), "loopback error should suggest public IP: {}", err);
}

#[test]
fn error_quality_validate_take_over_max_has_tip() {
    let err = validate_take_pct(20.0).unwrap_err().to_string();
    assert!(err.contains("Tip:"), "over-max take error should have tip: {}", err);
    assert!(err.contains("18"), "should mention maximum: {}", err);
}

#[test]
fn error_quality_validate_port_zero_has_tip() {
    let err = agcli::cli::helpers::validate_port(0, "axon").unwrap_err().to_string();
    assert!(err.contains("Tip:"), "port zero error should have tip: {}", err);
    assert!(err.contains("8091") || err.contains("443"), "should suggest common ports: {}", err);
}

#[test]
fn error_quality_validate_netuid_zero_has_tip() {
    let err = agcli::cli::helpers::validate_netuid(0).unwrap_err().to_string();
    assert!(err.contains("Tip:"), "netuid zero error should have tip: {}", err);
    assert!(err.contains("netuid 1"), "should mention subnets start at 1: {}", err);
}

#[test]
fn error_quality_validate_symbol_empty_has_tip() {
    let err = validate_symbol("").unwrap_err().to_string();
    assert!(err.contains("Tip:"), "empty symbol error should have tip: {}", err);
    assert!(err.contains("ALPHA") || err.contains("SN1"), "should suggest example: {}", err);
}

#[test]
fn error_quality_validate_mnemonic_wrong_count_has_tip() {
    let err = validate_mnemonic("abandon abandon abandon").unwrap_err().to_string();
    assert!(err.contains("Tip:"), "wrong word count error should have tip: {}", err);
}

#[test]
fn error_quality_validate_mnemonic_bad_word_has_tip() {
    let err = validate_mnemonic("xylophone abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about").unwrap_err().to_string();
    assert!(err.contains("BIP-39") || err.contains("dictionary"), "bad word error should mention BIP-39: {}", err);
}

#[test]
fn error_quality_parse_weight_invalid_has_format() {
    let err = parse_weight_pairs("bad").unwrap_err().to_string();
    assert!(err.contains("uid:weight") || err.contains("Format:"), "should show format: {}", err);
}

#[test]
fn error_quality_parse_children_invalid_has_format() {
    let err = parse_children("bad").unwrap_err().to_string();
    assert!(err.contains("proportion:hotkey") || err.contains("Format:"), "should show format: {}", err);
}

#[test]
fn error_quality_validate_max_cost_negative_mentions_value() {
    let err = validate_max_cost(-5.0).unwrap_err().to_string();
    assert!(err.contains("-5"), "should show the invalid value: {}", err);
    assert!(err.contains("negative"), "should explain why invalid: {}", err);
}

#[test]
fn error_quality_validate_delegate_take_over_max_has_tip() {
    let err = validate_delegate_take(25.0).unwrap_err().to_string();
    assert!(err.contains("Tip:"), "over-max delegate take should have tip: {}", err);
    assert!(err.contains("18"), "should mention maximum: {}", err);
}

// ──── Dry-run related parse tests ────
// These verify --dry-run flag is parseable across different command positions

#[test]
fn dry_run_flag_parses_with_transfer() {
    use clap::Parser;
    let args = vec!["agcli", "--dry-run", "transfer", "--dest", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", "--amount", "1.0"];
    let cli = agcli::cli::Cli::try_parse_from(args);
    assert!(cli.is_ok(), "--dry-run before subcommand: {:?}", cli.err());
    assert!(cli.unwrap().dry_run);
}

#[test]
fn dry_run_flag_parses_after_subcommand() {
    use clap::Parser;
    let args = vec!["agcli", "transfer", "--dry-run", "--dest", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", "--amount", "1.0"];
    let cli = agcli::cli::Cli::try_parse_from(args);
    assert!(cli.is_ok(), "--dry-run after subcommand: {:?}", cli.err());
    assert!(cli.unwrap().dry_run);
}

#[test]
fn dry_run_flag_absent_means_false() {
    use clap::Parser;
    // When --dry-run is not passed and no env var, should be false
    // Note: clean env for this test
    let saved = std::env::var("AGCLI_DRY_RUN").ok();
    std::env::remove_var("AGCLI_DRY_RUN");
    let args = vec!["agcli", "balance"];
    let cli = agcli::cli::Cli::try_parse_from(args).unwrap();
    assert!(!cli.dry_run, "dry_run should be false when flag absent");
    // Restore
    if let Some(v) = saved {
        std::env::set_var("AGCLI_DRY_RUN", v);
    }
}

#[test]
fn dry_run_with_stake_add() {
    use clap::Parser;
    let args = vec!["agcli", "--dry-run", "stake", "add", "--netuid", "1", "--amount", "1.0"];
    let cli = agcli::cli::Cli::try_parse_from(args);
    assert!(cli.is_ok(), "--dry-run with stake add: {:?}", cli.err());
    assert!(cli.unwrap().dry_run);
}

#[test]
fn dry_run_with_subnet_register() {
    use clap::Parser;
    // subnet register takes no args — it creates a new network
    let args = vec!["agcli", "--dry-run", "subnet", "register"];
    let cli = agcli::cli::Cli::try_parse_from(args);
    assert!(cli.is_ok(), "--dry-run with subnet register: {:?}", cli.err());
    assert!(cli.unwrap().dry_run);
}

#[test]
fn dry_run_with_weights_set() {
    use clap::Parser;
    let args = vec!["agcli", "--dry-run", "weights", "set", "--netuid", "1", "--weights", "0:100,1:200"];
    let cli = agcli::cli::Cli::try_parse_from(args);
    assert!(cli.is_ok(), "--dry-run with weights set: {:?}", cli.err());
    assert!(cli.unwrap().dry_run);
}

#[test]
fn dry_run_with_delegate_increase() {
    use clap::Parser;
    let args = vec!["agcli", "--dry-run", "delegate", "increase-take", "--take", "10.0"];
    let cli = agcli::cli::Cli::try_parse_from(args);
    assert!(cli.is_ok(), "--dry-run with delegate increase-take: {:?}", cli.err());
    assert!(cli.unwrap().dry_run);
}

#[test]
fn dry_run_with_proxy_add() {
    use clap::Parser;
    let args = vec!["agcli", "--dry-run", "proxy", "add", "--delegate", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"];
    let cli = agcli::cli::Cli::try_parse_from(args);
    assert!(cli.is_ok(), "--dry-run with proxy add: {:?}", cli.err());
    assert!(cli.unwrap().dry_run);
}

#[test]
fn dry_run_with_serve_axon() {
    use clap::Parser;
    let args = vec!["agcli", "--dry-run", "serve", "axon",
        "--netuid", "1", "--ip", "1.2.3.4", "--port", "8091"];
    let cli = agcli::cli::Cli::try_parse_from(args);
    assert!(cli.is_ok(), "--dry-run with serve axon: {:?}", cli.err());
    assert!(cli.unwrap().dry_run);
}

// ──── parse_children enhanced edge cases ────

#[test]
fn parse_children_trailing_comma_ok() {
    let alice = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
    let result = parse_children(&format!("1000:{},", alice));
    assert!(result.is_ok(), "trailing comma should be tolerated: {:?}", result.err());
    assert_eq!(result.unwrap().len(), 1);
}

#[test]
// ──── validate_derive_input tests ────

#[test]
fn validate_derive_input_valid_hex_32_bytes() {
    let hex = "0x0000000000000000000000000000000000000000000000000000000000000001";
    assert!(validate_derive_input(hex).is_ok());
}

#[test]
fn validate_derive_input_valid_mnemonic() {
    let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    assert!(validate_derive_input(mnemonic).is_ok());
}

#[test]
fn validate_derive_input_empty() {
    let err = validate_derive_input("").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("cannot be empty"), "got: {}", msg);
    assert!(msg.contains("Tip:"), "should have tip: {}", msg);
}

#[test]
fn validate_derive_input_whitespace_only() {
    let err = validate_derive_input("   ").unwrap_err();
    assert!(err.to_string().contains("cannot be empty"));
}

#[test]
fn validate_derive_input_hex_empty_after_prefix() {
    let err = validate_derive_input("0x").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("empty after"), "got: {}", msg);
}

#[test]
fn validate_derive_input_hex_odd_length() {
    let err = validate_derive_input("0x012345678901234567890123456789012345678901234567890123456789012").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("odd length"), "got: {}", msg);
}

#[test]
fn validate_derive_input_hex_too_short() {
    let err = validate_derive_input("0x0123456789abcdef").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("32 bytes"), "got: {}", msg);
}

#[test]
fn validate_derive_input_hex_too_long() {
    let err = validate_derive_input("0x00000000000000000000000000000000000000000000000000000000000000000000").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("32 bytes"), "got: {}", msg);
}

#[test]
fn validate_derive_input_hex_invalid_chars() {
    let err = validate_derive_input("0xGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGG").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("Invalid hex character"), "got: {}", msg);
}

#[test]
fn validate_derive_input_hex_uppercase_0X() {
    // 0X prefix should also be recognized as hex
    let err = validate_derive_input("0X0123").unwrap_err();
    // Should treat as hex path and reject for wrong length
    let msg = err.to_string();
    assert!(msg.contains("odd length") || msg.contains("32 bytes"), "got: {}", msg);
}

#[test]
fn validate_derive_input_hex_with_spaces() {
    // Hex with trailing spaces — trimmed first
    let hex = "0x0000000000000000000000000000000000000000000000000000000000000001  ";
    assert!(validate_derive_input(hex).is_ok(), "trailing spaces should be trimmed");
}

#[test]
fn validate_derive_input_invalid_mnemonic() {
    // Something that's not hex but also not a valid mnemonic
    let err = validate_derive_input("not a valid mnemonic at all").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("mnemonic") || msg.contains("word"), "got: {}", msg);
}

// ──── validate_multisig_json_args tests ────

#[test]
fn validate_multisig_json_args_valid_simple() {
    let result = validate_multisig_json_args(r#"[1, "hello", true]"#);
    assert!(result.is_ok(), "simple array: {:?}", result.err());
    assert_eq!(result.unwrap().len(), 3);
}

#[test]
fn validate_multisig_json_args_valid_hex_bytes() {
    let result = validate_multisig_json_args(r#"["0xdeadbeef", 42]"#);
    assert!(result.is_ok());
}

#[test]
fn validate_multisig_json_args_valid_nested_object() {
    let result = validate_multisig_json_args(r#"[{"Id": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"}, 1000]"#);
    assert!(result.is_ok());
}

#[test]
fn validate_multisig_json_args_empty_string() {
    let err = validate_multisig_json_args("").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("Empty JSON"), "got: {}", msg);
}

#[test]
fn validate_multisig_json_args_not_json() {
    let err = validate_multisig_json_args("not json at all").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("Invalid JSON"), "got: {}", msg);
}

#[test]
fn validate_multisig_json_args_json_object_not_array() {
    let err = validate_multisig_json_args(r#"{"key": "value"}"#).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("Expected a JSON array"), "got: {}", msg);
    assert!(msg.contains("object"), "should say it got an object: {}", msg);
}

#[test]
fn validate_multisig_json_args_json_string_not_array() {
    let err = validate_multisig_json_args(r#""just a string""#).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("Expected a JSON array"), "got: {}", msg);
}

#[test]
fn validate_multisig_json_args_json_number_not_array() {
    let err = validate_multisig_json_args("42").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("Expected a JSON array"), "got: {}", msg);
}

#[test]
fn validate_multisig_json_args_null_element() {
    let err = validate_multisig_json_args("[1, null, 3]").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("null"), "got: {}", msg);
    assert!(msg.contains("index 1"), "should identify the index: {}", msg);
}

#[test]
fn validate_multisig_json_args_deeply_nested() {
    let deep = r#"[[[[[["too deep"]]]]]]"#;
    let err = validate_multisig_json_args(deep).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("nesting too deep"), "got: {}", msg);
}

#[test]
fn validate_multisig_json_args_long_string() {
    let long_str = format!(r#"["{}"]"#, "a".repeat(2000));
    let err = validate_multisig_json_args(&long_str).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("too long"), "got: {}", msg);
}

#[test]
fn validate_multisig_json_args_empty_array() {
    let result = validate_multisig_json_args("[]");
    assert!(result.is_ok(), "empty array should be valid: {:?}", result.err());
    assert_eq!(result.unwrap().len(), 0);
}

#[test]
fn validate_multisig_json_args_whitespace_around() {
    let result = validate_multisig_json_args("  [1, 2]  ");
    assert!(result.is_ok(), "whitespace should be trimmed: {:?}", result.err());
}

#[test]
fn validate_multisig_json_args_nested_null() {
    let err = validate_multisig_json_args(r#"[{"key": null}]"#).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("null"), "nested null: {}", msg);
}

#[test]
fn validate_multisig_json_args_valid_bool_and_negative() {
    let result = validate_multisig_json_args(r#"[true, false, -1, 0]"#);
    assert!(result.is_ok());
}

#[test]
fn validate_multisig_json_args_nested_array_ok() {
    // 3 levels deep — should be fine (limit is 4)
    let result = validate_multisig_json_args(r#"[[[1, 2]], "ok"]"#);
    assert!(result.is_ok(), "3 levels of nesting: {:?}", result.err());
}

// ──── json_to_subxt_value tests ────

#[test]
fn json_to_subxt_value_large_u64() {
    let v = serde_json::json!(u64::MAX);
    let result = json_to_subxt_value(&v);
    let _ = result;
}

#[test]
fn json_to_subxt_value_negative_i64() {
    let v = serde_json::json!(i64::MIN);
    let result = json_to_subxt_value(&v);
    let _ = result;
}

#[test]
fn json_to_subxt_value_hex_string_short() {
    let v = serde_json::json!("0xab");
    let result = json_to_subxt_value(&v);
    let _ = result;
}

#[test]
fn json_to_subxt_value_invalid_hex_string_fallback() {
    // "0xZZ" should fail hex decode and fall back to string
    let v = serde_json::json!("0xZZ");
    let result = json_to_subxt_value(&v);
    let _ = result;
}

#[test]
fn json_to_subxt_value_bool_false() {
    let v = serde_json::json!(false);
    let result = json_to_subxt_value(&v);
    let _ = result;
}

#[test]
fn json_to_subxt_value_nested_array() {
    let v = serde_json::json!([[1, 2], [3, 4]]);
    let result = json_to_subxt_value(&v);
    let _ = result;
}

#[test]
fn json_to_subxt_value_null_to_string() {
    let v = serde_json::json!(null);
    let result = json_to_subxt_value(&v);
    let _ = result; // null maps to string "null"
}

#[test]
fn json_to_subxt_value_object_to_string() {
    let v = serde_json::json!({"key": "val"});
    let result = json_to_subxt_value(&v);
    let _ = result; // object falls through to string
}

#[test]
fn parse_children_duplicate_hotkeys_allowed() {
    let alice = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
    let result = parse_children(&format!("500:{},500:{}", alice, alice));
    // Duplicate hotkeys should still parse — the chain will reject if needed
    assert!(result.is_ok(), "duplicate hotkeys: {:?}", result.err());
    assert_eq!(result.unwrap().len(), 2);
}

// =====================================================================
// validate_evm_address tests
// =====================================================================

#[test]
fn evm_address_valid_with_0x_prefix() {
    assert!(validate_evm_address("0x1234567890abcdef1234567890abcdef12345678", "test").is_ok());
}

#[test]
fn evm_address_valid_without_0x_prefix() {
    assert!(validate_evm_address("1234567890abcdef1234567890abcdef12345678", "test").is_ok());
}

#[test]
fn evm_address_valid_uppercase() {
    assert!(validate_evm_address("0x1234567890ABCDEF1234567890ABCDEF12345678", "test").is_ok());
}

#[test]
fn evm_address_valid_mixed_case() {
    assert!(validate_evm_address("0xABCDef1234567890abcdef1234567890AbCdEf12", "test").is_ok());
}

#[test]
fn evm_address_valid_all_zeros() {
    assert!(validate_evm_address("0x0000000000000000000000000000000000000000", "test").is_ok());
}

#[test]
fn evm_address_valid_all_f() {
    assert!(validate_evm_address("0xffffffffffffffffffffffffffffffffffffffff", "test").is_ok());
}

#[test]
fn evm_address_empty() {
    let err = validate_evm_address("", "source").unwrap_err();
    assert!(err.to_string().contains("cannot be empty"), "got: {}", err);
}

#[test]
fn evm_address_just_0x() {
    let err = validate_evm_address("0x", "source").unwrap_err();
    assert!(err.to_string().contains("empty after '0x'"), "got: {}", err);
}

#[test]
fn evm_address_too_short() {
    let err = validate_evm_address("0x1234", "target").unwrap_err();
    assert!(err.to_string().contains("20 bytes"), "got: {}", err);
}

#[test]
fn evm_address_too_long() {
    let err = validate_evm_address("0x1234567890abcdef1234567890abcdef1234567800", "target").unwrap_err();
    assert!(err.to_string().contains("20 bytes"), "got: {}", err);
}

#[test]
fn evm_address_odd_length() {
    let err = validate_evm_address("0x1234567890abcdef1234567890abcdef1234567", "src").unwrap_err();
    assert!(err.to_string().contains("odd hex length"), "got: {}", err);
}

#[test]
fn evm_address_invalid_hex_char() {
    let err = validate_evm_address("0x1234567890abcdef1234567890abcdef1234567g", "src").unwrap_err();
    assert!(err.to_string().contains("not valid hex"), "got: {}", err);
}

#[test]
fn evm_address_with_spaces() {
    let err = validate_evm_address("  0x1234  ", "src").unwrap_err();
    // trimmed = "0x1234" → too short
    assert!(err.to_string().contains("20 bytes"), "got: {}", err);
}

#[test]
fn evm_address_0X_prefix() {
    assert!(validate_evm_address("0X1234567890abcdef1234567890abcdef12345678", "test").is_ok());
}

#[test]
fn evm_address_19_bytes() {
    let err = validate_evm_address("0x1234567890abcdef1234567890abcdef123456", "test").unwrap_err();
    assert!(err.to_string().contains("20 bytes"), "got: {}", err);
}

#[test]
fn evm_address_21_bytes() {
    let err = validate_evm_address("0x1234567890abcdef1234567890abcdef123456789a", "test").unwrap_err();
    assert!(err.to_string().contains("20 bytes"), "got: {}", err);
}

#[test]
fn evm_address_error_includes_tip() {
    let err = validate_evm_address("", "source").unwrap_err();
    assert!(err.to_string().contains("Tip:"), "error should include Tip: {}", err);
}

#[test]
fn evm_address_unicode() {
    let err = validate_evm_address("0x123こんにちは", "test").unwrap_err();
    assert!(err.to_string().contains("not valid hex"), "got: {}", err);
}

// =====================================================================
// validate_hex_data tests
// =====================================================================

#[test]
fn hex_data_valid_empty_0x() {
    assert!(validate_hex_data("0x", "test").is_ok());
}

#[test]
fn hex_data_valid_short() {
    assert!(validate_hex_data("0xdeadbeef", "test").is_ok());
}

#[test]
fn hex_data_valid_long() {
    assert!(validate_hex_data("0x0000000000000000000000000000000000000000000000000000000000000001", "test").is_ok());
}

#[test]
fn hex_data_valid_no_prefix() {
    assert!(validate_hex_data("cafebabe", "test").is_ok());
}

#[test]
fn hex_data_empty_string() {
    let err = validate_hex_data("", "code-hash").unwrap_err();
    assert!(err.to_string().contains("cannot be empty"), "got: {}", err);
}

#[test]
fn hex_data_odd_length() {
    let err = validate_hex_data("0xabc", "data").unwrap_err();
    assert!(err.to_string().contains("odd length"), "got: {}", err);
}

#[test]
fn hex_data_invalid_chars() {
    let err = validate_hex_data("0xnothex", "salt").unwrap_err();
    assert!(err.to_string().contains("not valid hex"), "got: {}", err);
}

#[test]
fn hex_data_spaces_only() {
    let err = validate_hex_data("   ", "test").unwrap_err();
    assert!(err.to_string().contains("cannot be empty"), "got: {}", err);
}

#[test]
fn hex_data_0X_prefix() {
    assert!(validate_hex_data("0Xabcd", "test").is_ok());
}

#[test]
fn hex_data_single_byte() {
    assert!(validate_hex_data("0xff", "test").is_ok());
}

#[test]
fn hex_data_error_includes_tip() {
    let err = validate_hex_data("0xabc", "salt").unwrap_err();
    assert!(err.to_string().contains("Tip:"), "error should include Tip: {}", err);
}

// =====================================================================
// validate_pallet_call tests
// =====================================================================

#[test]
fn pallet_call_valid_pascal_case() {
    assert!(validate_pallet_call("System", "pallet").is_ok());
}

#[test]
fn pallet_call_valid_pascal_multi_word() {
    assert!(validate_pallet_call("SubtensorModule", "pallet").is_ok());
}

#[test]
fn pallet_call_valid_snake_case() {
    assert!(validate_pallet_call("remark", "call").is_ok());
}

#[test]
fn pallet_call_valid_snake_multi_word() {
    assert!(validate_pallet_call("transfer_keep_alive", "call").is_ok());
}

#[test]
fn pallet_call_valid_with_numbers() {
    assert!(validate_pallet_call("Erc20", "pallet").is_ok());
}

#[test]
fn pallet_call_empty() {
    let err = validate_pallet_call("", "pallet").unwrap_err();
    assert!(err.to_string().contains("cannot be empty"), "got: {}", err);
}

#[test]
fn pallet_call_spaces_only() {
    let err = validate_pallet_call("   ", "call").unwrap_err();
    assert!(err.to_string().contains("cannot be empty"), "got: {}", err);
}

#[test]
fn pallet_call_starts_with_number() {
    let err = validate_pallet_call("1System", "pallet").unwrap_err();
    assert!(err.to_string().contains("must start with a letter"), "got: {}", err);
}

#[test]
fn pallet_call_starts_with_underscore() {
    let err = validate_pallet_call("_private", "call").unwrap_err();
    assert!(err.to_string().contains("must start with a letter"), "got: {}", err);
}

#[test]
fn pallet_call_contains_dash() {
    let err = validate_pallet_call("my-pallet", "pallet").unwrap_err();
    assert!(err.to_string().contains("not allowed"), "got: {}", err);
}

#[test]
fn pallet_call_contains_space() {
    let err = validate_pallet_call("my pallet", "pallet").unwrap_err();
    assert!(err.to_string().contains("not allowed"), "got: {}", err);
}

#[test]
fn pallet_call_contains_dot() {
    let err = validate_pallet_call("System.remark", "pallet").unwrap_err();
    assert!(err.to_string().contains("not allowed"), "got: {}", err);
}

#[test]
fn pallet_call_too_long() {
    let long = "A".repeat(129);
    let err = validate_pallet_call(&long, "pallet").unwrap_err();
    assert!(err.to_string().contains("too long"), "got: {}", err);
}

#[test]
fn pallet_call_exactly_128() {
    let ok = "A".repeat(128);
    assert!(validate_pallet_call(&ok, "pallet").is_ok());
}

#[test]
fn pallet_call_unicode() {
    let err = validate_pallet_call("Sÿstem", "pallet").unwrap_err();
    assert!(err.to_string().contains("not allowed"), "got: {}", err);
}

#[test]
fn pallet_call_error_includes_tip() {
    let err = validate_pallet_call("", "pallet").unwrap_err();
    assert!(err.to_string().contains("Tip:"), "error should include Tip: {}", err);
}

// =====================================================================
// validate_schedule_id tests
// =====================================================================

#[test]
fn schedule_id_valid_short() {
    assert!(validate_schedule_id("my_task").is_ok());
}

#[test]
fn schedule_id_valid_32_bytes() {
    assert!(validate_schedule_id(&"x".repeat(32)).is_ok());
}

#[test]
fn schedule_id_empty() {
    let err = validate_schedule_id("").unwrap_err();
    assert!(err.to_string().contains("cannot be empty"), "got: {}", err);
}

#[test]
fn schedule_id_too_long() {
    let err = validate_schedule_id(&"a".repeat(33)).unwrap_err();
    assert!(err.to_string().contains("too long"), "got: {}", err);
}

#[test]
fn schedule_id_single_char() {
    assert!(validate_schedule_id("x").is_ok());
}

#[test]
fn schedule_id_with_special_chars() {
    // The chain interprets id as bytes; any non-empty ≤32 is valid
    assert!(validate_schedule_id("my-task-#1!").is_ok());
}

#[test]
fn schedule_id_error_includes_tip() {
    let err = validate_schedule_id("").unwrap_err();
    assert!(err.to_string().contains("Tip:"), "error should include Tip: {}", err);
}
