# identity — On-Chain Identity

Set and query on-chain identity information for hotkeys and subnets.

## Subcommands

### identity show
Query on-chain identity for an address.

```bash
agcli identity show --address SS58
# JSON: {"name", "url", "github", "description", "image", "discord"}
```

### identity set
Set identity information for your hotkey.

```bash
agcli identity set --name "MyValidator" [--url "https://..."] [--github "user/repo"] [--description "..."]
```

**On-chain**: `SubtensorModule::set_identity(origin, name, url, github_repo, image, discord, description, additional)`
- Requires hotkey to be the signer
- Events: identity storage updated

### identity set-subnet
Set identity for a subnet (owner only).

```bash
agcli identity set-subnet --netuid 1 --name "MySubnet" [--github "..."] [--url "..."]
```

**On-chain**: `SubtensorModule::set_subnet_identity(origin, netuid, subnet_name, github_repo, subnet_contact, subnet_url, discord, description, logo_url, additional)`
- Errors: `NotSubnetOwner`

## Related Commands
- `agcli view account` — See identity in account overview
- `agcli delegate show` — Validator identity
