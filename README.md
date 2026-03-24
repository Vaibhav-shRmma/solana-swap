# Solana Swap

A simple **Automated Market Maker (AMM)** swap program built on **Solana** using the **Anchor framework**.
This project demonstrates core DeFi concepts like liquidity pools, token swaps, and fee mechanisms.

---

## Features

* Initialize a liquidity pool
* Add liquidity to the pool
* Swap Token A → Token B
* Swap Token B → Token A
* Remove liquidity from the pool
* Configurable swap fee (e.g., 0.3%)

---

## Tech Stack

* **Rust** — Smart contract development
* **Anchor** — Solana framework for programs
* **TypeScript** — Testing and client interaction
* **Solana Web3.js** — Blockchain interaction
* **SPL Token Program** — Token operations

---

## Project Structure

```
solana-swap/
├── programs/
│   └── solana-swap/
│       └── src/lib.rs       # Smart contract logic
├── tests/
│   └── solana-swap.ts       # Test cases
├── Anchor.toml
└── README.md
```

---

## How It Works

This project implements a basic AMM using the **constant product formula**:

```
x * y = k
```

* `x` = reserve of Token A
* `y` = reserve of Token B
* `k` = constant

Swaps adjust token reserves while maintaining this invariant (minus fees).

---

## Setup & Run 

### 1. Install Dependencies

Make sure you have installed:

* [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools)
* [Anchor](https://www.anchor-lang.com/docs/installation)
* Node.js (v16+ recommended)

---

### 2. Clone the Repository

```bash
git clone https://github.com/YOUR_USERNAME/solana-swap.git
cd solana-swap
```

---

### 3. Install Node Dependencies

```bash
npm install
```

---

### 4. Build the Program

```bash
anchor build
```

---

### 5. Run Tests

```bash
anchor test
```

---

## Deployment (Optional)

To deploy on devnet:

```bash
solana config set --url devnet
anchor deploy
```

## License

MIT License
