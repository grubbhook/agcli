# transfer — TAO Transfer Operations

Send TAO tokens between accounts. Uses the Substrate Balances pallet (not SubtensorModule).

## Subcommands

### transfer
Send a specific amount of TAO to another account.

```bash
agcli transfer --dest 5Dest... --amount 10.0 [--password PW] [--yes]
```

| Flag | Required | Description |
|------|----------|-------------|
| `--dest` | yes | Destination SS58 address |
| `--amount` | yes | TAO to send (decimal, e.g. 10.5) |

**On-chain**: `Balances::transfer_allow_death(origin, dest, value)`
- 1 TAO = 1,000,000,000 RAO (u64)
- Events: `Transfer { from, to, amount }`
- Pre-checks: balance >= amount (checked client-side before submission)
- Confirmation prompt unless `--yes` or `--batch`
- Errors: `InsufficientBalance`, `ExistentialDeposit` (if transfer would kill sender account)

**Output** (JSON mode):
```json
{"tx_hash": "0x...", "message": "Transaction submitted."}
```

### transfer-all
Send entire balance to another account (minus transaction fees).

```bash
agcli transfer-all --dest 5Dest... [--keep-alive] [--password PW] [--yes]
```

| Flag | Description |
|------|-------------|
| `--keep-alive` | Keep sender account alive (leave existential deposit) |

**On-chain**: `Balances::transfer_all(origin, dest, keep_alive)`
- If `keep_alive=false`: sends everything, account may be reaped
- If `keep_alive=true`: leaves existential deposit (500,000 RAO = 0.0005 TAO)
- Extra confirmation required: "Transfer ALL funds? This will empty your account."

## Common Errors
| Error | Cause | Fix |
|-------|-------|-----|
| `InsufficientBalance` | Not enough TAO | Check `agcli balance` |
| `ExistentialDeposit` | Transfer would kill account | Use `--keep-alive` or transfer less |
| Invalid SS58 | Bad destination address | Verify address format (prefix 42) |

## Fees
- Transaction fee: ~0.000125 TAO per transfer (varies with network load)
- Existential deposit: 0.0005 TAO (500,000 RAO)

## Related Commands
- `agcli balance` — Check balance before transfer
- `agcli view history` — See recent transactions
- `agcli stake add` — Stake TAO instead of transferring
