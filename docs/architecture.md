# Architecture

Transparent SACCO is a Logos stack product: LEZ program + core module + UI module, distributed as `.lgx` packages for Basecamp and standalone hosts.

## Components

### LEZ program — `programs/sacco_ledger`

On-chain (zone) logic for institutions, fee categories, payments, release requests, and multi-sig approvals. Prefer **public** LEZ accounts for the audit trail so payers can verify allocations without a privileged indexer.

**Attestation model:** the program records and approves off-chain money events — it holds no value and performs no token transfers. (The earlier custody design — institution vault PDA + category ledger — is kept dormant; see [custody.md](./custody.md).)

| Instruction | Purpose |
|-------------|---------|
| `Initialize` | Create the config PDA |
| `RegisterSacco` | Onboard a SACCO (admin, treasury, per-SACCO approver config) |
| `VerifySacco` | Mark the SACCO verified |
| `AddApprover` / `AddSupervisor` | Multi-sig approvers and the separate Supervisory Committee |
| `AddMember` | Register a member identity |
| `RecordDeposit` / `ConfirmDeposit` | Treasurer records (savings vs share capital), member confirms |
| `ApplyLoan` → `ApproveLoan` / `RejectLoan` → `DisburseLoan` → `RecordRepayment` | Loan lifecycle with multi-sig and evidence |
| `ProposeExpense` → `ApproveExpense` / `RejectExpense` → `ExecuteExpense` | Multi-sig expenses; execution records the off-chain payment proof |
| `PostPeriodSummary` | Public snapshot of SACCO totals |

Implementation: Rust guest for the Logos Execution Environment (LEE / RISC Zero), SPEL macros, deployed with LEZ wallet tooling — not Solidity/EVM.

IDL: [idl.md](./idl.md) · [`artifacts/sacco_ledger-idl.json`](../artifacts/sacco_ledger-idl.json)

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
