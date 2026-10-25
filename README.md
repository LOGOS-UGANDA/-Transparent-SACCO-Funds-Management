# Transparent SACCO on Logos

**Every fee accounted for. Every shilling transparent.**

A Logos-native platform that locks institutional fees on the [Logos Execution Zone (LEZ)](https://docs.logos.co/), so funds are only released for their stated purpose. Built for universities, SACCOs, and cooperatives — shippable both as a **standalone app** and as a **Basecamp module** users can install from a package repository.

> **Pivot:** the product is pivoting from university fee transparency to a **SACCO ledger** — deposits, loans and expenses recorded and approved on chain, with member balances private. The university-fee model below is the earlier concept and stays for reference. See the [pivot roadmap](./docs/roadmap-sacco-pivot.md) (12 steps, 8 phases).

---

## The problem

Fees paid to schools, universities, SACCOs, and cooperatives disappear into systems with no accountability. Payers have no way to verify where the money goes.

We used **Makerere University** and **Victoria University** as research anchors to test whether a transparency solution could work at institutional scale. It can — and it is needed.

## The solution

On-chain logic locks every fee payment and allocates funds to predefined categories. Release requires multi-signature approval tied to a stated purpose and evidence. Anyone can verify a payment’s allocation and status.

| Capability | What it does |
|------------|--------------|
| Immutable fee locking | Payments are locked on LEZ and allocated to categories |
| Real-time verification | Anyone can verify where funds sit |
| Multi-sig release | Funds need multiple approvals before release |
| Full audit trail | Public LEZ state records every material action |
| Category allocation | Tuition, library, ICT, sports — each with exact percentages |

---

## Architecture (Logos-native)

This is **not** an EVM / Solidity dApp. Logos LEZ runs RISC-V programs (RISC Zero zkVM). The product ships as Logos modules packaged as `.lgx` files.

```text
┌─────────────────────────────────────────────────────────┐
│  Transparent SACCO                                       │
│  • programs/fee_transparency  — LEZ program              │
│  • modules/sacco_core         — core module (APIs)       │
│  • modules/sacco_ui           — QML UI module            │
└───────────────────────────┬─────────────────────────────┘
                            │
            ┌───────────────┴───────────────┐
            ▼                               ▼
   logos-standalone-app              Logos Basecamp
   (focused desktop app)             (shell + Package Manager)
```

| Layer | Technology |
|-------|------------|
| Execution | Logos Execution Zone (LEZ) / LEE |
| On-chain logic | LEZ program (Rust guest) |
| App runtime | Logos Core modules (`.lgx`) |
| UI | QML UI module (`logos-standalone-app` or Basecamp) |
| Distribution | `logos-repo.json` catalogue + release index |
| Prototype (non-production) | React + Solidity demo under `prototype/` |

Docs: [docs.logos.co](https://docs.logos.co/) · [Basecamp](https://logos.co/basecamp)

---

## Repository layout

```text
.
├── README.md
├── logos-repo.json              # Basecamp package-repository entry (fill in when publishing)
├── docs/                        # Architecture, custody, IDL, publishing
├── artifacts/                   # fee_transparency IDL
├── programs/
│   └── fee_transparency/        # LEZ program (custody-enabled)
├── modules/
│   ├── sacco_core/              # Headless core module (scaffold APIs)
│   └── sacco_ui/                # QML UI module (scaffold)
└── prototype/                   # Earlier React + Hardhat demo (not Logos-native)
```

---

## Status

| Area | Status |
|------|--------|
| Product research & UX flows | Captured in `prototype/` |
| LEZ fee-transparency program | Types, handlers, guest entrypoints, **institution-vault custody** |
| IDL | [`artifacts/fee_transparency-idl.json`](./artifacts/fee_transparency-idl.json) (see [docs/idl.md](./docs/idl.md)) |
| Core + UI Logos modules | Scaffold: `sacco_core` + `sacco_ui` (Nix / QML); LEZ wiring next |
| Standalone + Basecamp packaging | Planned (`.lgx` + filled `logos-repo.json`) |
| Public module repository | Placeholder `logos-repo.json` |

### Docs

| Doc | Topic |
|-----|--------|
| [docs/architecture.md](./docs/architecture.md) | Stack overview |
| [docs/custody.md](./docs/custody.md) | How fees are locked / released |
| [docs/idl.md](./docs/idl.md) | Client IDL |
| [docs/basecamp-publishing.md](./docs/basecamp-publishing.md) | Basecamp distribution |
| [docs/testing.md](./docs/testing.md) | How to test (prototype, CLI, Basecamp) |

---

## Getting started

### 1. Install Basecamp (recommended)

Download from [logos.co/basecamp](https://logos.co/basecamp). Use it to explore the stack (wallet, LEZ, Package Manager) while this product is under construction.

### 2. Explore the product prototype (optional)

The web demo illustrates flows and research narrative. It uses a local Hardhat/Solidity stand-in and is **not** the Logos production path.

```bash
cd prototype
npm install
npm run dev
```

See [prototype/README.md](./prototype/README.md).

### 3. Build the Logos-native product (next)

1. Implement `programs/fee_transparency` as an LEZ program (port of the fee-lock / multi-sig model).
2. Scaffold modules with [`logos-module-builder`](https://docs.logos.co/core/build-modules/build-and-run-a-logos-core-module).
3. Preview UI via `logos-standalone-app` (`nix run` in the UI module).
4. Package `.lgx`, publish a release index, and point `logos-repo.json` at it.
5. Users add the repo in Basecamp: **Settings → Package Repositories → Add**, then install from Applications.

More detail: [docs/architecture.md](./docs/architecture.md) and [docs/basecamp-publishing.md](./docs/basecamp-publishing.md).

---

## Research anchors

- **Makerere University** — 45,000+ students, 7+ functional fee categories per semester
- **Victoria University** — 12,000+ students, technology and library fees with little payer visibility

Both were used to validate that on-chain transparency can operate at institutional scale.

---

## License

MIT
