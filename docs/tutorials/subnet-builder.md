# Subnet Builder Guide

This guide covers creating and managing a Bittensor subnet, registering neurons, setting up miners and validators, and managing weights.

## Creating a Subnet

Subnet registration costs TAO (the lock cost decreases over time after each registration).

```bash
# Register a new subnet (uses your default hotkey)
agcli subnet register

# View your new subnet
agcli subnet list
```

## Setting Subnet Identity

Give your subnet a name, description, and links visible on explorers:

```bash
agcli identity set-subnet 42 --name "MySubnet" --github "user/repo" --url "https://example.com"
```

## Registering Neurons

Miners and validators must register on a subnet before participating.

### Burn Registration (Fastest)

Burns TAO to register immediately:

```bash
agcli subnet register-neuron 42
```

### POW Registration (Free)

Solves a proof-of-work puzzle to register without burning TAO. Harder on competitive subnets.

```bash
# Use 8 CPU threads for POW solving
agcli subnet pow 42 --threads 8
```

**Tips:**
- More threads = faster solve, but higher CPU usage
- If POW fails after max attempts, consider burn registration instead
- Check difficulty first: `agcli subnet hyperparams 42` (look at `difficulty`)

## Serving an Axon (Miners)

After registration, miners must announce their network endpoint:

```bash
# Serve axon on subnet 42
agcli serve axon --netuid 42 --ip 1.2.3.4 --port 8091
```

Or via the Rust SDK:

```rust
use agcli::{Client, types::chain_data::AxonInfo};

let client = Client::connect("wss://entrypoint-finney.opentensor.ai:443").await?;
let axon = AxonInfo {
    block: 0, version: 1,
    ip: "1234".to_string(),  // IP as u128 encoded
    port: 8091, ip_type: 4, protocol: 0,
};
client.serve_axon(&keypair, 42.into(), &axon).await?;
```

## Setting Weights (Validators)

Validators rank miners by setting weights. This determines how emissions are distributed.

```bash
# Set weights: UID 0 gets weight 100, UID 1 gets 200
agcli weights set --netuid 42 "0:100,1:200"

# View the metagraph to see current weights/ranks
agcli subnet metagraph 42
```

### Commit-Reveal Weights

Some subnets require commit-reveal to prevent weight-copying:

```bash
# Step 1: Commit (generates a random salt)
agcli weights commit --netuid 42 "0:100,1:200"
# Output: salt = "abc123..."  — SAVE THIS

# Step 2: Reveal (after the commit-reveal period passes)
agcli weights reveal --netuid 42 "0:100,1:200" abc123...
```

Check if a subnet uses commit-reveal:

```bash
agcli subnet hyperparams 42
# Look for: commit_reveal_weights = true
```

### Batch Weight Operations

For validators on multiple subnets:

```bash
# Batch set weights across subnets (SubtensorModule native batch)
# See docs/llm.txt for SDK examples
```

## Root Network

The root network (netuid 0) governs emission distribution to all subnets:

```bash
# Register on root (requires large stake)
agcli root register

# Set root weights (determines how emissions flow to subnets)
agcli root weights "1:100,2:200,3:50"
```

## Monitoring Your Subnet

```bash
# Subnet analytics — neurons, stake, emissions, top miners/validators
agcli view subnet-analytics --netuid 42

# Live metagraph (refreshes every N seconds)
agcli subnet metagraph 42 --live 30

# Live dynamic TAO (prices, pools)
agcli view dynamic --live 30

# Subscribe to chain events
agcli subscribe events --filter stakes
agcli subscribe events --filter weights
agcli subscribe blocks
```

## Subnet Hyperparameters Reference

View with `agcli subnet hyperparams <netuid>`. Key parameters:

| Parameter | Description |
|-----------|-------------|
| `tempo` | Blocks between epochs (emission distribution cycles) |
| `immunity_period` | Blocks before a new neuron can be pruned |
| `min_allowed_weights` | Minimum number of weight entries a validator must set |
| `max_weights_limit` | Maximum weight value per entry (u16) |
| `min_burn` / `max_burn` | TAO range for burn registration cost |
| `difficulty` | POW difficulty (higher = harder to register) |
| `registration_allowed` | Whether new neurons can register |
| `target_regs_per_interval` | Target registration rate per tempo |
| `commit_reveal_weights_enabled` | Whether weights must use commit-reveal |
| `commit_reveal_period` | Blocks between commit and reveal |
| `liquid_alpha_enabled` | Whether the subnet uses Dynamic TAO |
| `serving_rate_limit` | Minimum blocks between axon updates |
| `weights_rate_limit` | Minimum blocks between weight updates |

## Dissolving a Subnet

If you're the owner and want to remove your subnet:

```bash
agcli subnet dissolve 42
```

**Warning:** This is irreversible and destroys all neuron registrations on the subnet.

## Troubleshooting

**"TooManyRegistrationsThisBlock"** — too many neurons registered this block. Wait ~12 seconds and retry.

**"NotOwner"** — you're not the subnet owner. Only the coldkey that registered the subnet can change its identity or dissolve it.

**"WeightsNotSettable"** — the subnet may require commit-reveal, or you're hitting the weights rate limit. Check hyperparams.

**"AlreadyRegistered"** — this hotkey is already registered on the subnet. Use a different hotkey.

**POW fails with "not found after N attempts"** — difficulty is too high for the given attempt count. Increase threads or switch to burn registration.
