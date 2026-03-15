# crowdloan — Crowdloan Operations

Participate in subnet crowdloans. Crowdloans pool contributions to fund leased subnet registrations.

## Subcommands

### crowdloan create
Create a new crowdloan for subnet lease funding.

```bash
agcli crowdloan create --cap 1000.0 --end-block 5000000 [--contribution-min 1.0] [--password PW]
```

**On-chain**: `Crowdloan::create(origin, cap, end_block, min_contribution)`

### crowdloan contribute
Contribute TAO to a crowdloan.

```bash
agcli crowdloan contribute --fund-index ID --amount 10.0 [--password PW] [--yes]
```

**On-chain**: `Crowdloan::contribute(origin, fund_index, amount)`

### crowdloan withdraw
Withdraw contribution from a crowdloan (after it ends or fails).

```bash
agcli crowdloan withdraw --fund-index ID [--password PW]
```

**On-chain**: `Crowdloan::withdraw(origin, fund_index)`

### crowdloan finalize
Finalize a completed crowdloan (triggers subnet lease registration).

```bash
agcli crowdloan finalize --fund-index ID [--password PW]
```

**On-chain**: triggers `register_leased_network` with pooled contributions.

### crowdloan refund
Refund a specific contribution from a crowdloan.

```bash
agcli crowdloan refund --fund-index ID --contribution-index N [--password PW]
```

### crowdloan dissolve
Dissolve a crowdloan after all funds are returned.

```bash
agcli crowdloan dissolve --fund-index ID [--password PW]
```

## Source Code
**agcli handler**: [`src/cli/network_cmds.rs`](https://github.com/unconst/agcli/blob/main/src/cli/network_cmds.rs) — `handle_crowdloan()` at L616, subcommands: List L624, Info L662, Contributors L683, Create L729, Contribute L760, Withdraw L775, Finalize L782, Refund L789, Dissolve L796, UpdateCap L803, UpdateEnd L817, UpdateMinContribution L832

**Substrate pallet**: Uses `Crowdloan` pallet for fund management and [`subnets/leasing.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/subnets/leasing.rs) for leased subnet registration.

## Related Commands
- `agcli subnet list` — See active subnets including leased ones
