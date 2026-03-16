# Why agcli?

A comparison of **agcli**, **btcli**, and the **Bittensor Python SDK** for operators, validators, miners, and AI agents working on the Bittensor network.

---

## TL;DR

| | agcli | btcli | Bittensor SDK |
|---|---|---|---|
| **Language** | Rust | Python | Python |
| **Type** | CLI + embeddable SDK | CLI | Library (CLI optional via `bittensor[cli]`) |
| **Startup** | ~50 ms (native binary) | 2–5 s (Python interpreter + imports) | 2–5 s (Python) |
| **Distribution** | Single static binary | pip, Python 3.10+, OpenSSL, ~20 deps | pip, Python 3.10+, optional PyTorch/CUDA |
| **Caching** | 3-layer (memory → disk → chain), request coalescing, stale-while-error | Disk cache (SQLite), in-memory option | None built-in |
| **Parallelism** | Tokio multi-threaded async | asyncio single-threaded (GIL-bound) | asyncio single-threaded |
| **Agent mode** | `--batch` (hard-error), `--yes`, JSON/CSV, spending limits, `--dry-run` | `--no-prompt`, `--json-output` | Library API |
| **MEV protection** | ML-KEM-768 post-quantum shield | MEV shield (details vary by version) | Via btcli |
| **Commands** | 150+ subcommands, 18 groups | ~30 subcommands | Python API |
| **Historical queries** | `--at-block` on any read command | Not supported | Not supported |
| **Live monitoring** | Built-in streaming (`--live`, `subscribe`) | Not supported | Manual polling |
| **State diffing** | `diff portfolio`, `diff subnet`, `diff network` | Not supported | Not supported |
| **Block explorer** | `block info`, `block range`, `block latest` | Not supported | Not supported |
| **Security audit** | `audit` (proxies, delegates, exposure) | Not supported | Not supported |
| **Diagnostics** | `doctor` (connectivity, wallet, chain version) | Not supported | Not supported |
| **Output formats** | Table, JSON, CSV | Table, JSON | Python objects |
| **Shell completions** | bash, zsh, fish, PowerShell | Not available | N/A |
| **Wallet compat** | Reads Python bittensor-wallet keyfiles | Native Python wallets | Native Python wallets |
| **Test suite** | 403 tests, 5,900 LOC | pytest suite | Unit + integration + e2e |
| **Windows** | Native | WSL 2 only | WSL 2 only |

---

## The case for agcli

### 1. Speed where it matters

agcli is a compiled Rust binary. No interpreter startup, no dependency resolution, no import chain.

```
agcli balance --address 5Gx...    # result in ~200ms
btcli wallet balance              # ~3s before first output
```

- **Cold start**: ~50 ms vs 2–5 s. That's 40–100x faster before the first byte of work.
- **Multi-threaded async**: Tokio's work-stealing scheduler across all CPU cores. Python's asyncio runs on a single thread constrained by the GIL — even with uvloop.
- **Parallel extrinsics**: Setting weights across 50 subnets fires concurrently. btcli submits sequentially.
- **Parallel queries**: Block ranges, metagraph fetches, and multi-subnet operations all parallelize automatically.

For validators running automated scripts that invoke the CLI hundreds of times per day, this adds up fast. A script that calls the CLI 500 times saves **20+ minutes per day** just on startup alone.

### 2. Three-layer caching with request coalescing

Repeated queries are nearly free:

```
Request → Memory cache (30s TTL, Moka concurrent cache)
        → Disk cache (5 min TTL, atomic JSON files)
        → Chain RPC (actual fetch)
```

What makes this different from btcli's disk cache:

| | agcli | btcli |
|---|---|---|
| In-memory layer | Yes (Moka, lock-free) | Optional `use_cache` flag |
| Disk layer | Atomic file writes, no contention | SQLite (potential lock contention under load) |
| Request coalescing | 10 concurrent requests for same key → 1 RPC call | No coalescing |
| Stale-while-error | Serves expired entries when chain is down | Hard failure |
| Metagraph snapshots | Save, load, diff, prune snapshots across blocks | Not available |
| Cross-invocation | Disk cache survives CLI exit, 5 min TTL | Survives exit (SQLite) |

**Request coalescing** is the key differentiator. When an agent fires 10 concurrent `subnet list` queries, agcli deduplicates them into a single RPC call. btcli makes 10 separate calls.

### 3. Built for AI agents, not just humans

agcli was designed ground-up for non-interactive, machine-driven use:

| Feature | agcli | btcli |
|---|---|---|
| Skip confirmations | `--yes` | `--no-prompt` |
| Hard-error mode | `--batch` — never prompts, structured errors | Not available |
| Structured errors | JSON on stderr in batch/JSON mode with error codes | Exit codes only |
| Spending limits | `config set spending_limit.1 100` per subnet | Not available |
| Dry-run preview | `--dry-run` on all write commands | Not available |
| Password via env | `AGCLI_PASSWORD` / `--password` (no tty) | Requires tty or keyring |
| Output formats | `--output json\|csv\|table` | `--json-output` (JSON only) |
| Shell completions | bash, zsh, fish, PowerShell | Not available |
| Env var config | Every flag has an `AGCLI_*` env var | Partial (config file + env) |

**Why `--batch` matters**: It guarantees the CLI will *never* block waiting for input. Missing a required argument? Structured JSON error with the field name — not a hanging prompt. This is the difference between an agent that works and one that deadlocks.

**Spending limits** let operators cap how much TAO an automated agent can stake per subnet per invocation. When you hand CLI access to an AI agent, this is the safety net between "helpful automation" and "drained wallet."

### 4. 150+ commands — everything btcli does and more

agcli covers the full btcli command surface and adds substantially more (150+ subcommands across 18 groups vs ~30 in btcli):

**Analytics & monitoring (unique to agcli):**
- `view portfolio` — cross-subnet stake portfolio with P&L
- `view validators` — ranked comparison (stake, VTrust, dividends)
- `view history` — transaction history for any account
- `view network` / `view dynamic` — real-time network analytics
- `subnet monitor` — continuous metagraph stream with delta tracking
- `subnet health` / `subnet emissions` — per-subnet diagnostics
- `subnet probe` — TCP reachability check for all miners (concurrent)

**State comparison (unique to agcli):**
- `diff portfolio --from-block 1000 --to-block 2000` — portfolio changes over time
- `diff subnet` / `diff network` — subnet and network state deltas
- `subnet cache-diff` — compare saved metagraph snapshots

**Block explorer (unique to agcli):**
- `block info --block 12345` — extrinsics, events, timestamp
- `block range --from 1000 --to 1100` — batch block analysis
- `block latest` — current finalized block

**Real-time subscriptions (unique to agcli):**
- `subscribe blocks` — finalized block stream
- `subscribe events --filter staking --netuid 1` — filtered event stream

**Security & diagnostics (unique to agcli):**
- `audit --address 5Gx...` — proxy permissions, delegate exposure, stake analysis
- `doctor` — connectivity, wallet health, chain version, endpoint latency

**Education (unique to agcli):**
- `explain tempo` / `explain commit-reveal` / `explain amm` — 31 built-in topics, no browser needed

**Operations (unique to agcli):**
- `batch --file ops.json` — atomic bulk extrinsics from file
- `utils convert` — alpha/TAO rate conversion
- `utils latency` — endpoint latency benchmark
- `weights commit-reveal --wait` — atomic commit + wait + reveal in one command

**Historical queries**: Any read command accepts `--at-block N` to query chain state at a specific block height. Debug a weight dispute, check what the metagraph looked like 1000 blocks ago, compare validator performance across time. btcli has no equivalent.

### 5. Post-quantum MEV protection

agcli includes an ML-KEM-768 encryption shield for weight submissions:

```bash
agcli weights set --netuid 1 --weights "0:100,1:200" --mev
```

Weights are encrypted client-side using ML-KEM-768 (NIST post-quantum standard) + XChaCha20-Poly1305 AEAD before being committed on-chain. The commitment uses Blake2s-256. This prevents MEV bots from reading your weights during the commit phase and front-running your reveals.

The `--mev` flag (or `AGCLI_MEV=1` env var) enables this globally. No additional setup needed.

### 6. Single binary, zero dependencies

```bash
# Install
cargo install --git https://github.com/unconst/agcli

# Or download a prebuilt binary — one file, done
```

That's it. No Python version management, no virtual environments, no OpenSSL compatibility headaches, no dependency trees. The binary embeds rustls (pure-Rust TLS) and has zero system library requirements.

**btcli requires:**
- Python 3.10+ (3.9 dropped in btcli v9.x)
- A virtual environment (strongly recommended)
- OpenSSL — macOS users must install Homebrew Python because the system Python uses LibreSSL (incompatible)
- ~20 mandatory dependencies (aiohttp, numpy, rich, typer, pycryptodome, scalecodec, etc.)
- SSL certificate fixups (`python -m bittensor certifi`) on fresh installs

**Bittensor SDK additionally requires:**
- FastAPI, Pydantic, uvicorn (for Axon server)
- Optional PyTorch (~800 MB) for tensor operations
- bittensor-wallet, bittensor-drand, async-substrate-interface

agcli ships as one file. Copy it to a server, a container, a CI runner — it just works.

### 7. Resilient by default

agcli handles network issues automatically:

| Behavior | agcli | btcli |
|---|---|---|
| Retry with backoff | 3 attempts, exponential (1→2→4s) | backoff library on some operations |
| Endpoint fallback | Rotates to next endpoint on failure | Not built-in |
| Best endpoint selection | `--best` tests all endpoints concurrently, picks fastest | Not available |
| Stale cache on error | Serves expired data instead of failing | Hard failure |
| Transient error detection | Retries connection/timeout only, not permanent errors | Varies by operation |

When the chain is congested or an endpoint goes down, agcli keeps working. Your scripts don't need try/catch wrappers or retry logic — it's built in.

### 8. Embeddable Rust SDK

agcli isn't just a CLI. Import it as a Rust library:

```toml
[dependencies]
agcli = { git = "https://github.com/unconst/agcli", default-features = false, features = ["sdk-only"] }
```

```rust
use agcli::{Client, Wallet, Balance};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::connect("wss://entrypoint-finney.opentensor.ai:443").await?;

    // 57 query methods — balance, metagraph, subnets, delegates, identity, etc.
    let balance = client.get_balance_ss58("5Gx...").await?;
    let subnets = client.get_all_subnets().await?;
    let metagraph = client.get_metagraph(1.into()).await?;

    // All caching, retry, and connection logic included
    Ok(())
}
```

Build validators, miners, dashboards, and monitoring tools in Rust using the same battle-tested chain interface the CLI runs on. The `sdk-only` feature gate keeps binary size minimal by excluding CLI dependencies.

**57 query methods** cover: balances, subnets, neurons, metagraphs, delegates, identities, proxies, hyperparameters, dynamic info, swap simulations, emission data, child/parent keys, crowdloans, and more — all with the same caching and retry behavior.

### 9. First-class live monitoring

Real-time visibility without external tools:

```bash
# Watch metagraph changes every block (delta mode — only shows what changed)
agcli subnet monitor --netuid 1 --json

# Stream staking events for a specific subnet
agcli subscribe events --filter staking --netuid 1

# Watch your portfolio update live
agcli view portfolio --live

# Track finalized blocks
agcli subscribe blocks
```

All live commands support `--json` for piping into dashboards, alerting systems, or log aggregators. The delta tracking in `subnet monitor` only emits changes (registrations, deregistrations, weight updates, emission shifts) — not the full metagraph every block.

btcli has no live monitoring. The Bittensor SDK requires writing custom polling loops.

### 10. Documentation designed for both agents and humans

Three tiers, each optimized for its audience:

| Tier | File | Audience | Size |
|---|---|---|---|
| Agent reference | [llm.txt](llm.txt) | LLMs and AI agents | ~15 KB, single file |
| Command deep-dives | [commands/](commands/) | Developers and operators | 22 documents |
| Step-by-step tutorials | [tutorials/](tutorials/) | Newcomers and operators | 5 guides |

The **`explain` command** embeds 31 educational topics directly in the terminal:

```bash
agcli explain tempo          # What is tempo and how does it affect weight setting?
agcli explain commit-reveal  # How does commit-reveal work?
agcli explain amm            # How does the Dynamic TAO AMM work?
agcli explain dynamic-tao    # What is Dynamic TAO?
agcli explain yuma           # How does Yuma Consensus work?
```

No browser, no docs site, no context switching. The knowledge is in the binary.

---

## Real-world scenarios

### Scenario: Validator automation

You run a validator on 12 subnets. Every epoch you need to query metagraphs, compute weights, and submit them.

**With btcli:**
```bash
for netuid in 1 2 3 4 5 6 7 8 9 10 11 12; do
    btcli subnet metagraph --netuid $netuid --json-output > "meta_$netuid.json"  # ~3s each
    # compute weights...
    btcli weights commit --netuid $netuid --weights "..."  # sequential
done
# Total: ~60s+ wall time
```

**With agcli:**
```bash
# All queries cached and coalesced
for netuid in 1 2 3 4 5 6 7 8 9 10 11 12; do
    agcli --output json subnet metagraph --netuid $netuid > "meta_$netuid.json"  # ~50ms + RPC
done
# Or use the SDK for true parallelism — all 12 metagraphs fetched concurrently

# Atomic commit-reveal (waits for reveal window automatically)
agcli weights commit-reveal --netuid 1 --weights "0:100,1:50" --wait --yes
```

### Scenario: AI agent operating autonomously

Your agent needs to check balances, stake, and set weights without human supervision.

```bash
# Set safety limits
agcli config set spending_limit.1 50   # Max 50 TAO per stake on subnet 1
agcli config set spending_limit.18 100 # Max 100 TAO per stake on subnet 18

# Agent runs in batch mode — structured errors, never hangs
export AGCLI_BATCH=1
export AGCLI_PASSWORD=...
export AGCLI_YES=1

agcli balance --address 5Gx...                    # Quick balance check
agcli stake add --netuid 1 --amount 10 --dry-run  # Preview before committing
agcli stake add --netuid 1 --amount 10             # Execute (capped by spending limit)
```

If the agent tries to stake 200 TAO on subnet 1, agcli blocks it — the spending limit is 50. btcli has no equivalent safety mechanism.

### Scenario: Forensic debugging

A validator's emissions dropped unexpectedly 500 blocks ago. What happened?

```bash
# Compare metagraph state across blocks
agcli diff subnet --netuid 18 --from-block 4500000 --to-block 4500500

# Check your portfolio change
agcli diff portfolio --from-block 4500000

# Look at specific block events
agcli block info --block 4500200

# Audit account security
agcli audit --address 5Gx...
```

None of this is possible with btcli or the Bittensor SDK.

---

## Who should use what

| You are... | Recommended tool | Why |
|---|---|---|
| A **validator** running automated infrastructure | **agcli** | Speed, caching, batch mode, spending limits, MEV shield, live monitoring |
| An **AI agent** operating on Bittensor | **agcli** | `--batch` guarantees no hangs, structured errors, spending limits, env var config |
| A **subnet operator** monitoring health | **agcli** | Live monitoring, metagraph diffs, historical queries, health checks, probe |
| A **developer** building Rust tooling | **agcli SDK** | Embeddable, type-safe, 57 query methods, same caching/retry as CLI |
| A **developer** building Python miners/validators | **Bittensor SDK** | Native Python, existing ecosystem integration, Axon/Dendrite built-in |
| A **newcomer** following existing tutorials | **btcli** | Matches current Bittensor documentation and community guides |
| Someone who needs **both CLI and Python library** | **btcli + SDK** | Unified Python ecosystem, shared wallet format |

agcli reads the same Python bittensor-wallet keyfiles. You can use agcli alongside btcli without migrating anything — try it on one workflow and expand from there.

---

## Migration from btcli

### Zero-friction start

agcli reads existing Python bittensor-wallet keyfiles directly. No key export, no migration script, no re-encryption. Point it at `~/.bittensor/wallets/` (the default) and go.

### Command mapping

| btcli | agcli | Notes |
|---|---|---|
| `btcli wallet create` | `agcli wallet create` | Same wallet format |
| `btcli wallet list` | `agcli wallet list` | |
| `btcli wallet balance` | `agcli balance` | Shorter, supports `--at-block` |
| `btcli stake add` | `agcli stake add` | Adds `--max-slippage`, `--dry-run` |
| `btcli stake remove` | `agcli stake remove` | |
| `btcli stake list` | `agcli stake list` | |
| `btcli subnet list` | `agcli subnet list` | Adds `--output json\|csv` |
| `btcli subnet metagraph` | `agcli subnet metagraph` | Adds `--at-block`, caching |
| `btcli root weights` | `agcli root weights` | |
| `btcli delegate list` | `agcli delegate list` | |
| `btcli weights commit` | `agcli weights commit` | Adds `--mev` |
| `btcli weights reveal` | `agcli weights reveal` | |
| — | `agcli weights commit-reveal --wait` | **New**: atomic commit + auto-reveal |
| — | `agcli view portfolio` | **New**: cross-subnet P&L |
| — | `agcli diff subnet` | **New**: state comparison |
| — | `agcli subnet monitor --json` | **New**: live streaming |
| — | `agcli audit` | **New**: security audit |
| — | `agcli doctor` | **New**: diagnostics |
| — | `agcli explain <topic>` | **New**: 31 educational topics |

### Configuration

```bash
# Set your defaults once
agcli config set network finney
agcli config set wallet_dir ~/.bittensor/wallets
agcli config set wallet default
agcli config set hotkey default

# Now just:
agcli balance
agcli stake list
agcli subnet metagraph --netuid 1
```

---

## Technical details

| | agcli |
|---|---|
| Source | 21,674 lines of Rust across 79 files |
| Tests | 403 test functions, 5,906 LOC (7 integration test modules) |
| Binary | Statically linked, LTO-optimized, stripped |
| TLS | rustls (pure Rust, no OpenSSL) |
| Async runtime | Tokio (multi-threaded work-stealing) |
| Cache | Moka (lock-free concurrent) + atomic disk JSON |
| Crypto | sr25519, Blake2, ML-KEM-768, XChaCha20-Poly1305, AES-GCM, Argon2, BIP39 |
| Substrate | subxt for type-safe chain interaction |
| License | MIT |
