//! CLI parsing and non-interactive flag tests.
//! Run with: cargo test --test cli_test

use agcli::cli::OutputFormat;
use clap::Parser;

/// Verify that --yes flag is parsed globally.
#[test]
fn parse_global_yes_flag() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--yes", "balance"]);
    assert!(cli.is_ok(), "should parse --yes flag: {:?}", cli.err());
    let cli = cli.unwrap();
    assert!(cli.yes);
}

/// Verify -y short form works.
#[test]
fn parse_global_y_short() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "-y", "balance"]);
    assert!(cli.is_ok());
    assert!(cli.unwrap().yes);
}

/// Verify --password is parsed globally.
#[test]
fn parse_global_password() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--password", "mysecret", "balance"]);
    assert!(cli.is_ok());
    assert_eq!(cli.unwrap().password, Some("mysecret".to_string()));
}

/// Verify wallet create accepts --password.
#[test]
fn parse_wallet_create_with_password() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "wallet",
        "create",
        "--name",
        "test",
        "--password",
        "abc123",
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
        "agcli",
        "--yes",
        "--password",
        "pass",
        "stake",
        "wizard",
        "--netuid",
        "1",
        "--amount",
        "0.5",
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
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--output", "json", "balance"]);
    assert!(cli.is_ok());
    assert_eq!(cli.unwrap().output, OutputFormat::Json);
}

/// Verify --output csv is accepted.
#[test]
fn parse_output_csv() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--output", "csv", "balance"]);
    assert!(cli.is_ok());
    assert_eq!(cli.unwrap().output, OutputFormat::Csv);
}

/// Invalid output format is rejected.
#[test]
fn parse_output_invalid_rejected() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--output", "xml", "balance"]);
    assert!(cli.is_err());
}

/// Verify all stake subcommands parse.
#[test]
fn parse_stake_add() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "add", "--amount", "1.5", "--netuid", "1",
    ]);
    assert!(cli.is_ok(), "stake add should parse: {:?}", cli.err());
}

/// Verify transfer parses.
#[test]
fn parse_transfer() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "transfer",
        "--dest",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--amount",
        "1.0",
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
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "completions", "--shell", "bash"]);
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
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "--endpoint", "ws://127.0.0.1:9944", "balance"]);
    assert!(cli.is_ok());
    assert_eq!(
        cli.unwrap().endpoint,
        Some("ws://127.0.0.1:9944".to_string())
    );
}

/// Verify live flag parses with a value.
#[test]
fn parse_live_flag() {
    // --live requires a value or no value; with Option<Option<u64>>,
    // the bare --live may conflict with subcommand parsing.
    // Test with explicit value:
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--live",
        "5",
        "subnet",
        "metagraph",
        "--netuid",
        "1",
    ]);
    assert!(cli.is_ok(), "should parse --live 5: {:?}", cli.err());
}

// ──── Step 17: New command parsing tests ────

/// Verify weights commit-reveal parses.
#[test]
fn parse_weights_commit_reveal() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "commit-reveal",
        "--netuid",
        "97",
        "--weights",
        "0:100,1:200",
        "--wait",
    ]);
    assert!(
        cli.is_ok(),
        "should parse weights commit-reveal: {:?}",
        cli.err()
    );
}

/// Verify weights set --dry-run parses.
#[test]
fn parse_weights_set_dry_run() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "set",
        "--netuid",
        "1",
        "--weights",
        "0:100,1:200",
        "--dry-run",
    ]);
    assert!(
        cli.is_ok(),
        "should parse weights set --dry-run: {:?}",
        cli.err()
    );
}

/// Verify subnet monitor parses with --json flag.
#[test]
fn parse_subnet_monitor() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "subnet", "monitor", "--netuid", "97", "--json"]);
    assert!(cli.is_ok(), "should parse subnet monitor: {:?}", cli.err());
}

/// Verify subnet monitor custom interval.
#[test]
fn parse_subnet_monitor_interval() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "monitor",
        "--netuid",
        "1",
        "--interval",
        "60",
    ]);
    assert!(
        cli.is_ok(),
        "should parse subnet monitor with interval: {:?}",
        cli.err()
    );
}

/// Verify subnet health parses.
#[test]
fn parse_subnet_health() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "health", "--netuid", "97"]);
    assert!(cli.is_ok(), "should parse subnet health: {:?}", cli.err());
}

/// Verify subnet emissions parses.
#[test]
fn parse_subnet_emissions() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "emissions", "--netuid", "97"]);
    assert!(
        cli.is_ok(),
        "should parse subnet emissions: {:?}",
        cli.err()
    );
}

/// Verify subnet cost parses.
#[test]
fn parse_subnet_cost() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "cost", "--netuid", "97"]);
    assert!(cli.is_ok(), "should parse subnet cost: {:?}", cli.err());
}

/// Verify metagraph --uid single-UID lookup parses.
#[test]
fn parse_metagraph_single_uid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "metagraph",
        "--netuid",
        "97",
        "--uid",
        "11",
    ]);
    assert!(cli.is_ok(), "should parse metagraph --uid: {:?}", cli.err());
}

/// Verify metagraph without --uid still works.
#[test]
fn parse_metagraph_full() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "metagraph", "--netuid", "1"]);
    assert!(cli.is_ok(), "should parse full metagraph: {:?}", cli.err());
}

// ──── Step 18: Batch mode, wallet sign/derive, events, balance watch tests ────

/// Verify --batch flag is parsed globally.
#[test]
fn parse_global_batch_flag() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--batch", "balance"]);
    assert!(cli.is_ok(), "should parse --batch flag: {:?}", cli.err());
    assert!(cli.unwrap().batch);
}

/// Verify --pretty flag is parsed globally.
#[test]
fn parse_global_pretty_flag() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--pretty", "--output", "json", "balance"]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    assert!(cli.pretty);
    assert_eq!(cli.output, OutputFormat::Json);
}

/// Verify wallet sign parses.
#[test]
fn parse_wallet_sign() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "wallet", "sign", "--message", "hello world"]);
    assert!(cli.is_ok(), "should parse wallet sign: {:?}", cli.err());
}

/// Verify wallet verify parses.
#[test]
fn parse_wallet_verify() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "wallet",
        "verify",
        "--message",
        "hello world",
        "--signature",
        "0xaabbccdd",
        "--signer",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "should parse wallet verify: {:?}", cli.err());
}

/// Verify wallet derive parses.
#[test]
fn parse_wallet_derive() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "wallet",
        "derive",
        "--input",
        "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d",
    ]);
    assert!(cli.is_ok(), "should parse wallet derive: {:?}", cli.err());
}

/// Verify balance --watch parses.
#[test]
fn parse_balance_watch() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "balance",
        "--watch",
        "30",
        "--threshold",
        "10.0",
    ]);
    assert!(cli.is_ok(), "should parse balance watch: {:?}", cli.err());
}

/// Verify subscribe events --netuid filter parses.
#[test]
fn parse_subscribe_events_with_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subscribe",
        "events",
        "--filter",
        "staking",
        "--netuid",
        "97",
    ]);
    assert!(
        cli.is_ok(),
        "should parse subscribe events with netuid: {:?}",
        cli.err()
    );
}

/// Verify subscribe events --account filter parses.
#[test]
fn parse_subscribe_events_with_account() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subscribe",
        "events",
        "--filter",
        "weights",
        "--account",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(
        cli.is_ok(),
        "should parse subscribe events with account: {:?}",
        cli.err()
    );
}

/// Verify --batch and --yes can be combined.
#[test]
fn parse_batch_and_yes_combined() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--batch",
        "--yes",
        "--password",
        "pass",
        "balance",
    ]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    assert!(cli.batch);
    assert!(cli.yes);
}

// ──── Step 20: Batch extrinsics command ────

/// Verify batch command parses with file argument.
#[test]
fn parse_batch_command() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "batch", "--file", "calls.json"]);
    assert!(cli.is_ok(), "should parse batch command: {:?}", cli.err());
}

/// Verify batch --no-atomic flag parses.
#[test]
fn parse_batch_no_atomic() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "batch", "--file", "calls.json", "--no-atomic"]);
    assert!(
        cli.is_ok(),
        "should parse batch --no-atomic: {:?}",
        cli.err()
    );
}

// ──── Step 25: Wallet CSV, explain, and missing command tests ────

/// Verify wallet list with --output csv parses.
#[test]
fn parse_wallet_list_csv() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--output", "csv", "wallet", "list"]);
    assert!(
        cli.is_ok(),
        "should parse wallet list --output csv: {:?}",
        cli.err()
    );
    assert_eq!(cli.unwrap().output, OutputFormat::Csv);
}

/// Verify wallet show --all with --output csv parses.
#[test]
fn parse_wallet_show_all_csv() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "--output", "csv", "wallet", "show", "--all"]);
    assert!(
        cli.is_ok(),
        "should parse wallet show --all --output csv: {:?}",
        cli.err()
    );
    let cli = cli.unwrap();
    assert_eq!(cli.output, OutputFormat::Csv);
}

/// Verify wallet show --all with --output json parses.
#[test]
fn parse_wallet_show_all_json() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "--output", "json", "wallet", "show", "--all"]);
    assert!(
        cli.is_ok(),
        "should parse wallet show --all --output json: {:?}",
        cli.err()
    );
    assert_eq!(cli.unwrap().output, OutputFormat::Json);
}

/// Verify explain without topic parses (lists all topics).
#[test]
fn parse_explain_no_topic() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "explain"]);
    assert!(
        cli.is_ok(),
        "should parse explain without topic: {:?}",
        cli.err()
    );
}

/// Verify explain with --topic parses.
#[test]
fn parse_explain_with_topic() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "explain", "--topic", "tempo"]);
    assert!(cli.is_ok(), "should parse explain --topic: {:?}", cli.err());
}

/// Verify explain with --output json parses.
#[test]
fn parse_explain_json() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "--output", "json", "explain", "--topic", "amm"]);
    assert!(
        cli.is_ok(),
        "should parse explain --output json: {:?}",
        cli.err()
    );
    assert_eq!(cli.unwrap().output, OutputFormat::Json);
}

/// Verify subnet liquidity without netuid parses (all subnets).
#[test]
fn parse_subnet_liquidity_all() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "liquidity"]);
    assert!(
        cli.is_ok(),
        "should parse subnet liquidity: {:?}",
        cli.err()
    );
}

/// Verify subnet liquidity with --netuid parses.
#[test]
fn parse_subnet_liquidity_single() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "liquidity", "--netuid", "1"]);
    assert!(
        cli.is_ok(),
        "should parse subnet liquidity --netuid: {:?}",
        cli.err()
    );
}

/// Verify subnet watch parses with custom interval.
#[test]
fn parse_subnet_watch_custom_interval() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "watch",
        "--netuid",
        "1",
        "--interval",
        "30",
    ]);
    assert!(
        cli.is_ok(),
        "should parse subnet watch --interval: {:?}",
        cli.err()
    );
}

/// Verify stake add with --max-slippage parses.
#[test]
fn parse_stake_add_max_slippage() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "add",
        "--amount",
        "1.0",
        "--netuid",
        "1",
        "--max-slippage",
        "2.0",
    ]);
    assert!(
        cli.is_ok(),
        "should parse stake add --max-slippage: {:?}",
        cli.err()
    );
}

/// Verify stake list with --output csv parses.
#[test]
fn parse_stake_list_csv() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--output", "csv", "stake", "list"]);
    assert!(
        cli.is_ok(),
        "should parse stake list --output csv: {:?}",
        cli.err()
    );
    assert_eq!(cli.unwrap().output, OutputFormat::Csv);
}

/// Verify view portfolio with --output json parses.
#[test]
fn parse_view_portfolio_json() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--output", "json", "view", "portfolio"]);
    assert!(
        cli.is_ok(),
        "should parse view portfolio --output json: {:?}",
        cli.err()
    );
    assert_eq!(cli.unwrap().output, OutputFormat::Json);
}

/// Verify view account parses.
#[test]
fn parse_view_account() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "view",
        "account",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "should parse view account: {:?}", cli.err());
}

/// Verify view staking-analytics parses.
#[test]
fn parse_view_staking_analytics() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "staking-analytics"]);
    assert!(
        cli.is_ok(),
        "should parse view staking-analytics: {:?}",
        cli.err()
    );
}

/// Verify all global flags can be combined with any subcommand.
#[test]
fn parse_all_global_flags_combined() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--output",
        "json",
        "--pretty",
        "--yes",
        "--batch",
        "--password",
        "pw",
        "--network",
        "test",
        "balance",
    ]);
    assert!(
        cli.is_ok(),
        "should parse all global flags combined: {:?}",
        cli.err()
    );
    let cli = cli.unwrap();
    assert_eq!(cli.output, OutputFormat::Json);
    assert!(cli.pretty);
    assert!(cli.yes);
    assert!(cli.batch);
    assert_eq!(cli.password, Some("pw".to_string()));
    assert_eq!(cli.network, "test");
}

/// Verify wallet new-hotkey parses.
#[test]
fn parse_wallet_new_hotkey() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "wallet", "new-hotkey", "--name", "validator"]);
    assert!(
        cli.is_ok(),
        "should parse wallet new-hotkey: {:?}",
        cli.err()
    );
}

// ════════════════════════════════════════════════════════════════════
// Weight commands
// ════════════════════════════════════════════════════════════════════

#[test]
fn parse_weights_set() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "set",
        "--netuid",
        "1",
        "--weights",
        "0:0.5,1:0.3,2:0.2",
    ]);
    assert!(cli.is_ok(), "should parse weights set: {:?}", cli.err());
}

#[test]
fn parse_weights_commit() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "commit",
        "--netuid",
        "97",
        "--weights",
        "0:0.5,1:0.5",
    ]);
    assert!(cli.is_ok(), "should parse weights commit: {:?}", cli.err());
}

#[test]
fn parse_weights_commit_with_salt() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "commit",
        "--netuid",
        "97",
        "--weights",
        "0:1.0",
        "--salt",
        "deadbeef",
    ]);
    assert!(
        cli.is_ok(),
        "should parse weights commit with salt: {:?}",
        cli.err()
    );
}

#[test]
fn parse_weights_reveal() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "reveal",
        "--netuid",
        "97",
        "--weights",
        "0:0.5,1:0.5",
        "--salt",
        "abc123",
        "--version-key",
        "42",
    ]);
    assert!(cli.is_ok(), "should parse weights reveal: {:?}", cli.err());
}

// ════════════════════════════════════════════════════════════════════
// Delegate commands
// ════════════════════════════════════════════════════════════════════

#[test]
fn parse_delegate_show() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "delegate",
        "show",
        "--hotkey",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "should parse delegate show: {:?}", cli.err());
}

#[test]
fn parse_delegate_show_default() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "delegate", "show"]);
    assert!(
        cli.is_ok(),
        "should parse delegate show without hotkey: {:?}",
        cli.err()
    );
}

#[test]
fn parse_delegate_list() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "delegate", "list"]);
    assert!(cli.is_ok(), "should parse delegate list: {:?}", cli.err());
}

#[test]
fn parse_delegate_decrease_take() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "delegate", "decrease-take", "--take", "9.5"]);
    assert!(
        cli.is_ok(),
        "should parse delegate decrease-take: {:?}",
        cli.err()
    );
}

#[test]
fn parse_delegate_increase_take() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "delegate",
        "increase-take",
        "--take",
        "11.0",
        "--hotkey",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(
        cli.is_ok(),
        "should parse delegate increase-take: {:?}",
        cli.err()
    );
}

// ════════════════════════════════════════════════════════════════════
// Identity commands
// ════════════════════════════════════════════════════════════════════

#[test]
fn parse_identity_set() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "identity",
        "set",
        "--name",
        "MyValidator",
        "--url",
        "https://example.com",
        "--github",
        "myuser",
    ]);
    assert!(cli.is_ok(), "should parse identity set: {:?}", cli.err());
}

#[test]
fn parse_identity_set_minimal() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "identity", "set", "--name", "ValidatorX"]);
    assert!(
        cli.is_ok(),
        "should parse identity set with name only: {:?}",
        cli.err()
    );
}

#[test]
fn parse_identity_show() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "identity",
        "show",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "should parse identity show: {:?}", cli.err());
}

#[test]
fn parse_identity_set_subnet() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "identity",
        "set-subnet",
        "--netuid",
        "97",
        "--name",
        "MySN",
        "--github",
        "org/repo",
        "--url",
        "https://sn97.io",
    ]);
    assert!(
        cli.is_ok(),
        "should parse identity set-subnet: {:?}",
        cli.err()
    );
}

// ════════════════════════════════════════════════════════════════════
// Serve commands
// ════════════════════════════════════════════════════════════════════

#[test]
fn parse_serve_axon() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "serve",
        "axon",
        "--netuid",
        "1",
        "--ip",
        "192.168.1.100",
        "--port",
        "8091",
    ]);
    assert!(cli.is_ok(), "should parse serve axon: {:?}", cli.err());
}

#[test]
fn parse_serve_axon_custom_protocol() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "serve",
        "axon",
        "--netuid",
        "1",
        "--ip",
        "10.0.0.1",
        "--port",
        "9090",
        "--protocol",
        "6",
        "--version",
        "42",
    ]);
    assert!(
        cli.is_ok(),
        "should parse serve axon with protocol/version: {:?}",
        cli.err()
    );
}

// ════════════════════════════════════════════════════════════════════
// Swap commands
// ════════════════════════════════════════════════════════════════════

#[test]
fn parse_swap_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "swap",
        "hotkey",
        "--new-hotkey",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
    ]);
    assert!(cli.is_ok(), "should parse swap hotkey: {:?}", cli.err());
}

#[test]
fn parse_swap_coldkey() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "swap",
        "coldkey",
        "--new-coldkey",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
    ]);
    assert!(cli.is_ok(), "should parse swap coldkey: {:?}", cli.err());
}

// ════════════════════════════════════════════════════════════════════
// Multisig commands
// ════════════════════════════════════════════════════════════════════

#[test]
fn parse_multisig_address() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "multisig", "address",
        "--signatories", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY,5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--threshold", "2",
    ]);
    assert!(
        cli.is_ok(),
        "should parse multisig address: {:?}",
        cli.err()
    );
}

#[test]
fn parse_multisig_submit() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "multisig",
        "submit",
        "--others",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--threshold",
        "2",
        "--pallet",
        "Balances",
        "--call",
        "transfer_keep_alive",
        "--args",
        r#"[{"Id":"5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"},1000000000]"#,
    ]);
    assert!(cli.is_ok(), "should parse multisig submit: {:?}", cli.err());
}

#[test]
fn parse_multisig_approve() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "multisig",
        "approve",
        "--others",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--threshold",
        "2",
        "--call-hash",
        "0xdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
    ]);
    assert!(
        cli.is_ok(),
        "should parse multisig approve: {:?}",
        cli.err()
    );
}

// ════════════════════════════════════════════════════════════════════
// Crowdloan commands
// ════════════════════════════════════════════════════════════════════

#[test]
fn parse_crowdloan_contribute() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "crowdloan",
        "contribute",
        "--crowdloan-id",
        "1",
        "--amount",
        "10.0",
    ]);
    assert!(
        cli.is_ok(),
        "should parse crowdloan contribute: {:?}",
        cli.err()
    );
}

#[test]
fn parse_crowdloan_withdraw() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "crowdloan", "withdraw", "--crowdloan-id", "1"]);
    assert!(
        cli.is_ok(),
        "should parse crowdloan withdraw: {:?}",
        cli.err()
    );
}

#[test]
fn parse_crowdloan_finalize() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "crowdloan", "finalize", "--crowdloan-id", "1"]);
    assert!(
        cli.is_ok(),
        "should parse crowdloan finalize: {:?}",
        cli.err()
    );
}

// ════════════════════════════════════════════════════════════════════
// Stake commands (untested operations)
// ════════════════════════════════════════════════════════════════════

#[test]
fn parse_stake_remove() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "remove",
        "--amount",
        "5.0",
        "--netuid",
        "1",
        "--hotkey",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "should parse stake remove: {:?}", cli.err());
}

#[test]
fn parse_stake_remove_max_slippage() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "remove",
        "--amount",
        "10.0",
        "--netuid",
        "3",
        "--max-slippage",
        "1.5",
    ]);
    assert!(
        cli.is_ok(),
        "should parse stake remove with slippage: {:?}",
        cli.err()
    );
}

#[test]
fn parse_stake_move() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "move", "--amount", "100.0", "--from", "1", "--to", "3",
    ]);
    assert!(cli.is_ok(), "should parse stake move: {:?}", cli.err());
}

#[test]
fn parse_stake_swap() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "swap",
        "--amount",
        "50.0",
        "--netuid",
        "1",
        "--from-hotkey",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--to-hotkey",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
    ]);
    assert!(cli.is_ok(), "should parse stake swap: {:?}", cli.err());
}

#[test]
fn parse_stake_unstake_all() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "stake", "unstake-all"]);
    assert!(
        cli.is_ok(),
        "should parse stake unstake-all: {:?}",
        cli.err()
    );
}

#[test]
fn parse_stake_unstake_all_with_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "unstake-all",
        "--hotkey",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(
        cli.is_ok(),
        "should parse stake unstake-all with hotkey: {:?}",
        cli.err()
    );
}

#[test]
fn parse_stake_claim_root() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "stake", "claim-root", "--netuid", "0"]);
    assert!(
        cli.is_ok(),
        "should parse stake claim-root: {:?}",
        cli.err()
    );
}

#[test]
fn parse_stake_add_limit() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "add-limit",
        "--amount",
        "10.0",
        "--netuid",
        "1",
        "--price",
        "0.05",
        "--partial",
    ]);
    assert!(cli.is_ok(), "should parse stake add-limit: {:?}", cli.err());
}

#[test]
fn parse_stake_remove_limit() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "remove-limit",
        "--amount",
        "100.0",
        "--netuid",
        "1",
        "--price",
        "0.05",
    ]);
    assert!(
        cli.is_ok(),
        "should parse stake remove-limit: {:?}",
        cli.err()
    );
}

#[test]
fn parse_stake_childkey_take() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "childkey-take",
        "--take",
        "12.5",
        "--netuid",
        "1",
    ]);
    assert!(
        cli.is_ok(),
        "should parse stake childkey-take: {:?}",
        cli.err()
    );
}

#[test]
fn parse_stake_set_children() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "set-children", "--netuid", "1",
        "--children", "0.5:5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY,0.5:5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
    ]);
    assert!(
        cli.is_ok(),
        "should parse stake set-children: {:?}",
        cli.err()
    );
}

#[test]
fn parse_stake_recycle_alpha() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "recycle-alpha",
        "--amount",
        "500.0",
        "--netuid",
        "3",
    ]);
    assert!(
        cli.is_ok(),
        "should parse stake recycle-alpha: {:?}",
        cli.err()
    );
}

#[test]
fn parse_stake_unstake_all_alpha() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "stake", "unstake-all-alpha"]);
    assert!(
        cli.is_ok(),
        "should parse stake unstake-all-alpha: {:?}",
        cli.err()
    );
}

#[test]
fn parse_stake_burn_alpha() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "burn-alpha",
        "--amount",
        "100.0",
        "--netuid",
        "1",
    ]);
    assert!(
        cli.is_ok(),
        "should parse stake burn-alpha: {:?}",
        cli.err()
    );
}

#[test]
fn parse_stake_swap_limit() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "swap-limit",
        "--amount",
        "100.0",
        "--from",
        "1",
        "--to",
        "3",
        "--price",
        "0.1",
        "--partial",
    ]);
    assert!(
        cli.is_ok(),
        "should parse stake swap-limit: {:?}",
        cli.err()
    );
}

// ════════════════════════════════════════════════════════════════════
// Proxy add/remove commands
// ════════════════════════════════════════════════════════════════════

#[test]
fn parse_proxy_add() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "add",
        "--delegate",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--proxy-type",
        "staking",
    ]);
    assert!(cli.is_ok(), "should parse proxy add: {:?}", cli.err());
}

#[test]
fn parse_proxy_remove() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "remove",
        "--delegate",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "should parse proxy remove: {:?}", cli.err());
}

// ════════════════════════════════════════════════════════════════════
// View analytics commands
// ════════════════════════════════════════════════════════════════════

#[test]
fn parse_view_subnet_analytics() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "view", "subnet-analytics", "--netuid", "1"]);
    assert!(
        cli.is_ok(),
        "should parse view subnet-analytics: {:?}",
        cli.err()
    );
}

#[test]
fn parse_view_swap_sim_reverse() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "view", "swap-sim", "--netuid", "1", "--alpha", "100.0",
    ]);
    assert!(
        cli.is_ok(),
        "should parse view swap-sim alpha→tao: {:?}",
        cli.err()
    );
}

#[test]
fn parse_view_nominations() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "view",
        "nominations",
        "--hotkey",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(
        cli.is_ok(),
        "should parse view nominations: {:?}",
        cli.err()
    );
}

#[test]
fn parse_view_neuron() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "view", "neuron", "--netuid", "1", "--uid", "0"]);
    assert!(cli.is_ok(), "should parse view neuron: {:?}", cli.err());
}

#[test]
fn parse_view_history() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "view",
        "history",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--limit",
        "50",
    ]);
    assert!(cli.is_ok(), "should parse view history: {:?}", cli.err());
}

// ════════════════════════════════════════════════════════════════════
// Subnet commands (untested)
// ════════════════════════════════════════════════════════════════════

#[test]
fn parse_subnet_show() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "show", "--netuid", "1"]);
    assert!(cli.is_ok(), "should parse subnet show: {:?}", cli.err());
}

#[test]
fn parse_subnet_hyperparams() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "hyperparams", "--netuid", "1"]);
    assert!(
        cli.is_ok(),
        "should parse subnet hyperparams: {:?}",
        cli.err()
    );
}

#[test]
fn parse_subnet_register() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "register"]);
    assert!(cli.is_ok(), "should parse subnet register: {:?}", cli.err());
}

#[test]
fn parse_subnet_register_neuron() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "subnet", "register-neuron", "--netuid", "1"]);
    assert!(
        cli.is_ok(),
        "should parse subnet register-neuron: {:?}",
        cli.err()
    );
}

#[test]
fn parse_subnet_pow() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "pow",
        "--netuid",
        "1",
        "--threads",
        "8",
    ]);
    assert!(cli.is_ok(), "should parse subnet pow: {:?}", cli.err());
}

// ════════════════════════════════════════════════════════════════════
// Config commands (untested set/unset/path)
// ════════════════════════════════════════════════════════════════════

#[test]
fn parse_config_set() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "config", "set", "--key", "network", "--value", "test",
    ]);
    assert!(cli.is_ok(), "should parse config set: {:?}", cli.err());
}

#[test]
fn parse_config_unset() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "config", "unset", "--key", "network"]);
    assert!(cli.is_ok(), "should parse config unset: {:?}", cli.err());
}

#[test]
fn parse_config_path() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "config", "path"]);
    assert!(cli.is_ok(), "should parse config path: {:?}", cli.err());
}

// ════════════════════════════════════════════════════════════════════
// Root, TransferAll, Update, Subscribe blocks
// ════════════════════════════════════════════════════════════════════

#[test]
fn parse_root_register() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "root", "register"]);
    assert!(cli.is_ok(), "should parse root register: {:?}", cli.err());
}

#[test]
fn parse_transfer_all() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "transfer-all",
        "--dest",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--keep-alive",
    ]);
    assert!(cli.is_ok(), "should parse transfer-all: {:?}", cli.err());
}

#[test]
fn parse_update() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "update"]);
    assert!(cli.is_ok(), "should parse update: {:?}", cli.err());
}

#[test]
fn parse_subscribe_blocks() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subscribe", "blocks"]);
    assert!(
        cli.is_ok(),
        "should parse subscribe blocks: {:?}",
        cli.err()
    );
}

#[test]
fn parse_subscribe_events_with_filters() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subscribe",
        "events",
        "--filter",
        "staking",
        "--netuid",
        "1",
        "--account",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(
        cli.is_ok(),
        "should parse subscribe events with filters: {:?}",
        cli.err()
    );
}

// ════════════════════════════════════════════════════════════════════
// Wallet regen commands
// ════════════════════════════════════════════════════════════════════

#[test]
fn parse_wallet_regen_coldkey() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "regen-coldkey",
        "--mnemonic", "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        "--password", "secret",
    ]);
    assert!(
        cli.is_ok(),
        "should parse wallet regen-coldkey: {:?}",
        cli.err()
    );
}

#[test]
fn parse_wallet_regen_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "regen-hotkey", "--name", "hk1",
        "--mnemonic", "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
    ]);
    assert!(
        cli.is_ok(),
        "should parse wallet regen-hotkey: {:?}",
        cli.err()
    );
}

// ── Audit command tests ──

#[test]
fn parse_audit_default() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "audit"]);
    assert!(cli.is_ok(), "should parse audit: {:?}", cli.err());
}

#[test]
fn parse_audit_with_address() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "audit",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(
        cli.is_ok(),
        "should parse audit with address: {:?}",
        cli.err()
    );
}

#[test]
fn parse_audit_json_output() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--output",
        "json",
        "audit",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(
        cli.is_ok(),
        "should parse audit with json output: {:?}",
        cli.err()
    );
}

// ── At-block wayback tests ──

#[test]
fn parse_balance_at_block() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "balance",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--at-block",
        "1000000",
    ]);
    assert!(
        cli.is_ok(),
        "should parse balance --at-block: {:?}",
        cli.err()
    );
}

#[test]
fn parse_balance_at_block_json() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--output",
        "json",
        "balance",
        "--at-block",
        "5000000",
    ]);
    assert!(
        cli.is_ok(),
        "should parse balance --at-block json: {:?}",
        cli.err()
    );
}

#[test]
fn parse_view_network_at_block() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "view", "network", "--at-block", "3000000"]);
    assert!(
        cli.is_ok(),
        "should parse view network --at-block: {:?}",
        cli.err()
    );
}

#[test]
fn parse_view_network_at_block_json() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--output",
        "json",
        "view",
        "network",
        "--at-block",
        "1",
    ]);
    assert!(
        cli.is_ok(),
        "should parse view network --at-block json: {:?}",
        cli.err()
    );
}

// ──── View Account --at-block (Step 29) ────

#[test]
fn parse_view_account_at_block() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "view",
        "account",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--at-block",
        "7000000",
    ]);
    assert!(
        cli.is_ok(),
        "should parse view account --at-block: {:?}",
        cli.err()
    );
    if let agcli::cli::Commands::View(agcli::cli::ViewCommands::Account {
        at_block, address, ..
    }) = &cli.unwrap().command
    {
        assert_eq!(*at_block, Some(7000000));
        assert!(address.is_some());
    } else {
        panic!("wrong command variant");
    }
}

#[test]
fn parse_view_account_at_block_json() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--output",
        "json",
        "view",
        "account",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--at-block",
        "100",
    ]);
    assert!(
        cli.is_ok(),
        "should parse view account --at-block json: {:?}",
        cli.err()
    );
}

#[test]
fn parse_view_account_without_at_block() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "view",
        "account",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok());
    if let agcli::cli::Commands::View(agcli::cli::ViewCommands::Account { at_block, .. }) =
        &cli.unwrap().command
    {
        assert_eq!(*at_block, None);
    } else {
        panic!("wrong command variant");
    }
}

// ──── Stake List --at-block (Step 29) ────

#[test]
fn parse_stake_list_at_block() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "list",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--at-block",
        "7000000",
    ]);
    assert!(
        cli.is_ok(),
        "should parse stake list --at-block: {:?}",
        cli.err()
    );
    if let agcli::cli::Commands::Stake(agcli::cli::StakeCommands::List { at_block, address }) =
        &cli.unwrap().command
    {
        assert_eq!(*at_block, Some(7000000));
        assert!(address.is_some());
    } else {
        panic!("wrong command variant");
    }
}

#[test]
fn parse_stake_list_at_block_json() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--output",
        "json",
        "stake",
        "list",
        "--at-block",
        "500",
    ]);
    assert!(
        cli.is_ok(),
        "should parse stake list --at-block json: {:?}",
        cli.err()
    );
}

#[test]
fn parse_stake_list_without_at_block() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "stake", "list"]);
    assert!(cli.is_ok());
    if let agcli::cli::Commands::Stake(agcli::cli::StakeCommands::List { at_block, .. }) =
        &cli.unwrap().command
    {
        assert_eq!(*at_block, None);
    } else {
        panic!("wrong command variant");
    }
}

// ──── Audit enhancements (Step 29 — coldkey swap + childkey) ────

#[test]
fn parse_audit_with_json_output_checks_fields() {
    // Ensure the audit command still parses with --output json
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--output",
        "json",
        "audit",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok());
    let parsed = cli.unwrap();
    assert_eq!(parsed.output, OutputFormat::Json);
    if let agcli::cli::Commands::Audit { address } = &parsed.command {
        assert_eq!(
            address.as_deref(),
            Some("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")
        );
    } else {
        panic!("wrong command variant");
    }
}

// ──── Step 30 — explain coldkey-swap + pending childkeys ────

#[test]
fn parse_explain_coldkey_swap() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "explain", "--topic", "coldkey-swap"]);
    assert!(cli.is_ok());
    if let agcli::cli::Commands::Explain { topic, .. } = &cli.unwrap().command {
        assert_eq!(topic.as_deref(), Some("coldkey-swap"));
    } else {
        panic!("wrong command variant (expected Explain)");
    }
}

#[test]
fn parse_explain_coldkey_alias_ckswap() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "explain", "--topic", "ckswap"]);
    assert!(cli.is_ok());
    if let agcli::cli::Commands::Explain { topic, .. } = &cli.unwrap().command {
        assert_eq!(topic.as_deref(), Some("ckswap"));
    } else {
        panic!("wrong command variant (expected Explain for ckswap)");
    }
}

// ──── Step 32 — explain governance, senate, mev-shield ────

#[test]
fn parse_explain_governance() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "explain", "--topic", "governance"]);
    assert!(cli.is_ok());
}

#[test]
fn parse_explain_senate() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "explain", "--topic", "senate"]);
    assert!(cli.is_ok());
}

#[test]
fn parse_explain_mev_shield() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "explain", "--topic", "mev-shield"]);
    assert!(cli.is_ok());
}

// ──── Step 33 — explain limits, hyperparams, axon ────

#[test]
fn parse_explain_limits() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "explain", "--topic", "limits"]);
    assert!(cli.is_ok());
}

#[test]
fn parse_explain_hyperparams() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "explain", "--topic", "hyperparams"]);
    assert!(cli.is_ok());
}

#[test]
fn parse_explain_axon() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "explain", "--topic", "axon"]);
    assert!(cli.is_ok());
}

#[test]
fn parse_explain_take() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "explain", "--topic", "take"]);
    assert!(cli.is_ok());
}

#[test]
fn parse_explain_recycle() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "explain", "--topic", "recycle"]);
    assert!(cli.is_ok());
}

#[test]
fn parse_explain_pow() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "explain", "--topic", "pow"]);
    assert!(cli.is_ok());
}

// ──── Step 34 — archive & block commands ────

#[test]
fn parse_network_archive() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--network", "archive", "balance"]);
    assert!(cli.is_ok());
    assert_eq!(cli.unwrap().network, "archive");
}

#[test]
fn parse_subnet_list_at_block() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "list", "--at-block", "5000000"]);
    assert!(cli.is_ok());
}

#[test]
fn parse_subnet_show_at_block() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "show",
        "--netuid",
        "1",
        "--at-block",
        "5000000",
    ]);
    assert!(cli.is_ok());
}

#[test]
fn parse_subnet_metagraph_at_block() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "metagraph",
        "--netuid",
        "1",
        "--at-block",
        "5000000",
    ]);
    assert!(cli.is_ok());
}

#[test]
fn parse_view_portfolio_at_block() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "view", "portfolio", "--at-block", "5000000"]);
    assert!(cli.is_ok());
}

#[test]
fn parse_view_dynamic_at_block() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "view", "dynamic", "--at-block", "5000000"]);
    assert!(cli.is_ok());
}

#[test]
fn parse_view_neuron_at_block() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "view",
        "neuron",
        "--netuid",
        "1",
        "--uid",
        "0",
        "--at-block",
        "5000000",
    ]);
    assert!(cli.is_ok());
}

#[test]
fn parse_view_validators_at_block() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "view", "validators", "--at-block", "5000000"]);
    assert!(cli.is_ok());
}

#[test]
fn parse_block_latest() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "block", "latest"]);
    assert!(cli.is_ok());
}

#[test]
fn parse_block_info() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "block", "info", "--number", "5000000"]);
    assert!(cli.is_ok());
}

#[test]
fn parse_explain_archive() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "explain", "--topic", "archive"]);
    assert!(cli.is_ok());
}

#[test]
fn resolve_network_archive() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--network", "archive", "balance"]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    let network = cli.resolve_network();
    assert!(matches!(network, agcli::types::network::Network::Archive));
    assert!(network.ws_url().starts_with("wss://"));
}

// ──────── Block Range Tests ────────

#[test]
fn parse_block_range() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "block", "range", "--from", "100", "--to", "110",
    ]);
    assert!(cli.is_ok());
}

#[test]
fn parse_block_range_missing_from() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "block", "range", "--to", "110"]);
    assert!(cli.is_err());
}

#[test]
fn parse_block_range_missing_to() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "block", "range", "--from", "100"]);
    assert!(cli.is_err());
}

// ──────── Diff Tests ────────

#[test]
fn parse_diff_portfolio() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "diff",
        "portfolio",
        "--address",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--block1",
        "4000000",
        "--block2",
        "5000000",
    ]);
    assert!(cli.is_ok());
}

#[test]
fn parse_diff_portfolio_no_address() {
    // Should parse OK — address is optional, will fall back to wallet
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "diff",
        "portfolio",
        "--block1",
        "4000000",
        "--block2",
        "5000000",
    ]);
    assert!(cli.is_ok());
}

#[test]
fn parse_diff_subnet() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "diff", "subnet", "--netuid", "1", "--block1", "4000000", "--block2", "5000000",
    ]);
    assert!(cli.is_ok());
}

#[test]
fn parse_diff_subnet_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "diff", "subnet", "--block1", "4000000", "--block2", "5000000",
    ]);
    assert!(cli.is_err());
}

#[test]
fn parse_diff_network() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "diff", "network", "--block1", "4000000", "--block2", "5000000",
    ]);
    assert!(cli.is_ok());
}

#[test]
fn parse_diff_network_missing_blocks() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "diff", "network", "--block1", "4000000"]);
    assert!(cli.is_err());
}

#[test]
fn parse_diff_with_archive_network() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--network",
        "archive",
        "diff",
        "network",
        "--block1",
        "1000000",
        "--block2",
        "2000000",
    ]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    assert!(matches!(
        cli.resolve_network(),
        agcli::types::network::Network::Archive
    ));
}

#[test]
fn parse_diff_with_json_output() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--output",
        "json",
        "diff",
        "portfolio",
        "--address",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--block1",
        "4000000",
        "--block2",
        "5000000",
    ]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    assert_eq!(cli.output, OutputFormat::Json);
}

#[test]
fn parse_explain_diff() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "explain", "--topic", "diff"]);
    assert!(cli.is_ok());
}

#[test]
fn explain_diff_topic_exists() {
    let text = agcli::utils::explain::explain("diff");
    assert!(text.is_some());
    assert!(text.unwrap().contains("HISTORICAL DIFF"));
}

#[test]
fn explain_diff_via_compare_alias() {
    let text = agcli::utils::explain::explain("compare");
    assert!(text.is_some());
}

#[test]
fn parse_subnet_commits() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "commits", "--netuid", "1"]);
    assert!(cli.is_ok(), "should parse subnet commits: {:?}", cli.err());
}

#[test]
fn parse_subnet_commits_with_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "commits",
        "--netuid",
        "1",
        "--hotkey",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
    ]);
    assert!(
        cli.is_ok(),
        "should parse subnet commits with hotkey: {:?}",
        cli.err()
    );
}

#[test]
fn parse_weights_status() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "weights", "status", "--netuid", "1"]);
    assert!(cli.is_ok(), "should parse weights status: {:?}", cli.err());
}

// ──── Sprint 4: --timeout and --time flags ────

#[test]
fn parse_global_timeout_flag() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--timeout", "30", "balance"]);
    assert!(cli.is_ok(), "should parse --timeout: {:?}", cli.err());
    assert_eq!(cli.unwrap().timeout, Some(30));
}

#[test]
fn parse_global_timeout_zero_means_no_timeout() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--timeout", "0", "balance"]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    assert_eq!(cli.timeout, Some(0));
    // 0 is treated as "no timeout" in main.rs via .filter(|&t| t > 0)
}

#[test]
fn parse_global_timeout_absent() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "balance"]);
    assert!(cli.is_ok());
    assert_eq!(cli.unwrap().timeout, None);
}

#[test]
fn parse_global_time_flag() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--time", "balance"]);
    assert!(cli.is_ok(), "should parse --time: {:?}", cli.err());
    assert!(cli.unwrap().time);
}

#[test]
fn parse_time_flag_absent_defaults_false() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "balance"]);
    assert!(cli.is_ok());
    assert!(!cli.unwrap().time);
}

#[test]
fn parse_timeout_with_subcommand() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--timeout", "60", "subnet", "list"]);
    assert!(
        cli.is_ok(),
        "should parse --timeout with subnet list: {:?}",
        cli.err()
    );
    assert_eq!(cli.unwrap().timeout, Some(60));
}

#[test]
fn parse_time_and_timeout_together() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--time", "--timeout", "120", "balance"]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    assert!(cli.time);
    assert_eq!(cli.timeout, Some(120));
}

// ──── Sprint 5: Liquidity commands ────

#[test]
fn parse_liquidity_add() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "liquidity",
        "add",
        "--netuid",
        "1",
        "--price-low",
        "0.5",
        "--price-high",
        "2.0",
        "--amount",
        "1000000",
    ]);
    assert!(cli.is_ok(), "should parse liquidity add: {:?}", cli.err());
}

#[test]
fn parse_liquidity_add_with_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "liquidity",
        "add",
        "--netuid",
        "1",
        "--price-low",
        "0.1",
        "--price-high",
        "10.0",
        "--amount",
        "500000",
        "--hotkey",
        "5GhostHotkey",
    ]);
    assert!(
        cli.is_ok(),
        "should parse liquidity add with hotkey: {:?}",
        cli.err()
    );
}

#[test]
fn parse_liquidity_remove() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "liquidity",
        "remove",
        "--netuid",
        "1",
        "--position-id",
        "42",
    ]);
    assert!(
        cli.is_ok(),
        "should parse liquidity remove: {:?}",
        cli.err()
    );
}

#[test]
fn parse_liquidity_modify() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "liquidity",
        "modify",
        "--netuid",
        "1",
        "--position-id",
        "42",
        "--delta=-500",
    ]);
    assert!(
        cli.is_ok(),
        "should parse liquidity modify: {:?}",
        cli.err()
    );
}

#[test]
fn parse_liquidity_modify_positive() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "liquidity",
        "modify",
        "--netuid",
        "1",
        "--position-id",
        "42",
        "--delta",
        "1000",
    ]);
    assert!(
        cli.is_ok(),
        "should parse liquidity modify positive: {:?}",
        cli.err()
    );
}

#[test]
fn parse_liquidity_toggle() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "liquidity",
        "toggle",
        "--netuid",
        "1",
        "--enable",
    ]);
    assert!(
        cli.is_ok(),
        "should parse liquidity toggle: {:?}",
        cli.err()
    );
}

// ──── Sprint 5: Auto-stake ────

#[test]
fn parse_stake_set_auto() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "stake", "set-auto", "--netuid", "1"]);
    assert!(cli.is_ok(), "should parse stake set-auto: {:?}", cli.err());
}

#[test]
fn parse_stake_set_auto_with_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "set-auto",
        "--netuid",
        "1",
        "--hotkey",
        "5GhostHotkey",
    ]);
    assert!(
        cli.is_ok(),
        "should parse stake set-auto with hotkey: {:?}",
        cli.err()
    );
}

// ──── Sprint 5: Root claim ────

#[test]
fn parse_stake_set_claim_swap() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "stake", "set-claim", "--claim-type", "swap"]);
    assert!(
        cli.is_ok(),
        "should parse stake set-claim swap: {:?}",
        cli.err()
    );
}

#[test]
fn parse_stake_set_claim_keep() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "stake", "set-claim", "--claim-type", "keep"]);
    assert!(
        cli.is_ok(),
        "should parse stake set-claim keep: {:?}",
        cli.err()
    );
}

#[test]
fn parse_stake_set_claim_keep_subnets() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "set-claim",
        "--claim-type",
        "keep-subnets",
        "--subnets",
        "1,3,5",
    ]);
    assert!(
        cli.is_ok(),
        "should parse stake set-claim keep-subnets: {:?}",
        cli.err()
    );
}

// ──── Sprint 5: Crowdloan expanded commands ────

#[test]
fn parse_crowdloan_create() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "crowdloan",
        "create",
        "--deposit",
        "10.0",
        "--min-contribution",
        "0.1",
        "--cap",
        "1000.0",
        "--end-block",
        "5000000",
    ]);
    assert!(
        cli.is_ok(),
        "should parse crowdloan create: {:?}",
        cli.err()
    );
}

#[test]
fn parse_crowdloan_create_with_target() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "crowdloan",
        "create",
        "--deposit",
        "10.0",
        "--min-contribution",
        "0.1",
        "--cap",
        "1000.0",
        "--end-block",
        "5000000",
        "--target",
        "5GhostTarget",
    ]);
    assert!(
        cli.is_ok(),
        "should parse crowdloan create with target: {:?}",
        cli.err()
    );
}

#[test]
fn parse_crowdloan_refund() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "crowdloan", "refund", "--crowdloan-id", "42"]);
    assert!(
        cli.is_ok(),
        "should parse crowdloan refund: {:?}",
        cli.err()
    );
}

#[test]
fn parse_crowdloan_dissolve() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "crowdloan", "dissolve", "--crowdloan-id", "42"]);
    assert!(
        cli.is_ok(),
        "should parse crowdloan dissolve: {:?}",
        cli.err()
    );
}

#[test]
fn parse_crowdloan_update_cap() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "crowdloan",
        "update-cap",
        "--crowdloan-id",
        "42",
        "--cap",
        "2000.0",
    ]);
    assert!(
        cli.is_ok(),
        "should parse crowdloan update-cap: {:?}",
        cli.err()
    );
}

#[test]
fn parse_crowdloan_update_end() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "crowdloan",
        "update-end",
        "--crowdloan-id",
        "42",
        "--end-block",
        "6000000",
    ]);
    assert!(
        cli.is_ok(),
        "should parse crowdloan update-end: {:?}",
        cli.err()
    );
}

#[test]
fn parse_crowdloan_update_min_contribution() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "crowdloan",
        "update-min-contribution",
        "--crowdloan-id",
        "42",
        "--min-contribution",
        "0.5",
    ]);
    assert!(
        cli.is_ok(),
        "should parse crowdloan update-min-contribution: {:?}",
        cli.err()
    );
}

// ──── Sprint 6: --mev flag tests ────

#[test]
fn parse_mev_flag_global() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--mev", "balance"]);
    assert!(cli.is_ok(), "should parse --mev flag: {:?}", cli.err());
    assert!(cli.unwrap().mev);
}

#[test]
fn parse_mev_flag_with_stake_add() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--mev", "stake", "add", "--amount", "1.0", "--netuid", "1",
    ]);
    assert!(
        cli.is_ok(),
        "should parse --mev with stake add: {:?}",
        cli.err()
    );
    let cli = cli.unwrap();
    assert!(cli.mev);
}

#[test]
fn parse_mev_flag_with_stake_remove() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--mev", "stake", "remove", "--amount", "1.0", "--netuid", "1",
    ]);
    assert!(
        cli.is_ok(),
        "should parse --mev with stake remove: {:?}",
        cli.err()
    );
    assert!(cli.unwrap().mev);
}

#[test]
fn parse_mev_flag_default_false() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "balance"]);
    assert!(cli.is_ok());
    assert!(!cli.unwrap().mev, "mev should default to false");
}

#[test]
fn parse_mev_combined_with_other_flags() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--mev",
        "--yes",
        "--verbose",
        "--time",
        "stake",
        "add",
        "--amount",
        "1.0",
        "--netuid",
        "1",
    ]);
    assert!(
        cli.is_ok(),
        "should parse multiple flags together: {:?}",
        cli.err()
    );
    let cli = cli.unwrap();
    assert!(cli.mev);
    assert!(cli.yes);
    assert!(cli.verbose);
    assert!(cli.time);
}

// ──── Sprint 6: error message quality tests ────

#[test]
fn parse_stake_add_missing_netuid_error() {
    let result = agcli::cli::Cli::try_parse_from(["agcli", "stake", "add", "--amount", "1.0"]);
    assert!(result.is_err(), "missing --netuid should error");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("netuid"),
        "error should mention netuid: {}",
        err
    );
}

#[test]
fn parse_stake_add_missing_amount_error() {
    let result = agcli::cli::Cli::try_parse_from(["agcli", "stake", "add", "--netuid", "1"]);
    assert!(result.is_err(), "missing --amount should error");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("amount"),
        "error should mention amount: {}",
        err
    );
}

#[test]
fn parse_transfer_missing_dest_error() {
    let result = agcli::cli::Cli::try_parse_from(["agcli", "transfer", "--amount", "1.0"]);
    assert!(result.is_err(), "missing --dest should error");
}

#[test]
fn parse_transfer_missing_amount_error() {
    let result = agcli::cli::Cli::try_parse_from([
        "agcli",
        "transfer",
        "--dest",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(result.is_err(), "missing --amount should error");
}

#[test]
fn parse_subnet_metagraph_missing_netuid_error() {
    let result = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "metagraph"]);
    assert!(result.is_err(), "missing --netuid should error");
}

#[test]
fn parse_invalid_network_string_still_parses() {
    // Custom/unknown network strings are accepted and treated as Custom
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "--network", "ws://custom:9944", "balance"]);
    assert!(cli.is_ok(), "custom network string should parse");
}

#[test]
fn parse_timeout_zero_is_valid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--timeout", "0", "balance"]);
    assert!(cli.is_ok());
    assert_eq!(cli.unwrap().timeout, Some(0));
}

#[test]
fn parse_timeout_large_value() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--timeout", "3600", "balance"]);
    assert!(cli.is_ok());
    assert_eq!(cli.unwrap().timeout, Some(3600));
}

// ──────── subnet set-param ────────

#[test]
fn parse_subnet_set_param() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "set-param",
        "--netuid",
        "1",
        "--param",
        "tempo",
        "--value",
        "100",
    ]);
    assert!(
        cli.is_ok(),
        "subnet set-param should parse: {:?}",
        cli.err()
    );
}

#[test]
fn parse_subnet_set_param_bool_value() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "set-param",
        "--netuid",
        "5",
        "--param",
        "registration_allowed",
        "--value",
        "false",
    ]);
    assert!(
        cli.is_ok(),
        "subnet set-param bool should parse: {:?}",
        cli.err()
    );
}

#[test]
fn parse_subnet_set_param_list_mode() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "set-param",
        "--netuid",
        "1",
        "--param",
        "list",
    ]);
    assert!(
        cli.is_ok(),
        "subnet set-param list should parse: {:?}",
        cli.err()
    );
}

#[test]
fn parse_subnet_set_param_requires_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "set-param",
        "--param",
        "tempo",
        "--value",
        "100",
    ]);
    assert!(cli.is_err(), "subnet set-param should require --netuid");
}

#[test]
fn parse_subnet_set_param_requires_param() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "set-param",
        "--netuid",
        "1",
        "--value",
        "100",
    ]);
    assert!(cli.is_err(), "subnet set-param should require --param");
}

#[test]
fn parse_subnet_set_param_value_is_optional() {
    // --value is optional (for list mode)
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "set-param",
        "--netuid",
        "1",
        "--param",
        "tempo",
    ]);
    assert!(cli.is_ok(), "subnet set-param without --value should parse");
}

// ──── Sprint 11: transfer-stake CLI ────

#[test]
fn parse_stake_transfer_stake() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "transfer-stake",
        "--dest",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--amount",
        "10.5",
        "--from",
        "1",
        "--to",
        "2",
        "--hotkey",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(
        cli.is_ok(),
        "should parse stake transfer-stake: {:?}",
        cli.err()
    );
}

#[test]
fn parse_stake_transfer_stake_requires_dest() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "transfer-stake",
        "--amount",
        "10",
        "--from",
        "1",
        "--to",
        "2",
    ]);
    assert!(cli.is_err(), "transfer-stake should require --dest");
}

#[test]
fn parse_stake_transfer_stake_requires_amount() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "transfer-stake",
        "--dest",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--from",
        "1",
        "--to",
        "2",
    ]);
    assert!(cli.is_err(), "transfer-stake should require --amount");
}

// ──────── Sprint 20: New Commands ────────

#[test]
fn parse_subnet_trim() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "trim",
        "--netuid",
        "1",
        "--max-uids",
        "256",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_subnet_check_start() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "check-start", "--netuid", "1"]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_subnet_start() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "start", "--netuid", "1"]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_subnet_mechanism_count() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "subnet", "mechanism-count", "--netuid", "1"]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_subnet_set_mechanism_count() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "set-mechanism-count",
        "--netuid",
        "1",
        "--count",
        "2",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_subnet_set_emission_split() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "set-emission-split",
        "--netuid",
        "1",
        "--weights",
        "50,50",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_serve_reset() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "serve", "reset", "--netuid", "1"]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_stake_process_claim() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "stake", "process-claim"]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_stake_process_claim_with_netuids() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "stake", "process-claim", "--netuids", "1,2,3"]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_utils_convert_to_rao() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "utils", "convert", "--amount", "1.5", "--to-rao",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_utils_convert_to_tao() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "utils", "convert", "--amount", "1500000000"]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_utils_latency() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "utils", "latency"]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_utils_latency_with_extra() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "utils",
        "latency",
        "--extra",
        "wss://custom.node:9944",
        "--pings",
        "3",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

// ──── Sprint 26 — explain --full, block pinning, multi-process safety ────

#[test]
fn parse_explain_full_flag() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "explain", "--topic", "stake", "--full"]);
    assert!(cli.is_ok(), "{:?}", cli.err());
    if let agcli::cli::Commands::Explain { topic, full } = &cli.unwrap().command {
        assert_eq!(topic.as_deref(), Some("stake"));
        assert!(full);
    } else {
        panic!("wrong command variant");
    }
}

#[test]
fn parse_explain_full_flag_defaults_false() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "explain", "--topic", "tempo"]);
    assert!(cli.is_ok(), "{:?}", cli.err());
    if let agcli::cli::Commands::Explain { full, .. } = &cli.unwrap().command {
        assert!(!full);
    } else {
        panic!("wrong command variant");
    }
}

#[test]
fn parse_explain_full_no_topic() {
    // --full without --topic should parse fine (lists all doc files)
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "explain", "--full"]);
    assert!(cli.is_ok(), "{:?}", cli.err());
    if let agcli::cli::Commands::Explain { topic, full } = &cli.unwrap().command {
        assert!(topic.is_none());
        assert!(full);
    } else {
        panic!("wrong command variant");
    }
}

#[test]
fn explain_full_loads_doc_file() {
    // Run from the repo root so docs/commands/ is found
    let result = agcli::utils::explain::explain("stake");
    assert!(result.is_some(), "built-in explain should find 'stake'");
}

#[test]
fn explain_all_topics_have_content() {
    // Every topic in list_topics() should resolve to Some
    for (key, _desc) in agcli::utils::explain::list_topics() {
        let content = agcli::utils::explain::explain(key);
        assert!(content.is_some(), "explain('{}') returned None", key);
        assert!(
            !content.unwrap().is_empty(),
            "explain('{}') returned empty string",
            key
        );
    }
}

#[test]
fn explain_topic_descriptions_unique() {
    // No two topics should share the same description
    let topics = agcli::utils::explain::list_topics();
    let mut descs: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for (_key, desc) in &topics {
        assert!(descs.insert(desc), "duplicate description: '{}'", desc);
    }
}

#[test]
fn explain_fuzzy_matching_works() {
    // Substring matching: "cold" should match "coldkey-swap"
    let result = agcli::utils::explain::explain("cold");
    assert!(
        result.is_some(),
        "fuzzy match for 'cold' should find coldkey-swap"
    );
}

#[test]
fn explain_normalization_strips_hyphens_underscores() {
    // "commit-reveal" and "commit_reveal" should both resolve
    let r1 = agcli::utils::explain::explain("commit-reveal");
    let r2 = agcli::utils::explain::explain("commit_reveal");
    assert!(r1.is_some());
    assert!(r2.is_some());
    assert_eq!(r1.unwrap(), r2.unwrap());
}

// ──────── New Feature CLI Parsing Tests ────────

#[test]
fn parse_weights_show() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "weights", "show", "--netuid", "97"]);
    assert!(cli.is_ok(), "weights show: {:?}", cli.err());
}


#[test]
fn parse_view_metagraph() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "metagraph", "--netuid", "97"]);
    assert!(cli.is_ok(), "view metagraph: {:?}", cli.err());
}

#[test]
fn parse_view_metagraph_with_since_block() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "view",
        "metagraph",
        "--netuid",
        "97",
        "--since-block",
        "1000000",
    ]);
    assert!(cli.is_ok(), "view metagraph --since-block: {:?}", cli.err());
}

#[test]
fn parse_view_axon_by_uid() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "view", "axon", "--netuid", "97", "--uid", "42"]);
    assert!(cli.is_ok(), "view axon --uid: {:?}", cli.err());
}

#[test]
fn parse_view_health() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "health", "--netuid", "97"]);
    assert!(cli.is_ok(), "view health: {:?}", cli.err());
}

#[test]
fn parse_view_health_with_tcp_check() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "view",
        "health",
        "--netuid",
        "97",
        "--tcp-check",
        "--probe-timeout-ms",
        "5000",
    ]);
    assert!(cli.is_ok(), "view health --tcp-check: {:?}", cli.err());
}

#[test]
fn parse_view_emissions() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "view",
        "emissions",
        "--netuid",
        "97",
        "--limit",
        "20",
    ]);
    assert!(cli.is_ok(), "view emissions: {:?}", cli.err());
}

#[test]
fn parse_serve_batch_axon() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "serve",
        "batch-axon",
        "--file",
        "/tmp/axons.json",
    ]);
    assert!(cli.is_ok(), "serve batch-axon: {:?}", cli.err());
}

#[test]
fn parse_diff_metagraph() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "diff",
        "metagraph",
        "--netuid",
        "97",
        "--block1",
        "1000000",
        "--block2",
        "1000100",
    ]);
    assert!(cli.is_ok(), "diff metagraph: {:?}", cli.err());
}

// ──── Commitment Commands ────

#[test]
fn parse_commitment_set() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "commitment",
        "set",
        "--netuid",
        "97",
        "--data",
        "endpoint:http://1.2.3.4:8091,version:1.0",
    ]);
    assert!(cli.is_ok(), "commitment set: {:?}", cli.err());
}

#[test]
fn parse_commitment_get() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--output",
        "json",
        "commitment",
        "get",
        "--netuid",
        "97",
        "--hotkey",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "commitment get: {:?}", cli.err());
}

#[test]
fn parse_commitment_list() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--output",
        "json",
        "commitment",
        "list",
        "--netuid",
        "97",
    ]);
    assert!(cli.is_ok(), "commitment list: {:?}", cli.err());
}

// ──── Utils Convert Alpha/TAO ────

#[test]
fn parse_utils_convert_tao_to_alpha() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "utils", "convert", "--tao", "10.0", "--netuid", "1",
    ]);
    assert!(cli.is_ok(), "utils convert --tao: {:?}", cli.err());
}

#[test]
fn parse_utils_convert_alpha_to_tao() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "utils", "convert", "--alpha", "500.0", "--netuid", "1",
    ]);
    assert!(cli.is_ok(), "utils convert --alpha: {:?}", cli.err());
}

#[test]
fn parse_utils_convert_tao_to_rao() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "utils", "convert", "--amount", "10.0", "--to-rao",
    ]);
    assert!(cli.is_ok(), "utils convert --to-rao: {:?}", cli.err());
}

// ──── Snipe ────

#[test]
fn parse_subnet_snipe_basic() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "snipe", "--netuid", "97"]);
    assert!(cli.is_ok(), "subnet snipe: {:?}", cli.err());
}

#[test]
fn parse_subnet_snipe_with_max_cost() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "snipe",
        "--netuid",
        "97",
        "--max-cost",
        "1.5",
    ]);
    assert!(cli.is_ok(), "subnet snipe --max-cost: {:?}", cli.err());
}

#[test]
fn parse_subnet_snipe_with_max_attempts() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "snipe",
        "--netuid",
        "1",
        "--max-attempts",
        "100",
    ]);
    assert!(cli.is_ok(), "subnet snipe --max-attempts: {:?}", cli.err());
}

#[test]
fn parse_subnet_snipe_all_opts() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--password",
        "test",
        "--batch",
        "subnet",
        "snipe",
        "--netuid",
        "97",
        "--max-cost",
        "0.5",
        "--max-attempts",
        "50",
    ]);
    assert!(cli.is_ok(), "subnet snipe all opts: {:?}", cli.err());
}

#[test]
fn parse_subnet_snipe_fast_mode() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "subnet", "snipe", "--netuid", "97", "--fast"]);
    assert!(cli.is_ok(), "subnet snipe --fast: {:?}", cli.err());
}

#[test]
fn parse_subnet_snipe_watch_mode() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "subnet", "snipe", "--netuid", "1", "--watch"]);
    assert!(cli.is_ok(), "subnet snipe --watch: {:?}", cli.err());
}

#[test]
fn parse_subnet_snipe_watch_with_max_cost() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "snipe",
        "--netuid",
        "1",
        "--watch",
        "--max-cost",
        "2.0",
    ]);
    assert!(
        cli.is_ok(),
        "subnet snipe --watch --max-cost: {:?}",
        cli.err()
    );
}

#[test]
fn parse_subnet_snipe_all_hotkeys() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--password",
        "test",
        "subnet",
        "snipe",
        "--netuid",
        "97",
        "--all-hotkeys",
    ]);
    assert!(cli.is_ok(), "subnet snipe --all-hotkeys: {:?}", cli.err());
}

#[test]
fn parse_subnet_snipe_fast_with_all_opts() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--password",
        "test",
        "--batch",
        "subnet",
        "snipe",
        "--netuid",
        "97",
        "--fast",
        "--max-cost",
        "1.0",
        "--max-attempts",
        "25",
        "--all-hotkeys",
    ]);
    assert!(cli.is_ok(), "subnet snipe full combo: {:?}", cli.err());
}

// ──── Comprehensive stake CLI arg edge case tests ────

// ── stake add edge cases ──

#[test]
fn parse_stake_add_with_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "add", "--amount", "1.5", "--netuid", "1",
        "--hotkey", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "stake add with hotkey: {:?}", cli.err());
}

#[test]
fn parse_stake_add_with_all_flags() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--yes", "--password", "pw", "--mev",
        "stake", "add", "--amount", "10.0", "--netuid", "42",
        "--hotkey", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--max-slippage", "5.0",
    ]);
    assert!(cli.is_ok(), "stake add full combo: {:?}", cli.err());
    let cli = cli.unwrap();
    assert!(cli.yes);
    assert!(cli.mev);
}

#[test]
fn parse_stake_add_zero_amount() {
    // 0 amount should parse (chain may reject, but CLI accepts)
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "add", "--amount", "0.0", "--netuid", "1",
    ]);
    assert!(cli.is_ok(), "zero amount should parse: {:?}", cli.err());
}

#[test]
fn parse_stake_add_tiny_amount() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "add", "--amount", "0.000000001", "--netuid", "1",
    ]);
    assert!(cli.is_ok(), "tiny amount (1 RAO): {:?}", cli.err());
}

#[test]
fn parse_stake_add_large_amount() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "add", "--amount", "1000000.0", "--netuid", "1",
    ]);
    assert!(cli.is_ok(), "large amount: {:?}", cli.err());
}

#[test]
fn parse_stake_add_netuid_zero() {
    // Root network is netuid 0
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "add", "--amount", "1.0", "--netuid", "0",
    ]);
    assert!(cli.is_ok(), "netuid 0 (root): {:?}", cli.err());
}

#[test]
fn parse_stake_add_netuid_max() {
    // Max u16 netuid
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "add", "--amount", "1.0", "--netuid", "65535",
    ]);
    assert!(cli.is_ok(), "netuid 65535: {:?}", cli.err());
}

#[test]
fn parse_stake_add_netuid_overflow() {
    // Beyond u16 should fail
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "add", "--amount", "1.0", "--netuid", "65536",
    ]);
    assert!(cli.is_err(), "netuid 65536 should overflow u16");
}

#[test]
fn parse_stake_add_negative_amount() {
    // Clap treats -1.0 as an unknown argument (the dash is ambiguous),
    // so negative amounts are rejected at the CLI parsing level — good UX.
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "add", "--amount", "-1.0", "--netuid", "1",
    ]);
    assert!(cli.is_err(), "negative amount should be rejected by clap");
}

#[test]
fn parse_stake_add_non_numeric_amount() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "add", "--amount", "abc", "--netuid", "1",
    ]);
    assert!(cli.is_err(), "non-numeric amount should fail");
}

#[test]
fn parse_stake_add_non_numeric_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "add", "--amount", "1.0", "--netuid", "abc",
    ]);
    assert!(cli.is_err(), "non-numeric netuid should fail");
}

#[test]
fn parse_stake_add_negative_slippage() {
    // Clap treats negative numbers as unknown args (dash prefix), so this is rejected.
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "add", "--amount", "1.0", "--netuid", "1",
        "--max-slippage", "-1.0",
    ]);
    assert!(cli.is_err(), "negative slippage should be rejected by clap");
}

// ── stake remove edge cases ──

#[test]
fn parse_stake_remove_with_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "remove", "--amount", "1.0", "--netuid", "1",
        "--hotkey", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_stake_remove_zero_amount() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "remove", "--amount", "0.0", "--netuid", "1",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_stake_remove_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "remove", "--amount", "1.0",
    ]);
    assert!(cli.is_err(), "remove without netuid should fail");
    let err = cli.unwrap_err().to_string();
    assert!(err.contains("netuid"), "error should mention netuid: {}", err);
}

#[test]
fn parse_stake_remove_missing_amount() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "remove", "--netuid", "1",
    ]);
    assert!(cli.is_err(), "remove without amount should fail");
    let err = cli.unwrap_err().to_string();
    assert!(err.contains("amount"), "error should mention amount: {}", err);
}

// ── stake move edge cases ──

#[test]
fn parse_stake_move_same_subnet() {
    // Moving from and to same subnet — semantically odd but should parse
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "move", "--amount", "1.0", "--from", "1", "--to", "1",
    ]);
    assert!(cli.is_ok(), "move same subnet: {:?}", cli.err());
}

#[test]
fn parse_stake_move_missing_from() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "move", "--amount", "1.0", "--to", "2",
    ]);
    assert!(cli.is_err(), "move without --from should fail");
}

#[test]
fn parse_stake_move_missing_to() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "move", "--amount", "1.0", "--from", "1",
    ]);
    assert!(cli.is_err(), "move without --to should fail");
}

#[test]
fn parse_stake_move_missing_amount() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "move", "--from", "1", "--to", "2",
    ]);
    assert!(cli.is_err(), "move without --amount should fail");
}

#[test]
fn parse_stake_move_with_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "move", "--amount", "5.0", "--from", "1", "--to", "2",
        "--hotkey", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

// ── stake swap edge cases ──

#[test]
fn parse_stake_swap_missing_from_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "swap", "--amount", "1.0", "--netuid", "1",
        "--to-hotkey", "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
    ]);
    assert!(cli.is_err(), "swap without --from-hotkey should fail");
}

#[test]
fn parse_stake_swap_missing_to_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "swap", "--amount", "1.0", "--netuid", "1",
        "--from-hotkey", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_err(), "swap without --to-hotkey should fail");
}

#[test]
fn parse_stake_swap_same_hotkey() {
    // Swap from and to same hotkey — semantically odd but should parse
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "swap", "--amount", "1.0", "--netuid", "1",
        "--from-hotkey", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--to-hotkey", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "same hotkey swap: {:?}", cli.err());
}

#[test]
fn parse_stake_swap_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "swap", "--amount", "1.0",
        "--from-hotkey", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--to-hotkey", "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
    ]);
    assert!(cli.is_err(), "swap without --netuid should fail");
}

// ── stake list edge cases ──

#[test]
fn parse_stake_list_default() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "stake", "list"]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_stake_list_with_address() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "list",
        "--address", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_stake_list_with_address_and_at_block() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "list",
        "--address", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--at-block", "1000000",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_stake_list_json_output() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--output", "json", "stake", "list",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
    assert_eq!(cli.unwrap().output, OutputFormat::Json);
}

// ── stake add-limit edge cases ──

#[test]
fn parse_stake_add_limit_missing_price() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "add-limit", "--amount", "1.0", "--netuid", "1", "--partial",
    ]);
    assert!(cli.is_err(), "add-limit without --price should fail");
}

#[test]
fn parse_stake_add_limit_missing_amount() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "add-limit", "--netuid", "1", "--price", "0.5", "--partial",
    ]);
    assert!(cli.is_err(), "add-limit without --amount should fail");
}

#[test]
fn parse_stake_add_limit_zero_price() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "add-limit", "--amount", "1.0", "--netuid", "1",
        "--price", "0.0", "--partial",
    ]);
    assert!(cli.is_ok(), "zero price: {:?}", cli.err());
}

#[test]
fn parse_stake_add_limit_with_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "add-limit", "--amount", "10.0", "--netuid", "1",
        "--price", "0.001", "--partial",
        "--hotkey", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

// ── stake remove-limit edge cases ──

#[test]
fn parse_stake_remove_limit_missing_price() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "remove-limit", "--amount", "1.0", "--netuid", "1", "--partial",
    ]);
    assert!(cli.is_err(), "remove-limit without --price should fail");
}

#[test]
fn parse_stake_remove_limit_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "remove-limit", "--amount", "1.0", "--price", "0.5", "--partial",
    ]);
    assert!(cli.is_err(), "remove-limit without --netuid should fail");
}

// ── stake swap-limit edge cases ──

#[test]
fn parse_stake_swap_limit_missing_from() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "swap-limit", "--amount", "1.0", "--to", "2",
        "--price", "0.5", "--partial",
    ]);
    assert!(cli.is_err(), "swap-limit without --from should fail");
}

#[test]
fn parse_stake_swap_limit_missing_to() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "swap-limit", "--amount", "1.0", "--from", "1",
        "--price", "0.5", "--partial",
    ]);
    assert!(cli.is_err(), "swap-limit without --to should fail");
}

#[test]
fn parse_stake_swap_limit_missing_price() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "swap-limit", "--amount", "1.0", "--from", "1", "--to", "2",
        "--partial",
    ]);
    assert!(cli.is_err(), "swap-limit without --price should fail");
}

#[test]
fn parse_stake_swap_limit_all_flags() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "swap-limit",
        "--amount", "50.0", "--from", "1", "--to", "5",
        "--price", "1.5", "--partial",
        "--hotkey", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

// ── stake childkey-take edge cases ──

#[test]
fn parse_stake_childkey_take_missing_take() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "childkey-take", "--netuid", "1",
    ]);
    assert!(cli.is_err(), "childkey-take without --take should fail");
}

#[test]
fn parse_stake_childkey_take_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "childkey-take", "--take", "5.0",
    ]);
    assert!(cli.is_err(), "childkey-take without --netuid should fail");
}

#[test]
fn parse_stake_childkey_take_zero() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "childkey-take", "--take", "0.0", "--netuid", "1",
    ]);
    assert!(cli.is_ok(), "zero take: {:?}", cli.err());
}

#[test]
fn parse_stake_childkey_take_max() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "childkey-take", "--take", "18.0", "--netuid", "1",
    ]);
    assert!(cli.is_ok(), "max take (18%): {:?}", cli.err());
}

#[test]
fn parse_stake_childkey_take_with_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "childkey-take", "--take", "10.0", "--netuid", "1",
        "--hotkey", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

// ── stake set-children edge cases ──

#[test]
fn parse_stake_set_children_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "set-children",
        "--children", "1000:5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_err(), "set-children without --netuid should fail");
}

#[test]
fn parse_stake_set_children_missing_children() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "set-children", "--netuid", "1",
    ]);
    assert!(cli.is_err(), "set-children without --children should fail");
}

#[test]
fn parse_stake_set_children_multiple() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "set-children", "--netuid", "1",
        "--children", "500:5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY,500:5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

// ── stake recycle-alpha edge cases ──

#[test]
fn parse_stake_recycle_alpha_missing_amount() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "recycle-alpha", "--netuid", "1",
    ]);
    assert!(cli.is_err(), "recycle-alpha without --amount should fail");
}

#[test]
fn parse_stake_recycle_alpha_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "recycle-alpha", "--amount", "1.0",
    ]);
    assert!(cli.is_err(), "recycle-alpha without --netuid should fail");
}

// ── stake burn-alpha edge cases ──

#[test]
fn parse_stake_burn_alpha_missing_amount() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "burn-alpha", "--netuid", "1",
    ]);
    assert!(cli.is_err(), "burn-alpha without --amount should fail");
}

#[test]
fn parse_stake_burn_alpha_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "burn-alpha", "--amount", "1.0",
    ]);
    assert!(cli.is_err(), "burn-alpha without --netuid should fail");
}

// ── stake unstake-all edge cases ──

#[test]
fn parse_stake_unstake_all_no_args() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "stake", "unstake-all"]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_stake_unstake_all_alpha_no_args() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "stake", "unstake-all-alpha"]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_stake_unstake_all_alpha_with_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "unstake-all-alpha",
        "--hotkey", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

// ── stake claim-root edge cases ──

#[test]
fn parse_stake_claim_root_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "stake", "claim-root"]);
    assert!(cli.is_err(), "claim-root without --netuid should fail");
}

#[test]
fn parse_stake_claim_root_with_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "claim-root", "--netuid", "1",
        "--hotkey", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

// ── stake set-auto edge cases ──

#[test]
fn parse_stake_set_auto_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "stake", "set-auto"]);
    assert!(cli.is_err(), "set-auto without --netuid should fail");
}

#[test]
fn parse_stake_set_auto_netuid_zero() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "set-auto", "--netuid", "0",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

// ── stake show-auto edge cases ──

#[test]
fn parse_stake_show_auto_default() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "stake", "show-auto"]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_stake_show_auto_with_address() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "show-auto",
        "--address", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

// ── stake set-claim edge cases ──

#[test]
fn parse_stake_set_claim_invalid_type() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "set-claim", "--claim-type", "invalid",
    ]);
    assert!(cli.is_err(), "invalid claim type should fail");
}

#[test]
fn parse_stake_set_claim_keep_subnets_without_subnets() {
    // keep-subnets without --subnets should parse (subnets is optional)
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "set-claim", "--claim-type", "keep-subnets",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_stake_set_claim_keep_subnets_with_subnets() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "set-claim", "--claim-type", "keep-subnets",
        "--subnets", "1,2,5,10",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_stake_set_claim_missing_type() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "set-claim",
    ]);
    assert!(cli.is_err(), "set-claim without --claim-type should fail");
}

// ── stake transfer-stake edge cases ──

#[test]
fn parse_stake_transfer_stake_missing_from() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "transfer-stake",
        "--dest", "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--amount", "10.0", "--to", "2",
    ]);
    assert!(cli.is_err(), "transfer-stake without --from should fail");
}

#[test]
fn parse_stake_transfer_stake_missing_to() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "transfer-stake",
        "--dest", "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--amount", "10.0", "--from", "1",
    ]);
    assert!(cli.is_err(), "transfer-stake without --to should fail");
}

#[test]
fn parse_stake_transfer_stake_same_subnet() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "transfer-stake",
        "--dest", "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--amount", "10.0", "--from", "1", "--to", "1",
    ]);
    assert!(cli.is_ok(), "same subnet transfer: {:?}", cli.err());
}

// ── stake process-claim edge cases ──

#[test]
fn parse_stake_process_claim_with_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "process-claim",
        "--hotkey", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_stake_process_claim_with_hotkey_and_netuids() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "process-claim",
        "--hotkey", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--netuids", "1,5,10",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

// ── stake wizard edge cases ──

#[test]
fn parse_stake_wizard_all_flags() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--yes", "--password", "pass",
        "stake", "wizard",
        "--netuid", "1", "--amount", "5.0",
        "--hotkey", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "wizard full non-interactive: {:?}", cli.err());
}

#[test]
fn parse_stake_wizard_partial_flags() {
    // Only netuid — amount and hotkey will be prompted
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--yes", "stake", "wizard", "--netuid", "5",
    ]);
    assert!(cli.is_ok(), "wizard with only netuid: {:?}", cli.err());
}

#[test]
fn parse_stake_wizard_no_flags() {
    // Fully interactive mode
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "stake", "wizard"]);
    assert!(cli.is_ok(), "wizard no flags: {:?}", cli.err());
}

// ── global flag combinations with stake commands ──

#[test]
fn parse_stake_add_with_dry_run() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--dry-run", "stake", "add", "--amount", "1.0", "--netuid", "1",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
    assert!(cli.unwrap().dry_run);
}

#[test]
fn parse_stake_add_with_batch_mode() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--batch", "stake", "add", "--amount", "1.0", "--netuid", "1",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
    assert!(cli.unwrap().batch);
}

#[test]
fn parse_stake_list_with_verbose_and_time() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--verbose", "--time", "stake", "list",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
    let cli = cli.unwrap();
    assert!(cli.verbose);
    assert!(cli.time);
}

#[test]
fn parse_stake_add_with_proxy() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--proxy", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "stake", "add", "--amount", "1.0", "--netuid", "1",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
    assert!(cli.unwrap().proxy.is_some());
}

// ══════════════════════════════════════════════════════════════════════
// Batch 3: Comprehensive subnet command edge cases
// ══════════════════════════════════════════════════════════════════════

// ── subnet list edge cases ──

#[test]
fn parse_subnet_list_with_at_block() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "list", "--at-block", "1000000",
    ]);
    assert!(cli.is_ok(), "list --at-block: {:?}", cli.err());
}

#[test]
fn parse_subnet_list_with_json_output() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--output", "json", "subnet", "list",
    ]);
    assert!(cli.is_ok(), "list --output json: {:?}", cli.err());
}

#[test]
fn parse_subnet_list_at_block_zero() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "list", "--at-block", "0",
    ]);
    assert!(cli.is_ok(), "list --at-block 0: {:?}", cli.err());
}

#[test]
fn parse_subnet_list_at_block_overflow() {
    // u32::MAX+1 should fail
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "list", "--at-block", "4294967296",
    ]);
    assert!(cli.is_err(), "at-block overflow should fail");
}

// ── subnet show edge cases ──

#[test]
fn parse_subnet_show_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "show"]);
    assert!(cli.is_err(), "show without --netuid should fail");
}

#[test]
fn parse_subnet_show_netuid_zero() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "show", "--netuid", "0",
    ]);
    assert!(cli.is_ok(), "show netuid 0 (root): {:?}", cli.err());
}

#[test]
fn parse_subnet_show_netuid_max() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "show", "--netuid", "65535",
    ]);
    assert!(cli.is_ok(), "show netuid max: {:?}", cli.err());
}

#[test]
fn parse_subnet_show_netuid_overflow() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "show", "--netuid", "65536",
    ]);
    assert!(cli.is_err(), "netuid overflow u16 should fail");
}

#[test]
fn parse_subnet_show_netuid_negative() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "show", "--netuid", "-1",
    ]);
    assert!(cli.is_err(), "negative netuid should fail");
}

#[test]
fn parse_subnet_show_netuid_non_numeric() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "show", "--netuid", "abc",
    ]);
    assert!(cli.is_err(), "non-numeric netuid should fail");
}

#[test]
fn parse_subnet_show_with_at_block() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "show", "--netuid", "1", "--at-block", "500000",
    ]);
    assert!(cli.is_ok(), "show with --at-block: {:?}", cli.err());
}

// ── subnet hyperparams edge cases ──

#[test]
fn parse_subnet_hyperparams_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "hyperparams"]);
    assert!(cli.is_err(), "hyperparams without --netuid should fail");
}

#[test]
fn parse_subnet_hyperparams_with_global_flags() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--verbose", "--output", "json", "subnet", "hyperparams", "--netuid", "1",
    ]);
    assert!(cli.is_ok(), "hyperparams with global flags: {:?}", cli.err());
}

// ── subnet metagraph edge cases ──

#[test]
fn parse_subnet_metagraph_with_uid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "metagraph", "--netuid", "1", "--uid", "0",
    ]);
    assert!(cli.is_ok(), "metagraph with --uid: {:?}", cli.err());
}

#[test]
fn parse_subnet_metagraph_full_flag() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "metagraph", "--netuid", "1", "--full",
    ]);
    assert!(cli.is_ok(), "metagraph --full: {:?}", cli.err());
}

#[test]
fn parse_subnet_metagraph_save_flag() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "metagraph", "--netuid", "1", "--save",
    ]);
    assert!(cli.is_ok(), "metagraph --save: {:?}", cli.err());
}

#[test]
fn parse_subnet_metagraph_all_opts() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "metagraph", "--netuid", "97",
        "--uid", "10", "--at-block", "1000", "--full", "--save",
    ]);
    assert!(cli.is_ok(), "metagraph all opts: {:?}", cli.err());
}

#[test]
fn parse_subnet_metagraph_uid_max() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "metagraph", "--netuid", "1", "--uid", "65535",
    ]);
    assert!(cli.is_ok(), "metagraph uid max: {:?}", cli.err());
}

#[test]
fn parse_subnet_metagraph_uid_overflow() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "metagraph", "--netuid", "1", "--uid", "65536",
    ]);
    assert!(cli.is_err(), "metagraph uid overflow should fail");
}

// ── subnet cache commands edge cases ──

#[test]
fn parse_subnet_cache_load_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "cache-load"]);
    assert!(cli.is_err(), "cache-load without --netuid should fail");
}

#[test]
fn parse_subnet_cache_load_with_block() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "cache-load", "--netuid", "1", "--block", "5000000",
    ]);
    assert!(cli.is_ok(), "cache-load block: {:?}", cli.err());
}

#[test]
fn parse_subnet_cache_list_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "cache-list"]);
    assert!(cli.is_err(), "cache-list without --netuid should fail");
}

#[test]
fn parse_subnet_cache_diff_all_blocks() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "cache-diff", "--netuid", "1",
        "--from-block", "100000", "--to-block", "200000",
    ]);
    assert!(cli.is_ok(), "cache-diff with blocks: {:?}", cli.err());
}

#[test]
fn parse_subnet_cache_diff_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "cache-diff"]);
    assert!(cli.is_err(), "cache-diff without --netuid should fail");
}

#[test]
fn parse_subnet_cache_prune_custom_keep() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "cache-prune", "--netuid", "1", "--keep", "5",
    ]);
    assert!(cli.is_ok(), "cache-prune --keep 5: {:?}", cli.err());
}

#[test]
fn parse_subnet_cache_prune_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "cache-prune"]);
    assert!(cli.is_err(), "cache-prune without --netuid should fail");
}

#[test]
fn parse_subnet_cache_prune_keep_zero() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "cache-prune", "--netuid", "1", "--keep", "0",
    ]);
    assert!(cli.is_ok(), "cache-prune --keep 0: {:?}", cli.err());
}

// ── subnet probe edge cases ──

#[test]
fn parse_subnet_probe_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "probe", "--netuid", "1",
    ]);
    assert!(cli.is_ok(), "probe basic: {:?}", cli.err());
}

#[test]
fn parse_subnet_probe_with_uids() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "probe", "--netuid", "1", "--uids", "0,1,5,10",
    ]);
    assert!(cli.is_ok(), "probe --uids: {:?}", cli.err());
}

#[test]
fn parse_subnet_probe_custom_timeout() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "probe", "--netuid", "1", "--timeout-ms", "10000",
    ]);
    assert!(cli.is_ok(), "probe --timeout-ms: {:?}", cli.err());
}

#[test]
fn parse_subnet_probe_custom_concurrency() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "probe", "--netuid", "1", "--concurrency", "64",
    ]);
    assert!(cli.is_ok(), "probe --concurrency: {:?}", cli.err());
}

#[test]
fn parse_subnet_probe_all_opts() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "probe", "--netuid", "97",
        "--uids", "0,1,2", "--timeout-ms", "5000", "--concurrency", "16",
    ]);
    assert!(cli.is_ok(), "probe all opts: {:?}", cli.err());
}

#[test]
fn parse_subnet_probe_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "probe"]);
    assert!(cli.is_err(), "probe without --netuid should fail");
}

// ── subnet register edge cases ──

#[test]
fn parse_subnet_register_with_global_flags() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--yes", "--password", "pw", "--batch", "subnet", "register",
    ]);
    assert!(cli.is_ok(), "register all flags: {:?}", cli.err());
    let cli = cli.unwrap();
    assert!(cli.yes);
    assert!(cli.batch);
}

// ── subnet register-neuron edge cases ──

#[test]
fn parse_subnet_register_neuron_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "register-neuron"]);
    assert!(cli.is_err(), "register-neuron without --netuid should fail");
}

#[test]
fn parse_subnet_register_neuron_netuid_zero() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "register-neuron", "--netuid", "0",
    ]);
    assert!(cli.is_ok(), "register-neuron netuid 0: {:?}", cli.err());
}

#[test]
fn parse_subnet_register_neuron_with_dry_run() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--dry-run", "subnet", "register-neuron", "--netuid", "1",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
    assert!(cli.unwrap().dry_run);
}

// ── subnet pow edge cases ──

#[test]
fn parse_subnet_pow_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "pow"]);
    assert!(cli.is_err(), "pow without --netuid should fail");
}

#[test]
fn parse_subnet_pow_custom_threads() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "pow", "--netuid", "1", "--threads", "16",
    ]);
    assert!(cli.is_ok(), "pow --threads 16: {:?}", cli.err());
}

#[test]
fn parse_subnet_pow_one_thread() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "pow", "--netuid", "1", "--threads", "1",
    ]);
    assert!(cli.is_ok(), "pow --threads 1: {:?}", cli.err());
}

#[test]
fn parse_subnet_pow_zero_threads() {
    // 0 thread should parse (runtime may reject)
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "pow", "--netuid", "1", "--threads", "0",
    ]);
    assert!(cli.is_ok(), "pow --threads 0: {:?}", cli.err());
}

// ── subnet dissolve edge cases ──

#[test]
fn parse_subnet_dissolve_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "dissolve"]);
    assert!(cli.is_err(), "dissolve without --netuid should fail");
}

#[test]
fn parse_subnet_dissolve_with_batch() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--yes", "--batch", "subnet", "dissolve", "--netuid", "5",
    ]);
    assert!(cli.is_ok(), "dissolve with batch: {:?}", cli.err());
}

// ── subnet watch edge cases ──

#[test]
fn parse_subnet_watch_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "watch"]);
    assert!(cli.is_err(), "watch without --netuid should fail");
}

#[test]
fn parse_subnet_watch_default_interval() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "watch", "--netuid", "1",
    ]);
    assert!(cli.is_ok(), "watch default interval: {:?}", cli.err());
}

#[test]
fn parse_subnet_watch_one_sec_interval() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "watch", "--netuid", "1", "--interval", "1",
    ]);
    assert!(cli.is_ok(), "watch --interval 1: {:?}", cli.err());
}

// ── subnet liquidity edge cases ──

#[test]
fn parse_subnet_liquidity_missing_netuid_ok() {
    // netuid is optional for all-subnet view
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "liquidity"]);
    assert!(cli.is_ok(), "liquidity without netuid (all): {:?}", cli.err());
}

#[test]
fn parse_subnet_liquidity_with_json() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--output", "json", "subnet", "liquidity", "--netuid", "1",
    ]);
    assert!(cli.is_ok(), "liquidity json: {:?}", cli.err());
}

// ── subnet monitor edge cases ──

#[test]
fn parse_subnet_monitor_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "monitor"]);
    assert!(cli.is_err(), "monitor without --netuid should fail");
}

#[test]
fn parse_subnet_monitor_all_opts() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "monitor", "--netuid", "97",
        "--interval", "60", "--json",
    ]);
    assert!(cli.is_ok(), "monitor all opts: {:?}", cli.err());
}

// ── subnet health edge cases ──

#[test]
fn parse_subnet_health_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "health"]);
    assert!(cli.is_err(), "health without --netuid should fail");
}

#[test]
fn parse_subnet_health_with_json() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--output", "json", "subnet", "health", "--netuid", "1",
    ]);
    assert!(cli.is_ok(), "health json: {:?}", cli.err());
}

// ── subnet emissions edge cases ──

#[test]
fn parse_subnet_emissions_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "emissions"]);
    assert!(cli.is_err(), "emissions without --netuid should fail");
}

// ── subnet cost edge cases ──

#[test]
fn parse_subnet_cost_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "cost"]);
    assert!(cli.is_err(), "cost without --netuid should fail");
}

#[test]
fn parse_subnet_cost_with_json() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--output", "json", "subnet", "cost", "--netuid", "42",
    ]);
    assert!(cli.is_ok(), "cost json: {:?}", cli.err());
}

// ── subnet commits edge cases ──

#[test]
fn parse_subnet_commits_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "commits"]);
    assert!(cli.is_err(), "commits without --netuid should fail");
}

#[test]
fn parse_subnet_commits_with_json() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--output", "json", "subnet", "commits", "--netuid", "1",
        "--hotkey", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "commits json + hotkey: {:?}", cli.err());
}

// ── subnet set-param comprehensive edge cases ──

#[test]
fn parse_subnet_set_param_u64_value() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "set-param", "--netuid", "1",
        "--param", "min_difficulty", "--value", "18446744073709551615",
    ]);
    assert!(cli.is_ok(), "set-param u64 max: {:?}", cli.err());
}

#[test]
fn parse_subnet_set_param_all_bool_variants() {
    for val in &["true", "false", "1", "0", "yes", "no", "on", "off"] {
        let cli = agcli::cli::Cli::try_parse_from([
            "agcli", "subnet", "set-param", "--netuid", "1",
            "--param", "registration_allowed", "--value", val,
        ]);
        assert!(cli.is_ok(), "set-param bool {} should parse: {:?}", val, cli.err());
    }
}

#[test]
fn parse_subnet_set_param_with_batch_and_yes() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--yes", "--batch", "subnet", "set-param",
        "--netuid", "1", "--param", "tempo", "--value", "100",
    ]);
    assert!(cli.is_ok(), "set-param batch+yes: {:?}", cli.err());
    let cli = cli.unwrap();
    assert!(cli.yes);
    assert!(cli.batch);
}

// ── subnet set-symbol edge cases ──

#[test]
fn parse_subnet_set_symbol() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "set-symbol", "--netuid", "1", "--symbol", "ALPHA",
    ]);
    assert!(cli.is_ok(), "set-symbol: {:?}", cli.err());
}

#[test]
fn parse_subnet_set_symbol_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "set-symbol", "--symbol", "ALPHA",
    ]);
    assert!(cli.is_err(), "set-symbol without --netuid should fail");
}

#[test]
fn parse_subnet_set_symbol_missing_symbol() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "set-symbol", "--netuid", "1",
    ]);
    assert!(cli.is_err(), "set-symbol without --symbol should fail");
}

#[test]
fn parse_subnet_set_symbol_empty_string() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "set-symbol", "--netuid", "1", "--symbol", "",
    ]);
    // Empty string should parse (validation at runtime)
    assert!(cli.is_ok(), "set-symbol empty: {:?}", cli.err());
}

#[test]
fn parse_subnet_set_symbol_long_name() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "set-symbol", "--netuid", "1",
        "--symbol", "VERYLONGSYMBOLNAME",
    ]);
    assert!(cli.is_ok(), "set-symbol long: {:?}", cli.err());
}

// ── subnet emission-split edge cases ──

#[test]
fn parse_subnet_emission_split_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "emission-split"]);
    assert!(cli.is_err(), "emission-split without --netuid should fail");
}

// ── subnet trim edge cases ──

#[test]
fn parse_subnet_trim_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "trim", "--max-uids", "256",
    ]);
    assert!(cli.is_err(), "trim without --netuid should fail");
}

#[test]
fn parse_subnet_trim_missing_max_uids() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "trim", "--netuid", "1",
    ]);
    assert!(cli.is_err(), "trim without --max-uids should fail");
}

#[test]
fn parse_subnet_trim_zero_max_uids() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "trim", "--netuid", "1", "--max-uids", "0",
    ]);
    assert!(cli.is_ok(), "trim --max-uids 0: {:?}", cli.err());
}

#[test]
fn parse_subnet_trim_max_uids_overflow() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "trim", "--netuid", "1", "--max-uids", "65536",
    ]);
    assert!(cli.is_err(), "trim --max-uids overflow should fail");
}

// ── subnet check-start edge cases ──

#[test]
fn parse_subnet_check_start_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "check-start"]);
    assert!(cli.is_err(), "check-start without --netuid should fail");
}

#[test]
fn parse_subnet_check_start_with_json() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--output", "json", "subnet", "check-start", "--netuid", "5",
    ]);
    assert!(cli.is_ok(), "check-start json: {:?}", cli.err());
}

// ── subnet start edge cases ──

#[test]
fn parse_subnet_start_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "start"]);
    assert!(cli.is_err(), "start without --netuid should fail");
}

// ── subnet mechanism-count edge cases ──

#[test]
fn parse_subnet_mechanism_count_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "mechanism-count"]);
    assert!(cli.is_err(), "mechanism-count without --netuid should fail");
}

// ── subnet set-mechanism-count edge cases ──

#[test]
fn parse_subnet_set_mechanism_count_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "set-mechanism-count", "--count", "2",
    ]);
    assert!(cli.is_err(), "set-mechanism-count without --netuid should fail");
}

#[test]
fn parse_subnet_set_mechanism_count_missing_count() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "set-mechanism-count", "--netuid", "1",
    ]);
    assert!(cli.is_err(), "set-mechanism-count without --count should fail");
}

#[test]
fn parse_subnet_set_mechanism_count_zero() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "set-mechanism-count", "--netuid", "1", "--count", "0",
    ]);
    assert!(cli.is_ok(), "set-mechanism-count 0: {:?}", cli.err());
}

// ── subnet set-emission-split edge cases ──

#[test]
fn parse_subnet_set_emission_split_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "set-emission-split", "--weights", "50,50",
    ]);
    assert!(cli.is_err(), "set-emission-split without --netuid should fail");
}

#[test]
fn parse_subnet_set_emission_split_missing_weights() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "set-emission-split", "--netuid", "1",
    ]);
    assert!(cli.is_err(), "set-emission-split without --weights should fail");
}

#[test]
fn parse_subnet_set_emission_split_three_way() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "set-emission-split", "--netuid", "1", "--weights", "33,33,34",
    ]);
    assert!(cli.is_ok(), "emission-split 3-way: {:?}", cli.err());
}

#[test]
fn parse_subnet_set_emission_split_single() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "set-emission-split", "--netuid", "1", "--weights", "100",
    ]);
    assert!(cli.is_ok(), "emission-split single: {:?}", cli.err());
}

// ── subnet snipe missing netuid ──

#[test]
fn parse_subnet_snipe_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "snipe"]);
    assert!(cli.is_err(), "snipe without --netuid should fail");
}

#[test]
fn parse_subnet_snipe_max_cost_zero() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "snipe", "--netuid", "1", "--max-cost", "0.0",
    ]);
    assert!(cli.is_ok(), "snipe --max-cost 0: {:?}", cli.err());
}

#[test]
fn parse_subnet_snipe_max_cost_negative() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "snipe", "--netuid", "1", "--max-cost", "-1.0",
    ]);
    // Clap rejects negative floats (treats -1.0 as unknown arg) — catches user error early
    assert!(cli.is_err(), "snipe --max-cost -1 should be rejected by clap");
}

#[test]
fn parse_subnet_snipe_max_attempts_zero() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "snipe", "--netuid", "1", "--max-attempts", "0",
    ]);
    assert!(cli.is_ok(), "snipe --max-attempts 0: {:?}", cli.err());
}

// ══════════════════════════════════════════════════════════════════════
// Batch 4: Weight commands comprehensive tests
// ══════════════════════════════════════════════════════════════════════

// ── weights set edge cases ──

#[test]
fn parse_weights_set_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "weights", "set", "--netuid", "1", "--weights", "0:100,1:200",
    ]);
    assert!(cli.is_ok(), "weights set: {:?}", cli.err());
}

#[test]
fn parse_weights_set_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "weights", "set", "--weights", "0:100",
    ]);
    assert!(cli.is_err(), "weights set without --netuid should fail");
}

#[test]
fn parse_weights_set_missing_weights() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "weights", "set", "--netuid", "1",
    ]);
    assert!(cli.is_err(), "weights set without --weights should fail");
}

#[test]
fn parse_weights_set_with_version_key() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "weights", "set", "--netuid", "1",
        "--weights", "0:100", "--version-key", "42",
    ]);
    assert!(cli.is_ok(), "weights set --version-key: {:?}", cli.err());
}

#[test]
fn parse_weights_set_stdin() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "weights", "set", "--netuid", "1", "--weights", "-",
    ]);
    assert!(cli.is_ok(), "weights set stdin: {:?}", cli.err());
}

#[test]
fn parse_weights_set_file_path() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "weights", "set", "--netuid", "1", "--weights", "@weights.json",
    ]);
    assert!(cli.is_ok(), "weights set file: {:?}", cli.err());
}

#[test]
fn parse_weights_set_single_weight() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "weights", "set", "--netuid", "1", "--weights", "0:65535",
    ]);
    assert!(cli.is_ok(), "weights set single: {:?}", cli.err());
}

#[test]
fn parse_weights_set_with_all_global_flags() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--yes", "--password", "pw", "--batch", "--dry-run",
        "weights", "set", "--netuid", "1", "--weights", "0:100",
    ]);
    assert!(cli.is_ok(), "weights set all flags: {:?}", cli.err());
    let cli = cli.unwrap();
    assert!(cli.yes);
    assert!(cli.batch);
    assert!(cli.dry_run);
}

// ── weights commit edge cases ──

#[test]
fn parse_weights_commit_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "weights", "commit", "--netuid", "1", "--weights", "0:100,1:200",
    ]);
    assert!(cli.is_ok(), "weights commit: {:?}", cli.err());
}

#[test]
fn parse_weights_commit_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "weights", "commit", "--weights", "0:100",
    ]);
    assert!(cli.is_err(), "commit without --netuid should fail");
}

#[test]
fn parse_weights_commit_missing_weights() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "weights", "commit", "--netuid", "1",
    ]);
    assert!(cli.is_err(), "commit without --weights should fail");
}

// ── weights reveal edge cases ──

#[test]
fn parse_weights_reveal_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "weights", "reveal", "--netuid", "1",
        "--weights", "0:100,1:200", "--salt", "mysalt",
    ]);
    assert!(cli.is_ok(), "weights reveal: {:?}", cli.err());
}

#[test]
fn parse_weights_reveal_missing_salt() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "weights", "reveal", "--netuid", "1", "--weights", "0:100",
    ]);
    assert!(cli.is_err(), "reveal without --salt should fail");
}

#[test]
fn parse_weights_reveal_missing_weights() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "weights", "reveal", "--netuid", "1", "--salt", "abc",
    ]);
    assert!(cli.is_err(), "reveal without --weights should fail");
}

#[test]
fn parse_weights_reveal_with_version_key() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "weights", "reveal", "--netuid", "1",
        "--weights", "0:100", "--salt", "abc", "--version-key", "99",
    ]);
    assert!(cli.is_ok(), "reveal --version-key: {:?}", cli.err());
}

// ── weights show edge cases ──

#[test]
fn parse_weights_show_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "weights", "show", "--netuid", "1",
    ]);
    assert!(cli.is_ok(), "weights show: {:?}", cli.err());
}

#[test]
fn parse_weights_show_with_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "weights", "show", "--netuid", "1",
        "--hotkey", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "weights show --hotkey: {:?}", cli.err());
}

#[test]
fn parse_weights_show_all_opts() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--output", "json", "weights", "show", "--netuid", "97",
        "--hotkey", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--limit", "5",
    ]);
    assert!(cli.is_ok(), "weights show all: {:?}", cli.err());
}

#[test]
fn parse_weights_show_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "weights", "show"]);
    assert!(cli.is_err(), "weights show without --netuid should fail");
}

// ── weights status edge cases ──

#[test]
fn parse_weights_status_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "weights", "status", "--netuid", "1",
    ]);
    assert!(cli.is_ok(), "weights status: {:?}", cli.err());
}

#[test]
fn parse_weights_status_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "weights", "status"]);
    assert!(cli.is_err(), "weights status without --netuid should fail");
}

// ── weights commit-reveal (atomic) edge cases ──

#[test]
fn parse_weights_commit_reveal_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "weights", "commit-reveal", "--netuid", "1", "--weights", "0:100",
    ]);
    assert!(cli.is_ok(), "commit-reveal basic: {:?}", cli.err());
}

#[test]
fn parse_weights_commit_reveal_with_wait() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "weights", "commit-reveal", "--netuid", "1",
        "--weights", "0:100,1:200", "--wait",
    ]);
    assert!(cli.is_ok(), "commit-reveal --wait: {:?}", cli.err());
}

#[test]
fn parse_weights_commit_reveal_with_version_key() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "weights", "commit-reveal", "--netuid", "1",
        "--weights", "0:100", "--version-key", "42",
    ]);
    assert!(cli.is_ok(), "commit-reveal --version-key: {:?}", cli.err());
}

#[test]
fn parse_weights_commit_reveal_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "weights", "commit-reveal", "--weights", "0:100",
    ]);
    assert!(cli.is_err(), "commit-reveal without --netuid should fail");
}

#[test]
fn parse_weights_commit_reveal_missing_weights() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "weights", "commit-reveal", "--netuid", "1",
    ]);
    assert!(cli.is_err(), "commit-reveal without --weights should fail");
}

#[test]
fn parse_weights_commit_reveal_stdin() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "weights", "commit-reveal", "--netuid", "1", "--weights", "-",
    ]);
    assert!(cli.is_ok(), "commit-reveal stdin: {:?}", cli.err());
}

#[test]
fn parse_weights_commit_reveal_file() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "weights", "commit-reveal", "--netuid", "1", "--weights", "@weights.json",
    ]);
    assert!(cli.is_ok(), "commit-reveal file: {:?}", cli.err());
}

// ══════════════════════════════════════════════════════════════════════
// Batch 5: Delegate, proxy, root, identity, serve commands
// ══════════════════════════════════════════════════════════════════════

// ── delegate show ──

#[test]
fn parse_delegate_show_with_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "delegate", "show",
        "--hotkey", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "delegate show --hotkey: {:?}", cli.err());
}

// ── delegate list ──

#[test]
fn parse_delegate_list_with_json() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--output", "json", "delegate", "list",
    ]);
    assert!(cli.is_ok(), "delegate list json: {:?}", cli.err());
}

// ── delegate decrease-take ──

#[test]
fn parse_delegate_decrease_take_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "delegate", "decrease-take", "--take", "5.0",
    ]);
    assert!(cli.is_ok(), "decrease-take: {:?}", cli.err());
}

#[test]
fn parse_delegate_decrease_take_with_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "delegate", "decrease-take", "--take", "10.0",
        "--hotkey", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "decrease-take --hotkey: {:?}", cli.err());
}

#[test]
fn parse_delegate_decrease_take_missing_take() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "delegate", "decrease-take"]);
    assert!(cli.is_err(), "decrease-take without --take should fail");
}

#[test]
fn parse_delegate_decrease_take_zero() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "delegate", "decrease-take", "--take", "0.0",
    ]);
    assert!(cli.is_ok(), "decrease-take 0: {:?}", cli.err());
}

// ── delegate increase-take ──

#[test]
fn parse_delegate_increase_take_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "delegate", "increase-take", "--take", "15.0",
    ]);
    assert!(cli.is_ok(), "increase-take: {:?}", cli.err());
}

#[test]
fn parse_delegate_increase_take_missing_take() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "delegate", "increase-take"]);
    assert!(cli.is_err(), "increase-take without --take should fail");
}

// ── root commands ──

#[test]
fn parse_root_weights() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "root", "weights", "--weights", "1:100,2:200",
    ]);
    assert!(cli.is_ok(), "root weights: {:?}", cli.err());
}

#[test]
fn parse_root_weights_missing_weights() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "root", "weights"]);
    assert!(cli.is_err(), "root weights without --weights should fail");
}

#[test]
fn parse_root_register_with_global_flags() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--yes", "--password", "pw", "root", "register",
    ]);
    assert!(cli.is_ok(), "root register flags: {:?}", cli.err());
}

// ── identity commands ──

#[test]
fn parse_identity_set_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "identity", "set", "--name", "MyValidator",
    ]);
    assert!(cli.is_ok(), "identity set: {:?}", cli.err());
}

#[test]
fn parse_identity_set_all_fields() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "identity", "set", "--name", "MyValidator",
        "--url", "https://example.com", "--github", "myuser",
        "--description", "My awesome validator",
    ]);
    assert!(cli.is_ok(), "identity set all: {:?}", cli.err());
}

#[test]
fn parse_identity_set_missing_name() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "identity", "set", "--url", "https://example.com",
    ]);
    assert!(cli.is_err(), "identity set without --name should fail");
}

#[test]
fn parse_identity_show_missing_address() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "identity", "show"]);
    assert!(cli.is_err(), "identity show without --address should fail");
}

#[test]
fn parse_identity_set_subnet_all_fields() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "identity", "set-subnet", "--netuid", "1", "--name", "MySN",
        "--github", "myrepo", "--url", "https://sn1.example.com",
    ]);
    assert!(cli.is_ok(), "identity set-subnet all: {:?}", cli.err());
}

#[test]
fn parse_identity_set_subnet_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "identity", "set-subnet", "--name", "MySN",
    ]);
    assert!(cli.is_err(), "set-subnet without --netuid should fail");
}

#[test]
fn parse_identity_set_subnet_missing_name() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "identity", "set-subnet", "--netuid", "1",
    ]);
    assert!(cli.is_err(), "set-subnet without --name should fail");
}

// ── serve commands ──

#[test]
fn parse_serve_axon_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "serve", "axon", "--netuid", "1",
        "--ip", "192.168.1.1", "--port", "8091",
    ]);
    assert!(cli.is_ok(), "serve axon: {:?}", cli.err());
}

#[test]
fn parse_serve_axon_all_opts() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "serve", "axon", "--netuid", "1",
        "--ip", "10.0.0.1", "--port", "8091",
        "--protocol", "4", "--version", "1",
    ]);
    assert!(cli.is_ok(), "serve axon all: {:?}", cli.err());
}

#[test]
fn parse_serve_axon_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "serve", "axon", "--ip", "1.2.3.4", "--port", "8091",
    ]);
    assert!(cli.is_err(), "serve axon without --netuid should fail");
}

#[test]
fn parse_serve_axon_missing_ip() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "serve", "axon", "--netuid", "1", "--port", "8091",
    ]);
    assert!(cli.is_err(), "serve axon without --ip should fail");
}

#[test]
fn parse_serve_axon_missing_port() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "serve", "axon", "--netuid", "1", "--ip", "1.2.3.4",
    ]);
    assert!(cli.is_err(), "serve axon without --port should fail");
}

#[test]
fn parse_serve_axon_port_overflow() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "serve", "axon", "--netuid", "1",
        "--ip", "1.2.3.4", "--port", "65536",
    ]);
    assert!(cli.is_err(), "serve axon port overflow should fail");
}

#[test]
fn parse_serve_axon_port_zero() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "serve", "axon", "--netuid", "1",
        "--ip", "0.0.0.0", "--port", "0",
    ]);
    assert!(cli.is_ok(), "serve axon port 0: {:?}", cli.err());
}

#[test]
fn parse_serve_reset_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "serve", "reset"]);
    assert!(cli.is_err(), "serve reset without --netuid should fail");
}

#[test]
fn parse_serve_batch_axon_missing_file() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "serve", "batch-axon"]);
    assert!(cli.is_err(), "batch-axon without --file should fail");
}

// ── proxy commands ──

#[test]
fn parse_proxy_add_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "proxy", "add",
        "--delegate", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "proxy add: {:?}", cli.err());
}

#[test]
fn parse_proxy_add_all_opts() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "proxy", "add",
        "--delegate", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--proxy-type", "staking", "--delay", "100",
    ]);
    assert!(cli.is_ok(), "proxy add all: {:?}", cli.err());
}

#[test]
fn parse_proxy_add_missing_delegate() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "proxy", "add", "--proxy-type", "any",
    ]);
    assert!(cli.is_err(), "proxy add without --delegate should fail");
}

#[test]
fn parse_proxy_remove_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "proxy", "remove",
        "--delegate", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "proxy remove: {:?}", cli.err());
}

#[test]
fn parse_proxy_remove_missing_delegate() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "proxy", "remove"]);
    assert!(cli.is_err(), "proxy remove without --delegate should fail");
}

#[test]
fn parse_proxy_create_pure_defaults() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "proxy", "create-pure"]);
    assert!(cli.is_ok(), "proxy create-pure defaults: {:?}", cli.err());
}

#[test]
fn parse_proxy_create_pure_all_opts() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "proxy", "create-pure",
        "--proxy-type", "staking", "--delay", "50", "--index", "3",
    ]);
    assert!(cli.is_ok(), "proxy create-pure all: {:?}", cli.err());
}

#[test]
fn parse_proxy_kill_pure() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "proxy", "kill-pure",
        "--spawner", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--height", "1000", "--ext-index", "2",
    ]);
    assert!(cli.is_ok(), "proxy kill-pure: {:?}", cli.err());
}

#[test]
fn parse_proxy_kill_pure_missing_spawner() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "proxy", "kill-pure", "--height", "1000", "--ext-index", "2",
    ]);
    assert!(cli.is_err(), "kill-pure without --spawner should fail");
}

#[test]
fn parse_proxy_kill_pure_missing_height() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "proxy", "kill-pure",
        "--spawner", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--ext-index", "2",
    ]);
    assert!(cli.is_err(), "kill-pure without --height should fail");
}

#[test]
fn parse_proxy_kill_pure_missing_ext_index() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "proxy", "kill-pure",
        "--spawner", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--height", "1000",
    ]);
    assert!(cli.is_err(), "kill-pure without --ext-index should fail");
}

#[test]
fn parse_proxy_list_default() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "proxy", "list"]);
    assert!(cli.is_ok(), "proxy list: {:?}", cli.err());
}

#[test]
fn parse_proxy_list_with_address() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "proxy", "list",
        "--address", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "proxy list --address: {:?}", cli.err());
}

#[test]
fn parse_proxy_announce() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "proxy", "announce",
        "--real", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--call-hash", "0xabcdef1234567890",
    ]);
    assert!(cli.is_ok(), "proxy announce: {:?}", cli.err());
}

#[test]
fn parse_proxy_announce_missing_real() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "proxy", "announce", "--call-hash", "0xabc",
    ]);
    assert!(cli.is_err(), "announce without --real should fail");
}

#[test]
fn parse_proxy_announce_missing_call_hash() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "proxy", "announce",
        "--real", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_err(), "announce without --call-hash should fail");
}

#[test]
fn parse_proxy_reject_announcement() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "proxy", "reject-announcement",
        "--delegate", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--call-hash", "0xabc",
    ]);
    assert!(cli.is_ok(), "proxy reject: {:?}", cli.err());
}

#[test]
fn parse_proxy_list_announcements() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "proxy", "list-announcements"]);
    assert!(cli.is_ok(), "proxy list-announcements: {:?}", cli.err());
}

#[test]
fn parse_proxy_list_announcements_with_address() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "proxy", "list-announcements",
        "--address", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "proxy list-announcements --address: {:?}", cli.err());
}

// ── swap commands ──

#[test]
fn parse_swap_hotkey_missing_new() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "swap", "hotkey"]);
    assert!(cli.is_err(), "swap hotkey without --new-hotkey should fail");
}

#[test]
fn parse_swap_coldkey_missing_new() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "swap", "coldkey"]);
    assert!(cli.is_err(), "swap coldkey without --new-coldkey should fail");
}

// ── view commands comprehensive ──

#[test]
fn parse_view_portfolio_with_address() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "view", "portfolio",
        "--address", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "view portfolio --address: {:?}", cli.err());
}

#[test]
fn parse_view_neuron_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "view", "neuron", "--uid", "0",
    ]);
    assert!(cli.is_err(), "view neuron without --netuid should fail");
}

#[test]
fn parse_view_neuron_missing_uid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "view", "neuron", "--netuid", "1",
    ]);
    assert!(cli.is_err(), "view neuron without --uid should fail");
}

#[test]
fn parse_view_validators_with_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "view", "validators", "--netuid", "1", "--limit", "20",
    ]);
    assert!(cli.is_ok(), "view validators --netuid: {:?}", cli.err());
}

#[test]
fn parse_view_subnet_analytics_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "subnet-analytics"]);
    assert!(cli.is_err(), "view subnet-analytics without --netuid should fail");
}

#[test]
fn parse_view_swap_sim_alpha_direction() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "view", "swap-sim", "--netuid", "1", "--alpha", "100.0",
    ]);
    assert!(cli.is_ok(), "view swap-sim --alpha: {:?}", cli.err());
}

#[test]
fn parse_view_swap_sim_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "view", "swap-sim", "--tao", "10.0",
    ]);
    assert!(cli.is_err(), "view swap-sim without --netuid should fail");
}

#[test]
fn parse_view_nominations_missing_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "nominations"]);
    assert!(cli.is_err(), "nominations without --hotkey should fail");
}

#[test]
fn parse_view_metagraph_with_diff() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "view", "metagraph", "--netuid", "1",
        "--since-block", "1000000", "--limit", "10",
    ]);
    assert!(cli.is_ok(), "view metagraph diff: {:?}", cli.err());
}

#[test]
fn parse_view_axon() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "view", "axon", "--netuid", "1", "--uid", "0",
    ]);
    assert!(cli.is_ok(), "view axon: {:?}", cli.err());
}

#[test]
fn parse_view_axon_by_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "view", "axon", "--netuid", "1",
        "--hotkey", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "view axon --hotkey: {:?}", cli.err());
}

// ── multisig commands ──

#[test]
fn parse_multisig_address_missing_signatories() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "multisig", "address", "--threshold", "2",
    ]);
    assert!(cli.is_err(), "multisig without --signatories should fail");
}

#[test]
fn parse_multisig_address_missing_threshold() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "multisig", "address",
        "--signatories", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_err(), "multisig without --threshold should fail");
}

#[test]
fn parse_multisig_cancel() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "multisig", "cancel",
        "--others", "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--threshold", "2",
        "--call-hash", "0xabc",
        "--timepoint-height", "1000", "--timepoint-index", "1",
    ]);
    assert!(cli.is_ok(), "multisig cancel: {:?}", cli.err());
}

// ── transfer edge cases ──

#[test]
fn parse_transfer_with_all_flags() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--yes", "--password", "pw", "--batch", "--dry-run",
        "transfer",
        "--dest", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--amount", "1.0",
    ]);
    assert!(cli.is_ok(), "transfer all flags: {:?}", cli.err());
    let cli = cli.unwrap();
    assert!(cli.yes);
    assert!(cli.batch);
    assert!(cli.dry_run);
}

#[test]
fn parse_transfer_zero_amount() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "transfer",
        "--dest", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--amount", "0.0",
    ]);
    assert!(cli.is_ok(), "transfer zero: {:?}", cli.err());
}

#[test]
fn parse_transfer_tiny_amount() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "transfer",
        "--dest", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--amount", "0.000000001",
    ]);
    assert!(cli.is_ok(), "transfer 1 RAO: {:?}", cli.err());
}

#[test]
fn parse_transfer_large_amount() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "transfer",
        "--dest", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--amount", "21000000.0",
    ]);
    assert!(cli.is_ok(), "transfer 21M TAO: {:?}", cli.err());
}

// ══════════════════════════════════════════════════════════════════════
// Wallet name CLI parsing tests (edge cases for --name, --wallet, --hotkey)
// ══════════════════════════════════════════════════════════════════════

#[test]
fn parse_wallet_create_valid_names() {
    for name in &["default", "my-wallet", "wallet_1", "Alice", "test123"] {
        let cli = agcli::cli::Cli::try_parse_from([
            "agcli", "wallet", "create", "--name", name, "--password", "test",
        ]);
        assert!(cli.is_ok(), "valid name '{}' should parse: {:?}", name, cli.err());
    }
}

#[test]
fn parse_wallet_create_empty_name() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "create", "--name", "", "--password", "test",
    ]);
    // clap accepts empty string, but validate_name will reject it at runtime
    assert!(cli.is_ok(), "empty name should parse (validation is runtime)");
}

#[test]
fn parse_wallet_create_long_name() {
    let long_name = "a".repeat(100);
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "create", "--name", &long_name, "--password", "test",
    ]);
    // clap accepts any string, runtime validation catches length
    assert!(cli.is_ok(), "long name should parse: {:?}", cli.err());
}

#[test]
fn parse_wallet_global_wallet_flag() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--wallet", "my-wallet", "balance",
    ]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    assert_eq!(cli.wallet, "my-wallet");
}

#[test]
fn parse_wallet_global_wallet_short() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "-w", "custom", "balance",
    ]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    assert_eq!(cli.wallet, "custom");
}

#[test]
fn parse_wallet_global_hotkey_flag() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--hotkey", "miner1", "balance",
    ]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    assert_eq!(cli.hotkey, "miner1");
}

#[test]
fn parse_wallet_new_hotkey_valid_name() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "new-hotkey", "--name", "validator-1",
    ]);
    assert!(cli.is_ok(), "valid hotkey name: {:?}", cli.err());
}

#[test]
fn parse_wallet_new_hotkey_empty_name_fails() {
    // --name is required for new-hotkey, clap should enforce this
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "new-hotkey",
    ]);
    assert!(cli.is_err(), "new-hotkey without --name should fail");
}

#[test]
fn parse_wallet_import_with_name() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "import",
        "--name", "imported-wallet",
        "--mnemonic", "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        "--password", "test",
    ]);
    assert!(cli.is_ok(), "import with name: {:?}", cli.err());
}

#[test]
fn parse_wallet_regen_hotkey_with_name() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "regen-hotkey",
        "--name", "hot-1",
        "--mnemonic", "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
    ]);
    assert!(cli.is_ok(), "regen-hotkey with name: {:?}", cli.err());
}

// Additional serve IP validation tests (runtime validation tests)

#[test]
fn parse_serve_axon_max_port_value() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "serve", "axon",
        "--netuid", "1",
        "--ip", "8.8.8.8",
        "--port", "65535",
    ]);
    assert!(cli.is_ok(), "max port 65535: {:?}", cli.err());
}

#[test]
fn parse_serve_axon_negative_port() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "serve", "axon",
        "--netuid", "1",
        "--ip", "1.2.3.4",
        "--port", "-1",
    ]);
    assert!(cli.is_err(), "negative port should fail u16 parse");
}

#[test]
fn parse_serve_axon_non_numeric_port() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "serve", "axon",
        "--netuid", "1",
        "--ip", "1.2.3.4",
        "--port", "abc",
    ]);
    assert!(cli.is_err(), "non-numeric port should fail");
}

#[test]
fn parse_serve_axon_protocol_overflow() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "serve", "axon",
        "--netuid", "1",
        "--ip", "1.2.3.4",
        "--port", "8080",
        "--protocol", "256",
    ]);
    assert!(cli.is_err(), "protocol 256 should overflow u8");
}

// ──── Transfer SS58 validation (CLI parsing tests) ────

#[test]
fn parse_transfer_valid_ss58_dest() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "transfer",
        "--dest", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--amount", "1.0",
    ]);
    assert!(cli.is_ok(), "valid SS58 dest should parse");
}

#[test]
fn parse_transfer_no_dest_arg() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "transfer",
        "--amount", "1.0",
    ]);
    assert!(cli.is_err(), "missing dest should fail");
}

#[test]
fn parse_transfer_no_amount_arg() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "transfer",
        "--dest", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_err(), "missing amount should fail");
}

#[test]
fn parse_transfer_all_bob_dest() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "transfer-all",
        "--dest", "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
    ]);
    assert!(cli.is_ok(), "transfer-all with valid dest should parse");
}

#[test]
fn parse_transfer_all_with_keep_alive() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "transfer-all",
        "--dest", "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--keep-alive",
    ]);
    assert!(cli.is_ok(), "transfer-all with --keep-alive should parse");
}

#[test]
fn parse_transfer_all_no_dest_arg() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "transfer-all",
    ]);
    assert!(cli.is_err(), "transfer-all without dest should fail");
}

// ──── Proxy command CLI parsing ────

#[test]
fn parse_proxy_add_ss58_delegate() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "proxy", "add",
        "--delegate", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--proxy-type", "Any",
    ]);
    assert!(cli.is_ok(), "proxy add with valid delegate should parse");
}

#[test]
fn parse_proxy_add_no_delegate_arg() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "proxy", "add",
        "--proxy-type", "Any",
    ]);
    assert!(cli.is_err(), "proxy add without delegate should fail");
}

#[test]
fn parse_proxy_remove_ss58_delegate() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "proxy", "remove",
        "--delegate", "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--proxy-type", "Staking",
    ]);
    assert!(cli.is_ok(), "proxy remove with valid delegate should parse");
}

#[test]
fn parse_proxy_add_delay_100() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "proxy", "add",
        "--delegate", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--proxy-type", "Any",
        "--delay", "100",
    ]);
    assert!(cli.is_ok(), "proxy add with delay should parse");
}

// ──── Swap command CLI parsing ────

#[test]
fn parse_swap_hotkey_ss58() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "swap", "hotkey",
        "--new-hotkey", "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
    ]);
    assert!(cli.is_ok(), "swap hotkey with valid address should parse");
}

#[test]
fn parse_swap_hotkey_no_arg() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "swap", "hotkey",
    ]);
    assert!(cli.is_err(), "swap hotkey without new-hotkey should fail");
}

#[test]
fn parse_swap_coldkey_ss58() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "swap", "coldkey",
        "--new-coldkey", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "swap coldkey with valid address should parse");
}

#[test]
fn parse_swap_coldkey_no_arg() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "swap", "coldkey",
    ]);
    assert!(cli.is_err(), "swap coldkey without new-coldkey should fail");
}

// ──── Stake transfer-stake CLI parsing ────

#[test]
fn parse_stake_transfer_stake_all_required() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "transfer-stake",
        "--dest", "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--amount", "10.0",
        "--from", "1",
        "--to", "2",
    ]);
    assert!(cli.is_ok(), "transfer-stake with all required args should parse");
}

#[test]
fn parse_stake_transfer_stake_no_dest() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "transfer-stake",
        "--amount", "10.0",
        "--from", "1",
        "--to", "2",
    ]);
    assert!(cli.is_err(), "transfer-stake without dest should fail");
}

#[test]
fn parse_stake_transfer_stake_no_amount() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "transfer-stake",
        "--dest", "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--from", "1",
        "--to", "2",
    ]);
    assert!(cli.is_err(), "transfer-stake without amount should fail");
}

#[test]
fn parse_stake_transfer_stake_optional_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "transfer-stake",
        "--dest", "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--amount", "10.0",
        "--from", "1",
        "--to", "2",
        "--hotkey", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "transfer-stake with --hotkey should parse");
}

// ──── Serve batch-axon CLI parsing ────

#[test]
fn parse_serve_batch_axon_with_file() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "serve", "batch-axon",
        "--file", "/tmp/axons.json",
    ]);
    assert!(cli.is_ok(), "batch-axon with --file should parse");
}

#[test]
fn parse_serve_batch_axon_no_file_arg() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "serve", "batch-axon",
    ]);
    assert!(cli.is_err(), "batch-axon without --file should fail");
}

// ──── Serve axon port boundary tests ────

#[test]
fn parse_serve_axon_port_zero_parses() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "serve", "axon",
        "--netuid", "1",
        "--ip", "1.2.3.4",
        "--port", "0",
    ]);
    assert!(cli.is_ok(), "port 0 should parse (rejected at runtime by validate_port)");
}

#[test]
fn parse_serve_axon_port_max_65535() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "serve", "axon",
        "--netuid", "1",
        "--ip", "1.2.3.4",
        "--port", "65535",
    ]);
    assert!(cli.is_ok(), "max port 65535 should parse");
}

#[test]
fn parse_serve_axon_port_65536_overflow() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "serve", "axon",
        "--netuid", "1",
        "--ip", "1.2.3.4",
        "--port", "65536",
    ]);
    assert!(cli.is_err(), "port 65536 should overflow u16");
}

// ──── Additional global flags edge cases ────

#[test]
fn parse_balance_ss58_address() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "balance",
        "--address", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "balance with valid address should parse");
}

#[test]
fn parse_balance_watch_30s() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "balance",
        "--watch", "30",
    ]);
    assert!(cli.is_ok(), "balance with --watch interval should parse");
}

#[test]
fn parse_balance_at_specific_block() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "balance",
        "--at-block", "1000000",
    ]);
    assert!(cli.is_ok(), "balance with --at-block should parse");
}

#[test]
fn parse_global_timeout_30() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--timeout", "30", "balance",
    ]);
    assert!(cli.is_ok(), "global --timeout flag should parse");
}

#[test]
fn parse_global_log_file() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--log-file", "/tmp/agcli.log", "balance",
    ]);
    assert!(cli.is_ok(), "global --log-file flag should parse");
}

#[test]
fn parse_global_best_endpoint() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--best", "balance",
    ]);
    assert!(cli.is_ok(), "global --best flag should parse");
}

#[test]
fn parse_global_debug_mode() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--debug", "balance",
    ]);
    assert!(cli.is_ok(), "global --debug flag should parse");
}

// ──── wallet sign/verify/derive CLI edge cases ────

#[test]
fn parse_wallet_sign_hex_message() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "sign", "--message", "0xdeadbeef",
    ]);
    assert!(cli.is_ok(), "wallet sign hex message: {:?}", cli.err());
}

#[test]
fn parse_wallet_sign_empty_message() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "sign", "--message", "",
    ]);
    assert!(cli.is_ok(), "wallet sign empty message should parse: {:?}", cli.err());
}

#[test]
fn parse_wallet_sign_unicode_message() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "sign", "--message", "Hello 🌐🔑",
    ]);
    assert!(cli.is_ok(), "wallet sign unicode: {:?}", cli.err());
}

#[test]
fn parse_wallet_sign_long_message() {
    let long_msg = "a".repeat(10_000);
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "sign", "--message", &long_msg,
    ]);
    assert!(cli.is_ok(), "wallet sign long message: {:?}", cli.err());
}

#[test]
fn parse_wallet_sign_missing_message() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "wallet", "sign"]);
    assert!(cli.is_err(), "wallet sign without --message should fail");
}

#[test]
fn parse_wallet_sign_with_wallet_flag() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--wallet", "mywallet", "wallet", "sign", "--message", "test",
    ]);
    assert!(cli.is_ok(), "wallet sign with --wallet: {:?}", cli.err());
}

#[test]
fn parse_wallet_verify_without_signer() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "verify",
        "--message", "hello",
        "--signature", "0xabcd1234",
    ]);
    assert!(cli.is_ok(), "wallet verify without --signer: {:?}", cli.err());
}

#[test]
fn parse_wallet_verify_with_signer() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "verify",
        "--message", "hello",
        "--signature", "0xabcd1234",
        "--signer", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "wallet verify with --signer: {:?}", cli.err());
}

#[test]
fn parse_wallet_verify_missing_signature() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "verify",
        "--message", "hello",
    ]);
    assert!(cli.is_err(), "wallet verify without --signature should fail");
}

#[test]
fn parse_wallet_verify_missing_message() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "verify",
        "--signature", "0xabcd1234",
    ]);
    assert!(cli.is_err(), "wallet verify without --message should fail");
}

#[test]
fn parse_wallet_verify_hex_message() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "verify",
        "--message", "0xdeadbeef",
        "--signature", "0x" ,
    ]);
    assert!(cli.is_ok(), "wallet verify hex message: {:?}", cli.err());
}

#[test]
fn parse_wallet_derive_from_hex() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "derive",
        "--input", "0x0000000000000000000000000000000000000000000000000000000000000001",
    ]);
    assert!(cli.is_ok(), "wallet derive from hex pubkey: {:?}", cli.err());
}

#[test]
fn parse_wallet_derive_from_mnemonic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "derive",
        "--input", "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
    ]);
    assert!(cli.is_ok(), "wallet derive from mnemonic: {:?}", cli.err());
}

#[test]
fn parse_wallet_derive_missing_input() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "wallet", "derive"]);
    assert!(cli.is_err(), "wallet derive without --input should fail");
}

#[test]
fn parse_wallet_derive_with_output_json() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--output", "json", "wallet", "derive",
        "--input", "0x0000000000000000000000000000000000000000000000000000000000000001",
    ]);
    assert!(cli.is_ok(), "wallet derive with --output json: {:?}", cli.err());
}

// ──── multisig JSON args CLI edge cases ────

#[test]
fn parse_multisig_submit_with_complex_args() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "multisig", "submit",
        "--others", "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--threshold", "2",
        "--pallet", "Balances",
        "--call", "transfer_keep_alive",
        "--args", r#"[{"Id":"5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"},1000000000]"#,
    ]);
    assert!(cli.is_ok(), "multisig submit complex args: {:?}", cli.err());
}

#[test]
fn parse_multisig_submit_without_args() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "multisig", "submit",
        "--others", "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--threshold", "2",
        "--pallet", "System",
        "--call", "remark",
    ]);
    assert!(cli.is_ok(), "multisig submit without --args: {:?}", cli.err());
}

#[test]
fn parse_multisig_execute_with_timepoint() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "multisig", "execute",
        "--others", "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--threshold", "2",
        "--pallet", "Balances",
        "--call", "transfer_keep_alive",
        "--args", "[1000]",
        "--timepoint-height", "100",
        "--timepoint-index", "1",
    ]);
    assert!(cli.is_ok(), "multisig execute with timepoint: {:?}", cli.err());
}

#[test]
fn parse_multisig_execute_missing_pallet() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "multisig", "execute",
        "--others", "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--threshold", "2",
        "--call", "transfer_keep_alive",
    ]);
    assert!(cli.is_err(), "multisig execute without --pallet should fail");
}

// ──── wallet new-hotkey / regen-hotkey CLI edge cases ────

#[test]
fn parse_wallet_new_hotkey_custom_name() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "new-hotkey", "--name", "miner1",
    ]);
    assert!(cli.is_ok(), "wallet new-hotkey with custom name: {:?}", cli.err());
}

#[test]
fn parse_wallet_new_hotkey_no_name_arg() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "new-hotkey",
    ]);
    assert!(cli.is_err(), "wallet new-hotkey without --name should fail");
}

#[test]
fn parse_wallet_regen_hotkey_with_mnemonic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "regen-hotkey",
        "--name", "recovered",
        "--mnemonic", "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
    ]);
    assert!(cli.is_ok(), "wallet regen-hotkey: {:?}", cli.err());
}

#[test]
fn parse_wallet_regen_hotkey_default_name() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "regen-hotkey",
        "--mnemonic", "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
    ]);
    assert!(cli.is_ok(), "wallet regen-hotkey with default name: {:?}", cli.err());
}

#[test]
fn parse_wallet_regen_coldkey_with_mnemonic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "regen-coldkey",
        "--mnemonic", "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        "--password", "testpass",
    ]);
    assert!(cli.is_ok(), "wallet regen-coldkey: {:?}", cli.err());
}

#[test]
fn parse_wallet_show_mnemonic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "show-mnemonic", "--password", "mypass",
    ]);
    assert!(cli.is_ok(), "wallet show-mnemonic: {:?}", cli.err());
}

#[test]
fn parse_wallet_import_with_all_args() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "import",
        "--name", "imported",
        "--mnemonic", "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        "--password", "mypass",
    ]);
    assert!(cli.is_ok(), "wallet import with all args: {:?}", cli.err());
}

#[test]
fn parse_wallet_create_with_no_mnemonic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "create",
        "--name", "quiet",
        "--password", "pw",
        "--no-mnemonic",
    ]);
    assert!(cli.is_ok(), "wallet create with --no-mnemonic: {:?}", cli.err());
}

#[test]
fn parse_wallet_dev_key() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "dev-key", "--uri", "Alice",
    ]);
    assert!(cli.is_ok(), "wallet dev-key: {:?}", cli.err());
}

#[test]
fn parse_wallet_dev_key_with_double_slash() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "dev-key", "--uri", "//Alice",
    ]);
    assert!(cli.is_ok(), "wallet dev-key with //: {:?}", cli.err());
}

#[test]
fn parse_proxy_list_ss58_address() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "proxy", "list",
        "--address", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "proxy list with --address should parse");
}

#[test]
fn parse_proxy_create_pure_any() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "proxy", "create-pure",
        "--proxy-type", "Any",
    ]);
    assert!(cli.is_ok(), "proxy create-pure should parse");
}

#[test]
fn parse_proxy_kill_pure_full_args() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "proxy", "kill-pure",
        "--spawner", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--proxy-type", "Any",
        "--index", "0",
        "--height", "100",
        "--ext-index", "1",
    ]);
    assert!(cli.is_ok(), "proxy kill-pure with all required args should parse");
}

// =====================================================================
// Admin commands
// =====================================================================

#[test]
fn parse_admin_set_tempo() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "admin", "set-tempo",
        "--netuid", "1",
        "--tempo", "360",
    ]);
    assert!(cli.is_ok(), "admin set-tempo: {:?}", cli.err());
}

#[test]
fn parse_admin_set_tempo_with_sudo_key() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "admin", "set-tempo",
        "--netuid", "1",
        "--tempo", "100",
        "--sudo-key", "//Alice",
    ]);
    assert!(cli.is_ok(), "admin set-tempo with sudo-key: {:?}", cli.err());
}

#[test]
fn parse_admin_set_tempo_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "admin", "set-tempo",
        "--tempo", "360",
    ]);
    assert!(cli.is_err(), "admin set-tempo without --netuid should fail");
}

#[test]
fn parse_admin_set_tempo_missing_tempo() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "admin", "set-tempo",
        "--netuid", "1",
    ]);
    assert!(cli.is_err(), "admin set-tempo without --tempo should fail");
}

#[test]
fn parse_admin_set_max_validators() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "admin", "set-max-validators",
        "--netuid", "1",
        "--max", "256",
    ]);
    assert!(cli.is_ok(), "admin set-max-validators: {:?}", cli.err());
}

#[test]
fn parse_admin_set_max_validators_with_sudo() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "admin", "set-max-validators",
        "--netuid", "3",
        "--max", "64",
        "--sudo-key", "//Bob",
    ]);
    assert!(cli.is_ok(), "admin set-max-validators with sudo: {:?}", cli.err());
}

#[test]
fn parse_admin_set_max_validators_missing_max() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "admin", "set-max-validators",
        "--netuid", "1",
    ]);
    assert!(cli.is_err(), "admin set-max-validators without --max should fail");
}

#[test]
fn parse_admin_set_max_uids() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "admin", "set-max-uids",
        "--netuid", "1",
        "--max", "4096",
    ]);
    assert!(cli.is_ok(), "admin set-max-uids: {:?}", cli.err());
}

#[test]
fn parse_admin_set_max_uids_with_sudo() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "admin", "set-max-uids",
        "--netuid", "2",
        "--max", "1024",
        "--sudo-key", "//Alice",
    ]);
    assert!(cli.is_ok(), "admin set-max-uids with sudo: {:?}", cli.err());
}

#[test]
fn parse_admin_set_immunity_period() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "admin", "set-immunity-period",
        "--netuid", "1",
        "--period", "7200",
    ]);
    assert!(cli.is_ok(), "admin set-immunity-period: {:?}", cli.err());
}

#[test]
fn parse_admin_set_immunity_period_zero() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "admin", "set-immunity-period",
        "--netuid", "1",
        "--period", "0",
    ]);
    assert!(cli.is_ok(), "admin set-immunity-period zero: {:?}", cli.err());
}

#[test]
fn parse_admin_set_min_weights() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "admin", "set-min-weights",
        "--netuid", "1",
        "--min", "10",
    ]);
    assert!(cli.is_ok(), "admin set-min-weights: {:?}", cli.err());
}

#[test]
fn parse_admin_set_min_weights_with_sudo() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "admin", "set-min-weights",
        "--netuid", "1",
        "--min", "0",
        "--sudo-key", "//Alice",
    ]);
    assert!(cli.is_ok(), "admin set-min-weights with sudo: {:?}", cli.err());
}

#[test]
fn parse_admin_set_max_weight_limit() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "admin", "set-max-weight-limit",
        "--netuid", "1",
        "--limit", "65535",
    ]);
    assert!(cli.is_ok(), "admin set-max-weight-limit: {:?}", cli.err());
}

#[test]
fn parse_admin_set_max_weight_limit_missing_limit() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "admin", "set-max-weight-limit",
        "--netuid", "1",
    ]);
    assert!(cli.is_err(), "admin set-max-weight-limit without --limit should fail");
}

#[test]
fn parse_admin_set_weights_rate_limit() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "admin", "set-weights-rate-limit",
        "--netuid", "1",
        "--limit", "100",
    ]);
    assert!(cli.is_ok(), "admin set-weights-rate-limit: {:?}", cli.err());
}

#[test]
fn parse_admin_set_weights_rate_limit_unlimited() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "admin", "set-weights-rate-limit",
        "--netuid", "1",
        "--limit", "0",
    ]);
    assert!(cli.is_ok(), "admin set-weights-rate-limit unlimited (0): {:?}", cli.err());
}

#[test]
fn parse_admin_set_commit_reveal_enable() {
    // --enabled is a bool flag: present = true, absent = false
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "admin", "set-commit-reveal",
        "--netuid", "1",
        "--enabled",
    ]);
    assert!(cli.is_ok(), "admin set-commit-reveal enable: {:?}", cli.err());
}

#[test]
fn parse_admin_set_commit_reveal_disable() {
    // Omitting --enabled means enabled = false
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "admin", "set-commit-reveal",
        "--netuid", "1",
    ]);
    assert!(cli.is_ok(), "admin set-commit-reveal disable: {:?}", cli.err());
}

#[test]
fn parse_admin_set_commit_reveal_with_sudo() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "admin", "set-commit-reveal",
        "--netuid", "1",
        "--enabled",
        "--sudo-key", "//Alice",
    ]);
    assert!(cli.is_ok(), "admin set-commit-reveal with sudo: {:?}", cli.err());
}

#[test]
fn parse_admin_set_difficulty() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "admin", "set-difficulty",
        "--netuid", "1",
        "--difficulty", "1000000",
    ]);
    assert!(cli.is_ok(), "admin set-difficulty: {:?}", cli.err());
}

#[test]
fn parse_admin_set_difficulty_with_sudo() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "admin", "set-difficulty",
        "--netuid", "1",
        "--difficulty", "999999999",
        "--sudo-key", "//Alice",
    ]);
    assert!(cli.is_ok(), "admin set-difficulty with sudo: {:?}", cli.err());
}

#[test]
fn parse_admin_set_activity_cutoff() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "admin", "set-activity-cutoff",
        "--netuid", "1",
        "--cutoff", "5000",
    ]);
    assert!(cli.is_ok(), "admin set-activity-cutoff: {:?}", cli.err());
}

#[test]
fn parse_admin_set_activity_cutoff_missing_cutoff() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "admin", "set-activity-cutoff",
        "--netuid", "1",
    ]);
    assert!(cli.is_err(), "admin set-activity-cutoff without --cutoff should fail");
}

#[test]
fn parse_admin_raw() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "admin", "raw",
        "--call", "sudo_set_tempo",
        "--args", "[1, 100]",
    ]);
    assert!(cli.is_ok(), "admin raw: {:?}", cli.err());
}

#[test]
fn parse_admin_raw_with_sudo_key() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "admin", "raw",
        "--call", "sudo_set_max_allowed_validators",
        "--args", "[1, 256]",
        "--sudo-key", "//Alice",
    ]);
    assert!(cli.is_ok(), "admin raw with sudo-key: {:?}", cli.err());
}

#[test]
fn parse_admin_raw_missing_call() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "admin", "raw",
        "--args", "[1, 100]",
    ]);
    assert!(cli.is_err(), "admin raw without --call should fail");
}

#[test]
fn parse_admin_raw_missing_args() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "admin", "raw",
        "--call", "sudo_set_tempo",
    ]);
    assert!(cli.is_err(), "admin raw without --args should fail");
}

#[test]
fn parse_admin_list() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "admin", "list",
    ]);
    assert!(cli.is_ok(), "admin list: {:?}", cli.err());
}

// =====================================================================
// Scheduler commands
// =====================================================================

#[test]
fn parse_scheduler_schedule_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "scheduler", "schedule",
        "--when", "1000",
        "--pallet", "Balances",
        "--call", "transfer_allow_death",
    ]);
    assert!(cli.is_ok(), "scheduler schedule basic: {:?}", cli.err());
}

#[test]
fn parse_scheduler_schedule_with_args() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "scheduler", "schedule",
        "--when", "500",
        "--pallet", "SubtensorModule",
        "--call", "add_stake",
        "--args", "[1, \"5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY\", 1000000000]",
    ]);
    assert!(cli.is_ok(), "scheduler schedule with args: {:?}", cli.err());
}

#[test]
fn parse_scheduler_schedule_with_priority() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "scheduler", "schedule",
        "--when", "2000",
        "--pallet", "Balances",
        "--call", "transfer_allow_death",
        "--priority", "0",
    ]);
    assert!(cli.is_ok(), "scheduler schedule with priority: {:?}", cli.err());
}

#[test]
fn parse_scheduler_schedule_with_repeat() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "scheduler", "schedule",
        "--when", "100",
        "--pallet", "System",
        "--call", "remark",
        "--repeat-every", "50",
        "--repeat-count", "10",
    ]);
    assert!(cli.is_ok(), "scheduler schedule with repeat: {:?}", cli.err());
}

#[test]
fn parse_scheduler_schedule_full_args() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "scheduler", "schedule",
        "--when", "3000",
        "--pallet", "SubtensorModule",
        "--call", "set_weights",
        "--args", "[1, [0,1], [100,200], 0]",
        "--priority", "255",
        "--repeat-every", "100",
        "--repeat-count", "5",
    ]);
    assert!(cli.is_ok(), "scheduler schedule full args: {:?}", cli.err());
}

#[test]
fn parse_scheduler_schedule_missing_when() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "scheduler", "schedule",
        "--pallet", "Balances",
        "--call", "transfer_allow_death",
    ]);
    assert!(cli.is_err(), "scheduler schedule without --when should fail");
}

#[test]
fn parse_scheduler_schedule_missing_pallet() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "scheduler", "schedule",
        "--when", "100",
        "--call", "transfer_allow_death",
    ]);
    assert!(cli.is_err(), "scheduler schedule without --pallet should fail");
}

#[test]
fn parse_scheduler_schedule_missing_call() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "scheduler", "schedule",
        "--when", "100",
        "--pallet", "Balances",
    ]);
    assert!(cli.is_err(), "scheduler schedule without --call should fail");
}

#[test]
fn parse_scheduler_schedule_named_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "scheduler", "schedule-named",
        "--id", "my_task_1",
        "--when", "5000",
        "--pallet", "Balances",
        "--call", "transfer_allow_death",
    ]);
    assert!(cli.is_ok(), "scheduler schedule-named: {:?}", cli.err());
}

#[test]
fn parse_scheduler_schedule_named_full() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "scheduler", "schedule-named",
        "--id", "recurring_stake",
        "--when", "1000",
        "--pallet", "SubtensorModule",
        "--call", "add_stake",
        "--args", "[1, \"5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY\", 500000000]",
        "--priority", "64",
        "--repeat-every", "200",
        "--repeat-count", "100",
    ]);
    assert!(cli.is_ok(), "scheduler schedule-named full: {:?}", cli.err());
}

#[test]
fn parse_scheduler_schedule_named_missing_id() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "scheduler", "schedule-named",
        "--when", "5000",
        "--pallet", "Balances",
        "--call", "transfer_allow_death",
    ]);
    assert!(cli.is_err(), "scheduler schedule-named without --id should fail");
}

#[test]
fn parse_scheduler_cancel() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "scheduler", "cancel",
        "--when", "1000",
        "--index", "0",
    ]);
    assert!(cli.is_ok(), "scheduler cancel: {:?}", cli.err());
}

#[test]
fn parse_scheduler_cancel_missing_when() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "scheduler", "cancel",
        "--index", "0",
    ]);
    assert!(cli.is_err(), "scheduler cancel without --when should fail");
}

#[test]
fn parse_scheduler_cancel_missing_index() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "scheduler", "cancel",
        "--when", "1000",
    ]);
    assert!(cli.is_err(), "scheduler cancel without --index should fail");
}

#[test]
fn parse_scheduler_cancel_named() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "scheduler", "cancel-named",
        "--id", "my_task_1",
    ]);
    assert!(cli.is_ok(), "scheduler cancel-named: {:?}", cli.err());
}

#[test]
fn parse_scheduler_cancel_named_missing_id() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "scheduler", "cancel-named",
    ]);
    assert!(cli.is_err(), "scheduler cancel-named without --id should fail");
}

// =====================================================================
// Preimage commands
// =====================================================================

#[test]
fn parse_preimage_note_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "preimage", "note",
        "--pallet", "SubtensorModule",
        "--call", "set_weights",
    ]);
    assert!(cli.is_ok(), "preimage note basic: {:?}", cli.err());
}

#[test]
fn parse_preimage_note_with_args() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "preimage", "note",
        "--pallet", "Balances",
        "--call", "transfer_allow_death",
        "--args", "[\"5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY\", 1000000000]",
    ]);
    assert!(cli.is_ok(), "preimage note with args: {:?}", cli.err());
}

#[test]
fn parse_preimage_note_missing_pallet() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "preimage", "note",
        "--call", "set_weights",
    ]);
    assert!(cli.is_err(), "preimage note without --pallet should fail");
}

#[test]
fn parse_preimage_note_missing_call() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "preimage", "note",
        "--pallet", "SubtensorModule",
    ]);
    assert!(cli.is_err(), "preimage note without --call should fail");
}

#[test]
fn parse_preimage_unnote() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "preimage", "unnote",
        "--hash", "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
    ]);
    assert!(cli.is_ok(), "preimage unnote: {:?}", cli.err());
}

#[test]
fn parse_preimage_unnote_missing_hash() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "preimage", "unnote",
    ]);
    assert!(cli.is_err(), "preimage unnote without --hash should fail");
}

// =====================================================================
// Contracts commands
// =====================================================================

#[test]
fn parse_contracts_upload_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "contracts", "upload",
        "--code", "/path/to/contract.wasm",
    ]);
    assert!(cli.is_ok(), "contracts upload basic: {:?}", cli.err());
}

#[test]
fn parse_contracts_upload_with_deposit() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "contracts", "upload",
        "--code", "/path/to/contract.wasm",
        "--storage-deposit-limit", "1000000000",
    ]);
    assert!(cli.is_ok(), "contracts upload with deposit: {:?}", cli.err());
}

#[test]
fn parse_contracts_upload_missing_code() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "contracts", "upload",
    ]);
    assert!(cli.is_err(), "contracts upload without --code should fail");
}

#[test]
fn parse_contracts_instantiate_minimal() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "contracts", "instantiate",
        "--code-hash", "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
    ]);
    assert!(cli.is_ok(), "contracts instantiate minimal: {:?}", cli.err());
}

#[test]
fn parse_contracts_instantiate_full() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "contracts", "instantiate",
        "--code-hash", "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
        "--value", "1000000",
        "--data", "0xdeadbeef",
        "--salt", "0x01020304",
        "--gas-ref-time", "50000000000",
        "--gas-proof-size", "2097152",
        "--storage-deposit-limit", "500000000",
    ]);
    assert!(cli.is_ok(), "contracts instantiate full: {:?}", cli.err());
}

#[test]
fn parse_contracts_instantiate_missing_code_hash() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "contracts", "instantiate",
    ]);
    assert!(cli.is_err(), "contracts instantiate without --code-hash should fail");
}

#[test]
fn parse_contracts_call_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "contracts", "call",
        "--contract", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--data", "0xdeadbeef",
    ]);
    assert!(cli.is_ok(), "contracts call basic: {:?}", cli.err());
}

#[test]
fn parse_contracts_call_full() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "contracts", "call",
        "--contract", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--data", "0x12345678",
        "--value", "500000",
        "--gas-ref-time", "20000000000",
        "--gas-proof-size", "524288",
        "--storage-deposit-limit", "100000000",
    ]);
    assert!(cli.is_ok(), "contracts call full: {:?}", cli.err());
}

#[test]
fn parse_contracts_call_missing_contract() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "contracts", "call",
        "--data", "0xdeadbeef",
    ]);
    assert!(cli.is_err(), "contracts call without --contract should fail");
}

#[test]
fn parse_contracts_call_missing_data() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "contracts", "call",
        "--contract", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_err(), "contracts call without --data should fail");
}

#[test]
fn parse_contracts_remove_code() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "contracts", "remove-code",
        "--code-hash", "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
    ]);
    assert!(cli.is_ok(), "contracts remove-code: {:?}", cli.err());
}

#[test]
fn parse_contracts_remove_code_missing_hash() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "contracts", "remove-code",
    ]);
    assert!(cli.is_err(), "contracts remove-code without --code-hash should fail");
}

// =====================================================================
// EVM commands
// =====================================================================

#[test]
fn parse_evm_call_minimal() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "evm", "call",
        "--source", "0x1234567890abcdef1234567890abcdef12345678",
        "--target", "0xabcdef1234567890abcdef1234567890abcdef12",
    ]);
    assert!(cli.is_ok(), "evm call minimal: {:?}", cli.err());
}

#[test]
fn parse_evm_call_full() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "evm", "call",
        "--source", "0x1234567890abcdef1234567890abcdef12345678",
        "--target", "0xabcdef1234567890abcdef1234567890abcdef12",
        "--input", "0xa9059cbb000000000000000000000000",
        "--value", "0x0000000000000000000000000000000000000000000000000000000000000064",
        "--gas-limit", "100000",
        "--max-fee-per-gas", "0x0000000000000000000000000000000000000000000000000000000000000010",
    ]);
    assert!(cli.is_ok(), "evm call full: {:?}", cli.err());
}

#[test]
fn parse_evm_call_missing_source() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "evm", "call",
        "--target", "0xabcdef1234567890abcdef1234567890abcdef12",
    ]);
    assert!(cli.is_err(), "evm call without --source should fail");
}

#[test]
fn parse_evm_call_missing_target() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "evm", "call",
        "--source", "0x1234567890abcdef1234567890abcdef12345678",
    ]);
    assert!(cli.is_err(), "evm call without --target should fail");
}

#[test]
fn parse_evm_call_custom_gas_limit() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "evm", "call",
        "--source", "0x0000000000000000000000000000000000000001",
        "--target", "0x0000000000000000000000000000000000000002",
        "--gas-limit", "500000",
    ]);
    assert!(cli.is_ok(), "evm call custom gas limit: {:?}", cli.err());
}

#[test]
fn parse_evm_withdraw() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "evm", "withdraw",
        "--address", "0x1234567890abcdef1234567890abcdef12345678",
        "--amount", "1000000000",
    ]);
    assert!(cli.is_ok(), "evm withdraw: {:?}", cli.err());
}

#[test]
fn parse_evm_withdraw_missing_address() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "evm", "withdraw",
        "--amount", "1000000000",
    ]);
    assert!(cli.is_err(), "evm withdraw without --address should fail");
}

#[test]
fn parse_evm_withdraw_missing_amount() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "evm", "withdraw",
        "--address", "0x1234567890abcdef1234567890abcdef12345678",
    ]);
    assert!(cli.is_err(), "evm withdraw without --amount should fail");
}

#[test]
fn parse_evm_withdraw_large_amount() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "evm", "withdraw",
        "--address", "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "--amount", "340282366920938463463374607431768211455",
    ]);
    assert!(cli.is_ok(), "evm withdraw u128::MAX: {:?}", cli.err());
}

// =====================================================================
// SafeMode commands
// =====================================================================

#[test]
fn parse_safe_mode_enter() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "safe-mode", "enter",
    ]);
    assert!(cli.is_ok(), "safe-mode enter: {:?}", cli.err());
}

#[test]
fn parse_safe_mode_extend() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "safe-mode", "extend",
    ]);
    assert!(cli.is_ok(), "safe-mode extend: {:?}", cli.err());
}

#[test]
fn parse_safe_mode_force_enter() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "safe-mode", "force-enter",
        "--duration", "100",
    ]);
    assert!(cli.is_ok(), "safe-mode force-enter: {:?}", cli.err());
}

#[test]
fn parse_safe_mode_force_enter_missing_duration() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "safe-mode", "force-enter",
    ]);
    assert!(cli.is_err(), "safe-mode force-enter without --duration should fail");
}

#[test]
fn parse_safe_mode_force_enter_large_duration() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "safe-mode", "force-enter",
        "--duration", "4294967295",
    ]);
    assert!(cli.is_ok(), "safe-mode force-enter max u32: {:?}", cli.err());
}

#[test]
fn parse_safe_mode_force_exit() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "safe-mode", "force-exit",
    ]);
    assert!(cli.is_ok(), "safe-mode force-exit: {:?}", cli.err());
}

// =====================================================================
// Drand commands
// =====================================================================

#[test]
fn parse_drand_write_pulse() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "drand", "write-pulse",
        "--payload", "0xdeadbeef",
        "--signature", "0xcafebabe",
    ]);
    assert!(cli.is_ok(), "drand write-pulse: {:?}", cli.err());
}

#[test]
fn parse_drand_write_pulse_missing_payload() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "drand", "write-pulse",
        "--signature", "0xcafebabe",
    ]);
    assert!(cli.is_err(), "drand write-pulse without --payload should fail");
}

#[test]
fn parse_drand_write_pulse_missing_signature() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "drand", "write-pulse",
        "--payload", "0xdeadbeef",
    ]);
    assert!(cli.is_err(), "drand write-pulse without --signature should fail");
}

// =====================================================================
// Admin commands — boundary value tests
// =====================================================================

#[test]
fn parse_admin_set_tempo_max_u16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "admin", "set-tempo",
        "--netuid", "65535",
        "--tempo", "65535",
    ]);
    assert!(cli.is_ok(), "admin set-tempo max u16: {:?}", cli.err());
}

#[test]
fn parse_admin_set_tempo_overflow_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "admin", "set-tempo",
        "--netuid", "65536",
        "--tempo", "360",
    ]);
    assert!(cli.is_err(), "admin set-tempo netuid > u16::MAX should fail");
}

#[test]
fn parse_admin_set_tempo_negative_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "admin", "set-tempo",
        "--netuid", "-1",
        "--tempo", "360",
    ]);
    assert!(cli.is_err(), "admin set-tempo negative netuid should fail");
}

#[test]
fn parse_admin_set_max_validators_zero() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "admin", "set-max-validators",
        "--netuid", "1",
        "--max", "0",
    ]);
    assert!(cli.is_ok(), "admin set-max-validators zero: {:?}", cli.err());
}

#[test]
fn parse_admin_set_weights_rate_limit_max_u64() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "admin", "set-weights-rate-limit",
        "--netuid", "1",
        "--limit", "18446744073709551615",
    ]);
    assert!(cli.is_ok(), "admin set-weights-rate-limit max u64: {:?}", cli.err());
}

#[test]
fn parse_admin_set_weights_rate_limit_overflow() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "admin", "set-weights-rate-limit",
        "--netuid", "1",
        "--limit", "18446744073709551616",
    ]);
    assert!(cli.is_err(), "admin set-weights-rate-limit > u64::MAX should fail");
}

#[test]
fn parse_admin_set_difficulty_zero() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "admin", "set-difficulty",
        "--netuid", "1",
        "--difficulty", "0",
    ]);
    assert!(cli.is_ok(), "admin set-difficulty zero: {:?}", cli.err());
}

// =====================================================================
// Scheduler — boundary / edge-case tests
// =====================================================================

#[test]
fn parse_scheduler_schedule_priority_min() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "scheduler", "schedule",
        "--when", "100",
        "--pallet", "System",
        "--call", "remark",
        "--priority", "0",
    ]);
    assert!(cli.is_ok(), "scheduler priority 0 (highest): {:?}", cli.err());
}

#[test]
fn parse_scheduler_schedule_priority_max() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "scheduler", "schedule",
        "--when", "100",
        "--pallet", "System",
        "--call", "remark",
        "--priority", "255",
    ]);
    assert!(cli.is_ok(), "scheduler priority 255 (lowest): {:?}", cli.err());
}

#[test]
fn parse_scheduler_schedule_priority_overflow() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "scheduler", "schedule",
        "--when", "100",
        "--pallet", "System",
        "--call", "remark",
        "--priority", "256",
    ]);
    assert!(cli.is_err(), "scheduler priority > 255 should fail");
}

#[test]
fn parse_scheduler_schedule_when_zero() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "scheduler", "schedule",
        "--when", "0",
        "--pallet", "System",
        "--call", "remark",
    ]);
    assert!(cli.is_ok(), "scheduler when=0: {:?}", cli.err());
}

#[test]
fn parse_scheduler_cancel_large_index() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "scheduler", "cancel",
        "--when", "999999",
        "--index", "4294967295",
    ]);
    assert!(cli.is_ok(), "scheduler cancel max u32 index: {:?}", cli.err());
}

#[test]
fn parse_scheduler_schedule_named_empty_id() {
    // Clap will accept an empty string for --id; runtime should validate
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "scheduler", "schedule-named",
        "--id", "",
        "--when", "100",
        "--pallet", "System",
        "--call", "remark",
    ]);
    assert!(cli.is_ok(), "scheduler schedule-named empty id (parses, runtime validates): {:?}", cli.err());
}

// =====================================================================
// Contracts — boundary tests
// =====================================================================

#[test]
fn parse_contracts_instantiate_defaults_only() {
    // Just code-hash; value, data, salt, gas all have defaults
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "contracts", "instantiate",
        "--code-hash", "0x0000000000000000000000000000000000000000000000000000000000000001",
    ]);
    assert!(cli.is_ok(), "contracts instantiate defaults: {:?}", cli.err());
}

#[test]
fn parse_contracts_call_value_and_gas() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "contracts", "call",
        "--contract", "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--data", "0x",
        "--value", "0",
        "--gas-ref-time", "1",
        "--gas-proof-size", "1",
    ]);
    assert!(cli.is_ok(), "contracts call min gas: {:?}", cli.err());
}

// =====================================================================
// EVM — boundary tests
// =====================================================================

#[test]
fn parse_evm_call_default_gas() {
    // Defaults: input="0x", value=0x00...00, gas_limit=21000, max_fee_per_gas=0x00...01
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "evm", "call",
        "--source", "0x0000000000000000000000000000000000000000",
        "--target", "0x0000000000000000000000000000000000000000",
    ]);
    assert!(cli.is_ok(), "evm call zero addresses with defaults: {:?}", cli.err());
}

#[test]
fn parse_evm_withdraw_zero_amount() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "evm", "withdraw",
        "--address", "0x1234567890abcdef1234567890abcdef12345678",
        "--amount", "0",
    ]);
    assert!(cli.is_ok(), "evm withdraw zero amount: {:?}", cli.err());
}

// =====================================================================
// Localnet commands — CLI parsing tests
// =====================================================================

#[test]
fn parse_localnet_start_defaults() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "localnet", "start",
    ]);
    assert!(cli.is_ok(), "localnet start defaults: {:?}", cli.err());
}

#[test]
fn parse_localnet_start_all_args() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "localnet", "start",
        "--image", "my-image:latest",
        "--container", "my_container",
        "--port", "9955",
        "--wait", "false",
        "--timeout", "300",
    ]);
    assert!(cli.is_ok(), "localnet start all args: {:?}", cli.err());
}

#[test]
fn parse_localnet_start_custom_port() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "localnet", "start",
        "--port", "8844",
    ]);
    assert!(cli.is_ok(), "localnet start port 8844: {:?}", cli.err());
}

#[test]
fn parse_localnet_start_port_zero() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "localnet", "start",
        "--port", "0",
    ]);
    assert!(cli.is_ok(), "localnet start port 0 (clap parse succeeds, runtime validates): {:?}", cli.err());
}

#[test]
fn parse_localnet_start_port_max() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "localnet", "start",
        "--port", "65535",
    ]);
    assert!(cli.is_ok(), "localnet start port 65535: {:?}", cli.err());
}

#[test]
fn parse_localnet_start_port_overflow() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "localnet", "start",
        "--port", "65536",
    ]);
    assert!(cli.is_err(), "localnet start port > 65535 should fail");
}

#[test]
fn parse_localnet_start_timeout_zero() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "localnet", "start",
        "--timeout", "0",
    ]);
    assert!(cli.is_ok(), "localnet start timeout 0: {:?}", cli.err());
}

#[test]
fn parse_localnet_start_wait_true() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "localnet", "start",
        "--wait", "true",
    ]);
    assert!(cli.is_ok(), "localnet start wait true: {:?}", cli.err());
}

#[test]
fn parse_localnet_stop_defaults() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "localnet", "stop",
    ]);
    assert!(cli.is_ok(), "localnet stop defaults: {:?}", cli.err());
}

#[test]
fn parse_localnet_stop_custom_container() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "localnet", "stop",
        "--container", "my_localnet",
    ]);
    assert!(cli.is_ok(), "localnet stop custom container: {:?}", cli.err());
}

#[test]
fn parse_localnet_status_defaults() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "localnet", "status",
    ]);
    assert!(cli.is_ok(), "localnet status defaults: {:?}", cli.err());
}

#[test]
fn parse_localnet_status_with_port() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "localnet", "status",
        "--port", "9944",
        "--container", "agcli_localnet",
    ]);
    assert!(cli.is_ok(), "localnet status with port: {:?}", cli.err());
}

#[test]
fn parse_localnet_reset_defaults() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "localnet", "reset",
    ]);
    assert!(cli.is_ok(), "localnet reset defaults: {:?}", cli.err());
}

#[test]
fn parse_localnet_reset_all_args() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "localnet", "reset",
        "--image", "custom:tag",
        "--container", "my_chain",
        "--port", "10000",
        "--timeout", "60",
    ]);
    assert!(cli.is_ok(), "localnet reset all args: {:?}", cli.err());
}

#[test]
fn parse_localnet_logs_defaults() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "localnet", "logs",
    ]);
    assert!(cli.is_ok(), "localnet logs defaults: {:?}", cli.err());
}

#[test]
fn parse_localnet_logs_with_tail() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "localnet", "logs",
        "--tail", "100",
    ]);
    assert!(cli.is_ok(), "localnet logs tail 100: {:?}", cli.err());
}

#[test]
fn parse_localnet_logs_with_container() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "localnet", "logs",
        "--container", "my_chain",
        "--tail", "50",
    ]);
    assert!(cli.is_ok(), "localnet logs container+tail: {:?}", cli.err());
}

#[test]
fn parse_localnet_scaffold_defaults() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "localnet", "scaffold",
    ]);
    assert!(cli.is_ok(), "localnet scaffold defaults: {:?}", cli.err());
}

#[test]
fn parse_localnet_scaffold_all_args() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "localnet", "scaffold",
        "--config", "scaffold.toml",
        "--image", "my-image:v1",
        "--port", "9955",
        "--no-start",
    ]);
    assert!(cli.is_ok(), "localnet scaffold all args: {:?}", cli.err());
}

#[test]
fn parse_localnet_scaffold_no_start_flag() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "localnet", "scaffold",
        "--no-start",
    ]);
    assert!(cli.is_ok(), "localnet scaffold no-start: {:?}", cli.err());
}

#[test]
fn parse_localnet_start_negative_port() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "localnet", "start",
        "--port", "-1",
    ]);
    assert!(cli.is_err(), "localnet start negative port should fail");
}

#[test]
fn parse_localnet_logs_tail_zero() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "localnet", "logs",
        "--tail", "0",
    ]);
    assert!(cli.is_ok(), "localnet logs tail 0: {:?}", cli.err());
}

#[test]
fn parse_localnet_reset_port_string() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "localnet", "reset",
        "--port", "abc",
    ]);
    assert!(cli.is_err(), "localnet reset non-numeric port should fail");
}

// =====================================================================
// Doctor command — CLI parsing tests
// =====================================================================

#[test]
fn parse_doctor_plain() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "doctor",
    ]);
    assert!(cli.is_ok(), "doctor plain: {:?}", cli.err());
}

#[test]
fn parse_doctor_json_output() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--output", "json", "doctor",
    ]);
    assert!(cli.is_ok(), "doctor json output: {:?}", cli.err());
}

#[test]
fn parse_doctor_with_network() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--network", "test", "doctor",
    ]);
    assert!(cli.is_ok(), "doctor with network: {:?}", cli.err());
}

#[test]
fn parse_doctor_with_wallet() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--wallet", "mywallet", "doctor",
    ]);
    assert!(cli.is_ok(), "doctor with wallet: {:?}", cli.err());
}

// =====================================================================
// EVM — additional boundary tests with validation
// =====================================================================

#[test]
fn parse_evm_call_max_gas_limit() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "evm", "call",
        "--source", "0x0000000000000000000000000000000000000001",
        "--target", "0x0000000000000000000000000000000000000002",
        "--gas-limit", "18446744073709551615",
    ]);
    assert!(cli.is_ok(), "evm call max u64 gas: {:?}", cli.err());
}

#[test]
fn parse_evm_call_overflow_gas_limit() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "evm", "call",
        "--source", "0x0000000000000000000000000000000000000001",
        "--target", "0x0000000000000000000000000000000000000002",
        "--gas-limit", "18446744073709551616",
    ]);
    assert!(cli.is_err(), "evm call gas > u64::MAX should fail");
}

#[test]
fn parse_evm_withdraw_max_u128() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "evm", "withdraw",
        "--address", "0x1234567890abcdef1234567890abcdef12345678",
        "--amount", "340282366920938463463374607431768211455",
    ]);
    assert!(cli.is_ok(), "evm withdraw max u128: {:?}", cli.err());
}

#[test]
fn parse_evm_withdraw_overflow_u128() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "evm", "withdraw",
        "--address", "0x1234567890abcdef1234567890abcdef12345678",
        "--amount", "340282366920938463463374607431768211456",
    ]);
    assert!(cli.is_err(), "evm withdraw > u128::MAX should fail");
}

#[test]
fn parse_evm_call_with_input_data() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "evm", "call",
        "--source", "0x0000000000000000000000000000000000000001",
        "--target", "0x0000000000000000000000000000000000000002",
        "--input", "0xa9059cbb0000000000000000000000000000000000000000000000000000000000000001",
    ]);
    assert!(cli.is_ok(), "evm call with ABI-encoded input: {:?}", cli.err());
}

#[test]
fn parse_evm_call_with_value() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "evm", "call",
        "--source", "0x0000000000000000000000000000000000000001",
        "--target", "0x0000000000000000000000000000000000000002",
        "--value", "0x0000000000000000000000000000000000000000000000000000000000000001",
    ]);
    assert!(cli.is_ok(), "evm call with value: {:?}", cli.err());
}

// =====================================================================
// Scheduler — additional validation-related tests
// =====================================================================

#[test]
fn parse_scheduler_schedule_repeat_both() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "scheduler", "schedule",
        "--when", "1000",
        "--pallet", "System",
        "--call", "remark",
        "--repeat-every", "100",
        "--repeat-count", "5",
    ]);
    assert!(cli.is_ok(), "scheduler schedule with repeat: {:?}", cli.err());
}

#[test]
fn parse_scheduler_schedule_repeat_every_only() {
    // Clap accepts partial repeats, runtime validates pair
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "scheduler", "schedule",
        "--when", "1000",
        "--pallet", "System",
        "--call", "remark",
        "--repeat-every", "100",
    ]);
    assert!(cli.is_ok(), "scheduler repeat-every alone (runtime validates): {:?}", cli.err());
}

#[test]
fn parse_scheduler_schedule_max_when() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "scheduler", "schedule",
        "--when", "4294967295",
        "--pallet", "System",
        "--call", "remark",
    ]);
    assert!(cli.is_ok(), "scheduler max u32 when: {:?}", cli.err());
}

#[test]
fn parse_scheduler_schedule_when_overflow() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "scheduler", "schedule",
        "--when", "4294967296",
        "--pallet", "System",
        "--call", "remark",
    ]);
    assert!(cli.is_err(), "scheduler when > u32::MAX should fail");
}

#[test]
fn parse_scheduler_cancel_named_long_id() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "scheduler", "cancel-named",
        "--id", "a_very_long_descriptive_task_name",
    ]);
    assert!(cli.is_ok(), "scheduler cancel-named long id: {:?}", cli.err());
}

// =====================================================================
// Contracts — additional boundary + missing field tests
// =====================================================================

#[test]
fn parse_contracts_instantiate_with_storage_limit() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "contracts", "instantiate",
        "--code-hash", "0x0000000000000000000000000000000000000000000000000000000000000001",
        "--storage-deposit-limit", "1000000",
    ]);
    assert!(cli.is_ok(), "contracts instantiate with storage limit: {:?}", cli.err());
}

#[test]
fn parse_contracts_upload_with_storage_limit() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "contracts", "upload",
        "--code", "/tmp/contract.wasm",
        "--storage-deposit-limit", "5000000",
    ]);
    assert!(cli.is_ok(), "contracts upload with storage limit: {:?}", cli.err());
}

#[test]
fn parse_contracts_instantiate_max_gas() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "contracts", "instantiate",
        "--code-hash", "0x0000000000000000000000000000000000000000000000000000000000000001",
        "--gas-ref-time", "18446744073709551615",
        "--gas-proof-size", "18446744073709551615",
    ]);
    assert!(cli.is_ok(), "contracts instantiate max u64 gas: {:?}", cli.err());
}
