# delegate — Delegation Operations

View and manage validator delegation (take percentage). Delegates are validators who accept stake nominations from other coldkeys.

## Subcommands

### delegate list
List all delegates with their stake, nominators, and take percentage.

```bash
agcli delegate list
# JSON: [{"hotkey", "stake", "nominators", "take_pct"}]
```

### delegate show
Show detailed info for a specific delegate.

```bash
agcli delegate show [--hotkey SS58]
```

### delegate decrease-take
Decrease validator take percentage. Takes effect immediately.

```bash
agcli delegate decrease-take --take 10.0 [--hotkey SS58]
```

**On-chain**: `SubtensorModule::decrease_take(origin, hotkey, take)` where take is u16 (pct * 65535 / 100)
- Errors: `DelegateTakeTooLow`, `NonAssociatedColdKey`

### delegate increase-take
Increase validator take percentage. Subject to rate limiting.

```bash
agcli delegate increase-take --take 15.0 [--hotkey SS58]
```

**On-chain**: `SubtensorModule::increase_take(origin, hotkey, take)`
- Errors: `DelegateTakeTooHigh`, `DelegateTxRateLimitExceeded`

## Related Commands
- `agcli stake add` — Stake behind a delegate
- `agcli view nominations --hotkey SS58` — See who nominates a delegate
- `agcli explain --topic take` — How take percentage works
- `agcli explain --topic delegation` — Delegation mechanics
