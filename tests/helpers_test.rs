//! Tests for CLI helper functions.
//! Run with: cargo test --test helpers_test

use agcli::cli::helpers::{parse_weight_pairs, parse_children};
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
    assert_eq!(result[0].1, "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY");
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
    assert!(msg.contains("Invalid UID"), "Expected helpful UID error, got: {}", msg);
}

#[test]
fn parse_weight_pairs_overflow_weight() {
    // Weight > 65535 should fail
    let result = parse_weight_pairs("0:70000");
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("Invalid weight"), "Expected helpful weight error, got: {}", msg);
}

#[test]
fn parse_children_bad_proportion() {
    let result = parse_children("abc:5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY");
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("Invalid proportion"), "Expected helpful proportion error, got: {}", msg);
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
    assert!(topics.len() >= 10, "Expected at least 10 topics, got {}", topics.len());
    for (key, desc) in &topics {
        assert!(!key.is_empty());
        assert!(!desc.is_empty());
    }
}

#[test]
fn explain_content_has_substance() {
    // Each explanation should be non-trivial
    let text = explain::explain("tempo").unwrap();
    assert!(text.len() > 100, "Explanation too short: {} chars", text.len());
    assert!(text.contains("blocks"), "Tempo explanation should mention blocks");
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

// ──── Step 18: Batch mode & spending limits tests ────

#[test]
fn batch_mode_flag_sets_global() {
    use agcli::cli::helpers::{set_batch_mode, is_batch_mode};
    set_batch_mode(true);
    assert!(is_batch_mode());
    set_batch_mode(false);
    assert!(!is_batch_mode());
}

#[test]
fn spending_limit_no_config_passes() {
    // No config file → should pass for any amount
    let result = agcli::cli::helpers::check_spending_limit(97, 100.0);
    assert!(result.is_ok(), "No config should always pass: {:?}", result.err());
}
