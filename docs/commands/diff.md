# diff — Historical State Comparison

Compare chain state between two blocks. Useful for tracking changes in portfolio value, subnet metrics, or network-wide statistics over time.

## Subcommands

### diff portfolio
Compare portfolio between two blocks.

```bash
agcli diff portfolio --from-block 3900000 --to-block 4000000 [--address SS58]
```

### diff subnet
Compare subnet state between two blocks.

```bash
agcli diff subnet --netuid 1 --from-block 3900000 --to-block 4000000
```

### diff network
Compare network-wide stats between two blocks.

```bash
agcli diff network --from-block 3900000 --to-block 4000000
```

Requires `--network archive` for blocks beyond the ~256 block pruning window.

## Related Commands
- `agcli block info --number N` — See block details
- `agcli subnet cache-diff` — Compare cached metagraph snapshots
- `agcli explain --topic diff` — How diff works
