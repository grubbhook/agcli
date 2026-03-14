//! Conversion from subxt-generated runtime types to our application-level types.

use subxt::utils::AccountId32;

use crate::types::balance::{AlphaBalance, Balance};
use crate::types::chain_data::*;
use crate::types::network::NetUid;
use crate::wallet::keypair::to_ss58;

/// Shorthand type alias for the generated runtime types.
pub(crate) type GenNeuronInfoLite =
    crate::api::runtime_types::pallet_subtensor::rpc_info::neuron_info::NeuronInfoLite<AccountId32>;
pub(crate) type GenNeuronInfo =
    crate::api::runtime_types::pallet_subtensor::rpc_info::neuron_info::NeuronInfo<AccountId32>;
pub(crate) type GenSubnetInfo =
    crate::api::runtime_types::pallet_subtensor::rpc_info::subnet_info::SubnetInfo<AccountId32>;
pub(crate) type GenSubnetHyperparams =
    crate::api::runtime_types::pallet_subtensor::rpc_info::subnet_info::SubnetHyperparams;
pub(crate) type GenStakeInfo =
    crate::api::runtime_types::pallet_subtensor::rpc_info::stake_info::StakeInfo<AccountId32>;
pub(crate) type GenDelegateInfo =
    crate::api::runtime_types::pallet_subtensor::rpc_info::delegate_info::DelegateInfo<AccountId32>;

fn account_to_ss58(a: &AccountId32) -> String {
    to_ss58(&sp_core::sr25519::Public::from_raw(a.0), 42)
}

fn ip_to_string(ip: u128, ip_type: u8) -> String {
    if ip_type == 4 {
        let ip4 = ip as u32;
        format!(
            "{}.{}.{}.{}",
            (ip4 >> 24) & 0xFF,
            (ip4 >> 16) & 0xFF,
            (ip4 >> 8) & 0xFF,
            ip4 & 0xFF
        )
    } else {
        format!("{:x}", ip)
    }
}

// ──────── NeuronInfoLite ────────

impl From<GenNeuronInfoLite> for NeuronInfoLite {
    fn from(n: GenNeuronInfoLite) -> Self {
        let total_stake: u64 = n.stake.iter().map(|(_, s)| s.0).sum();
        NeuronInfoLite {
            hotkey: account_to_ss58(&n.hotkey),
            coldkey: account_to_ss58(&n.coldkey),
            uid: n.uid,
            netuid: NetUid(n.netuid),
            active: n.active,
            stake: Balance::from_rao(total_stake),
            rank: n.rank as f64 / 65535.0,
            emission: n.emission as f64,
            incentive: n.incentive as f64 / 65535.0,
            consensus: n.consensus as f64 / 65535.0,
            trust: n.trust as f64 / 65535.0,
            validator_trust: n.validator_trust as f64 / 65535.0,
            dividends: n.dividends as f64 / 65535.0,
            last_update: n.last_update,
            validator_permit: n.validator_permit,
            pruning_score: n.pruning_score as f64 / 65535.0,
        }
    }
}

// ──────── NeuronInfo ────────

impl From<GenNeuronInfo> for NeuronInfo {
    fn from(n: GenNeuronInfo) -> Self {
        let total_stake: u64 = n.stake.iter().map(|(_, s)| s.0).sum();
        NeuronInfo {
            hotkey: account_to_ss58(&n.hotkey),
            coldkey: account_to_ss58(&n.coldkey),
            uid: n.uid,
            netuid: NetUid(n.netuid),
            active: n.active,
            stake: Balance::from_rao(total_stake),
            rank: n.rank as f64 / 65535.0,
            emission: n.emission as f64,
            incentive: n.incentive as f64 / 65535.0,
            consensus: n.consensus as f64 / 65535.0,
            trust: n.trust as f64 / 65535.0,
            validator_trust: n.validator_trust as f64 / 65535.0,
            dividends: n.dividends as f64 / 65535.0,
            last_update: n.last_update,
            validator_permit: n.validator_permit,
            pruning_score: n.pruning_score as f64 / 65535.0,
            axon_info: Some(AxonInfo {
                block: n.axon_info.block,
                version: n.axon_info.version,
                ip: ip_to_string(n.axon_info.ip, n.axon_info.ip_type),
                port: n.axon_info.port,
                ip_type: n.axon_info.ip_type,
                protocol: n.axon_info.protocol,
            }),
            prometheus_info: Some(PrometheusInfo {
                block: n.prometheus_info.block,
                version: n.prometheus_info.version,
                ip: ip_to_string(n.prometheus_info.ip, n.prometheus_info.ip_type),
                port: n.prometheus_info.port,
                ip_type: n.prometheus_info.ip_type,
            }),
        }
    }
}

// ──────── SubnetInfo ────────

impl From<GenSubnetInfo> for SubnetInfo {
    fn from(s: GenSubnetInfo) -> Self {
        SubnetInfo {
            netuid: NetUid(s.netuid),
            name: format!("SN{}", s.netuid),
            symbol: format!("α{}", s.netuid),
            n: s.subnetwork_n,
            max_n: s.max_allowed_uids,
            tempo: s.tempo,
            emission_value: s.emission_values,
            burn: Balance::from_rao(s.burn),
            difficulty: s.difficulty,
            immunity_period: s.immunity_period,
            owner: account_to_ss58(&s.owner),
            registration_allowed: true,
        }
    }
}

// ──────── SubnetHyperparameters ────────

impl SubnetHyperparameters {
    pub fn from_gen(h: GenSubnetHyperparams, netuid: NetUid) -> Self {
        SubnetHyperparameters {
            netuid,
            rho: h.rho,
            kappa: h.kappa,
            immunity_period: h.immunity_period,
            min_allowed_weights: h.min_allowed_weights,
            max_weights_limit: h.max_weights_limit,
            tempo: h.tempo,
            min_difficulty: h.min_difficulty,
            max_difficulty: h.max_difficulty,
            weights_version: h.weights_version,
            weights_rate_limit: h.weights_rate_limit,
            adjustment_interval: h.adjustment_interval,
            activity_cutoff: h.activity_cutoff,
            registration_allowed: h.registration_allowed,
            target_regs_per_interval: h.target_regs_per_interval,
            min_burn: Balance::from_rao(h.min_burn),
            max_burn: Balance::from_rao(h.max_burn),
            bonds_moving_avg: h.bonds_moving_avg,
            max_regs_per_block: h.max_regs_per_block,
            serving_rate_limit: h.serving_rate_limit,
            max_validators: h.max_validators,
            adjustment_alpha: h.adjustment_alpha,
            difficulty: h.difficulty,
            commit_reveal_weights_enabled: h.commit_reveal_weights_enabled,
            commit_reveal_weights_interval: h.commit_reveal_period,
            liquid_alpha_enabled: h.liquid_alpha_enabled,
        }
    }
}

// ──────── DelegateInfo ────────

impl From<GenDelegateInfo> for DelegateInfo {
    fn from(d: GenDelegateInfo) -> Self {
        let total_stake: u64 = d
            .nominators
            .iter()
            .flat_map(|(_, stakes)| stakes.iter().map(|(_, s)| s.0))
            .sum();
        DelegateInfo {
            hotkey: account_to_ss58(&d.delegate_ss58),
            owner: account_to_ss58(&d.owner_ss58),
            take: d.take as f64 / 65535.0,
            total_stake: Balance::from_rao(total_stake),
            nominators: d
                .nominators
                .into_iter()
                .map(|(a, stakes)| {
                    let total: u64 = stakes.iter().map(|(_, s)| s.0).sum();
                    (account_to_ss58(&a), Balance::from_rao(total))
                })
                .collect(),
            registrations: d.registrations.into_iter().map(|r| NetUid(r.0)).collect(),
            validator_permits: d.validator_permits.into_iter().map(|p| NetUid(p.0)).collect(),
            return_per_1000: Balance::from_rao(d.return_per_1000),
        }
    }
}

// ──────── StakeInfo ────────

impl From<GenStakeInfo> for StakeInfo {
    fn from(s: GenStakeInfo) -> Self {
        let netuid = NetUid(s.netuid);
        StakeInfo {
            hotkey: account_to_ss58(&s.hotkey),
            coldkey: account_to_ss58(&s.coldkey),
            netuid,
            stake: Balance::from_rao(s.stake),
            alpha_stake: AlphaBalance::from_raw(s.stake),
        }
    }
}
