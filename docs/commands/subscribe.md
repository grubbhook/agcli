# subscribe — Real-Time Event Streaming

Subscribe to finalized blocks or filtered chain events in real-time. Useful for monitoring, alerting, and integration pipelines.

## Subcommands

### subscribe blocks
Watch finalized blocks as they are produced (~12s intervals).

```bash
agcli subscribe blocks
```

### subscribe events
Stream chain events with optional filtering.

```bash
agcli subscribe events [--filter staking] [--netuid 1] [--account SS58]
```

| Flag | Description |
|------|-------------|
| `--filter` | Event category: all, staking, registration, transfer, weights, subnet |
| `--netuid` | Only show events for this subnet |
| `--account` | Only show events involving this address |

## Filter Categories
| Filter | Events Included |
|--------|----------------|
| `all` | Everything |
| `staking` | StakeAdded, StakeRemoved, StakeMoved |
| `registration` | NeuronRegistered, BulkNeuronsRegistered |
| `transfer` | Transfer, Deposit, Withdraw |
| `weights` | WeightsSet, WeightsCommitted, WeightsRevealed |
| `subnet` | NetworkAdded, NetworkRemoved, hyperparameter changes |

## Source Code
**agcli handler**: [`src/cli/network_cmds.rs`](https://github.com/unconst/agcli/blob/main/src/cli/network_cmds.rs) — `handle_subscribe()` at L284, subcommands: Blocks L292, Events L293

**On-chain**: uses subxt subscription APIs (`subscribe_finalized_blocks`, `subscribe_events`) — no extrinsics.

## Related Commands
- `agcli subnet monitor` — Higher-level subnet monitoring with anomaly detection
- `agcli subnet watch` — Tempo countdown and weight window status
