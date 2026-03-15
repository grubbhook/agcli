//! Chain data structures decoded from subtensor storage.

use crate::types::balance::{AlphaBalance, Balance};
use crate::types::network::NetUid;
use serde::{Deserialize, Serialize};

/// Neuron (miner/validator) information on a subnet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuronInfo {
    pub hotkey: String,
    pub coldkey: String,
    pub uid: u16,
    pub netuid: NetUid,
    pub active: bool,
    pub stake: Balance,
    pub rank: f64,
    pub emission: f64,
    pub incentive: f64,
    pub consensus: f64,
    pub trust: f64,
    pub validator_trust: f64,
    pub dividends: f64,
    pub last_update: u64,
    pub validator_permit: bool,
    pub pruning_score: f64,
    pub axon_info: Option<AxonInfo>,
    pub prometheus_info: Option<PrometheusInfo>,
}

/// Lightweight neuron info (no axon/prometheus).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuronInfoLite {
    pub hotkey: String,
    pub coldkey: String,
    pub uid: u16,
    pub netuid: NetUid,
    pub active: bool,
    pub stake: Balance,
    pub rank: f64,
    pub emission: f64,
    pub incentive: f64,
    pub consensus: f64,
    pub trust: f64,
    pub validator_trust: f64,
    pub dividends: f64,
    pub last_update: u64,
    pub validator_permit: bool,
    pub pruning_score: f64,
}

/// Axon (miner endpoint) metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxonInfo {
    pub block: u64,
    pub version: u32,
    pub ip: String,
    pub port: u16,
    pub ip_type: u8,
    pub protocol: u8,
}

/// Prometheus endpoint metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrometheusInfo {
    pub block: u64,
    pub version: u32,
    pub ip: String,
    pub port: u16,
    pub ip_type: u8,
}

/// Subnet information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubnetInfo {
    pub netuid: NetUid,
    pub name: String,
    pub symbol: String,
    pub n: u16,
    pub max_n: u16,
    pub tempo: u16,
    pub emission_value: u64,
    pub burn: Balance,
    pub difficulty: u64,
    pub immunity_period: u16,
    pub owner: String,
    pub registration_allowed: bool,
}

/// Dynamic subnet information (Dynamic TAO).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicInfo {
    pub netuid: NetUid,
    pub name: String,
    pub symbol: String,
    pub tempo: u16,
    pub emission: u64,
    pub tao_in: Balance,
    pub alpha_in: AlphaBalance,
    pub alpha_out: AlphaBalance,
    pub price: f64,
    pub owner_hotkey: String,
    pub owner_coldkey: String,
    pub last_step: u64,
    pub blocks_since_last_step: u64,
    pub alpha_out_emission: u64,
    pub alpha_in_emission: u64,
    pub tao_in_emission: u64,
    pub pending_alpha_emission: u64,
    pub pending_root_emission: u64,
    pub subnet_volume: u128,
    pub network_registered_at: u64,
}

impl DynamicInfo {
    /// Compute total emission per tempo from component fields.
    /// The `emission` field on-chain is deprecated/zero; use the sum of
    /// alpha_out_emission + alpha_in_emission + tao_in_emission instead.
    pub fn total_emission(&self) -> u64 {
        self.alpha_out_emission
            .saturating_add(self.alpha_in_emission)
            .saturating_add(self.tao_in_emission)
    }
}

/// Subnet hyperparameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubnetHyperparameters {
    pub netuid: NetUid,
    pub rho: u16,
    pub kappa: u16,
    pub immunity_period: u16,
    pub min_allowed_weights: u16,
    pub max_weights_limit: u16,
    pub tempo: u16,
    pub min_difficulty: u64,
    pub max_difficulty: u64,
    pub weights_version: u64,
    pub weights_rate_limit: u64,
    pub adjustment_interval: u16,
    pub activity_cutoff: u16,
    pub registration_allowed: bool,
    pub target_regs_per_interval: u16,
    pub min_burn: Balance,
    pub max_burn: Balance,
    pub bonds_moving_avg: u64,
    pub max_regs_per_block: u16,
    pub serving_rate_limit: u64,
    pub max_validators: u16,
    pub adjustment_alpha: u64,
    pub difficulty: u64,
    pub commit_reveal_weights_enabled: bool,
    pub commit_reveal_weights_interval: u64,
    pub liquid_alpha_enabled: bool,
}

/// Delegate information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegateInfo {
    pub hotkey: String,
    pub owner: String,
    pub take: f64,
    pub total_stake: Balance,
    pub nominators: Vec<(String, Balance)>,
    pub registrations: Vec<NetUid>,
    pub validator_permits: Vec<NetUid>,
    pub return_per_1000: Balance,
}

/// Stake information for a coldkey-hotkey-subnet triple.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeInfo {
    pub hotkey: String,
    pub coldkey: String,
    pub netuid: NetUid,
    pub stake: Balance,
    pub alpha_stake: AlphaBalance,
}

/// On-chain identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainIdentity {
    pub name: String,
    pub url: String,
    pub github: String,
    pub image: String,
    pub discord: String,
    pub description: String,
    pub additional: String,
}

/// Subnet identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubnetIdentity {
    pub subnet_name: String,
    pub github_repo: String,
    pub subnet_contact: String,
    pub subnet_url: String,
    pub discord: String,
    pub description: String,
    pub additional: String,
}

/// Metagraph: full snapshot of a subnet's state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metagraph {
    pub netuid: NetUid,
    pub n: u16,
    pub block: u64,
    pub neurons: Vec<NeuronInfoLite>,
    pub stake: Vec<Balance>,
    pub ranks: Vec<f64>,
    pub trust: Vec<f64>,
    pub consensus: Vec<f64>,
    pub incentive: Vec<f64>,
    pub dividends: Vec<f64>,
    pub emission: Vec<f64>,
    pub validator_trust: Vec<f64>,
    pub validator_permit: Vec<bool>,
    pub uids: Vec<u16>,
    pub active: Vec<bool>,
    pub last_update: Vec<u64>,
}

