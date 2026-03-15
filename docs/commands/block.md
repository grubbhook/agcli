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

### block range
Query a range of blocks (max 1000).

```bash
agcli block range --from 3999900 --to 4000000
```

Parallel queries for each block in range. Shows number, hash, extrinsic count, timestamp.

## Source Code
**agcli handler**: [`src/cli/block_cmds.rs`](https://github.com/unconst/agcli/blob/main/src/cli/block_cmds.rs) — `handle_block()` at L9, subcommands: Info L11, Range L50, Latest L109

**On-chain**: read-only queries using subxt block APIs (`get_block`, `get_block_hash`).

## Related Commands
- `agcli diff` — Compare chain state between two blocks
- `agcli subscribe blocks` — Watch blocks in real-time
- `agcli --network archive block info --number N` — Query historical blocks
