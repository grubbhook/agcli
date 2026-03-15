# crowdloan — Crowdloan Operations

Participate in subnet crowdloans. Crowdloans pool contributions to fund leased subnet registrations.

## Subcommands

### crowdloan contribute
Contribute TAO to a crowdloan.

```bash
agcli crowdloan contribute --crowdloan-id ID --amount 10.0 [--password PW] [--yes]
```

### crowdloan withdraw
Withdraw contribution from a crowdloan (after it ends or fails).

```bash
agcli crowdloan withdraw --crowdloan-id ID [--password PW]
```

### crowdloan finalize
Finalize a completed crowdloan (triggers subnet lease registration).

```bash
agcli crowdloan finalize --crowdloan-id ID [--password PW]
```

**On-chain**: triggers `register_leased_network` with pooled contributions.

## Related Commands
- `agcli subnet list` — See active subnets including leased ones
