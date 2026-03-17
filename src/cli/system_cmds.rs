//! System utility command handlers (config, completions, explain, update, batch, doctor).

use crate::chain::Client;
use crate::cli::helpers::*;
use crate::cli::*;
use anyhow::{Context, Result};
use clap::CommandFactory;

pub(super) async fn handle_config(cmd: ConfigCommands) -> Result<()> {
    match cmd {
        ConfigCommands::Show => {
            let cfg = crate::config::Config::load();
            let s = toml::to_string_pretty(&cfg)?;
            if s.trim().is_empty() {
                println!(
                    "No configuration set. Use 'agcli config set <key> <value>' to configure."
                );
            } else {
                println!("{}", s);
            }
            Ok(())
        }
        ConfigCommands::Set { key, value } => {
            let mut cfg = crate::config::Config::load();
            match key.as_str() {
                "network" => {
                    validate_config_network(&value)?;
                    cfg.network = Some(value);
                }
                "endpoint" => {
                    validate_url(&value, "endpoint")?;
                    cfg.endpoint = Some(value);
                }
                "wallet_dir" => cfg.wallet_dir = Some(value),
                "wallet" => cfg.wallet = Some(value),
                "hotkey" => cfg.hotkey = Some(value),
                "output" => {
                    if !["table", "json", "csv"].contains(&value.as_str()) {
                        anyhow::bail!("Invalid output format '{}'. Must be: table, json, csv", value);
                    }
                    cfg.output = Some(value);
                }
                "proxy" => {
                    validate_ss58(&value, "proxy address")?;
                    cfg.proxy = Some(value);
                }
                "live_interval" => {
                    let v: u64 = value.parse().map_err(|_| anyhow::anyhow!("Invalid interval '{}'", value))?;
                    cfg.live_interval = Some(v);
                }
                "batch" => {
                    let v: bool = value.parse().map_err(|_| anyhow::anyhow!("Invalid bool '{}'. Use: true, false", value))?;
                    cfg.batch = Some(v);
                }
                k if k.starts_with("spending_limit.") => {
                    let netuid = &k["spending_limit.".len()..];
                    let limit: f64 = value.parse().map_err(|_| anyhow::anyhow!("Invalid TAO amount '{}'", value))?;
                    validate_spending_limit(limit, netuid)?;
                    let limits = cfg.spending_limits.get_or_insert_with(Default::default);
                    limits.insert(netuid.to_string(), limit);
                }
                _ => anyhow::bail!("Unknown config key '{}'. Valid keys: network, endpoint, wallet_dir, wallet, hotkey, output, proxy, live_interval, batch, spending_limit.<netuid>", key),
            }
            cfg.save()?;
            println!("Set {} = {}", key, cfg_value_display(&key, &cfg));
            Ok(())
        }
        ConfigCommands::Unset { key } => {
            let mut cfg = crate::config::Config::load();
            match key.as_str() {
                "network" => cfg.network = None,
                "endpoint" => cfg.endpoint = None,
                "wallet_dir" => cfg.wallet_dir = None,
                "wallet" => cfg.wallet = None,
                "hotkey" => cfg.hotkey = None,
                "output" => cfg.output = None,
                "proxy" => cfg.proxy = None,
                "live_interval" => cfg.live_interval = None,
                "batch" => cfg.batch = None,
                k if k.starts_with("spending_limit.") => {
                    let netuid = &k["spending_limit.".len()..];
                    if let Some(ref mut limits) = cfg.spending_limits {
                        limits.remove(netuid);
                    }
                }
                _ => anyhow::bail!("Unknown config key '{}'", key),
            }
            cfg.save()?;
            println!("Unset {}", key);
            Ok(())
        }
        ConfigCommands::Path => {
            println!("{}", crate::config::Config::default_path().display());
            Ok(())
        }
        ConfigCommands::CacheClear => {
            let keys = crate::queries::disk_cache::list_keys();
            if keys.is_empty() {
                println!("Disk cache is already empty.");
            } else {
                for key in &keys {
                    crate::queries::disk_cache::remove(key);
                }
                println!("Cleared {} cached entries.", keys.len());
            }
            Ok(())
        }
        ConfigCommands::CacheInfo => {
            let keys = crate::queries::disk_cache::list_keys();
            let cache_path = crate::queries::disk_cache::path();
            println!("Cache directory: {}", cache_path.display());
            if keys.is_empty() {
                println!("No cached entries.");
            } else {
                let mut total_bytes: u64 = 0;
                for key in &keys {
                    let path = cache_path.join(format!("{}.json", key));
                    let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                    total_bytes += size;
                    println!("  {} ({:.1}KB)", key, size as f64 / 1024.0);
                }
                println!(
                    "Total: {} entries, {:.1}KB",
                    keys.len(),
                    total_bytes as f64 / 1024.0
                );
            }
            Ok(())
        }
    }
}

fn cfg_value_display(key: &str, cfg: &crate::config::Config) -> String {
    match key {
        "network" => cfg.network.clone().unwrap_or_default(),
        "endpoint" => cfg.endpoint.clone().unwrap_or_default(),
        "wallet_dir" => cfg.wallet_dir.clone().unwrap_or_default(),
        "wallet" => cfg.wallet.clone().unwrap_or_default(),
        "hotkey" => cfg.hotkey.clone().unwrap_or_default(),
        "output" => cfg.output.clone().unwrap_or_default(),
        "proxy" => cfg.proxy.clone().unwrap_or_default(),
        "live_interval" => cfg.live_interval.map(|v| v.to_string()).unwrap_or_default(),
        "batch" => cfg.batch.map(|v| v.to_string()).unwrap_or_default(),
        k if k.starts_with("spending_limit.") => {
            let netuid = &k["spending_limit.".len()..];
            cfg.spending_limits
                .as_ref()
                .and_then(|m| m.get(netuid))
                .map(|v| format!("{} TAO", v))
                .unwrap_or_default()
        }
        _ => String::new(),
    }
}

pub(super) fn generate_completions(shell: &str) {
    use clap_complete::{generate, Shell};
    let mut cmd = Cli::command();
    let shell_enum = match shell {
        "bash" => Shell::Bash,
        "zsh" => Shell::Zsh,
        "fish" => Shell::Fish,
        "powershell" => Shell::PowerShell,
        _ => {
            eprintln!(
                "Unsupported shell: {}. Use: bash, zsh, fish, powershell",
                shell
            );
            return;
        }
    };
    generate(shell_enum, &mut cmd, "agcli", &mut std::io::stdout());
}

pub(super) fn handle_explain(topic: Option<&str>, output: OutputFormat, full: bool) -> Result<()> {
    match topic {
        Some(t) => {
            // --full: load from docs/commands/<topic>.md on disk
            if full {
                return load_full_doc(t, output);
            }
            match crate::utils::explain::explain(t) {
                Some(text) => {
                    if output.is_json() {
                        print_json(&serde_json::json!({
                            "topic": t,
                            "content": text,
                        }));
                    } else {
                        println!("{}", text);
                    }
                }
                None => {
                    let topics: Vec<serde_json::Value> = crate::utils::explain::list_topics()
                        .iter()
                        .map(|(k, d)| serde_json::json!({"topic": k, "description": d}))
                        .collect();
                    if output.is_json() {
                        eprint_json(&serde_json::json!({
                            "error": true,
                            "message": format!("Unknown topic '{}'", t),
                            "available_topics": topics,
                        }));
                    } else {
                        eprintln!("Unknown topic '{}'. Available topics:\n", t);
                        for (key, desc) in crate::utils::explain::list_topics() {
                            eprintln!("  {:<16} {}", key, desc);
                        }
                        eprintln!("\nUsage: agcli explain --topic <topic>");
                    }
                    anyhow::bail!("Unknown topic '{}'", t);
                }
            }
        }
        None => {
            if full {
                return list_full_docs(output);
            }
            let topics: Vec<serde_json::Value> = crate::utils::explain::list_topics()
                .iter()
                .map(|(k, d)| serde_json::json!({"topic": k, "description": d}))
                .collect();
            if output.is_json() {
                print_json(&serde_json::json!(topics));
            } else {
                println!("Available topics:\n");
                for (key, desc) in crate::utils::explain::list_topics() {
                    println!("  {:<16} {}", key, desc);
                }
                println!("\nUsage: agcli explain --topic <topic>");
                println!("       agcli explain --topic <topic> --full   (full agent docs)");
            }
        }
    }
    Ok(())
}

/// Resolve the docs/ directory — checks next to the binary first, then CWD.
fn find_docs_dir() -> Option<std::path::PathBuf> {
    // 1. Next to the binary (production install)
    if let Ok(exe) = std::env::current_exe() {
        let dir = exe.parent().map(|p| p.join("docs/commands"));
        if let Some(ref d) = dir {
            if d.is_dir() {
                return dir;
            }
        }
    }
    // 2. Repo root (development)
    let cwd = std::path::PathBuf::from("docs/commands");
    if cwd.is_dir() {
        return Some(cwd);
    }
    // 3. Try CARGO_MANIFEST_DIR (cargo run)
    if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        let dir = std::path::PathBuf::from(manifest).join("docs/commands");
        if dir.is_dir() {
            return Some(dir);
        }
    }
    None
}

/// Load full documentation from docs/commands/<topic>.md
fn load_full_doc(topic: &str, output: OutputFormat) -> Result<()> {
    let docs_dir = find_docs_dir()
        .ok_or_else(|| anyhow::anyhow!("docs/commands/ directory not found. Run from the agcli repo root or install docs alongside the binary."))?;

    // Normalize topic to filename: "commit-reveal" → "commit-reveal", "stake" → "stake"
    let normalized = topic.to_lowercase().replace('_', "-");
    let path = docs_dir.join(format!("{}.md", normalized));
    if !path.exists() {
        // Try common aliases: if topic matches a command group, use that
        let aliases = [
            ("cr", "weights"),
            ("dtao", "subnet"),
            ("amm", "subnet"),
            ("nominate", "delegate"),
            ("delegation", "delegate"),
        ];
        for (alias, target) in &aliases {
            if normalized == *alias {
                let alt = docs_dir.join(format!("{}.md", target));
                if alt.exists() {
                    return load_doc_file(&alt, topic, output);
                }
            }
        }
        // List available docs
        let available = list_doc_files(&docs_dir);
        if output.is_json() {
            eprint_json(&serde_json::json!({
                "error": true,
                "message": format!("No full doc for '{}'. Available: {}", topic, available.join(", ")),
            }));
        } else {
            eprintln!(
                "No full documentation for '{}'. Available doc files:",
                topic
            );
            for name in &available {
                eprintln!("  {}", name);
            }
        }
        anyhow::bail!("No full doc for '{}'", topic);
    }
    load_doc_file(&path, topic, output)
}

fn load_doc_file(path: &std::path::Path, topic: &str, output: OutputFormat) -> Result<()> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    if output.is_json() {
        print_json(&serde_json::json!({
            "topic": topic,
            "source": path.display().to_string(),
            "content": content,
        }));
    } else {
        println!("{}", content);
    }
    Ok(())
}

fn list_doc_files(dir: &std::path::Path) -> Vec<String> {
    let mut names = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.path().file_stem() {
                names.push(name.to_string_lossy().to_string());
            }
        }
    }
    names.sort();
    names
}

/// List all available full doc files.
fn list_full_docs(output: OutputFormat) -> Result<()> {
    let docs_dir =
        find_docs_dir().ok_or_else(|| anyhow::anyhow!("docs/commands/ directory not found."))?;
    let names = list_doc_files(&docs_dir);
    if output.is_json() {
        print_json(&serde_json::json!(names));
    } else {
        println!("Available full documentation files:\n");
        for name in &names {
            println!("  {:<20} docs/commands/{}.md", name, name);
        }
        println!("\nUsage: agcli explain --topic <name> --full");
    }
    Ok(())
}

pub(super) async fn handle_utils(
    cmd: crate::cli::UtilsCommands,
    network: &crate::types::Network,
    output: OutputFormat,
    client: Option<&crate::chain::Client>,
) -> Result<()> {
    use crate::cli::UtilsCommands;
    match cmd {
        UtilsCommands::Convert {
            amount,
            to_rao,
            tao,
            alpha,
            netuid,
        } => {
            // Alpha↔TAO conversion (requires chain connection for price)
            if let Some(tao_amount) = tao {
                let nid = netuid.ok_or_else(|| {
                    anyhow::anyhow!("--netuid is required for TAO↔Alpha conversion")
                })?;
                let c = client.ok_or_else(|| anyhow::anyhow!("Chain connection required"))?;
                let (alpha_out, _, _) = c
                    .sim_swap_tao_for_alpha(crate::types::NetUid(nid), (tao_amount * 1e9) as u64)
                    .await?;
                let alpha_display = alpha_out as f64 / 1e9;
                if output.is_json() {
                    print_json(&serde_json::json!({
                        "netuid": nid,
                        "tao_in": tao_amount,
                        "alpha_out": alpha_display,
                    }));
                } else {
                    println!(
                        "{:.4} TAO → {:.4} Alpha (SN{})",
                        tao_amount, alpha_display, nid
                    );
                }
                return Ok(());
            }
            if let Some(alpha_amount) = alpha {
                let nid = netuid.ok_or_else(|| {
                    anyhow::anyhow!("--netuid is required for TAO↔Alpha conversion")
                })?;
                let c = client.ok_or_else(|| anyhow::anyhow!("Chain connection required"))?;
                let (tao_out, _, _) = c
                    .sim_swap_alpha_for_tao(crate::types::NetUid(nid), (alpha_amount * 1e9) as u64)
                    .await?;
                let tao_display = tao_out as f64 / 1e9;
                if output.is_json() {
                    print_json(&serde_json::json!({
                        "netuid": nid,
                        "alpha_in": alpha_amount,
                        "tao_out": tao_display,
                    }));
                } else {
                    println!(
                        "{:.4} Alpha (SN{}) → {:.4} TAO",
                        alpha_amount, nid, tao_display
                    );
                }
                return Ok(());
            }
            // TAO↔RAO conversion
            let amount = amount.unwrap_or(0.0);
            if to_rao {
                let rao = (amount * 1e9) as u64;
                if output.is_json() {
                    print_json(&serde_json::json!({
                        "tao": amount,
                        "rao": rao,
                    }));
                } else {
                    println!("{} TAO = {} RAO", amount, rao);
                }
            } else {
                let tao = amount / 1e9;
                if output.is_json() {
                    print_json(&serde_json::json!({
                        "rao": amount as u64,
                        "tao": tao,
                    }));
                } else {
                    println!("{} RAO = {:.9} TAO", amount as u64, tao);
                }
            }
            Ok(())
        }
        UtilsCommands::Latency { extra, pings } => {
            use crate::chain::Client;
            use std::time::Instant;

            let mut endpoints: Vec<(String, String)> = Vec::new();
            // Add standard network endpoints
            let urls = network.ws_urls();
            for url in &urls {
                endpoints.push((format!("{}", network), url.to_string()));
            }
            // Add extra endpoints
            if let Some(ref extra_str) = extra {
                for url in extra_str.split(',') {
                    let url = url.trim().to_string();
                    if !url.is_empty() {
                        endpoints.push(("custom".to_string(), url));
                    }
                }
            }

            if endpoints.is_empty() {
                anyhow::bail!("No endpoints to test");
            }

            println!(
                "Testing {} endpoint(s) with {} pings each...\n",
                endpoints.len(),
                pings
            );

            #[derive(serde::Serialize)]
            struct EndpointResult {
                label: String,
                url: String,
                connected: bool,
                avg_ms: Option<u128>,
                min_ms: Option<u128>,
                max_ms: Option<u128>,
                failures: u32,
            }

            let mut results: Vec<EndpointResult> = Vec::new();

            for (label, url) in &endpoints {
                let custom_network = crate::types::Network::Custom(url.clone());
                let connect_start = Instant::now();
                match Client::connect_network(&custom_network).await {
                    Ok(client) => {
                        let connect_ms = connect_start.elapsed().as_millis();
                        let mut latencies = Vec::new();
                        let mut failures = 0u32;
                        for _ in 0..pings {
                            let t = Instant::now();
                            match client.get_block_number().await {
                                Ok(_) => latencies.push(t.elapsed().as_millis()),
                                Err(_) => failures += 1,
                            }
                        }
                        if latencies.is_empty() {
                            results.push(EndpointResult {
                                label: label.clone(),
                                url: url.clone(),
                                connected: true,
                                avg_ms: None,
                                min_ms: None,
                                max_ms: None,
                                failures,
                            });
                            println!("{:<12} {}", label, url);
                            println!("  Connect: {}ms, pings: all {} failed\n", connect_ms, pings);
                        } else {
                            let avg = latencies.iter().sum::<u128>() / latencies.len() as u128;
                            let min = latencies.iter().copied().min().unwrap_or_default();
                            let max = latencies.iter().copied().max().unwrap_or_default();
                            results.push(EndpointResult {
                                label: label.clone(),
                                url: url.clone(),
                                connected: true,
                                avg_ms: Some(avg),
                                min_ms: Some(min),
                                max_ms: Some(max),
                                failures,
                            });
                            if !output.is_json() {
                                println!("{:<12} {}", label, url);
                                let fail_note = if failures > 0 {
                                    format!(" ({} failed)", failures)
                                } else {
                                    String::new()
                                };
                                println!(
                                    "  Connect: {}ms | avg: {}ms | min: {}ms | max: {}ms{}\n",
                                    connect_ms, avg, min, max, fail_note
                                );
                            }
                        }
                    }
                    Err(e) => {
                        results.push(EndpointResult {
                            label: label.clone(),
                            url: url.clone(),
                            connected: false,
                            avg_ms: None,
                            min_ms: None,
                            max_ms: None,
                            failures: pings as u32,
                        });
                        if !output.is_json() {
                            println!("{:<12} {}", label, url);
                            println!("  FAILED to connect: {}\n", e);
                        }
                    }
                }
            }

            if output.is_json() {
                print_json(&serde_json::json!({"latency": results}));
            }

            Ok(())
        }
    }
}

pub(super) async fn handle_update() -> Result<()> {
    println!("Updating agcli from GitHub...");
    let status = std::process::Command::new("cargo")
        .args([
            "install",
            "--git",
            "https://github.com/unconst/agcli",
            "--force",
        ])
        .status();
    match status {
        Ok(s) if s.success() => {
            println!("agcli updated successfully!");
            Ok(())
        }
        Ok(s) => anyhow::bail!("Update failed with exit code: {}", s),
        Err(e) => anyhow::bail!(
            "Failed to run cargo install: {}. Make sure cargo is installed.",
            e
        ),
    }
}

pub(super) async fn handle_batch(
    client: &Client,
    pair: &sp_core::sr25519::Pair,
    file_path: &str,
    no_atomic: bool,
    force: bool,
    output: OutputFormat,
) -> Result<()> {
    let content = std::fs::read_to_string(file_path)
        .map_err(|e| anyhow::anyhow!("Failed to read batch file '{}': {}", file_path, e))?;
    let calls = validate_batch_file(&content, file_path)?;

    let mode_name = if force {
        "force_batch (non-atomic, continues on failure)"
    } else if no_atomic {
        "batch (non-atomic)"
    } else {
        "batch_all (atomic)"
    };
    eprintln!("Batch: {} calls, mode={}", calls.len(), mode_name);

    let mut encoded_calls: Vec<Vec<u8>> = Vec::with_capacity(calls.len());
    for (i, call_json) in calls.iter().enumerate() {
        let pallet = call_json
            .get("pallet")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Call #{}: missing \"pallet\" field", i))?;
        let call_name = call_json
            .get("call")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Call #{}: missing \"call\" field", i))?;
        let args = call_json
            .get("args")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("Call #{}: missing \"args\" array", i))?;

        let fields: Vec<subxt::dynamic::Value> = args.iter().map(json_to_subxt_value).collect();

        let tx = subxt::dynamic::tx(pallet, call_name, fields);
        let encoded = client.subxt().tx().call_data(&tx).map_err(|e| {
            anyhow::anyhow!(
                "Call #{} ({}.{}): encoding failed: {}",
                i,
                pallet,
                call_name,
                e
            )
        })?;
        eprintln!(
            "  #{}: {}.{} ({} bytes)",
            i,
            pallet,
            call_name,
            encoded.len()
        );
        encoded_calls.push(encoded);
    }

    // Build Utility.batch_all, Utility.batch, or Utility.force_batch
    let batch_call_name = if force {
        "force_batch"
    } else if no_atomic {
        "batch"
    } else {
        "batch_all"
    };
    let call_values: Vec<subxt::dynamic::Value> = encoded_calls
        .iter()
        .map(|c| subxt::dynamic::Value::from_bytes(c.clone()))
        .collect();

    let batch_tx = subxt::dynamic::tx(
        "Utility",
        batch_call_name,
        vec![subxt::dynamic::Value::unnamed_composite(call_values)],
    );

    let hash = client.sign_submit_dyn(&batch_tx, pair).await?;
    print_tx_result(
        output,
        &hash,
        &format!("Batch ({} calls) submitted.", calls.len()),
    );
    Ok(())
}

pub(super) async fn handle_doctor(
    network: &crate::types::Network,
    wallet_dir: &str,
    wallet_name: &str,
    output: OutputFormat,
) -> Result<()> {
    use std::time::Instant;

    let mut checks: Vec<(&str, String, bool)> = Vec::new();

    // 1. Version
    let version = env!("CARGO_PKG_VERSION");
    checks.push(("Version", format!("agcli v{}", version), true));

    // 2. Network
    let urls = network.ws_urls();
    checks.push((
        "Network",
        format!(
            "{} ({} endpoint{})",
            network,
            urls.len(),
            if urls.len() > 1 { "s" } else { "" }
        ),
        true,
    ));

    // 3. Connectivity test
    let conn_start = Instant::now();
    let client_result = Client::connect_network(network).await;
    let conn_elapsed = conn_start.elapsed();

    match &client_result {
        Ok(_) => {
            checks.push((
                "Connection",
                format!("OK ({:.0}ms)", conn_elapsed.as_millis()),
                true,
            ));
        }
        Err(e) => {
            checks.push(("Connection", format!("FAILED: {}", e), false));
        }
    }

    // 4. Chain queries (only if connected)
    if let Ok(ref client) = client_result {
        // Block number
        let t = Instant::now();
        match client.get_block_number().await {
            Ok(block) => {
                checks.push((
                    "Block height",
                    format!("{} ({:.0}ms)", block, t.elapsed().as_millis()),
                    true,
                ));
            }
            Err(e) => {
                checks.push(("Block height", format!("FAILED: {}", e), false));
            }
        }

        // Total subnets
        let t = Instant::now();
        match client.get_total_networks().await {
            Ok(n) => {
                checks.push((
                    "Subnets",
                    format!("{} ({:.0}ms)", n, t.elapsed().as_millis()),
                    true,
                ));
            }
            Err(e) => {
                checks.push(("Subnets", format!("FAILED: {}", e), false));
            }
        }

        // Latency test: 3 quick block queries
        let mut latencies = Vec::new();
        let mut rpc_failures = 0u32;
        for _ in 0..3 {
            let t = Instant::now();
            match client.get_block_number().await {
                Ok(_) => latencies.push(t.elapsed().as_millis()),
                Err(_) => rpc_failures += 1,
            }
        }
        if latencies.is_empty() {
            checks.push((
                "Latency (3 pings)",
                format!("FAILED: all {} RPC calls failed", rpc_failures),
                false,
            ));
        } else {
            let avg: u128 = latencies.iter().sum::<u128>() / latencies.len() as u128;
            let min = latencies.iter().min().copied().unwrap_or_default();
            let max = latencies.iter().max().copied().unwrap_or_default();
            let fail_note = if rpc_failures > 0 {
                format!("  ({} failed)", rpc_failures)
            } else {
                String::new()
            };
            checks.push((
                "Latency (3 pings)",
                format!("avg {avg}ms  min {min}ms  max {max}ms{fail_note}"),
                rpc_failures == 0,
            ));
        }
    }

    // 5. Disk cache
    let cache_keys = crate::queries::disk_cache::list_keys();
    let cache_path = crate::queries::disk_cache::path();
    if cache_keys.is_empty() {
        checks.push((
            "Disk cache",
            format!("empty ({})", cache_path.display()),
            true,
        ));
    } else {
        // Calculate total size
        let mut total_bytes: u64 = 0;
        for key in &cache_keys {
            let path = cache_path.join(format!("{}.json", key));
            if let Ok(meta) = std::fs::metadata(&path) {
                total_bytes += meta.len();
            }
        }
        let size_kb = total_bytes as f64 / 1024.0;
        checks.push((
            "Disk cache",
            format!(
                "{} entries, {:.1}KB ({})",
                cache_keys.len(),
                size_kb,
                cache_path.display()
            ),
            true,
        ));
    }

    // 6. Wallet check
    let wallet_path = format!("{}/{}", wallet_dir, wallet_name);
    match crate::wallet::Wallet::open(&wallet_path) {
        Ok(w) => {
            let has_coldkey = w.coldkey_ss58().is_some();
            let hotkeys = w.list_hotkeys().unwrap_or_default();
            checks.push((
                "Wallet",
                format!(
                    "'{}' (coldkey: {}, hotkeys: {})",
                    wallet_name,
                    if has_coldkey { "present" } else { "missing" },
                    hotkeys.len()
                ),
                has_coldkey,
            ));
        }
        Err(_) => {
            checks.push((
                "Wallet",
                format!("'{}' not found at {}", wallet_name, wallet_path),
                false,
            ));
        }
    }

    // Output
    if output.is_json() {
        let items: Vec<serde_json::Value> = checks
            .iter()
            .map(
                |(name, detail, ok)| serde_json::json!({"check": name, "detail": detail, "ok": ok}),
            )
            .collect();
        print_json(&serde_json::json!({"doctor": items}));
    } else {
        println!("agcli doctor");
        println!("{}", "-".repeat(60));
        for (name, detail, ok) in &checks {
            let status = if *ok { "OK" } else { "FAIL" };
            println!("  [{:>4}] {:<20} {}", status, name, detail);
        }
        println!("{}", "-".repeat(60));
        let failed = checks.iter().filter(|(_, _, ok)| !ok).count();
        if failed == 0 {
            println!("  All checks passed.");
        } else {
            println!("  {} check(s) failed.", failed);
        }
    }

    Ok(())
}
