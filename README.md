# Polymarket Copy Trading Bot

Copy trades from a target Polymarket wallet in real time. **Rust**, WebSocket + CLOB API. Auto-redemption, size multiplier, and max order limits included.

[![Telegram](https://img.shields.io/badge/Telegram-@toptrendev_66-2CA5E0?style=for-the-badge&logo=telegram)](https://t.me/TopTrenDev_66)
[![Twitter](https://img.shields.io/badge/Twitter-@toptrendev-1DA1F2?style=for-the-badge&logo=twitter)](https://x.com/toptrendev)

---

## Quick start

| Step | Action                                                                                                          |
| ---- | --------------------------------------------------------------------------------------------------------------- |
| 1    | **Rust** (1.84+), **Polygon wallet** with USDC, **Polymarket** account                                          |
| 2    | `git clone <repo> && cd polymarket-copy-trading-bot && cargo build --release`                                   |
| 3    | `cp .env.example .env` → set `PRIVATE_KEY` and `TARGET_WALLET`                                                  |
| 4    | `cargo run --release --bin polymarket-bot` (loads API credentials from `src/data/credential.json` on first run) |

**Commands**

| What                   | Command                                               |
| ---------------------- | ----------------------------------------------------- |
| Run bot                | `cargo run --release --bin polymarket-bot`            |
| Auto-redeem (holdings) | `cargo run --release --bin auto-redeem`               |
| Auto-redeem (dry run)  | `cargo run --release --bin auto-redeem -- --dry-run`  |
| Redeem one market      | `cargo run --release --bin redeem -- <conditionId>`   |
| Redeem (via env)       | `CONDITION_ID=0x... cargo run --release --bin redeem` |

**Note:** The Rust bot requires `src/data/credential.json`. Create it by running the original TypeScript bot once, or implement EIP-712 auth in Rust. Until then, copy a valid `credential.json` from a working setup.

---

## What it does

- **Mirrors trades** from a target wallet via WebSocket and CLOB.
- **Auto-redeems** winning positions (optional interval in minutes).
- **Risk controls**: size multiplier (e.g. 30% of target size), max order amount, optional negative risk.
- **Order types**: FAK / FOK; tick size configurable.
- **Holdings**: local `src/data/token-holding.json` for redemption; credentials in `src/data/credential.json`.

---

## Configuration (env)

Copy `.env.example` to `.env` and edit. **Required:** `PRIVATE_KEY`, `TARGET_WALLET`.

| Variable              | Description                                         | Example / default |
| --------------------- | --------------------------------------------------- | ----------------- |
| `PRIVATE_KEY`         | Wallet private key (Polygon, USDC)                  | **required**      |
| `TARGET_WALLET`       | Address to copy                                     | `0x...`           |
| `SIZE_MULTIPLIER`     | Fraction of target size (e.g. 30% of target amount) | `0.3`             |
| `MAX_ORDER_AMOUNT`    | Max order amount in USDC                            | `5`               |
| `ENABLE_COPY_TRADING` | Master switch for copy trading                      | `true`            |
| `REDEEM_DURATION`     | Minutes between auto-redeem runs                    | `15`              |

---

## Flow (high level)

1. **WebSocket** → trade activity from Polymarket.
2. **Filter** by `TARGET_WALLET` → build order (multiplier, max amount, tick size, type).
3. **CLOB** → place order; update local holdings.
4. **Redemption** (periodic or manual) → resolve markets, redeem winning positions from `token-holding.json` (or API).

---

## Project layout

```
src/
├── main.rs              # Bot entry (WebSocket + copy + optional auto-redeem)
├── lib.rs               # Library exports
├── bin/
│   ├── redeem.rs        # Single-market redeem
│   └── auto_redeem.rs   # Batch redeem (holdings)
├── config.rs            # Config from env
├── logger.rs            # Logging helpers
├── redemption/          # Redemption logic (CTF, stubs)
│   └── mod.rs
├── data/
│   ├── credential.json  # API creds (load from file)
│   └── token-holding.json
├── order_builder/       # Trade → order (multiplier, limits, FAK/FOK)
├── providers/           # CLOB, WebSocket, RPC
├── security/            # Allowance, credential
└── utils/               # balance, holdings, types
```

**Stack:** Rust, Tokio, reqwest, tokio-tungstenite, ethers, Polygon.

---

## Security & safety

- Private key and API creds from env/file only (never hardcoded).
- Allowances and balance checks before orders.
- Start with small `SIZE_MULTIPLIER` and low `MAX_ORDER_AMOUNT`; use `--dry-run` for redemption tests.

**Risks:** Market/liquidity/slippage, gas, API limits, latency. Use at your own risk; never risk more than you can afford to lose.

---

## Development

```bash
cargo build
cargo run --bin polymarket-bot
cargo clippy
```
