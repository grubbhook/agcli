//! CLI parsing and non-interactive flag tests.
//! Run with: cargo test --test cli_test

use clap::Parser;

/// Verify that --yes flag is parsed globally.
#[test]
fn parse_global_yes_flag() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--yes", "balance",
    ]);
    assert!(cli.is_ok(), "should parse --yes flag: {:?}", cli.err());
    let cli = cli.unwrap();
    assert!(cli.yes);
}

/// Verify -y short form works.
#[test]
fn parse_global_y_short() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "-y", "balance",
    ]);
    assert!(cli.is_ok());
    assert!(cli.unwrap().yes);
}

/// Verify --password is parsed globally.
#[test]
fn parse_global_password() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--password", "mysecret", "balance",
    ]);
    assert!(cli.is_ok());
    assert_eq!(cli.unwrap().password, Some("mysecret".to_string()));
}

/// Verify wallet create accepts --password.
#[test]
fn parse_wallet_create_with_password() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "create", "--name", "test", "--password", "abc123",
    ]);
    assert!(cli.is_ok());
}

/// Verify wallet import accepts --mnemonic and --password.
#[test]
fn parse_wallet_import_non_interactive() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "import", "--name", "test",
        "--mnemonic", "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        "--password", "pass",
    ]);
    assert!(cli.is_ok());
}

/// Verify stake wizard accepts --netuid, --amount, --hotkey.
#[test]
fn parse_stake_wizard_non_interactive() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--yes", "--password", "pass",
        "stake", "wizard", "--netuid", "1", "--amount", "0.5",
    ]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    assert!(cli.yes);
    assert_eq!(cli.password, Some("pass".to_string()));
}

/// Verify network flag defaults to finney.
#[test]
fn default_network_is_finney() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "balance"]);
    assert!(cli.is_ok());
    assert_eq!(cli.unwrap().network, "finney");
}

/// Verify --output json is accepted.
#[test]
fn parse_output_json() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--output", "json", "balance",
    ]);
    assert!(cli.is_ok());
    assert_eq!(cli.unwrap().output, "json");
}

/// Verify --output csv is accepted.
#[test]
fn parse_output_csv() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--output", "csv", "balance",
    ]);
    assert!(cli.is_ok());
    assert_eq!(cli.unwrap().output, "csv");
}

/// Invalid output format is rejected.
#[test]
fn parse_output_invalid_rejected() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--output", "xml", "balance",
    ]);
    assert!(cli.is_err());
}

/// Verify all stake subcommands parse.
#[test]
fn parse_stake_add() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "add", "1.5", "--netuid", "1",
    ]);
    assert!(cli.is_ok(), "stake add should parse: {:?}", cli.err());
}

/// Verify transfer parses.
#[test]
fn parse_transfer() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "transfer", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", "1.0",
    ]);
    assert!(cli.is_ok());
}

/// Verify subnet list parses.
#[test]
fn parse_subnet_list() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "list"]);
    assert!(cli.is_ok());
}

/// Verify view portfolio parses.
#[test]
fn parse_view_portfolio() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "portfolio"]);
    assert!(cli.is_ok());
}

/// Verify regen-coldkey accepts --mnemonic.
#[test]
fn parse_regen_coldkey_with_mnemonic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "regen-coldkey",
        "--mnemonic", "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        "--password", "pass",
    ]);
    assert!(cli.is_ok());
}

/// Verify config subcommands parse.
#[test]
fn parse_config_show() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "config", "show"]);
    assert!(cli.is_ok());
}

/// Verify completions subcommand parses.
#[test]
fn parse_completions() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "completions", "bash"]);
    assert!(cli.is_ok());
}

/// Verify all view subcommands parse.
#[test]
fn parse_view_network() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "network"]);
    assert!(cli.is_ok());
}

#[test]
fn parse_view_dynamic() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "dynamic"]);
    assert!(cli.is_ok());
}

#[test]
fn parse_view_validators() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "validators", "--limit", "10"]);
    assert!(cli.is_ok());
}

#[test]
fn parse_view_swap_sim() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "view", "swap-sim", "--netuid", "1", "--tao", "10.0",
    ]);
    assert!(cli.is_ok());
}

/// Verify proxy subcommands parse.
#[test]
fn parse_proxy_list() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "proxy", "list"]);
    assert!(cli.is_ok());
}

/// Verify endpoint override works.
#[test]
fn parse_endpoint_override() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--endpoint", "ws://127.0.0.1:9944", "balance",
    ]);
    assert!(cli.is_ok());
    assert_eq!(cli.unwrap().endpoint, Some("ws://127.0.0.1:9944".to_string()));
}

/// Verify live flag parses with a value.
#[test]
fn parse_live_flag() {
    // --live requires a value or no value; with Option<Option<u64>>,
    // the bare --live may conflict with subcommand parsing.
    // Test with explicit value:
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--live", "5", "subnet", "metagraph", "1",
    ]);
    assert!(cli.is_ok(), "should parse --live 5: {:?}", cli.err());
}
