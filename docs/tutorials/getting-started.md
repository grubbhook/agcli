# Getting Started with agcli

agcli is a Rust CLI + SDK for the Bittensor network. It covers wallet management, staking, transfers, subnet operations, weight setting, delegation, and more.

## Install

```bash
# From source (requires Rust toolchain)
git clone https://github.com/unconst/agcli && cd agcli
cargo install --path .

# Or install directly from GitHub
cargo install --git https://github.com/unconst/agcli

# Self-update an existing installation
agcli update
```

**Requirements:** Rust 1.75+, a C compiler (gcc/clang), and network access (build fetches chain metadata).

## Key Concepts

- **TAO** — the native token of Bittensor (1 TAO = 1,000,000,000 rao)
- **Coldkey** — your spending key (encrypted on disk, password-protected)
- **Hotkey** — your operational key (used for mining/validating, stored in plaintext)
- **Wallet** — a directory containing your coldkey + hotkey files
- **Subnet** — an independent network within Bittensor, identified by a `netuid` (0–65535)
- **Alpha** — each subnet has its own alpha token; staking TAO buys alpha at the current price
- **Dynamic TAO** — the AMM mechanism that prices alpha tokens based on supply and demand

## Create a Wallet

```bash
agcli wallet create --name my_wallet
# Prompts for a password to encrypt the coldkey.
# Generates a 12-word mnemonic — WRITE IT DOWN AND STORE IT SECURELY.

# Non-interactive (for scripts/agents):
agcli wallet create --name my_wallet --password mypass123
```

Create a hotkey for mining or validating:

```bash
agcli wallet create-hotkey --name my_wallet --hotkey default
```

List all wallets and check details:

```bash
agcli wallet list
agcli wallet show -w my_wallet --password mypass123
```

## Check Your Balance

```bash
agcli balance
# Uses your default wallet. Or specify an address:
agcli balance --address 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY

# JSON output for scripts:
agcli balance --output json
```

## View the Network

```bash
# Network overview (block height, issuance, staking ratio)
agcli view network

# List all subnets
agcli subnet list

# Detailed subnet info
agcli subnet show 1

# Subnet hyperparameters
agcli subnet hyperparams 1

# Metagraph (all neurons on a subnet)
agcli subnet metagraph 1

# Dynamic TAO info (prices, pools, emissions)
agcli view dynamic
```

## Stake TAO

```bash
# Interactive wizard — shows subnets, asks for your choice, confirms
agcli stake wizard

# Direct staking
agcli stake add 10.0 --netuid 1

# View your stakes
agcli stake list

# Staking analytics (APY estimates, yield projections)
agcli view staking-analytics
```

## Transfer TAO

```bash
agcli transfer 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY 1.5
```

## Configuration

Settings are resolved in priority order: CLI flags > env vars > config file > defaults.

### Config file (persisted to `~/.agcli/config.toml`)

```bash
agcli config set network finney
agcli config set wallet my_wallet
agcli config set output json     # table, json, or csv

agcli config show                # view all settings
agcli config unset output        # remove a setting
agcli config path                # show config file location
```

### Environment variables

```bash
export AGCLI_NETWORK=finney
export AGCLI_WALLET=my_wallet
export AGCLI_WALLET_DIR=~/.bittensor/wallets
export AGCLI_PASSWORD=mypass     # skip password prompts
export AGCLI_YES=1               # skip confirmation prompts
```

### CLI flags

```bash
agcli --network test --wallet my_wallet --output json subnet list
agcli --yes --password mypass stake add 10.0 --netuid 1
```

## Shell Completions

```bash
agcli completions bash > /etc/bash_completion.d/agcli    # Bash
agcli completions zsh > ~/.zfunc/_agcli                   # Zsh
agcli completions fish > ~/.config/fish/completions/agcli.fish  # Fish
```

## Networks

| Network | Flag | Endpoint |
|---------|------|----------|
| Finney (mainnet) | `--network finney` | `wss://entrypoint-finney.opentensor.ai:443` |
| Testnet | `--network test` | `wss://test.finney.opentensor.ai:443` |
| Local | `--network local` | `ws://127.0.0.1:9944` |
| Custom | `--endpoint wss://...` | any WebSocket URL |

## Troubleshooting

**Connection refused / timeout:**
- Check your network connection
- Verify the endpoint: `agcli --network finney view network`
- Finney nodes can be slow at peak times — retry after a moment

**Wrong password:**
- Error: "Decryption failed — wrong password"
- If you forgot your password, restore from mnemonic: `agcli wallet regen-coldkey --name my_wallet`

**Wallet not found:**
- Error: "Wallet 'X' not found"
- List available wallets: `agcli wallet list`
- Check wallet directory: `agcli config show` (look at `wallet_dir`)

**Invalid address:**
- Bittensor SS58 addresses are 48 characters and start with `5`
- Verify on taostats.io before sending funds

**Transaction failed:**
- Read the error hint carefully — agcli maps chain errors to suggestions
- Common: insufficient balance, rate limits, wrong subnet, hotkey not registered
