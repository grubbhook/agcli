//! Subnet queries.

use crate::chain::Client;
use crate::types::chain_data::SubnetInfo;
use anyhow::Result;

/// List all subnets with basic info.
pub async fn list_subnets(client: &Client) -> Result<Vec<SubnetInfo>> {
    // Enrich subnet list with real names from DynamicInfo (one call vs N identity queries)
    let subnets_arc = client.get_all_subnets().await?;
    let mut subnets = (*subnets_arc).clone();
    if let Ok(dynamic) = client.get_all_dynamic_info().await {
        let name_map: std::collections::HashMap<u16, (String, u64)> = dynamic
            .iter()
            .filter(|d| !d.name.is_empty())
            .map(|d| (d.netuid.0, (d.name.clone(), d.total_emission())))
            .collect();
        for s in &mut subnets {
            if let Some((name, emission)) = name_map.get(&s.netuid.0) {
                s.name = name.clone();
                if s.emission_value == 0 {
                    s.emission_value = *emission;
                }
            }
        }
    }
    Ok(subnets)
}

