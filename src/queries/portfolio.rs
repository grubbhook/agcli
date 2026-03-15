//! Portfolio queries — aggregate all stakes, balances, and subnet positions.

use crate::chain::Client;
use crate::types::Balance;
use anyhow::Result;
use serde::Serialize;

/// A user's complete portfolio across all subnets.
#[derive(Debug, Serialize)]
pub struct Portfolio {
    pub coldkey_ss58: String,
    pub free_balance: Balance,
    pub total_staked: Balance,
    pub positions: Vec<SubnetPosition>,
}

/// Stake position on a single subnet.
#[derive(Debug, Serialize)]
pub struct SubnetPosition {
    pub netuid: u16,
    pub subnet_name: String,
    pub hotkey_ss58: String,
    pub alpha_stake: u64,
    pub tao_equivalent: Balance,
    pub price: f64,
}

/// Fetch the full portfolio for a coldkey (resolves subnet names and prices from DynamicInfo).
pub async fn fetch_portfolio(client: &Client, coldkey_ss58: &str) -> Result<Portfolio> {
    // Parallel fetch: balance, stakes, and dynamic info all at once
    let (balance, stakes, dynamic) = tokio::try_join!(
        client.get_balance_ss58(coldkey_ss58),
        client.get_stake_for_coldkey(coldkey_ss58),
        async {
            match client.get_all_dynamic_info().await {
                Ok(d) => Ok::<_, anyhow::Error>(d),
                Err(e) => {
                    tracing::warn!("Failed to fetch dynamic info for portfolio: {e:#}");
                    Ok(std::sync::Arc::new(vec![]))
                }
            }
        },
    )?;
    let dynamic_map: std::collections::HashMap<u16, &crate::types::chain_data::DynamicInfo> =
        dynamic.iter().map(|d| (d.netuid.0, d)).collect();

    let positions: Vec<SubnetPosition> = stakes
        .iter()
        .map(|s| {
            let di = dynamic_map.get(&s.netuid.0);
            SubnetPosition {
                netuid: s.netuid.0,
                subnet_name: di.map(|d| d.name.clone()).unwrap_or_default(),
                hotkey_ss58: s.hotkey.clone(),
                alpha_stake: s.alpha_stake.raw(),
                tao_equivalent: s.stake,
                price: di.map(|d| d.price).unwrap_or(0.0),
            }
        })
        .collect();

    let total_staked = positions
        .iter()
        .fold(Balance::ZERO, |acc, p| acc + p.tao_equivalent);

    Ok(Portfolio {
        coldkey_ss58: coldkey_ss58.to_string(),
        free_balance: balance,
        total_staked,
        positions,
    })
}
