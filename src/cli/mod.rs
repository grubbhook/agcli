//! CLI command definitions and handlers.

pub mod commands;
pub mod helpers;
pub mod stake_cmds;
pub mod view_cmds;
pub mod wallet_cmds;

use crate::types::network::Network;
use clap::{Parser, Subcommand};

/// agcli — Rust CLI for the Bittensor network
#[derive(Parser, Debug)]
#[command(name = "agcli", version, about, long_about = None)]
pub struct Cli {
    /// Network to connect to
    #[arg(long, short, default_value = "finney", env = "AGCLI_NETWORK")]
    pub network: String,

    /// Custom chain endpoint (overrides --network)
    #[arg(long, env = "AGCLI_ENDPOINT")]
    pub endpoint: Option<String>,

    /// Wallet directory
    #[arg(long, default_value = "~/.bittensor/wallets", env = "AGCLI_WALLET_DIR")]
    pub wallet_dir: String,

    /// Wallet name
    #[arg(long, short, default_value = "default", env = "AGCLI_WALLET")]
    pub wallet: String,

    /// Hotkey name
    #[arg(long, default_value = "default", env = "AGCLI_HOTKEY")]
    pub hotkey: String,

    /// Output format
    #[arg(long, default_value = "table", value_parser = ["table", "json", "csv"])]
    pub output: String,

    /// Enable live polling mode (interval in seconds, default 12)
    #[arg(long)]
    pub live: Option<Option<u64>>,

    /// Proxy account SS58 — wrap all extrinsics through Proxy.proxy
    #[arg(long, env = "AGCLI_PROXY")]
    pub proxy: Option<String>,

    /// Skip all confirmation prompts (for non-interactive / agent use)
    #[arg(long, short = 'y', global = true, env = "AGCLI_YES")]
    pub yes: bool,

    /// Batch mode: all missing args are hard errors, never prompt for input
    #[arg(long, global = true, env = "AGCLI_BATCH")]
    pub batch: bool,

    /// Pretty-print JSON output (when --output json)
    #[arg(long, global = true)]
    pub pretty: bool,

    /// Verbose output: show connection info and query timing
    #[arg(long, short = 'v', global = true)]
    pub verbose: bool,

    /// Debug output: show all RPC calls and detailed diagnostics
    #[arg(long, global = true)]
    pub debug: bool,

    /// Write logs to a file (in addition to stderr). Supports daily rotation.
    #[arg(long, global = true, env = "AGCLI_LOG_FILE")]
    pub log_file: Option<String>,

    /// Global operation timeout in seconds (0 or omitted = no timeout)
    #[arg(long, global = true, env = "AGCLI_TIMEOUT")]
    pub timeout: Option<u64>,

    /// Print operation timing to stderr
    #[arg(long, global = true)]
    pub time: bool,

    /// Wallet password (avoids interactive prompt; prefer env var for security)
    #[arg(long, global = true, env = "AGCLI_PASSWORD", hide_env_values = true)]
    pub password: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    // ──── Wallet ────
    /// Wallet management
    #[command(subcommand)]
    Wallet(WalletCommands),

    // ──── Balance ────
    /// Show account balance (or watch with --watch --threshold)
    Balance {
        /// SS58 address (defaults to wallet coldkey)
        #[arg(long)]
        address: Option<String>,
        /// Watch mode: poll balance every N seconds (default 60)
        #[arg(long)]
        watch: Option<Option<u64>>,
        /// Alert threshold: warn when balance drops below this TAO amount
        #[arg(long)]
        threshold: Option<f64>,
        /// Query balance at a specific block number (historical wayback)
        #[arg(long)]
        at_block: Option<u32>,
    },

    // ──── Transfer ────
    /// Transfer TAO to another account
    Transfer {
        /// Destination SS58 address
        #[arg(long)]
        dest: String,
        /// Amount of TAO to send
        #[arg(long)]
        amount: f64,
    },

    /// Transfer entire balance to another account (minus fees)
    TransferAll {
        /// Destination SS58 address
        #[arg(long)]
        dest: String,
        /// Keep sender account alive (leave existential deposit)
        #[arg(long)]
        keep_alive: bool,
    },

    // ──── Staking ────
    /// Staking operations
    #[command(subcommand)]
    Stake(StakeCommands),

    // ──── Subnets ────
    /// Subnet operations
    #[command(subcommand)]
    Subnet(SubnetCommands),

    // ──── Weights ────
    /// Weight setting operations
    #[command(subcommand)]
    Weights(WeightCommands),

    // ──── Root ────
    /// Root network operations
    #[command(subcommand)]
    Root(RootCommands),

    // ──── Delegate ────
    /// Delegate operations (take management)
    #[command(subcommand)]
    Delegate(DelegateCommands),

    // ──── Info ────
    /// View detailed information
    #[command(subcommand)]
    View(ViewCommands),

    // ──── Identity ────
    /// On-chain identity operations
    #[command(subcommand)]
    Identity(IdentityCommands),

    // ──── Serve ────
    /// Serve axon endpoint (for miners)
    #[command(subcommand)]
    Serve(ServeCommands),

    // ──── Proxy ────
    /// Proxy account management (add/remove)
    #[command(subcommand)]
    Proxy(ProxyCommands),

    // ──── Swap ────
    /// Key swap operations
    #[command(subcommand)]
    Swap(SwapCommands),

    // ──── Subscribe ────
    /// Subscribe to real-time chain events
    #[command(subcommand)]
    Subscribe(SubscribeCommands),

    // ──── Multisig ────
    /// Multi-signature account operations
    #[command(subcommand)]
    Multisig(MultisigCommands),

    // ──── Crowdloan ────
    /// Crowdloan operations (create, contribute, withdraw, finalize, refund, dissolve)
    #[command(subcommand)]
    Crowdloan(CrowdloanCommands),

    // ──── Liquidity ────
    /// Liquidity pool management (add, remove, modify positions)
    #[command(subcommand)]
    Liquidity(LiquidityCommands),

    // ──── Config ────
    /// Manage persistent configuration (~/.agcli/config.toml)
    #[command(subcommand)]
    Config(ConfigCommands),

    // ──── Completions ────
    /// Generate shell completions (bash, zsh, fish, powershell)
    Completions {
        /// Shell to generate completions for
        #[arg(long, value_parser = ["bash", "zsh", "fish", "powershell"])]
        shell: String,
    },

    // ──── Update ────
    /// Self-update agcli to the latest version from GitHub
    Update,

    // ──── Doctor ────
    /// Diagnostic check: test connectivity, wallet health, chain version, and latency
    Doctor,

    // ──── Explain ────
    /// Built-in Bittensor concept reference (tempo, commit-reveal, AMM, etc.)
    Explain {
        /// Topic to explain (e.g., tempo, commit-reveal, amm, bootstrap)
        /// Omit to list all available topics.
        #[arg(long)]
        topic: Option<String>,
    },

    // ──── Audit ────
    /// Security audit of an account: proxies, delegates, stake exposure, permissions
    Audit {
        /// SS58 address to audit (defaults to wallet coldkey)
        #[arg(long)]
        address: Option<String>,
    },

    // ──── Block ────
    /// Block explorer (info, latest, range)
    #[command(subcommand)]
    Block(BlockCommands),

    // ──── Diff ────
    /// Compare chain state between two blocks (portfolio, subnet, network)
    #[command(subcommand)]
    Diff(DiffCommands),

    // ──── Batch ────
    /// Submit multiple extrinsics from a JSON file via Utility.batch_all
    Batch {
        /// Path to JSON file containing an array of calls.
        /// Each call: {"pallet": "SubtensorModule", "call": "add_stake", "args": [...]}
        #[arg(long)]
        file: String,
        /// Use batch (fail-safe, continues on error) instead of batch_all (atomic)
        #[arg(long)]
        no_atomic: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum SubscribeCommands {
    /// Watch finalized blocks
    Blocks,
    /// Watch chain events (all, staking, registration, transfer, weights, subnet)
    Events {
        /// Event filter category
        #[arg(long, default_value = "all")]
        filter: String,
        /// Filter by subnet UID (only show events mentioning this netuid)
        #[arg(long)]
        netuid: Option<u16>,
        /// Filter by account SS58 (only show events involving this address)
        #[arg(long)]
        account: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum WalletCommands {
    /// Create a new wallet
    Create {
        /// Wallet name
        #[arg(long, default_value = "default")]
        name: String,
        /// Coldkey password (non-interactive)
        #[arg(long, env = "AGCLI_PASSWORD", hide_env_values = true)]
        password: Option<String>,
    },
    /// List all wallets
    List,
    /// Show wallet details (keys, addresses)
    Show {
        /// Show all hotkeys
        #[arg(long)]
        all: bool,
    },
    /// Import wallet from mnemonic
    Import {
        /// Wallet name
        #[arg(long, default_value = "default")]
        name: String,
        /// Mnemonic phrase (non-interactive)
        #[arg(long)]
        mnemonic: Option<String>,
        /// Coldkey password (non-interactive)
        #[arg(long, env = "AGCLI_PASSWORD", hide_env_values = true)]
        password: Option<String>,
    },
    /// Regenerate coldkey from mnemonic
    RegenColdkey {
        /// Mnemonic phrase (non-interactive)
        #[arg(long)]
        mnemonic: Option<String>,
        /// Coldkey password (non-interactive)
        #[arg(long, env = "AGCLI_PASSWORD", hide_env_values = true)]
        password: Option<String>,
    },
    /// Regenerate hotkey from mnemonic
    RegenHotkey {
        /// Hotkey name
        #[arg(long, default_value = "default")]
        name: String,
        /// Mnemonic phrase (non-interactive)
        #[arg(long)]
        mnemonic: Option<String>,
    },
    /// Create a new hotkey
    NewHotkey {
        /// Hotkey name
        #[arg(long)]
        name: String,
    },
    /// Sign an arbitrary message with the coldkey
    Sign {
        /// Message to sign (hex-encoded if prefixed with 0x, otherwise UTF-8)
        #[arg(long)]
        message: String,
    },
    /// Verify a signature against the coldkey
    Verify {
        /// Message that was signed
        #[arg(long)]
        message: String,
        /// Signature (hex-encoded, 0x prefix optional)
        #[arg(long)]
        signature: String,
        /// SS58 address of the signer (defaults to wallet coldkey)
        #[arg(long)]
        signer: Option<String>,
    },
    /// Derive SS58 address from a public key or mnemonic (no secrets printed)
    Derive {
        /// Public key (0x hex) or mnemonic phrase
        #[arg(long)]
        input: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum StakeCommands {
    /// Add stake to a hotkey on a subnet
    Add {
        /// Amount of TAO to stake
        #[arg(long)]
        amount: f64,
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Hotkey SS58 (defaults to wallet hotkey)
        #[arg(long)]
        hotkey: Option<String>,
        /// Maximum allowed slippage percentage (e.g., 2.0 for 2%). Aborts if exceeded.
        #[arg(long)]
        max_slippage: Option<f64>,
    },
    /// Remove stake from a hotkey on a subnet
    Remove {
        /// Amount to unstake
        #[arg(long)]
        amount: f64,
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Hotkey SS58
        #[arg(long)]
        hotkey: Option<String>,
        /// Maximum allowed slippage percentage (e.g., 2.0 for 2%). Aborts if exceeded.
        #[arg(long)]
        max_slippage: Option<f64>,
    },
    /// Show all stakes for current wallet
    List {
        /// Coldkey SS58 address
        #[arg(long)]
        address: Option<String>,
        /// Query stakes at a specific block number (historical wayback)
        #[arg(long)]
        at_block: Option<u32>,
    },
    /// Move stake between subnets
    Move {
        /// Amount of alpha to move
        #[arg(long)]
        amount: f64,
        /// Source subnet
        #[arg(long)]
        from: u16,
        /// Destination subnet
        #[arg(long)]
        to: u16,
        /// Hotkey SS58
        #[arg(long)]
        hotkey: Option<String>,
    },
    /// Swap stake between hotkeys on same subnet
    Swap {
        /// Amount
        #[arg(long)]
        amount: f64,
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Source hotkey
        #[arg(long)]
        from_hotkey: String,
        /// Destination hotkey
        #[arg(long)]
        to_hotkey: String,
    },
    /// Unstake all from a hotkey
    UnstakeAll {
        /// Hotkey SS58
        #[arg(long)]
        hotkey: Option<String>,
    },
    /// Claim root dividends
    ClaimRoot {
        /// Hotkey SS58
        #[arg(long)]
        hotkey: Option<String>,
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
    },
    /// Add stake with a limit price
    AddLimit {
        /// Amount of TAO
        #[arg(long)]
        amount: f64,
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Limit price
        #[arg(long)]
        price: f64,
        /// Allow partial fill
        #[arg(long)]
        partial: bool,
        /// Hotkey SS58
        #[arg(long)]
        hotkey: Option<String>,
    },
    /// Remove stake with limit price
    RemoveLimit {
        /// Amount of alpha
        #[arg(long)]
        amount: f64,
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Limit price
        #[arg(long)]
        price: f64,
        /// Allow partial fill
        #[arg(long)]
        partial: bool,
        /// Hotkey SS58
        #[arg(long)]
        hotkey: Option<String>,
    },
    /// Set childkey take
    ChildkeyTake {
        /// Take percentage (0-18)
        #[arg(long)]
        take: f64,
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Hotkey SS58
        #[arg(long)]
        hotkey: Option<String>,
    },
    /// Set children for hotkey
    SetChildren {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Children as "proportion:hotkey_ss58" pairs, comma-separated
        #[arg(long)]
        children: String,
    },
    /// Recycle alpha tokens back to TAO
    RecycleAlpha {
        /// Amount of alpha to recycle
        #[arg(long)]
        amount: f64,
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Hotkey SS58
        #[arg(long)]
        hotkey: Option<String>,
    },
    /// Unstake all alpha across all subnets for a hotkey
    UnstakeAllAlpha {
        /// Hotkey SS58
        #[arg(long)]
        hotkey: Option<String>,
    },
    /// Burn alpha tokens permanently (reduce supply)
    BurnAlpha {
        /// Amount of alpha to burn
        #[arg(long)]
        amount: f64,
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Hotkey SS58
        #[arg(long)]
        hotkey: Option<String>,
    },
    /// Swap stake between subnets with a limit price
    SwapLimit {
        /// Amount of alpha to swap
        #[arg(long)]
        amount: f64,
        /// Source subnet
        #[arg(long)]
        from: u16,
        /// Destination subnet
        #[arg(long)]
        to: u16,
        /// Limit price
        #[arg(long)]
        price: f64,
        /// Allow partial fill
        #[arg(long)]
        partial: bool,
        /// Hotkey SS58
        #[arg(long)]
        hotkey: Option<String>,
    },
    /// Set auto-stake hotkey for a subnet (rewards auto-compound to this hotkey)
    SetAuto {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Hotkey SS58 to auto-stake to
        #[arg(long)]
        hotkey: Option<String>,
    },
    /// Set root claim type (how root emissions are handled)
    SetClaim {
        /// Claim type: swap (alpha→TAO), keep (keep alpha), keep-subnets (keep for specific subnets)
        #[arg(long, value_parser = ["swap", "keep", "keep-subnets"])]
        claim_type: String,
        /// Subnet UIDs to keep alpha for (only with --claim-type keep-subnets, comma-separated)
        #[arg(long)]
        subnets: Option<String>,
    },
    /// Full staking wizard (interactive or non-interactive with flags)
    Wizard {
        /// Subnet UID (skip interactive subnet selection)
        #[arg(long)]
        netuid: Option<u16>,
        /// Amount of TAO to stake (skip interactive amount input)
        #[arg(long)]
        amount: Option<f64>,
        /// Hotkey SS58 (skip interactive hotkey selection)
        #[arg(long)]
        hotkey: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum SubnetCommands {
    /// List all subnets
    List {
        /// Query at a specific block number (historical wayback)
        #[arg(long)]
        at_block: Option<u32>,
    },
    /// Show detailed info for a subnet
    Show {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Query at a specific block number (historical wayback)
        #[arg(long)]
        at_block: Option<u32>,
    },
    /// Show subnet hyperparameters
    Hyperparams {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
    },
    /// Show metagraph for a subnet (full or single UID)
    Metagraph {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Show only a specific neuron UID
        #[arg(long)]
        uid: Option<u16>,
        /// Query at a specific block number (historical wayback)
        #[arg(long)]
        at_block: Option<u32>,
        /// Fetch full neuron info including axon/prometheus endpoints
        #[arg(long)]
        full: bool,
        /// Save snapshot to disk cache (~/.agcli/metagraph/)
        #[arg(long)]
        save: bool,
    },
    /// Load a cached metagraph snapshot from disk
    CacheLoad {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Block number to load (default: latest)
        #[arg(long)]
        block: Option<u64>,
    },
    /// List cached metagraph snapshots for a subnet
    CacheList {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
    },
    /// Diff two metagraph snapshots (current vs cached, or two cached blocks)
    CacheDiff {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// First block number (older, default: latest cached)
        #[arg(long)]
        from_block: Option<u64>,
        /// Second block number (newer, default: fetch live from chain)
        #[arg(long)]
        to_block: Option<u64>,
    },
    /// Prune old cached metagraph snapshots
    CachePrune {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Number of snapshots to keep (default: 10)
        #[arg(long, default_value = "10")]
        keep: usize,
    },
    /// Probe axon health for neurons on a subnet
    Probe {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Only probe specific UIDs (comma-separated)
        #[arg(long)]
        uids: Option<String>,
        /// Timeout per probe in milliseconds (default: 3000)
        #[arg(long, default_value = "3000")]
        timeout_ms: u64,
        /// Max concurrent probes (default: 32)
        #[arg(long, default_value = "32")]
        concurrency: usize,
    },
    /// Register a new subnet
    Register,
    /// Register a neuron on a subnet (burn)
    RegisterNeuron {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
    },
    /// Register via POW
    Pow {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Number of threads
        #[arg(long, default_value = "4")]
        threads: u32,
    },
    /// Dissolve a subnet (owner only)
    Dissolve {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
    },
    /// Live watch: tempo countdown, rate limits, commit-reveal status
    Watch {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Polling interval in seconds (default 12)
        #[arg(long, default_value = "12")]
        interval: u64,
    },
    /// AMM liquidity dashboard: pool depth, slippage at common trade sizes
    Liquidity {
        /// Subnet UID (omit for all subnets)
        #[arg(long)]
        netuid: Option<u16>,
    },
    /// Monitor a subnet: track registrations, weight changes, emission shifts, anomalies
    Monitor {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Polling interval in seconds (default 24 = ~2 blocks)
        #[arg(long, default_value = "24")]
        interval: u64,
        /// Output in JSON streaming mode (one JSON object per event, for piping)
        #[arg(long)]
        json: bool,
    },
    /// Show subnet health: all miners, status, weights vs consensus
    Health {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
    },
    /// Show who's earning what, projected next epoch
    Emissions {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
    },
    /// Show current registration cost + recent trend
    Cost {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
    },
    /// Show pending weight commits on a subnet (commit-reveal status)
    Commits {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Filter by hotkey SS58 address (default: show all)
        #[arg(long)]
        hotkey: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum WeightCommands {
    /// Set weights on a subnet
    Set {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Weights as "uid:weight" pairs, comma-separated
        #[arg(long)]
        weights: String,
        /// Version key
        #[arg(long, default_value = "0")]
        version_key: u64,
        /// Dry-run: check pre-conditions without submitting
        #[arg(long)]
        dry_run: bool,
    },
    /// Commit weights (for commit-reveal enabled subnets)
    Commit {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Weights as "uid:weight" pairs
        #[arg(long)]
        weights: String,
        /// Salt (random if not specified)
        #[arg(long)]
        salt: Option<String>,
    },
    /// Reveal previously committed weights
    Reveal {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Weights as "uid:weight" pairs
        #[arg(long)]
        weights: String,
        /// Salt used in commit
        #[arg(long)]
        salt: String,
        /// Version key
        #[arg(long, default_value = "0")]
        version_key: u64,
    },
    /// Check commit status for your hotkey on a subnet
    Status {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
    },
    /// Atomic commit-reveal: commit weights, wait for reveal window, then auto-reveal
    CommitReveal {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Weights as "uid:weight" pairs, comma-separated
        #[arg(long)]
        weights: String,
        /// Version key
        #[arg(long, default_value = "0")]
        version_key: u64,
        /// Wait for reveal to be confirmed on-chain before exiting
        #[arg(long)]
        wait: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum RootCommands {
    /// Register on root network
    Register,
    /// Set root weights
    Weights {
        /// Weights as "netuid:weight" pairs
        #[arg(long)]
        weights: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum DelegateCommands {
    /// Show delegate info
    Show {
        /// Hotkey SS58
        #[arg(long)]
        hotkey: Option<String>,
    },
    /// List all delegates
    List,
    /// Decrease take
    DecreaseTake {
        /// New take percentage
        #[arg(long)]
        take: f64,
        /// Hotkey SS58
        #[arg(long)]
        hotkey: Option<String>,
    },
    /// Increase take
    IncreaseTake {
        /// New take percentage
        #[arg(long)]
        take: f64,
        /// Hotkey SS58
        #[arg(long)]
        hotkey: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum ViewCommands {
    /// Show full portfolio (all stakes, balances)
    Portfolio {
        /// Coldkey SS58
        #[arg(long)]
        address: Option<String>,
        /// Query at a specific block number (historical wayback)
        #[arg(long)]
        at_block: Option<u32>,
    },
    /// Show neuron details
    Neuron {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Neuron UID
        #[arg(long)]
        uid: u16,
        /// Query at a specific block number (historical wayback)
        #[arg(long)]
        at_block: Option<u32>,
    },
    /// Show network overview
    Network {
        /// Query network stats at a specific block number (historical wayback)
        #[arg(long)]
        at_block: Option<u32>,
    },
    /// Show Dynamic TAO info for all subnets (prices, pools, volumes)
    Dynamic {
        /// Query at a specific block number (historical wayback)
        #[arg(long)]
        at_block: Option<u32>,
    },
    /// Show top validators by stake across subnets
    Validators {
        /// Subnet UID (omit for all subnets)
        #[arg(long)]
        netuid: Option<u16>,
        /// Max number of validators to show
        #[arg(long, default_value = "50")]
        limit: usize,
        /// Query at a specific block number (historical wayback)
        #[arg(long)]
        at_block: Option<u32>,
    },
    /// Show recent extrinsics for an account (via Subscan)
    History {
        /// SS58 address (defaults to wallet coldkey)
        #[arg(long)]
        address: Option<String>,
        /// Number of transactions to show
        #[arg(long, default_value = "20")]
        limit: usize,
    },
    /// Detailed account explorer (balance, stakes, identity, registrations)
    Account {
        /// SS58 address (defaults to wallet coldkey)
        #[arg(long)]
        address: Option<String>,
        /// Query account info at a specific block number (historical wayback)
        #[arg(long)]
        at_block: Option<u32>,
    },
    /// Subnet analytics (emission rates, top miners/validators, stats)
    SubnetAnalytics {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
    },
    /// Staking analytics (APY estimates, emission projections)
    StakingAnalytics {
        /// Coldkey SS58 (defaults to wallet coldkey)
        #[arg(long)]
        address: Option<String>,
    },
    /// Simulate a TAO→Alpha swap (see how much alpha you'd get)
    SwapSim {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Amount of TAO to swap
        #[arg(long)]
        tao: Option<f64>,
        /// Amount of Alpha to swap (for reverse direction)
        #[arg(long)]
        alpha: Option<f64>,
    },
    /// Show who has nominated/delegated to a hotkey
    Nominations {
        /// Hotkey SS58 address
        #[arg(long)]
        hotkey: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum IdentityCommands {
    /// Set on-chain identity
    Set {
        /// Name
        #[arg(long)]
        name: String,
        /// URL
        #[arg(long)]
        url: Option<String>,
        /// GitHub
        #[arg(long)]
        github: Option<String>,
        /// Description
        #[arg(long)]
        description: Option<String>,
    },
    /// Show identity
    Show {
        /// SS58 address
        #[arg(long)]
        address: String,
    },
    /// Set subnet identity (subnet owner only)
    SetSubnet {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Subnet name
        #[arg(long)]
        name: String,
        /// GitHub
        #[arg(long)]
        github: Option<String>,
        /// URL
        #[arg(long)]
        url: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum ServeCommands {
    /// Set axon endpoint (IP, port, protocol) for a subnet
    Axon {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// IP address (IPv4)
        #[arg(long)]
        ip: String,
        /// Port number
        #[arg(long)]
        port: u16,
        /// Protocol version (default 4)
        #[arg(long, default_value = "4")]
        protocol: u8,
        /// Axon version
        #[arg(long, default_value = "0")]
        version: u32,
    },
}

#[derive(Subcommand, Debug)]
pub enum ProxyCommands {
    /// Add a proxy account
    Add {
        /// Proxy delegate SS58 address
        #[arg(long)]
        delegate: String,
        /// Proxy type (any, owner, staking, non_transfer, non_critical, governance, senate)
        #[arg(long, default_value = "any")]
        proxy_type: String,
        /// Delay in blocks before proxy can execute (0 = immediate)
        #[arg(long, default_value = "0")]
        delay: u32,
    },
    /// Remove a proxy account
    Remove {
        /// Proxy delegate SS58 address
        #[arg(long)]
        delegate: String,
        /// Proxy type (must match what was set)
        #[arg(long, default_value = "any")]
        proxy_type: String,
        /// Delay (must match what was set)
        #[arg(long, default_value = "0")]
        delay: u32,
    },
    /// List proxy accounts for an address
    List {
        /// SS58 address (defaults to wallet coldkey)
        #[arg(long)]
        address: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum SwapCommands {
    /// Swap hotkey
    Hotkey {
        /// New hotkey SS58
        #[arg(long)]
        new_hotkey: String,
    },
    /// Schedule coldkey swap
    Coldkey {
        /// New coldkey SS58
        #[arg(long)]
        new_coldkey: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum MultisigCommands {
    /// Derive a multisig account address from signatories
    Address {
        /// Signatories as comma-separated SS58 addresses
        #[arg(long)]
        signatories: String,
        /// Approval threshold
        #[arg(long)]
        threshold: u16,
    },
    /// Submit a multisig call (as_multi)
    Submit {
        /// Other signatories (comma-separated SS58, excluding yourself)
        #[arg(long)]
        others: String,
        /// Approval threshold
        #[arg(long)]
        threshold: u16,
        /// Pallet name for the inner call
        #[arg(long)]
        pallet: String,
        /// Call name
        #[arg(long)]
        call: String,
        /// Call args as JSON (string values in the format expected by subxt dynamic)
        #[arg(long)]
        args: Option<String>,
    },
    /// Approve a pending multisig call (approve_as_multi)
    Approve {
        /// Other signatories (comma-separated SS58, excluding yourself)
        #[arg(long)]
        others: String,
        /// Approval threshold
        #[arg(long)]
        threshold: u16,
        /// Call hash (0x-prefixed hex)
        #[arg(long)]
        call_hash: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum CrowdloanCommands {
    /// Create a new crowdloan campaign
    Create {
        /// Initial deposit in TAO
        #[arg(long)]
        deposit: f64,
        /// Minimum contribution in TAO
        #[arg(long)]
        min_contribution: f64,
        /// Funding cap in TAO
        #[arg(long)]
        cap: f64,
        /// End block number
        #[arg(long)]
        end_block: u32,
        /// Target SS58 address for funds (optional; if omitted, creator receives)
        #[arg(long)]
        target: Option<String>,
    },
    /// Contribute TAO to a crowdloan
    Contribute {
        /// Crowdloan ID
        #[arg(long)]
        crowdloan_id: u32,
        /// Amount of TAO to contribute
        #[arg(long)]
        amount: f64,
    },
    /// Withdraw contribution from an active crowdloan
    Withdraw {
        /// Crowdloan ID
        #[arg(long)]
        crowdloan_id: u32,
    },
    /// Finalize a crowdloan that has reached its cap
    Finalize {
        /// Crowdloan ID
        #[arg(long)]
        crowdloan_id: u32,
    },
    /// Refund all contributors of a failed/expired crowdloan
    Refund {
        /// Crowdloan ID
        #[arg(long)]
        crowdloan_id: u32,
    },
    /// Dissolve a crowdloan (creator only, after refunding)
    Dissolve {
        /// Crowdloan ID
        #[arg(long)]
        crowdloan_id: u32,
    },
    /// Update crowdloan funding cap (creator only)
    UpdateCap {
        /// Crowdloan ID
        #[arg(long)]
        crowdloan_id: u32,
        /// New cap in TAO
        #[arg(long)]
        cap: f64,
    },
    /// Update crowdloan end block (creator only)
    UpdateEnd {
        /// Crowdloan ID
        #[arg(long)]
        crowdloan_id: u32,
        /// New end block number
        #[arg(long)]
        end_block: u32,
    },
    /// Update minimum contribution (creator only)
    UpdateMinContribution {
        /// Crowdloan ID
        #[arg(long)]
        crowdloan_id: u32,
        /// New minimum contribution in TAO
        #[arg(long)]
        min_contribution: f64,
    },
}

#[derive(Subcommand, Debug)]
pub enum LiquidityCommands {
    /// Add a liquidity position to a subnet's AMM pool
    Add {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Lower price bound (TAO per Alpha)
        #[arg(long)]
        price_low: f64,
        /// Upper price bound (TAO per Alpha)
        #[arg(long)]
        price_high: f64,
        /// Liquidity amount (in RAO units)
        #[arg(long)]
        amount: u64,
        /// Hotkey SS58 (defaults to wallet hotkey)
        #[arg(long)]
        hotkey: Option<String>,
    },
    /// Remove a liquidity position entirely
    Remove {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Position ID
        #[arg(long)]
        position_id: u128,
        /// Hotkey SS58 (defaults to wallet hotkey)
        #[arg(long)]
        hotkey: Option<String>,
    },
    /// Modify liquidity in an existing position
    Modify {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Position ID
        #[arg(long)]
        position_id: u128,
        /// Liquidity delta (positive = add, negative = remove)
        #[arg(long, allow_hyphen_values = true)]
        delta: i64,
        /// Hotkey SS58 (defaults to wallet hotkey)
        #[arg(long)]
        hotkey: Option<String>,
    },
    /// Toggle user liquidity for a subnet (subnet owner only)
    Toggle {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Enable user liquidity
        #[arg(long)]
        enable: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum BlockCommands {
    /// Show info for a specific block number
    Info {
        /// Block number
        #[arg(long)]
        number: u32,
    },
    /// Show the latest finalized block
    Latest,
    /// Summarize a range of blocks (hash, timestamp, extrinsic count)
    Range {
        /// Start block number (inclusive)
        #[arg(long)]
        from: u32,
        /// End block number (inclusive)
        #[arg(long)]
        to: u32,
    },
}

#[derive(Subcommand, Debug)]
pub enum DiffCommands {
    /// Compare portfolio (balance + stakes) between two blocks
    Portfolio {
        /// SS58 address (defaults to wallet coldkey)
        #[arg(long)]
        address: Option<String>,
        /// First block number
        #[arg(long)]
        block1: u32,
        /// Second block number
        #[arg(long)]
        block2: u32,
    },
    /// Compare subnet state between two blocks
    Subnet {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// First block number
        #[arg(long)]
        block1: u32,
        /// Second block number
        #[arg(long)]
        block2: u32,
    },
    /// Compare network-wide stats between two blocks
    Network {
        /// First block number
        #[arg(long)]
        block1: u32,
        /// Second block number
        #[arg(long)]
        block2: u32,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Show current configuration
    Show,
    /// Set a config value (e.g., agcli config set --key network --value finney)
    Set {
        /// Key to set
        #[arg(long)]
        key: String,
        /// Value to set
        #[arg(long)]
        value: String,
    },
    /// Remove a config value
    Unset {
        /// Key to remove
        #[arg(long)]
        key: String,
    },
    /// Show config file path
    Path,
}

impl Cli {
    /// Apply config file defaults to CLI args (CLI flags take precedence).
    pub fn apply_config(&mut self, cfg: &crate::config::Config) {
        // Only apply config if CLI still has the clap default
        if self.network == "finney" {
            if let Some(ref n) = cfg.network {
                self.network = n.clone();
            }
        }
        if self.endpoint.is_none() {
            self.endpoint = cfg.endpoint.clone();
        }
        if self.wallet_dir == "~/.bittensor/wallets" {
            if let Some(ref d) = cfg.wallet_dir {
                self.wallet_dir = d.clone();
            }
        }
        if self.wallet == "default" {
            if let Some(ref w) = cfg.wallet {
                self.wallet = w.clone();
            }
        }
        if self.hotkey == "default" {
            if let Some(ref h) = cfg.hotkey {
                self.hotkey = h.clone();
            }
        }
        if self.output == "table" {
            if let Some(ref o) = cfg.output {
                self.output = o.clone();
            }
        }
        if self.proxy.is_none() {
            self.proxy = cfg.proxy.clone();
        }
        if !self.batch {
            if let Some(true) = cfg.batch {
                self.batch = true;
            }
        }
    }

    /// Get live polling interval (None = not live, Some(secs) = live mode).
    pub fn live_interval(&self) -> Option<u64> {
        self.live.map(|opt| opt.unwrap_or(12))
    }

    /// Resolve the network from CLI args.
    pub fn resolve_network(&self) -> Network {
        if let Some(ref endpoint) = self.endpoint {
            Network::Custom(endpoint.clone())
        } else {
            match self.network.as_str() {
                "finney" | "main" => Network::Finney,
                "test" | "testnet" => Network::Test,
                "local" | "localhost" => Network::Local,
                "archive" => Network::Archive,
                other => Network::Custom(other.to_string()),
            }
        }
    }
}
