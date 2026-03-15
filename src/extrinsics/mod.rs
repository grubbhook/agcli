//! Extrinsic construction helpers.
//!
//! All extrinsic submission goes through `Client::sign_submit()` in `chain/mod.rs`.
//! This module contains utility functions used during extrinsic construction.
//!
//! ## Supported Extrinsics (all on `Client`)
//!
//! **Staking**: add_stake, remove_stake, unstake_all, move_stake, swap_stake,
//! transfer_stake, add_stake_limit, remove_stake_limit, recycle_alpha, claim_root,
//! set_childkey_take, set_children
//!
//! **Transfers**: transfer (transfer_allow_death)
//!
//! **Registration**: burned_register, pow_register, root_register, register_network
//!
//! **Weights**: set_weights, commit_weights, reveal_weights, batch_set_weights,
//! batch_commit_weights, batch_reveal_weights
//!
//! **Subnets**: serve_axon, dissolve_network, set_subnet_identity
//!
//! **Delegation**: decrease_take, increase_take
//!
//! **Keys**: swap_hotkey, schedule_swap_coldkey
//!
//! **Dynamic**: submit_raw_call (for EVM, MEV Shield, Contracts, etc.)

pub mod mev_shield;
pub mod weights;

pub use mev_shield::encrypt_for_mev_shield;
pub use weights::compute_weight_commit_hash;
