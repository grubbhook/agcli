# multisig — Multisig Operations

Create and manage multi-signature transactions. Requires M-of-N signatories to approve before execution.

## Subcommands

### multisig address
Derive a deterministic multisig address from signatories and threshold.

```bash
agcli multisig address --signatories "SS58_1,SS58_2,SS58_3" --threshold 2
# Output: multisig SS58 address
```

### multisig submit
Submit a new multisig call (first approval).

```bash
agcli multisig submit --others "SS58_2,SS58_3" --threshold 2 \
  --pallet SubtensorModule --call add_stake --args '[...]'
```

| Flag | Description |
|------|-------------|
| `--others` | Other signatories (comma-separated SS58) |
| `--threshold` | Approvals needed (including submitter) |
| `--pallet` | Target pallet name |
| `--call` | Call name within pallet |
| `--args` | JSON array of call arguments |

**On-chain**: `Multisig::as_multi(origin, threshold, other_signatories, maybe_timepoint, call, max_weight)`

### multisig approve
Approve a pending multisig call by its hash.

```bash
agcli multisig approve --others "SS58_2,SS58_3" --threshold 2 --call-hash 0x...
```

**On-chain**: `Multisig::approve_as_multi(origin, threshold, other_signatories, maybe_timepoint, call_hash, max_weight)`

## Source Code
**agcli handler**: [`src/cli/network_cmds.rs`](https://github.com/unconst/agcli/blob/main/src/cli/network_cmds.rs) — `handle_multisig()` at L314, subcommands: Address L323, Submit L356, Approve L400

**Substrate pallet**: Uses standard `Multisig` pallet (`Multisig::as_multi`, `Multisig::approve_as_multi`).

## Related Commands
- `agcli proxy add` — Simpler delegation (single signer)
