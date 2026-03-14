//! Legacy storage queries via raw RPC.
//! Most queries now go through subxt runtime APIs (see chain/mod.rs).
//! These remain for raw storage key access if needed.

use super::connection::rpc_call;
use anyhow::Result;

/// Query a raw storage key.
pub async fn get_storage(rpc_url: &str, storage_key: &str) -> Result<Option<String>> {
    let result = rpc_call(
        rpc_url,
        "state_getStorage",
        vec![serde_json::Value::String(storage_key.to_string())],
    )
    .await?;

    Ok(result.as_str().map(|s| s.to_string()))
}

/// Query storage at a specific block hash.
pub async fn get_storage_at(
    rpc_url: &str,
    storage_key: &str,
    block_hash: &str,
) -> Result<Option<String>> {
    let result = rpc_call(
        rpc_url,
        "state_getStorage",
        vec![
            serde_json::Value::String(storage_key.to_string()),
            serde_json::Value::String(block_hash.to_string()),
        ],
    )
    .await?;

    Ok(result.as_str().map(|s| s.to_string()))
}
