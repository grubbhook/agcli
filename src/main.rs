use clap::Parser;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();

    let mut cli = agcli::cli::Cli::parse();

    // Load config file and apply defaults (CLI flags take precedence)
    let cfg = agcli::Config::load();
    cli.apply_config(&cfg);

    let json_errors = cli.output == "json" || cli.batch;

    match agcli::cli::commands::execute(cli).await {
        Ok(()) => std::process::exit(0),
        Err(e) => {
            if json_errors {
                // Structured error JSON on stderr for agents
                eprintln!("{}", serde_json::json!({
                    "error": true,
                    "message": format!("{:#}", e),
                }));
            } else {
                eprintln!("Error: {:#}", e);
            }
            std::process::exit(1);
        }
    }
}
