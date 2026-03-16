//! Localnet command handlers — Docker chain lifecycle management.

use crate::cli::helpers::*;
use crate::cli::LocalnetCommands;
use crate::localnet::{self, LocalnetConfig};
use anyhow::Result;

pub(super) async fn handle_localnet(cmd: LocalnetCommands, ctx: &Ctx<'_>) -> Result<()> {
    match cmd {
        LocalnetCommands::Start {
            image,
            container,
            port,
            wait,
            timeout,
        } => {
            let cfg = LocalnetConfig {
                image: image.unwrap_or_else(|| localnet::DEFAULT_IMAGE.to_string()),
                container_name: container
                    .unwrap_or_else(|| localnet::DEFAULT_CONTAINER.to_string()),
                port: port.unwrap_or(9944),
                wait: wait.unwrap_or(true),
                wait_timeout: timeout.unwrap_or(120),
            };

            if !ctx.output.is_json() {
                eprintln!(
                    "Starting localnet (image: {}, port: {})...",
                    cfg.image, cfg.port
                );
            }

            let info = localnet::start(&cfg).await?;

            if ctx.output.is_json() {
                print_json(&serde_json::json!({
                    "status": "started",
                    "container_name": info.container_name,
                    "container_id": info.container_id,
                    "image": info.image,
                    "endpoint": info.endpoint,
                    "port": info.port,
                    "block_height": info.block_height,
                    "dev_accounts": info.dev_accounts.iter().map(|a| {
                        serde_json::json!({
                            "name": a.name,
                            "uri": a.uri,
                            "ss58": a.ss58,
                            "balance": a.balance,
                        })
                    }).collect::<Vec<_>>(),
                }));
            } else {
                println!("Localnet started successfully!");
                println!("  Container: {} ({})", info.container_name, &info.container_id[..12]);
                println!("  Image:     {}", info.image);
                println!("  Endpoint:  {}", info.endpoint);
                println!("  Block:     {}", info.block_height);
                println!();
                println!("Dev accounts:");
                for a in &info.dev_accounts {
                    println!("  {} ({}) — {}", a.name, a.ss58, a.balance);
                }
                println!();
                println!("Connect: agcli --network local <command>");
            }
            Ok(())
        }

        LocalnetCommands::Stop { container } => {
            let name = container.unwrap_or_else(|| localnet::DEFAULT_CONTAINER.to_string());
            localnet::stop(&name)?;

            if ctx.output.is_json() {
                print_json(&serde_json::json!({
                    "status": "stopped",
                    "container_name": name,
                }));
            } else {
                println!("Localnet '{}' stopped.", name);
            }
            Ok(())
        }

        LocalnetCommands::Status { container, port } => {
            let name = container.unwrap_or_else(|| localnet::DEFAULT_CONTAINER.to_string());
            let p = port.unwrap_or(9944);
            let st = localnet::status(&name, p).await?;

            if ctx.output.is_json() {
                print_json_ser(&st);
            } else if st.running {
                println!("Localnet '{}': RUNNING", st.container_name);
                if let Some(ref id) = st.container_id {
                    println!("  Container ID: {}", id);
                }
                if let Some(ref img) = st.image {
                    println!("  Image:        {}", img);
                }
                if let Some(ref ep) = st.endpoint {
                    println!("  Endpoint:     {}", ep);
                }
                if let Some(bh) = st.block_height {
                    println!("  Block height: {}", bh);
                }
            } else {
                println!("Localnet '{}': NOT RUNNING", st.container_name);
                println!("  Start with: agcli localnet start");
            }
            Ok(())
        }

        LocalnetCommands::Reset {
            image,
            container,
            port,
            timeout,
        } => {
            let cfg = LocalnetConfig {
                image: image.unwrap_or_else(|| localnet::DEFAULT_IMAGE.to_string()),
                container_name: container
                    .unwrap_or_else(|| localnet::DEFAULT_CONTAINER.to_string()),
                port: port.unwrap_or(9944),
                wait: true,
                wait_timeout: timeout.unwrap_or(120),
            };

            if !ctx.output.is_json() {
                eprintln!("Resetting localnet '{}'...", cfg.container_name);
            }

            let info = localnet::reset(&cfg).await?;

            if ctx.output.is_json() {
                print_json(&serde_json::json!({
                    "status": "reset",
                    "container_name": info.container_name,
                    "container_id": info.container_id,
                    "endpoint": info.endpoint,
                    "block_height": info.block_height,
                }));
            } else {
                println!("Localnet reset complete!");
                println!("  Container: {} ({})", info.container_name, &info.container_id[..12]);
                println!("  Endpoint:  {}", info.endpoint);
                println!("  Block:     {}", info.block_height);
            }
            Ok(())
        }

        LocalnetCommands::Logs { container, tail } => {
            let name = container.unwrap_or_else(|| localnet::DEFAULT_CONTAINER.to_string());
            let log_output = localnet::logs(&name, tail)?;
            print!("{}", log_output);
            Ok(())
        }
    }
}
