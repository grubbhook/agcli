#!/usr/bin/env bash
set -euo pipefail

WALLET="${1:-}"
HOTKEY="${2:-}"
NETUID="${3:-}"
PASSWORD="${4:-}"
WALLET_DIR="${AGCLI_WALLET_DIR:-/root/.bittensor/wallets}"

if [[ -z "$WALLET" ]]; then
  read -r -p "Wallet name: " WALLET
fi

if [[ -z "$HOTKEY" ]]; then
  read -r -p "Hotkey name: " HOTKEY
fi

if [[ -z "$NETUID" ]]; then
  read -r -p "Netuid: " NETUID
fi

if [[ -z "$PASSWORD" ]]; then
  read -r -s -p "Password: " PASSWORD
  echo
fi

export AGCLI_PASSWORD="$PASSWORD"

while true; do
  agcli --wallet-dir "$WALLET_DIR" \
        --wallet "$WALLET" \
        --hotkey-name "$HOTKEY" \
        subnet register-neuron --netuid "$NETUID" --yes && break
  sleep 1
done