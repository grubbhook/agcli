//! Extrinsic submission methods — staking, weights, registration, governance, liquidity.

use anyhow::{Context, Result};
use sp_core::{sr25519, Pair as _};

use crate::types::balance::Balance;
use crate::types::chain_data::*;
use crate::types::network::NetUid;
use crate::{api, AccountId};

use super::Client;

impl Client {
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

    // ──────── Staking ────────

    /// Add stake to a hotkey on a subnet.
    pub async fn add_stake(
        &self,
        pair: &sr25519::Pair,
        hotkey_ss58: &str,
        netuid: NetUid,
        amount: Balance,
    ) -> Result<String> {
        self.add_stake_mev(pair, hotkey_ss58, netuid, amount, false)
            .await
    }

    /// Add stake, optionally wrapping through MEV shield.
    pub async fn add_stake_mev(
        &self,
        pair: &sr25519::Pair,
        hotkey_ss58: &str,
        netuid: NetUid,
        amount: Balance,
        mev: bool,
    ) -> Result<String> {
        let hk = Self::ss58_to_account_id(hotkey_ss58)?;
        let tx = api::tx()
            .subtensor_module()
            .add_stake(hk, netuid.0, amount.rao());
        self.sign_submit_or_mev(&tx, pair, mev).await
    }

    /// Remove stake from a hotkey on a subnet.
    pub async fn remove_stake(
        &self,
        pair: &sr25519::Pair,
        hotkey_ss58: &str,
        netuid: NetUid,
        amount: Balance,
    ) -> Result<String> {
        self.remove_stake_mev(pair, hotkey_ss58, netuid, amount, false)
            .await
    }

    /// Remove stake, optionally wrapping through MEV shield.
    pub async fn remove_stake_mev(
        &self,
        pair: &sr25519::Pair,
        hotkey_ss58: &str,
        netuid: NetUid,
        amount: Balance,
        mev: bool,
    ) -> Result<String> {
        let hk = Self::ss58_to_account_id(hotkey_ss58)?;
        let tx = api::tx()
            .subtensor_module()
            .remove_stake(hk, netuid.0, amount.rao());
        self.sign_submit_or_mev(&tx, pair, mev).await
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

    /// Unstake all from a hotkey.
    pub async fn unstake_all(&self, pair: &sr25519::Pair, hotkey_ss58: &str) -> Result<String> {
        let hk = Self::ss58_to_account_id(hotkey_ss58)?;
        self.sign_submit(&api::tx().subtensor_module().unstake_all(hk), pair)
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

    // ──────── Weights ────────

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
        let mut iter = self.inner.storage().at_latest().await?.iter(addr).await?;
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
        match val {
            Some(v) => Ok(v),
            None => {
                tracing::debug!(
                    netuid = netuid.0,
                    default = 1,
                    "reveal_period_epochs not set on-chain, using default"
                );
                Ok(1)
            }
        }
    }

    // ──────── Registration ────────

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
        match val {
            Some(v) => Ok(v),
            None => {
                tracing::warn!(
                    netuid = netuid.0,
                    default = 10_000_000,
                    "Difficulty not set on-chain, using default"
                );
                Ok(10_000_000)
            }
        }
    }

    // ──────── Child Keys ────────

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

    // ──────── Serve ────────

    /// Serve axon metadata.
    pub async fn serve_axon(
        &self,
        pair: &sr25519::Pair,
        netuid: NetUid,
        axon: &AxonInfo,
    ) -> Result<String> {
        let ip: u128 = axon
            .ip
            .parse()
            .with_context(|| format!("Invalid IP address for axon: {:?}", axon.ip))?;
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

    // ──────── Take ────────

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

    // ──────── Key Swap ────────

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

    // ──────── Root ────────

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

    // ──────── Hotkey Alpha ────────

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

    // ──────── Subnet Identity ────────

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

    // ──────── Proxy ────────

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

    /// Create a pure (anonymous) proxy account.
    pub async fn create_pure_proxy(
        &self,
        pair: &sr25519::Pair,
        proxy_type: &str,
        delay: u32,
        index: u16,
    ) -> Result<String> {
        use subxt::dynamic::Value;
        let variant = parse_proxy_type(proxy_type);
        let tx = subxt::dynamic::tx(
            "Proxy",
            "create_pure",
            vec![
                Value::unnamed_variant(variant, []),
                Value::u128(delay as u128),
                Value::u128(index as u128),
            ],
        );
        self.sign_submit(&tx, pair).await
    }

    /// Kill (destroy) a pure proxy account. Funds become inaccessible.
    pub async fn kill_pure_proxy(
        &self,
        pair: &sr25519::Pair,
        spawner_ss58: &str,
        proxy_type: &str,
        index: u16,
        height: u32,
        ext_index: u32,
    ) -> Result<String> {
        use subxt::dynamic::Value;
        let spawner = Self::ss58_to_account_id(spawner_ss58)?;
        let variant = parse_proxy_type(proxy_type);
        let tx = subxt::dynamic::tx(
            "Proxy",
            "kill_pure",
            vec![
                Value::unnamed_variant("Id", [Value::from_bytes(spawner.0)]),
                Value::unnamed_variant(variant, []),
                Value::u128(index as u128),
                Value::u128(height as u128),    // compact<BlockNumber>
                Value::u128(ext_index as u128), // compact<u32>
            ],
        );
        self.sign_submit(&tx, pair).await
    }

    /// Associate a hotkey with the coldkey on-chain.
    pub async fn try_associate_hotkey(
        &self,
        pair: &sr25519::Pair,
        hotkey_ss58: &str,
    ) -> Result<String> {
        use subxt::dynamic::Value;
        let hk = Self::ss58_to_account_id(hotkey_ss58)?;
        self.submit_raw_call(
            pair,
            "SubtensorModule",
            "try_associate_hotkey",
            vec![Value::from_bytes(hk.0)],
        )
        .await
    }

    /// Set subnet token symbol (subnet owner only).
    pub async fn set_subnet_symbol(
        &self,
        pair: &sr25519::Pair,
        netuid: NetUid,
        symbol: &str,
    ) -> Result<String> {
        use subxt::dynamic::Value;
        self.submit_raw_call(
            pair,
            "SubtensorModule",
            "update_symbol",
            vec![
                Value::u128(netuid.0 as u128),
                Value::from_bytes(symbol.as_bytes()),
            ],
        )
        .await
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

    // ──────── Root Claim ────────

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
            _ => anyhow::bail!(
                "Invalid claim type: {}. Use 'swap', 'keep', or 'keep-subnets'",
                claim_type
            ),
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
            vec![Value::u128(netuid.0 as u128), Value::bool(enable)],
        )
        .await
    }

    // ──────── MEV Shield ────────

    /// Fetch the current ML-KEM-768 public key for MEV shield from chain storage.
    pub async fn get_mev_shield_next_key(&self) -> Result<Vec<u8>> {
        let storage_query = subxt::dynamic::storage("MevShield", "NextKey", ());
        let result = self
            .inner
            .storage()
            .at_latest()
            .await?
            .fetch(&storage_query)
            .await?;
        match result {
            Some(val) => {
                // Decode the BoundedVec<u8> from the storage value
                let bytes: Vec<u8> = val.as_type()?;
                Ok(bytes)
            }
            None => anyhow::bail!(
                "MEV shield key not available — the chain may not have MEV shield enabled"
            ),
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
            vec![Value::from_bytes(commitment), Value::from_bytes(ciphertext)],
        )
        .await
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

    // ──────── Dynamic Dispatch ────────

    /// Sign and submit any Payload (public, for batch/dynamic calls from outside the chain module).
    pub async fn sign_submit_dyn<T: subxt::tx::Payload>(
        &self,
        tx: &T,
        pair: &sr25519::Pair,
    ) -> Result<String> {
        self.sign_submit(tx, pair).await
    }

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

    /// Submit a call wrapped in `Sudo.sudo()` — used for AdminUtils and other privileged calls.
    ///
    /// The `pair` must be the chain's sudo key (typically `//Alice` on localnet).
    /// Returns an error (not panic) if the call doesn't exist in the runtime metadata.
    pub async fn submit_sudo_raw_call(
        &self,
        pair: &sr25519::Pair,
        pallet: &str,
        call: &str,
        fields: Vec<subxt::dynamic::Value>,
    ) -> Result<String> {
        // Validate the call exists in the pallet's metadata before encoding.
        // subxt panics (rather than returning Err) if the call variant is missing,
        // so we pre-check using the metadata API directly.
        let metadata = self.inner.metadata();
        match metadata.pallet_by_name(pallet) {
            Some(p) => {
                if p.call_variant_by_name(call).is_none() {
                    anyhow::bail!("Call {}.{} not found in runtime metadata", pallet, call);
                }
            }
            None => {
                anyhow::bail!("Pallet '{}' not found in runtime metadata", pallet);
            }
        }

        // Convert to a Value variant (RuntimeCall enum shape) for Sudo wrapping.
        let inner = subxt::dynamic::tx(pallet, call, fields);
        let inner_value = inner.into_value();
        let sudo_tx = subxt::dynamic::tx("Sudo", "sudo", vec![inner_value]);
        self.sign_submit(&sudo_tx, pair).await
    }

    /// Submit a call wrapped in `Sudo.sudo()` and verify the inner dispatch succeeded.
    ///
    /// Unlike `submit_sudo_raw_call`, this checks the `Sudo::Sudid` event to detect inner
    /// dispatch errors that `Sudo.sudo()` would otherwise swallow (the outer tx always succeeds
    /// if the caller is the sudo key).
    pub async fn submit_sudo_raw_call_checked(
        &self,
        pair: &sr25519::Pair,
        pallet: &str,
        call: &str,
        fields: Vec<subxt::dynamic::Value>,
    ) -> Result<String> {
        // Validate the call exists in the pallet's metadata before encoding.
        let metadata = self.inner.metadata();
        match metadata.pallet_by_name(pallet) {
            Some(p) => {
                if p.call_variant_by_name(call).is_none() {
                    anyhow::bail!("Call {}.{} not found in runtime metadata", pallet, call);
                }
            }
            None => {
                anyhow::bail!("Pallet '{}' not found in runtime metadata", pallet);
            }
        }

        // Build and submit the Sudo-wrapped tx
        let inner_tx = subxt::dynamic::tx(pallet, call, fields);
        let inner_value = inner_tx.into_value();
        let sudo_tx = subxt::dynamic::tx("Sudo", "sudo", vec![inner_value]);

        let signer = Self::signer(pair);
        let progress = self
            .inner
            .tx()
            .sign_and_submit_then_watch_default(&sudo_tx, &signer)
            .await
            .map_err(|e| anyhow::anyhow!("Sudo submission failed: {}", e))?;

        let events = progress
            .wait_for_finalized_success()
            .await
            .map_err(|e| anyhow::anyhow!("Sudo tx dispatch failed: {}", e))?;

        let hash = format!("{:?}", events.extrinsic_hash());

        // Check the Sudid event for inner dispatch errors.
        // The Sudid event has: sudo_result: Result<(), DispatchError>.
        // In the debug format, a failed inner call produces:
        //   Variant { name: "Err", values: Unnamed([...Module error...]) }
        // We look specifically for the Err variant name in the sudo_result field.
        for event in events.all_events_in_block().iter() {
            let event = match event {
                Ok(e) => e,
                Err(_) => continue,
            };
            if event.pallet_name() == "Sudo" && event.variant_name() == "Sudid" {
                let field_str = format!("{:?}", event.field_values());
                // Check for 'name: "Err"' which indicates the inner dispatch returned an error.
                // We use 'name: "Err"' rather than just 'Err(' to avoid false positives from
                // field names like "error" that appear in the module error structure.
                if field_str.contains("name: \"Err\"") {
                    anyhow::bail!(
                        "Sudo inner dispatch failed for {}.{}: {}",
                        pallet,
                        call,
                        field_str
                    );
                }
            }
        }

        Ok(hash)
    }

    // ──────── Commitments ────────

    /// Set commitment data on a subnet (miners use this to publish endpoints).
    pub async fn set_commitment(
        &self,
        pair: &sr25519::Pair,
        netuid: u16,
        data: &str,
    ) -> Result<String> {
        use subxt::dynamic::Value;
        // Build the Data fields — each comma-separated entry becomes a Raw field.
        // The commitment pallet uses the same Data enum as identity (Raw0..Raw128).
        // We encode each field as a named variant "RawN" with the byte array.
        let fields: Vec<Value> = data
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| {
                let bytes = s.as_bytes();
                let len = bytes.len().min(128);
                let variant_name = format!("Raw{}", len);
                Value::unnamed_variant(variant_name, [Value::from_bytes(&bytes[..len])])
            })
            .collect();
        let info = Value::named_composite([("fields", Value::unnamed_composite(fields))]);
        self.submit_raw_call(
            pair,
            "Commitments",
            "set_commitment",
            vec![Value::u128(netuid as u128), info],
        )
        .await
    }

    // ──────── Multisig ────────

    /// Submit a multisig call (propose via approve_as_multi).
    pub async fn submit_multisig_call(
        &self,
        pair: &sr25519::Pair,
        other_signatories: &[AccountId],
        threshold: u16,
        pallet: &str,
        call: &str,
        fields: Vec<subxt::dynamic::Value>,
    ) -> Result<String> {
        let inner = subxt::dynamic::tx(pallet, call, fields);
        let encoded = self.inner.tx().call_data(&inner)?;
        let call_hash = sp_core::hashing::blake2_256(&encoded);
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
}

/// Parse a proxy type string to the on-chain variant name.
pub(crate) fn parse_proxy_type(s: &str) -> &'static str {
    match s.to_lowercase().as_str() {
        "any" => "Any",
        "owner" => "Owner",
        "nontransfer" | "non_transfer" => "NonTransfer",
        "staking" => "Staking",
        "noncritical" | "non_critical" => "NonCritical",
        "triumvirate" => "Triumvirate",
        "governance" => "Governance",
        "senate" => "Senate",
        "nonfungible" | "non_fungible" => "NonFungible",
        "registration" => "Registration",
        "transfer" => "Transfer",
        "smalltransfer" | "small_transfer" => "SmallTransfer",
        "rootweights" | "root_weights" => "RootWeights",
        "childkeys" | "child_keys" => "ChildKeys",
        "sudouncheckedsetcode" | "sudo_unchecked_set_code" => "SudoUncheckedSetCode",
        "swaphotkey" | "swap_hotkey" => "SwapHotkey",
        "subnetleasebeneficiary" | "subnet_lease_beneficiary" => "SubnetLeaseBeneficiary",
        "rootclaim" | "root_claim" => "RootClaim",
        _ => "Any",
    }
}
