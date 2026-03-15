# wallet — Wallet Management

Create, import, and manage sr25519 keypairs. Wallets consist of a coldkey (encrypted, for signing transactions) and one or more hotkeys (plaintext, for automated operations). Compatible with Python bittensor-wallet keyfile format (NaCl SecretBox + JSON).

## Subcommands

### wallet create
Create a new wallet with coldkey + default hotkey.

```bash
agcli wallet create [--name mywallet] [--password PW] [--yes]
# JSON: {"name", "coldkey", "hotkey"}
```

Generates sr25519 keypair, encrypts coldkey with password, saves to `~/.bittensor/wallets/<name>/`.

### wallet list
List all wallets in the wallet directory.

```bash
agcli wallet list
# JSON: [{"name", "coldkey"}]
```

### wallet show
Show wallet details including all hotkeys.

```bash
agcli wallet show [--all]
# JSON: [{"name", "coldkey", "hotkeys": [...]}]
```

| Flag | Description |
|------|-------------|
| `--all` | Show all hotkeys (not just default) |

### wallet import
Import wallet from mnemonic phrase.

```bash
agcli wallet import --name mywallet --mnemonic "word1 word2 ... word12" [--password PW]
# JSON: {"name", "coldkey"}
```

### wallet regen-coldkey
Regenerate coldkey from mnemonic (overwrites existing).

```bash
agcli wallet regen-coldkey --mnemonic "word1 word2 ... word12" [--password PW]
```

### wallet regen-hotkey
Regenerate a hotkey from mnemonic.

```bash
agcli wallet regen-hotkey --name default --mnemonic "word1 word2 ... word12"
```

### wallet new-hotkey
Create an additional hotkey for the current wallet.

```bash
agcli wallet new-hotkey --name myhotkey
# JSON: {"name", "hotkey"}
```

### wallet sign
Sign an arbitrary message with the coldkey.

```bash
agcli wallet sign --message "hello world" [--password PW]
# JSON: {"signer", "message", "signature"}
```

### wallet verify
Verify a signature.

```bash
agcli wallet verify --message "hello world" --signature 0xabcdef... [--signer SS58]
```

Exit code 0 = valid, 1 = invalid.

### wallet derive
Derive SS58 address from a public key hex or mnemonic (no secrets printed).

```bash
agcli wallet derive --input 0xd43593c715fdd31c61141abd...
agcli wallet derive --input "word1 word2 ... word12"
```

### wallet associate-hotkey
Associate a hotkey with your coldkey on-chain.

```bash
agcli wallet associate-hotkey [--hotkey SS58]
```

**On-chain**: `SubtensorModule::try_associate_hotkey(origin, hotkey)`

### wallet check-swap
Check if a coldkey swap is scheduled for an address.

```bash
agcli wallet check-swap [--address SS58]
# JSON: {"address", "swap_scheduled", "execution_block", "new_coldkey"}
```

## Wallet Storage
```
~/.bittensor/wallets/
├── default/
│   ├── coldkey           # encrypted sr25519 (NaCl SecretBox)
│   ├── coldkeypub.txt    # SS58 address (plaintext)
│   └── hotkeys/
│       └── default       # plaintext sr25519
```

## Key Concepts
- **Coldkey**: Main signing key, always encrypted. Used for transfers, staking, governance.
- **Hotkey**: Automated key, stored plaintext. Used for weight setting, serving, registration.
- **SS58 address**: Base58 encoding with prefix 42 (Bittensor network).
- **Mnemonic**: 12-word BIP39 phrase for key recovery.

## Security
- Coldkeys are encrypted with NaCl SecretBox (XSalsa20-Poly1305)
- Password can be supplied via `--password`, `AGCLI_PASSWORD` env var, or interactive prompt
- Wallet creation is protected by a directory-level lock (prevents concurrent creation corruption)
- Never expose mnemonics or private keys in logs or output

## Source Code
**agcli handler**: [`src/cli/wallet_cmds.rs`](https://github.com/unconst/agcli/blob/main/src/cli/wallet_cmds.rs) — `handle_wallet()` at L9, subcommands: Create L17, List L41, Show L78, Import L171, RegenColdkey L193, RegenHotkey L213, NewHotkey L238, Sign L259, Verify L277, Derive L325. AssociateHotkey and CheckSwap dispatched from [`src/cli/commands.rs`](https://github.com/unconst/agcli/blob/main/src/cli/commands.rs) at L79 and L90.

**Subtensor pallet** (for on-chain ops):
- [`macros/dispatches.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/macros/dispatches.rs) — `try_associate_hotkey` dispatch
- [`swap/swap_coldkey.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/swap/swap_coldkey.rs) — swap status queries

## Related Commands
- `agcli balance` — Check wallet balance
- `agcli stake list` — View stakes for wallet
- `agcli swap coldkey` — Schedule coldkey swap
- `agcli proxy add` — Delegate signing to another key
