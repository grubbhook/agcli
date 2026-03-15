# explain — Built-in Concept Reference

Built-in educational reference for Bittensor concepts. 33 topics covering all major protocol mechanics.

## Usage

### List all topics
```bash
agcli explain
# JSON: [{"topic", "description"}]
```

### Explain a specific topic
```bash
agcli explain --topic tempo
# JSON: {"topic", "content"}
```

## Available Topics (33)
| Topic | Description |
|-------|-------------|
| `tempo` | Block cadence for subnet weight evaluation |
| `commit-reveal` | Two-phase weight submission scheme |
| `yuma` | Yuma consensus — the incentive mechanism |
| `rate-limits` | Weight setting frequency constraints |
| `stake-weight` | Minimum stake required to set weights |
| `amm` | Automated Market Maker (Dynamic TAO pools) |
| `bootstrap` | Getting started as a new subnet owner |
| `alpha` | Subnet-specific alpha tokens |
| `emission` | How TAO emissions are distributed |
| `registration` | Registering neurons on subnets |
| `subnets` | What subnets are and how they work |
| `validators` | Validator role and responsibilities |
| `miners` | Miner role and responsibilities |
| `immunity` | Immunity period for new registrations |
| `delegation` | Delegating/nominating stake to validators |
| `childkeys` | Childkey take and delegation within subnets |
| `root` | Root network (SN0) and root weights |
| `proxy` | Proxy accounts for delegated signing |
| `coldkey-swap` | Coldkey swap scheduling and security |
| `governance` | On-chain governance and proposals |
| `senate` | Senate / triumvirate governance body |
| `mev-shield` | MEV protection on Bittensor |
| `limits` | Network and chain operational limits |
| `hyperparams` | Subnet hyperparameters reference |
| `axon` | Axon serving endpoint for miners/validators |
| `take` | Validator/delegate take percentage |
| `recycle` | Recycling and burning alpha tokens |
| `pow` | Proof-of-work registration mechanics |
| `archive` | Archive nodes and historical data queries |
| `diff` | Compare chain state between two blocks |
| `owner-workflow` | Step-by-step guide for subnet owners |

## Related
- `docs/commands/*.md` — Detailed command reference
- `docs/tutorials/` — Step-by-step guides
