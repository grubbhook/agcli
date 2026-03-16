# agcli — Rust CLI + SDK for Bittensor

[![CI](https://github.com/unconst/agcli/actions/workflows/ci.yml/badge.svg)](https://github.com/unconst/agcli/actions/workflows/ci.yml)

Fast, safe Rust toolkit for the [Bittensor](https://bittensor.com) network. Wallets, staking, transfers, subnets, weights, metagraph queries, Dynamic TAO, monitoring, and more.

## Install

```bash
cargo install --git https://github.com/unconst/agcli
```

## Quick Examples

```bash
# Check balance
agcli balance --address 5Gx...

# List subnets as JSON
agcli --output json subnet list

# Stake TAO with slippage protection
agcli stake add --amount 10 --netuid 1 --max-slippage 2.0 --password p --yes

# Atomic commit-reveal weights
agcli weights commit-reveal --netuid 1 --weights "0:100,1:200" --wait

# Live subnet monitoring (JSON streaming)
agcli subnet monitor --netuid 97 --json
```

Every command supports `--output json|csv`, `--yes` (skip prompts), `--batch` (hard-error mode), and `--dry-run` (preview). Full non-interactive operation for AI agents.

## Documentation

| Resource | Description |
|----------|-------------|
| **[docs/why-agcli.md](docs/why-agcli.md)** | Why agcli? Comparison with btcli and the Bittensor Python SDK |
| **[docs/llm.txt](docs/llm.txt)** | Agent/LLM reference — quick-ref card + full command reference |
| **[docs/commands/](docs/commands/)** | Per-command deep dives — on-chain behavior, pallet refs, storage keys, events, errors |
| **[docs/tutorials/](docs/tutorials/)** | Step-by-step guides: [getting started](docs/tutorials/getting-started.md), [staking](docs/tutorials/staking-guide.md), [validator](docs/tutorials/validator-guide.md), [subnet builder](docs/tutorials/subnet-builder.md), [agent automation](docs/tutorials/agent-automation.md) |

## SDK Usage

```toml
[dependencies]
agcli = { git = "https://github.com/unconst/agcli", default-features = false, features = ["sdk-only"] }
```

```rust
use agcli::{Client, Wallet, Balance};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::connect("wss://entrypoint-finney.opentensor.ai:443").await?;
    let balance = client.get_balance_ss58("5Gx...").await?;
    let subnets = client.get_all_subnets().await?;
    let metagraph = client.get_metagraph(1.into()).await?;
    Ok(())
}
```

## Architecture

```
agcli/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # SDK re-exports (Client, Wallet, Balance, Config)
│   ├── config.rs            # Persistent config (~/.agcli/config.toml)
│   ├── error.rs             # Error classification + exit codes
│   ├── events.rs            # Real-time block/event subscription
│   ├── live.rs              # Live polling with delta tracking
│   ├── chain/
│   │   ├── mod.rs           # Client: connection, retry, 40+ queries + extrinsics
│   │   ├── queries.rs       # Chain query methods
│   │   ├── extrinsics.rs    # Transaction builders
│   │   └── rpc_types.rs     # Type conversions
│   ├── cli/
│   │   ├── mod.rs           # Clap parser: 18 command groups, 80+ subcommands
│   │   ├── commands.rs      # Main dispatcher
│   │   ├── helpers.rs       # Shared CLI helpers
│   │   ├── subnet_cmds.rs   # Subnet operations
│   │   ├── view_cmds.rs     # View/query handlers
│   │   ├── stake_cmds.rs    # Staking operations
│   │   ├── weights_cmds.rs  # Weight setting + commit-reveal
│   │   ├── wallet_cmds.rs   # Wallet management
│   │   ├── block_cmds.rs    # Block explorer
│   │   ├── network_cmds.rs  # Network queries + commitment commands
│   │   └── system_cmds.rs   # Config, proxy, delegate, identity
│   ├── wallet/              # Key management (Python wallet compat)
│   ├── types/               # Balance, NeuronInfo, SubnetInfo, etc.
│   ├── queries/             # Cache layer (Moka + disk)
│   ├── extrinsics/          # Weight hashing, MEV shield
│   └── utils/               # Explain, format, POW solver
├── docs/
│   ├── llm.txt              # Agent-optimized reference
│   ├── commands/             # 22 per-command docs
│   └── tutorials/            # 5 step-by-step guides
├── tests/                    # 7 integration test files
├── build.rs                  # Compile-time chain metadata fetch
└── Cargo.toml
```

## Building

Requires Rust 1.75+ and network access (fetches chain metadata at build time):

```bash
git clone https://github.com/unconst/agcli && cd agcli && cargo build --release
```

## License

MIT
