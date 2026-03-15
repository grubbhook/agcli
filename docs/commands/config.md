# config — Persistent Configuration

Manage agcli configuration stored in `~/.agcli/config.toml`. Settings persist across invocations. Priority: CLI flags > env vars > config > defaults.

## Subcommands

### config show
Show current configuration.

```bash
agcli config show
```

### config set
Set a configuration value.

```bash
agcli config set --key network --value finney
agcli config set --key batch --value true
agcli config set --key spending_limit.97 --value 100.0
agcli config set --key spending_limit.* --value 500.0
```

### config unset
Remove a configuration value.

```bash
agcli config unset --key network
```

### config path
Show config file path.

```bash
agcli config path
# Output: /root/.agcli/config.toml
```

## Configurable Keys
| Key | Description | Example |
|-----|-------------|---------|
| `network` | Default network | finney, test, local, archive |
| `endpoint` | Custom RPC endpoint | wss://... |
| `wallet_dir` | Wallet directory | ~/.bittensor/wallets |
| `wallet` | Default wallet name | default |
| `hotkey` | Default hotkey name | default |
| `output` | Default output format | json, csv, table |
| `proxy` | Default proxy account | SS58 address |
| `live_interval` | Default live poll interval | 12 |
| `batch` | Enable batch mode | true/false |
| `spending_limit.N` | Max TAO per stake on SN N | 100.0 |
| `spending_limit.*` | Global max TAO per stake | 500.0 |

## Spending Limits (Agent Safety)
```bash
agcli config set --key spending_limit.97 --value 100.0   # Max 100 TAO on SN97
agcli config set --key spending_limit.* --value 500.0     # Global max
```

Pre-flight check runs before every `stake add`. Prevents accidental large stakes.

## Config writes are atomic
Uses temp-file + rename to prevent corruption on crash.
