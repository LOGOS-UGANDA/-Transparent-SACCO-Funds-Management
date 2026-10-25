# Issue #3 — Phase 1: `sacco_ledger` Domain Model and LEZ Program (Design)

| | |
|---|---|
| **Issue** | [#3 Phase 1: Domain model and LEZ program](https://github.com/LOGOS-UGANDA/-Transparent-SACCO-Funds-Management/issues/3) (Steps 2–4) |
| **Branch / PR** | `issue-3-sacco-ledger` → one PR, `Closes #3` |
| **Date** | 2026-10-25 |
| **Status** | Awaiting approval |

## 1. Intent

Turn `programs/fee_transparency` (university-fee custody model, `lee_core` v0.2.0) into `programs/sacco_ledger`: the attestation ledger for a SACCO's deposits, loans and expenses, with multi-sig approval, a Supervisory Committee distinct from approvers, per-SACCO approver configuration, and a test suite plus CI. The program is the record, not a custodian — no token transfers on the active path.

## 2. Goals

1. `lee_core` bumped v0.2.0 → **v0.3.1**; breakages fixed; workspace builds and tests pass.
2. Rename `fee_transparency` → `sacco_ledger` across crates, workspace, docs, artifacts; no university-fee concepts in the new domain.
3. New domain model (below) with unit tests per instruction and CI running `RISC0_DEV_MODE=1 cargo test`.
4. `RecordDeposit` carries `deposit_type: Savings | ShareCapital` (approved by the lead); totals per type; period summary reports both.

## 3. Non-goals / anti-goals

- **No privacy** (private accounts are Phase 4) — all accounts public now, types shaped so member data can move behind private accounts later.
- **No custody on the active path** — vault/token code stays in the repo as a documented dormant `custody` module, not dispatched by the new instruction set; deleting it is an anti-goal (settled in PR #1 review).
- No UI/module wiring (Phase 3), no Logos Storage integration (Phase 5; evidence stays a `String` field named `evidence_cid`).
- No single PR splitting — the lead ruled **one PR per issue**; within the PR, commits sequence bump → rename → domain so review reads chronologically.

## 4. Constraints

- Rust 1.94.0 (`programs/rust-toolchain.toml`); workspace lints deny `unwrap_used`, `todo`, `unimplemented`.
- Accounts are Borsh (`#[account_type]`), instructions serde (SPEL/wallet path) — keep the existing codec pattern.
- Local `cargo` resolution of git deps timed out in sandbox; CI on GitHub runners is the authoritative green. Implementation must make local runs possible where network allows.
- **Known scaffold gap:** `fee_transparency/methods/` has `build.rs` (`risc0_build::embed_methods()`) but no `src/lib.rs`, so the package has no target and `cargo fetch`/`build` fails with "no targets specified". The implementation adds the standard risc0 `methods/src/lib.rs` (guest ELF/image-id re-exports); git deps themselves fetch fine once that is fixed.

## 5. Approaches considered

| Approach | Assessment |
|---|---|
| Three stacked PRs (bump, rename, domain) | Rejected by the lead — each issue gets exactly one PR. |
| Fresh `sacco_ledger` crate written beside the old one | Rejected: duplicates custody code that must stay dormant, loses the multi-sig release flow that the roadmap explicitly reuses. |
| **In-place rename + domain swap (selected)** | Rename the existing crates, adapt the proven release flow into `ProposeExpense`/`ExecuteExpense`, add the new accounts/instructions, park custody code in a dormant module. Reuses tested structure; one PR. |

## 6. Domain model (selected design)

**Accounts** (Borsh, PDA/public per existing pattern):

- `Sacco` (was `Institution`): name, registration number, admin, is_verified/is_active, totals (`total_deposits_savings`, `total_deposits_share_capital`, `total_loans_disbursed`, `total_loans_repaid`, `total_expenses`), **approver config**: `approvers: Vec<AccountId>`, `required_approvals: u8`, plus `supervisory_committee: Vec<AccountId>` — per-SACCO configuration, not a chain-wide constant.
- `Member`: sacco id, identity AccountId, `savings_balance`, `share_capital_balance`, `loan_ids`.
- `Deposit`: sacco, member, `amount`, `deposit_type`, `reference` (mobile-money/bank ref), recorded_by, `confirmed_by_member: bool`.
- `Loan`: sacco, member, principal, purpose, status (`Applied | Approved | Disbursed | Repaid | Rejected`), approvals (multi-sig), `evidence_cid` on disbursement, repaid amount.
- `Expense` (was `ReleaseRequest`): sacco, amount, purpose, `evidence_cid`, requester, status (`Proposed | Approved | Rejected | Executed`), approvers/rejectors, `executed_evidence` — execution **records** the off-chain release; no token movement.
- `PeriodSummary`: sacco, label (e.g. `2026-11`), per-type deposit totals, loan movement, expense total.

**Instructions** (serde enum; handlers pure state transitions, one file each, tests beside them):

`Initialize` (no `token_program_id` — attestation needs none) · `RegisterSacco` · `VerifySacco` · `AddApprover` · `AddSupervisor` · `AddMember` · `RecordDeposit{ deposit_type }` · `ConfirmDeposit` (member sign-off) · `ApplyLoan` · `ApproveLoan` · `RejectLoan` · `DisburseLoan{ evidence_cid }` · `RecordRepayment` · `ProposeExpense` · `ApproveExpense` · `RejectExpense` · `ExecuteExpense` · `PostPeriodSummary`.

**Dormant custody**: `pay_fees.rs`, `initialize_institution_vault.rs`, `execute_release.rs` token-transfer paths move to `src/custody/` with module docs marking them dormant (reference: PR #1 review decision); not in the new `Instruction` enum; vault PDA helpers stay in core for that module's use.

**IDL**: `artifacts/fee_transparency-idl.json` regenerated as `artifacts/sacco_ledger-idl.json`; `docs/idl.md` and `docs/custody.md` updated (custody doc gains a "dormant" note); `docs/architecture.md` naming updated.

**CI**: `.github/workflows/lez-program.yml` — checkout, Rust 1.94, `RISC0_DEV_MODE=1 cargo test --workspace`, clippy with workspace lints.

## 7. Decisions ledger

- Deposit types: **yes** (lead, this design session).
- One PR per issue: lead, this design session.
- Everything else in §5–§6 is the builder's call as marked; veto any of it by commenting on this doc.

## 8. Verification

- `RISC0_DEV_MODE=1 cargo test --workspace` green locally (where network permits) and in CI.
- Per-instruction unit tests: happy path + auth failures + multi-sig threshold edges (below/at/above `required_approvals`), deposit-type split totals, loan lifecycle transitions, expense lifecycle, period summary math.
- `grep` checks: no `fee_transparency` / `Institution` / university wording left outside the dormant custody module and git history.
