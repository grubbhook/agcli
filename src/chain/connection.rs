//! Legacy RPC connection utilities.
//! Most chain interaction now goes through subxt (see chain/mod.rs).
//! These utilities remain for direct JSON-RPC calls if needed.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC request payload.
#[derive(Serialize)]
pub struct RpcRequest {
    pub jsonrpc: &'static str,
    pub id: u64,
    pub method: String,
    pub params: Vec<Value>,
}

/// JSON-RPC response.
#[derive(Deserialize)]
pub struct RpcResponse {
    pub id: u64,
    pub result: Option<Value>,
    pub error: Option<RpcError>,
}

#[derive(Deserialize, Debug)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
}

/// Make a single JSON-RPC call via HTTP.
pub async fn rpc_call(url: &str, method: &str, params: Vec<Value>) -> Result<Value> {
    let client = reqwest::Client::new();
    let req = RpcRequest {
        jsonrpc: "2.0",
        id: 1,
        method: method.to_string(),
        params,
    };

    let resp: RpcResponse = client.post(url).json(&req).send().await?.json().await?;

    if let Some(err) = resp.error {
        anyhow::bail!("RPC error {}: {}", err.code, err.message);
    }

    resp.result
        .ok_or_else(|| anyhow::anyhow!("RPC returned null result"))
}
