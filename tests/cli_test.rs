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
    if let agcli::cli::Commands::Explain { topic } = &cli.unwrap().command {
        assert_eq!(topic.as_deref(), Some("coldkey-swap"));
    } else {
        panic!("wrong command variant (expected Explain)");
    }
}

#[test]
fn parse_explain_coldkey_alias_ckswap() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "explain", "--topic", "ckswap"]);
    assert!(cli.is_ok());
    if let agcli::cli::Commands::Explain { topic } = &cli.unwrap().command {
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
    assert!(matches!(cli.resolve_network(), agcli::types::network::Network::Archive));
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
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "commits",
        "--netuid",
        "1",
    ]);
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
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "status",
        "--netuid",
        "1",
    ]);
    assert!(
        cli.is_ok(),
        "should parse weights status: {:?}",
        cli.err()
    );
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
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--timeout", "60", "subnet", "list",
    ]);
    assert!(cli.is_ok(), "should parse --timeout with subnet list: {:?}", cli.err());
    assert_eq!(cli.unwrap().timeout, Some(60));
}

#[test]
fn parse_time_and_timeout_together() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--time", "--timeout", "120", "balance",
    ]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    assert!(cli.time);
    assert_eq!(cli.timeout, Some(120));
}

// ──── Sprint 5: Liquidity commands ────

#[test]
fn parse_liquidity_add() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "liquidity", "add", "--netuid", "1", "--price-low", "0.5",
        "--price-high", "2.0", "--amount", "1000000",
    ]);
    assert!(cli.is_ok(), "should parse liquidity add: {:?}", cli.err());
}

#[test]
fn parse_liquidity_add_with_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "liquidity", "add", "--netuid", "1", "--price-low", "0.1",
        "--price-high", "10.0", "--amount", "500000", "--hotkey", "5GhostHotkey",
    ]);
    assert!(cli.is_ok(), "should parse liquidity add with hotkey: {:?}", cli.err());
}

#[test]
fn parse_liquidity_remove() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "liquidity", "remove", "--netuid", "1", "--position-id", "42",
    ]);
    assert!(cli.is_ok(), "should parse liquidity remove: {:?}", cli.err());
}

#[test]
fn parse_liquidity_modify() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "liquidity", "modify", "--netuid", "1", "--position-id", "42",
        "--delta=-500",
    ]);
    assert!(cli.is_ok(), "should parse liquidity modify: {:?}", cli.err());
}

#[test]
fn parse_liquidity_modify_positive() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "liquidity", "modify", "--netuid", "1", "--position-id", "42",
        "--delta", "1000",
    ]);
    assert!(cli.is_ok(), "should parse liquidity modify positive: {:?}", cli.err());
}

#[test]
fn parse_liquidity_toggle() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "liquidity", "toggle", "--netuid", "1", "--enable",
    ]);
    assert!(cli.is_ok(), "should parse liquidity toggle: {:?}", cli.err());
}

// ──── Sprint 5: Auto-stake ────

#[test]
fn parse_stake_set_auto() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "set-auto", "--netuid", "1",
    ]);
    assert!(cli.is_ok(), "should parse stake set-auto: {:?}", cli.err());
}

#[test]
fn parse_stake_set_auto_with_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "set-auto", "--netuid", "1", "--hotkey", "5GhostHotkey",
    ]);
    assert!(cli.is_ok(), "should parse stake set-auto with hotkey: {:?}", cli.err());
}

// ──── Sprint 5: Root claim ────

#[test]
fn parse_stake_set_claim_swap() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "set-claim", "--claim-type", "swap",
    ]);
    assert!(cli.is_ok(), "should parse stake set-claim swap: {:?}", cli.err());
}

#[test]
fn parse_stake_set_claim_keep() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "set-claim", "--claim-type", "keep",
    ]);
    assert!(cli.is_ok(), "should parse stake set-claim keep: {:?}", cli.err());
}

#[test]
fn parse_stake_set_claim_keep_subnets() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "set-claim", "--claim-type", "keep-subnets", "--subnets", "1,3,5",
    ]);
    assert!(cli.is_ok(), "should parse stake set-claim keep-subnets: {:?}", cli.err());
}

// ──── Sprint 5: Crowdloan expanded commands ────

#[test]
fn parse_crowdloan_create() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "crowdloan", "create", "--deposit", "10.0", "--min-contribution", "0.1",
        "--cap", "1000.0", "--end-block", "5000000",
    ]);
    assert!(cli.is_ok(), "should parse crowdloan create: {:?}", cli.err());
}

#[test]
fn parse_crowdloan_create_with_target() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "crowdloan", "create", "--deposit", "10.0", "--min-contribution", "0.1",
        "--cap", "1000.0", "--end-block", "5000000", "--target", "5GhostTarget",
    ]);
    assert!(cli.is_ok(), "should parse crowdloan create with target: {:?}", cli.err());
}

#[test]
fn parse_crowdloan_refund() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "crowdloan", "refund", "--crowdloan-id", "42",
    ]);
    assert!(cli.is_ok(), "should parse crowdloan refund: {:?}", cli.err());
}

#[test]
fn parse_crowdloan_dissolve() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "crowdloan", "dissolve", "--crowdloan-id", "42",
    ]);
    assert!(cli.is_ok(), "should parse crowdloan dissolve: {:?}", cli.err());
}

#[test]
fn parse_crowdloan_update_cap() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "crowdloan", "update-cap", "--crowdloan-id", "42", "--cap", "2000.0",
    ]);
    assert!(cli.is_ok(), "should parse crowdloan update-cap: {:?}", cli.err());
}

#[test]
fn parse_crowdloan_update_end() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "crowdloan", "update-end", "--crowdloan-id", "42", "--end-block", "6000000",
    ]);
    assert!(cli.is_ok(), "should parse crowdloan update-end: {:?}", cli.err());
}

#[test]
fn parse_crowdloan_update_min_contribution() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "crowdloan", "update-min-contribution", "--crowdloan-id", "42",
        "--min-contribution", "0.5",
    ]);
    assert!(cli.is_ok(), "should parse crowdloan update-min-contribution: {:?}", cli.err());
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
    assert!(cli.is_ok(), "should parse --mev with stake add: {:?}", cli.err());
    let cli = cli.unwrap();
    assert!(cli.mev);
}

#[test]
fn parse_mev_flag_with_stake_remove() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--mev", "stake", "remove", "--amount", "1.0", "--netuid", "1",
    ]);
    assert!(cli.is_ok(), "should parse --mev with stake remove: {:?}", cli.err());
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
        "agcli", "--mev", "--yes", "--verbose", "--time", "stake", "add",
        "--amount", "1.0", "--netuid", "1",
    ]);
    assert!(cli.is_ok(), "should parse multiple flags together: {:?}", cli.err());
    let cli = cli.unwrap();
    assert!(cli.mev);
    assert!(cli.yes);
    assert!(cli.verbose);
    assert!(cli.time);
}

// ──── Sprint 6: error message quality tests ────

#[test]
fn parse_stake_add_missing_netuid_error() {
    let result = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "add", "--amount", "1.0",
    ]);
    assert!(result.is_err(), "missing --netuid should error");
    let err = result.unwrap_err().to_string();
    assert!(err.contains("netuid"), "error should mention netuid: {}", err);
}

#[test]
fn parse_stake_add_missing_amount_error() {
    let result = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "add", "--netuid", "1",
    ]);
    assert!(result.is_err(), "missing --amount should error");
    let err = result.unwrap_err().to_string();
    assert!(err.contains("amount"), "error should mention amount: {}", err);
}

#[test]
fn parse_transfer_missing_dest_error() {
    let result = agcli::cli::Cli::try_parse_from([
        "agcli", "transfer", "--amount", "1.0",
    ]);
    assert!(result.is_err(), "missing --dest should error");
}

#[test]
fn parse_transfer_missing_amount_error() {
    let result = agcli::cli::Cli::try_parse_from([
        "agcli", "transfer", "--dest", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(result.is_err(), "missing --amount should error");
}

#[test]
fn parse_subnet_metagraph_missing_netuid_error() {
    let result = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "metagraph",
    ]);
    assert!(result.is_err(), "missing --netuid should error");
}

#[test]
fn parse_invalid_network_string_still_parses() {
    // Custom/unknown network strings are accepted and treated as Custom
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--network", "ws://custom:9944", "balance",
    ]);
    assert!(cli.is_ok(), "custom network string should parse");
}

#[test]
fn parse_timeout_zero_is_valid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--timeout", "0", "balance",
    ]);
    assert!(cli.is_ok());
    assert_eq!(cli.unwrap().timeout, Some(0));
}

#[test]
fn parse_timeout_large_value() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--timeout", "3600", "balance",
    ]);
    assert!(cli.is_ok());
    assert_eq!(cli.unwrap().timeout, Some(3600));
}

// ──────── subnet set-param ────────

#[test]
fn parse_subnet_set_param() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "set-param", "--netuid", "1", "--param", "tempo", "--value", "100",
    ]);
    assert!(cli.is_ok(), "subnet set-param should parse: {:?}", cli.err());
}

#[test]
fn parse_subnet_set_param_bool_value() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "set-param", "--netuid", "5", "--param", "registration_allowed", "--value", "false",
    ]);
    assert!(cli.is_ok(), "subnet set-param bool should parse: {:?}", cli.err());
}

#[test]
fn parse_subnet_set_param_list_mode() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "set-param", "--netuid", "1", "--param", "list",
    ]);
    assert!(cli.is_ok(), "subnet set-param list should parse: {:?}", cli.err());
}

#[test]
fn parse_subnet_set_param_requires_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "set-param", "--param", "tempo", "--value", "100",
    ]);
    assert!(cli.is_err(), "subnet set-param should require --netuid");
}

#[test]
fn parse_subnet_set_param_requires_param() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "set-param", "--netuid", "1", "--value", "100",
    ]);
    assert!(cli.is_err(), "subnet set-param should require --param");
}

#[test]
fn parse_subnet_set_param_value_is_optional() {
    // --value is optional (for list mode)
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "set-param", "--netuid", "1", "--param", "tempo",
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
        "agcli", "subnet", "trim", "--netuid", "1", "--max-uids", "256",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_subnet_check_start() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "check-start", "--netuid", "1",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_subnet_start() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "start", "--netuid", "1",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_subnet_mechanism_count() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "mechanism-count", "--netuid", "1",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_subnet_set_mechanism_count() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "set-mechanism-count", "--netuid", "1", "--count", "2",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_subnet_set_emission_split() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "set-emission-split", "--netuid", "1", "--weights", "50,50",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_serve_reset() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "serve", "reset", "--netuid", "1",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_stake_process_claim() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "process-claim",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_stake_process_claim_with_netuids() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "process-claim", "--netuids", "1,2,3",
    ]);
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
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "utils", "convert", "--amount", "1500000000",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_utils_latency() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "utils", "latency",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_utils_latency_with_extra() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "utils", "latency", "--extra", "wss://custom.node:9944", "--pings", "3",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}
