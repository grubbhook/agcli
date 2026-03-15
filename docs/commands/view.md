# view — Query & Analytics Commands

Read-only commands for querying chain state, analytics, and account information. No wallet unlock required for most commands.

## Subcommands

### view portfolio
Full stake portfolio for a coldkey: balance, all stake positions, alpha holdings, estimated values.

```bash
agcli view portfolio [--address SS58] [--at-block N]
# JSON: {"address", "balance", "total_stake", "positions": [...]}
```

Uses parallelized queries (`try_join!`) for fast loading.

### view network
Network-wide overview: total issuance, total stake, emission rate, block height, active subnets.

```bash
agcli view network [--at-block N]
```

### view dynamic
Dynamic TAO info for all subnets: prices, AMM pool depths, volumes, emissions.

```bash
agcli view dynamic [--at-block N]
# CSV: netuid,name,price,tao_in,alpha_in,alpha_out,emission
```

### view neuron
Full detail for a single neuron: stake, weights, bonds, axon, rank, trust, consensus.

```bash
agcli view neuron --netuid 1 --uid 0 [--at-block N]
```

### view validators
Top validators by stake across subnets.

```bash
agcli view validators [--netuid N] [--limit 50] [--at-block N]
```

### view history
Recent extrinsics for an account (via Subscan API).

```bash
agcli view history [--address SS58] [--limit 20]
```

Fetches from `https://bittensor.api.subscan.io/api/v2/scan/extrinsics`.

### view account
Comprehensive account explorer: balance, stakes, identities, proxy delegations, recent history.

```bash
agcli view account [--address SS58] [--at-block N]
```

### view subnet-analytics
Detailed subnet analysis: neuron distribution, weight patterns, emission concentration.

```bash
agcli view subnet-analytics --netuid 1
```

### view staking-analytics
APY estimates and yield analysis for staked positions.

```bash
agcli view staking-analytics [--address SS58]
```

### view swap-sim
Simulate an AMM swap without executing. Shows expected output, fees, and price impact.

```bash
agcli view swap-sim --netuid 1 --tao 10.0
agcli view swap-sim --netuid 1 --alpha 1000.0
# JSON: {"alpha_out", "tao_fee", "alpha_fee", "price_impact_pct"}
```

Use this before `stake add` to estimate slippage.

### view nominations
Show who delegates to a specific validator.

```bash
agcli view nominations --hotkey SS58
```

## Live Mode
Any view command can be polled continuously with `--live`:

```bash
agcli --live view dynamic          # poll with delta tracking
agcli --live 30 view portfolio     # poll every 30s
agcli --live subnet metagraph --netuid 1  # track neuron changes
```

## Historical Queries
Most view commands support `--at-block N`:

```bash
agcli --network archive view portfolio --at-block 3000000
agcli --network archive view dynamic --at-block 3500000
```

Requires an archive node for blocks beyond ~256 block pruning window.

## Source Code
**agcli handler**: [`src/cli/view_cmds.rs`](https://github.com/unconst/agcli/blob/main/src/cli/view_cmds.rs) — `handle_view()` at L9, subcommands: Portfolio L17, Network L27, Dynamic L28, Neuron L37, Validators L42, History L47, Account L51, SubnetAnalytics L55, StakingAnalytics L58, SwapSim L62, Nominations L65. Audit: `handle_audit()` at L1287.

**On-chain**: read-only queries against `System::Account`, `SubtensorModule` storage maps (Stake, Alpha, DynamicInfo, SubnetHyperparams, Metagraph, etc.). History uses [Subscan API](https://bittensor.api.subscan.io/api/v2/scan/extrinsics).

## Related Commands
- `agcli balance` — Simple balance check
- `agcli stake list` — Stake positions only
- `agcli subnet metagraph` — Full metagraph data
- `agcli explain --topic amm` — How Dynamic TAO AMM works
