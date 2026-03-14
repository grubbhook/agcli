//! Substrate chain client — connect, query storage, submit extrinsics.

pub mod connection;
pub mod rpc_types;
pub mod storage;

use anyhow::{Context, Result};
use sp_core::{sr25519, Pair as _};
use subxt::tx::PairSigner;
use subxt::OnlineClient;

use crate::types::balance::Balance;
use crate::types::chain_data::*;
use crate::types::network::NetUid;
use crate::{AccountId, SubtensorConfig, api};

/// Signer type for extrinsic submission.
pub type Signer = PairSigner<SubtensorConfig, sr25519::Pair>;

/// High-level client for the Bittensor (subtensor) chain.
pub struct Client {
    inner: OnlineClient<SubtensorConfig>,
}

impl Client {
    /// Connect to a subtensor node.
    pub async fn connect(url: &str) -> Result<Self> {
        tracing::info!("Connecting to {}", url);
        let inner = OnlineClient::from_url(url)
            .await
            .context("Failed to connect to subtensor node")?;
        Ok(Self { inner })
    }

    /// Connect to a well-known network.
    pub async fn connect_network(network: &crate::types::Network) -> Result<Self> {
        Self::connect(network.ws_url()).await
    }

    /// Get a reference to the underlying subxt client.
    pub fn subxt(&self) -> &OnlineClient<SubtensorConfig> {
        &self.inner
    }

    /// Create a signer from an sr25519 keypair.
    pub fn signer(pair: &sr25519::Pair) -> Signer {
        PairSigner::new(pair.clone())
    }

    fn to_account_id(pk: &sr25519::Public) -> AccountId {
        AccountId::from(pk.0)
    }

    fn ss58_to_account_id(ss58: &str) -> Result<AccountId> {
        let pk = crate::wallet::keypair::from_ss58(ss58)?;
        Ok(AccountId::from(pk.0))
    }

    // ──────── Balance Queries ────────

    /// Get TAO balance (free) for an account.
    pub async fn get_balance(&self, account: &sr25519::Public) -> Result<Balance> {
        let account_id = Self::to_account_id(account);
        let addr = api::storage().system().account(&account_id);
        let info = self.inner.storage().at_latest().await?.fetch(&addr).await?;
        match info {
            Some(info) => Ok(Balance::from_rao(info.data.free as u64)),
            None => Ok(Balance::ZERO),
        }
    }

    /// Get TAO balance for an SS58 address.
    pub async fn get_balance_ss58(&self, ss58: &str) -> Result<Balance> {
        let pk = crate::wallet::keypair::from_ss58(ss58)?;
        self.get_balance(&pk).await
    }

    // ──────── Block Info ────────

    /// Current block number.
    pub async fn get_block_number(&self) -> Result<u64> {
        let block = self.inner.blocks().at_latest().await?;
        Ok(block.number() as u64)
    }

    // ──────── Network Params ────────

    /// Total TAO issuance.
    pub async fn get_total_issuance(&self) -> Result<Balance> {
        let addr = api::storage().balances().total_issuance();
        let val = self.inner.storage().at_latest().await?.fetch(&addr).await?;
        Ok(Balance::from_rao(val.unwrap_or(0) as u64))
    }

    /// Total staked TAO.
    pub async fn get_total_stake(&self) -> Result<Balance> {
        let addr = api::storage().subtensor_module().total_stake();
        let val = self.inner.storage().at_latest().await?.fetch(&addr).await?;
        Ok(Balance::from_rao(val.unwrap_or(0)))
    }

    /// Total number of subnets.
    pub async fn get_total_networks(&self) -> Result<u16> {
        let addr = api::storage().subtensor_module().total_networks();
        let val = self.inner.storage().at_latest().await?.fetch(&addr).await?;
        Ok(val.unwrap_or(0))
    }

    /// Block emission rate.
    pub async fn get_block_emission(&self) -> Result<Balance> {
        let addr = api::storage().subtensor_module().block_emission();
        let val = self.inner.storage().at_latest().await?.fetch(&addr).await?;
        Ok(Balance::from_rao(val.unwrap_or(0)))
    }

    // ──────── Stake Queries ────────

    /// Get all stakes for a coldkey (via runtime API).
    pub async fn get_stake_for_coldkey(&self, coldkey_ss58: &str) -> Result<Vec<StakeInfo>> {
        let account_id = Self::ss58_to_account_id(coldkey_ss58)?;
        let payload = api::apis()
            .stake_info_runtime_api()
            .get_stake_info_for_coldkey(account_id);
        let result = self.inner.runtime_api().at_latest().await?.call(payload).await?;
        Ok(result.into_iter().map(StakeInfo::from).collect())
    }

    // ──────── Subnet Queries ────────

    /// List all subnets (via runtime API).
    pub async fn get_all_subnets(&self) -> Result<Vec<SubnetInfo>> {
        let payload = api::apis().subnet_info_runtime_api().get_subnets_info();
        let result = self.inner.runtime_api().at_latest().await?.call(payload).await?;
        Ok(result.into_iter().flatten().map(SubnetInfo::from).collect())
    }

    /// Get info for a specific subnet.
    pub async fn get_subnet_info(&self, netuid: NetUid) -> Result<Option<SubnetInfo>> {
        let payload = api::apis().subnet_info_runtime_api().get_subnet_info(netuid.0);
        let result = self.inner.runtime_api().at_latest().await?.call(payload).await?;
        Ok(result.map(SubnetInfo::from))
    }

    /// Get subnet hyperparameters.
    pub async fn get_subnet_hyperparams(&self, netuid: NetUid) -> Result<Option<SubnetHyperparameters>> {
        let payload = api::apis().subnet_info_runtime_api().get_subnet_hyperparams(netuid.0);
        let result = self.inner.runtime_api().at_latest().await?.call(payload).await?;
        Ok(result.map(|h| SubnetHyperparameters::from_gen(h, netuid)))
    }

    /// Get dynamic info for all subnets.
    pub async fn get_all_dynamic_info(&self) -> Result<Vec<DynamicInfo>> {
        let subnets = self.get_all_subnets().await?;
        Ok(subnets
            .into_iter()
            .map(|si| DynamicInfo {
                netuid: si.netuid,
                symbol: si.symbol.clone(),
                tempo: si.tempo,
                n: si.n,
                max_n: si.max_n,
                emission_value: si.emission_value,
                tao_in: Balance::ZERO,
                alpha_in: crate::types::balance::AlphaBalance::from_raw(0),
                alpha_out: crate::types::balance::AlphaBalance::from_raw(0),
                price: 0.0,
                owner: si.owner,
            })
            .collect())
    }

    // ──────── Neuron / Metagraph Queries ────────

    /// Get lightweight neuron info for a subnet (via runtime API).
    pub async fn get_neurons_lite(&self, netuid: NetUid) -> Result<Vec<NeuronInfoLite>> {
        let payload = api::apis().neuron_info_runtime_api().get_neurons_lite(netuid.0);
        let result = self.inner.runtime_api().at_latest().await?.call(payload).await?;
        Ok(result.into_iter().map(NeuronInfoLite::from).collect())
    }

    /// Get full neuron info for a specific UID.
    pub async fn get_neuron(&self, netuid: NetUid, uid: u16) -> Result<Option<NeuronInfo>> {
        let payload = api::apis().neuron_info_runtime_api().get_neuron(netuid.0, uid);
        let result = self.inner.runtime_api().at_latest().await?.call(payload).await?;
        Ok(result.map(NeuronInfo::from))
    }

    /// Get the metagraph for a subnet.
    pub async fn get_metagraph(&self, netuid: NetUid) -> Result<Metagraph> {
        crate::queries::fetch_metagraph(self, netuid).await
    }

    // ──────── Delegate Queries ────────

    /// Get all delegates (via runtime API).
    pub async fn get_delegates(&self) -> Result<Vec<DelegateInfo>> {
        let payload = api::apis().delegate_info_runtime_api().get_delegates();
        let result = self.inner.runtime_api().at_latest().await?.call(payload).await?;
        Ok(result.into_iter().map(DelegateInfo::from).collect())
    }

    /// Get delegate info for a specific hotkey.
    pub async fn get_delegate(&self, hotkey_ss58: &str) -> Result<Option<DelegateInfo>> {
        let account_id = Self::ss58_to_account_id(hotkey_ss58)?;
        let payload = api::apis().delegate_info_runtime_api().get_delegate(account_id);
        let result = self.inner.runtime_api().at_latest().await?.call(payload).await?;
        Ok(result.map(DelegateInfo::from))
    }

    // ──────── Identity Queries ────────

    /// Get on-chain identity for an account.
    pub async fn get_identity(&self, _ss58: &str) -> Result<Option<ChainIdentity>> {
        // TODO: map to correct storage key once identity pallet schema is confirmed
        Ok(None)
    }

    /// Get subnet identity.
    pub async fn get_subnet_identity(&self, _netuid: NetUid) -> Result<Option<SubnetIdentity>> {
        // TODO: map to correct storage key
        Ok(None)
    }

    // ──────── Extrinsic Submission ────────

    /// Transfer TAO from coldkey to destination.
    pub async fn transfer(
        &self,
        signer_pair: &sr25519::Pair,
        dest_ss58: &str,
        amount: Balance,
    ) -> Result<String> {
        let dest_id = Self::ss58_to_account_id(dest_ss58)?;
        let dest = subxt::utils::MultiAddress::Id(dest_id);
        let tx = api::tx().balances().transfer_allow_death(dest, amount.rao());
        let signer = Self::signer(signer_pair);
        let result = self.inner.tx()
            .sign_and_submit_then_watch_default(&tx, &signer).await?
            .wait_for_finalized_success().await?;
        Ok(format!("{:?}", result.extrinsic_hash()))
    }

    /// Add stake to a hotkey on a subnet.
    pub async fn add_stake(
        &self,
        signer_pair: &sr25519::Pair,
        hotkey_ss58: &str,
        netuid: NetUid,
        amount: Balance,
    ) -> Result<String> {
        let hotkey_id = Self::ss58_to_account_id(hotkey_ss58)?;
        let tx = api::tx().subtensor_module().add_stake(hotkey_id, netuid.0, amount.rao());
        let signer = Self::signer(signer_pair);
        let result = self.inner.tx()
            .sign_and_submit_then_watch_default(&tx, &signer).await?
            .wait_for_finalized_success().await?;
        Ok(format!("{:?}", result.extrinsic_hash()))
    }

    /// Remove stake from a hotkey on a subnet.
    pub async fn remove_stake(
        &self,
        signer_pair: &sr25519::Pair,
        hotkey_ss58: &str,
        netuid: NetUid,
        amount: Balance,
    ) -> Result<String> {
        let hotkey_id = Self::ss58_to_account_id(hotkey_ss58)?;
        let tx = api::tx().subtensor_module().remove_stake(hotkey_id, netuid.0, amount.rao());
        let signer = Self::signer(signer_pair);
        let result = self.inner.tx()
            .sign_and_submit_then_watch_default(&tx, &signer).await?
            .wait_for_finalized_success().await?;
        Ok(format!("{:?}", result.extrinsic_hash()))
    }

    /// Register on a subnet via burned TAO.
    pub async fn burned_register(
        &self,
        signer_pair: &sr25519::Pair,
        netuid: NetUid,
        hotkey_ss58: &str,
    ) -> Result<String> {
        let hotkey_id = Self::ss58_to_account_id(hotkey_ss58)?;
        let tx = api::tx().subtensor_module().burned_register(netuid.0, hotkey_id);
        let signer = Self::signer(signer_pair);
        let result = self.inner.tx()
            .sign_and_submit_then_watch_default(&tx, &signer).await?
            .wait_for_finalized_success().await?;
        Ok(format!("{:?}", result.extrinsic_hash()))
    }

    /// Set weights on a subnet.
    pub async fn set_weights(
        &self,
        signer_pair: &sr25519::Pair,
        netuid: NetUid,
        uids: &[u16],
        weights: &[u16],
        version_key: u64,
    ) -> Result<String> {
        let tx = api::tx().subtensor_module()
            .set_weights(netuid.0, uids.to_vec(), weights.to_vec(), version_key);
        let signer = Self::signer(signer_pair);
        let result = self.inner.tx()
            .sign_and_submit_then_watch_default(&tx, &signer).await?
            .wait_for_finalized_success().await?;
        Ok(format!("{:?}", result.extrinsic_hash()))
    }

    /// Register a new subnet.
    pub async fn register_network(
        &self,
        signer_pair: &sr25519::Pair,
        hotkey_ss58: &str,
    ) -> Result<String> {
        let hotkey_id = Self::ss58_to_account_id(hotkey_ss58)?;
        let tx = api::tx().subtensor_module().register_network(hotkey_id);
        let signer = Self::signer(signer_pair);
        let result = self.inner.tx()
            .sign_and_submit_then_watch_default(&tx, &signer).await?
            .wait_for_finalized_success().await?;
        Ok(format!("{:?}", result.extrinsic_hash()))
    }

    /// Move stake between subnets (same coldkey, different hotkeys/subnets).
    pub async fn move_stake(
        &self,
        signer_pair: &sr25519::Pair,
        hotkey_ss58: &str,
        from_netuid: NetUid,
        to_netuid: NetUid,
        amount: Balance,
    ) -> Result<String> {
        let hotkey_id = Self::ss58_to_account_id(hotkey_ss58)?;
        // move_stake takes: origin_hotkey, destination_hotkey, origin_netuid, destination_netuid, alpha_amount
        let tx = api::tx().subtensor_module()
            .move_stake(hotkey_id.clone(), hotkey_id, from_netuid.0, to_netuid.0, amount.rao());
        let signer = Self::signer(signer_pair);
        let result = self.inner.tx()
            .sign_and_submit_then_watch_default(&tx, &signer).await?
            .wait_for_finalized_success().await?;
        Ok(format!("{:?}", result.extrinsic_hash()))
    }

    /// Swap stake between subnets (same hotkey).
    pub async fn swap_stake(
        &self,
        signer_pair: &sr25519::Pair,
        hotkey_ss58: &str,
        from_netuid: NetUid,
        to_netuid: NetUid,
        amount: Balance,
    ) -> Result<String> {
        let hotkey_id = Self::ss58_to_account_id(hotkey_ss58)?;
        // swap_stake takes: hotkey, origin_netuid, destination_netuid, alpha_amount
        let tx = api::tx().subtensor_module()
            .swap_stake(hotkey_id, from_netuid.0, to_netuid.0, amount.rao());
        let signer = Self::signer(signer_pair);
        let result = self.inner.tx()
            .sign_and_submit_then_watch_default(&tx, &signer).await?
            .wait_for_finalized_success().await?;
        Ok(format!("{:?}", result.extrinsic_hash()))
    }

    /// Set childkey take.
    pub async fn set_childkey_take(
        &self,
        signer_pair: &sr25519::Pair,
        hotkey_ss58: &str,
        netuid: NetUid,
        take: u16,
    ) -> Result<String> {
        let hotkey_id = Self::ss58_to_account_id(hotkey_ss58)?;
        let tx = api::tx().subtensor_module().set_childkey_take(hotkey_id, netuid.0, take);
        let signer = Self::signer(signer_pair);
        let result = self.inner.tx()
            .sign_and_submit_then_watch_default(&tx, &signer).await?
            .wait_for_finalized_success().await?;
        Ok(format!("{:?}", result.extrinsic_hash()))
    }

    /// Set children for a hotkey.
    pub async fn set_children(
        &self,
        signer_pair: &sr25519::Pair,
        hotkey_ss58: &str,
        netuid: NetUid,
        children: &[(u64, String)],
    ) -> Result<String> {
        let hotkey_id = Self::ss58_to_account_id(hotkey_ss58)?;
        let children_accounts: Vec<(u64, AccountId)> = children
            .iter()
            .map(|(proportion, ss58)| {
                let id = Self::ss58_to_account_id(ss58).unwrap();
                (*proportion, id)
            })
            .collect();
        let tx = api::tx().subtensor_module()
            .set_children(hotkey_id, netuid.0, children_accounts);
        let signer = Self::signer(signer_pair);
        let result = self.inner.tx()
            .sign_and_submit_then_watch_default(&tx, &signer).await?
            .wait_for_finalized_success().await?;
        Ok(format!("{:?}", result.extrinsic_hash()))
    }

    /// Commit weights (commit-reveal scheme).
    pub async fn commit_weights(
        &self,
        signer_pair: &sr25519::Pair,
        netuid: NetUid,
        commit_hash: [u8; 32],
    ) -> Result<String> {
        let hash = subxt::utils::H256::from(commit_hash);
        let tx = api::tx().subtensor_module().commit_weights(netuid.0, hash);
        let signer = Self::signer(signer_pair);
        let result = self.inner.tx()
            .sign_and_submit_then_watch_default(&tx, &signer).await?
            .wait_for_finalized_success().await?;
        Ok(format!("{:?}", result.extrinsic_hash()))
    }

    /// Reveal weights.
    pub async fn reveal_weights(
        &self,
        signer_pair: &sr25519::Pair,
        netuid: NetUid,
        uids: &[u16],
        values: &[u16],
        salt: &[u16],
        version_key: u64,
    ) -> Result<String> {
        let tx = api::tx().subtensor_module()
            .reveal_weights(netuid.0, uids.to_vec(), values.to_vec(), salt.to_vec(), version_key);
        let signer = Self::signer(signer_pair);
        let result = self.inner.tx()
            .sign_and_submit_then_watch_default(&tx, &signer).await?
            .wait_for_finalized_success().await?;
        Ok(format!("{:?}", result.extrinsic_hash()))
    }

    /// Serve axon metadata.
    pub async fn serve_axon(
        &self,
        signer_pair: &sr25519::Pair,
        netuid: NetUid,
        axon: &AxonInfo,
    ) -> Result<String> {
        let ip: u128 = axon.ip.parse().unwrap_or(0);
        let tx = api::tx().subtensor_module()
            .serve_axon(netuid.0, axon.version, ip, axon.port, axon.ip_type, axon.protocol, 0, 0);
        let signer = Self::signer(signer_pair);
        let result = self.inner.tx()
            .sign_and_submit_then_watch_default(&tx, &signer).await?
            .wait_for_finalized_success().await?;
        Ok(format!("{:?}", result.extrinsic_hash()))
    }

    /// Decrease delegate take.
    pub async fn decrease_take(
        &self,
        signer_pair: &sr25519::Pair,
        hotkey_ss58: &str,
        take: u16,
    ) -> Result<String> {
        let hotkey_id = Self::ss58_to_account_id(hotkey_ss58)?;
        let tx = api::tx().subtensor_module().decrease_take(hotkey_id, take);
        let signer = Self::signer(signer_pair);
        let result = self.inner.tx()
            .sign_and_submit_then_watch_default(&tx, &signer).await?
            .wait_for_finalized_success().await?;
        Ok(format!("{:?}", result.extrinsic_hash()))
    }

    /// Increase delegate take.
    pub async fn increase_take(
        &self,
        signer_pair: &sr25519::Pair,
        hotkey_ss58: &str,
        take: u16,
    ) -> Result<String> {
        let hotkey_id = Self::ss58_to_account_id(hotkey_ss58)?;
        let tx = api::tx().subtensor_module().increase_take(hotkey_id, take);
        let signer = Self::signer(signer_pair);
        let result = self.inner.tx()
            .sign_and_submit_then_watch_default(&tx, &signer).await?
            .wait_for_finalized_success().await?;
        Ok(format!("{:?}", result.extrinsic_hash()))
    }

    /// Unstake all from a hotkey.
    pub async fn unstake_all(
        &self,
        signer_pair: &sr25519::Pair,
        hotkey_ss58: &str,
    ) -> Result<String> {
        let hotkey_id = Self::ss58_to_account_id(hotkey_ss58)?;
        let tx = api::tx().subtensor_module().unstake_all(hotkey_id);
        let signer = Self::signer(signer_pair);
        let result = self.inner.tx()
            .sign_and_submit_then_watch_default(&tx, &signer).await?
            .wait_for_finalized_success().await?;
        Ok(format!("{:?}", result.extrinsic_hash()))
    }

    /// Schedule coldkey swap.
    pub async fn schedule_swap_coldkey(
        &self,
        signer_pair: &sr25519::Pair,
        new_coldkey_ss58: &str,
    ) -> Result<String> {
        let new_id = Self::ss58_to_account_id(new_coldkey_ss58)?;
        let tx = api::tx().subtensor_module().schedule_swap_coldkey(new_id);
        let signer = Self::signer(signer_pair);
        let result = self.inner.tx()
            .sign_and_submit_then_watch_default(&tx, &signer).await?
            .wait_for_finalized_success().await?;
        Ok(format!("{:?}", result.extrinsic_hash()))
    }

    /// Swap hotkey.
    pub async fn swap_hotkey(
        &self,
        signer_pair: &sr25519::Pair,
        old_hotkey_ss58: &str,
        new_hotkey_ss58: &str,
    ) -> Result<String> {
        let old_id = Self::ss58_to_account_id(old_hotkey_ss58)?;
        let new_id = Self::ss58_to_account_id(new_hotkey_ss58)?;
        let tx = api::tx().subtensor_module().swap_hotkey(old_id, new_id, None);
        let signer = Self::signer(signer_pair);
        let result = self.inner.tx()
            .sign_and_submit_then_watch_default(&tx, &signer).await?
            .wait_for_finalized_success().await?;
        Ok(format!("{:?}", result.extrinsic_hash()))
    }

    /// Root register.
    pub async fn root_register(
        &self,
        signer_pair: &sr25519::Pair,
        hotkey_ss58: &str,
    ) -> Result<String> {
        let hotkey_id = Self::ss58_to_account_id(hotkey_ss58)?;
        let tx = api::tx().subtensor_module().root_register(hotkey_id);
        let signer = Self::signer(signer_pair);
        let result = self.inner.tx()
            .sign_and_submit_then_watch_default(&tx, &signer).await?
            .wait_for_finalized_success().await?;
        Ok(format!("{:?}", result.extrinsic_hash()))
    }

    /// Dissolve a subnet.
    pub async fn dissolve_network(
        &self,
        signer_pair: &sr25519::Pair,
        netuid: NetUid,
    ) -> Result<String> {
        let coldkey_id = AccountId::from(signer_pair.public().0);
        let tx = api::tx().subtensor_module().dissolve_network(coldkey_id, netuid.0);
        let signer = Self::signer(signer_pair);
        let result = self.inner.tx()
            .sign_and_submit_then_watch_default(&tx, &signer).await?
            .wait_for_finalized_success().await?;
        Ok(format!("{:?}", result.extrinsic_hash()))
    }

    /// Transfer stake to another coldkey.
    pub async fn transfer_stake(
        &self,
        signer_pair: &sr25519::Pair,
        dest_coldkey_ss58: &str,
        hotkey_ss58: &str,
        from_netuid: NetUid,
        to_netuid: NetUid,
        amount: Balance,
    ) -> Result<String> {
        let dest_id = Self::ss58_to_account_id(dest_coldkey_ss58)?;
        let hotkey_id = Self::ss58_to_account_id(hotkey_ss58)?;
        let tx = api::tx().subtensor_module()
            .transfer_stake(dest_id, hotkey_id, from_netuid.0, to_netuid.0, amount.rao());
        let signer = Self::signer(signer_pair);
        let result = self.inner.tx()
            .sign_and_submit_then_watch_default(&tx, &signer).await?
            .wait_for_finalized_success().await?;
        Ok(format!("{:?}", result.extrinsic_hash()))
    }

    /// Recycle alpha for TAO.
    pub async fn recycle_alpha(
        &self,
        signer_pair: &sr25519::Pair,
        hotkey_ss58: &str,
        netuid: NetUid,
        amount: u64,
    ) -> Result<String> {
        let hotkey_id = Self::ss58_to_account_id(hotkey_ss58)?;
        let tx = api::tx().subtensor_module().recycle_alpha(hotkey_id, amount, netuid.0);
        let signer = Self::signer(signer_pair);
        let result = self.inner.tx()
            .sign_and_submit_then_watch_default(&tx, &signer).await?
            .wait_for_finalized_success().await?;
        Ok(format!("{:?}", result.extrinsic_hash()))
    }

    /// Claim root dividends.
    pub async fn claim_root(
        &self,
        signer_pair: &sr25519::Pair,
        subnets: &[u16],
    ) -> Result<String> {
        let tx = api::tx().subtensor_module().claim_root(subnets.to_vec());
        let signer = Self::signer(signer_pair);
        let result = self.inner.tx()
            .sign_and_submit_then_watch_default(&tx, &signer).await?
            .wait_for_finalized_success().await?;
        Ok(format!("{:?}", result.extrinsic_hash()))
    }

    /// Add stake with limit order.
    pub async fn add_stake_limit(
        &self,
        signer_pair: &sr25519::Pair,
        hotkey_ss58: &str,
        netuid: NetUid,
        amount: Balance,
        limit_price: u64,
        allow_partial: bool,
    ) -> Result<String> {
        let hotkey_id = Self::ss58_to_account_id(hotkey_ss58)?;
        let tx = api::tx().subtensor_module()
            .add_stake_limit(hotkey_id, netuid.0, amount.rao(), limit_price, allow_partial);
        let signer = Self::signer(signer_pair);
        let result = self.inner.tx()
            .sign_and_submit_then_watch_default(&tx, &signer).await?
            .wait_for_finalized_success().await?;
        Ok(format!("{:?}", result.extrinsic_hash()))
    }

    /// Remove stake with limit order.
    pub async fn remove_stake_limit(
        &self,
        signer_pair: &sr25519::Pair,
        hotkey_ss58: &str,
        netuid: NetUid,
        amount: u64,
        limit_price: u64,
        allow_partial: bool,
    ) -> Result<String> {
        let hotkey_id = Self::ss58_to_account_id(hotkey_ss58)?;
        let tx = api::tx().subtensor_module()
            .remove_stake_limit(hotkey_id, netuid.0, amount, limit_price, allow_partial);
        let signer = Self::signer(signer_pair);
        let result = self.inner.tx()
            .sign_and_submit_then_watch_default(&tx, &signer).await?
            .wait_for_finalized_success().await?;
        Ok(format!("{:?}", result.extrinsic_hash()))
    }

    /// Set on-chain identity.
    pub async fn set_identity(
        &self,
        _signer: &sr25519::Pair,
        _identity: &ChainIdentity,
    ) -> Result<String> {
        anyhow::bail!("set_identity requires chain-specific identity type mapping — TODO")
    }

    /// Get total hotkey alpha on a subnet.
    pub async fn get_total_hotkey_alpha(
        &self,
        _hotkey_ss58: &str,
        _netuid: NetUid,
    ) -> Result<Balance> {
        // Alpha storage uses FixedU128 which requires special handling
        Ok(Balance::ZERO)
    }
}
