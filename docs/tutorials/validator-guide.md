# Validator Guide

This guide covers setting up and running a Bittensor validator using agcli.

## What Validators Do

Validators evaluate miners by setting weights, which determines emission distribution. In return, validators earn dividends proportional to their stake and performance.

## Setup

### 1. Create Wallet and Keys

```bash
# Create a coldkey (spending key — keep the mnemonic safe)
agcli wallet create --name validator

# Create a hotkey (operational key — used for on-chain validator operations)
agcli wallet create-hotkey --name validator --hotkey-name default
```

### 2. Fund Your Coldkey

Transfer TAO to your coldkey address:

```bash
agcli wallet show -w validator
# Note the coldkey SS58 address, then fund it from an exchange or another wallet
```

### 3. Register on Subnets

```bash
# Burn register on subnet 1
agcli subnet register-neuron 1 -w validator

# Register on root network (for emission governance)
agcli root register -w validator
```

### 4. Stake TAO

```bash
# Stake on subnet 1 using your hotkey
agcli stake add 1000.0 --netuid 1 -w validator

# Or use the wizard for guided staking
agcli stake wizard -w validator
```

## Setting Weights

This is the core validator activity — ranking miners based on their performance:

```bash
# Rate miners: UID 0 gets weight 500 (best), UID 3 gets 200, UID 7 gets 100
agcli weights set --netuid 1 "0:500,3:200,7:100" -w validator
```

### Commit-Reveal (if required by the subnet)

```bash
# Commit your weights (save the generated salt!)
agcli weights commit --netuid 1 "0:500,3:200,7:100" -w validator

# After the commit-reveal interval, reveal
agcli weights reveal --netuid 1 "0:500,3:200,7:100" YOUR_SALT -w validator
```

## Delegation (Attracting Nominators)

Nominators delegate TAO to validators. Set a competitive take rate:

```bash
# View your current delegate status
agcli delegate show -w validator

# Lower your take to attract more nominators (takes effect immediately)
agcli delegate decrease-take 9.0 -w validator

# View who delegates to you
agcli view nominations --hotkey YOUR_HOTKEY_SS58
```

## Monitoring

```bash
# Your portfolio (balance + all stakes)
agcli view portfolio -w validator

# Staking analytics (APY, daily yield projections)
agcli view staking-analytics -w validator

# Subnet metagraph (see your rank, trust, dividends)
agcli subnet metagraph 1

# Live metagraph with auto-refresh
agcli subnet metagraph 1 --live 60

# Account explorer (full overview of any address)
agcli view account --address YOUR_COLDKEY_SS58

# Chain events for your subnet
agcli subscribe events --filter stakes
```

## Multi-Subnet Validation

Top validators run on multiple subnets:

```bash
# Register on additional subnets
agcli subnet register-neuron 3 -w validator
agcli subnet register-neuron 8 -w validator

# Stake across subnets
agcli stake add 500.0 --netuid 3 -w validator
agcli stake add 500.0 --netuid 8 -w validator

# Set weights on each subnet
agcli weights set --netuid 3 "0:100,1:200" -w validator
agcli weights set --netuid 8 "0:300,2:100,5:50" -w validator

# Set root weights (governs emission flow to subnets you support)
agcli root weights "1:100,3:200,8:150" -w validator
```

## Childkey Delegation

Delegate part of your validator work to child keys:

```bash
# Set childkey take to 5%
agcli stake childkey-take 5.0 --netuid 1 -w validator

# Delegate 50/50 to two child validators
agcli stake set-children --netuid 1 --children "50:5ChildA...,50:5ChildB..." -w validator
```

## Automation (Agent/Script Mode)

All commands support non-interactive flags for automation:

```bash
# Environment setup
export AGCLI_WALLET=validator
export AGCLI_PASSWORD=mypassword
export AGCLI_YES=1

# Now all commands run non-interactively
agcli stake add 10.0 --netuid 1
agcli weights set --netuid 1 "0:100,1:200"

# JSON output for parsing in scripts
agcli view portfolio --output json | jq '.positions[].netuid'
```

## Key Metrics to Watch

- **VTrust** — your validator trust score (higher = more trusted by other validators)
- **Dividends** — your share of subnet emissions (proportional to VTrust and stake)
- **Stake** — total TAO staked through your hotkey (your own + nominators)
- **APY** — estimated annual yield from `agcli view staking-analytics`
- **Emission** — raw emission per block for your UID from the metagraph
