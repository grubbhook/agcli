# agcli — Agent optimized Rust CLI + SDK for Bittensor

A fast, safe Rust toolkit for interacting with the [Bittensor](https://bittensor.com) network.
Covers everything: wallet management, staking, transfers, subnet operations, weight setting, registration, metagraph queries, and more.

## Features

| Category | Capabilities |
|---|---|
| **Wallet** | Create, import (mnemonic/seed), encrypt/decrypt coldkeys, manage multiple hotkeys, **Python wallet compat** (NaCl SecretBox keyfiles) |
| **Staking** | Add/remove stake, move between subnets, swap between hotkeys, limit orders, claim root dividends, recycle alpha |
| **Transfers** | Send TAO, transfer-all (full balance minus fees) |
| **Subnets** | List subnets with real names, view metagraph, register neurons (burn/POW), create/dissolve subnets, hyperparameters |
| **Dynamic TAO** | Real-time subnet pricing, TAO/Alpha pool balances, emission breakdown, subnet volume |
| **Weights** | Set weights, commit-reveal with blake2 hashing, reveal operations, **batch set/commit/reveal** |
| **Delegates** | View delegates, manage take rates, childkey delegation |
| **Identity** | Query on-chain identity (Registry pallet), set/view subnet identity (SubnetIdentitiesV3) |
| **Queries** | Portfolio view (with prices), neuron info, network overview, dynamic info |
| **Live Mode** | `--live` polling with delta tracking for dynamic, metagraph, portfolio |
| **Events** | Real-time block/event subscription with filtering (staking, transfer, weights, etc.) |
| **Key Swaps** | Hotkey swap, coldkey swap (scheduled) |
| **Root** | Root registration, root weights |
| **Raw Calls** | Submit to any pallet via dynamic dispatch (EVM, MEV Shield, Contracts) |
| **Config** | Persistent settings (`~/.agcli/config.toml`), set/unset/show config values |
| **Proxy** | Wrap any extrinsic through Proxy.proxy, add/remove proxy accounts |
| **Serve** | Set axon endpoint (IP/port/protocol) for miners on subnets |
| **Multisig** | Derive multisig address, submit/approve multisig calls |
| **Validators** | Top validators overview by stake (per-subnet or global) |
| **History** | Transaction history via Subscan API |
| **Account** | Detailed account explorer — balance, stakes, identity, delegate info |
| **Analytics** | Subnet analytics (miner/validator stats, economics) + staking analytics (APY, projections) |
| **Wizard** | Interactive staking wizard — shows top subnets, guided flow |
| **Completions** | Shell completions for bash, zsh, fish, powershell |
| **Update** | Self-update via `agcli update` (cargo install from GitHub) |
| **Output** | Table (default), JSON (`--output json`), CSV (`--output csv`) |

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

# Create a wallet
agcli wallet create --name my_wallet

# List all subnets (with real names from SubnetIdentitiesV3)
agcli subnet list

# View metagraph as JSON
agcli --output json subnet metagraph 1

# Add stake
agcli stake add 10.0 --netuid 1 --hotkey 5Hx...

# Transfer TAO
agcli transfer 5Dest... 1.5

# Set weights
agcli weights set --netuid 1 "0:100,1:200,2:300"

# Commit-reveal weights
agcli weights commit --netuid 1 "0:100,1:200"

# View portfolio (with real prices and subnet names)
agcli view portfolio

# View Dynamic TAO (prices, pools, volumes)
agcli view dynamic

# View Dynamic TAO as CSV
agcli --output csv view dynamic

# Live mode — poll dynamic prices every 12s, show deltas
agcli --live view dynamic

# Live metagraph — track neuron changes on SN1 every 30s
agcli --live 30 subnet metagraph 1

# Live portfolio — watch your portfolio in real-time
agcli --live view portfolio

# Subscribe to finalized blocks
agcli subscribe blocks

# Subscribe to all chain events (as JSON)
agcli --output json subscribe events

# Subscribe to staking events only
agcli subscribe events staking

# View network info as JSON
agcli --output json view network

# POW registration (multi-threaded)
agcli subnet pow 1 --threads 8

# Set subnet identity
agcli identity set-subnet 1 --name "My Subnet" --github "user/repo"

# Query on-chain identity
agcli identity show 5GrwvaEF5zXb...

# Interactive staking wizard
agcli stake wizard

# Configuration (persistent to ~/.agcli/config.toml)
agcli config set network finney
agcli config set wallet my_wallet
agcli config set output json
agcli config show
agcli config unset output
agcli config path

# Proxy — execute through a proxy account
agcli --proxy 5ProxyAccount... stake add 10 --netuid 1

# View top validators (global)
agcli view validators --limit 20

# View validators on a specific subnet
agcli view validators --netuid 1

# Transaction history (via Subscan)
agcli view history --address 5Gx... --limit 10

# Multisig — derive address
agcli multisig address "5Addr1...,5Addr2...,5Addr3..." --threshold 2

# Multisig — submit a call
agcli multisig submit --others "5Addr2...,5Addr3..." --threshold 2 \
  --pallet Balances --call transfer_allow_death --args '[...]'

# Multisig — approve a pending call
agcli multisig approve --others "5Addr2...,5Addr3..." --threshold 2 \
  --call-hash 0xabcdef...

# Account explorer (balance, stakes, identity, delegate info)
agcli view account --address 5Gx...

# Subnet analytics (miners, validators, economics, top performers)
agcli view subnet-analytics 1

# Staking analytics (APY estimates, emission projections)
agcli view staking-analytics --address 5Gx...

# Transfer entire balance (minus fees)
agcli transfer-all 5Dest... --keep-alive

# Serve axon endpoint (miners)
agcli serve axon --netuid 1 --ip 1.2.3.4 --port 8091

# Recycle alpha back to TAO
agcli stake recycle-alpha 100.0 --netuid 1

# Dissolve a subnet (owner only)
agcli subnet dissolve 42

# Add a proxy account
agcli proxy add 5DelegateAddr... --proxy-type staking --delay 0

# Remove a proxy account
agcli proxy remove 5DelegateAddr... --proxy-type staking

# Self-update to latest version
agcli update

# Shell completions
agcli completions bash > /etc/bash_completion.d/agcli
agcli completions zsh > ~/.zfunc/_agcli
agcli completions fish > ~/.config/fish/completions/agcli.fish
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

## Architecture

```
agcli/
├── src/
│   ├── config.rs        # Persistent config file (~/.agcli/config.toml)
│   ├── lib.rs           # Library root, re-exports Client/Wallet/Balance/Config
│   ├── main.rs          # CLI entry point
│   ├── chain/           # Substrate client (subxt-based)
│   │   ├── mod.rs         # Client: 35+ queries + 30+ extrinsics + multisig + sign_submit
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
│       ├── mod.rs         # Clap parser: 15 command groups, 65+ subcommands
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

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `AGCLI_NETWORK` | `finney` | Network preset (finney/test/local) |
| `AGCLI_ENDPOINT` | — | Custom WS endpoint (overrides network) |
| `AGCLI_WALLET_DIR` | `~/.bittensor/wallets` | Wallet directory |
| `AGCLI_WALLET` | `default` | Active wallet name |
| `AGCLI_HOTKEY` | `default` | Active hotkey name |
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
