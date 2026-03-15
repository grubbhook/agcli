use clap::Parser;
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() {
    let mut cli = agcli::cli::Cli::parse();

    // Configure logging level: --debug > --verbose > RUST_LOG > warn
    let filter = if cli.debug {
        tracing_subscriber::EnvFilter::new("debug")
    } else if cli.verbose {
        tracing_subscriber::EnvFilter::new("agcli=info,warn")
    } else {
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn"))
    };

    // Set up logging: stderr always, file optionally
    if let Some(ref log_path) = cli.log_file {
        let path = std::path::Path::new(log_path);
        let dir = path.parent().unwrap_or(std::path::Path::new("."));
        let filename = path
            .file_name()
            .unwrap_or(std::ffi::OsStr::new("agcli.log"))
            .to_string_lossy();

        let file_appender = tracing_appender::rolling::daily(dir, filename.as_ref());
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

        // Keep _guard alive for the duration of main
        let _file_guard = _guard;

        tracing_subscriber::registry()
            .with(filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(non_blocking)
                    .with_ansi(false),
            )
            .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .init();
    }

    // Load config file and apply defaults (CLI flags take precedence)
    let cfg = agcli::Config::load();
    cli.apply_config(&cfg);

    // Warn if password was passed via CLI flag (visible in process list)
    if cli.password.is_some() && std::env::var("AGCLI_PASSWORD").is_err() {
        eprintln!(
            "Warning: password passed via --password flag is visible in `ps`. \
             Prefer AGCLI_PASSWORD env var for security."
        );
    }

    let json_errors = cli.output.is_json() || cli.batch;
    let show_time = cli.time;
    let timeout_secs = cli.timeout.filter(|&t| t > 0);

    let start = std::time::Instant::now();

    // Wrap execution in an optional timeout
    let result = if let Some(secs) = timeout_secs {
        match tokio::time::timeout(
            std::time::Duration::from_secs(secs),
            agcli::cli::commands::execute(cli),
        )
        .await
        {
            Ok(r) => r,
            Err(_) => Err(anyhow::anyhow!(
                "Operation timed out after {}s (--timeout {})",
                secs,
                secs
            )),
        }
    } else {
        agcli::cli::commands::execute(cli).await
    };

    let elapsed = start.elapsed();
    if show_time {
        eprintln!("[time] {:.3}s", elapsed.as_secs_f64());
    }

    match result {
        Ok(()) => std::process::exit(0),
        Err(e) => {
            let code = agcli::error::classify(&e);
            let msg = format!("{:#}", e);
            if json_errors {
                // Structured error JSON on stderr for agents
                let mut payload = serde_json::json!({
                    "error": true,
                    "code": code,
                    "message": &msg,
                });
                if let Some(hint) = agcli::error::hint(code, &msg) {
                    payload["hint"] = serde_json::Value::String(hint.to_string());
                }
                agcli::cli::helpers::eprint_json(&payload);
            } else {
                eprintln!("Error: {}", msg);
                if let Some(hint) = agcli::error::hint(code, &msg) {
                    eprintln!("{}", hint);
                }
            }
            std::process::exit(code);
        }
    }
}
