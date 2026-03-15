//! Chain query methods — subnet, neuron, delegate, identity, and historical queries.

use anyhow::Result;

use crate::types::balance::Balance;
use crate::types::chain_data::*;
use crate::types::network::NetUid;
use crate::api;

use super::Client;

impl Client {
    // ──────── Stake Queries ────────

    /// Get all stakes for a coldkey (via runtime API).
    pub async fn get_stake_for_coldkey(&self, coldkey_ss58: &str) -> Result<Vec<StakeInfo>> {
        let start = std::time::Instant::now();
        let account_id = Self::ss58_to_account_id(coldkey_ss58)?;
        let payload = api::apis()
            .stake_info_runtime_api()
            .get_stake_info_for_coldkey(account_id);
        let result = self
            .inner
            .runtime_api()
            .at_latest()
            .await?
            .call(payload)
            .await?;
        let stakes: Vec<StakeInfo> = result.into_iter().map(StakeInfo::from).collect();
        tracing::debug!(elapsed_ms = start.elapsed().as_millis() as u64, count = stakes.len(), "get_stake_for_coldkey");
        Ok(stakes)
    }

    // ──────── Subnet Queries ────────

    /// List all subnets (via runtime API, cached for 30s).
    /// Returns `Arc<Vec<SubnetInfo>>` to avoid cloning the entire collection.
    pub async fn get_all_subnets(&self) -> Result<std::sync::Arc<Vec<SubnetInfo>>> {
        let inner = &self.inner;
        self.cache
            .get_all_subnets(|| async {
                let payload = api::apis().subnet_info_runtime_api().get_subnets_info();
                let result = inner
                    .runtime_api()
                    .at_latest()
                    .await?
                    .call(payload)
                    .await?;
                Ok(result.into_iter().flatten().map(SubnetInfo::from).collect())
            })
            .await
    }

    /// Get info for a specific subnet.
    pub async fn get_subnet_info(&self, netuid: NetUid) -> Result<Option<SubnetInfo>> {
        let payload = api::apis()
            .subnet_info_runtime_api()
            .get_subnet_info(netuid.0);
        let result = self
            .inner
            .runtime_api()
            .at_latest()
            .await?
            .call(payload)
            .await?;
        Ok(result.map(SubnetInfo::from))
    }

    /// Get subnet hyperparameters.
    pub async fn get_subnet_hyperparams(
        &self,
        netuid: NetUid,
    ) -> Result<Option<SubnetHyperparameters>> {
        let payload = api::apis()
            .subnet_info_runtime_api()
            .get_subnet_hyperparams(netuid.0);
        let result = self
            .inner
            .runtime_api()
            .at_latest()
            .await?
            .call(payload)
            .await?;
        Ok(result.map(|h| SubnetHyperparameters::from_gen(h, netuid)))
    }

    /// Get dynamic info for all subnets (cached for 30s).
    /// Returns `Arc<Vec<DynamicInfo>>` to avoid cloning the entire collection.
    pub async fn get_all_dynamic_info(&self) -> Result<std::sync::Arc<Vec<DynamicInfo>>> {
        let inner = &self.inner;
        self.cache
            .get_all_dynamic_info(|| async {
                let payload = api::apis().subnet_info_runtime_api().get_all_dynamic_info();
                let result = inner
                    .runtime_api()
                    .at_latest()
                    .await?
                    .call(payload)
                    .await?;
                Ok(result
                    .into_iter()
                    .flatten()
                    .map(DynamicInfo::from)
                    .collect())
            })
            .await
    }

    /// Get dynamic info for a specific subnet (cached for 30s).
    pub async fn get_dynamic_info(&self, netuid: NetUid) -> Result<Option<DynamicInfo>> {
        let inner = &self.inner;
        let result = self
            .cache
            .get_dynamic_info(netuid.0, || async {
                let payload = api::apis()
                    .subnet_info_runtime_api()
                    .get_dynamic_info(netuid.0);
                let result = inner
                    .runtime_api()
                    .at_latest()
                    .await?
                    .call(payload)
                    .await?;
                Ok(result.map(DynamicInfo::from))
            })
            .await?;
        Ok(result.map(|arc| (*arc).clone()))
    }

    // ──────── Neuron / Metagraph Queries ────────

    /// Get lightweight neuron info for a subnet (via runtime API).
    pub async fn get_neurons_lite(&self, netuid: NetUid) -> Result<Vec<NeuronInfoLite>> {
        let payload = api::apis()
            .neuron_info_runtime_api()
            .get_neurons_lite(netuid.0);
        let result = self
            .inner
            .runtime_api()
            .at_latest()
            .await?
            .call(payload)
            .await?;
        Ok(result.into_iter().map(NeuronInfoLite::from).collect())
    }

    /// Get full neuron info for a specific UID.
    pub async fn get_neuron(&self, netuid: NetUid, uid: u16) -> Result<Option<NeuronInfo>> {
        let payload = api::apis()
            .neuron_info_runtime_api()
            .get_neuron(netuid.0, uid);
        let result = self
            .inner
            .runtime_api()
            .at_latest()
            .await?
            .call(payload)
            .await?;
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
        let result = self
            .inner
            .runtime_api()
            .at_latest()
            .await?
            .call(payload)
            .await?;
        Ok(result.into_iter().map(DelegateInfo::from).collect())
    }

    /// Get delegate info for a specific hotkey.
    pub async fn get_delegate(&self, hotkey_ss58: &str) -> Result<Option<DelegateInfo>> {
        let account_id = Self::ss58_to_account_id(hotkey_ss58)?;
        let payload = api::apis()
            .delegate_info_runtime_api()
            .get_delegate(account_id);
        let result = self
            .inner
            .runtime_api()
            .at_latest()
            .await?
            .call(payload)
            .await?;
        Ok(result.map(DelegateInfo::from))
    }

    // ──────── Identity Queries ────────

    /// Get on-chain identity for an account (from Registry pallet).
    pub async fn get_identity(&self, ss58: &str) -> Result<Option<ChainIdentity>> {
        let account_id = Self::ss58_to_account_id(ss58)?;
        let addr = api::storage().registry().identity_of(&account_id);
        let result = self.inner.storage().at_latest().await?.fetch(&addr).await?;
        Ok(result.map(|reg| {
            let info = reg.info;
            ChainIdentity {
                name: decode_identity_data(&info.display),
                url: decode_identity_data(&info.web),
                github: String::new(), // Registry pallet doesn't have github field
                image: decode_identity_data(&info.image),
                discord: decode_identity_data(&info.riot),
                description: String::new(),
                additional: info
                    .additional
                    .0
                    .iter()
                    .map(|(k, v)| {
                        format!("{}={}", decode_identity_data(k), decode_identity_data(v))
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
            }
        }))
    }

    /// Get subnet identity (from SubtensorModule SubnetIdentitiesV3).
    pub async fn get_subnet_identity(&self, netuid: NetUid) -> Result<Option<SubnetIdentity>> {
        let addr = api::storage()
            .subtensor_module()
            .subnet_identities_v3(netuid.0);
        let result = self.inner.storage().at_latest().await?.fetch(&addr).await?;
        Ok(result.map(|id| SubnetIdentity {
            subnet_name: String::from_utf8_lossy(&id.subnet_name).into_owned(),
            github_repo: String::from_utf8_lossy(&id.github_repo).into_owned(),
            subnet_contact: String::from_utf8_lossy(&id.subnet_contact).into_owned(),
            subnet_url: String::from_utf8_lossy(&id.subnet_url).into_owned(),
            discord: String::from_utf8_lossy(&id.discord).into_owned(),
            description: String::from_utf8_lossy(&id.description).into_owned(),
            additional: String::from_utf8_lossy(&id.additional).into_owned(),
        }))
    }

    // ──────── Delegation / Nomination Queries ────────

    /// Get who has delegated/nominated to a specific hotkey (delegatee).
    /// Returns list of delegate info via DelegateInfoRuntimeApi.
    pub async fn get_delegated(&self, hotkey_ss58: &str) -> Result<Vec<DelegateInfo>> {
        let account_id = Self::ss58_to_account_id(hotkey_ss58)?;
        let payload = api::apis()
            .delegate_info_runtime_api()
            .get_delegated(account_id);
        let result = self
            .inner
            .runtime_api()
            .at_latest()
            .await?
            .call(payload)
            .await?;
        Ok(result
            .into_iter()
            .map(|(di, _extra)| DelegateInfo::from(di))
            .collect())
    }

    // ──────── Proxy List ────────

    /// List proxy accounts for a given address (reads Proxy.Proxies storage).
    pub async fn list_proxies(&self, ss58: &str) -> Result<Vec<(String, String, u32)>> {
        let account_id = Self::ss58_to_account_id(ss58)?;
        let addr = api::storage().proxy().proxies(&account_id);
        let result = self.inner.storage().at_latest().await?.fetch(&addr).await?;
        match result {
            Some((proxies, _deposit)) => Ok(proxies
                .0
                .into_iter()
                .map(|p| {
                    let delegate_ss58 =
                        sp_core::crypto::AccountId32::from(p.delegate.0).to_string();
                    let proxy_type = format!("{:?}", p.proxy_type);
                    (delegate_ss58, proxy_type, p.delay)
                })
                .collect()),
            None => Ok(vec![]),
        }
    }

    // ──────── Coldkey Swap Detection ────────

    /// Check if a coldkey has a scheduled swap. Returns (execution_block, new_coldkey_ss58) if scheduled.
    pub async fn get_coldkey_swap_scheduled(&self, ss58: &str) -> Result<Option<(u32, String)>> {
        let account_id = Self::ss58_to_account_id(ss58)?;
        let addr = api::storage()
            .subtensor_module()
            .coldkey_swap_scheduled(&account_id);
        let result = self.inner.storage().at_latest().await?.fetch(&addr).await?;
        Ok(result.map(|(block, new_coldkey)| {
            let new_ss58 = sp_core::crypto::AccountId32::from(new_coldkey.0).to_string();
            (block, new_ss58)
        }))
    }

    // ──────── Child Keys Query ────────

    /// Get child keys for a hotkey on a specific subnet. Returns Vec<(proportion, child_ss58)>.
    pub async fn get_child_keys(
        &self,
        hotkey_ss58: &str,
        netuid: NetUid,
    ) -> Result<Vec<(u64, String)>> {
        let account_id = Self::ss58_to_account_id(hotkey_ss58)?;
        let addr = api::storage()
            .subtensor_module()
            .child_keys(&account_id, netuid.0);
        let result = self.inner.storage().at_latest().await?.fetch(&addr).await?;
        Ok(result
            .unwrap_or_default()
            .into_iter()
            .map(|(proportion, child)| {
                let child_ss58 = sp_core::crypto::AccountId32::from(child.0).to_string();
                (proportion, child_ss58)
            })
            .collect())
    }

    /// Get parent keys for a hotkey on a specific subnet. Returns Vec<(proportion, parent_ss58)>.
    pub async fn get_parent_keys(
        &self,
        hotkey_ss58: &str,
        netuid: NetUid,
    ) -> Result<Vec<(u64, String)>> {
        let account_id = Self::ss58_to_account_id(hotkey_ss58)?;
        let addr = api::storage()
            .subtensor_module()
            .parent_keys(&account_id, netuid.0);
        let result = self.inner.storage().at_latest().await?.fetch(&addr).await?;
        Ok(result
            .unwrap_or_default()
            .into_iter()
            .map(|(proportion, parent)| {
                let parent_ss58 = sp_core::crypto::AccountId32::from(parent.0).to_string();
                (proportion, parent_ss58)
            })
            .collect())
    }

    // ──────── Pending Child Keys Query ────────

    /// Get pending child key changes for a hotkey on a specific subnet.
    /// Returns (Vec<(proportion, child_ss58)>, cooldown_block) if pending, or None.
    pub async fn get_pending_child_keys(
        &self,
        hotkey_ss58: &str,
        netuid: NetUid,
    ) -> Result<Option<(Vec<(u64, String)>, u64)>> {
        let account_id = Self::ss58_to_account_id(hotkey_ss58)?;
        let addr = api::storage()
            .subtensor_module()
            .pending_child_keys(netuid.0, &account_id);
        let result = self.inner.storage().at_latest().await?.fetch(&addr).await?;
        Ok(result.map(|(children, cooldown_block)| {
            let children_parsed: Vec<(u64, String)> = children
                .into_iter()
                .map(|(proportion, child)| {
                    let child_ss58 = sp_core::crypto::AccountId32::from(child.0).to_string();
                    (proportion, child_ss58)
                })
                .collect();
            (children_parsed, cooldown_block)
        }))
    }

    // ──────── Historical Queries ────────

    /// Get stake info for a coldkey at a specific block hash (via runtime API at block).
    pub async fn get_stake_for_coldkey_at_block(
        &self,
        coldkey_ss58: &str,
        block_hash: subxt::utils::H256,
    ) -> Result<Vec<StakeInfo>> {
        let account_id = Self::ss58_to_account_id(coldkey_ss58)?;
        let payload = api::apis()
            .stake_info_runtime_api()
            .get_stake_info_for_coldkey(account_id);
        let result = self
            .inner
            .runtime_api()
            .at(block_hash)
            .call(payload)
            .await
            .map_err(|e| Self::annotate_at_block_error(e.into(), None))?;
        Ok(result.into_iter().map(StakeInfo::from).collect())
    }

    /// Get identity at a specific block hash.
    pub async fn get_identity_at_block(
        &self,
        ss58: &str,
        block_hash: subxt::utils::H256,
    ) -> Result<Option<ChainIdentity>> {
        let account_id = Self::ss58_to_account_id(ss58)?;
        let addr = api::storage().registry().identity_of(&account_id);
        let result = self
            .inner
            .storage()
            .at(block_hash)
            .fetch(&addr)
            .await
            .map_err(|e| Self::annotate_at_block_error(e.into(), None))?;
        Ok(result.map(|reg| {
            let info = reg.info;
            ChainIdentity {
                name: decode_identity_data(&info.display),
                url: decode_identity_data(&info.web),
                github: String::new(),
                image: decode_identity_data(&info.image),
                discord: decode_identity_data(&info.riot),
                description: String::new(),
                additional: info
                    .additional
                    .0
                    .iter()
                    .map(|(k, v)| {
                        format!("{}={}", decode_identity_data(k), decode_identity_data(v))
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
            }
        }))
    }

    /// Get all subnets at a specific block hash (via runtime API at block).
    pub async fn get_all_subnets_at_block(
        &self,
        block_hash: subxt::utils::H256,
    ) -> Result<Vec<SubnetInfo>> {
        let payload = api::apis().subnet_info_runtime_api().get_subnets_info();
        let result = self
            .inner
            .runtime_api()
            .at(block_hash)
            .call(payload)
            .await
            .map_err(|e| Self::annotate_at_block_error(e.into(), None))?;
        Ok(result.into_iter().flatten().map(SubnetInfo::from).collect())
    }

    /// Get all dynamic info at a specific block hash.
    pub async fn get_all_dynamic_info_at_block(
        &self,
        block_hash: subxt::utils::H256,
    ) -> Result<Vec<DynamicInfo>> {
        let payload = api::apis().subnet_info_runtime_api().get_all_dynamic_info();
        let result = self
            .inner
            .runtime_api()
            .at(block_hash)
            .call(payload)
            .await
            .map_err(|e| Self::annotate_at_block_error(e.into(), None))?;
        Ok(result
            .into_iter()
            .flatten()
            .map(DynamicInfo::from)
            .collect())
    }

    /// Get dynamic info for a specific subnet at a block hash.
    pub async fn get_dynamic_info_at_block(
        &self,
        netuid: NetUid,
        block_hash: subxt::utils::H256,
    ) -> Result<Option<DynamicInfo>> {
        let payload = api::apis()
            .subnet_info_runtime_api()
            .get_dynamic_info(netuid.0);
        let result = self
            .inner
            .runtime_api()
            .at(block_hash)
            .call(payload)
            .await
            .map_err(|e| Self::annotate_at_block_error(e.into(), None))?;
        Ok(result.map(DynamicInfo::from))
    }

    /// Get lightweight neuron info for a subnet at a specific block hash.
    pub async fn get_neurons_lite_at_block(
        &self,
        netuid: NetUid,
        block_hash: subxt::utils::H256,
    ) -> Result<Vec<NeuronInfoLite>> {
        let payload = api::apis()
            .neuron_info_runtime_api()
            .get_neurons_lite(netuid.0);
        let result = self
            .inner
            .runtime_api()
            .at(block_hash)
            .call(payload)
            .await
            .map_err(|e| Self::annotate_at_block_error(e.into(), None))?;
        Ok(result.into_iter().map(NeuronInfoLite::from).collect())
    }

    /// Get full neuron info for a specific UID at a block hash.
    pub async fn get_neuron_at_block(
        &self,
        netuid: NetUid,
        uid: u16,
        block_hash: subxt::utils::H256,
    ) -> Result<Option<NeuronInfo>> {
        let payload = api::apis()
            .neuron_info_runtime_api()
            .get_neuron(netuid.0, uid);
        let result = self
            .inner
            .runtime_api()
            .at(block_hash)
            .call(payload)
            .await
            .map_err(|e| Self::annotate_at_block_error(e.into(), None))?;
        Ok(result.map(NeuronInfo::from))
    }

    /// Get all delegates at a specific block hash.
    pub async fn get_delegates_at_block(
        &self,
        block_hash: subxt::utils::H256,
    ) -> Result<Vec<DelegateInfo>> {
        let payload = api::apis().delegate_info_runtime_api().get_delegates();
        let result = self
            .inner
            .runtime_api()
            .at(block_hash)
            .call(payload)
            .await
            .map_err(|e| Self::annotate_at_block_error(e.into(), None))?;
        Ok(result.into_iter().map(DelegateInfo::from).collect())
    }

    /// Get total issuance at a specific block hash.
    pub async fn get_total_issuance_at_block(
        &self,
        block_hash: subxt::utils::H256,
    ) -> Result<Balance> {
        let addr = api::storage().balances().total_issuance();
        let val = self
            .inner
            .storage()
            .at(block_hash)
            .fetch(&addr)
            .await
            .map_err(|e| Self::annotate_at_block_error(e.into(), None))?;
        Ok(Balance::from_rao(val.unwrap_or(0) as u64))
    }

    // ──────── Block Header / Info ────────

    /// Get block header (number, hash, parent_hash, extrinsics_root, state_root).
    pub async fn get_block_header(
        &self,
        block_hash: subxt::utils::H256,
    ) -> Result<(
        u32,
        subxt::utils::H256,
        subxt::utils::H256,
        subxt::utils::H256,
    )> {
        let block = self.inner.blocks().at(block_hash).await?;
        let header = block.header();
        Ok((
            header.number,
            block_hash,
            header.parent_hash,
            header.state_root,
        ))
    }

    /// Get extrinsic count in a block.
    pub async fn get_block_extrinsic_count(&self, block_hash: subxt::utils::H256) -> Result<usize> {
        let block = self.inner.blocks().at(block_hash).await?;
        let extrinsics = block.extrinsics().await?;
        Ok(extrinsics.len())
    }

    /// Get the timestamp for a block by reading the Timestamp.set() inherent.
    pub async fn get_block_timestamp(&self, block_hash: subxt::utils::H256) -> Result<Option<u64>> {
        let addr = api::storage().timestamp().now();
        let val = self.inner.storage().at(block_hash).fetch(&addr).await?;
        Ok(val)
    }

    // ──────── Swap Simulation (Runtime APIs) ────────

    /// Get current alpha price for a subnet.
    pub async fn current_alpha_price(&self, netuid: NetUid) -> Result<u64> {
        let payload = api::apis().swap_runtime_api().current_alpha_price(netuid.0);
        let result = self
            .inner
            .runtime_api()
            .at_latest()
            .await?
            .call(payload)
            .await?;
        Ok(result)
    }

    /// Simulate swapping TAO for alpha on a subnet.
    /// Returns (alpha_amount, tao_fee, alpha_fee).
    pub async fn sim_swap_tao_for_alpha(
        &self,
        netuid: NetUid,
        tao_rao: u64,
    ) -> Result<(u64, u64, u64)> {
        let payload = api::apis()
            .swap_runtime_api()
            .sim_swap_tao_for_alpha(netuid.0, tao_rao);
        let result = self
            .inner
            .runtime_api()
            .at_latest()
            .await?
            .call(payload)
            .await?;
        Ok((result.alpha_amount, result.tao_fee, result.alpha_fee))
    }

    /// Simulate swapping alpha for TAO on a subnet.
    /// Returns (tao_amount, tao_fee, alpha_fee).
    pub async fn sim_swap_alpha_for_tao(
        &self,
        netuid: NetUid,
        alpha: u64,
    ) -> Result<(u64, u64, u64)> {
        let payload = api::apis()
            .swap_runtime_api()
            .sim_swap_alpha_for_tao(netuid.0, alpha);
        let result = self
            .inner
            .runtime_api()
            .at_latest()
            .await?
            .call(payload)
            .await?;
        Ok((result.tao_amount, result.tao_fee, result.alpha_fee))
    }
}

/// Decode the Registry pallet `Data` enum to a string.
/// Uses a macro to collapse the 33 Raw variants into a single pattern.
fn decode_identity_data(data: &api::runtime_types::pallet_registry::types::Data) -> String {
    use api::runtime_types::pallet_registry::types::Data;
    macro_rules! raw_to_string {
        ($($variant:ident),+) => {
            match data {
                Data::None => String::new(),
                $(Data::$variant(b) => String::from_utf8_lossy(b).into_owned(),)+
                _ => format!("<hash:{:?}>", data),
            }
        }
    }
    raw_to_string!(
        Raw0, Raw1, Raw2, Raw3, Raw4, Raw5, Raw6, Raw7, Raw8, Raw9, Raw10, Raw11, Raw12, Raw13,
        Raw14, Raw15, Raw16, Raw17, Raw18, Raw19, Raw20, Raw21, Raw22, Raw23, Raw24, Raw25, Raw26,
        Raw27, Raw28, Raw29, Raw30, Raw31, Raw32
    )
}
