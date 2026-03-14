//! CLI parsing and non-interactive flag tests.
//! Run with: cargo test --test cli_test

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
    assert_eq!(cli.unwrap().output, "json");
}

/// Verify --output csv is accepted.
#[test]
fn parse_output_csv() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--output", "csv", "balance"]);
    assert!(cli.is_ok());
    assert_eq!(cli.unwrap().output, "csv");
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
    assert_eq!(cli.output, "json");
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
    assert_eq!(cli.unwrap().output, "csv");
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
    assert_eq!(cli.output, "csv");
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
    assert_eq!(cli.unwrap().output, "json");
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
    assert_eq!(cli.unwrap().output, "json");
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
    assert_eq!(cli.unwrap().output, "csv");
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
    assert_eq!(cli.unwrap().output, "json");
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
    assert_eq!(cli.output, "json");
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
    assert_eq!(parsed.output, "json");
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
