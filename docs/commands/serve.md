# serve — Miner/Validator Serving

Announce axon endpoint on-chain so other neurons can connect to your miner or validator.

## Subcommands

### serve axon
Set your axon serving endpoint for a subnet.

```bash
agcli serve axon --netuid 1 --ip 1.2.3.4 --port 8091 [--protocol 4] [--version 0]
```

| Flag | Required | Description |
|------|----------|-------------|
| `--netuid` | yes | Subnet UID |
| `--ip` | yes | Public IP address |
| `--port` | yes | Serving port |
| `--protocol` | no | IP version: 4 (IPv4) or 6 (IPv6) |
| `--version` | no | Axon version number |

**On-chain**: `SubtensorModule::serve_axon(origin, netuid, version, ip, port, ip_type, protocol, placeholder1, placeholder2)`
- Storage: `Axons` map keyed by (netuid, hotkey)
- Events: `AxonServed(netuid, hotkey)`
- Errors: `InvalidIpType`, `InvalidIpAddress`, `InvalidPort`, `ServingRateLimitExceeded`
- Rate limited per `serving_rate_limit` hyperparameter

## Related Commands
- `agcli subnet metagraph --netuid N --full` — See axon endpoints for all neurons
- `agcli subnet probe --netuid N` — Test axon connectivity
- `agcli explain --topic axon` — What axons are
