# subnet — Subnet Operations

Create, manage, monitor, and query subnets on the Bittensor network. Subnets are independent networks identified by a netuid (u16), each with its own metagraph, hyperparameters, and alpha token.

## Query Subcommands

### subnet list
List all active subnets with names, neuron counts, emissions, and burn costs.

```bash
agcli subnet list [--at-block N]
# JSON: [{"netuid", "name", "n", "max_n", "tempo", "emission", "burn_rao", "owner"}]
```

**On-chain**: reads `NetworksAdded`, `SubnetIdentitiesV3`, `DynamicInfo` storage maps.

### subnet show
Show detailed info for a single subnet including Dynamic TAO pricing.

```bash
agcli subnet show --netuid 1 [--at-block N]
```

**On-chain**: reads `SubnetHyperparams`, `DynamicInfo` (tao_in, alpha_in, alpha_out, price).

### subnet hyperparams
Show all hyperparameters for a subnet.

```bash
agcli subnet hyperparams --netuid 1
```

Shows: tempo, immunity_period, max_allowed_uids, min_burn, max_burn, difficulty, weights_rate_limit, commit_reveal_weights_enabled, commit_reveal_period, and 20+ more params.

### subnet metagraph
View the full metagraph (all neurons) or a single UID.

```bash
agcli subnet metagraph --netuid 1 [--uid 0] [--at-block N] [--full] [--save]
# CSV: uid,hotkey,coldkey,stake,trust,consensus,incentive,dividends,emission
```

| Flag | Description |
|------|-------------|
| `--uid` | Show only one neuron (returns JSON object) |
| `--full` | Include axon/prometheus endpoint info |
| `--save` | Cache snapshot to disk (~/.agcli/metagraph/) |
| `--at-block` | Historical query (use `--network archive`) |

### subnet cache-load / cache-list / cache-diff / cache-prune
Manage cached metagraph snapshots for offline comparison.

```bash
agcli subnet cache-list --netuid 1
agcli subnet cache-load --netuid 1 [--block N]
agcli subnet cache-diff --netuid 1 [--from-block A] [--to-block B]
agcli subnet cache-prune --netuid 1 [--keep 10]
```

### subnet probe
Probe axon health for neurons on a subnet (TCP connectivity check).

```bash
agcli subnet probe --netuid 1 [--uids "0,1,2"] [--timeout-ms 3000] [--concurrency 32]
```

### subnet watch
Live tempo countdown, rate limits, and commit-reveal status.

```bash
agcli subnet watch --netuid 1 [--interval 12]
```

### subnet monitor
Track registrations, weight changes, emission shifts, and anomalies in real-time.

```bash
agcli subnet monitor --netuid 1 [--interval 24] [--json]
```

### subnet health
Health dashboard: miner/validator status, weight staleness, consensus alignment.

```bash
agcli subnet health --netuid 1
```

### subnet emissions
Per-UID emission breakdown for a subnet.

```bash
agcli subnet emissions --netuid 1
```

### subnet cost
Registration cost, difficulty, and burn range for a subnet.

```bash
agcli subnet cost --netuid 1
```

### subnet commits
Show pending weight commits on a subnet.

```bash
agcli subnet commits --netuid 1 [--hotkey SS58]
```

### subnet liquidity
AMM depth dashboard: tao_in, alpha_in, slippage at 0.1/1/10/100 TAO trade sizes.

```bash
agcli subnet liquidity [--netuid 1]
```

### subnet emission-split / mechanism-count
View emission split across mechanisms for a subnet.

```bash
agcli subnet emission-split --netuid 1
agcli subnet mechanism-count --netuid 1
```

## Extrinsic Subcommands (Write Operations)

### subnet register
Create a new subnet. Burns the current subnet registration cost (lock cost).

```bash
agcli subnet register [--password PW] [--yes]
```

**On-chain**: `SubtensorModule::register_network(origin, hotkey)` or `register_network_with_identity(origin, hotkey, identity)`
- Storage writes: `SubnetMechanism`, `NetworkRegisteredAt`, `TokenSymbol`, `SubnetTAO`, `SubnetAlphaIn`, `SubnetOwner`, `SubnetOwnerHotkey`, `SubnetLocked`, `SubnetworkN`, `NetworksAdded`, `Tempo`, `TotalNetworks` + all hyperparam defaults
- Events: `NetworkAdded(netuid, mechid)`, optionally `SubnetIdentitySet(netuid)`
- Errors: `SubnetLimitReached`, `CannotAffordLockCost`, `BalanceWithdrawalError`, `NetworkTxRateLimitExceeded`
- Note: Registration cost increases with each new subnet; requires `StartCallDelay` blocks before emissions begin

### subnet register-neuron
Register a neuron on an existing subnet (burn registration).

```bash
agcli subnet register-neuron --netuid 1 [--password PW] [--yes]
```

**On-chain**: `SubtensorModule::burned_register(origin, netuid, hotkey)`
- Events: `NeuronRegistered(netuid, uid, hotkey)`
- Errors: `SubNetRegistrationDisabled`, `TooManyRegistrationsThisBlock`, `TooManyRegistrationsThisInterval`

### subnet pow
Register via proof-of-work (multi-threaded CPU mining).

```bash
agcli subnet pow --netuid 1 [--threads 4]
```

**On-chain**: `SubtensorModule::register(origin, netuid, block, nonce, work, hotkey, coldkey)`

### subnet dissolve
Dissolve a subnet (owner only). Permanently removes the subnet.

```bash
agcli subnet dissolve --netuid 1 [--password PW] [--yes]
```

**On-chain**: `SubtensorModule::schedule_dissolve_network(origin, netuid)`
- Events: `DissolveNetworkScheduled(account, netuid, execution_block)`
- Errors: `NotSubnetOwner`, `SubnetNotExists`

### subnet start
Start a subnet's emission schedule (owner only).

```bash
agcli subnet start --netuid 1
```

**On-chain**: `SubtensorModule::start_call(origin, netuid)` — sets `FirstEmissionBlockNumber`.

### subnet check-start
Check if a subnet's emission schedule can be started.

```bash
agcli subnet check-start --netuid 1
```

### subnet set-param
Set a subnet hyperparameter (owner only). 31 configurable params.

```bash
agcli subnet set-param --netuid 1 --param tempo --value 100
agcli subnet set-param --netuid 1 --param list  # show all settable params
```

Settable params: tempo, max_allowed_uids, min_allowed_uids, immunity_period, max_allowed_validators, min_burn, max_burn, difficulty, weights_rate_limit, weights_version_key, commit_reveal_period, adjustment_interval, target_regs_per_interval, activity_cutoff, serving_rate_limit, bonds_moving_average, bonds_penalty, and more.

### subnet set-symbol
Set subnet token symbol (owner only).

```bash
agcli subnet set-symbol --netuid 1 --symbol "ALPHA"
```

**On-chain**: `SubtensorModule::update_symbol(origin, netuid, symbol)`

### subnet trim
Trim UIDs to a max count (owner only).

```bash
agcli subnet trim --netuid 1 --max-uids 256
```

### subnet set-mechanism-count / set-emission-split
Configure emission mechanisms (owner only).

```bash
agcli subnet set-mechanism-count --netuid 1 --count 2
agcli subnet set-emission-split --netuid 1 --weights "50,50"
```

## Common Errors
| Error | Cause | Fix |
|-------|-------|-----|
| `SubnetNotExists` | Invalid netuid | Check `agcli subnet list` |
| `NotSubnetOwner` | Not the subnet owner | Use owner coldkey |
| `SubnetLimitReached` | Max subnet count reached | Wait for a subnet to be pruned |
| `TooManyRegistrationsThisBlock` | Registration flood | Wait 1+ blocks |
| `SubNetRegistrationDisabled` | Subnet has registration off | Check hyperparams |

## Source Code
**agcli handler**: [`src/cli/subnet_cmds.rs`](https://github.com/unconst/agcli/blob/main/src/cli/subnet_cmds.rs) — `handle_subnet()` at L9, subcommands: List L17, Show L115, Hyperparams L182, Metagraph L264, Register L472, RegisterNeuron L480, Pow L492, Dissolve L540, Watch L561, Monitor L567, Health L572, Emissions L573, Cost L576, Commits L721, SetParam L724, SetSymbol L738, Trim L768, Start L821, CheckStart L796, EmissionSplit L748, MechanismCount L836, SetMechanismCount L846, SetEmissionSplit L864, CacheLoad L577, CacheList L635, CacheDiff L660, CachePrune L704, Probe L713, Liquidity L564

**Subtensor pallet**:
- [`subnets/registration.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/subnets/registration.rs) — `register_network`, `burned_register`, `register` (PoW)
- [`subnets/subnet.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/subnets/subnet.rs) — `schedule_dissolve_network`, `start_call`, subnet lifecycle
- [`subnets/mechanism.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/subnets/mechanism.rs) — emission mechanisms, mechanism counts
- [`subnets/symbols.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/subnets/symbols.rs) — `update_symbol`
- [`subnets/uids.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/subnets/uids.rs) — UID management, trim
- [`subnets/serving.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/subnets/serving.rs) — axon serving
- [`subnets/weights.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/subnets/weights.rs) — weight commit/reveal
- [`macros/dispatches.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/macros/dispatches.rs) — all dispatch entry points
- [`macros/events.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/macros/events.rs) — event definitions
- [`macros/errors.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/macros/errors.rs) — error definitions

## Related Commands
- `agcli stake add --netuid N` — Stake on a subnet
- `agcli weights set --netuid N` — Set weights on a subnet
- `agcli view dynamic` — See all subnet prices and pools
- `agcli explain --topic subnets` — What subnets are
- `agcli explain --topic hyperparams` — Hyperparameters reference
