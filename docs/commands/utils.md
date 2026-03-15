# utils — Utility Commands

Miscellaneous tools: unit conversion, latency benchmarking, shell completions, self-update, diagnostics.

## Subcommands

### utils convert
Convert between TAO and RAO.

```bash
agcli utils convert --tao 1.5      # → 1500000000 RAO
agcli utils convert --rao 1000000000  # → 1.0 TAO
```

### utils latency
Benchmark RPC endpoint latency.

```bash
agcli utils latency [--count 10]
```

Measures round-trip time for chain queries.

### completions
Generate shell completions.

```bash
agcli completions --shell bash > ~/.bash_completion.d/agcli
agcli completions --shell zsh > ~/.zfunc/_agcli
agcli completions --shell fish > ~/.config/fish/completions/agcli.fish
agcli completions --shell powershell > _agcli.ps1
```

### update
Self-update agcli from GitHub.

```bash
agcli update
```

### doctor
Diagnostic check: connectivity, wallet access, chain state.

```bash
agcli doctor
```

## Source Code
**agcli handler**: [`src/cli/system_cmds.rs`](https://github.com/unconst/agcli/blob/main/src/cli/system_cmds.rs) — `handle_utils()` at L184 (Convert L191, Latency L215), `generate_completions()` at L112, `handle_update()` at L331, `handle_doctor()` at L439

**No on-chain interaction** for convert/completions/update. Latency and doctor make RPC test calls.

## Related Commands
- `agcli explain` — Built-in concept reference
- `agcli config show` — Current configuration
