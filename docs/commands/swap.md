# swap — Key Swap Operations

Swap hotkeys or schedule coldkey swaps. Critical security operations with rate limiting.

## Subcommands

### swap hotkey
Swap to a new hotkey. Transfers all registrations, stake, and state to the new hotkey.

```bash
agcli swap hotkey --new-hotkey SS58 [--password PW] [--yes]
```

**On-chain**: `SubtensorModule::swap_hotkey(origin, old_hotkey, new_hotkey)`
- Errors: `NewHotKeyIsSameWithOld`, `HotKeySetTxRateLimitExceeded`, `NonAssociatedColdKey`

### swap coldkey
Schedule a coldkey swap. Executes after a security delay period.

```bash
agcli swap coldkey --new-coldkey SS58 [--password PW] [--yes]
```

**On-chain**: `SubtensorModule::schedule_coldkey_swap(origin, new_coldkey, work, block_number, nonce)`
- Scheduled for future block (security delay)
- Check status: `agcli wallet check-swap`

## Related Commands
- `agcli wallet check-swap` — Check pending swap status
- `agcli explain --topic coldkey-swap` — Coldkey swap mechanics
