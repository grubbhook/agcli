# proxy — Proxy Account Management

Delegate signing authority to another account. Proxy accounts can sign transactions on behalf of the delegator, filtered by operation type.

## Subcommands

### proxy add
Add a proxy delegate.

```bash
agcli proxy add --delegate SS58 [--proxy-type staking] [--delay 0]
```

**On-chain**: `Proxy::add_proxy(origin, delegate, proxy_type, delay)`

### proxy remove
Remove a proxy delegate.

```bash
agcli proxy remove --delegate SS58 [--proxy-type staking] [--delay 0]
```

### proxy list
List all proxy delegates for an account.

```bash
agcli proxy list [--address SS58]
# JSON: [{"delegate", "proxy_type", "delay"}]
```

## Proxy Types
| Type | Allowed Operations |
|------|-------------------|
| `any` | All operations |
| `owner` | Subnet owner operations |
| `staking` | Stake add/remove/move only |
| `non_transfer` | Everything except transfers |
| `non_critical` | Non-critical operations |
| `governance` | Governance voting |
| `senate` | Senate operations |

## Using Proxied Operations
Any agcli command can be run through a proxy with `--proxy SS58`:

```bash
agcli --proxy 5RealAccount... stake add --amount 10 --netuid 1
```

This wraps the extrinsic in `Proxy.proxy(real_account, call)`.

## Source Code
**agcli handler**: [`src/cli/network_cmds.rs`](https://github.com/unconst/agcli/blob/main/src/cli/network_cmds.rs) — `handle_proxy()` at L500, subcommands: Add L509, CreatePure L545, KillPure L563, List L581

**Substrate pallet**: Uses standard `Proxy` pallet (`Proxy::add_proxy`, `Proxy::remove_proxy`, `Proxy::create_pure`, `Proxy::kill_pure`).

## Related Commands
- `agcli explain --topic proxy` — How proxies work on Bittensor
