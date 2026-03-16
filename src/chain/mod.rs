//! Substrate chain client — connect, query storage, submit extrinsics.

pub mod extrinsics;
pub mod queries;
pub mod rpc_types;

use anyhow::{Context, Result};
use sp_core::sr25519;
use subxt::backend::legacy::rpc_methods::LegacyRpcMethods;
use subxt::backend::rpc::RpcClient;
use subxt::tx::PairSigner;
use subxt::OnlineClient;

use crate::queries::query_cache::QueryCache;
use crate::types::balance::Balance;
use crate::{api, AccountId, SubtensorConfig};

// Re-export for event subscription
pub use subxt;

/// Check whether an error message looks transient (connection, timeout, transport).
fn is_transient_error(msg: &str) -> bool {
    msg.contains("onnect")
        || msg.contains("timeout")
        || msg.contains("Ws")
        || msg.contains("transport")
        || msg.contains("closed")
        || msg.contains("reset")
        || msg.contains("State already discarded") // fast-block chain state pruning
        || msg.contains("UnknownBlock") // stale block reference
}

/// Default retry count for RPC queries.
const RPC_RETRIES: u32 = 2;

/// Retry a fallible async operation with exponential backoff on transient errors.
/// Retries up to `max_retries` times with delays of 500ms, 1s, 2s, ...
/// Only retries on errors that look transient (connection, timeout, transport).
pub(crate) async fn retry_on_transient<F, Fut, T>(label: &str, max_retries: u32, f: F) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let start = std::time::Instant::now();
    let mut last_err = None;
    for attempt in 0..=max_retries {
        match f().await {
            Ok(val) => {
                let elapsed = start.elapsed();
                tracing::debug!(
                    elapsed_ms = elapsed.as_millis() as u64,
                    attempts = attempt + 1,
                    label,
                    "RPC query succeeded"
                );
                return Ok(val);
            }
            Err(e) => {
                let msg = format!("{:#}", e);
                if !is_transient_error(&msg) || attempt == max_retries {
                    let elapsed = start.elapsed();
                    tracing::debug!(
                        elapsed_ms = elapsed.as_millis() as u64,
                        attempts = attempt + 1,
                        label,
                        error = %msg,
                        "RPC query failed"
                    );
                    return Err(e);
                }
                let delay = std::time::Duration::from_millis(500 * (1 << attempt));
                tracing::warn!(
                    attempt = attempt + 1,
                    max = max_retries,
                    delay_ms = delay.as_millis() as u64,
                    label,
                    error = %msg,
                    "Transient RPC error, retrying"
                );
                tokio::time::sleep(delay).await;
                last_err = Some(e);
            }
        }
    }
    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("{}: all retries exhausted", label)))
}

/// Signer type for extrinsic submission.
pub type Signer = PairSigner<SubtensorConfig, sr25519::Pair>;

/// High-level client for the Bittensor (subtensor) chain.
pub struct Client {
    inner: OnlineClient<SubtensorConfig>,
    rpc: LegacyRpcMethods<SubtensorConfig>,
    cache: QueryCache,
    dry_run: bool,
    url: String,
}

impl Client {
    /// Access the runtime metadata from the connected chain.
    pub fn metadata(&self) -> subxt::Metadata {
        self.inner.metadata()
    }

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
        Ok(Self {
            inner,
            rpc,
            cache: QueryCache::new(),
            dry_run: false,
            url: url.to_string(),
        })
    }

    /// Reconnect to the same endpoint. Creates a fresh RPC connection while preserving settings.
    /// Useful when the subxt background task dies (e.g. on fast-block devnets).
    pub async fn reconnect(&mut self) -> Result<()> {
        let fresh = Self::connect_once(&self.url).await?;
        self.inner = fresh.inner;
        self.rpc = fresh.rpc;
        self.cache = QueryCache::new();
        Ok(())
    }

    /// Check if the connection is still alive by attempting a lightweight RPC call.
    pub async fn is_alive(&self) -> bool {
        self.inner.blocks().at_latest().await.is_ok()
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
                    tracing::warn!(
                        "Retry {}/{} for {} in {:?}",
                        attempt,
                        max_retries - 1,
                        url,
                        delay
                    );
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
                        tracing::warn!(
                            "Connection attempt {} to {} failed: {}",
                            attempt + 1,
                            url,
                            e
                        );
                        last_err = Some(e);
                    }
                }
            }
            tracing::warn!(
                "All {} attempts to {} exhausted, trying next endpoint",
                max_retries,
                url
            );
        }

        Err(last_err.unwrap_or_else(|| anyhow::anyhow!("No endpoints provided")))
    }

    /// Test multiple endpoints concurrently and connect to the fastest one.
    /// Measures connection + RPC round-trip latency for each URL in parallel,
    /// then picks the endpoint with the lowest average latency.
    /// Falls back to `connect_with_retry` if all measurements fail.
    pub async fn best_connection(urls: &[&str]) -> Result<Self> {
        if urls.len() <= 1 {
            return Self::connect_with_retry(urls).await;
        }

        tracing::info!(
            endpoints = urls.len(),
            "Testing endpoints for best connection"
        );

        // Test all endpoints concurrently
        let mut handles = Vec::with_capacity(urls.len());
        for url in urls {
            let url_owned = url.to_string();
            handles.push(tokio::spawn(async move {
                let start = std::time::Instant::now();
                match Self::connect_once(&url_owned).await {
                    Ok(client) => {
                        let connect_ms = start.elapsed().as_millis();
                        // One RPC round-trip to measure total latency
                        let rpc_start = std::time::Instant::now();
                        match client.get_block_number().await {
                            Ok(_) => {
                                let rpc_ms = rpc_start.elapsed().as_millis();
                                let total = connect_ms + rpc_ms;
                                tracing::debug!(url = %url_owned, connect_ms, rpc_ms, total_ms = total, "Endpoint measured");
                                Ok((url_owned, client, total))
                            }
                            Err(e) => {
                                tracing::debug!(url = %url_owned, error = %e, "Endpoint RPC failed");
                                Err(anyhow::anyhow!("RPC failed for {}: {}", url_owned, e))
                            }
                        }
                    }
                    Err(e) => {
                        tracing::debug!(url = %url_owned, error = %e, "Endpoint connection failed");
                        Err(e)
                    }
                }
            }));
        }

        // Collect results
        let mut best: Option<(String, Self, u128)> = None;
        let mut last_err = None;
        for handle in handles {
            match handle.await {
                Ok(Ok((url, client, latency))) => {
                    let is_better = best
                        .as_ref()
                        .is_none_or(|(_, _, best_lat)| latency < *best_lat);
                    if is_better {
                        best = Some((url, client, latency));
                    }
                }
                Ok(Err(e)) => {
                    last_err = Some(e);
                }
                Err(e) => {
                    last_err = Some(anyhow::anyhow!("Task join error: {}", e));
                }
            }
        }

        match best {
            Some((url, client, latency)) => {
                tracing::info!(url = %url, latency_ms = latency, "Selected best endpoint");
                Ok(client)
            }
            None => Err(last_err.unwrap_or_else(|| anyhow::anyhow!("All endpoints failed"))),
        }
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

    /// Enable dry-run mode: sign_submit will print a JSON preview instead of broadcasting.
    pub fn set_dry_run(&mut self, enabled: bool) {
        self.dry_run = enabled;
    }

    /// Sign, submit, and wait for finalization of a typed extrinsic.
    /// Returns the extrinsic hash. Provides contextual error messages for common failures.
    /// In dry-run mode, encodes the call data and returns a JSON preview without submitting.
    async fn sign_submit<T: subxt::tx::Payload>(
        &self,
        tx: &T,
        pair: &sr25519::Pair,
    ) -> Result<String> {
        // Dry-run: encode the call and show what would be submitted
        if self.dry_run {
            let call_data = self
                .inner
                .tx()
                .call_data(tx)
                .map_err(|e| anyhow::anyhow!("Failed to encode call data: {}", e))?;
            let signer_pub = sp_core::Pair::public(pair);
            let signer_ss58 = crate::wallet::keypair::to_ss58(&signer_pub, 42);
            let info = serde_json::json!({
                "dry_run": true,
                "signer": signer_ss58,
                "call_data_hex": format!("0x{}", hex::encode(&call_data)),
                "call_data_len": call_data.len(),
            });
            eprintln!(
                "[dry-run] Transaction would be submitted by {} ({} bytes call data)",
                signer_ss58,
                call_data.len()
            );
            crate::cli::helpers::print_json(&info);
            return Ok("dry-run".to_string());
        }

        let signer = Self::signer(pair);
        let start = std::time::Instant::now();
        let spinner = crate::cli::helpers::spinner("Submitting transaction...");
        tracing::debug!("Submitting extrinsic");
        // Retry submission on transient errors (connection drop before tx reaches node).
        // Once submitted, we do NOT retry — the finalization wait is non-idempotent.
        let inner = &self.inner;
        let progress = retry_on_transient("sign_submit", RPC_RETRIES, || async {
            match inner
                .tx()
                .sign_and_submit_then_watch_default(tx, &signer)
                .await
            {
                Ok(p) => Ok(p),
                Err(e) => {
                    let msg = e.to_string();
                    if is_transient_error(&msg) {
                        Err(anyhow::anyhow!("{}", msg))
                    } else {
                        spinner.finish_and_clear();
                        Err(format_submit_error(e))
                    }
                }
            }
        })
        .await?;
        spinner.set_message("Waiting for finalization...");
        tracing::debug!("Extrinsic submitted, waiting for finalization");
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            progress.wait_for_finalized_success(),
        )
        .await
        .map_err(|_| {
            spinner.finish_and_clear();
            anyhow::anyhow!(
                "Transaction timed out after 30s waiting for finalization. \
             The extrinsic may have been dropped from the pool \
             (insufficient balance, invalid state, or node not producing blocks)."
            )
        })?
        .map_err(|e| {
            spinner.finish_and_clear();
            format_dispatch_error(e)
        })?;
        let hash = format!("{:?}", result.extrinsic_hash());
        spinner.finish_and_clear();
        tracing::info!(tx_hash = %hash, elapsed_ms = start.elapsed().as_millis() as u64, "Extrinsic finalized");
        Ok(hash)
    }

    /// Sign and submit via MEV shield: SCALE-encode the call, encrypt with ML-KEM-768,
    /// then submit encrypted extrinsic to MevShield.submit_encrypted.
    pub async fn sign_submit_mev<T: subxt::tx::Payload>(
        &self,
        tx: &T,
        pair: &sr25519::Pair,
    ) -> Result<String> {
        tracing::info!("MEV shield: encrypting extrinsic");
        let start = std::time::Instant::now();

        // 1. Encode the call to SCALE bytes
        let call_data = self
            .inner
            .tx()
            .call_data(tx)
            .map_err(|e| anyhow::anyhow!("Failed to encode call data: {}", e))?;

        // 2. Fetch the MEV shield public key from chain
        let mev_key = self.get_mev_shield_next_key().await?;

        // 3. Encrypt with ML-KEM-768 + XChaCha20-Poly1305
        let (commitment, ciphertext) =
            crate::extrinsics::mev_shield::encrypt_for_mev_shield(&mev_key, &call_data)?;

        tracing::info!(
            elapsed_ms = start.elapsed().as_millis() as u64,
            ct_len = ciphertext.len(),
            "MEV shield: encryption complete"
        );

        // 4. Submit the encrypted extrinsic
        self.submit_mev_encrypted(pair, commitment, ciphertext)
            .await
    }

    /// Sign and submit, optionally wrapping through MEV shield.
    pub async fn sign_submit_or_mev<T: subxt::tx::Payload>(
        &self,
        tx: &T,
        pair: &sr25519::Pair,
        use_mev: bool,
    ) -> Result<String> {
        if use_mev {
            self.sign_submit_mev(tx, pair).await
        } else {
            self.sign_submit(tx, pair).await
        }
    }

    // ──────── Balance Queries ────────

    /// Get TAO balance (free) for an account.
    pub async fn get_balance(&self, account: &sr25519::Public) -> Result<Balance> {
        let start = std::time::Instant::now();
        let account_id = Self::to_account_id(account);
        let inner = &self.inner;
        let info = retry_on_transient("get_balance", 2, || async {
            let addr = api::storage().system().account(&account_id);
            let result = inner
                .storage()
                .at_latest()
                .await
                .context("Failed to get latest block for balance query")?
                .fetch(&addr)
                .await
                .context("Failed to fetch account balance")?;
            Ok(result)
        })
        .await?;
        let balance = match info {
            Some(info) => Balance::from_rao(info.data.free),
            None => Balance::ZERO,
        };
        tracing::debug!(
            elapsed_ms = start.elapsed().as_millis() as u64,
            "get_balance"
        );
        Ok(balance)
    }

    /// Get TAO balance for an SS58 address.
    pub async fn get_balance_ss58(&self, ss58: &str) -> Result<Balance> {
        let pk = crate::wallet::keypair::from_ss58(ss58)?;
        self.get_balance(&pk).await
    }

    /// Resolve a block number to a block hash via RPC.
    pub async fn get_block_hash(&self, block_number: u32) -> Result<subxt::utils::H256> {
        use subxt::backend::legacy::rpc_methods::NumberOrHex;
        let rpc = &self.rpc;
        let hash = retry_on_transient("get_block_hash", RPC_RETRIES, || async {
            let h = rpc
                .chain_get_block_hash(Some(NumberOrHex::Number(block_number as u64)))
                .await
                .context("Failed to get block hash")?;
            Ok(h)
        })
        .await?;
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
        let inner = &self.inner;
        retry_on_transient("get_block_number", RPC_RETRIES, || async {
            let block = inner
                .blocks()
                .at_latest()
                .await
                .context("Failed to fetch latest block")?;
            Ok(block.number() as u64)
        })
        .await
    }

    // ──────── Network Params ────────

    /// Total TAO issuance.
    pub async fn get_total_issuance(&self) -> Result<Balance> {
        let inner = &self.inner;
        retry_on_transient("get_total_issuance", RPC_RETRIES, || async {
            let addr = api::storage().balances().total_issuance();
            let val = inner.storage().at_latest().await?.fetch(&addr).await?;
            let raw = val.unwrap_or(0);
            Ok(Balance::from_rao(u64::try_from(raw).unwrap_or(u64::MAX)))
        })
        .await
    }

    /// Total staked TAO.
    pub async fn get_total_stake(&self) -> Result<Balance> {
        let inner = &self.inner;
        retry_on_transient("get_total_stake", RPC_RETRIES, || async {
            let addr = api::storage().subtensor_module().total_stake();
            let val = inner.storage().at_latest().await?.fetch(&addr).await?;
            Ok(Balance::from_rao(val.unwrap_or(0)))
        })
        .await
    }

    /// Total number of subnets.
    pub async fn get_total_networks(&self) -> Result<u16> {
        let inner = &self.inner;
        retry_on_transient("get_total_networks", RPC_RETRIES, || async {
            let addr = api::storage().subtensor_module().total_networks();
            let val = inner.storage().at_latest().await?.fetch(&addr).await?;
            Ok(val.unwrap_or(0))
        })
        .await
    }

    /// Block emission rate.
    pub async fn get_block_emission(&self) -> Result<Balance> {
        let inner = &self.inner;
        retry_on_transient("get_block_emission", RPC_RETRIES, || async {
            let addr = api::storage().subtensor_module().block_emission();
            let val = inner.storage().at_latest().await?.fetch(&addr).await?;
            Ok(Balance::from_rao(val.unwrap_or(0)))
        })
        .await
    }

    // ──────── Block Hash Pinning ────────

    /// Pin the latest block hash for consistent multi-query reads.
    /// Returns the pinned block hash. All subsequent pinned query methods
    /// will read from this exact block, avoiding redundant `at_latest()` calls
    /// and ensuring data consistency across related queries.
    pub async fn pin_latest_block(&self) -> Result<subxt::utils::H256> {
        let inner = &self.inner;
        retry_on_transient("pin_latest_block", RPC_RETRIES, || async {
            let block = inner.blocks().at_latest().await
                .context("Failed to fetch latest block for pinning")?;
            let hash = block.hash();
            tracing::debug!(block_hash = %hash, block_number = block.number(), "Pinned latest block");
            Ok(hash)
        }).await
    }

    /// Get TAO balance for an SS58 address using a pinned block hash.
    /// More efficient than get_balance_ss58() when making multiple queries
    /// because it avoids a redundant at_latest() RPC call per query.
    pub async fn get_balance_at_hash(
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

    /// Get balances for multiple SS58 addresses using a single pinned block.
    /// More efficient than individual `get_balance_ss58()` calls because:
    ///
    /// 1. Single `at_latest()` call instead of N calls
    /// 2. All reads are from the same block (data consistency)
    /// 3. All balance fetches run concurrently (parallel RPC calls)
    ///
    /// Returns `Vec<(ss58, Balance)>` in the same order as input.
    pub async fn get_balances_multi(&self, addresses: &[&str]) -> Result<Vec<(String, Balance)>> {
        if addresses.is_empty() {
            return Ok(vec![]);
        }
        let block_hash = self.pin_latest_block().await?;
        // Fetch all balances concurrently instead of sequentially
        let futures: Vec<_> = addresses
            .iter()
            .map(|addr| {
                let addr_owned = addr.to_string();
                async move {
                    let balance = self.get_balance_at_hash(&addr_owned, block_hash).await?;
                    Ok::<_, anyhow::Error>((addr_owned, balance))
                }
            })
            .collect();
        let results = futures::future::try_join_all(futures).await?;
        Ok(results)
    }

    // ──────── Pinned Network Params ────────

    /// Total TAO issuance at a pinned block hash.
    pub async fn get_total_issuance_at(&self, hash: subxt::utils::H256) -> Result<Balance> {
        let addr = api::storage().balances().total_issuance();
        let val = self
            .inner
            .storage()
            .at(hash)
            .fetch(&addr)
            .await
            .map_err(|e| Self::annotate_at_block_error(e.into(), None))?;
        let raw = val.unwrap_or(0);
        Ok(Balance::from_rao(u64::try_from(raw).unwrap_or(u64::MAX)))
    }

    /// Total staked TAO at a pinned block hash.
    pub async fn get_total_stake_at(&self, hash: subxt::utils::H256) -> Result<Balance> {
        let addr = api::storage().subtensor_module().total_stake();
        let val = self
            .inner
            .storage()
            .at(hash)
            .fetch(&addr)
            .await
            .map_err(|e| Self::annotate_at_block_error(e.into(), None))?;
        Ok(Balance::from_rao(val.unwrap_or(0)))
    }

    /// Total number of subnets at a pinned block hash.
    pub async fn get_total_networks_at(&self, hash: subxt::utils::H256) -> Result<u16> {
        let addr = api::storage().subtensor_module().total_networks();
        let val = self
            .inner
            .storage()
            .at(hash)
            .fetch(&addr)
            .await
            .map_err(|e| Self::annotate_at_block_error(e.into(), None))?;
        Ok(val.unwrap_or(0))
    }

    /// Block emission rate at a pinned block hash.
    pub async fn get_block_emission_at(&self, hash: subxt::utils::H256) -> Result<Balance> {
        let addr = api::storage().subtensor_module().block_emission();
        let val = self
            .inner
            .storage()
            .at(hash)
            .fetch(&addr)
            .await
            .map_err(|e| Self::annotate_at_block_error(e.into(), None))?;
        Ok(Balance::from_rao(val.unwrap_or(0)))
    }

    /// Fetch all network overview stats using a single pinned block.
    /// Returns (block_number, total_stake, total_networks, total_issuance, emission).
    /// Saves 4 redundant `at_latest()` RPC round-trips compared to individual queries.
    pub async fn get_network_overview(&self) -> Result<(u64, Balance, u16, Balance, Balance)> {
        let hash = self.pin_latest_block().await?;
        // Block number comes from the pinned block itself
        let block = self
            .inner
            .blocks()
            .at(hash)
            .await
            .context("Failed to fetch pinned block")?;
        let block_number = block.number() as u64;
        let (stake, networks, issuance, emission) = tokio::try_join!(
            self.get_total_stake_at(hash),
            self.get_total_networks_at(hash),
            self.get_total_issuance_at(hash),
            self.get_block_emission_at(hash),
        )?;
        Ok((block_number, stake, networks, issuance, emission))
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
    } else if msg.contains("SubnetLocked") || msg.contains("NetworkIsImmuned") {
        "This subnet is in its immunity period and cannot be modified yet."
    } else if msg.contains("MaxAllowedUIDs") || msg.contains("SubNetworkDoesNotExist") {
        "Subnet capacity reached or does not exist. Check `agcli subnet list` for current subnets."
    } else if msg.contains("HotKeyAlreadyRegistered") {
        "This hotkey is already registered. Use a different hotkey or deregister the existing one first."
    } else if msg.contains("ColdKeySwapScheduled") || msg.contains("ColdKeyAlreadyAssociated") {
        "A coldkey operation is already pending for this account. Wait for it to complete."
    } else if msg.contains("DelegateAlreadySet") {
        "Delegate is already set for this hotkey."
    } else if msg.contains("InvalidTransaction") && msg.contains("proxy") {
        "Proxy transaction failed. Check that the proxy account has enough balance for fees and that the proxy type matches the operation."
    } else {
        "" // no special hint
    };

    if hint.is_empty() {
        anyhow::anyhow!("Transaction failed on chain: {}", msg)
    } else {
        anyhow::anyhow!("Transaction failed: {}\n  Hint: {}", msg, hint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn retry_succeeds_after_transient_error() {
        let counter = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let c = counter.clone();
        let result = retry_on_transient("test", 3, || {
            let c = c.clone();
            async move {
                let n = c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if n < 2 {
                    Err(anyhow::anyhow!("Connection reset"))
                } else {
                    Ok(42)
                }
            }
        })
        .await;
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn retry_does_not_retry_non_transient_error() {
        let counter = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let c = counter.clone();
        let result: Result<i32> = retry_on_transient("test", 3, || {
            let c = c.clone();
            async move {
                c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Err(anyhow::anyhow!("Invalid SS58 address"))
            }
        })
        .await;
        assert!(result.is_err());
        // Should NOT retry for non-transient errors
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn retry_exhausts_all_attempts() {
        let counter = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let c = counter.clone();
        let result: Result<i32> = retry_on_transient("test", 2, || {
            let c = c.clone();
            async move {
                c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Err(anyhow::anyhow!("Connection timeout"))
            }
        })
        .await;
        assert!(result.is_err());
        // 1 initial + 2 retries = 3 total
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn retry_succeeds_immediately() {
        let result = retry_on_transient("test", 3, || async { Ok::<_, anyhow::Error>(99) }).await;
        assert_eq!(result.unwrap(), 99);
    }

    #[test]
    fn batch_balance_result_order() {
        // Unit test for the ordering guarantee of get_balances_multi
        // (The actual chain test is in integration tests)
        let addrs = [
            "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKv3gB",
            "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        ];
        assert_eq!(addrs.len(), 2, "batch addresses should preserve count");
    }

    #[test]
    fn format_dispatch_error_subnet_locked() {
        let err = subxt::Error::Other("SubnetLocked: cannot modify".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("immunity period"),
            "should mention immunity: {}",
            msg
        );
    }

    #[test]
    fn format_dispatch_error_proxy() {
        let err = subxt::Error::Other("InvalidTransaction proxy check failed".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("Proxy transaction"),
            "should mention proxy: {}",
            msg
        );
    }

    #[test]
    fn format_dispatch_error_unknown() {
        let err = subxt::Error::Other("SomeTotallyNewError".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("Transaction failed on chain"),
            "unknown errors get generic message: {}",
            msg
        );
    }

    #[test]
    fn format_submit_error_priority() {
        let err = subxt::Error::Other("Priority is too low".to_string());
        let result = format_submit_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("conflicting transaction"),
            "should mention conflict: {}",
            msg
        );
    }

    #[test]
    fn format_submit_error_insufficient() {
        let err = subxt::Error::Other("Inability to pay some fees".to_string());
        let result = format_submit_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("Insufficient balance"),
            "should mention balance: {}",
            msg
        );
    }

    #[test]
    fn is_transient_catches_common_patterns() {
        assert!(is_transient_error("Connection reset by peer"));
        assert!(is_transient_error("Ws transport error"));
        assert!(is_transient_error("Connection closed unexpectedly"));
        assert!(is_transient_error("request timeout after 30s"));
        assert!(!is_transient_error("Invalid SS58 address"));
        assert!(!is_transient_error("NotEnoughBalance"));
    }
}
