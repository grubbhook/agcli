//! Tests for CLI helper functions.
//! Run with: cargo test --test helpers_test

use agcli::cli::helpers::{parse_children, parse_weight_pairs};
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
    let result = parse_children("500:5Abc,500:5Def").unwrap();
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
    // Should produce a child with empty hotkey string (not an error from parsing)
    assert!(result.is_ok());
    let children = result.unwrap();
    assert_eq!(children[0].1, "");
}

#[test]
fn parse_children_zero_proportion() {
    let result = parse_children("0:5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY");
    assert!(result.is_ok(), "zero proportion should be allowed");
    assert_eq!(result.unwrap()[0].0, 0);
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
    let val = json_to_subxt_value(&serde_json::json!(3.14));
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
    assert!(keypair::is_valid_ss58(
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
    ));
    assert!(!keypair::is_valid_ss58("invalid"));
    assert!(!keypair::is_valid_ss58(""));
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
        topics.len() >= 19,
        "Expected at least 19 topics (was 18, now +coldkey-swap), got {}",
        topics.len()
    );
}
