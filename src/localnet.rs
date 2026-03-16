//! Docker-based local chain management.
//!
//! Provides `Localnet` for starting, stopping, and managing subtensor Docker
//! containers — the same infrastructure used in e2e tests, exposed as a
//! first-class SDK primitive.
//!
//! ```rust,no_run
//! use agcli::localnet::{self, LocalnetConfig};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let info = localnet::start(&LocalnetConfig::default()).await?;
//!     println!("Chain ready at {} (block {})", info.endpoint, info.block_height);
//!     localnet::stop(&info.container_name)?;
//!     Ok(())
//! }
//! ```

use anyhow::{bail, Context, Result};
use std::process::Command;
use std::time::Duration;

/// Default Docker image for local subtensor.
pub const DEFAULT_IMAGE: &str = "ghcr.io/opentensor/subtensor-localnet:devnet-ready";

/// Default container name.
pub const DEFAULT_CONTAINER: &str = "agcli_localnet";

/// Default WebSocket endpoint.
pub const DEFAULT_WS: &str = "ws://127.0.0.1:9944";

/// Configuration for starting a local chain.
#[derive(Debug, Clone)]
pub struct LocalnetConfig {
    /// Docker image to use.
    pub image: String,
    /// Container name.
    pub container_name: String,
    /// Host port for RPC (mapped to container 9944).
    pub port: u16,
    /// Wait for blocks to be produced before returning.
    pub wait: bool,
    /// Maximum seconds to wait for chain readiness.
    pub wait_timeout: u64,
}

impl Default for LocalnetConfig {
    fn default() -> Self {
        Self {
            image: DEFAULT_IMAGE.to_string(),
            container_name: DEFAULT_CONTAINER.to_string(),
            port: 9944,
            wait: true,
            wait_timeout: 120,
        }
    }
}

/// Status of a localnet container.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LocalnetStatus {
    pub running: bool,
    pub container_name: String,
    pub container_id: Option<String>,
    pub image: Option<String>,
    pub endpoint: Option<String>,
    pub block_height: Option<u64>,
    pub uptime: Option<String>,
}

/// Result returned after successfully starting a local chain.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LocalnetInfo {
    pub container_name: String,
    pub container_id: String,
    pub image: String,
    pub endpoint: String,
    pub port: u16,
    pub block_height: u64,
    pub dev_accounts: Vec<DevAccount>,
}

/// Pre-funded dev account available on localnet.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DevAccount {
    pub name: String,
    pub uri: String,
    pub ss58: String,
    pub balance: String,
}

/// Well-known dev accounts on localnet.
pub fn dev_accounts() -> Vec<DevAccount> {
    vec![
        DevAccount {
            name: "Alice".to_string(),
            uri: "//Alice".to_string(),
            ss58: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".to_string(),
            balance: "1000000 TAO (sudo)".to_string(),
        },
        DevAccount {
            name: "Bob".to_string(),
            uri: "//Bob".to_string(),
            ss58: "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty".to_string(),
            balance: "1000000 TAO".to_string(),
        },
    ]
}

/// Start a localnet Docker container.
///
/// Kills any stale container on the same port, starts fresh, and optionally
/// waits for the chain to produce blocks.
pub async fn start(cfg: &LocalnetConfig) -> Result<LocalnetInfo> {
    // Kill stale containers
    let _ = Command::new("docker")
        .args(["rm", "-f", &cfg.container_name])
        .output();
    let port_str = cfg.port.to_string();
    let _ = Command::new("bash")
        .args([
            "-c",
            &format!(
                "docker ps -q --filter publish={} | xargs -r docker rm -f",
                port_str
            ),
        ])
        .output();

    std::thread::sleep(Duration::from_secs(1));

    // Start container
    let output = Command::new("docker")
        .args([
            "run",
            "--rm",
            "-d",
            "--name",
            &cfg.container_name,
            "-p",
            &format!("{}:9944", cfg.port),
            "-p",
            &format!("{}:9945", cfg.port + 1),
            &cfg.image,
        ])
        .output()
        .context("Failed to run Docker — is Docker installed and running?")?;

    if !output.status.success() {
        bail!(
            "Docker container failed to start:\n  stdout: {}\n  stderr: {}\n  Pull image: docker pull {}",
            String::from_utf8_lossy(&output.stdout).trim(),
            String::from_utf8_lossy(&output.stderr).trim(),
            cfg.image
        );
    }

    let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let endpoint = format!("ws://127.0.0.1:{}", cfg.port);

    // Wait for chain readiness
    let block_height = if cfg.wait {
        wait_for_blocks(&endpoint, cfg.wait_timeout).await?
    } else {
        0
    };

    Ok(LocalnetInfo {
        container_name: cfg.container_name.clone(),
        container_id,
        image: cfg.image.clone(),
        endpoint,
        port: cfg.port,
        block_height,
        dev_accounts: dev_accounts(),
    })
}

/// Stop and remove a localnet container.
pub fn stop(container_name: &str) -> Result<()> {
    let output = Command::new("docker")
        .args(["rm", "-f", container_name])
        .output()
        .context("Failed to stop Docker container")?;

    let stderr = String::from_utf8_lossy(&output.stderr);

    // docker rm -f may return exit 0 even for missing containers (Docker 28+),
    // so check stderr regardless of exit code.
    if stderr.contains("No such container") {
        bail!(
            "Container '{}' not found (already stopped?)",
            container_name
        );
    }

    if !output.status.success() {
        bail!(
            "Failed to stop container '{}': {}",
            container_name,
            stderr.trim()
        );
    }
    Ok(())
}

/// Query the status of a localnet container.
pub async fn status(container_name: &str, port: u16) -> Result<LocalnetStatus> {
    let inspect = Command::new("docker")
        .args([
            "inspect",
            "--format",
            "{{.State.Running}}|{{.Id}}|{{.Config.Image}}|{{.State.StartedAt}}",
            container_name,
        ])
        .output();

    match inspect {
        Ok(out) if out.status.success() => {
            let line = String::from_utf8_lossy(&out.stdout).trim().to_string();
            let parts: Vec<&str> = line.split('|').collect();
            let running = parts.first().map(|s| *s == "true").unwrap_or(false);
            let container_id = parts.get(1).map(|s| s[..12.min(s.len())].to_string());
            let image = parts.get(2).map(|s| s.to_string());
            let started_at = parts.get(3).map(|s| s.to_string());

            let endpoint = format!("ws://127.0.0.1:{}", port);

            // Try to get block height if running
            let block_height = if running {
                get_block_height(&endpoint).await.ok()
            } else {
                None
            };

            Ok(LocalnetStatus {
                running,
                container_name: container_name.to_string(),
                container_id,
                image,
                endpoint: if running { Some(endpoint) } else { None },
                block_height,
                uptime: started_at,
            })
        }
        _ => Ok(LocalnetStatus {
            running: false,
            container_name: container_name.to_string(),
            container_id: None,
            image: None,
            endpoint: None,
            block_height: None,
            uptime: None,
        }),
    }
}

/// Reset: stop, then start fresh.
pub async fn reset(cfg: &LocalnetConfig) -> Result<LocalnetInfo> {
    let _ = stop(&cfg.container_name);
    start(cfg).await
}

/// Get container logs.
pub fn logs(container_name: &str, tail: Option<u32>) -> Result<String> {
    let mut args = vec!["logs".to_string()];
    if let Some(n) = tail {
        args.push("--tail".to_string());
        args.push(n.to_string());
    }
    args.push(container_name.to_string());

    let output = Command::new("docker")
        .args(&args)
        .output()
        .context("Failed to get Docker logs")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("No such container") {
            bail!("Container '{}' not found", container_name);
        }
        bail!(
            "Failed to get logs for '{}': {}",
            container_name,
            stderr.trim()
        );
    }

    // Docker logs go to both stdout and stderr
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if stdout.is_empty() {
        Ok(stderr.to_string())
    } else if stderr.is_empty() {
        Ok(stdout.to_string())
    } else {
        Ok(format!("{}{}", stderr, stdout))
    }
}

/// Wait for a chain at `endpoint` to produce blocks.
async fn wait_for_blocks(endpoint: &str, timeout_secs: u64) -> Result<u64> {
    let max_attempts = (timeout_secs / 2).max(1);
    for attempt in 1..=max_attempts {
        if let Ok(client) = crate::chain::Client::connect(endpoint).await {
            if let Ok(block) = client.get_block_number().await {
                if block > 0 {
                    return Ok(block);
                }
            }
        }
        if attempt == max_attempts {
            bail!(
                "Chain at {} did not become ready after {} seconds",
                endpoint,
                timeout_secs
            );
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    unreachable!()
}

/// Quick block height check.
async fn get_block_height(endpoint: &str) -> Result<u64> {
    let client = crate::chain::Client::connect(endpoint).await?;
    let block = client.get_block_number().await?;
    Ok(block)
}
