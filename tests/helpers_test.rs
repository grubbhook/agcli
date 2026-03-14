//! Tests for CLI helper functions.
//! Run with: cargo test --test helpers_test

use agcli::cli::helpers::{parse_weight_pairs, parse_children};

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
