# block — Block Explorer

Query finalized block information. Useful for debugging, auditing, and historical analysis.

## Subcommands

### block latest
Show the latest finalized block.

```bash
agcli block latest
# JSON: {"number", "hash", "parent_hash", "timestamp", "extrinsic_count"}
```

### block info
Show details for a specific block.

```bash
agcli block info --number 4000000
# JSON: {"number", "hash", "parent_hash", "state_root", "timestamp", "extrinsics": [...]}
```

## Related Commands
- `agcli diff` — Compare chain state between two blocks
- `agcli subscribe blocks` — Watch blocks in real-time
- `agcli --network archive block info --number N` — Query historical blocks
