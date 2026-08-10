# Architecture

Transparent SACCO is a Logos stack product: LEZ program + core module + UI module, distributed as `.lgx` packages for Basecamp and standalone hosts.

## Components

### LEZ program — `programs/fee_transparency`

On-chain (zone) logic for institutions, fee categories, payments, release requests, and multi-sig approvals. Prefer **public** LEZ accounts for the audit trail so payers can verify allocations without a privileged indexer.

**Custody:** institution vault PDA + category ledger. Full design: [custody.md](./custody.md).

| Instruction | Purpose |
|-------------|---------|
| `Initialize` | Create config PDA; store Token Program id |
| `RegisterInstitution` | Onboard a university, SACCO, or cooperative |
| `VerifyInstitution` | Mark institution verified |
| `InitializeInstitutionVault` | Bind fee token + custody vault PDA |
| `AddApprover` | Register multi-sig approver |
| `CreateFeeCategory` | Category with basis-point allocation |
| `PayFees` | Ledger allocate + `Token::Transfer` into vault |
| `RequestRelease` | Propose release with purpose/evidence |
| `ApproveRelease` / `RejectRelease` | Multi-sig votes |
| `ExecuteRelease` | Ledger debit + `Token::Transfer` vault → treasury |

Implementation: Rust guest for the Logos Execution Environment (LEE / RISC Zero), SPEL macros, deployed with LEZ wallet tooling — not Solidity/EVM.

IDL: [idl.md](./idl.md) · [`artifacts/fee_transparency-idl.json`](../artifacts/fee_transparency-idl.json)

### Core module — `modules/sacco_core`

Headless Logos module that:

- Will talk to the LEZ wallet / program APIs using the IDL (scaffold APIs today)
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

There is no central app-store gatekeeper. Discovery is repository-based. See [basecamp-publishing.md](./basecamp-publishing.md).

## Prototype boundary

`prototype/` is a React + Hardhat + Solidity demo used for UX and research narrative. It must not be described as the Logos production stack. Production path is LEZ + modules above.

## Doc map

| Doc | Topic |
|-----|--------|
| [custody.md](./custody.md) | How fees are locked and released on LEZ |
| [idl.md](./idl.md) | Client IDL artifact and regeneration |
| [basecamp-publishing.md](./basecamp-publishing.md) | Shipping modules to Basecamp |
