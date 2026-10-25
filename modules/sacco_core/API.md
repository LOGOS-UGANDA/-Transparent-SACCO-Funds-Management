# `sacco_core` API contract — Phase 3 wiring (issue #5, first slice)

> The module methods `sacco_ui` (and CLI via `logosctl call`) use, mapped one-to-one to the `sacco_ledger` instruction set (`artifacts/sacco_ledger-idl.json`). Signers are wallet identities; every method returns the transaction result plus the decoded post-state diff for the UI.

## Setup

| Method | Instruction | Signer | Notes |
|---|---|---|---|
| `initialize()` | `Initialize` | admin | once per deployment |
| `registerSacco(name, registrationNumber, treasury)` | `RegisterSacco` | admin | sets threshold 1, admin as first approver |
| `verifySacco()` | `VerifySacco` | SACCO admin (v1 registrar) | |
| `addApprover(approver)` | `AddApprover` | admin | rejects duplicates and Supervisory overlap |
| `addSupervisor(supervisor)` | `AddSupervisor` | admin | rejects duplicates and approver overlap |
| `addMember(memberAccount, identity)` | `AddMember` | admin | creates the Member account |

## Deposits

| Method | Instruction | Signer | Notes |
|---|---|---|---|
| `recordDeposit(memberAccount, depositAccount, member, amount, type, reference)` | `RecordDeposit` | any approver (treasurer) | type: `savings` \| `shareCapital`; verified SACCO required |
| `confirmDeposit(depositAccount)` | `ConfirmDeposit` | the deposit's member | |

## Loans

| Method | Instruction | Signer | Notes |
|---|---|---|---|
| `applyLoan(memberAccount, loanAccount, amount, purpose)` | `ApplyLoan` | the member | |
| `approveLoan(loanAccount)` | `ApproveLoan` | approver | flips to Approved at exactly `requiredApprovals` |
| `rejectLoan(loanAccount)` | `RejectLoan` | approver | |
| `disburseLoan(memberAccount, loanAccount, evidenceCid)` | `DisburseLoan` | approver | evidence from Logos Storage |
| `recordRepayment(memberAccount, loanAccount, loan, amount, reference)` | `RecordRepayment` | approver | capped at outstanding |

## Expenses

| Method | Instruction | Signer | Notes |
|---|---|---|---|
| `proposeExpense(expenseAccount, amount, purpose, evidenceCid)` | `ProposeExpense` | approver | |
| `approveExpense(expenseAccount)` | `ApproveExpense` | approver | multi-sig threshold |
| `rejectExpense(expenseAccount)` | `RejectExpense` | approver | |
| `executeExpense(expenseAccount, executedEvidence)` | `ExecuteExpense` | approver | records the off-chain payment proof |

## Reporting

| Method | Instruction | Signer | Notes |
|---|---|---|---|
| `postPeriodSummary(summaryAccount, label)` | `PostPeriodSummary` | approver | public snapshot |

## Read-only queries (module-local)

| Method | Returns |
|---|---|
| `getSacco()` | decoded `Sacco` account (totals, approvers, threshold, committee) |
| `getMember(memberAccount)` | decoded `Member` (savings, share capital, loans) |
| `getDeposit(depositAccount)` / `getLoan(…)` / `getExpense(…)` | decoded records |
| `listPeriodSummaries()` | posted summaries for the SACCO |

## Wiring notes

- Call path: `sacco_ui` → `sacco_core` → LEZ wallet APIs → `sacco_ledger` guest; account discovery (config PDA, program id) comes from `docs/deploy.md` once the program id is recorded.
- Every mutation method takes the target accounts as handles; the UI never constructs PDAs itself.
- Error surface: handler assert messages are the user-facing errors (e.g. "Sacco is not verified", "Already approved by this approver") — the UI renders them verbatim.
