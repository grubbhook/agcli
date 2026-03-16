//! Admin command handlers — sudo AdminUtils hyperparameter management.

use crate::admin;
use crate::chain::Client;
use crate::cli::helpers::*;
use crate::cli::AdminCommands;
use anyhow::Result;
use sp_core::{sr25519, Pair as _};
use subxt::dynamic::Value;

/// Resolve a sudo keypair from a URI string (e.g. "//Alice") or from the wallet.
fn resolve_sudo_key(
    sudo_key: &Option<String>,
    ctx: &Ctx<'_>,
) -> Result<sr25519::Pair> {
    if let Some(ref uri) = sudo_key {
        // Try as dev URI first (//Alice, //Bob, etc.)
        match sr25519::Pair::from_string(uri, None) {
            Ok(pair) => return Ok(pair),
            Err(_) => {
                anyhow::bail!(
                    "Invalid sudo key URI '{}'. Use a dev URI like //Alice or //Bob.",
                    uri
                );
            }
        }
    }
    // Fall back to wallet coldkey
    let mut wallet = open_wallet(ctx.wallet_dir, ctx.wallet_name)?;
    unlock_coldkey(&mut wallet, ctx.password)?;
    Ok(wallet.coldkey()?.clone())
}

pub(super) async fn handle_admin(
    cmd: AdminCommands,
    client: &Client,
    ctx: &Ctx<'_>,
) -> Result<()> {
    match cmd {
        AdminCommands::SetTempo {
            netuid,
            tempo,
            sudo_key,
        } => {
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash = admin::set_tempo(client, &pair, netuid, tempo).await?;
            print_tx_result(ctx.output, &hash, &format!("Tempo set to {} on SN{}", tempo, netuid));
            Ok(())
        }

        AdminCommands::SetMaxValidators {
            netuid,
            max,
            sudo_key,
        } => {
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash = admin::set_max_allowed_validators(client, &pair, netuid, max).await?;
            print_tx_result(
                ctx.output,
                &hash,
                &format!("Max validators set to {} on SN{}", max, netuid),
            );
            Ok(())
        }

        AdminCommands::SetMaxUids {
            netuid,
            max,
            sudo_key,
        } => {
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash = admin::set_max_allowed_uids(client, &pair, netuid, max).await?;
            print_tx_result(
                ctx.output,
                &hash,
                &format!("Max UIDs set to {} on SN{}", max, netuid),
            );
            Ok(())
        }

        AdminCommands::SetImmunityPeriod {
            netuid,
            period,
            sudo_key,
        } => {
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash = admin::set_immunity_period(client, &pair, netuid, period).await?;
            print_tx_result(
                ctx.output,
                &hash,
                &format!("Immunity period set to {} on SN{}", period, netuid),
            );
            Ok(())
        }

        AdminCommands::SetMinWeights {
            netuid,
            min,
            sudo_key,
        } => {
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash = admin::set_min_allowed_weights(client, &pair, netuid, min).await?;
            print_tx_result(
                ctx.output,
                &hash,
                &format!("Min weights set to {} on SN{}", min, netuid),
            );
            Ok(())
        }

        AdminCommands::SetMaxWeightLimit {
            netuid,
            limit,
            sudo_key,
        } => {
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash = admin::set_max_weight_limit(client, &pair, netuid, limit).await?;
            print_tx_result(
                ctx.output,
                &hash,
                &format!("Max weight limit set to {} on SN{}", limit, netuid),
            );
            Ok(())
        }

        AdminCommands::SetWeightsRateLimit {
            netuid,
            limit,
            sudo_key,
        } => {
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash =
                admin::set_weights_set_rate_limit(client, &pair, netuid, limit).await?;
            print_tx_result(
                ctx.output,
                &hash,
                &format!("Weights rate limit set to {} on SN{}", limit, netuid),
            );
            Ok(())
        }

        AdminCommands::SetCommitReveal {
            netuid,
            enabled,
            sudo_key,
        } => {
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash = admin::set_commit_reveal_weights_enabled(client, &pair, netuid, enabled)
                .await?;
            let state = if enabled { "enabled" } else { "disabled" };
            print_tx_result(
                ctx.output,
                &hash,
                &format!("Commit-reveal {} on SN{}", state, netuid),
            );
            Ok(())
        }

        AdminCommands::SetDifficulty {
            netuid,
            difficulty,
            sudo_key,
        } => {
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash = admin::set_difficulty(client, &pair, netuid, difficulty).await?;
            print_tx_result(
                ctx.output,
                &hash,
                &format!("Difficulty set to {} on SN{}", difficulty, netuid),
            );
            Ok(())
        }

        AdminCommands::SetActivityCutoff {
            netuid,
            cutoff,
            sudo_key,
        } => {
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash = admin::set_activity_cutoff(client, &pair, netuid, cutoff).await?;
            print_tx_result(
                ctx.output,
                &hash,
                &format!("Activity cutoff set to {} on SN{}", cutoff, netuid),
            );
            Ok(())
        }

        AdminCommands::Raw {
            call,
            args,
            sudo_key,
        } => {
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            // Parse args as JSON array of values
            let values = parse_raw_args(&args)?;
            let hash = admin::raw_admin_call(client, &pair, &call, values).await?;
            print_tx_result(ctx.output, &hash, &format!("AdminUtils.{} executed", call));
            Ok(())
        }

        AdminCommands::List => {
            let params = admin::known_params();
            if ctx.output.is_json() {
                let items: Vec<_> = params
                    .iter()
                    .map(|(name, desc, args)| {
                        serde_json::json!({
                            "call": name,
                            "description": desc,
                            "args": args,
                        })
                    })
                    .collect();
                print_json(&serde_json::json!(items));
            } else {
                println!("Available AdminUtils parameters:\n");
                for (name, desc, args) in &params {
                    println!("  {} — {}", name, desc);
                    println!("    args: {}", args.join(", "));
                    println!();
                }
                println!("Use `agcli admin raw --call <name> --args '[...]' --sudo-key //Alice` for any call.");
            }
            Ok(())
        }
    }
}

/// Parse a JSON array string into dynamic Values.
/// Accepts: '[1, 2, true]' or '[]' or individual values.
fn parse_raw_args(args: &str) -> Result<Vec<Value>> {
    let parsed: serde_json::Value = serde_json::from_str(args)
        .map_err(|e| anyhow::anyhow!("Invalid JSON args '{}': {}", args, e))?;

    match parsed {
        serde_json::Value::Array(arr) => arr.iter().map(json_to_value).collect(),
        _ => anyhow::bail!("Args must be a JSON array, got: {}", args),
    }
}

fn json_to_value(v: &serde_json::Value) -> Result<Value> {
    match v {
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_u64() {
                Ok(Value::u128(i as u128))
            } else if let Some(i) = n.as_i64() {
                Ok(Value::u128(i as u128))
            } else {
                anyhow::bail!("Unsupported number type: {}", n)
            }
        }
        serde_json::Value::Bool(b) => Ok(Value::bool(*b)),
        serde_json::Value::String(s) => Ok(Value::string(s.clone())),
        _ => anyhow::bail!("Unsupported arg type: {}", v),
    }
}
