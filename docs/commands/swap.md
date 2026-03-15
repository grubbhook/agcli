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
Schedule a coldkey swap via two-phase announcement flow.

```bash
agcli swap coldkey --new-coldkey SS58 [--password PW] [--yes]
```

**On-chain (two-phase)**:
1. `SubtensorModule::announce_coldkey_swap(origin, new_coldkey_hash)` — announces intent, starts delay
2. `SubtensorModule::swap_coldkey_announced(origin, new_coldkey)` — executes after delay period

**What migrates**: All Alpha stakes, StakingHotkeys, OwnedHotkeys, Owner mappings, SubnetOwner, full account balance, identities, AutoStakeDestination.

- Cost: swap fee recycled via `recycle_tao()`
- Can be cancelled: `SubtensorModule::dispute_coldkey_swap(origin)`
- Check status: `agcli wallet check-swap`

## Source Code
**agcli handler**: [`src/cli/network_cmds.rs`](https://github.com/unconst/agcli/blob/main/src/cli/network_cmds.rs) — `handle_swap()` at L231, Hotkey L238, Coldkey L264

**Subtensor pallet**:
- [`swap/swap_hotkey.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/swap/swap_hotkey.rs) — `swap_hotkey` extrinsic
- [`swap/swap_coldkey.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/swap/swap_coldkey.rs) — `announce_coldkey_swap`, `swap_coldkey_announced`, `dispute_coldkey_swap`
- [`macros/dispatches.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/macros/dispatches.rs) — dispatch entry points
- [`macros/events.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/macros/events.rs) — swap event definitions

## Related Commands
- `agcli wallet check-swap` — Check pending swap status
- `agcli explain --topic coldkey-swap` — Coldkey swap mechanics
