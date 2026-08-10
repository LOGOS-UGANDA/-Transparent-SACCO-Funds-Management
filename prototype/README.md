# Prototype — web demo (not Logos-native)

**Status: prototype / research UX only.**

This folder is the earlier React + Hardhat + Solidity demo. It illustrates product flows and the Makerere / Victoria research narrative. It is **not** the production Logos stack.

Production lives at the repo root: LEZ program under `programs/`, Logos modules under `modules/`, Basecamp catalogue in `logos-repo.json`. See the [root README](../README.md) and [docs/architecture.md](../docs/architecture.md).

---

## What this demo shows

Smart-contract-style fee locking, category allocation, multi-sig release, and verification — implemented against a local Hardhat network with a Solidity stand-in (`contracts/FeeTransparency.sol`).

### Key flows

- Immutable fee locking and category allocation
- Payment verification by reference
- Multi-sig fund release with purpose
- Dashboards for institutions and releases

### Stack (prototype only)

| Layer | Technology |
|-------|-----------|
| Contracts | Solidity 0.8.x + Hardhat |
| Frontend | React 18 + Vite + Tailwind |
| Web3 | ethers.js v6 |

---

## Quick start

```bash
npm install
npm run dev
```

Demo data works without a chain connection.

### Local contracts (optional)

```bash
npx hardhat node
# other terminal
npm run deploy:local
```

Copy `.env.example` to `.env` if you need deployer / RPC settings for experiments. Do not treat `LOGOS_RPC_URL` here as the real LEZ deployment path.

---

## License

MIT (same as the parent repository)
