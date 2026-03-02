# token-layer-cli

Rust CLI for Token Layer endpoints, aligned with the SDK surface for:

- actions: `createToken`, `tradeToken`, `transferToken`
- info: `getTokensV2`, `getPoolData`, `me`
- local wallets/profiles: `init`, `wallet add`, `wallet list`, `profile list`, `profile use`

## Build

```bash
cd packages/token-layer-cli
cargo build
```

## Wallets

Run the init wizard to create a profile with one of:

- wallet auth
- JWT auth
- API key auth

```bash
tokenlayer init
```

Quick non-interactive wallet-only init (legacy behavior):

```bash
tokenlayer init --quick-wallet --name default
```

Add additional wallets:

```bash
tokenlayer wallet add --chain ethereum --name bot-1-eth
tokenlayer wallet add --chain solana --name bot-1-sol
```

Wallets are saved at `~/.token-layer-cli/wallets.json` (or `$TL_CLI_HOME/wallets.json`).
Profiles (auth configs) are saved at `~/.token-layer-cli/profiles.json`.

List/switch profiles:

```bash
tokenlayer profile list
tokenlayer profile use --name <PROFILE_NAME>
```

## Auth

Authenticated commands use bearer auth like the SDK JWT/API key mode.
Provide one of:

- `--token <value>`
- `--jwt <value>`
- `--api-key <value>`
- env vars: `TL_JWT` or `TL_API_KEY`

Wallet-signature auth (SDK-style EIP-712 + SIWE) is supported for:

- `action register`
- `action create-token` when `--wallet-name` or `--wallet-address` is provided

## Examples

```bash
# me (authenticated)
tokenlayer info me --include-testnets --jwt "$TL_JWT"

# public info
tokenlayer info get-tokens-v2 --order-by volume_24h --limit 20
tokenlayer info get-pool-data --token-id <TOKEN_UUID>

# trade (authenticated)
tokenlayer action trade-token \
  --token-id <TOKEN_UUID> \
  --chain-slug base \
  --direction buy \
  --buy-amount-usd 1 \
  --jwt "$TL_JWT"

# transfer (authenticated)
tokenlayer action transfer-token \
  --token-id <TOKEN_UUID> \
  --recipient-address 0x000000000000000000000000000000000000dEaD \
  --amount 0.1 \
  --from-chain-slug base \
  --to-chain-slug base \
  --jwt "$TL_JWT"

# register (wallet-signed)
tokenlayer action register \
  --wallet-name default-eth \
  --signature-chain-id 0x1

# create token (wallet-signed)
tokenlayer action create-token \
  --name "Signed Token" \
  --symbol SGN \
  --description "wallet signed create token" \
  --image "https://example.com/logo.png" \
  --chain-slug base \
  --wallet-name default-eth \
  --signature-chain-id 0x1

# use a specific stored profile for auth defaults
tokenlayer --profile trader-jwt info me --include-testnets
```
