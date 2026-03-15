# balance — Account Balance

Query TAO balance for any account. Supports one-shot lookup, historical queries, and continuous monitoring with alerts.

## Usage

### One-shot balance check
```bash
agcli balance [--address SS58]
# JSON: {"address", "balance_rao", "balance_tao"}
```

| Flag | Description |
|------|-------------|
| `--address` | SS58 address to query (defaults to wallet coldkey) |

**On-chain**: reads `System::Account` storage for the account's free balance.

### Historical balance (wayback)
```bash
agcli balance --at-block 4000000 [--address SS58]
# JSON: {"address", "block", "block_hash", "balance_rao", "balance_tao"}
```

Requires `--network archive` for blocks beyond the ~256 block pruning window.

### Watch mode (continuous monitoring)
```bash
agcli balance --watch [60] --threshold 10.0 [--address SS58]
```

| Flag | Description |
|------|-------------|
| `--watch [N]` | Poll every N seconds (default: 60) |
| `--threshold T` | Alert when balance drops below T TAO |

**JSON streaming** (one object per poll):
```json
{"address": "5G...", "balance_rao": 10000000000, "balance_tao": 10.0,
 "below_threshold": false, "timestamp": "2024-01-01T00:00:00Z"}
```

Stops on Ctrl+C. Continues through transient RPC errors with warnings.

## Common Errors
| Error | Cause | Fix |
|-------|-------|-----|
| Connection timeout | RPC endpoint down | Try `--network archive` or check endpoint |
| Invalid SS58 | Bad address format | Verify address (prefix 42) |

## Related Commands
- `agcli transfer` — Send TAO
- `agcli stake list` — View staked positions
- `agcli view portfolio` — Full portfolio with all stakes and balance
- `agcli view account` — Comprehensive account explorer
