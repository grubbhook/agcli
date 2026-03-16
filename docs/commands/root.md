# root — Root Network Operations

The root network (SN0) is the meta-subnet that governs emission distribution across all subnets. Root validators set weights to determine which subnets receive more emissions.

## Subcommands

### root register
Register on the root network. Requires a hotkey with sufficient total stake.

```bash
agcli root register [--password PW] [--yes]
```

**On-chain**: `SubtensorModule::root_register(origin, hotkey)`
- Errors: `StakeTooLowForRoot`, `HotKeyAlreadyRegisteredInSubNet`

### root weights
Set root weights to influence subnet emission distribution.

```bash
agcli root weights --weights "1:500,2:300,3:200"
```

**On-chain**: `SubtensorModule::set_root_weights(origin, netuid, hotkey, dests, weights, version_key)`
- Weights determine relative emission share per subnet
- Events: `WeightsSet(0, uid)` (netuid=0 for root)
- Errors: `NotEnoughStakeToSetWeights`, `SettingWeightsTooFast`

## Source Code
**agcli handler**: [`src/cli/network_cmds.rs`](https://github.com/unconst/agcli/blob/main/src/cli/network_cmds.rs) — `handle_root()` at L11, subcommands: Register L19, Weights L32

**Subtensor pallet**:
- [`macros/dispatches.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/macros/dispatches.rs) — `root_register`, `set_root_weights` dispatch entry points
- [`subnets/weights.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/subnets/weights.rs) — root weight setting logic

## Related Commands
- `agcli view dynamic` — See current emission distribution
- `agcli subnet list` — View all subnets
- `agcli explain --topic root` — Root network mechanics
