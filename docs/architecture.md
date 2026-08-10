# Architecture

Transparent SACCO is a Logos stack product: LEZ program + core module + UI module, distributed as `.lgx` packages for Basecamp and standalone hosts.

## Components

### LEZ program — `programs/fee_transparency`

On-chain (zone) logic for institutions, fee categories, payments, release requests, and multi-sig approvals. Prefer **public** LEZ accounts for the audit trail so payers can verify allocations without a privileged indexer.

Target instruction set (port from the Solidity prototype):

| Instruction / capability | Purpose |
|--------------------------|---------|
| Register institution | Onboard a university, SACCO, or cooperative |
| Create fee category | Define categories with percentage allocations |
| Pay fees | Lock payment and allocate to categories |
| Request release | Propose a release with purpose and evidence |
| Approve release | Multi-sig approval toward release threshold |
| Verify payment | Public read of payment allocation and status |

Implementation target: Rust guest for the Logos Execution Environment (LEE / RISC Zero), deployed with LEZ wallet tooling — not Solidity/EVM.

### Core module — `modules/sacco_core`

Headless Logos module that:

- Talks to the LEZ wallet / program APIs
- Optionally stores evidence blobs via Logos Storage (CIDs referenced on-chain)
- Exposes methods the UI calls over Logos IPC

### UI module — `modules/sacco_ui`

QML UI for dashboard, institutions, pay fees, verify payment, and releases. Runs inside:

- **Basecamp** — discovered via Package Manager after the user adds this repo’s catalogue
- **`logos-standalone-app`** — focused desktop binary for institutions that do not want the full shell

QML UIs are sandboxed; network and chain access go through the core module (`logos.callModule(...)`), not direct HTTP from QML.

## Dual delivery

Same packages, two hosts:

1. **Standalone** — ship UI + required modules for a single-purpose client.
2. **Basecamp** — publish `.lgx` to a release index; users install from a module repository (`logos-repo.json`).

There is no central app-store gatekeeper. Discovery is repository-based.

## Prototype boundary

`prototype/` is a React + Hardhat + Solidity demo used for UX and research narrative. It must not be described as the Logos production stack. Production path is LEZ + modules above.
