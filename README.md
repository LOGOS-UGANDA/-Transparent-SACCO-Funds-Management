# Transparent SACCO

<p align="left">
  <a href="./LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-blue.svg"></a>
  <img alt="Status: in development" src="https://img.shields.io/badge/status-in%20development-orange">
  <img alt="Platform: Logos LEZ" src="https://img.shields.io/badge/platform-Logos%20LEZ-6C47FF">
</p>

**Every shilling accounted for. Every record verifiable.**

Transparent SACCO is a Logos-native **SACCO ledger**: every deposit, loan and expense a savings and credit cooperative handles is recorded and approved on the [Logos Execution Zone (LEZ)](https://docs.logos.co/), with multi-signature approval before any expense passes and members' personal balances kept private. It ships both as a **standalone desktop app** and as a **Basecamp module** users install from a package repository.

> **Status:** the pivot from the earlier university fee-transparency model is decided and the [pivot roadmap](./docs/roadmap-sacco-pivot.md) is the plan of record (12 steps, 8 phases). Phase 1 — the `sacco_ledger` LEZ program — is in flight on [issue #3](https://github.com/LOGOS-UGANDA/transparent-sacco/issues/3).

## Contents

- [The problem](#the-problem)
- [The solution: a ledger, not a custodian](#the-solution-a-ledger-not-a-custodian)
- [Architecture](#architecture-logos-native)
- [Roadmap](#roadmap)
- [Getting started](#getting-started)
- [Repository layout](#repository-layout)
- [Contributing](#contributing)
- [Research anchors](#research-anchors)
- [License](#license)

---

## The problem

SACCO members cannot see how their savings, loans and expenses are handled. Records live in treasurers' books and spreadsheets; trust is low, and when something goes wrong — lost savings, disputed expenses, unresolved estates — there is no shared record anyone can check.

## The solution: a ledger, not a custodian

The program is **the record, not the custodian of value**. Real money keeps moving through mobile money, bank accounts and cash; the chain attests to what happened:

| Capability | What it does |
|------------|--------------|
| Typed deposits | Every deposit recorded with type (savings vs share capital), reference and member confirmation |
| Loan lifecycle | Apply → multi-sig approve → disburse (with evidence) → repay, all on chain |
| Multi-sig expenses | Expenses need multiple approvals against a stated purpose and evidence before execution is recorded |
| Supervisory oversight | A Supervisory Committee is distinct from approvers — approve ≠ audit |
| Per-SACCO config | Approver thresholds and role composition are configured per SACCO, not chain-wide |
| Public totals, private members | SACCO totals are public and auditable; individual balances become private in the privacy phase |

## Architecture (Logos-native)

This is **not** an EVM / Solidity dApp. Logos LEZ runs RISC-V programs (RISC Zero zkVM). The product ships as Logos modules packaged as `.lgx` files.

```text
┌─────────────────────────────────────────────────────────┐
│  Transparent SACCO                                       │
│  • programs/sacco_ledger      — LEZ program (in flight)  │
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
| On-chain logic | LEZ program (Rust guest, RISC Zero) — attestation model, no custody |
| App runtime | Logos Core modules (`.lgx`) |
| UI | QML UI module (`logos-standalone-app` or Basecamp) |
| Distribution | `logos-repo.json` catalogue + release index |

Docs: [docs.logos.co](https://docs.logos.co/) · [Basecamp](https://logos.co/basecamp)

---

## Roadmap

The [pivot roadmap](./docs/roadmap-sacco-pivot.md) carries the full plan: 8 phases across 12 steps — domain model, deploy, core/UI wiring, privacy (private member accounts), storage and messaging (receipts by CID, member alerts), packaging, and a real-SACCO pilot with published metrics. Work is tracked as phase issues; see [CONTRIBUTING.md](./CONTRIBUTING.md) before picking one up.

## Getting started

### 1. Install Basecamp (recommended)

Download from [logos.co/basecamp](https://logos.co/basecamp) and use it to explore the stack (wallet, LEZ, Package Manager) while the product is under construction.

### 2. Work on the program

```bash
cd programs
RISC0_DEV_MODE=1 cargo test --workspace
```

See [docs/testing.md](./docs/testing.md) and the phase issues for what's next.

### 3. Explore the earlier web prototype (optional, legacy)

The React + Hardhat demo illustrates the original fee-transparency flows. It predates the pivot and is **not** the Logos production path.

```bash
cd prototype && npm install && npm run dev
```

---

## Repository layout

```text
.
├── README.md
├── LICENSE                     # MIT license
├── CONTRIBUTING.md             # ground rules for contributors and AI agents
├── logos-repo.json             # Basecamp package-repository entry (filled at Phase 6)
├── docs/
│   ├── roadmap-sacco-pivot.md  # plan of record
│   ├── architecture.md · custody.md · idl.md · testing.md · basecamp-publishing.md
│   └── superpowers/            # design and plan records
├── artifacts/                  # program IDL
├── programs/                   # LEZ program (fee_transparency → sacco_ledger, Phase 1)
├── modules/
│   ├── sacco_core/             # headless core module (scaffold)
│   └── sacco_ui/               # QML UI module (scaffold)
└── prototype/                  # earlier React + Hardhat demo — reference only
```

## Contributing

Contributions are welcome — from humans and AI coding agents alike. Read [CONTRIBUTING.md](./CONTRIBUTING.md) first: work is organized as phase issues, branches are named `phase<N>/<short-topic>`, commits follow [conventional commits](https://www.conventionalcommits.org/), and nothing lands on `main` without pull-request review.

## Research anchors

- **Makerere University** — 45,000+ students; the original fee-transparency study site
- **Ugandan SACCO landscape** — Wazalendo, Parliamentary SACCO, KACITA, Yetu, Bugongi, Obulamu and others informed the pivot (membership patterns, governance separation, the deceased-members recovery problem); see the roadmap's open questions

## License

Released under the [MIT License](./LICENSE).
