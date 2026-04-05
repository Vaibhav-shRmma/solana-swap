# Solana Swap — AMM on Solana

A minimal but fully functional **Automated Market Maker (AMM)** built on Solana using the Anchor framework. This project demonstrates core DeFi mechanics — liquidity provisioning, constant-product swaps, and configurable fees — all implemented as an on-chain Solana program.

---

## What It Does

| Feature | Details |
|---|---|
| Initialize Pool | Deploy a liquidity pool with a configurable swap fee (e.g. 0.3%) |
| Add Liquidity | Deposit Token A and Token B into the pool |
| Swap A → B | Trade Token A for Token B using the AMM formula |
| Swap B → A | Trade Token B for Token A using the AMM formula |
| Remove Liquidity | Withdraw a proportional share of both tokens from the pool |

---

## How It Works

This AMM uses the **constant product formula**:

```
x * y = k
```

- `x` = reserve of Token A in the pool
- `y` = reserve of Token B in the pool
- `k` = constant that must be maintained after every swap

When a user swaps Token A for Token B, the pool receives Token A and sends out Token B — adjusted so that `x * y` remains constant. A small fee is deducted before the swap, which stays in the pool as earnings for liquidity providers.

**Fee calculation example (0.3% fee):**
```
amount_after_fee = amount_in * (1 - fee_numerator / fee_denominator)
                 = amount_in * (1 - 3/1000)
                 = amount_in * 0.997
```

---

## Tech Stack

- **Rust** — Smart contract (program) logic
- **Anchor** — Solana's framework for writing and testing programs
- **TypeScript** — Test suite and client interaction
- **Solana Web3.js** — Blockchain interaction
- **SPL Token Program** — Token minting, transfers, and account management

---

## Project Structure

```
solana-swap/
├── programs/
│   └── solana-swap/
│       └── src/lib.rs        # On-chain program logic (Rust)
├── tests/
│   └── solana-swap.ts        # Integration test suite (TypeScript)
├── Anchor.toml
└── README.md
```

---

## Test Suite Walkthrough

The test suite (`tests/solana-swap.ts`) runs 5 end-to-end integration tests on a local Solana validator. Here's exactly what each test does:

### Setup (before all tests)

Before tests run, the following state is initialized:

- Two SPL token mints created: **Token A** and **Token B** (both 6 decimals)
- Pool PDA (Program Derived Address) derived from the seed `"pool"`
- Pool token accounts created for both mints, owned by the PDA
- User token accounts created for the wallet
- **1,000,000,000 units** of Token A minted to the user
- **1,000,000,000 units** of Token B minted to the user
- **500,000,000 units** of Token A minted to the pool
- **500,000,000 units** of Token B minted to the pool

---

### Test 1 — Initialize the Pool

```
program.methods.initializePool(feeNumerator: 3, feeDenominator: 1000)
```

Initializes the on-chain `Pool` account at the PDA with:
- **Fee:** `3/1000` = **0.3%** per swap
- **Authority:** set to the deployer's wallet

**Asserts:** pool fee values and authority match the input.

---

### Test 2 — Add Liquidity

```
program.methods.addLiquidity(amountA: 100_000_000, amountB: 100_000_000)
```

Deposits **100,000,000 units each** of Token A and Token B from the user's wallet into the pool's token accounts.

**Asserts:** pool Token A balance increases by exactly `100,000,000`.

> Pool balances after: ~600M Token A, ~600M Token B

---

### Test 3 — Swap Token A → Token B

```
program.methods.swapAToB(amountIn: 50_000_000)
```

Swaps **50,000,000 units of Token A** from the user's wallet into the pool, and receives Token B back — calculated using the constant product formula minus the 0.3% fee.

**Asserts:** user Token B balance increases (i.e. swap produced a non-zero output).

The exact Token B received depends on pool reserves at the time of the swap (slightly less than 50M due to the 0.3% fee and price impact).

---

### Test 4 — Swap Token B → Token A

```
program.methods.swapBToA(amountIn: 50_000_000)
```

Swaps **50,000,000 units of Token B** into the pool and receives Token A back.

**Asserts:**
- Token A received by user is greater than 0
- Exactly `50,000,000` Token B was deducted from the user

The exact Token A received depends on pool reserves at the time of the swap (slightly less than 50M due to the 0.3% fee and price impact).

---

### Test 5 — Remove Liquidity

```
program.methods.removeLiquidity(shareNumerator: 1, shareDenominator: 2)
```

Removes **50% of the pool's liquidity** (ratio `1/2`). Both Token A and Token B are proportionally withdrawn from the pool and sent back to the authority's wallet.

**Asserts:**
- Pool Token A balance decreases
- Pool Token B balance decreases
- User receives exactly the same amounts that left the pool (no slippage on removal)

The exact amounts reflect the pool state after both swaps have run.

---

## Setup & Run

### Prerequisites

- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools)
- [Anchor CLI](https://www.anchor-lang.com/docs/installation)
- Node.js v16+

### Install

```bash
git clone https://github.com/YOUR_USERNAME/solana-swap.git
cd solana-swap
npm install
```

### Build & Test

```bash
anchor build
anchor test
```

### Deploy to Devnet (Optional)

```bash
solana config set --url devnet
anchor deploy
```

---

## Key Concepts Demonstrated

- **PDAs (Program Derived Addresses)** — Pool state stored at a deterministic address, no private key needed
- **SPL Token CPI** — Cross-program invocations to the SPL Token program for transfers
- **Constant Product AMM** — Core DeFi primitive powering protocols like Uniswap v1/v2
- **Fee mechanism** — Configurable fee taken on each swap, accrues to the pool

---

## License

MIT
