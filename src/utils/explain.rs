//! Built-in Bittensor concept reference for `agcli explain <concept>`.

/// Return the explanation text for a concept, or None if not found.
pub fn explain(topic: &str) -> Option<&'static str> {
    match topic.to_lowercase().replace(['-', '_'], "").as_str() {
        "tempo" => Some(TEMPO),
        "commitreveal" | "cr" => Some(COMMIT_REVEAL),
        "yuma" | "yumaconsensus" => Some(YUMA),
        "ratelimit" | "ratelimits" | "weightsratelimit" => Some(RATE_LIMITS),
        "stakeweight" | "stakeweightminimum" | "1000" => Some(STAKE_WEIGHT),
        "amm" | "dynamictao" | "dtao" | "pool" => Some(AMM),
        "bootstrap" => Some(BOOTSTRAP),
        "alpha" | "alphatoken" => Some(ALPHA),
        "emission" | "emissions" => Some(EMISSION),
        "registration" | "register" => Some(REGISTRATION),
        "subnet" | "subnets" => Some(SUBNETS),
        "validator" | "validators" => Some(VALIDATORS),
        "miner" | "miners" => Some(MINERS),
        "immunity" | "immunityperiod" => Some(IMMUNITY),
        "delegate" | "delegation" | "nominate" => Some(DELEGATION),
        "childkey" | "childkeys" => Some(CHILDKEYS),
        "root" | "rootnetwork" => Some(ROOT_NETWORK),
        "proxy" => Some(PROXY),
        "coldkeyswap" | "coldkey" | "ckswap" => Some(COLDKEY_SWAP),
        topics => {
            // Fuzzy: check if the topic is a substring of any key
            let all = list_topics();
            for (key, _) in &all {
                if key.contains(topics) {
                    return explain(key);
                }
            }
            None
        }
    }
}

/// List all available topics with short descriptions.
pub fn list_topics() -> Vec<(&'static str, &'static str)> {
    vec![
        ("tempo", "Block cadence for subnet weight evaluation"),
        ("commit-reveal", "Two-phase weight submission scheme"),
        ("yuma", "Yuma consensus — the incentive mechanism"),
        ("rate-limits", "Weight setting frequency constraints"),
        ("stake-weight", "Minimum stake required to set weights"),
        ("amm", "Automated Market Maker (Dynamic TAO pools)"),
        ("bootstrap", "Getting started as a new subnet owner"),
        ("alpha", "Subnet-specific alpha tokens"),
        ("emission", "How TAO emissions are distributed"),
        ("registration", "Registering neurons on subnets"),
        ("subnets", "What subnets are and how they work"),
        ("validators", "Validator role and responsibilities"),
        ("miners", "Miner role and responsibilities"),
        ("immunity", "Immunity period for new registrations"),
        ("delegation", "Delegating/nominating stake to validators"),
        ("childkeys", "Childkey take and delegation within subnets"),
        ("root", "Root network (SN0) and root weights"),
        ("proxy", "Proxy accounts for delegated signing"),
        ("coldkey-swap", "Coldkey swap scheduling and security"),
    ]
}

const TEMPO: &str = "\
TEMPO
=====
Tempo is the number of blocks between weight evaluation rounds on a subnet.

- Each subnet has its own tempo (e.g., 360 blocks ≈ 72 minutes at 12s/block).
- At each tempo boundary, Yuma consensus runs: weights are evaluated, ranks
  computed, and emissions distributed.
- Miners/validators are scored based on weights set during the tempo.
- Check a subnet's tempo: `agcli subnet hyperparams <netuid>`
- Blocks until next tempo = tempo - (current_block % tempo)

Practical impact:
- Weight changes only take effect at the next tempo boundary.
- If you set weights right after a tempo, you wait the full cycle.
- Plan your weight updates to land before the next tempo.";

const COMMIT_REVEAL: &str = "\
COMMIT-REVEAL
=============
A two-phase weight submission scheme that prevents weight copying.

Phase 1 — COMMIT: Hash your weights + a secret salt, submit the hash on-chain.
Phase 2 — REVEAL: After a waiting period, reveal the actual weights + salt.

Why it exists:
- Without commit-reveal, validators can observe others' weight transactions in
  the mempool and copy them before the tempo evaluates. This is a form of
  'weight mimicry' that undermines honest scoring.
- Commit-reveal ensures weights stay secret until the reveal window.

How to use it:
  # Commit (saves the salt — keep it!)
  agcli weights commit --netuid 97 \"0:100,1:200\" --salt mysecret

  # Wait for the reveal window (check commit_reveal_weights_interval in hyperparams)

  # Reveal (must use same weights + salt)
  agcli weights reveal --netuid 97 \"0:100,1:200\" mysecret

Check if a subnet uses commit-reveal:
  agcli subnet hyperparams <netuid>  →  commit_reveal_weights = true/false

The commit_reveal_weights_interval hyperparam controls how many tempos
you must wait before revealing.";

const YUMA: &str = "\
YUMA CONSENSUS
==============
Yuma consensus is Bittensor's incentive mechanism. It determines how emissions
are distributed based on validator weight-setting agreements.

How it works:
1. Validators set weights on miners based on perceived performance.
2. At each tempo, the chain aggregates all validator weights.
3. Consensus is reached: miners that multiple validators agree on get higher
   incentive scores. The consensus mechanism rewards agreement.
4. Emissions split: miners get incentive-based share, validators get
   dividends proportional to how well their weights matched consensus.

Key metrics (visible in metagraph):
- Trust:      How much a miner's performance is trusted by validators
- Consensus:  Degree of agreement on a miner's value
- Incentive:  Final score → determines miner's emission share
- Dividends:  Validator's emission share for accurate scoring
- VTrust:     Validator trust — how well a validator's weights match consensus

Why it matters:
- Validators who set accurate weights earn more dividends.
- Miners who deliver real value to multiple validators earn more incentive.
- Gaming the system (weight copying, collusion) is penalized by consensus.

View metagraph: `agcli subnet metagraph <netuid>`";

const RATE_LIMITS: &str = "\
RATE LIMITS
===========
The chain enforces rate limits on weight-setting to prevent spam and ensure stability.

weights_rate_limit: Number of blocks you must wait between set_weights calls.
  - Typical: 100 blocks (≈20 minutes at 12s/block)
  - If you try to set weights before the limit expires, the extrinsic fails.

tx_rate_limit: Global transaction rate limit per account.

How to check:
  agcli subnet hyperparams <netuid>  →  weights_rate_limit

Practical tips:
- Before calling set_weights, check when you last set weights.
- The error 'SettingWeightsTooFast' or 'TxRateLimitExceeded' means you need to wait.
- Rate limits apply per-hotkey per-subnet, not globally.
- Plan weight updates to be infrequent but well-timed (just before tempo).";

const STAKE_WEIGHT: &str = "\
STAKE-WEIGHT MINIMUM (1000τ)
=============================
To set weights on a subnet, your validator needs a minimum amount of effective
stake-weight. The typical threshold is 1000 TAO equivalent.

Why it exists:
- Prevents low-stake accounts from manipulating subnet scoring.
- Ensures validators have meaningful economic commitment.

What counts toward stake-weight:
- Direct stake from your coldkey to your hotkey on the subnet.
- Delegated (nominated) stake from other coldkeys.
- Childkey delegations from parent hotkeys.

If you're below the threshold:
  1. Ask others to stake/delegate to your validator hotkey.
  2. Move more of your own TAO to stake on that subnet.
  3. Use commit-reveal instead of direct set_weights (some subnets allow
     commit-reveal at lower thresholds).

Check your stake: `agcli stake list`
Check subnet requirements: `agcli subnet hyperparams <netuid>`";

const AMM: &str = "\
AMM (DYNAMIC TAO / ALPHA POOLS)
================================
Each subnet has a constant-product AMM (Automated Market Maker) pool that
creates a market between TAO and the subnet's alpha token.

Pool mechanics:
- Two-sided pool: TAO side (tao_in) and Alpha side (alpha_in).
- Price = tao_in / alpha_in (constant-product formula: x * y = k).
- When you stake TAO on a subnet, it swaps through the AMM → you get alpha.
- When you unstake, alpha swaps back → you get TAO.

Slippage:
- Small pools = high slippage. A 10τ stake on a pool with 100τ depth causes
  ~10% slippage (you get fewer alpha per TAO than the listed price).
- Check slippage before staking: `agcli view swap-sim --netuid N --tao 10`

Key metrics:
- price: Current τ/α exchange rate
- tao_in: TAO side of the pool (liquidity depth)
- alpha_in: Alpha side of the pool
- moving_price: Exponential moving average of price (32 fractional bits)

Tips for operators:
- Don't stake/unstake large amounts on shallow pools — use limit orders.
- Watch the pool depth: `agcli subnet show <netuid>` shows tao_in.
- The AMM means your alpha is always liquid — you can unstake anytime.";

const BOOTSTRAP: &str = "\
BOOTSTRAP GUIDE — New Subnet Owners
====================================
Getting a new subnet operational step-by-step:

1. REGISTER THE SUBNET
   agcli subnet register
   (Costs the current subnet registration price — check with `agcli view network`)

2. GET STAKE ON YOUR VALIDATOR
   Your hotkey needs enough stake to set weights (typically 1000τ stake-weight).
   agcli stake add <amount> --netuid <your_netuid>
   Or ask delegators/nominators to stake to your hotkey.

3. SET INITIAL WEIGHTS
   agcli weights set --netuid <your_netuid> \"0:100\"
   (Set weights on at least one UID — usually yourself for bootstrapping)

4. CONFIGURE HYPERPARAMS (as subnet owner)
   The owner can adjust subnet parameters through governance proposals or
   directly if the chain allows direct hyperparam setting.

5. REGISTER MINERS
   Miners register via burn or POW:
   agcli subnet register-neuron <netuid>
   agcli subnet pow <netuid>

6. ONBOARD VALIDATORS
   Other validators register and start setting weights.
   Your subnet becomes healthy when multiple validators independently score miners.

7. MONITOR
   agcli subnet metagraph <netuid>           — see all UIDs and scores
   agcli view subnet-analytics <netuid>      — emission and performance stats

Common pitfalls:
- Forgetting to set weights initially (no emissions flow if no weights set)
- Not having enough stake to pass the stake-weight minimum
- Setting tempo too low (frequent evals) or too high (slow feedback)";

const ALPHA: &str = "\
ALPHA TOKENS
============
Each subnet issues its own alpha token. Alpha represents your share of the
subnet's staking pool and emission flow.

When you stake TAO on a subnet:
- Your TAO enters the AMM pool.
- You receive alpha tokens in return (at the current exchange rate).
- Your alpha entitles you to a share of the subnet's emissions.

When you unstake:
- Your alpha goes back through the AMM.
- You receive TAO (at the current exchange rate — may differ from when you staked).

Alpha operations:
  agcli stake add 10 --netuid 5         # TAO → alpha (stake)
  agcli stake remove 10 --netuid 5      # alpha → TAO (unstake)
  agcli stake recycle-alpha 10 --netuid 5   # recycle alpha back to TAO
  agcli stake burn-alpha 10 --netuid 5      # permanently burn alpha (reduce supply)

Key insight: alpha is always liquid through the AMM, but slippage matters on
small pools. Use `agcli view swap-sim` to preview swap amounts.";

const EMISSION: &str = "\
EMISSIONS
=========
TAO is emitted every block and distributed across subnets and within each subnet.

Block emission: ~1τ per block (halving schedule applies).

Distribution chain:
1. BLOCK EMISSION → split across all subnets based on root weights.
2. SUBNET EMISSION → split between:
   - alpha_out_emission: goes to alpha holders (stakers)
   - alpha_in_emission: goes into the AMM pool
   - tao_in_emission: goes into the TAO side of the pool
3. WITHIN SUBNET → Yuma consensus distributes to validators (dividends)
   and miners (incentive) based on weights and consensus scores.

Check emission rates:
  agcli view network                    — block emission, total stake
  agcli subnet show <netuid>            — subnet emission per tempo
  agcli view subnet-analytics <netuid>  — detailed emission breakdown
  agcli view staking-analytics          — your personal emission estimates";

const REGISTRATION: &str = "\
REGISTRATION
============
Neurons (miners/validators) must register on a subnet to participate.

Two registration methods:

1. BURN REGISTRATION — Pay TAO to register instantly.
   agcli subnet register-neuron <netuid>
   Cost varies per subnet and adjusts with demand (check `agcli subnet show`).

2. POW REGISTRATION — Solve a proof-of-work puzzle.
   agcli subnet pow <netuid> --threads 8
   Free but competitive — difficulty adjusts to target registration rate.

After registration:
- You get a UID (0 to max_n-1) on the subnet.
- New registrants have an immunity period (immunity_period blocks) where
  they cannot be deregistered.
- If the subnet is full, the lowest-score neuron gets replaced.

Prerequisites:
- A wallet with a coldkey and hotkey: `agcli wallet create`
- TAO balance (for burn registration) or CPU time (for POW)";

const SUBNETS: &str = "\
SUBNETS
=======
Subnets are the core unit of Bittensor. Each subnet defines an incentive game
where validators evaluate miners on a specific task.

Subnet properties:
- netuid: Unique identifier (0 = root network)
- tempo: Evaluation frequency (blocks between consensus rounds)
- max_n: Maximum number of neurons (UIDs) on the subnet
- emission_value: TAO emitted to this subnet per tempo
- Hyperparameters: rho, kappa, immunity_period, weights settings, etc.

Subnet lifecycle:
  1. Owner registers subnet (pays registration price)
  2. Owner configures identity and hyperparameters
  3. Miners and validators register
  4. Validators set weights → emissions flow
  5. Subnet grows or gets dissolved

List subnets: `agcli subnet list`
Subnet details: `agcli subnet show <netuid>`
Hyperparams: `agcli subnet hyperparams <netuid>`";

const VALIDATORS: &str = "\
VALIDATORS
==========
Validators evaluate miners and set weights that determine emission distribution.

Validator responsibilities:
1. Run scoring infrastructure (query miners, evaluate responses).
2. Set weights based on miner performance: `agcli weights set --netuid N \"uid:weight,...\"`
3. Participate in Yuma consensus — accurate weights earn dividends.

Becoming a validator:
1. Register on the subnet: `agcli subnet register-neuron <netuid>`
2. Accumulate enough stake-weight (typically 1000τ).
3. Get validator_permit = true (top N validators by stake get permits).
4. Set weights each tempo.

Key metrics:
- validator_trust (VTrust): How well your weights match consensus.
- dividends: Your share of validator emissions.
- validator_permit: Whether you can set weights (top staked validators).

Common issues:
- 'SettingWeightsTooFast' — wait for rate limit to expire
- 'CommitRevealEnabled' — use commit+reveal workflow instead
- Low VTrust — your weights diverge from other validators";

const MINERS: &str = "\
MINERS
======
Miners perform the actual work on a subnet and earn incentive-based emissions.

Miner responsibilities:
1. Serve an axon endpoint for validators to query.
2. Respond to validator queries with high-quality results.
3. Stay competitive — low-performing miners get deregistered.

Becoming a miner:
1. Register on the subnet: `agcli subnet register-neuron <netuid>`
   Or via POW: `agcli subnet pow <netuid>`
2. Set your axon endpoint: `agcli serve axon --netuid N --ip <ip> --port <port>`
3. Run your miner software (subnet-specific).

Key metrics:
- incentive: Your emission share based on validator weights.
- trust: How much validators trust your responses.
- rank: Your position relative to other miners.
- pruning_score: How likely you are to be replaced (low = at risk).

Protect your position:
- Consistently produce high-quality responses.
- Monitor your scores: `agcli view neuron --netuid N <uid>`
- Watch for adversarial actors (UIDs copying your work).";

const IMMUNITY: &str = "\
IMMUNITY PERIOD
===============
Newly registered neurons get a grace period where they cannot be deregistered.

- Measured in blocks (subnet-specific, typically 4096 blocks ≈ 13.6 hours).
- During immunity, even if your scores are low, you won't be pruned.
- After immunity expires, the lowest-scoring neuron is replaced when a new
  registration arrives and the subnet is full.

Check a subnet's immunity period:
  agcli subnet hyperparams <netuid>  →  immunity_period

Why it matters:
- New miners need time to set up their infrastructure and start responding.
- Without immunity, new registrants would be instantly replaced by existing neurons.
- Use the immunity period to get your miner running and serving.";

const DELEGATION: &str = "\
DELEGATION / NOMINATION
=======================
Delegation allows TAO holders to stake their TAO through a validator (delegate),
earning a share of that validator's emissions.

How it works:
1. Validator sets their delegate take (0-11.11%): `agcli delegate increase-take <pct>`
2. Nominator stakes through the validator's hotkey: `agcli stake add <amount> --netuid N --hotkey <validator_hotkey>`
3. Emissions earned by the validator are split: validator keeps their take,
   rest is distributed pro-rata to all nominators.

For nominators:
- Research validators: `agcli delegate list` or `agcli view validators`
- Check take %: lower take = more emissions for you
- Check validator performance: high VTrust = consistently accurate weights
- Diversify across subnets and validators to manage risk

For validators:
- Set a competitive take: `agcli delegate decrease-take <pct>`
- Low take attracts more delegation → more total stake → more influence
- Your reputation matters — consistent performance attracts long-term delegators";

const CHILDKEYS: &str = "\
CHILDKEYS
=========
Childkey delegation allows a parent validator hotkey to share its stake-weight
with child hotkeys on specific subnets.

Use cases:
- Run multiple validator instances with shared stake.
- Delegate your weight to specialized scoring infrastructure.
- Split your validator operations across teams/machines.

Set children: `agcli stake set-children --netuid N --children \"proportion:hotkey,...\"`
Set childkey take: `agcli stake childkey-take <pct> --netuid N`

The proportion determines how much of the parent's stake-weight flows to
each child. Proportions are u64 values — use relative ratios.";

const ROOT_NETWORK: &str = "\
ROOT NETWORK (SN0)
==================
The root network (netuid 0) controls emission distribution across all subnets.

Root validators set weights on subnet netuids to determine how much emission
each subnet receives. Higher root weight → more emission for that subnet.

Joining root:
  agcli root register    — register your hotkey on the root network

Setting root weights:
  agcli root weights \"1:100,5:50,97:200\"   — weight netuids, not UIDs

Root is special:
- Validators on root must have high total stake.
- Root weights directly control the economic incentives for all subnets.
- Changing root weights shifts emission flow across the entire network.";

const PROXY: &str = "\
PROXY ACCOUNTS
==============
Proxy accounts allow one account to act on behalf of another with restricted
permissions, enhancing security for validators and stakers.

Add a proxy:
  agcli proxy add <delegate_ss58> --proxy-type staking

Proxy types:
- any: Full access (dangerous — use sparingly)
- staking: Can stake/unstake but not transfer
- non_transfer: Can do anything except transfer TAO
- governance: Can participate in governance votes
- owner: Subnet owner operations

Why use proxies:
- Keep your coldkey on an air-gapped machine.
- Give your automation/bot limited permissions via a proxy.
- Revoke access without moving funds.

List proxies: `agcli proxy list`
Remove proxy: `agcli proxy remove <delegate_ss58>`";

const COLDKEY_SWAP: &str = "\
COLDKEY SWAP
============
Coldkey swap allows you to migrate your account to a new coldkey. This is a
scheduled operation — it does not execute immediately.

How it works:
1. SCHEDULE: Submit a swap request specifying the new coldkey.
   agcli swap coldkey --new-coldkey <new_ss58>
   The chain records the swap with an execution block (typically days away).

2. WAITING PERIOD: The swap is pending for ColdkeySwapScheduleDuration blocks.
   During this window, the original coldkey still controls the account.

3. EXECUTION: At the execution block, the chain automatically transfers all
   balances, stakes, and permissions from the old coldkey to the new one.

Security implications:
- If someone gains access to your coldkey, they can schedule a swap.
- This is a CRITICAL security event — monitor with `agcli audit`.
- The audit command checks for scheduled swaps and flags them as [!!] high severity.

Detection:
  agcli audit --address <your_coldkey>
  # Shows: 'Coldkey swap scheduled! New coldkey: ... at block ...'

Prevention:
- Use proxy accounts with limited permissions for daily operations.
- Keep your coldkey on an air-gapped or hardware-secured machine.
- Monitor your account regularly with `agcli audit`.
- Set up alerts: `agcli subscribe events --filter all --account <your_coldkey>`

Note: The chain does NOT currently expose a cancel-swap extrinsic. Once scheduled,
a coldkey swap will execute at the scheduled block unless chain governance intervenes.
If you detect an unauthorized swap, contact the Bittensor community immediately.";
