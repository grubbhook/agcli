# agcli — Agent optimized Rust CLI + SDK for Bittensor

[![CI](https://github.com/unconst/agcli/actions/workflows/ci.yml/badge.svg)](https://github.com/unconst/agcli/actions/workflows/ci.yml)

A fast, safe Rust toolkit for interacting with the [Bittensor](https://bittensor.com) network.
Covers everything: wallet management, staking, transfers, subnet operations, weight setting, registration, metagraph queries, and more.

## Features

| Category | Capabilities |
|---|---|
| **Wallet** | Create, import (mnemonic/seed), encrypt/decrypt coldkeys, manage multiple hotkeys, sign/verify messages, derive addresses, **Python wallet compat** (NaCl SecretBox keyfiles) |
| **Staking** | Add/remove stake, move between subnets, swap between hotkeys, limit orders, swap-limit, claim root dividends, recycle alpha, unstake-all-alpha, burn alpha, **slippage protection** (`--max-slippage`), **spending limits** |
| **Transfers** | Send TAO, transfer-all (full balance minus fees) |
| **Subnets** | List subnets with real names, view metagraph (full or single UID), register neurons (burn/POW), create/dissolve subnets, hyperparameters |
| **Subnet Ops** | **Live watch** (tempo countdown, rate limits), **liquidity dashboard** (AMM depth, slippage), **monitor** (UID tracking, anomaly detection), **health/emissions/cost** dashboards |
| **Dynamic TAO** | Real-time subnet pricing, TAO/Alpha pool balances, emission breakdown, subnet volume |
| **Weights** | Set weights, commit-reveal, reveal, **atomic commit-reveal** (commit+wait+auto-reveal), **dry-run** pre-flight checks, **batch set/commit/reveal** |
| **Delegates** | View delegates, manage take rates, childkey delegation |
| **Identity** | Query on-chain identity (Registry pallet), set/view subnet identity (SubnetIdentitiesV3) |
| **Queries** | Portfolio view (with prices), neuron info, network overview, dynamic info |
| **Live Mode** | `--live` polling with delta tracking for dynamic, metagraph, portfolio |
| **Events** | Real-time block/event subscription with filtering by category, **netuid**, **account** |
| **Balance Watch** | `--watch N --threshold T` — low-balance monitoring with JSON alerts |
| **Explain** | Built-in Bittensor concept reference (29 topics: tempo, commit-reveal, AMM, alpha, emission, registration, governance, archive, etc.) |
| **Archive** | Historical queries via archive node (`--network archive`), `--at-block N` on 12 commands |
| **Block** | Block explorer: `block info --number N`, `block latest` (hash, timestamp, extrinsics) |
| **Key Swaps** | Hotkey swap, coldkey swap (scheduled) |
| **Root** | Root registration, root weights |
| **Raw Calls** | Submit to any pallet via dynamic dispatch (EVM, MEV Shield, Contracts) |
| **Config** | Persistent settings (`~/.agcli/config.toml`), batch mode, spending limits |
| **Proxy** | Wrap any extrinsic through Proxy.proxy, add/remove/list proxy accounts |
| **Serve** | Set axon endpoint (IP/port/protocol) for miners on subnets |
| **Multisig** | Derive multisig address, submit/approve multisig calls |
| **Validators** | Top validators overview by stake (per-subnet or global) |
| **History** | Transaction history via Subscan API |
| **Account** | Detailed account explorer — balance, stakes, identity, delegate info |
| **Analytics** | Subnet analytics (miner/validator stats, economics) + staking analytics (APY, projections) |
| **Wizard** | Interactive staking wizard — shows top subnets, guided flow |
| **Completions** | Shell completions for bash, zsh, fish, powershell |
| **Swap Sim** | Simulate TAO↔Alpha swaps with slippage/fee estimates, current alpha price |
| **Nominations** | View who nominates/delegates to a hotkey |
| **Crowdloan** | Contribute, withdraw, finalize crowdloans |
| **Batch** | Submit multiple extrinsics atomically from a JSON file (`agcli batch calls.json`) |
| **Update** | Self-update via `agcli update` (cargo install from GitHub) |
| **Agent Mode** | `--batch` (hard errors, no stdin), `--pretty` (pretty JSON), structured error JSON, spending limits, fully non-interactive |
| **Output** | Table (default), JSON (`--output json`), CSV (`--output csv`), `--pretty` for formatted JSON |

## Quick Start

### Install

```bash
cargo install --git https://github.com/unconst/agcli
```

### CLI Usage

```bash
# Check balance
agcli balance --address 5Gx...

# Check balance as JSON
agcli --output json balance --address 5Gx...

# Create a wallet (non-interactive)
agcli wallet create --name my_wallet --password mypass

# List all subnets (with real names from SubnetIdentitiesV3)
agcli subnet list

# View metagraph as JSON
agcli --output json subnet metagraph --netuid 1

# Add stake (with slippage protection)
agcli stake add --amount 10.0 --netuid 1 --hotkey 5Hx... --max-slippage 2.0

# Transfer TAO
agcli transfer --dest 5Dest... --amount 1.5

# Set weights (dry-run first, then submit)
agcli weights set --netuid 1 --weights "0:100,1:200,2:300" --dry-run
agcli weights set --netuid 1 --weights "0:100,1:200,2:300"

# Commit-reveal weights (auto waits + reveals)
agcli weights commit-reveal --netuid 1 --weights "0:100,1:200" --wait

# View portfolio (with real prices and subnet names)
agcli view portfolio

# View Dynamic TAO (prices, pools, volumes)
agcli view dynamic

# View Dynamic TAO as CSV
agcli --output csv view dynamic

# Live mode — poll dynamic prices every 12s, show deltas
agcli --live view dynamic

# Live metagraph — track neuron changes on SN1 every 30s
agcli --live 30 subnet metagraph --netuid 1

# Live portfolio — watch your portfolio in real-time
agcli --live view portfolio

# Subscribe to finalized blocks
agcli subscribe blocks

# Subscribe to all chain events (as JSON)
agcli --output json subscribe events

# Subscribe to staking events only
agcli subscribe events --filter staking

# Filter events by subnet and account
agcli subscribe events --filter all --netuid 1 --account 5Gx...

# View network info as JSON
agcli --output json view network

# POW registration (multi-threaded)
agcli subnet pow --netuid 1 --threads 8

# Set subnet identity
agcli identity set-subnet --netuid 1 --name "My Subnet" --github "user/repo"

# Query on-chain identity
agcli identity show --address 5GrwvaEF5zXb...

# Interactive staking wizard
agcli stake wizard
# Non-interactive wizard
agcli stake wizard --netuid 1 --amount 0.5 --password mypass --yes

# Configuration (persistent to ~/.agcli/config.toml)
agcli config set --key network --value finney
agcli config set --key wallet --value my_wallet
agcli config set --key output --value json
agcli config show
agcli config unset --key output
agcli config path

# Proxy — execute through a proxy account
agcli --proxy 5ProxyAccount... stake add --amount 10 --netuid 1

# View top validators (global)
agcli view validators --limit 20

# View validators on a specific subnet
agcli view validators --netuid 1

# Transaction history (via Subscan)
agcli view history --address 5Gx... --limit 10

# Multisig — derive address
agcli multisig address --signatories "5Addr1...,5Addr2...,5Addr3..." --threshold 2

# Multisig — submit a call
agcli multisig submit --others "5Addr2...,5Addr3..." --threshold 2 \
  --pallet Balances --call transfer_allow_death --args '[...]'

# Multisig — approve a pending call
agcli multisig approve --others "5Addr2...,5Addr3..." --threshold 2 \
  --call-hash 0xabcdef...

# Account explorer (balance, stakes, identity, delegate info)
agcli view account --address 5Gx...

# Subnet analytics (miners, validators, economics, top performers)
agcli view subnet-analytics --netuid 1

# Staking analytics (APY estimates, emission projections)
agcli view staking-analytics --address 5Gx...

# Transfer entire balance (minus fees)
agcli transfer-all --dest 5Dest... --keep-alive

# Serve axon endpoint (miners)
agcli serve axon --netuid 1 --ip 1.2.3.4 --port 8091

# Recycle alpha back to TAO
agcli stake recycle-alpha --amount 100.0 --netuid 1

# Dissolve a subnet (owner only)
agcli subnet dissolve --netuid 42

# Add a proxy account
agcli proxy add --delegate 5DelegateAddr... --proxy-type staking --delay 0

# Remove a proxy account
agcli proxy remove --delegate 5DelegateAddr... --proxy-type staking

# Simulate TAO→Alpha swap (with slippage + fees)
agcli view swap-sim --netuid 1 --tao 10.0

# Simulate Alpha→TAO swap
agcli view swap-sim --netuid 1 --alpha 500.0

# View who nominates to a validator
agcli view nominations --hotkey 5HotkeyAddr...

# Unstake all alpha across all subnets
agcli stake unstake-all-alpha

# Burn alpha tokens permanently
agcli stake burn-alpha --amount 100.0 --netuid 1

# Swap stake between subnets with limit price
agcli stake swap-limit --amount 100.0 --from 1 --to 2 --price 0.5 --partial

# List proxy accounts
agcli proxy list --address 5Gx...

# Crowdloan — contribute
agcli crowdloan contribute --crowdloan-id 1 --amount 10.0

# Crowdloan — withdraw contribution
agcli crowdloan withdraw --crowdloan-id 1

# Crowdloan — finalize
agcli crowdloan finalize --crowdloan-id 1

# Subnet liquidity dashboard (AMM depth, slippage estimates)
agcli subnet liquidity --netuid 1

# Subnet health (miner status, weight vs consensus)
agcli subnet health --netuid 1

# Subnet emissions (who earns what, projected next epoch)
agcli subnet emissions --netuid 1

# Subnet registration cost + trend
agcli subnet cost --netuid 1

# Live subnet watcher (tempo countdown, rate limits)
agcli subnet watch --netuid 1 --interval 12

# Subnet monitor (track registrations, anomalies)
agcli subnet monitor --netuid 1 --json

# Batch extrinsics from JSON file
agcli batch --file calls.json
agcli batch --file calls.json --no-atomic

# Self-update to latest version
agcli update

# Shell completions
agcli completions --shell bash > /etc/bash_completion.d/agcli
agcli completions --shell zsh > ~/.zfunc/_agcli
agcli completions --shell fish > ~/.config/fish/completions/agcli.fish

# Explain Bittensor concepts
agcli explain
agcli explain --topic tempo
agcli --output json explain --topic commit-reveal

# Block explorer
agcli block latest
agcli block info --number 4000000

# Historical queries (archive node)
agcli --network archive view portfolio --at-block 3500000
agcli --network archive subnet list --at-block 3000000
agcli --network archive subnet metagraph --netuid 1 --at-block 3500000
agcli --network archive view dynamic --at-block 4000000
```

### SDK Usage (as library)

Add to your `Cargo.toml`:
```toml
[dependencies]
agcli = { git = "https://github.com/unconst/agcli", default-features = false, features = ["sdk-only"] }
```

```rust
use agcli::{Client, Wallet, Balance};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Connect to finney
    let client = Client::connect("wss://entrypoint-finney.opentensor.ai:443").await?;

    // Open wallet
    let mut wallet = Wallet::open("~/.bittensor/wallets/default")?;

    // Check balance
    let balance = client.get_balance(&wallet.coldkey_public()).await?;
    println!("Balance: {}", balance.display_tao());

    // Get block number
    let block = client.get_block_number().await?;
    println!("Block: {}", block);

    // List all subnets
    let subnets = client.get_all_subnets().await?;
    println!("Subnets: {}", subnets.len());

    // Query subnet identity
    let id = client.get_subnet_identity(1.into()).await?;
    println!("SN1 name: {:?}", id.map(|i| i.subnet_name));

    // Get stake info
    let stakes = client.get_stake_for_coldkey("5Gx...").await?;
    for s in &stakes {
        println!("SN{}: {} staked", s.netuid, s.stake.display_tao());
    }

    Ok(())
}
```

## JSON Output Examples

Every data command supports `--output json` for agent/pipeline consumption:

```bash
# Balance → {"free": "1234.567", "staked": "500.0", "total": "1734.567"}
agcli --output json balance --address 5Gx...

# Subnet list → [{"netuid": 1, "name": "Prompting", "tao_in": "...", ...}]
agcli --output json subnet list

# Portfolio → {"coldkey": "5Gx...", "total_tao": "1234.56", "positions": [...]}
agcli --output json view portfolio

# Wallet list → [{"name": "default", "coldkey": "5Gx..."}]
agcli --output json wallet list

# Wallet show → [{"name": "default", "coldkey": "5Gx...", "hotkeys": [{"name": "default", "address": "5Hx..."}]}]
agcli --output json wallet show --all

# Explain topics → [{"topic": "tempo", "description": "..."}, ...]
agcli --output json explain

# Pretty-print with --pretty
agcli --output json --pretty view dynamic
```

## Architecture

```
agcli/
├── src/
│   ├── config.rs        # Persistent config file (~/.agcli/config.toml)
│   ├── lib.rs           # Library root, re-exports Client/Wallet/Balance/Config
│   ├── main.rs          # CLI entry point
│   ├── chain/           # Substrate client (subxt-based)
│   │   ├── mod.rs         # Client: 40+ queries + 40+ extrinsics + multisig + sign_submit
│   │   ├── rpc_types.rs   # Type conversions (NeuronInfo, DynamicInfo, DelegateInfo, etc.)
│   │   ├── connection.rs  # Legacy JSON-RPC transport
│   │   └── storage.rs     # Raw storage queries
│   ├── wallet/          # Wallet management
│   │   ├── keypair.rs     # SR25519 key generation, SS58 encoding
│   │   ├── keyfile.rs     # Encryption: AES-256-GCM (agcli) + NaCl SecretBox (Python compat)
│   │   └── mod.rs         # Wallet abstraction (auto-detects keyfile format)
│   ├── types/           # Core data types (Serialize/Deserialize)
│   │   ├── balance.rs     # TAO/Alpha balances with arithmetic
│   │   ├── network.rs     # Network presets (finney/test/local)
│   │   └── chain_data.rs  # NeuronInfo, SubnetInfo, StakeInfo, etc.
│   ├── extrinsics/      # Extrinsic helpers
│   │   └── weights.rs     # Weight commit hash computation
│   ├── queries/         # Composed query helpers
│   │   ├── portfolio.rs   # Full portfolio aggregation (with DynamicInfo prices)
│   │   ├── metagraph.rs   # Metagraph fetch
│   │   └── subnet.rs      # Subnet queries
│   ├── live.rs          # Live polling mode with delta tracking
│   ├── events.rs        # Real-time block/event subscription
│   ├── utils/           # Shared utilities
│   │   ├── format.rs      # SS58 truncation, weight conversion
│   │   └── pow.rs         # POW solver (multi-threaded)
│   └── cli/             # CLI definitions
│       ├── mod.rs         # Clap parser: 18 command groups, 80+ subcommands
│       ├── commands.rs    # Main dispatcher + subnet/weights/delegate/identity/swap/config/multisig
│       ├── helpers.rs     # Shared helpers (wallet open/unlock, hotkey resolve, parsers)
│       ├── wallet_cmds.rs # Wallet command handlers
│       ├── stake_cmds.rs  # Stake command handlers + staking wizard
│       └── view_cmds.rs   # View handlers (portfolio, analytics, account, history)
├── docs/
│   ├── llm.txt          # Agent-friendly docs
│   └── tutorials/
│       ├── getting-started.md
│       ├── staking-guide.md
│       └── subnet-builder.md
├── build.rs             # Fetches chain metadata for subxt codegen
├── Cargo.toml
└── README.md
```

## Bittensor Concepts

- **TAO**: Native token. 1 TAO = 1,000,000,000 RAO.
- **Subnets**: Independent networks (netuid 0-N) each running their own incentive mechanism.
- **Coldkey**: Offline signing key for high-value ops (staking, transfers). Encrypted on disk with AES-256-GCM + Argon2id (agcli) or NaCl SecretBox + Argon2i (Python bittensor-wallet compat — auto-detected).
- **Hotkey**: Online key for automated ops (weights, serving). Stored plaintext (mnemonic or JSON).
- **Neurons**: Registered entities on a subnet (miners and validators).
- **Weights**: Validators set weights to score miners, determining emission distribution.
- **Dynamic TAO**: Each subnet has its own alpha token. Staking buys alpha; emission is in alpha.
- **Root Network** (SN0): Special subnet governing emission distribution across all subnets.
- **Commit-Reveal**: Weight privacy scheme. Commit a blake2 hash, reveal later.
- **Take**: Percentage of delegated emissions that a validator keeps.

## Agent / Non-Interactive Mode

Every command is fully scriptable for AI agents and automation pipelines:

```bash
# Global flags for non-interactive operation
--yes / -y           # Skip all confirmation prompts
--password PASS      # Provide wallet password (avoids interactive prompt)
--output json/csv    # Machine-readable output

# Environment variables (alternative to flags)
AGCLI_YES=1          # Same as --yes
AGCLI_PASSWORD=pass  # Same as --password

# Examples — zero stdin blocking when all args provided:
agcli wallet create --name mywallet --password mypass --yes
agcli wallet import --name w --mnemonic "abandon ... about" --password p
agcli stake wizard --netuid 1 --amount 0.5 --password p --yes
agcli stake add --amount 10.0 --netuid 1 --password mypass --yes
AGCLI_PASSWORD=mypass agcli transfer --dest 5Dest... --amount 1.0 --yes
agcli --output json view portfolio             # JSON output for parsing
agcli --output json --pretty view dynamic      # Pretty-printed JSON

# Exit codes: 0=success, 1=error, 2=usage error
# Errors → stderr, data → stdout
```

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `AGCLI_NETWORK` | `finney` | Network preset (finney/test/local/archive) |
| `AGCLI_ENDPOINT` | — | Custom WS endpoint (overrides network) |
| `AGCLI_WALLET_DIR` | `~/.bittensor/wallets` | Wallet directory |
| `AGCLI_WALLET` | `default` | Active wallet name |
| `AGCLI_HOTKEY` | `default` | Active hotkey name |
| `AGCLI_PASSWORD` | — | Wallet password (non-interactive unlock) |
| `AGCLI_YES` | — | Skip all confirmation prompts |
| `METADATA_CHAIN_ENDPOINT` | finney | Chain endpoint for build-time metadata fetch |

## Building

Requires Rust 1.75+ and network access (fetches chain metadata at build time):

```bash
git clone https://github.com/unconst/agcli
cd agcli
cargo build --release
```

## License

MIT
