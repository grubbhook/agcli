# Staking Guide

## How Staking Works in Bittensor

Staking means locking TAO on a subnet to support validators and earn rewards. With Dynamic TAO, each subnet has its own **alpha token** — when you stake, you buy alpha at the current market price via an AMM (Automated Market Maker).

**Key concepts:**
- **TAO pool** — the amount of TAO locked in a subnet's AMM
- **Alpha price** — how much TAO one unit of alpha costs (fluctuates with supply/demand)
- **Emission** — new TAO minted each block, distributed across subnets based on root weights
- **APY** — estimated annual return based on your share of the subnet's emission pool

When you stake, you're buying alpha. When you unstake, you sell alpha back for TAO. Prices move, so you may get more or less TAO than you originally staked.

## Before You Stake

Check your balance and research subnets:

```bash
# Your balance
agcli balance

# List subnets with key metrics
agcli subnet list

# Dynamic TAO info (prices, pool sizes, emissions)
agcli view dynamic

# Deep-dive on a specific subnet
agcli view subnet-analytics --netuid 1

# Simulate a swap before committing
agcli view swap-sim --netuid 1 --tao 100.0
```

## Staking Wizard (Recommended for Beginners)

The wizard walks you through subnet selection and amount choice:

```bash
agcli stake wizard

# Fully non-interactive:
agcli stake wizard --netuid 1 --amount 10.0 --password mypass --yes
```

## Basic Staking

```bash
# Stake 100 TAO on subnet 1 (uses default hotkey)
agcli stake add 100.0 --netuid 1

# Stake with a specific hotkey
agcli stake add 50.0 --netuid 1 --hotkey 5HotkeyAddress...

# View all your positions
agcli stake list

# View APY estimates for each position
agcli view staking-analytics
```

## Unstaking

```bash
# Remove 25 TAO worth of alpha from subnet 1
agcli stake remove 25.0 --netuid 1

# Unstake everything from a specific hotkey
agcli stake unstake-all --hotkey 5HotkeyAddress...

# Unstake all alpha tokens across all subnets
agcli stake unstake-all-alpha --hotkey 5HotkeyAddress...
```

## Moving and Swapping Stake

```bash
# Move alpha between subnets (same coldkey, same hotkey)
agcli stake move 10.0 --from 1 --to 3

# Swap stake between hotkeys on the same subnet
agcli stake swap 5.0 --netuid 1 --from-hotkey 5A... --to-hotkey 5B...
```

## Limit Orders (Price-Conditional Staking)

Limit orders let you stake/unstake only if the alpha price meets your conditions. Useful for avoiding bad entries during high volatility.

```bash
# Add stake only if alpha price <= 0.5 TAO/α
agcli stake add-limit 100.0 --netuid 1 --price 0.5

# Allow partial fills (get as much as possible at or below the price)
agcli stake add-limit 100.0 --netuid 1 --price 0.5 --partial

# Remove stake only if price >= 0.8 TAO/α
agcli stake remove-limit 50.0 --netuid 1 --price 0.8

# Swap between subnets with a price limit
agcli stake swap-limit 10.0 --from 1 --to 3 --price 0.6 --partial
```

## Alpha Operations

```bash
# Burn alpha tokens (permanently removes from supply, increases price for others)
agcli stake burn-alpha 100.0 --netuid 1

# Recycle alpha back to TAO (goes through the AMM)
agcli stake recycle-alpha 100.0 --netuid 1
```

## Claiming Root Dividends

Root network validators earn dividends from all subnets:

```bash
agcli stake claim-root --netuid 1
```

## Delegation Management

If you run a validator, nominators delegate to your hotkey and you earn a take percentage:

```bash
# View your delegate info
agcli delegate show

# Decrease take (immediate effect — signals trust to nominators)
agcli delegate decrease-take 10.0

# Increase take (rate-limited to prevent abuse)
agcli delegate increase-take 12.0

# View who delegates to a hotkey
agcli view nominations --hotkey 5Hotkey...
```

## Childkey Delegation

Validators can delegate to child validators and set a take percentage:

```bash
# Set your childkey take to 5%
agcli stake childkey-take 5.0 --netuid 1

# Delegate to children (proportion:hotkey format, proportions are relative weights)
agcli stake set-children --netuid 1 --children "50:5Child1...,50:5Child2..."
```

## Strategy Tips

1. **Diversify** — spread stakes across multiple subnets to reduce risk
2. **Check APY** — use `agcli view staking-analytics` to compare yields across your positions
3. **Watch slippage** — use `agcli view swap-sim` before large stakes to estimate price impact
4. **Use limit orders** — protect against sudden price moves with `add-limit`/`remove-limit`
5. **Monitor emissions** — subnets with higher emission/TAO-in ratios tend to have higher APY
6. **Stake with top validators** — validators with higher VTrust and more subnet registrations tend to earn more consistently

## Troubleshooting

**"Insufficient balance"** — you don't have enough free TAO. Check with `agcli balance`.

**"HotKeyNotRegisteredInSubNet"** — the hotkey isn't registered on that subnet. Register first with `agcli subnet register-neuron`.

**"StakeRateLimitExceeded"** — too many stake operations too quickly. Wait a few blocks (~12 seconds each) and retry.

**"TxRateLimitExceeded"** — general rate limit. Wait and retry.

**Staked amount differs from expected** — alpha prices fluctuate. Use `agcli view swap-sim` to preview the swap before committing. Consider `--partial` limit orders.
