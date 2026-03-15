# stake — Staking Operations

Lock TAO on subnets behind hotkeys to earn emission rewards. Staking converts TAO into subnet-specific alpha tokens via the AMM pool.

## Subcommands

### stake add
Stake TAO on a subnet. Converts TAO → alpha via the subnet's AMM pool.

```bash
agcli stake add --amount 10.0 --netuid 1 [--hotkey SS58] [--max-slippage 2.0] [--password PW] [--yes]
```

| Flag | Required | Description |
|------|----------|-------------|
| `--amount` | yes | TAO to stake (decimal, e.g. 10.5) |
| `--netuid` | yes | Subnet UID (u16) |
| `--hotkey` | no | Hotkey SS58 (defaults to wallet hotkey) |
| `--max-slippage` | no | Max slippage % — aborts if AMM price impact exceeds this |

**On-chain**: `SubtensorModule::add_stake(origin, hotkey, netuid, amount_staked)`
- Flow: withdraw TAO from coldkey → `stake_into_subnet()` → `swap_tao_for_alpha()` via AMM → increase alpha shares
- Storage: `Alpha(hotkey, coldkey, netuid)` shares, `TotalHotkeyAlpha`, `SubnetTAO`, `SubnetAlphaIn/Out`, `TotalStake`
- Events: `StakeAdded(coldkey, hotkey, tao_amount, alpha_amount, netuid, block)`
- Pre-checks: balance >= amount, spending limit check, slippage simulation
- Errors: `NotEnoughBalanceToStake`, `HotKeyAccountNotExists`, `StakingRateLimitExceeded`, `InsufficientLiquidity`

### stake remove
Unstake alpha from a subnet. Converts alpha → TAO via the AMM pool.

```bash
agcli stake remove --amount 5.0 --netuid 1 [--hotkey SS58] [--max-slippage 2.0]
```

**On-chain**: `SubtensorModule::remove_stake(origin, hotkey, netuid, amount_unstaked)`
- Flow: decrease alpha shares → `unstake_from_subnet()` → `swap_alpha_for_tao()` via AMM → deposit TAO to coldkey
- Events: `StakeRemoved(coldkey, hotkey, tao_amount, alpha_amount, netuid, block)`
- Errors: `NotEnoughStakeToWithdraw`, `StakingRateLimitExceeded`, `InsufficientLiquidity`

### stake list
Show all stakes for a coldkey across all subnets.

```bash
agcli stake list [--address SS58] [--at-block N]
# JSON output: [{"netuid", "hotkey", "stake_rao", "alpha_raw"}]
```

**On-chain**: reads `StakingHotkeys` → per-hotkey `Stake` entries. No extrinsic.

### stake move
Move alpha between subnets (same hotkey). Sells alpha on source subnet, buys on destination.

```bash
agcli stake move --amount 5.0 --from 1 --to 2 [--hotkey SS58]
```

**On-chain**: `SubtensorModule::move_stake(origin, hotkey, origin_netuid, destination_netuid, alpha_amount)`
- Events: `StakeMoved(coldkey, origin_hotkey, origin_netuid, dest_hotkey, dest_netuid, tao_equivalent)`
- Two AMM operations: `swap_alpha_for_tao()` on source, `swap_tao_for_alpha()` on destination
- All move/swap/transfer operations funnel through `transition_stake_internal()`

### stake swap
Swap stake between hotkeys on the same subnet.

```bash
agcli stake swap --amount 5.0 --netuid 1 --from-hotkey HK1 --to-hotkey HK2
```

**On-chain**: `SubtensorModule::swap_stake(origin, from_hotkey, from_netuid, to_netuid, alpha_amount)`

### stake unstake-all
Unstake all alpha from a hotkey across all subnets.

```bash
agcli stake unstake-all [--hotkey SS58]
```

### stake add-limit / remove-limit / swap-limit
Limit orders for staking operations. Execute when AMM price reaches target.

```bash
agcli stake add-limit --amount 10.0 --netuid 1 --price 0.5 [--partial] [--hotkey SS58]
agcli stake remove-limit --amount 5.0 --netuid 1 --price 0.8 [--partial] [--hotkey SS58]
agcli stake swap-limit --amount 5.0 --from 1 --to 2 --price 0.5 [--partial] [--hotkey SS58]
```

| Flag | Description |
|------|-------------|
| `--price` | Target price in TAO per alpha (decimal) |
| `--partial` | Allow partial fill at target price |

### stake childkey-take
Set the childkey take percentage for a hotkey on a subnet.

```bash
agcli stake childkey-take --take 10.0 --netuid 1 [--hotkey SS58]
```

**On-chain**: `SubtensorModule::set_childkey_take(origin, hotkey, netuid, take)` where take is u16 (pct * 65535 / 100)
- Errors: `InvalidChildkeyTake`, `TxChildkeyTakeRateLimitExceeded`

### stake set-children
Delegate weight to child hotkeys on a subnet.

```bash
agcli stake set-children --netuid 1 --children "0.5:5Child1...,0.3:5Child2..."
```

**On-chain**: `SubtensorModule::set_children(origin, hotkey, netuid, children_with_proportions)`
- Errors: `InvalidChild`, `DuplicateChild`, `ProportionOverflow`, `TooManyChildren` (max 5)

### stake recycle-alpha
Recycle alpha tokens back to TAO (burns alpha, credits TAO to the subnet's emission pool).

```bash
agcli stake recycle-alpha --amount 100.0 --netuid 1 [--hotkey SS58]
```

### stake burn-alpha
Permanently burn alpha tokens (deflationary — no TAO credited back).

```bash
agcli stake burn-alpha --amount 50.0 --netuid 1 [--hotkey SS58]
```

### stake unstake-all-alpha
Unstake all alpha across all subnets for a hotkey.

```bash
agcli stake unstake-all-alpha [--hotkey SS58]
```

### stake claim-root
Claim root network dividends for a specific subnet.

```bash
agcli stake claim-root --netuid 1 [--hotkey SS58]
```

**On-chain**: `SubtensorModule::claim_root_dividends(origin, hotkey, netuid)`

### stake process-claim
Batch claim root dividends across multiple subnets.

```bash
agcli stake process-claim [--hotkey SS58] [--netuids "1,2,3"]
```

Iterates over all subnets where the hotkey has stake and calls `claim_root_dividends` for each.

### stake set-auto
Set automatic staking destination for a subnet.

```bash
agcli stake set-auto --netuid 1 [--hotkey SS58]
```

### stake show-auto
Show auto-stake destinations for a coldkey.

```bash
agcli stake show-auto [--address SS58]
```

### stake set-claim
Set how root emissions are handled (swap to TAO, keep as alpha, or keep for specific subnets).

```bash
agcli stake set-claim --claim-type swap|keep|keep-subnets [--subnets "1,2,3"]
```

### stake transfer-stake
Transfer stake to a different coldkey owner.

```bash
agcli stake transfer-stake --dest 5Dest... --amount 10.0 --from 1 --to 2 [--hotkey SS58]
```

**On-chain**: `SubtensorModule::transfer_stake(origin, destination_coldkey, hotkey, origin_netuid, destination_netuid, alpha_amount)`

### stake wizard
Interactive or fully-scripted staking wizard.

```bash
agcli stake wizard [--netuid 1] [--amount 5.0] [--hotkey SS58] [--password PW] [--yes]
```

## Global Flags That Affect Staking
- `--mev` — Encrypt staking extrinsic via MEV shield (ML-KEM-768)
- `--dry-run` — Show what would be submitted without broadcasting
- `--output json` — Machine-readable JSON output
- `--batch` / `--yes` — Non-interactive mode

## Common Errors
| Error | Cause | Fix |
|-------|-------|-----|
| `NotEnoughBalanceToStake` | Coldkey balance < stake amount | Check `agcli balance` |
| `StakingRateLimitExceeded` | Too many stake ops in short time | Wait and retry |
| `NotEnoughStakeToWithdraw` | Unstake amount > staked amount | Check `agcli stake list` |
| `HotKeyAccountNotExists` | Hotkey not registered on chain | Register hotkey first |
| `TooManyChildren` | >5 children set | Reduce child count |
| `AmountTooLow` | Stake amount below minimum | Increase amount |

## Related Commands
- `agcli balance` — Check balance before staking
- `agcli view portfolio` — See all stakes and positions
- `agcli subnet show --netuid N` — Check subnet AMM pool depth
- `agcli view swap-sim --netuid N --tao X` — Simulate swap before staking
- `agcli explain --topic stake-weight` — Min stake for weight setting
