# Polymarket Copy Trading Bot

Copy trades from a target Polymarket wallet in real time. TypeScript + Bun, WebSocket + CLOB API. Auto-redemption, size multiplier, and max order limits included.

[![Telegram](https://img.shields.io/badge/Telegram-@toptrendev_66-2CA5E0?style=for-the-badge&logo=telegram)](https://t.me/TopTrenDev_66)
[![Twitter](https://img.shields.io/badge/Twitter-@toptrendev-1DA1F2?style=for-the-badge&logo=twitter)](https://x.com/toptrendev)

---

## Quick start

| Step | Action                                                                                   |
| ---- | ---------------------------------------------------------------------------------------- |
| 1    | **Bun** ([bun.sh](https://bun.sh)), **Polygon wallet** with USDC, **Polymarket** account |
| 2    | `git clone <repo> && cd polymarket-copy-trading-bot && bun install`                      |
| 3    | `cp .env.example .env` → set `PRIVATE_KEY` and `TARGET_WALLET`                           |
| 4    | `bun src/index.ts` (creates API credentials on first run)                                |

**Commands**

| What                   | Command                                                                    |
| ---------------------- | -------------------------------------------------------------------------- |
| Run bot                | `bun src/index.ts` or `npm start`                                          |
| Auto-redeem (holdings) | `bun src/cli/auto-redeem.ts` or `npm run auto-redeem`                      |
| Auto-redeem (dry run)  | `bun src/cli/auto-redeem.ts --dry-run`                                     |
| Auto-redeem (from API) | `bun src/cli/auto-redeem.ts --api`                                         |
| Redeem one market      | `bun src/cli/redeem.ts <conditionId>` or `npm run redeem -- <conditionId>` |
| Check market           | `bun src/cli/auto-redeem.ts --check <conditionId>`                         |

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
├── index.ts              # Bot entry (WebSocket + copy + optional auto-redeem)
├── cli/
│   ├── redeem.ts         # Single-market redeem / check
│   └── auto-redeem.ts    # Batch redeem (holdings or API)
├── redemption/           # Redemption logic (CTF, API, auto-redeem)
│   └── redeem.ts
├── data/
│   ├── credential.json   # API creds (auto-created)
│   └── token-holding.json
├── order-builder/        # Trade → order (multiplier, limits, FAK/FOK)
├── providers/            # CLOB, WebSocket, RPC
├── security/             # Allowance, createCredential
└── utils/                # balance, holdings, logger, types
```

**Stack:** Bun, TypeScript, `@polymarket/clob-client`, `@polymarket/real-time-data-client`, Ethers v6, Polygon.

---

## Security & safety

- Private key and API creds from env/file only (never hardcoded).
- Allowances and balance checks before orders.
- Start with small `SIZE_MULTIPLIER` and low `MAX_ORDER_AMOUNT`; use `--dry-run` for redemption tests.

**Risks:** Market/liquidity/slippage, gas, API limits, latency. Use at your own risk; never risk more than you can afford to lose.

---

## Development

```bash
bun run tsc --noEmit
bun --watch src/index.ts
```

---

## License & contributing

**License:** ISC. Contributions welcome (TypeScript, types, error handling, docs). For issues or questions, open a GitHub issue or check Polymarket API docs.

**Disclaimer:** Provided as-is. Prediction markets and crypto are risky; use at your own discretion.
