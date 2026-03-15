//! Event and block subscription — real-time chain event streaming.
//!
//! Uses subxt's block subscription to watch for new blocks and decode
//! relevant SubtensorModule events (stakes, transfers, registrations, etc.).

use anyhow::Result;
use subxt::ext::scale_value::{Composite, Primitive, ValueDef};
use subxt::OnlineClient;

use crate::utils::truncate;
use crate::SubtensorConfig;

/// Categories of events to filter for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventFilter {
    /// All events
    All,
    /// Staking events (add/remove/move/swap stake)
    Staking,
    /// Registration events (neuron/subnet registration)
    Registration,
    /// Transfer events
    Transfer,
    /// Weight events (set/commit/reveal)
    Weights,
    /// Subnet events (hyperparams, identity)
    Subnet,
}

impl std::str::FromStr for EventFilter {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "staking" | "stake" => Self::Staking,
            "registration" | "register" | "reg" => Self::Registration,
            "transfer" | "transfers" => Self::Transfer,
            "weights" | "weight" => Self::Weights,
            "subnet" | "subnets" => Self::Subnet,
            _ => Self::All,
        })
    }
}

/// Known staking-related event variant names.
const STAKING_VARIANTS: &[&str] = &[
    "StakeAdded",
    "StakeRemoved",
    "StakeMoved",
    "StakeSwapped",
    "AllStakeRemoved",
];

/// Known registration-related event variant names.
const REGISTRATION_VARIANTS: &[&str] = &[
    "NeuronRegistered",
    "BurnedRegister",
    "SubnetRegistered",
    "PowRegistered",
];

/// Known weight-related event variant names.
const WEIGHT_VARIANTS: &[&str] = &[
    "WeightsSet",
    "WeightsCommitted",
    "WeightsRevealed",
    "WeightsBatchRevealed",
];

/// Known subnet management event variant names.
const SUBNET_VARIANTS: &[&str] = &[
    "SubnetHyperparamsSet",
    "SubnetIdentitySet",
    "SubnetIdentityRemoved",
    "NetworkAdded",
    "NetworkRemoved",
    "TempoSet",
];

impl EventFilter {
    fn matches(&self, pallet: &str, variant: &str) -> bool {
        match self {
            Self::All => true,
            Self::Staking => pallet == "SubtensorModule" && STAKING_VARIANTS.contains(&variant),
            Self::Registration => {
                pallet == "SubtensorModule" && REGISTRATION_VARIANTS.contains(&variant)
            }
            Self::Transfer => pallet == "Balances",
            Self::Weights => pallet == "SubtensorModule" && WEIGHT_VARIANTS.contains(&variant),
            Self::Subnet => pallet == "SubtensorModule" && SUBNET_VARIANTS.contains(&variant),
        }
    }
}

/// A decoded chain event for display.
#[derive(Debug)]
pub struct ChainEvent {
    pub block_number: u64,
    pub block_hash: String,
    pub pallet: String,
    pub variant: String,
    pub fields: String,
}

impl std::fmt::Display for ChainEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "#{} {}::{} {}",
            self.block_number, self.pallet, self.variant, self.fields
        )
    }
}

/// Extract a u16 netuid value from a named Composite field.
fn extract_netuid<T: Clone>(composite: &Composite<T>) -> Option<u16> {
    if let Composite::Named(fields) = composite {
        for (name, val) in fields {
            if name == "netuid" {
                if let ValueDef::Primitive(Primitive::U128(n)) = &val.value {
                    return Some(*n as u16);
                }
            }
        }
    }
    None
}

/// Convert a Composite to structured JSON for output.
fn composite_to_json<T: Clone>(composite: &Composite<T>) -> serde_json::Value {
    match composite {
        Composite::Named(fields) => {
            let map: serde_json::Map<String, serde_json::Value> = fields
                .iter()
                .map(|(k, v)| (k.clone(), value_to_json(v)))
                .collect();
            serde_json::Value::Object(map)
        }
        Composite::Unnamed(fields) => {
            // Check if this looks like raw bytes (AccountId32, etc.)
            if fields.len() == 32 || fields.len() == 64 {
                let bytes: Vec<u8> = fields
                    .iter()
                    .filter_map(|v| match &v.value {
                        ValueDef::Primitive(Primitive::U128(n)) => Some(*n as u8),
                        _ => None,
                    })
                    .collect();
                if bytes.len() == fields.len() {
                    return serde_json::Value::String(format!("0x{}", hex::encode(&bytes)));
                }
            }
            let arr: Vec<serde_json::Value> = fields.iter().map(value_to_json).collect();
            serde_json::Value::Array(arr)
        }
    }
}

/// Convert a SCALE Value to a serde_json::Value for structured JSON output.
fn value_to_json<T: Clone>(val: &subxt::ext::scale_value::Value<T>) -> serde_json::Value {
    match &val.value {
        ValueDef::Primitive(p) => match p {
            Primitive::Bool(b) => serde_json::Value::Bool(*b),
            Primitive::Char(c) => serde_json::Value::String(c.to_string()),
            Primitive::U128(n) => {
                if *n <= u64::MAX as u128 {
                    serde_json::json!(*n as u64)
                } else {
                    serde_json::Value::String(n.to_string())
                }
            }
            Primitive::I128(n) => {
                if *n >= i64::MIN as i128 && *n <= i64::MAX as i128 {
                    serde_json::json!(*n as i64)
                } else {
                    serde_json::Value::String(n.to_string())
                }
            }
            Primitive::U256(n) => serde_json::Value::String(format!("{:?}", n)),
            Primitive::I256(n) => serde_json::Value::String(format!("{:?}", n)),
            Primitive::String(s) => serde_json::Value::String(s.clone()),
        },
        ValueDef::Composite(composite) => composite_to_json(composite),
        ValueDef::Variant(variant) => {
            let inner = composite_to_json(&variant.values);
            serde_json::json!({ &variant.name: inner })
        }
        ValueDef::BitSequence(bits) => serde_json::Value::String(format!("bits({})", bits.len())),
    }
}

/// Subscribe to new blocks and stream events matching the filter.
pub async fn subscribe_events(
    client: &OnlineClient<SubtensorConfig>,
    filter: EventFilter,
    json_output: bool,
) -> Result<()> {
    subscribe_events_filtered(client, filter, json_output, None, None).await
}

/// Subscribe to events with optional netuid and account filters.
pub async fn subscribe_events_filtered(
    client: &OnlineClient<SubtensorConfig>,
    filter: EventFilter,
    json_output: bool,
    netuid_filter: Option<u16>,
    account_filter: Option<&str>,
) -> Result<()> {
    let mut block_sub = client.blocks().subscribe_finalized().await?;

    if !json_output {
        let mut desc = format!("filter: {:?}", filter);
        if let Some(n) = netuid_filter {
            desc.push_str(&format!(", netuid={}", n));
        }
        if let Some(a) = account_filter {
            desc.push_str(&format!(", account={}", crate::utils::short_ss58(a)));
        }
        println!(
            "Subscribed to finalized blocks ({}). Ctrl+C to stop.\n",
            desc
        );
    }

    while let Some(block_result) = block_sub.next().await {
        let block = block_result?;
        let block_number = block.number() as u64;
        let block_hash = format!("{:?}", block.hash());

        let events = block.events().await?;
        for event in events.iter() {
            let event = event?;
            let pallet = event.pallet_name().to_string();
            let variant = event.variant_name().to_string();

            if !filter.matches(&pallet, &variant) {
                continue;
            }

            let field_values = event.field_values()?;

            // Structured netuid filtering — try structured extraction first, then debug fallback
            if let Some(target_netuid) = netuid_filter {
                if let Some(found) = extract_netuid(&field_values) {
                    if found != target_netuid {
                        continue;
                    }
                } else {
                    // Fallback: check debug string for netuid references
                    let debug_str = format!("{:?}", field_values);
                    if !debug_str.contains(&format!("netuid: {}", target_netuid))
                        && !debug_str.contains(&format!("Unnamed({})", target_netuid))
                    {
                        continue;
                    }
                }
            }

            // Account filtering via debug string (SS58 addresses are reliably present in debug output)
            if let Some(target_account) = account_filter {
                let debug_str = format!("{:?}", field_values);
                if !debug_str.contains(target_account) {
                    continue;
                }
            }

            if json_output {
                let structured_fields = composite_to_json(&field_values);
                println!(
                    "{}",
                    serde_json::json!({
                        "block": block_number,
                        "hash": block_hash,
                        "pallet": pallet,
                        "event": variant,
                        "fields": structured_fields,
                    })
                );
            } else {
                let fields_str = format!("{:?}", field_values);
                let ce = ChainEvent {
                    block_number,
                    block_hash: block_hash.clone(),
                    pallet: pallet.clone(),
                    variant: variant.clone(),
                    fields: truncate(&fields_str, 200),
                };
                println!("{}", ce);
            }
        }
    }
    Ok(())
}

/// Subscribe to new blocks only (no event decoding).
pub async fn subscribe_blocks(
    client: &OnlineClient<SubtensorConfig>,
    json_output: bool,
) -> Result<()> {
    let mut block_sub = client.blocks().subscribe_finalized().await?;

    println!("Subscribed to finalized blocks. Ctrl+C to stop.\n");

    while let Some(block_result) = block_sub.next().await {
        let block = block_result?;
        let number = block.number() as u64;
        let hash = format!("{:?}", block.hash());
        let extrinsic_count = block.extrinsics().await?.len();

        if json_output {
            println!(
                "{}",
                serde_json::json!({
                    "block": number,
                    "hash": hash,
                    "extrinsics": extrinsic_count,
                })
            );
        } else {
            println!(
                "Block #{} hash={} extrinsics={}",
                number, hash, extrinsic_count
            );
        }
    }
    Ok(())
}

