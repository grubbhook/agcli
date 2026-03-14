//! CLI command definitions and handlers.

pub mod commands;
pub mod helpers;
pub mod wallet_cmds;
pub mod stake_cmds;
pub mod view_cmds;

use clap::{Parser, Subcommand};
use crate::types::network::Network;

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
    /// Show account balance
    Balance {
        /// SS58 address (defaults to wallet coldkey)
        #[arg(long)]
        address: Option<String>,
    },

    // ──── Transfer ────
    /// Transfer TAO to another account
    Transfer {
        /// Destination SS58 address
        dest: String,
        /// Amount of TAO to send
        amount: f64,
    },

    /// Transfer entire balance to another account (minus fees)
    TransferAll {
        /// Destination SS58 address
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
    /// Crowdloan operations (create, contribute, withdraw, finalize)
    #[command(subcommand)]
    Crowdloan(CrowdloanCommands),

    // ──── Config ────
    /// Manage persistent configuration (~/.agcli/config.toml)
    #[command(subcommand)]
    Config(ConfigCommands),

    // ──── Completions ────
    /// Generate shell completions (bash, zsh, fish, powershell)
    Completions {
        /// Shell to generate completions for
        #[arg(value_parser = ["bash", "zsh", "fish", "powershell"])]
        shell: String,
    },

    // ──── Update ────
    /// Self-update agcli to the latest version from GitHub
    Update,
}

#[derive(Subcommand, Debug)]
pub enum SubscribeCommands {
    /// Watch finalized blocks
    Blocks,
    /// Watch chain events (all, staking, registration, transfer, weights, subnet)
    Events {
        /// Event filter category
        #[arg(default_value = "all")]
        filter: String,
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
        name: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum StakeCommands {
    /// Add stake to a hotkey on a subnet
    Add {
        /// Amount of TAO to stake
        amount: f64,
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Hotkey SS58 (defaults to wallet hotkey)
        #[arg(long)]
        hotkey: Option<String>,
    },
    /// Remove stake from a hotkey on a subnet
    Remove {
        /// Amount to unstake
        amount: f64,
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Hotkey SS58
        #[arg(long)]
        hotkey: Option<String>,
    },
    /// Show all stakes for current wallet
    List {
        /// Coldkey SS58 address
        #[arg(long)]
        address: Option<String>,
    },
    /// Move stake between subnets
    Move {
        /// Amount of alpha to move
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
    List,
    /// Show detailed info for a subnet
    Show {
        /// Subnet UID
        netuid: u16,
    },
    /// Show subnet hyperparameters
    Hyperparams {
        /// Subnet UID
        netuid: u16,
    },
    /// Show metagraph for a subnet
    Metagraph {
        /// Subnet UID
        netuid: u16,
    },
    /// Register a new subnet
    Register,
    /// Register a neuron on a subnet (burn)
    RegisterNeuron {
        /// Subnet UID
        netuid: u16,
    },
    /// Register via POW
    Pow {
        /// Subnet UID
        netuid: u16,
        /// Number of threads
        #[arg(long, default_value = "4")]
        threads: u32,
    },
    /// Dissolve a subnet (owner only)
    Dissolve {
        /// Subnet UID
        netuid: u16,
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
        weights: String,
        /// Version key
        #[arg(long, default_value = "0")]
        version_key: u64,
    },
    /// Commit weights (for commit-reveal enabled subnets)
    Commit {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Weights as "uid:weight" pairs
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
        weights: String,
        /// Salt used in commit
        salt: String,
        /// Version key
        #[arg(long, default_value = "0")]
        version_key: u64,
    },
}

#[derive(Subcommand, Debug)]
pub enum RootCommands {
    /// Register on root network
    Register,
    /// Set root weights
    Weights {
        /// Weights as "netuid:weight" pairs
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
        take: f64,
        /// Hotkey SS58
        #[arg(long)]
        hotkey: Option<String>,
    },
    /// Increase take
    IncreaseTake {
        /// New take percentage
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
    },
    /// Show neuron details
    Neuron {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Neuron UID
        uid: u16,
    },
    /// Show network overview
    Network,
    /// Show Dynamic TAO info for all subnets (prices, pools, volumes)
    Dynamic,
    /// Show top validators by stake across subnets
    Validators {
        /// Subnet UID (omit for all subnets)
        #[arg(long)]
        netuid: Option<u16>,
        /// Max number of validators to show
        #[arg(long, default_value = "50")]
        limit: usize,
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
    },
    /// Subnet analytics (emission rates, top miners/validators, stats)
    SubnetAnalytics {
        /// Subnet UID
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
        address: String,
    },
    /// Set subnet identity (subnet owner only)
    SetSubnet {
        /// Subnet UID
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
        new_hotkey: String,
    },
    /// Schedule coldkey swap
    Coldkey {
        /// New coldkey SS58
        new_coldkey: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum MultisigCommands {
    /// Derive a multisig account address from signatories
    Address {
        /// Signatories as comma-separated SS58 addresses
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
    /// Contribute TAO to a crowdloan
    Contribute {
        /// Crowdloan ID
        crowdloan_id: u32,
        /// Amount of TAO to contribute
        amount: f64,
    },
    /// Withdraw contribution from an active crowdloan
    Withdraw {
        /// Crowdloan ID
        crowdloan_id: u32,
    },
    /// Finalize a crowdloan that has reached its cap
    Finalize {
        /// Crowdloan ID
        crowdloan_id: u32,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Show current configuration
    Show,
    /// Set a config value (e.g., agcli config set network finney)
    Set {
        /// Key to set
        key: String,
        /// Value to set
        value: String,
    },
    /// Remove a config value
    Unset {
        /// Key to remove
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
                other => Network::Custom(other.to_string()),
            }
        }
    }
}
