//! Config and network resolution tests.
//! Run with: cargo test --test config_test

use clap::Parser;
use agcli::cli::Cli;
use agcli::types::network::Network;

#[test]
fn resolve_finney() {
    let cli = Cli::try_parse_from(["agcli", "balance"]).unwrap();
    let net = cli.resolve_network();
    assert!(matches!(net, Network::Finney));
    assert_eq!(net.ws_url(), "wss://entrypoint-finney.opentensor.ai:443");
}

#[test]
fn resolve_test_network() {
    let cli = Cli::try_parse_from(["agcli", "--network", "test", "balance"]).unwrap();
    let net = cli.resolve_network();
    assert!(matches!(net, Network::Test));
}

#[test]
fn resolve_local_network() {
    let cli = Cli::try_parse_from(["agcli", "--network", "local", "balance"]).unwrap();
    let net = cli.resolve_network();
    assert!(matches!(net, Network::Local));
}

#[test]
fn endpoint_overrides_network() {
    let cli = Cli::try_parse_from([
        "agcli", "--endpoint", "ws://custom:9944", "--network", "test", "balance",
    ]).unwrap();
    let net = cli.resolve_network();
    assert!(matches!(net, Network::Custom(_)));
    assert_eq!(net.ws_url(), "ws://custom:9944");
}

#[test]
fn config_apply_defaults() {
    let mut cli = Cli::try_parse_from(["agcli", "balance"]).unwrap();
    let mut cfg = agcli::Config::default();
    cfg.network = Some("test".to_string());
    cfg.wallet = Some("mywallet".to_string());
    cli.apply_config(&cfg);
    assert_eq!(cli.network, "test");
    assert_eq!(cli.wallet, "mywallet");
}

#[test]
fn cli_flags_override_config() {
    let mut cli = Cli::try_parse_from([
        "agcli", "--network", "local", "--wallet", "explicit", "balance",
    ]).unwrap();
    let mut cfg = agcli::Config::default();
    cfg.network = Some("test".to_string());
    cfg.wallet = Some("config_wallet".to_string());
    cli.apply_config(&cfg);
    // CLI flags should take precedence
    assert_eq!(cli.network, "local");
    assert_eq!(cli.wallet, "explicit");
}

#[test]
fn live_interval_parsing() {
    // --live with explicit value
    let cli = Cli::try_parse_from(["agcli", "--live", "5", "subnet", "metagraph", "1"]).unwrap();
    assert_eq!(cli.live_interval(), Some(5));
}
