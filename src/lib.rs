//! # agcli — Rust SDK + CLI for the Bittensor Network
//!
//! `agcli` provides a complete toolkit for interacting with the Bittensor
//! blockchain (subtensor). It covers wallet management, staking, transfers,
//! subnet operations, weight setting, registration, and chain queries.
//!
//! ## Quick Start (SDK)
//!
//! ```rust,no_run
//! use agcli::{Client, Wallet};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let client = Client::connect("wss://entrypoint-finney.opentensor.ai:443").await?;
//!     let wallet = Wallet::open("~/.bittensor/wallets/default")?;
//!     let balance = client.get_balance(&wallet.coldkey_public()).await?;
//!     println!("Balance: {} TAO", balance.tao());
//!     Ok(())
//! }
//! ```

pub mod chain;
pub mod config;
pub mod events;
pub mod extrinsics;
pub mod live;
pub mod queries;
pub mod types;
pub mod utils;
pub mod wallet;

#[cfg(feature = "cli")]
pub mod cli;

/// Generated chain API from subtensor runtime metadata (build.rs).
#[allow(dead_code, unused_imports, non_camel_case_types, clippy::all)]
mod generated {
    include!(concat!(env!("OUT_DIR"), "/metadata.rs"));
}
pub use generated::api;

pub use subxt::config::SubstrateConfig as SubtensorConfig;

pub type AccountId = <SubtensorConfig as subxt::Config>::AccountId;
pub type Hash = <SubtensorConfig as subxt::Config>::Hash;

// Re-exports for ergonomic SDK use
pub use chain::Client;
pub use config::Config;
pub use types::balance::Balance;
pub use wallet::Wallet;
