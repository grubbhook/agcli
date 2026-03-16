//! Chain query methods — subnet, neuron, delegate, identity, and historical queries.

use anyhow::{Context, Result};

use crate::api;
use crate::types::balance::Balance;
use crate::types::chain_data::*;
use crate::types::network::NetUid;

use super::{retry_on_transient, Client, RPC_RETRIES};

impl Client {
    // ──────── Stake Queries ────────

    /// Get all stakes for a coldkey (via runtime API).
    pub async fn get_stake_for_coldkey(&self, coldkey_ss58: &str) -> Result<Vec<StakeInfo>> {
        let start = std::time::Instant::now();
        let account_id = Self::ss58_to_account_id(coldkey_ss58)?;
        let inner = &self.inner;
        let short = crate::utils::short_ss58(coldkey_ss58);
        let result = retry_on_transient("get_stake_for_coldkey", RPC_RETRIES, || async {
            let payload = api::apis()
                .stake_info_runtime_api()
                .get_stake_info_for_coldkey(account_id.clone());
            let r = inner
                .runtime_api()
                .at_latest()
                .await
                .context("Failed to get latest block for stake query")?
                .call(payload)
                .await
                .with_context(|| format!("Failed to query stakes for coldkey {}", short))?;
            Ok(r)
        })
        .await?;
        let stakes: Vec<StakeInfo> = result.into_iter().map(StakeInfo::from).collect();
        tracing::debug!(
            elapsed_ms = start.elapsed().as_millis() as u64,
            count = stakes.len(),
            "get_stake_for_coldkey"
        );
        Ok(stakes)
    }

    // ──────── Subnet Queries ────────

    /// List all subnets (via runtime API, cached for 30s).
    /// Returns `Arc<Vec<SubnetInfo>>` to avoid cloning the entire collection.
    pub async fn get_all_subnets(&self) -> Result<std::sync::Arc<Vec<SubnetInfo>>> {
        let inner = &self.inner;
        self.cache
            .get_all_subnets(|| async {
                retry_on_transient("get_all_subnets", RPC_RETRIES, || async {
                    let payload = api::apis().subnet_info_runtime_api().get_subnets_info();
                    let result = inner
                        .runtime_api()
                        .at_latest()
                        .await
                        .context("Failed to get latest block for subnet list")?
                        .call(payload)
                        .await
                        .context("Failed to query all subnets")?;
                    Ok(result.into_iter().flatten().map(SubnetInfo::from).collect())
                })
                .await
            })
            .await
    }

    /// Get info for a specific subnet.
    pub async fn get_subnet_info(&self, netuid: NetUid) -> Result<Option<SubnetInfo>> {
        let inner = &self.inner;
        let nid = netuid.0;
        let result = retry_on_transient("get_subnet_info", RPC_RETRIES, || async {
            let payload = api::apis().subnet_info_runtime_api().get_subnet_info(nid);
            let r = inner
                .runtime_api()
                .at_latest()
                .await
                .context("Failed to get latest block for subnet query")?
                .call(payload)
                .await
                .with_context(|| format!("Failed to query subnet info for SN{}", nid))?;
            Ok(r)
        })
        .await?;
        Ok(result.map(SubnetInfo::from))
    }

    /// Get subnet hyperparameters.
    pub async fn get_subnet_hyperparams(
        &self,
        netuid: NetUid,
    ) -> Result<Option<SubnetHyperparameters>> {
        let inner = &self.inner;
        let nid = netuid.0;
        let result = retry_on_transient("get_subnet_hyperparams", RPC_RETRIES, || async {
            let payload = api::apis()
                .subnet_info_runtime_api()
                .get_subnet_hyperparams(nid);
            let r = inner
                .runtime_api()
                .at_latest()
                .await
                .context("Failed to get latest block for hyperparams query")?
                .call(payload)
                .await
                .with_context(|| format!("Failed to query hyperparams for SN{}", nid))?;
            Ok(r)
        })
        .await?;
        Ok(result.map(|h| SubnetHyperparameters::from_gen(h, netuid)))
    }

    /// Get dynamic info for all subnets (cached for 30s).
    /// Returns `Arc<Vec<DynamicInfo>>` to avoid cloning the entire collection.
    pub async fn get_all_dynamic_info(&self) -> Result<std::sync::Arc<Vec<DynamicInfo>>> {
        let inner = &self.inner;
        self.cache
            .get_all_dynamic_info(|| async {
                retry_on_transient("get_all_dynamic_info", RPC_RETRIES, || async {
                    let payload = api::apis().subnet_info_runtime_api().get_all_dynamic_info();
                    let result = inner
                        .runtime_api()
                        .at_latest()
                        .await
                        .context("Failed to get latest block for dynamic info")?
                        .call(payload)
                        .await
                        .context("Failed to query all dynamic info")?;
                    Ok(result
                        .into_iter()
                        .flatten()
                        .map(DynamicInfo::from)
                        .collect())
                })
                .await
            })
            .await
    }

    /// Get dynamic info for a specific subnet (cached for 30s).
    pub async fn get_dynamic_info(&self, netuid: NetUid) -> Result<Option<DynamicInfo>> {
        let inner = &self.inner;
        let nid = netuid.0;
        let result = self
            .cache
            .get_dynamic_info(nid, || async {
                retry_on_transient("get_dynamic_info", RPC_RETRIES, || async {
                    let payload = api::apis().subnet_info_runtime_api().get_dynamic_info(nid);
                    let r = inner
                        .runtime_api()
                        .at_latest()
                        .await
                        .context("Failed to get latest block for dynamic info")?
                        .call(payload)
                        .await
                        .with_context(|| format!("Failed to query dynamic info for SN{}", nid))?;
                    Ok(r.map(DynamicInfo::from))
                })
                .await
            })
            .await?;
        Ok(result.map(|arc| (*arc).clone()))
    }

    // ──────── Neuron / Metagraph Queries ────────

    /// Get lightweight neuron info for a subnet (via runtime API, cached 30s).
    /// Returns `Arc<Vec<NeuronInfoLite>>` to avoid cloning thousands of neuron records.
    /// This is one of the most expensive chain queries — caching with request coalescing
    /// prevents redundant fetches when multiple commands or views access the same subnet.
    pub async fn get_neurons_lite(
        &self,
        netuid: NetUid,
    ) -> Result<std::sync::Arc<Vec<NeuronInfoLite>>> {
        let inner = &self.inner;
        let nid = netuid.0;
        self.cache
            .get_neurons_lite(nid, || async {
                retry_on_transient("get_neurons_lite", RPC_RETRIES, || async {
                    let payload = api::apis().neuron_info_runtime_api().get_neurons_lite(nid);
                    let r = inner
                        .runtime_api()
                        .at_latest()
                        .await
                        .context("Failed to get latest block for neuron query")?
                        .call(payload)
                        .await
                        .with_context(|| format!("Failed to query neurons for SN{}", nid))?;
                    Ok(r.into_iter().map(NeuronInfoLite::from).collect())
                })
                .await
            })
            .await
    }

    /// Get full neuron info for a specific UID.
    pub async fn get_neuron(&self, netuid: NetUid, uid: u16) -> Result<Option<NeuronInfo>> {
        let inner = &self.inner;
        let nid = netuid.0;
        let result = retry_on_transient("get_neuron", RPC_RETRIES, || async {
            let payload = api::apis().neuron_info_runtime_api().get_neuron(nid, uid);
            let r = inner
                .runtime_api()
                .at_latest()
                .await
                .context("Failed to get latest block for neuron query")?
                .call(payload)
                .await
                .with_context(|| format!("Failed to query neuron UID {} on SN{}", uid, nid))?;
            Ok(r)
        })
        .await?;
        Ok(result.map(NeuronInfo::from))
    }

    /// Get the metagraph for a subnet.
    pub async fn get_metagraph(&self, netuid: NetUid) -> Result<Metagraph> {
        crate::queries::fetch_metagraph(self, netuid).await
    }

    // ──────── Delegate Queries ────────

    /// Get all delegates (via runtime API, cached for 30s).
    /// Returns `Arc<Vec<DelegateInfo>>` to avoid cloning the entire collection.
    pub async fn get_all_delegates_cached(&self) -> Result<std::sync::Arc<Vec<DelegateInfo>>> {
        let inner = &self.inner;
        self.cache
            .get_all_delegates(|| async {
                retry_on_transient("get_delegates", RPC_RETRIES, || async {
                    let payload = api::apis().delegate_info_runtime_api().get_delegates();
                    let r = inner
                        .runtime_api()
                        .at_latest()
                        .await
                        .context("Failed to get latest block for delegate query")?
                        .call(payload)
                        .await
                        .context("Failed to query delegates")?;
                    Ok(r.into_iter().map(DelegateInfo::from).collect())
                })
                .await
            })
            .await
    }

    /// Get all delegates (via runtime API). Uncached — prefer `get_all_delegates_cached()`.
    pub async fn get_delegates(&self) -> Result<Vec<DelegateInfo>> {
        let arc = self.get_all_delegates_cached().await?;
        Ok((*arc).clone())
    }

    /// Get delegate info for a specific hotkey.
    pub async fn get_delegate(&self, hotkey_ss58: &str) -> Result<Option<DelegateInfo>> {
        let account_id = Self::ss58_to_account_id(hotkey_ss58)?;
        let inner = &self.inner;
        let result = retry_on_transient("get_delegate", RPC_RETRIES, || async {
            let payload = api::apis()
                .delegate_info_runtime_api()
                .get_delegate(account_id.clone());
            let r = inner
                .runtime_api()
                .at_latest()
                .await
                .context("Failed to get latest block for delegate query")?
                .call(payload)
                .await
                .context("Failed to query delegate info")?;
            Ok(r)
        })
        .await?;
        Ok(result.map(DelegateInfo::from))
    }

    // ──────── Identity Queries ────────

    /// Get on-chain identity for an account (from Registry pallet).
    pub async fn get_identity(&self, ss58: &str) -> Result<Option<ChainIdentity>> {
        let account_id = Self::ss58_to_account_id(ss58)?;
        let inner = &self.inner;
        let short = crate::utils::short_ss58(ss58);
        let result = retry_on_transient("get_identity", RPC_RETRIES, || async {
            let addr = api::storage().registry().identity_of(&account_id);
            let r = inner
                .storage()
                .at_latest()
                .await?
                .fetch(&addr)
                .await
                .with_context(|| format!("Failed to fetch identity for {}", short))?;
            Ok(r)
        })
        .await?;
        Ok(result.map(|reg| chain_identity_from_registration(reg.info)))
    }

    /// Get subnet identity (from SubtensorModule SubnetIdentitiesV3).
    pub async fn get_subnet_identity(&self, netuid: NetUid) -> Result<Option<SubnetIdentity>> {
        let inner = &self.inner;
        let nid = netuid.0;
        let result = retry_on_transient("get_subnet_identity", RPC_RETRIES, || async {
            let addr = api::storage().subtensor_module().subnet_identities_v3(nid);
            let r = inner
                .storage()
                .at_latest()
                .await?
                .fetch(&addr)
                .await
                .with_context(|| format!("Failed to fetch subnet identity for SN{}", nid))?;
            Ok(r)
        })
        .await?;
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
        let inner = &self.inner;
        let result = retry_on_transient("get_delegated", RPC_RETRIES, || async {
            let payload = api::apis()
                .delegate_info_runtime_api()
                .get_delegated(account_id.clone());
            let r = inner
                .runtime_api()
                .at_latest()
                .await
                .context("Failed to get latest block for delegated query")?
                .call(payload)
                .await
                .context("Failed to query delegated info")?;
            Ok(r)
        })
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
        let inner = &self.inner;
        let result = retry_on_transient("list_proxies", RPC_RETRIES, || async {
            let addr = api::storage().proxy().proxies(&account_id);
            let r = inner
                .storage()
                .at_latest()
                .await?
                .fetch(&addr)
                .await
                .context("Failed to fetch proxy list")?;
            Ok(r)
        })
        .await?;
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
        let inner = &self.inner;
        let result = retry_on_transient("get_coldkey_swap_scheduled", RPC_RETRIES, || async {
            let addr = api::storage()
                .subtensor_module()
                .coldkey_swap_scheduled(&account_id);
            let r = inner
                .storage()
                .at_latest()
                .await?
                .fetch(&addr)
                .await
                .context("Failed to query coldkey swap status")?;
            Ok(r)
        })
        .await?;
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
        let inner = &self.inner;
        let nid = netuid.0;
        let result = retry_on_transient("get_child_keys", RPC_RETRIES, || async {
            let addr = api::storage()
                .subtensor_module()
                .child_keys(&account_id, nid);
            let r = inner
                .storage()
                .at_latest()
                .await?
                .fetch(&addr)
                .await
                .context("Failed to fetch child keys")?;
            Ok(r)
        })
        .await?;
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
        let inner = &self.inner;
        let nid = netuid.0;
        let result = retry_on_transient("get_parent_keys", RPC_RETRIES, || async {
            let addr = api::storage()
                .subtensor_module()
                .parent_keys(&account_id, nid);
            let r = inner
                .storage()
                .at_latest()
                .await?
                .fetch(&addr)
                .await
                .context("Failed to fetch parent keys")?;
            Ok(r)
        })
        .await?;
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
        let inner = &self.inner;
        let nid = netuid.0;
        let result = retry_on_transient("get_pending_child_keys", RPC_RETRIES, || async {
            let addr = api::storage()
                .subtensor_module()
                .pending_child_keys(nid, &account_id);
            let r = inner
                .storage()
                .at_latest()
                .await?
                .fetch(&addr)
                .await
                .context("Failed to fetch pending child keys")?;
            Ok(r)
        })
        .await?;
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
        Ok(result.map(|reg| chain_identity_from_registration(reg.info)))
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
        let raw = val.unwrap_or(0);
        Ok(Balance::from_rao(u64::try_from(raw).unwrap_or(u64::MAX)))
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
        let inner = &self.inner;
        retry_on_transient("get_block_header", RPC_RETRIES, || async {
            let block = inner
                .blocks()
                .at(block_hash)
                .await
                .context("Failed to fetch block header")?;
            let header = block.header();
            Ok((
                header.number,
                block_hash,
                header.parent_hash,
                header.state_root,
            ))
        })
        .await
    }

    /// Get extrinsic count in a block.
    pub async fn get_block_extrinsic_count(&self, block_hash: subxt::utils::H256) -> Result<usize> {
        let inner = &self.inner;
        retry_on_transient("get_block_extrinsic_count", RPC_RETRIES, || async {
            let block = inner
                .blocks()
                .at(block_hash)
                .await
                .context("Failed to fetch block")?;
            let extrinsics = block
                .extrinsics()
                .await
                .context("Failed to decode block extrinsics")?;
            Ok(extrinsics.len())
        })
        .await
    }

    /// Get the timestamp for a block by reading the Timestamp.set() inherent.
    pub async fn get_block_timestamp(&self, block_hash: subxt::utils::H256) -> Result<Option<u64>> {
        let inner = &self.inner;
        retry_on_transient("get_block_timestamp", RPC_RETRIES, || async {
            let addr = api::storage().timestamp().now();
            let val = inner
                .storage()
                .at(block_hash)
                .fetch(&addr)
                .await
                .context("Failed to fetch block timestamp")?;
            Ok(val)
        })
        .await
    }

    // ──────── Swap Simulation (Runtime APIs) ────────

    /// Get current alpha price for a subnet.
    pub async fn current_alpha_price(&self, netuid: NetUid) -> Result<u64> {
        let inner = &self.inner;
        let nid = netuid.0;
        retry_on_transient("current_alpha_price", RPC_RETRIES, || async {
            let payload = api::apis().swap_runtime_api().current_alpha_price(nid);
            let r = inner
                .runtime_api()
                .at_latest()
                .await
                .context("Failed to get latest block for alpha price query")?
                .call(payload)
                .await
                .with_context(|| format!("Failed to query alpha price for SN{}", nid))?;
            Ok(r)
        })
        .await
    }

    /// Simulate swapping TAO for alpha on a subnet.
    /// Returns (alpha_amount, tao_fee, alpha_fee).
    pub async fn sim_swap_tao_for_alpha(
        &self,
        netuid: NetUid,
        tao_rao: u64,
    ) -> Result<(u64, u64, u64)> {
        let inner = &self.inner;
        let nid = netuid.0;
        let result = retry_on_transient("sim_swap_tao_for_alpha", RPC_RETRIES, || async {
            let payload = api::apis()
                .swap_runtime_api()
                .sim_swap_tao_for_alpha(nid, tao_rao);
            let r = inner
                .runtime_api()
                .at_latest()
                .await
                .context("Failed to get latest block for swap simulation")?
                .call(payload)
                .await
                .context("Failed to simulate TAO→alpha swap")?;
            Ok(r)
        })
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
        let inner = &self.inner;
        let nid = netuid.0;
        let result = retry_on_transient("sim_swap_alpha_for_tao", RPC_RETRIES, || async {
            let payload = api::apis()
                .swap_runtime_api()
                .sim_swap_alpha_for_tao(nid, alpha);
            let r = inner
                .runtime_api()
                .at_latest()
                .await
                .context("Failed to get latest block for swap simulation")?
                .call(payload)
                .await
                .context("Failed to simulate alpha→TAO swap")?;
            Ok(r)
        })
        .await?;
        Ok((result.tao_amount, result.tao_fee, result.alpha_fee))
    }

    // ──────── Auto-Stake Queries ────────

    /// Get auto-stake hotkey for a coldkey on a subnet.
    /// Returns the hotkey SS58 if set, or None.
    pub async fn get_auto_stake_hotkey(
        &self,
        coldkey_ss58: &str,
        netuid: NetUid,
    ) -> Result<Option<String>> {
        let coldkey_id = Self::ss58_to_account_id(coldkey_ss58)?;
        let inner = &self.inner;
        let nid = netuid.0;
        let result = retry_on_transient("get_auto_stake_hotkey", RPC_RETRIES, || async {
            let addr = subxt::dynamic::storage(
                "SubtensorModule",
                "AutoStakeHotkeys",
                vec![
                    subxt::dynamic::Value::from_bytes(coldkey_id.0),
                    subxt::dynamic::Value::u128(nid as u128),
                ],
            );
            let r = inner
                .storage()
                .at_latest()
                .await?
                .fetch(&addr)
                .await
                .context("Failed to fetch auto-stake hotkey")?;
            Ok(r)
        })
        .await?;
        match result {
            Some(val) => {
                let account_bytes: [u8; 32] = val.as_type()?;
                let account = crate::AccountId::from(account_bytes);
                Ok(Some(account.to_string()))
            }
            None => Ok(None),
        }
    }

    // ──────── Emission Split Queries ────────

    /// Get mechanism emission split for a subnet (if set).
    pub async fn get_emission_split(&self, netuid: NetUid) -> Result<Option<Vec<(String, u64)>>> {
        let inner = &self.inner;
        let nid = netuid.0;
        let result = retry_on_transient("get_emission_split", RPC_RETRIES, || async {
            let addr = subxt::dynamic::storage(
                "SubtensorModule",
                "MechanismEmissionSplit",
                vec![subxt::dynamic::Value::u128(nid as u128)],
            );
            let r = inner
                .storage()
                .at_latest()
                .await?
                .fetch(&addr)
                .await
                .with_context(|| format!("Failed to fetch emission split for SN{}", nid))?;
            Ok(r)
        })
        .await?;
        match result {
            Some(val) => {
                let raw: Vec<(u8, u64)> = val.as_type()?;
                let splits: Vec<(String, u64)> = raw
                    .into_iter()
                    .map(|(k, v)| {
                        let name = match k {
                            0 => "Yuma".to_string(),
                            1 => "Oracle".to_string(),
                            _ => format!("Unknown({})", k),
                        };
                        (name, v)
                    })
                    .collect();
                Ok(Some(splits))
            }
            None => Ok(None),
        }
    }

    // ──────── Subnet State Queries ────────

    /// Check if a subnet is active (emissions are running).
    pub async fn is_subnet_active(&self, netuid: NetUid) -> Result<bool> {
        let inner = &self.inner;
        let nid = netuid.0;
        let result = retry_on_transient("is_subnet_active", RPC_RETRIES, || async {
            let addr = subxt::dynamic::storage(
                "SubtensorModule",
                "NetworksAdded",
                vec![subxt::dynamic::Value::u128(nid as u128)],
            );
            let r = inner
                .storage()
                .at_latest()
                .await?
                .fetch(&addr)
                .await
                .with_context(|| format!("Failed to check active status for SN{}", nid))?;
            Ok(r)
        })
        .await?;
        match result {
            Some(val) => {
                // Try bool first, fall back to u8 (some runtimes encode as 0/1 u8)
                if let Ok(b) = val.as_type::<bool>() {
                    Ok(b)
                } else if let Ok(n) = val.as_type::<u8>() {
                    Ok(n != 0)
                } else {
                    anyhow::bail!(
                        "Failed to decode NetworksAdded for SN{}: not bool or u8",
                        nid
                    );
                }
            }
            None => Ok(false),
        }
    }

    /// Get mechanism count for a subnet.
    pub async fn get_mechanism_count(&self, netuid: NetUid) -> Result<u16> {
        let inner = &self.inner;
        let nid = netuid.0;
        let result = retry_on_transient("get_mechanism_count", RPC_RETRIES, || async {
            let addr = subxt::dynamic::storage(
                "SubtensorModule",
                "MechanismCountCurrent",
                vec![subxt::dynamic::Value::u128(nid as u128)],
            );
            let r = inner
                .storage()
                .at_latest()
                .await?
                .fetch(&addr)
                .await
                .with_context(|| format!("Failed to fetch mechanism count for SN{}", nid))?;
            Ok(r)
        })
        .await?;
        match result {
            Some(val) => Ok(val
                .as_type::<u16>()
                .context("Failed to decode mechanism count as u16")?),
            None => Ok(1), // Default is 1 mechanism
        }
    }

    // ──────── Crowdloan Queries ────────

    /// List all crowdloans by iterating Crowdloan storage.
    /// Returns Vec<(id, creator_ss58, deposit, raised, cap, end_block, finalized)>.
    pub async fn list_crowdloans(&self) -> Result<Vec<(u32, String, u64, u64, u64, u32, bool)>> {
        let inner = &self.inner;
        let mut results = Vec::new();
        let mut iter = retry_on_transient("list_crowdloans", RPC_RETRIES, || async {
            let addr = subxt::dynamic::storage("Crowdloan", "Crowdloans", ());
            let i = inner
                .storage()
                .at_latest()
                .await?
                .iter(addr)
                .await
                .context("Failed to iterate crowdloans")?;
            Ok(i)
        })
        .await?;
        while let Some(Ok(kv)) = iter.next().await {
            // Extract crowdloan ID from key (last 4 bytes for u32)
            let key_bytes = &kv.key_bytes;
            if key_bytes.len() >= 4 {
                let id_bytes: [u8; 4] = match key_bytes[key_bytes.len() - 4..].try_into() {
                    Ok(b) => b,
                    Err(_) => continue, // skip malformed key
                };
                let id = u32::from_le_bytes(id_bytes);

                // Try to decode the value
                if let Ok((
                    creator_bytes,
                    deposit,
                    raised,
                    cap,
                    end_block,
                    _min_contrib,
                    finalized,
                    _target,
                    _call,
                )) = kv.value.as_type::<(
                    [u8; 32],
                    u64,
                    u64,
                    u64,
                    u32,
                    u64,
                    bool,
                    Option<[u8; 32]>,
                    Option<Vec<u8>>,
                )>() {
                    let creator = crate::AccountId::from(creator_bytes).to_string();
                    results.push((id, creator, deposit, raised, cap, end_block, finalized));
                }
            }
        }
        results.sort_by_key(|(id, _, _, _, _, _, _)| *id);
        Ok(results)
    }

    /// Get detailed info for a specific crowdloan.
    /// Returns (creator, deposit, raised, cap, end_block, min_contribution, finalized, target).
    pub async fn get_crowdloan_info(
        &self,
        crowdloan_id: u32,
    ) -> Result<Option<(String, u64, u64, u64, u32, u64, bool, Option<String>)>> {
        let inner = &self.inner;
        let result = retry_on_transient("get_crowdloan_info", RPC_RETRIES, || async {
            let addr = subxt::dynamic::storage(
                "Crowdloan",
                "Crowdloans",
                vec![subxt::dynamic::Value::u128(crowdloan_id as u128)],
            );
            let r = inner
                .storage()
                .at_latest()
                .await?
                .fetch(&addr)
                .await
                .with_context(|| format!("Failed to fetch crowdloan {}", crowdloan_id))?;
            Ok(r)
        })
        .await?;
        match result {
            Some(val) => {
                if let Ok((
                    creator_bytes,
                    deposit,
                    raised,
                    cap,
                    end_block,
                    min_contrib,
                    finalized,
                    target_opt,
                    _call,
                )) = val.as_type::<(
                    [u8; 32],
                    u64,
                    u64,
                    u64,
                    u32,
                    u64,
                    bool,
                    Option<[u8; 32]>,
                    Option<Vec<u8>>,
                )>() {
                    let creator = crate::AccountId::from(creator_bytes).to_string();
                    let target = target_opt.map(|t| crate::AccountId::from(t).to_string());
                    Ok(Some((
                        creator,
                        deposit,
                        raised,
                        cap,
                        end_block,
                        min_contrib,
                        finalized,
                        target,
                    )))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    /// Get contributors for a specific crowdloan.
    /// Returns Vec<(address, amount_rao)>.
    pub async fn get_crowdloan_contributors(
        &self,
        crowdloan_id: u32,
    ) -> Result<Vec<(String, u64)>> {
        let inner = &self.inner;
        let mut results = Vec::new();
        let mut iter = retry_on_transient("get_crowdloan_contributors", RPC_RETRIES, || async {
            let addr = subxt::dynamic::storage(
                "Crowdloan",
                "Contributors",
                vec![subxt::dynamic::Value::u128(crowdloan_id as u128)],
            );
            let i = inner
                .storage()
                .at_latest()
                .await?
                .iter(addr)
                .await
                .with_context(|| {
                    format!(
                        "Failed to iterate contributors for crowdloan {}",
                        crowdloan_id
                    )
                })?;
            Ok(i)
        })
        .await?;
        while let Some(Ok(kv)) = iter.next().await {
            let key_bytes = &kv.key_bytes;
            if key_bytes.len() >= 32 {
                let account_bytes: [u8; 32] = match key_bytes[key_bytes.len() - 32..].try_into() {
                    Ok(b) => b,
                    Err(_) => continue, // skip malformed key
                };
                let account = crate::AccountId::from(account_bytes).to_string();
                if let Ok(amount) = kv.value.as_type::<u64>() {
                    results.push((account, amount));
                }
            }
        }
        results.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by amount descending
        Ok(results)
    }

    // ──────── Weight Reads ────────

    /// Read the on-chain weights set by a specific UID on a subnet.
    /// Returns Vec<(target_uid, weight_value)>.
    pub async fn get_weights_for_uid(&self, netuid: NetUid, uid: u16) -> Result<Vec<(u16, u16)>> {
        let addr = api::storage().subtensor_module().weights(netuid.0, uid);
        let val = self
            .inner
            .storage()
            .at_latest()
            .await?
            .fetch(&addr)
            .await
            .with_context(|| {
                format!("Failed to fetch weights for UID {} on SN{}", uid, netuid.0)
            })?;
        Ok(val.unwrap_or_default())
    }

    /// Read all on-chain weights for a subnet (all UIDs).
    /// Returns Vec<(setting_uid, Vec<(target_uid, weight)>)>.
    pub async fn get_all_weights(&self, netuid: NetUid) -> Result<Vec<(u16, Vec<(u16, u16)>)>> {
        let addr = api::storage().subtensor_module().weights_iter1(netuid.0);
        let mut results = Vec::new();
        let mut iter = self.inner.storage().at_latest().await?.iter(addr).await?;
        while let Some(Ok(kv)) = iter.next().await {
            // Extract UID from the storage key (last 2 bytes before key suffix)
            let key_bytes = &kv.key_bytes;
            if key_bytes.len() >= 2 {
                let uid_bytes: [u8; 2] = match key_bytes[key_bytes.len() - 2..].try_into() {
                    Ok(b) => b,
                    Err(_) => continue, // skip malformed key
                };
                let uid = u16::from_le_bytes(uid_bytes);
                results.push((uid, kv.value));
            }
        }
        results.sort_by_key(|(uid, _)| *uid);
        Ok(results)
    }
}

/// Convert a Registry pallet `IdentityInfo` into our `ChainIdentity` struct.
fn chain_identity_from_registration(
    info: api::runtime_types::pallet_registry::types::IdentityInfo,
) -> ChainIdentity {
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
            .map(|(k, v)| format!("{}={}", decode_identity_data(k), decode_identity_data(v)))
            .collect::<Vec<_>>()
            .join(", "),
    }
}

// ──────── Commitments ────────

impl Client {
    /// Get commitment for a specific hotkey on a subnet.
    /// Returns (block_number, decoded_fields) or None.
    pub async fn get_commitment(
        &self,
        netuid: u16,
        hotkey_ss58: &str,
    ) -> Result<Option<(u32, Vec<String>)>> {
        let account_id = Self::ss58_to_account_id(hotkey_ss58)?;
        let addr = api::storage()
            .commitments()
            .commitment_of(netuid, &account_id);
        let inner = &self.inner;
        let result = retry_on_transient("get_commitment", RPC_RETRIES, || async {
            inner
                .storage()
                .at_latest()
                .await
                .context("Failed to get latest block for commitment query")?
                .fetch(&addr)
                .await
                .context("Failed to query commitment")
        })
        .await?;

        Ok(result.map(|reg| {
            let block = reg.block;
            let fields: Vec<String> = reg
                .info
                .fields
                .0
                .iter()
                .map(decode_commitment_data)
                .collect();
            (block, fields)
        }))
    }

    /// List all commitments on a subnet.
    /// Returns Vec<(ss58, block, fields)>.
    pub async fn get_all_commitments(
        &self,
        netuid: u16,
    ) -> Result<Vec<(String, u32, Vec<String>)>> {
        let addr = api::storage().commitments().commitment_of_iter1(netuid);
        let inner = &self.inner;
        let mut entries = retry_on_transient("get_all_commitments", RPC_RETRIES, || async {
            let stream = inner
                .storage()
                .at_latest()
                .await
                .context("Failed to get latest block for commitment list")?
                .iter(addr.clone())
                .await
                .context("Failed to iterate commitments")?;
            Ok(stream)
        })
        .await?;

        let mut result = Vec::new();
        while let Some(Ok(kv)) = entries.next().await {
            // Extract the account_id from the storage key (last 32 bytes)
            let key_bytes = kv.key_bytes;
            if key_bytes.len() >= 32 {
                let account_bytes: [u8; 32] = match key_bytes[key_bytes.len() - 32..].try_into() {
                    Ok(b) => b,
                    Err(_) => continue, // skip malformed key
                };
                let account_id = subxt::ext::subxt_core::utils::AccountId32::from(account_bytes);
                let ss58 = account_id.to_string();
                let block = kv.value.block;
                let fields: Vec<String> = kv
                    .value
                    .info
                    .fields
                    .0
                    .iter()
                    .map(decode_commitment_data)
                    .collect();
                result.push((ss58, block, fields));
            }
        }
        Ok(result)
    }
}

fn decode_commitment_data(data: &api::runtime_types::pallet_commitments::types::Data) -> String {
    use api::runtime_types::pallet_commitments::types::Data;
    macro_rules! raw_to_string {
        ($($variant:ident),+) => {
            match data {
                Data::None => String::new(),
                $(Data::$variant(b) => String::from_utf8_lossy(b).into_owned(),)+
                _ => format!("{:?}", data),
            }
        }
    }
    raw_to_string!(
        Raw0, Raw1, Raw2, Raw3, Raw4, Raw5, Raw6, Raw7, Raw8, Raw9, Raw10, Raw11, Raw12, Raw13,
        Raw14, Raw15, Raw16, Raw17, Raw18, Raw19, Raw20, Raw21, Raw22, Raw23, Raw24, Raw25, Raw26,
        Raw27, Raw28, Raw29, Raw30, Raw31, Raw32, Raw33, Raw34, Raw35, Raw36, Raw37, Raw38, Raw39,
        Raw40, Raw41, Raw42, Raw43, Raw44, Raw45, Raw46, Raw47, Raw48, Raw49, Raw50, Raw51, Raw52,
        Raw53, Raw54, Raw55, Raw56, Raw57, Raw58, Raw59, Raw60, Raw61, Raw62, Raw63, Raw64, Raw65,
        Raw66, Raw67, Raw68, Raw69, Raw70, Raw71, Raw72, Raw73, Raw74, Raw75, Raw76, Raw77, Raw78,
        Raw79, Raw80, Raw81, Raw82, Raw83, Raw84, Raw85, Raw86, Raw87, Raw88, Raw89, Raw90, Raw91,
        Raw92, Raw93, Raw94, Raw95, Raw96, Raw97, Raw98, Raw99, Raw100, Raw101, Raw102, Raw103,
        Raw104, Raw105, Raw106, Raw107, Raw108, Raw109, Raw110, Raw111, Raw112, Raw113, Raw114,
        Raw115, Raw116, Raw117, Raw118, Raw119, Raw120, Raw121, Raw122, Raw123, Raw124, Raw125,
        Raw126, Raw127, Raw128
    )
}

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
