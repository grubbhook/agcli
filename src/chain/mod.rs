//! Substrate chain client — connect, query storage, submit extrinsics.

pub mod connection;
pub mod rpc_types;
pub mod storage;

use anyhow::{Context, Result};
use sp_core::{sr25519, Pair as _};
use subxt::backend::legacy::rpc_methods::LegacyRpcMethods;
use subxt::backend::rpc::RpcClient;
use subxt::tx::PairSigner;
use subxt::OnlineClient;

use crate::queries::query_cache::QueryCache;
use crate::types::balance::Balance;
use crate::types::chain_data::*;
use crate::types::network::NetUid;
use crate::{api, AccountId, SubtensorConfig};

// Re-export for event subscription
pub use subxt;

/// Signer type for extrinsic submission.
pub type Signer = PairSigner<SubtensorConfig, sr25519::Pair>;

/// High-level client for the Bittensor (subtensor) chain.
pub struct Client {
    inner: OnlineClient<SubtensorConfig>,
    rpc: LegacyRpcMethods<SubtensorConfig>,
    cache: QueryCache,
}

impl Client {
    /// Connect to a subtensor node (single URL, no retry).
    async fn connect_once(url: &str) -> Result<Self> {
        let start = std::time::Instant::now();
        tracing::info!("Connecting to {}", url);
        let rpc_client = RpcClient::from_url(url)
            .await
            .with_context(|| format!(
                "Failed to connect to subtensor node at '{}'. Check your network connection and endpoint.\n  Finney:  wss://entrypoint-finney.opentensor.ai:443\n  Test:    wss://test.finney.opentensor.ai:443\n  Archive: wss://bittensor-finney.api.onfinality.io/public-ws\n  Local:   ws://127.0.0.1:9944\n  Set with: --network finney|test|local|archive  or  --endpoint <url>",
                url
            ))?;
        let rpc = LegacyRpcMethods::new(rpc_client.clone());
        let inner = OnlineClient::from_rpc_client(rpc_client)
            .await
            .with_context(|| "Failed to initialize subxt client from RPC connection")?;
        tracing::info!("Connected to {} in {:?}", url, start.elapsed());
        Ok(Self { inner, rpc, cache: QueryCache::new() })
    }

    /// Connect to a subtensor node with retry + exponential backoff.
    /// Tries each URL in order, retrying up to 3 times per URL with 1s→2s→4s delays.
    pub async fn connect(url: &str) -> Result<Self> {
        Self::connect_with_retry(&[url]).await
    }

    /// Connect with retry across multiple endpoints.
    /// Tries each URL in order; on failure retries with exponential backoff (1s, 2s, 4s).
    pub async fn connect_with_retry(urls: &[&str]) -> Result<Self> {
        let max_retries: u32 = 3;
        let mut last_err = None;

        for url in urls {
            for attempt in 0..max_retries {
                if attempt > 0 {
                    let delay = std::time::Duration::from_secs(1 << (attempt - 1));
                    tracing::warn!("Retry {}/{} for {} in {:?}", attempt, max_retries - 1, url, delay);
                    tokio::time::sleep(delay).await;
                }
                match Self::connect_once(url).await {
                    Ok(client) => {
                        if attempt > 0 {
                            tracing::info!("Connected to {} on attempt {}", url, attempt + 1);
                        }
                        return Ok(client);
                    }
                    Err(e) => {
                        tracing::warn!("Connection attempt {} to {} failed: {}", attempt + 1, url, e);
                        last_err = Some(e);
                    }
                }
            }
            tracing::warn!("All {} attempts to {} exhausted, trying next endpoint", max_retries, url);
        }

        Err(last_err.unwrap_or_else(|| anyhow::anyhow!("No endpoints provided")))
    }

    /// Connect to a well-known network with automatic fallback endpoints.
    pub async fn connect_network(network: &crate::types::Network) -> Result<Self> {
        let urls = network.ws_urls();
        Self::connect_with_retry(&urls).await
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

    /// Public version of ss58_to_account_id for use outside chain module.
    pub fn ss58_to_account_id_pub(ss58: &str) -> Result<AccountId> {
        Self::ss58_to_account_id(ss58)
    }

    /// Sign, submit, and wait for finalization of a typed extrinsic.
    /// Returns the extrinsic hash. Provides contextual error messages for common failures.
    async fn sign_submit<T: subxt::tx::Payload>(
        &self,
        tx: &T,
        pair: &sr25519::Pair,
    ) -> Result<String> {
        let signer = Self::signer(pair);
        let progress = self
            .inner
            .tx()
            .sign_and_submit_then_watch_default(tx, &signer)
            .await
            .map_err(format_submit_error)?;
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            progress.wait_for_finalized_success(),
        )
        .await
        .map_err(|_| anyhow::anyhow!(
            "Transaction timed out after 30s waiting for finalization. \
             The extrinsic may have been dropped from the pool \
             (insufficient balance, invalid state, or node not producing blocks)."
        ))?
        .map_err(format_dispatch_error)?;
        Ok(format!("{:?}", result.extrinsic_hash()))
    }

    // ──────── Balance Queries ────────

    /// Get TAO balance (free) for an account.
    pub async fn get_balance(&self, account: &sr25519::Public) -> Result<Balance> {
        let account_id = Self::to_account_id(account);
        let addr = api::storage().system().account(&account_id);
        let info = self.inner.storage().at_latest().await?.fetch(&addr).await?;
        match info {
            Some(info) => Ok(Balance::from_rao(info.data.free)),
            None => Ok(Balance::ZERO),
        }
    }

    /// Get TAO balance for an SS58 address.
    pub async fn get_balance_ss58(&self, ss58: &str) -> Result<Balance> {
        let pk = crate::wallet::keypair::from_ss58(ss58)?;
        self.get_balance(&pk).await
    }

    /// Resolve a block number to a block hash via RPC.
    pub async fn get_block_hash(&self, block_number: u32) -> Result<subxt::utils::H256> {
        use subxt::backend::legacy::rpc_methods::NumberOrHex;
        let hash = self
            .rpc
            .chain_get_block_hash(Some(NumberOrHex::Number(block_number as u64)))
            .await
            .context("Failed to get block hash")?;
        hash.ok_or_else(|| anyhow::anyhow!("Block {} not found", block_number))
    }

    /// Wrap at-block storage errors with an archive node hint when state is pruned.
    fn annotate_at_block_error(err: anyhow::Error, block_number: Option<u32>) -> anyhow::Error {
        let msg = format!("{:#}", err);
        let is_state_pruned = msg.contains("pruned")
            || msg.contains("State already discarded")
            || msg.contains("UnknownBlock")
            || msg.contains("not found");
        if is_state_pruned {
            if let Some(bn) = block_number {
                return anyhow::anyhow!(
                    "{}\n\n  Hint: Block {} state may have been pruned from this node.\n  Use --network archive (or --endpoint <archive-url>) to query historical state.\n  Example: agcli balance --at-block {} --network archive",
                    msg, bn, bn
                );
            }
        }
        err
    }

    /// Get TAO balance at a specific block hash.
    pub async fn get_balance_at_block(
        &self,
        ss58: &str,
        block_hash: subxt::utils::H256,
    ) -> Result<Balance> {
        let pk = crate::wallet::keypair::from_ss58(ss58)?;
        let account_id = Self::to_account_id(&pk);
        let addr = api::storage().system().account(&account_id);
        let info = self
            .inner
            .storage()
            .at(block_hash)
            .fetch(&addr)
            .await
            .map_err(|e| Self::annotate_at_block_error(e.into(), None))?;
        match info {
            Some(info) => Ok(Balance::from_rao(info.data.free)),
            None => Ok(Balance::ZERO),
        }
    }

    /// Get total staked TAO at a specific block hash.
    pub async fn get_total_stake_at_block(
        &self,
        block_hash: subxt::utils::H256,
    ) -> Result<Balance> {
        let addr = api::storage().subtensor_module().total_stake();
        let val = self
            .inner
            .storage()
            .at(block_hash)
            .fetch(&addr)
            .await
            .map_err(|e| Self::annotate_at_block_error(e.into(), None))?;
        Ok(Balance::from_rao(val.unwrap_or(0)))
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
        let result = self
            .inner
            .runtime_api()
            .at_latest()
            .await?
            .call(payload)
            .await?;
        Ok(result.into_iter().map(StakeInfo::from).collect())
    }

    // ──────── Subnet Queries ────────

    /// List all subnets (via runtime API, cached for 30s).
    pub async fn get_all_subnets(&self) -> Result<Vec<SubnetInfo>> {
        let inner = &self.inner;
        let result = self
            .cache
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
            .await?;
        Ok((*result).clone())
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
    pub async fn get_all_dynamic_info(&self) -> Result<Vec<DynamicInfo>> {
        let inner = &self.inner;
        let result = self
            .cache
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
            .await?;
        Ok((*result).clone())
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
            subnet_name: String::from_utf8_lossy(&id.subnet_name).to_string(),
            github_repo: String::from_utf8_lossy(&id.github_repo).to_string(),
            subnet_contact: String::from_utf8_lossy(&id.subnet_contact).to_string(),
            subnet_url: String::from_utf8_lossy(&id.subnet_url).to_string(),
            discord: String::from_utf8_lossy(&id.discord).to_string(),
            description: String::from_utf8_lossy(&id.description).to_string(),
            additional: String::from_utf8_lossy(&id.additional).to_string(),
        }))
    }

    // ──────── Extrinsic Submission ────────
    // All extrinsics use sign_submit() to reduce boilerplate.

    /// Transfer TAO from coldkey to destination.
    pub async fn transfer(
        &self,
        pair: &sr25519::Pair,
        dest_ss58: &str,
        amount: Balance,
    ) -> Result<String> {
        let dest = subxt::utils::MultiAddress::Id(Self::ss58_to_account_id(dest_ss58)?);
        self.sign_submit(
            &api::tx()
                .balances()
                .transfer_allow_death(dest, amount.rao()),
            pair,
        )
        .await
    }

    /// Add stake to a hotkey on a subnet.
    pub async fn add_stake(
        &self,
        pair: &sr25519::Pair,
        hotkey_ss58: &str,
        netuid: NetUid,
        amount: Balance,
    ) -> Result<String> {
        let hk = Self::ss58_to_account_id(hotkey_ss58)?;
        self.sign_submit(
            &api::tx()
                .subtensor_module()
                .add_stake(hk, netuid.0, amount.rao()),
            pair,
        )
        .await
    }

    /// Remove stake from a hotkey on a subnet.
    pub async fn remove_stake(
        &self,
        pair: &sr25519::Pair,
        hotkey_ss58: &str,
        netuid: NetUid,
        amount: Balance,
    ) -> Result<String> {
        let hk = Self::ss58_to_account_id(hotkey_ss58)?;
        self.sign_submit(
            &api::tx()
                .subtensor_module()
                .remove_stake(hk, netuid.0, amount.rao()),
            pair,
        )
        .await
    }

    /// Register on a subnet via burned TAO.
    pub async fn burned_register(
        &self,
        pair: &sr25519::Pair,
        netuid: NetUid,
        hotkey_ss58: &str,
    ) -> Result<String> {
        let hk = Self::ss58_to_account_id(hotkey_ss58)?;
        self.sign_submit(
            &api::tx().subtensor_module().burned_register(netuid.0, hk),
            pair,
        )
        .await
    }

    /// Set weights on a subnet.
    pub async fn set_weights(
        &self,
        pair: &sr25519::Pair,
        netuid: NetUid,
        uids: &[u16],
        weights: &[u16],
        version_key: u64,
    ) -> Result<String> {
        let tx = api::tx().subtensor_module().set_weights(
            netuid.0,
            uids.to_vec(),
            weights.to_vec(),
            version_key,
        );
        self.sign_submit(&tx, pair).await
    }

    /// Register a new subnet.
    pub async fn register_network(
        &self,
        pair: &sr25519::Pair,
        hotkey_ss58: &str,
    ) -> Result<String> {
        let hk = Self::ss58_to_account_id(hotkey_ss58)?;
        self.sign_submit(&api::tx().subtensor_module().register_network(hk), pair)
            .await
    }

    /// Move stake between subnets (same coldkey).
    pub async fn move_stake(
        &self,
        pair: &sr25519::Pair,
        hotkey_ss58: &str,
        from: NetUid,
        to: NetUid,
        amount: Balance,
    ) -> Result<String> {
        let hk = Self::ss58_to_account_id(hotkey_ss58)?;
        self.sign_submit(
            &api::tx()
                .subtensor_module()
                .move_stake(hk.clone(), hk, from.0, to.0, amount.rao()),
            pair,
        )
        .await
    }

    /// Swap stake between subnets (same hotkey).
    pub async fn swap_stake(
        &self,
        pair: &sr25519::Pair,
        hotkey_ss58: &str,
        from: NetUid,
        to: NetUid,
        amount: Balance,
    ) -> Result<String> {
        let hk = Self::ss58_to_account_id(hotkey_ss58)?;
        self.sign_submit(
            &api::tx()
                .subtensor_module()
                .swap_stake(hk, from.0, to.0, amount.rao()),
            pair,
        )
        .await
    }

    /// Set childkey take.
    pub async fn set_childkey_take(
        &self,
        pair: &sr25519::Pair,
        hotkey_ss58: &str,
        netuid: NetUid,
        take: u16,
    ) -> Result<String> {
        let hk = Self::ss58_to_account_id(hotkey_ss58)?;
        self.sign_submit(
            &api::tx()
                .subtensor_module()
                .set_childkey_take(hk, netuid.0, take),
            pair,
        )
        .await
    }

    /// Set children for a hotkey.
    pub async fn set_children(
        &self,
        pair: &sr25519::Pair,
        hotkey_ss58: &str,
        netuid: NetUid,
        children: &[(u64, String)],
    ) -> Result<String> {
        let hk = Self::ss58_to_account_id(hotkey_ss58)?;
        let c: Vec<(u64, AccountId)> = children
            .iter()
            .map(|(p, ss58)| Ok((*p, Self::ss58_to_account_id(ss58)?)))
            .collect::<Result<_>>()?;
        self.sign_submit(
            &api::tx().subtensor_module().set_children(hk, netuid.0, c),
            pair,
        )
        .await
    }

    /// Commit weights (commit-reveal scheme).
    pub async fn commit_weights(
        &self,
        pair: &sr25519::Pair,
        netuid: NetUid,
        commit_hash: [u8; 32],
    ) -> Result<String> {
        let hash = subxt::utils::H256::from(commit_hash);
        self.sign_submit(
            &api::tx().subtensor_module().commit_weights(netuid.0, hash),
            pair,
        )
        .await
    }

    /// Reveal weights.
    pub async fn reveal_weights(
        &self,
        pair: &sr25519::Pair,
        netuid: NetUid,
        uids: &[u16],
        values: &[u16],
        salt: &[u16],
        version_key: u64,
    ) -> Result<String> {
        let tx = api::tx().subtensor_module().reveal_weights(
            netuid.0,
            uids.to_vec(),
            values.to_vec(),
            salt.to_vec(),
            version_key,
        );
        self.sign_submit(&tx, pair).await
    }

    /// Query weight commits for a hotkey on a subnet.
    /// Returns Vec<(hash, commit_block, first_reveal_block, last_reveal_block)>.
    pub async fn get_weight_commits(
        &self,
        netuid: NetUid,
        hotkey_ss58: &str,
    ) -> Result<Option<Vec<(subxt::utils::H256, u64, u64, u64)>>> {
        let hotkey_id = Self::ss58_to_account_id(hotkey_ss58)?;
        let addr = api::storage()
            .subtensor_module()
            .weight_commits(netuid.0, &hotkey_id);
        self.inner
            .storage()
            .at_latest()
            .await?
            .fetch(&addr)
            .await
            .context("Failed to fetch weight commits")
    }

    /// Iterate all weight commits on a subnet (all hotkeys).
    /// Returns Vec<(AccountId32, Vec<(hash, commit_block, first_reveal_block, last_reveal_block)>)>.
    pub async fn get_all_weight_commits(
        &self,
        netuid: NetUid,
    ) -> Result<Vec<(AccountId, Vec<(subxt::utils::H256, u64, u64, u64)>)>> {
        let addr = api::storage()
            .subtensor_module()
            .weight_commits_iter1(netuid.0);
        let mut results = Vec::new();
        let mut iter = self
            .inner
            .storage()
            .at_latest()
            .await?
            .iter(addr)
            .await?;
        while let Some(Ok(kv)) = iter.next().await {
            // Extract the account ID from the storage key (last 32 bytes)
            let key_bytes = kv.key_bytes;
            if key_bytes.len() >= 32 {
                let account_bytes: [u8; 32] = key_bytes[key_bytes.len() - 32..]
                    .try_into()
                    .unwrap_or([0u8; 32]);
                let account = AccountId::from(account_bytes);
                results.push((account, kv.value));
            }
        }
        Ok(results)
    }

    /// Get reveal period epochs for a subnet.
    pub async fn get_reveal_period_epochs(&self, netuid: NetUid) -> Result<u64> {
        let addr = api::storage()
            .subtensor_module()
            .reveal_period_epochs(netuid.0);
        let val = self.inner.storage().at_latest().await?.fetch(&addr).await?;
        Ok(val.unwrap_or(1))
    }

    /// Serve axon metadata.
    pub async fn serve_axon(
        &self,
        pair: &sr25519::Pair,
        netuid: NetUid,
        axon: &AxonInfo,
    ) -> Result<String> {
        let ip: u128 = axon.ip.parse().unwrap_or(0);
        let tx = api::tx().subtensor_module().serve_axon(
            netuid.0,
            axon.version,
            ip,
            axon.port,
            axon.ip_type,
            axon.protocol,
            0,
            0,
        );
        self.sign_submit(&tx, pair).await
    }

    /// Decrease delegate take.
    pub async fn decrease_take(
        &self,
        pair: &sr25519::Pair,
        hotkey_ss58: &str,
        take: u16,
    ) -> Result<String> {
        let hk = Self::ss58_to_account_id(hotkey_ss58)?;
        self.sign_submit(&api::tx().subtensor_module().decrease_take(hk, take), pair)
            .await
    }

    /// Increase delegate take.
    pub async fn increase_take(
        &self,
        pair: &sr25519::Pair,
        hotkey_ss58: &str,
        take: u16,
    ) -> Result<String> {
        let hk = Self::ss58_to_account_id(hotkey_ss58)?;
        self.sign_submit(&api::tx().subtensor_module().increase_take(hk, take), pair)
            .await
    }

    /// Unstake all from a hotkey.
    pub async fn unstake_all(&self, pair: &sr25519::Pair, hotkey_ss58: &str) -> Result<String> {
        let hk = Self::ss58_to_account_id(hotkey_ss58)?;
        self.sign_submit(&api::tx().subtensor_module().unstake_all(hk), pair)
            .await
    }

    /// Schedule coldkey swap.
    pub async fn schedule_swap_coldkey(
        &self,
        pair: &sr25519::Pair,
        new_coldkey_ss58: &str,
    ) -> Result<String> {
        let new_id = Self::ss58_to_account_id(new_coldkey_ss58)?;
        self.sign_submit(
            &api::tx().subtensor_module().schedule_swap_coldkey(new_id),
            pair,
        )
        .await
    }

    /// Swap hotkey.
    pub async fn swap_hotkey(
        &self,
        pair: &sr25519::Pair,
        old_ss58: &str,
        new_ss58: &str,
    ) -> Result<String> {
        let old_id = Self::ss58_to_account_id(old_ss58)?;
        let new_id = Self::ss58_to_account_id(new_ss58)?;
        self.sign_submit(
            &api::tx()
                .subtensor_module()
                .swap_hotkey(old_id, new_id, None),
            pair,
        )
        .await
    }

    /// Root register.
    pub async fn root_register(&self, pair: &sr25519::Pair, hotkey_ss58: &str) -> Result<String> {
        let hk = Self::ss58_to_account_id(hotkey_ss58)?;
        self.sign_submit(&api::tx().subtensor_module().root_register(hk), pair)
            .await
    }

    /// Dissolve a subnet.
    pub async fn dissolve_network(&self, pair: &sr25519::Pair, netuid: NetUid) -> Result<String> {
        let coldkey_id = AccountId::from(pair.public().0);
        self.sign_submit(
            &api::tx()
                .subtensor_module()
                .dissolve_network(coldkey_id, netuid.0),
            pair,
        )
        .await
    }

    /// Transfer stake to another coldkey.
    pub async fn transfer_stake(
        &self,
        pair: &sr25519::Pair,
        dest_ss58: &str,
        hotkey_ss58: &str,
        from: NetUid,
        to: NetUid,
        amount: Balance,
    ) -> Result<String> {
        let dest = Self::ss58_to_account_id(dest_ss58)?;
        let hk = Self::ss58_to_account_id(hotkey_ss58)?;
        self.sign_submit(
            &api::tx()
                .subtensor_module()
                .transfer_stake(dest, hk, from.0, to.0, amount.rao()),
            pair,
        )
        .await
    }

    /// Recycle alpha for TAO.
    pub async fn recycle_alpha(
        &self,
        pair: &sr25519::Pair,
        hotkey_ss58: &str,
        netuid: NetUid,
        amount: u64,
    ) -> Result<String> {
        let hk = Self::ss58_to_account_id(hotkey_ss58)?;
        self.sign_submit(
            &api::tx()
                .subtensor_module()
                .recycle_alpha(hk, amount, netuid.0),
            pair,
        )
        .await
    }

    /// Claim root dividends.
    pub async fn claim_root(&self, pair: &sr25519::Pair, subnets: &[u16]) -> Result<String> {
        self.sign_submit(
            &api::tx().subtensor_module().claim_root(subnets.to_vec()),
            pair,
        )
        .await
    }

    /// Add stake with limit order.
    pub async fn add_stake_limit(
        &self,
        pair: &sr25519::Pair,
        hotkey_ss58: &str,
        netuid: NetUid,
        amount: Balance,
        limit_price: u64,
        allow_partial: bool,
    ) -> Result<String> {
        let hk = Self::ss58_to_account_id(hotkey_ss58)?;
        self.sign_submit(
            &api::tx().subtensor_module().add_stake_limit(
                hk,
                netuid.0,
                amount.rao(),
                limit_price,
                allow_partial,
            ),
            pair,
        )
        .await
    }

    /// Remove stake with limit order.
    pub async fn remove_stake_limit(
        &self,
        pair: &sr25519::Pair,
        hotkey_ss58: &str,
        netuid: NetUid,
        amount: u64,
        limit_price: u64,
        allow_partial: bool,
    ) -> Result<String> {
        let hk = Self::ss58_to_account_id(hotkey_ss58)?;
        self.sign_submit(
            &api::tx().subtensor_module().remove_stake_limit(
                hk,
                netuid.0,
                amount,
                limit_price,
                allow_partial,
            ),
            pair,
        )
        .await
    }

    /// Set subnet identity (subnet owner only).
    pub async fn set_subnet_identity(
        &self,
        pair: &sr25519::Pair,
        _netuid: NetUid,
        identity: &SubnetIdentity,
    ) -> Result<String> {
        let tx = api::tx().subtensor_module().set_identity(
            identity.subnet_name.as_bytes().to_vec(),
            identity.subnet_url.as_bytes().to_vec(),
            identity.github_repo.as_bytes().to_vec(),
            Vec::new(),
            identity.discord.as_bytes().to_vec(),
            identity.description.as_bytes().to_vec(),
            identity.additional.as_bytes().to_vec(),
        );
        self.sign_submit(&tx, pair).await
    }

    /// Get total hotkey alpha on a subnet.
    pub async fn get_total_hotkey_alpha(
        &self,
        hotkey_ss58: &str,
        netuid: NetUid,
    ) -> Result<Balance> {
        let hotkey_id = Self::ss58_to_account_id(hotkey_ss58)?;
        let addr = api::storage()
            .subtensor_module()
            .total_hotkey_alpha(&hotkey_id, netuid.0);
        let val = self.inner.storage().at_latest().await?.fetch(&addr).await?;
        // Alpha storage returns u64 raw value
        Ok(Balance::from_rao(val.unwrap_or(0)))
    }

    /// POW register on a subnet.
    pub async fn pow_register(
        &self,
        signer_pair: &sr25519::Pair,
        netuid: NetUid,
        hotkey_ss58: &str,
        block_number: u64,
        nonce: u64,
        work: [u8; 32],
    ) -> Result<String> {
        let hotkey_id = Self::ss58_to_account_id(hotkey_ss58)?;
        let coldkey_id = AccountId::from(signer_pair.public().0);
        let tx = api::tx().subtensor_module().register(
            netuid.0,
            block_number,
            nonce,
            work.to_vec(),
            hotkey_id,
            coldkey_id,
        );
        self.sign_submit(&tx, signer_pair).await
    }

    /// Get the current block number and hash for POW solving.
    pub async fn get_block_info_for_pow(&self) -> Result<(u64, [u8; 32])> {
        let block = self.inner.blocks().at_latest().await?;
        Ok((block.number() as u64, block.hash().0))
    }

    /// Get registration difficulty for a subnet.
    pub async fn get_difficulty(&self, netuid: NetUid) -> Result<u64> {
        let addr = api::storage().subtensor_module().difficulty(netuid.0);
        let val = self.inner.storage().at_latest().await?.fetch(&addr).await?;
        Ok(val.unwrap_or(10_000_000))
    }

    // ──────── Block Subscription ────────

    // inner_client() removed — use subxt() instead

    // ──────── Proxy Support ────────

    /// Submit an extrinsic through a proxy account using dynamic dispatch.
    /// `real_ss58` is the proxied account. `pair` is the proxy signer.
    /// `pallet`, `call`, and `fields` describe the inner call.
    pub async fn proxy_call(
        &self,
        pair: &sr25519::Pair,
        real_ss58: &str,
        pallet: &str,
        call: &str,
        fields: Vec<subxt::dynamic::Value>,
    ) -> Result<String> {
        use subxt::dynamic::Value;
        let real_id = Self::ss58_to_account_id(real_ss58)?;
        let inner_call = Value::named_composite([
            ("pallet", Value::string(pallet)),
            ("call", Value::string(call)),
            ("fields", Value::unnamed_composite(fields)),
        ]);
        let proxy_tx = subxt::dynamic::tx(
            "Proxy",
            "proxy",
            vec![
                Value::unnamed_variant("Id", [Value::from_bytes(real_id.0)]),
                Value::unnamed_variant("None", []),
                inner_call,
            ],
        );
        self.sign_submit(&proxy_tx, pair).await
    }

    // ──────── Raw Dynamic Extrinsic Submission ────────

    /// Submit a raw SCALE-encoded call via dynamic dispatch (for pallets not in compile-time metadata).
    pub async fn submit_raw_call(
        &self,
        pair: &sr25519::Pair,
        pallet: &str,
        call: &str,
        fields: Vec<subxt::dynamic::Value>,
    ) -> Result<String> {
        let tx = subxt::dynamic::tx(pallet, call, fields);
        self.sign_submit(&tx, pair).await
    }

    /// Sign and submit any Payload (public, for batch/dynamic calls from outside the chain module).
    pub async fn sign_submit_dyn<T: subxt::tx::Payload>(
        &self,
        tx: &T,
        pair: &sr25519::Pair,
    ) -> Result<String> {
        self.sign_submit(tx, pair).await
    }

    // ──────── Multisig ────────

    /// Submit a multisig call (Multisig::as_multi) wrapping a dynamic inner call.
    /// Uses dynamic dispatch so no compile-time type bindings needed.
    pub async fn submit_multisig_call(
        &self,
        pair: &sr25519::Pair,
        other_signatories: &[AccountId],
        threshold: u16,
        pallet: &str,
        call: &str,
        fields: Vec<subxt::dynamic::Value>,
    ) -> Result<String> {
        // Build the inner call, encode it, and use as_multi_threshold_1 for 1-of-N
        // or approve_as_multi for N-of-M (first step is to propose)
        let inner = subxt::dynamic::tx(pallet, call, fields);
        // For as_multi, we need the inner call as encoded bytes
        let encoded = self.inner.tx().call_data(&inner)?;
        let call_hash = sp_core::hashing::blake2_256(&encoded);

        // Use approve_as_multi (which just records the hash) as the first step
        self.approve_multisig(pair, other_signatories, threshold, call_hash)
            .await
    }

    /// Approve a pending multisig call (Multisig::approve_as_multi).
    pub async fn approve_multisig(
        &self,
        pair: &sr25519::Pair,
        other_signatories: &[AccountId],
        threshold: u16,
        call_hash: [u8; 32],
    ) -> Result<String> {
        use subxt::dynamic::Value;
        let others: Vec<Value> = other_signatories
            .iter()
            .map(|id| Value::from_bytes(id.0))
            .collect();
        let tx = subxt::dynamic::tx(
            "Multisig",
            "approve_as_multi",
            vec![
                Value::u128(threshold as u128),
                Value::unnamed_composite(others),
                Value::unnamed_variant("None", []), // maybe_timepoint
                Value::from_bytes(call_hash),       // call_hash
                Value::named_composite([
                    ("ref_time", Value::u128(0)),
                    ("proof_size", Value::u128(0)),
                ]),
            ],
        );
        self.sign_submit(&tx, pair).await
    }

    // ──────── Transfer All ────────

    /// Transfer entire free balance to destination (minus fees).
    pub async fn transfer_all(
        &self,
        pair: &sr25519::Pair,
        dest_ss58: &str,
        keep_alive: bool,
    ) -> Result<String> {
        let dest = subxt::utils::MultiAddress::Id(Self::ss58_to_account_id(dest_ss58)?);
        self.sign_submit(&api::tx().balances().transfer_all(dest, keep_alive), pair)
            .await
    }

    // ──────── Proxy Management ────────

    /// Add a proxy account.
    pub async fn add_proxy(
        &self,
        pair: &sr25519::Pair,
        delegate_ss58: &str,
        proxy_type: &str,
        delay: u32,
    ) -> Result<String> {
        self.proxy_op("add_proxy", pair, delegate_ss58, proxy_type, delay)
            .await
    }

    /// Remove a proxy account.
    pub async fn remove_proxy(
        &self,
        pair: &sr25519::Pair,
        delegate_ss58: &str,
        proxy_type: &str,
        delay: u32,
    ) -> Result<String> {
        self.proxy_op("remove_proxy", pair, delegate_ss58, proxy_type, delay)
            .await
    }

    async fn proxy_op(
        &self,
        call: &str,
        pair: &sr25519::Pair,
        delegate_ss58: &str,
        proxy_type: &str,
        delay: u32,
    ) -> Result<String> {
        use subxt::dynamic::Value;
        let delegate = Self::ss58_to_account_id(delegate_ss58)?;
        let variant = parse_proxy_type(proxy_type);
        let tx = subxt::dynamic::tx(
            "Proxy",
            call,
            vec![
                Value::unnamed_variant("Id", [Value::from_bytes(delegate.0)]),
                Value::unnamed_variant(variant, []),
                Value::u128(delay as u128),
            ],
        );
        self.sign_submit(&tx, pair).await
    }

    // ──────── Unstake All Alpha ────────

    /// Unstake all alpha across all subnets for a hotkey.
    pub async fn unstake_all_alpha(
        &self,
        pair: &sr25519::Pair,
        hotkey_ss58: &str,
    ) -> Result<String> {
        let hk = Self::ss58_to_account_id(hotkey_ss58)?;
        self.sign_submit(&api::tx().subtensor_module().unstake_all_alpha(hk), pair)
            .await
    }

    /// Burn alpha tokens (permanently remove from supply).
    pub async fn burn_alpha(
        &self,
        pair: &sr25519::Pair,
        hotkey_ss58: &str,
        amount: u64,
        netuid: NetUid,
    ) -> Result<String> {
        let hk = Self::ss58_to_account_id(hotkey_ss58)?;
        self.sign_submit(
            &api::tx()
                .subtensor_module()
                .burn_alpha(hk, amount, netuid.0),
            pair,
        )
        .await
    }

    /// Swap stake between subnets with a limit price.
    #[allow(clippy::too_many_arguments)]
    pub async fn swap_stake_limit(
        &self,
        pair: &sr25519::Pair,
        hotkey_ss58: &str,
        from: NetUid,
        to: NetUid,
        amount: u64,
        limit_price: u64,
        allow_partial: bool,
    ) -> Result<String> {
        let hk = Self::ss58_to_account_id(hotkey_ss58)?;
        self.sign_submit(
            &api::tx().subtensor_module().swap_stake_limit(
                hk,
                from.0,
                to.0,
                amount,
                limit_price,
                allow_partial,
            ),
            pair,
        )
        .await
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

    // ──────── Crowdloan ────────

    /// Contribute to a crowdloan.
    pub async fn crowdloan_contribute(
        &self,
        pair: &sr25519::Pair,
        crowdloan_id: u32,
        amount: Balance,
    ) -> Result<String> {
        self.sign_submit(
            &api::tx().crowdloan().contribute(crowdloan_id, amount.rao()),
            pair,
        )
        .await
    }

    /// Withdraw from a crowdloan.
    pub async fn crowdloan_withdraw(
        &self,
        pair: &sr25519::Pair,
        crowdloan_id: u32,
    ) -> Result<String> {
        self.sign_submit(&api::tx().crowdloan().withdraw(crowdloan_id), pair)
            .await
    }

    /// Finalize a crowdloan.
    pub async fn crowdloan_finalize(
        &self,
        pair: &sr25519::Pair,
        crowdloan_id: u32,
    ) -> Result<String> {
        self.sign_submit(&api::tx().crowdloan().finalize(crowdloan_id), pair)
            .await
    }

    // ──────── Batch Extrinsics ────────

    /// Batch set weights across multiple subnets.
    pub async fn batch_set_weights(
        &self,
        pair: &sr25519::Pair,
        netuids: &[u16],
        weights: &[Vec<(u16, u16)>],
        version_keys: &[u64],
    ) -> Result<String> {
        use parity_scale_codec::Compact;
        let n: Vec<Compact<u16>> = netuids.iter().map(|n| Compact(*n)).collect();
        let w: Vec<Vec<(Compact<u16>, Compact<u16>)>> = weights
            .iter()
            .map(|w| w.iter().map(|(u, v)| (Compact(*u), Compact(*v))).collect())
            .collect();
        let v: Vec<Compact<u64>> = version_keys.iter().map(|k| Compact(*k)).collect();
        self.sign_submit(
            &api::tx().subtensor_module().batch_set_weights(n, w, v),
            pair,
        )
        .await
    }

    /// Batch commit weights across multiple subnets.
    pub async fn batch_commit_weights(
        &self,
        pair: &sr25519::Pair,
        netuids: &[u16],
        commit_hashes: &[[u8; 32]],
    ) -> Result<String> {
        use parity_scale_codec::Compact;
        let n: Vec<Compact<u16>> = netuids.iter().map(|n| Compact(*n)).collect();
        let h: Vec<subxt::utils::H256> = commit_hashes
            .iter()
            .map(|h| subxt::utils::H256::from(*h))
            .collect();
        self.sign_submit(
            &api::tx().subtensor_module().batch_commit_weights(n, h),
            pair,
        )
        .await
    }

    /// Batch reveal weights for a subnet.
    pub async fn batch_reveal_weights(
        &self,
        pair: &sr25519::Pair,
        netuid: NetUid,
        uids_list: &[Vec<u16>],
        values_list: &[Vec<u16>],
        salts_list: &[Vec<u16>],
        version_keys: &[u64],
    ) -> Result<String> {
        let tx = api::tx().subtensor_module().batch_reveal_weights(
            netuid.0,
            uids_list.to_vec(),
            values_list.to_vec(),
            salts_list.to_vec(),
            version_keys.to_vec(),
        );
        self.sign_submit(&tx, pair).await
    }

    // ──────── Auto-Stake ────────

    /// Set the auto-stake hotkey for a subnet (rewards auto-compound to this hotkey).
    pub async fn set_auto_stake(
        &self,
        pair: &sr25519::Pair,
        netuid: NetUid,
        hotkey_ss58: &str,
    ) -> Result<String> {
        use subxt::dynamic::Value;
        let hotkey_id = Self::ss58_to_account_id(hotkey_ss58)?;
        self.submit_raw_call(
            pair,
            "SubtensorModule",
            "set_coldkey_auto_stake_hotkey",
            vec![
                Value::u128(netuid.0 as u128),
                Value::from_bytes(hotkey_id.0),
            ],
        )
        .await
    }

    // ──────── Root Claim Management ────────

    /// Set root claim type: "Swap" (swap alpha→TAO), "Keep" (keep alpha), or KeepSubnets.
    pub async fn set_root_claim_type(
        &self,
        pair: &sr25519::Pair,
        claim_type: &str,
        keep_subnets: Option<&[u16]>,
    ) -> Result<String> {
        use subxt::dynamic::Value;
        let variant = match claim_type {
            "swap" | "Swap" => Value::unnamed_variant("Swap", []),
            "keep" | "Keep" => Value::unnamed_variant("Keep", []),
            "keep-subnets" | "KeepSubnets" => {
                let subnets = keep_subnets.unwrap_or(&[]);
                let subnet_vals: Vec<Value> =
                    subnets.iter().map(|n| Value::u128(*n as u128)).collect();
                Value::named_variant(
                    "KeepSubnets",
                    [("subnets", Value::unnamed_composite(subnet_vals))],
                )
            }
            _ => anyhow::bail!("Invalid claim type: {}. Use 'swap', 'keep', or 'keep-subnets'", claim_type),
        };
        self.submit_raw_call(
            pair,
            "SubtensorModule",
            "set_root_claim_type",
            vec![variant],
        )
        .await
    }

    // ──────── Liquidity Pool (Swap pallet) ────────

    /// Add liquidity to a subnet's AMM pool with a concentrated price range.
    pub async fn add_liquidity(
        &self,
        pair: &sr25519::Pair,
        hotkey_ss58: &str,
        netuid: NetUid,
        tick_low: i32,
        tick_high: i32,
        liquidity: u64,
    ) -> Result<String> {
        use subxt::dynamic::Value;
        let hotkey_id = Self::ss58_to_account_id(hotkey_ss58)?;
        self.submit_raw_call(
            pair,
            "Swap",
            "add_liquidity",
            vec![
                Value::from_bytes(hotkey_id.0),
                Value::u128(netuid.0 as u128),
                Value::i128(tick_low as i128),
                Value::i128(tick_high as i128),
                Value::u128(liquidity as u128),
            ],
        )
        .await
    }

    /// Remove a liquidity position entirely.
    pub async fn remove_liquidity(
        &self,
        pair: &sr25519::Pair,
        hotkey_ss58: &str,
        netuid: NetUid,
        position_id: u128,
    ) -> Result<String> {
        use subxt::dynamic::Value;
        let hotkey_id = Self::ss58_to_account_id(hotkey_ss58)?;
        self.submit_raw_call(
            pair,
            "Swap",
            "remove_liquidity",
            vec![
                Value::from_bytes(hotkey_id.0),
                Value::u128(netuid.0 as u128),
                Value::u128(position_id),
            ],
        )
        .await
    }

    /// Modify liquidity in an existing position (positive = add, negative = remove).
    pub async fn modify_liquidity(
        &self,
        pair: &sr25519::Pair,
        hotkey_ss58: &str,
        netuid: NetUid,
        position_id: u128,
        liquidity_delta: i64,
    ) -> Result<String> {
        use subxt::dynamic::Value;
        let hotkey_id = Self::ss58_to_account_id(hotkey_ss58)?;
        self.submit_raw_call(
            pair,
            "Swap",
            "modify_position",
            vec![
                Value::from_bytes(hotkey_id.0),
                Value::u128(netuid.0 as u128),
                Value::u128(position_id),
                Value::i128(liquidity_delta as i128),
            ],
        )
        .await
    }

    // ──────── MEV Shield ────────

    /// Fetch the current ML-KEM-768 public key for MEV shield from chain storage.
    pub async fn get_mev_shield_next_key(&self) -> Result<Vec<u8>> {
        let storage_query = subxt::dynamic::storage("MevShield", "NextKey", ());
        let result = self.inner.storage().at_latest().await?.fetch(&storage_query).await?;
        match result {
            Some(val) => {
                // Decode the BoundedVec<u8> from the storage value
                let bytes: Vec<u8> = val.as_type()?;
                Ok(bytes)
            }
            None => anyhow::bail!("MEV shield key not available — the chain may not have MEV shield enabled"),
        }
    }

    /// Submit an encrypted extrinsic through the MEV shield.
    /// Takes a pre-computed commitment (Blake2-256) and ciphertext.
    pub async fn submit_mev_encrypted(
        &self,
        pair: &sr25519::Pair,
        commitment: [u8; 32],
        ciphertext: Vec<u8>,
    ) -> Result<String> {
        use subxt::dynamic::Value;
        self.submit_raw_call(
            pair,
            "MevShield",
            "submit_encrypted",
            vec![
                Value::from_bytes(commitment),
                Value::from_bytes(ciphertext),
            ],
        )
        .await
    }

    /// Toggle user liquidity on a subnet (subnet owner only).
    pub async fn toggle_user_liquidity(
        &self,
        pair: &sr25519::Pair,
        netuid: NetUid,
        enable: bool,
    ) -> Result<String> {
        use subxt::dynamic::Value;
        self.submit_raw_call(
            pair,
            "Swap",
            "toggle_user_liquidity",
            vec![
                Value::u128(netuid.0 as u128),
                Value::bool(enable),
            ],
        )
        .await
    }

    // ──────── Crowdloan (full) ────────

    /// Create a new crowdloan campaign.
    pub async fn crowdloan_create(
        &self,
        pair: &sr25519::Pair,
        deposit_rao: u64,
        min_contribution_rao: u64,
        cap_rao: u64,
        end_block: u32,
        target_ss58: Option<&str>,
    ) -> Result<String> {
        use subxt::dynamic::Value;
        let target = match target_ss58 {
            Some(ss58) => {
                let id = Self::ss58_to_account_id(ss58)?;
                Value::unnamed_variant("Some", [Value::from_bytes(id.0)])
            }
            None => Value::unnamed_variant("None", []),
        };
        self.submit_raw_call(
            pair,
            "Crowdloan",
            "create",
            vec![
                Value::u128(deposit_rao as u128),
                Value::u128(min_contribution_rao as u128),
                Value::u128(cap_rao as u128),
                Value::u128(end_block as u128),
                Value::unnamed_variant("None", []), // call (None for simple fund)
                target,
            ],
        )
        .await
    }

    /// Refund all contributors of a failed/expired crowdloan.
    pub async fn crowdloan_refund(
        &self,
        pair: &sr25519::Pair,
        crowdloan_id: u32,
    ) -> Result<String> {
        use subxt::dynamic::Value;
        self.submit_raw_call(
            pair,
            "Crowdloan",
            "refund",
            vec![Value::u128(crowdloan_id as u128)],
        )
        .await
    }

    /// Dissolve a crowdloan (creator only, after refunding).
    pub async fn crowdloan_dissolve(
        &self,
        pair: &sr25519::Pair,
        crowdloan_id: u32,
    ) -> Result<String> {
        use subxt::dynamic::Value;
        self.submit_raw_call(
            pair,
            "Crowdloan",
            "dissolve",
            vec![Value::u128(crowdloan_id as u128)],
        )
        .await
    }

    /// Update cap of a crowdloan (creator only).
    pub async fn crowdloan_update_cap(
        &self,
        pair: &sr25519::Pair,
        crowdloan_id: u32,
        new_cap_rao: u64,
    ) -> Result<String> {
        use subxt::dynamic::Value;
        self.submit_raw_call(
            pair,
            "Crowdloan",
            "update_cap",
            vec![
                Value::u128(crowdloan_id as u128),
                Value::u128(new_cap_rao as u128),
            ],
        )
        .await
    }

    /// Update end block of a crowdloan (creator only).
    pub async fn crowdloan_update_end(
        &self,
        pair: &sr25519::Pair,
        crowdloan_id: u32,
        new_end: u32,
    ) -> Result<String> {
        use subxt::dynamic::Value;
        self.submit_raw_call(
            pair,
            "Crowdloan",
            "update_end",
            vec![
                Value::u128(crowdloan_id as u128),
                Value::u128(new_end as u128),
            ],
        )
        .await
    }

    /// Update minimum contribution of a crowdloan (creator only).
    pub async fn crowdloan_update_min_contribution(
        &self,
        pair: &sr25519::Pair,
        crowdloan_id: u32,
        new_min_rao: u64,
    ) -> Result<String> {
        use subxt::dynamic::Value;
        self.submit_raw_call(
            pair,
            "Crowdloan",
            "update_min_contribution",
            vec![
                Value::u128(crowdloan_id as u128),
                Value::u128(new_min_rao as u128),
            ],
        )
        .await
    }
}

/// Parse a proxy type string to the on-chain variant name.
fn parse_proxy_type(s: &str) -> &'static str {
    match s {
        "any" | "Any" => "Any",
        "owner" | "Owner" => "Owner",
        "non_transfer" | "NonTransfer" => "NonTransfer",
        "staking" | "Staking" => "Staking",
        "non_critical" | "NonCritical" => "NonCritical",
        "triumvirate" | "Triumvirate" => "Triumvirate",
        "governance" | "Governance" => "Governance",
        "senate" | "Senate" => "Senate",
        _ => "Any",
    }
}

/// Format submission errors (before tx reaches chain) with actionable hints.
fn format_submit_error(e: subxt::Error) -> anyhow::Error {
    let msg = e.to_string();
    if msg.contains("connection") || msg.contains("Connection") || msg.contains("Ws") {
        anyhow::anyhow!("Connection lost while submitting transaction. Check your network and endpoint.\n  Error: {}", msg)
    } else if msg.contains("Priority is too low") || msg.contains("priority") {
        anyhow::anyhow!("Transaction rejected: a conflicting transaction is already pending. Wait for it to finalize or use a different nonce.\n  Error: {}", msg)
    } else if msg.contains("Inability to pay") || msg.contains("InsufficientBalance") {
        anyhow::anyhow!("Insufficient balance to pay transaction fees. Check your free TAO balance with `agcli balance`.\n  Error: {}", msg)
    } else {
        anyhow::anyhow!("Transaction submission failed: {}", msg)
    }
}

/// Format dispatch errors (tx reached chain but execution failed) with contextual hints.
fn format_dispatch_error(e: subxt::Error) -> anyhow::Error {
    let msg = e.to_string();
    // Map common SubtensorModule errors to helpful messages
    let hint = if msg.contains("NotEnoughBalanceToStake") || msg.contains("NotEnoughStake") {
        "Insufficient balance or stake for this operation. Check `agcli balance` and `agcli stake list`."
    } else if msg.contains("NotRegistered") || msg.contains("HotKeyNotRegisteredInSubNet") {
        "Hotkey is not registered on this subnet. Register first with `agcli subnet register-neuron`."
    } else if msg.contains("NotEnoughBalance") || msg.contains("InsufficientBalance") {
        "Insufficient TAO balance. Check your balance with `agcli balance`."
    } else if msg.contains("AlreadyRegistered") {
        "This hotkey is already registered on the subnet."
    } else if msg.contains("TooManyRegistrationsThisBlock") {
        "Registration limit reached for this block. Try again in the next block (~12 seconds)."
    } else if msg.contains("InvalidNetuid") || msg.contains("NetworkDoesNotExist") {
        "Invalid subnet ID. List available subnets with `agcli subnet list`."
    } else if msg.contains("BadOrigin") || msg.contains("NotOwner") {
        "Permission denied — you are not the owner of this subnet or account."
    } else if msg.contains("WeightsNotSettable") || msg.contains("SettingWeightsTooFast") {
        "Cannot set weights: either rate-limited or commit-reveal is required. Check subnet hyperparams."
    } else if msg.contains("TxRateLimitExceeded") {
        "Rate limit exceeded. Wait before retrying this operation."
    } else if msg.contains("StakeRateLimitExceeded") {
        "Staking rate limit exceeded. Wait before staking/unstaking again."
    } else if msg.contains("InvalidTake") || msg.contains("DelegateTakeTooHigh") {
        "Invalid delegate take percentage. Take must be between 0% and 11.11%."
    } else if msg.contains("NonAssociatedColdKey") {
        "This coldkey is not associated with the specified hotkey."
    } else if msg.contains("CommitRevealEnabled") {
        "This subnet requires commit-reveal for weights. Use `agcli weights commit` then `agcli weights reveal`."
    } else {
        "" // no special hint
    };

    if hint.is_empty() {
        anyhow::anyhow!("Transaction failed on chain: {}", msg)
    } else {
        anyhow::anyhow!("Transaction failed: {}\n  Hint: {}", msg, hint)
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
                $(Data::$variant(b) => String::from_utf8_lossy(b).to_string(),)+
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
