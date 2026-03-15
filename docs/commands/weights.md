# weights — Weight Setting Operations

Validators set weights to score miners on a subnet. Weights determine how emissions are distributed. Supports direct set, two-phase commit-reveal, and atomic commit-reveal workflows.

## Subcommands

### weights set
Directly set weights on a subnet. Cannot be used when commit-reveal is enabled.

```bash
agcli weights set --netuid 1 --weights "0:100,1:200,2:50" [--version-key 0] [--dry-run]
```

| Flag | Required | Description |
|------|----------|-------------|
| `--netuid` | yes | Subnet UID |
| `--weights` | yes | Comma-separated `uid:weight` pairs (u16 values) |
| `--version-key` | no | Version key for weight compatibility (default: 0) |
| `--dry-run` | no | Pre-flight check: validate without submitting |

**On-chain**: `SubtensorModule::set_weights(origin, netuid, dests, weights, version_key)`
- Storage writes: `Weights` map for the hotkey's UID
- Events: `WeightsSet(netuid, uid)`
- Pre-checks: hotkey registered, sufficient stake (>=1000τ alpha), rate limit, version key match, commit-reveal disabled
- Errors: `NotEnoughStakeToSetWeights`, `SettingWeightsTooFast`, `CommitRevealEnabled`, `IncorrectWeightVersionKey`, `WeightVecNotEqualSize`, `UidVecContainInvalidOne`

**Dry-run output** (JSON):
```json
{"dry_run": true, "netuid": 1, "num_weights": 3, "version_key": 0,
 "stake_sufficient": true, "commit_reveal_enabled": false,
 "weights_rate_limit_blocks": 100, "weights": [{"uid": 0, "weight": 100}]}
```

### weights commit
Commit a blake2 hash of weights (phase 1 of commit-reveal). Save the salt for reveal.

```bash
agcli weights commit --netuid 1 --weights "0:100,1:200" [--salt "mysecret"]
```

| Flag | Required | Description |
|------|----------|-------------|
| `--salt` | no | Salt string (auto-generated 32-char random if omitted) |

**On-chain**: `SubtensorModule::commit_crv3_weights(origin, netuid, commit_hash)`
- Hash: blake2b-256 of (uids, weights, salt)
- Events: `CRV3WeightsCommitted(account, netuid, hash)`
- Errors: `CommittingWeightsTooFast`, `CommitRevealDisabled`, `TooManyUnrevealedCommits`

### weights reveal
Reveal previously committed weights (phase 2 of commit-reveal).

```bash
agcli weights reveal --netuid 1 --weights "0:100,1:200" --salt "mysecret" [--version-key 0]
```

**On-chain**: `SubtensorModule::reveal_crv3_weights(origin, netuid, uids, values, salt, version_key)`
- Events: `CRV3WeightsRevealed(netuid, account)`
- Errors: `NoWeightsCommitFound`, `InvalidRevealCommitHashNotMatch`, `ExpiredWeightCommit`, `RevealTooEarly`

### weights commit-reveal
Atomic: commit, wait for reveal window, then auto-reveal in a single command.

```bash
agcli weights commit-reveal --netuid 1 --weights "0:100,1:200" [--version-key 0] [--wait]
```

| Flag | Description |
|------|-------------|
| `--wait` | Block until reveal is confirmed on-chain |

**Behavior**:
1. Fetches hyperparams to check if commit-reveal is enabled
2. If disabled: falls back to direct `set_weights`
3. If enabled: generates random salt, computes blake2 hash, commits
4. Waits for reveal window (based on `commit_reveal_period` and `tempo`)
5. Auto-reveals with the stored weights and salt

### weights status
Check commit status for your hotkey on a subnet.

```bash
agcli weights status --netuid 1
```

## Weight Format
Weights are comma-separated `uid:weight` pairs where:
- `uid` = neuron UID (u16, must exist in metagraph)
- `weight` = weight value (u16, 0-65535)

Weights are normalized on-chain to sum to 1.0 (u16::MAX).

## Commit-Reveal Flow
```
1. agcli weights commit --netuid N --weights "..." [--salt S]
   → saves salt (print to stdout)
2. Wait for reveal window (commit_reveal_period blocks after commit)
3. agcli weights reveal --netuid N --weights "..." --salt S
   → must match exact same weights and salt
```

Or use `agcli weights commit-reveal` to do both automatically.

## Common Errors
| Error | Cause | Fix |
|-------|-------|-----|
| `NotEnoughStakeToSetWeights` | Hotkey alpha < ~1000τ on subnet | Stake more on this subnet |
| `SettingWeightsTooFast` | Rate limit not expired | Wait `weights_rate_limit` blocks |
| `CommitRevealEnabled` | Used `set` when CR is on | Use `commit-reveal` instead |
| `CommitRevealDisabled` | Used `commit` when CR is off | Use `set` instead |
| `InvalidRevealCommitHashNotMatch` | Wrong weights or salt on reveal | Use exact same values from commit |
| `ExpiredWeightCommit` | Reveal window passed | Re-commit and reveal sooner |
| `RevealTooEarly` | Reveal window not open yet | Wait for reveal window |
| `UidVecContainInvalidOne` | UID not in metagraph | Check `agcli subnet metagraph` |

## Related Commands
- `agcli subnet hyperparams --netuid N` — Check weights_rate_limit, commit_reveal settings
- `agcli subnet watch --netuid N` — Live tempo countdown and weight window status
- `agcli subnet commits --netuid N` — See pending commits
- `agcli explain --topic commit-reveal` — How commit-reveal works
- `agcli explain --topic rate-limits` — Weight rate limit details
- `agcli explain --topic yuma` — How weights feed into consensus
