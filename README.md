# TokenLayer CLI

Autonomous interface for AI agents to operate on TokenLayer.

`tokenlayer` gives agents a reliable command surface for identity, market actions, and onchain activity intelligence without building custom API wiring every time.

## Why This Exists

Most agent workflows fail at execution: they can reason, but they cannot act consistently.

TokenLayer CLI closes that gap by letting agents:

- manage multiple identities per machine (wallet, JWT, API key)
- execute TokenLayer actions (`createToken`, `tradeToken`, `transferToken`)
- fetch live protocol context (`getTokensV2`, `getTokenActivity`, `getPoolData`, `me`, and more)
- run deterministic, scriptable command chains for autonomous loops

If your goal is an agent that can trade, launch, transfer, and react to market events on TokenLayer, this is the entry point.

## Core Capabilities

- Agent profiles with isolated auth contexts
- Wallet-signing support for TokenLayer-compatible signed flows
- Wallet/JWT/API-key auth for account-bound action and info requests
- Local wallet generation (Ethereum + Solana)
- Action commands:
  - `createToken`
  - `tradeToken`
  - `transferToken`
- Info commands:
  - `getTokensV2`
  - `getPoolData`
  - `me`
  - `getTokenTrades`
  - `getTokenTransfers`
  - `getTokenActivity`

## Installation

```bash
cd packages/token-layer-cli
cargo install --path .
```

Run:

```bash
tokenlayer --help
```

## Quickstart (Agent Setup)

Initialize an agent profile:

```bash
tokenlayer init
```

The wizard lets you pick:

- wallet auth
- JWT auth
- API key auth

Profile data is stored locally and can be switched instantly.

## Profiles (Multi-Agent / Multi-Account)

List profiles:

```bash
tokenlayer profile list
```

Set active profile:

```bash
tokenlayer profile use --name <PROFILE_NAME>
```

Run with a specific profile (without changing active):

```bash
tokenlayer --profile <PROFILE_NAME> info me --include-testnets
```

Profiles are stored at:

- `~/.token-layer-cli/profiles.json`
- or `$TL_CLI_HOME/profiles.json`

## Wallets

Quick wallet-only init (non-interactive):

```bash
tokenlayer init --quick-wallet --name default
```

Add more wallets:

```bash
tokenlayer wallet add --chain ethereum --name bot-1-eth
tokenlayer wallet add --chain solana --name bot-1-sol
```

List wallets:

```bash
tokenlayer wallet list
```

Wallets (including private keys) are stored at:

- `~/.token-layer-cli/wallets.json`
- or `$TL_CLI_HOME/wallets.json`

## Agent Execution Examples

Market discovery:

```bash
tokenlayer info get-tokens-v2 --order-by volume_24h --limit 20
tokenlayer info get-token-activity --token-id <TOKEN_ID_OR_ADDRESS> --limit 20
```

Execution:

```bash
tokenlayer action trade-token \
  --token-id <TOKEN_UUID> \
  --chain-slug base \
  --direction buy \
  --buy-amount-usd 1

tokenlayer action transfer-token \
  --token-id <TOKEN_UUID> \
  --recipient-address 0x000000000000000000000000000000000000dEaD \
  --amount 0.1 \
  --from-chain-slug base \
  --to-chain-slug base
```

Token launch:

```bash
tokenlayer action create-token \
  --name "Agent Token" \
  --symbol AGNT \
  --description "Autonomous launch" \
  --image "https://example.com/logo.png" \
  --chain-slug base
```

## Auth Modes

### 1) Profile-based (recommended for agents)
Set during `tokenlayer init` and reuse automatically.

### 2) Flags

- `--token`
- `--jwt`
- `--api-key`

### 3) Environment

- `TL_JWT`
- `TL_API_KEY`
- `TL_PROFILE`

## Wallet-Signed Flows

SDK-style wallet-signing is supported for:

- `action register`
- `action create-token` (with wallet context)

Examples:

```bash
tokenlayer action register --wallet-name default-eth --signature-chain-id 0x1

tokenlayer action create-token \
  --name "Signed Token" \
  --symbol SGN \
  --description "wallet signed create token" \
  --image "https://example.com/logo.png" \
  --chain-slug base \
  --wallet-name default-eth \
  --signature-chain-id 0x1
```

## Practical Pattern for Autonomous Agents

A common loop:

1. pull candidate markets (`getTokensV2`)
2. inspect momentum (`getTokenTrades` / `getTokenActivity`)
3. decide and execute (`tradeToken` / `transferToken`)
4. persist outcomes in your agent memory/store
5. repeat on schedule

This CLI is designed to be the execution + data plane for that loop.
