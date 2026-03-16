# batch — Batch Extrinsic Submission

Submit multiple extrinsics atomically (or non-atomically) from a JSON file. Uses Substrate's `Utility.batch_all` or `Utility.batch`.

## Usage

```bash
agcli batch --file calls.json [--no-atomic] [--password PW] [--yes]
```

## JSON Format
```json
[
  {"pallet": "SubtensorModule", "call": "add_stake", "args": ["5Hotkey...", 1, 1000000000]},
  {"pallet": "Balances", "call": "transfer_allow_death", "args": ["5Dest...", 5000000000]},
  {"pallet": "SubtensorModule", "call": "set_weights", "args": [1, [0,1], [100,200], 0]}
]
```

- Hex strings in args (`"0xdead..."`) are auto-decoded as bytes
- Uses `submit_raw_call` for each call — any pallet/call combo works

## Atomic vs Non-Atomic
- **atomic** (`batch_all`, default): All calls succeed or all revert. Safe for related operations.
- **non-atomic** (`batch`, `--no-atomic`): Each call independent. Failed calls don't revert others.

## Source Code
**agcli handler**: [`src/cli/system_cmds.rs`](https://github.com/unconst/agcli/blob/main/src/cli/system_cmds.rs) — `handle_batch()` at L354. Dispatched from [`src/cli/commands.rs`](https://github.com/unconst/agcli/blob/main/src/cli/commands.rs) at L384.

**Substrate pallet**: Uses standard `Utility` pallet (`Utility::batch_all`, `Utility::batch`).

## Related Commands
- `agcli weights set` — Single weight set (simpler than batch)
- `agcli stake add` — Single stake (simpler than batch)
