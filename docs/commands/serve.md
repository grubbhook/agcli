# serve — Miner/Validator Serving

Announce axon endpoint on-chain so other neurons can connect to your miner or validator.

## Subcommands

### serve axon
Set your axon serving endpoint for a subnet.

```bash
agcli serve axon --netuid 1 --ip 1.2.3.4 --port 8091 [--protocol 4] [--version 0]
```

**On-chain**: `SubtensorModule::serve_axon(origin, netuid, version, ip, port, ip_type, protocol, placeholder1, placeholder2)`
- Storage: `Axons` map keyed by (netuid, hotkey)
- Events: `AxonServed(netuid, hotkey)`
- Errors: `InvalidIpType`, `InvalidIpAddress`, `InvalidPort`, `ServingRateLimitExceeded`
- Rate limited per `serving_rate_limit` hyperparameter

### serve prometheus
Set prometheus monitoring endpoint for your neuron.

```bash
agcli serve prometheus --ip 1.2.3.4 --port 9090 [--version 0]
```

**On-chain**: `SubtensorModule::serve_prometheus(origin, netuid, version, ip, port, ip_type)`
- Events: `PrometheusServed(netuid, hotkey)`

## Source Code
**agcli handler**: [`src/cli/network_cmds.rs`](https://github.com/unconst/agcli/blob/main/src/cli/network_cmds.rs) — `handle_serve()` at L430, subcommands: Axon L438, Reset L474

**Subtensor pallet**:
- [`subnets/serving.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/subnets/serving.rs) — `serve_axon`, `serve_prometheus` logic
- [`macros/dispatches.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/macros/dispatches.rs) — dispatch entry points

## Related Commands
- `agcli subnet metagraph --netuid N --full` — See axon endpoints for all neurons
- `agcli subnet probe --netuid N` — Test axon connectivity
- `agcli explain --topic axon` — What axons are
