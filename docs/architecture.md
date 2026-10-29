# Architecture

Transparent SACCO is a Logos stack product: LEZ program + core module + UI module, distributed as `.lgx` packages for Basecamp and standalone hosts.

## Components

### LEZ program — `programs/sacco_ledger`

On-chain (zone) logic for SACCOs, members, deposits, loans, expenses, governance, and period summaries. LEZ stores ledger records and approval state. Actual savings deposits, loan disbursements, repayments, and expense payments happen off chain and are evidenced by references or CIDs.

**Attestation model:** the program records and approves off-chain money events. It does not receive, hold, or transfer SACCO money. `ExecuteExpense` records that an approved off-chain payment was made; it does not make the payment. The former fee-custody proposal is historical only; see [custody.md](./custody.md).

| Instruction | Purpose |
|-------------|---------|
| `Initialize` | Create the config PDA |
| `RegisterSacco` | Register a SACCO and its governance config; no funds are deposited |
| `VerifySacco` | Mark the SACCO verified |
| `AddApprover` / `AddSupervisor` | Multi-sig approvers and the separate Supervisory Committee |
| `AddMember` | Register a member identity |
| `RecordDeposit` / `ConfirmDeposit` | Attest to an off-chain savings or share-capital deposit; member confirms the record |
| `ApplyLoan` → `ApproveLoan` / `RejectLoan` → `DisburseLoan` → `RecordRepayment` | Record loan status, approvals, and evidence of off-chain disbursement and repayment |
| `ProposeExpense` → `ApproveExpense` / `RejectExpense` → `ExecuteExpense` | Approve an expense on chain; record evidence of its off-chain payment |
| `PostPeriodSummary` | Public snapshot of SACCO totals |

Implementation: Rust guest for the Logos Execution Environment (LEE / RISC Zero), SPEL macros, deployed with LEZ wallet tooling — not Solidity/EVM.

IDL: [idl.md](./idl.md) · [`artifacts/sacco_ledger-idl.json`](../artifacts/sacco_ledger-idl.json)

### Core module — `modules/sacco_core`

Headless Logos module that:

- Is currently a scaffold; its existing fee-payment and release methods are legacy stubs, not the current SACCO API
- Will submit `sacco_ledger` IDL instructions to record and approve off-chain SACCO activity
- Optionally stores evidence blobs via Logos Storage (CIDs referenced on-chain)
- Exposes methods the UI calls over Logos IPC

### UI module — `modules/sacco_ui`

QML UI scaffold for future member, treasurer, committee, and auditor views. The
current fee-payment and release controls are explicitly marked as legacy demos.
Runs inside:

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
| [custody.md](./custody.md) | Superseded custody proposal retained for history; not current design |
| [idl.md](./idl.md) | Client IDL artifact and regeneration |
| [basecamp-publishing.md](./basecamp-publishing.md) | Shipping modules to Basecamp |
