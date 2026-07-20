# LOGOS Fee Transparency Platform

**Every fee accounted for. Every shilling transparent.**

A blockchain-powered platform that locks institutional fees using smart contracts, ensuring funds are only released for their stated purpose. Built on the Logos blockchain for universities, SACCOs, and cooperatives in Uganda.

---

## The Problem

Fees paid to schools, universities, SACCOs, and cooperatives disappear into systems with no accountability. Payers have no way to verify where the money goes.

We used **Makerere University** and **Victoria University** as real research anchors to test whether a transparency solution could work at that scale. The answer was clear: **it can, and it's needed.**

## The Solution

Smart contracts lock every fee payment and automatically allocate funds to predefined categories. Funds can only be released through multi-signature approval for their stated purpose.

### Key Features

- **Immutable Fee Locking** — Payments are locked on-chain and allocated to categories
- **Real-Time Verification** — Anyone can verify where their money sits
- **Multi-Sig Release** — Funds require multiple approvals before release
- **Full Audit Trail** — Every transaction is permanently recorded on-chain
- **Category Allocation** — Tuition, library, ICT, sports — each with exact percentages

---

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Blockchain | Logos (EVM-compatible) |
| Smart Contracts | Solidity 0.8.19 |
| Frontend | React 18 + Vite |
| Styling | Tailwind CSS |
| Charts | Recharts |
| Web3 | ethers.js v6 |
| Dev Tools | Hardhat |

---

## Quick Start

### 1. Install Dependencies

```bash
npm install
```

### 2. Run the Frontend (Demo Mode)

```bash
npm run dev
```

The app runs with demo data out of the box — no blockchain connection required.

### 3. Deploy Smart Contract (Local)

```bash
# Start local blockchain
npx hardhat node

# Deploy and seed data (in another terminal)
npm run deploy:local
```

### 4. Deploy to Logos Blockchain

```bash
# Copy and configure environment
cp .env.example .env
# Edit .env with your LOGOS_RPC_URL and PRIVATE_KEY

npm run deploy:logos
```

---

## Smart Contract

The `FeeTransparency.sol` contract includes:

| Function | Description |
|----------|------------|
| `registerInstitution()` | Register a university, SACCO, or cooperative |
| `createFeeCategory()` | Define fee categories with percentage allocations |
| `payFees()` | Pay fees — auto-allocated and locked on-chain |
| `requestRelease()` | Request fund release with purpose and evidence |
| `approveRelease()` | Multi-sig approval for fund release |
| `verifyPaymentByRef()` | Public verification of any payment |

---

## Project Structure

```
├── contracts/
│   └── FeeTransparency.sol      # Core smart contract
├── scripts/
│   └── deploy.js                # Deployment & seeding script
├── src/
│   ├── components/              # Reusable UI components
│   ├── context/                 # React context (state management)
│   ├── pages/                   # Application pages
│   │   ├── Dashboard.jsx        # Overview with charts & stats
│   │   ├── Institutions.jsx     # Browse registered institutions
│   │   ├── InstitutionDetail.jsx # Detailed view with categories
│   │   ├── PayFees.jsx          # Fee payment flow
│   │   ├── VerifyPayment.jsx    # Payment verification
│   │   ├── Releases.jsx         # Fund release tracking
│   │   └── About.jsx            # Project information
│   └── utils/                   # Helpers, ABI, demo data
├── hardhat.config.js
├── tailwind.config.js
└── package.json
```

---

## Research Anchors

- **Makerere University** — 45,000+ students, 7+ functional fee categories per semester
- **Victoria University** — 12,000+ students, technology and library fees with no visibility

Both institutions were used to validate that blockchain transparency can operate at institutional scale.

---

## License

MIT
