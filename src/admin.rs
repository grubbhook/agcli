//! AdminUtils sudo calls — set subnet hyperparameters via the chain's sudo mechanism.
//!
//! These functions wrap `submit_raw_call` for the `AdminUtils` pallet, making
//! it easy to configure subnets programmatically.
//!
//! ```rust,no_run
//! use agcli::admin;
//! use agcli::Client;
//! use sp_core::Pair as _;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let client = Client::connect("ws://127.0.0.1:9944").await?;
//! let alice = sp_core::sr25519::Pair::from_string("//Alice", None)?;
//! admin::set_tempo(&client, &alice, 1, 100).await?;
//! # Ok(())
//! # }
//! ```

use crate::chain::Client;
use anyhow::Result;
use sp_core::sr25519;
use subxt::dynamic::Value;

/// Set the tempo (blocks per epoch) for a subnet.
pub async fn set_tempo(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    tempo: u16,
) -> Result<String> {
    client
        .submit_sudo_raw_call(
            sudo_key,
            "AdminUtils",
            "sudo_set_tempo",
            vec![Value::u128(netuid as u128), Value::u128(tempo as u128)],
        )
        .await
}

/// Set max allowed validators for a subnet.
pub async fn set_max_allowed_validators(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    max: u16,
) -> Result<String> {
    client
        .submit_sudo_raw_call(
            sudo_key,
            "AdminUtils",
            "sudo_set_max_allowed_validators",
            vec![Value::u128(netuid as u128), Value::u128(max as u128)],
        )
        .await
}

/// Set max allowed UIDs for a subnet.
pub async fn set_max_allowed_uids(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    max: u16,
) -> Result<String> {
    client
        .submit_sudo_raw_call(
            sudo_key,
            "AdminUtils",
            "sudo_set_max_allowed_uids",
            vec![Value::u128(netuid as u128), Value::u128(max as u128)],
        )
        .await
}

/// Set immunity period for a subnet.
pub async fn set_immunity_period(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    period: u16,
) -> Result<String> {
    client
        .submit_sudo_raw_call(
            sudo_key,
            "AdminUtils",
            "sudo_set_immunity_period",
            vec![Value::u128(netuid as u128), Value::u128(period as u128)],
        )
        .await
}

/// Set min allowed weights for a subnet.
pub async fn set_min_allowed_weights(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    min: u16,
) -> Result<String> {
    client
        .submit_sudo_raw_call(
            sudo_key,
            "AdminUtils",
            "sudo_set_min_allowed_weights",
            vec![Value::u128(netuid as u128), Value::u128(min as u128)],
        )
        .await
}

/// Set max weights limit for a subnet.
pub async fn set_max_weight_limit(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    limit: u16,
) -> Result<String> {
    client
        .submit_sudo_raw_call(
            sudo_key,
            "AdminUtils",
            "sudo_set_max_weight_limit",
            vec![Value::u128(netuid as u128), Value::u128(limit as u128)],
        )
        .await
}

/// Set weights rate limit (0 = no rate limit).
pub async fn set_weights_set_rate_limit(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    limit: u64,
) -> Result<String> {
    client
        .submit_sudo_raw_call(
            sudo_key,
            "AdminUtils",
            "sudo_set_weights_set_rate_limit",
            vec![Value::u128(netuid as u128), Value::u128(limit as u128)],
        )
        .await
}

/// Set commit-reveal weights enabled/disabled.
pub async fn set_commit_reveal_weights_enabled(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    enabled: bool,
) -> Result<String> {
    client
        .submit_sudo_raw_call(
            sudo_key,
            "AdminUtils",
            "sudo_set_commit_reveal_weights_enabled",
            vec![Value::u128(netuid as u128), Value::bool(enabled)],
        )
        .await
}

/// Set difficulty for a subnet.
pub async fn set_difficulty(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    difficulty: u64,
) -> Result<String> {
    client
        .submit_sudo_raw_call(
            sudo_key,
            "AdminUtils",
            "sudo_set_difficulty",
            vec![Value::u128(netuid as u128), Value::u128(difficulty as u128)],
        )
        .await
}

/// Set bonds moving average for a subnet.
pub async fn set_bonds_moving_average(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    avg: u64,
) -> Result<String> {
    client
        .submit_sudo_raw_call(
            sudo_key,
            "AdminUtils",
            "sudo_set_bonds_moving_average",
            vec![Value::u128(netuid as u128), Value::u128(avg as u128)],
        )
        .await
}

/// Set target registrations per interval for a subnet.
pub async fn set_target_registrations_per_interval(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    target: u16,
) -> Result<String> {
    client
        .submit_sudo_raw_call(
            sudo_key,
            "AdminUtils",
            "sudo_set_target_registrations_per_interval",
            vec![Value::u128(netuid as u128), Value::u128(target as u128)],
        )
        .await
}

/// Set activity cutoff for a subnet.
pub async fn set_activity_cutoff(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    cutoff: u16,
) -> Result<String> {
    client
        .submit_sudo_raw_call(
            sudo_key,
            "AdminUtils",
            "sudo_set_activity_cutoff",
            vec![Value::u128(netuid as u128), Value::u128(cutoff as u128)],
        )
        .await
}

/// Set serving rate limit for a subnet.
pub async fn set_serving_rate_limit(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    limit: u64,
) -> Result<String> {
    client
        .submit_sudo_raw_call(
            sudo_key,
            "AdminUtils",
            "sudo_set_serving_rate_limit",
            vec![Value::u128(netuid as u128), Value::u128(limit as u128)],
        )
        .await
}

/// Generic AdminUtils call for parameters not covered by specific helpers.
///
/// `call_name` is the AdminUtils extrinsic name (e.g. "sudo_set_tempo").
/// `args` are the SCALE-encoded arguments as dynamic values.
pub async fn raw_admin_call(
    client: &Client,
    sudo_key: &sr25519::Pair,
    call_name: &str,
    args: Vec<Value>,
) -> Result<String> {
    client
        .submit_sudo_raw_call(sudo_key, "AdminUtils", call_name, args)
        .await
}

/// All known AdminUtils parameters and their expected argument counts.
/// Returns (call_name, description, arg_types).
pub fn known_params() -> Vec<(&'static str, &'static str, &'static [&'static str])> {
    vec![
        (
            "sudo_set_tempo",
            "Blocks per epoch",
            &["netuid: u16", "tempo: u16"],
        ),
        (
            "sudo_set_max_allowed_validators",
            "Max validator slots",
            &["netuid: u16", "max: u16"],
        ),
        (
            "sudo_set_max_allowed_uids",
            "Max total UID slots",
            &["netuid: u16", "max: u16"],
        ),
        (
            "sudo_set_immunity_period",
            "Blocks of immunity after registration",
            &["netuid: u16", "period: u16"],
        ),
        (
            "sudo_set_min_allowed_weights",
            "Minimum weights a validator must set",
            &["netuid: u16", "min: u16"],
        ),
        (
            "sudo_set_max_weight_limit",
            "Maximum weight value",
            &["netuid: u16", "limit: u16"],
        ),
        (
            "sudo_set_weights_set_rate_limit",
            "Blocks between weight submissions (0=unlimited)",
            &["netuid: u16", "limit: u64"],
        ),
        (
            "sudo_set_commit_reveal_weights_enabled",
            "Enable/disable commit-reveal weights",
            &["netuid: u16", "enabled: bool"],
        ),
        (
            "sudo_set_difficulty",
            "POW registration difficulty",
            &["netuid: u16", "difficulty: u64"],
        ),
        (
            "sudo_set_bonds_moving_average",
            "Bonds moving average",
            &["netuid: u16", "avg: u64"],
        ),
        (
            "sudo_set_target_registrations_per_interval",
            "Target registrations per interval",
            &["netuid: u16", "target: u16"],
        ),
        (
            "sudo_set_activity_cutoff",
            "Blocks before a neuron is considered inactive",
            &["netuid: u16", "cutoff: u16"],
        ),
        (
            "sudo_set_serving_rate_limit",
            "Axon serving rate limit",
            &["netuid: u16", "limit: u64"],
        ),
    ]
}
