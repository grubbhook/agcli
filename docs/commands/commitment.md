# commitment — Miner Commitment Operations

Miners publish endpoint and metadata via on-chain commitments. This replaces the legacy Axon serving mechanism for broadcasting miner endpoints.

## Subcommands

### commitment set
Publish commitment data on a subnet.

```bash
agcli commitment set --netuid 1 --data "endpoint:http://1.2.3.4:8091,version:1.0"
```

**On-chain**: `Commitments::set_commitment(origin, netuid, info)`
- Storage writes: `CommitmentOf` map (keyed by netuid + account)
- Events: `Commitment { who, netuid }`
- Errors: `TooManyFieldsInCommitmentInfo`
- The data fields are stored as bounded Raw bytes on-chain
- Requires a deposit to set (refunded when cleared)

### commitment get
Read the commitment for a specific hotkey on a subnet.

```bash
agcli commitment get --netuid 1 --hotkey 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
# JSON: {"hotkey", "netuid", "block", "fields": [...]}
```

**On-chain**: reads `Commitments::CommitmentOf(netuid, account_id)` storage map.

### commitment list
List all commitments on a subnet.

```bash
agcli commitment list --netuid 1
# JSON: [{"hotkey", "block", "fields": [...]}, ...]
```

**On-chain**: iterates `Commitments::CommitmentOf` storage prefix for the given netuid.

## Common Errors
| Error | Cause | Fix |
|-------|-------|-----|
| `TooManyFieldsInCommitmentInfo` | Data exceeds field limit | Use fewer comma-separated values |
| `InsufficientBalance` | Not enough for deposit | Top up balance |

## Source Code
**agcli handler**: `src/cli/network_cmds.rs` — `handle_commitment()`

**Subtensor pallet**: `pallet_commitments` — `set_commitment`, `CommitmentOf` storage

## Related Commands
- `agcli serve axon` — Legacy axon endpoint announcement
- `agcli identity set-subnet` — Set subnet identity (different from miner commitment)
