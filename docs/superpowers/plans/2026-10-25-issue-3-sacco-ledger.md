# Issue #3 — `sacco_ledger` Domain Model Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn `programs/fee_transparency` into `programs/sacco_ledger` — `lee_core` on v0.3.1, SACCO attestation domain (deposits with types, loans, expenses, multi-sig with Supervisory Committee), tests per instruction, CI — landing as one PR that closes #3.

**Architecture:** In-place rename preserving the proven multi-sig release flow, adapted to attestation (execution records, never transfers); custody code moves to a dormant `custody` module; new accounts/instructions follow the existing split (Borsh accounts + serde `Instruction` enum in `sacco_ledger_core`, one pure handler file per instruction, risc0 guest dispatch in `methods`).

**Tech Stack:** Rust 1.94.0, `lee_core` (logos-execution-zone v0.3.1), borsh, serde, spel-framework-macros v0.6.0, risc0-zkvm 3.0.5, GitHub Actions.

**Spec:** `docs/superpowers/specs/2026-10-25-issue-3-sacco-ledger-design.md` (branch `issue-3-sacco-ledger`)

## Global Constraints

- All commands run from `programs/`; test command is `RISC0_DEV_MODE=1 cargo test --workspace`.
- Workspace lints deny: `unwrap_used`, `todo`, `unimplemented`, `dbg_macro`, `unsafe_code`, clippy correctness/suspicious/perf. No `unwrap`/`expect` outside const-context codec macros already in the codebase.
- Accounts: Borsh + `#[account_type]`; instructions: serde. Follow the existing patterns in `core/src/lib.rs` and `src/*.rs`.
- One PR for the whole issue (`Closes #3`); commits sequence: build-fix/bump → rename → domain, in that order.
- Custody code (`pay_fees`, `initialize_institution_vault`, `execute_release` transfer paths, vault PDA helpers, old `Institution`/`FeeCategory`/`Payment` types) is **dormant, not deleted**: lives under `src/custody/` and `core` custody module, compiling, unexposed to the new `Instruction` enum.
- Commit messages follow the `push-and-commit-changes-to-github` standard (subject, `Changes:`, `Verification:`), conventional prefix, `#3` referenced.
- No week/date language in any doc or code comment this PR touches.

## Review Focus

Failure modes the task tests must pin (each is assigned below):

1. **Unverified SACCO accepts money** — every deposit/loan/expense handler must reject when `sacco.is_verified == false`. Tests: Task 6 (`record_deposit_rejected_for_unverified_sacco`), Task 7 (`apply_loan_rejected_for_unverified_sacco`), Task 8 (`propose_expense_rejected_for_unverified_sacco`).
2. **Approval threshold off-by-one** — approval counts strictly below `required_approvals` must not unlock; at exactly `required_approvals` must. Tests: Task 7 `approve_loan_reaches_threshold_at_exact_count`, Task 8 `approve_expense_below_threshold_stays_pending` / `approve_expense_executes_at_exact_threshold`.
3. **Duplicate approver double-counts** — the same approver approving twice must be rejected, never counted twice. Tests: Task 7 `approve_loan_duplicate_approver_rejected`, Task 8 `approve_expense_duplicate_approver_rejected`.
4. **Deposit-type totals cross-contaminate** — a `ShareCapital` deposit must not move `savings` totals and vice versa. Test: Task 6 `record_deposit_splits_totals_by_type`.
5. **Repayment edge** — repayment on a fully `Repaid` loan, or an amount above outstanding, must be rejected. Test: Task 7 `record_repayment_rejects_overpay_and_closed_loan`.

---

### Task 1: Make the workspace build — methods lib + `lee_core` v0.3.1

**Files:**
- Create: `programs/fee_transparency/methods/src/lib.rs`
- Modify: `programs/Cargo.toml` (workspace.dependencies: `nssa_core` tag `v0.2.0` → `v0.3.1`)
- Modify: any files broken by the v0.3 API (discovered by compile errors)

**Interfaces:**
- Produces: a green baseline build/test on v0.3.1 that every later task runs against.

- [ ] **Step 1: Add the missing methods lib target** — `methods/src/lib.rs` is the standard risc0 pattern: `include!(concat!(env!("OUT_DIR"), "/methods.rs"));` plus a doc comment. Run `RISC0_DEV_MODE=1 cargo fetch` — Expected: succeeds (no "no targets specified").
- [ ] **Step 2: Bump the tag in `programs/Cargo.toml`** and run `RISC0_DEV_MODE=1 cargo test --workspace`. Expected: compile errors from v0.3 API changes; fix each mechanically (rename/moved items in `nssa_core`), no behavior redesign. Baseline `tests.rs` passes.
- [ ] **Step 3: Full suite green** — Run: `RISC0_DEV_MODE=1 cargo test --workspace`. Expected: PASS.
- [ ] **Step 4: Commit** — `fix: build the LEZ workspace on lee_core v0.3.1 (fixes #3 in part)` with `Changes:` (methods lib target added; tag bumped; API breakages fixed) and `Verification:` (the test command + result).

### Task 2: Rename `fee_transparency` → `sacco_ledger`

**Files:**
- Rename: `programs/fee_transparency/` → `programs/sacco_ledger/` (git mv)
- Modify: all four crate `Cargo.toml`s (names `sacco_ledger`, `sacco_ledger_core`, `sacco_ledger-methods`, `sacco_ledger-guest`, bin `sacco_ledger`), workspace members/paths/dep names, `use` paths in all sources, `docs/architecture.md`, `docs/custody.md` (+ dormant note), `docs/idl.md`, `README.md` (§ Getting started), `docs/roadmap-sacco-pivot.md` ("How to start building" step 2), `artifacts/fee_transparency-idl.json` → `artifacts/sacco_ledger-idl.json` (name fields updated)

**Interfaces:**
- Produces: crate names `sacco_ledger_core`, `sacco_ledger`; module layout identical to today.

- [ ] **Step 1: Mechanical rename via `git mv` + targeted edits**; run suite. Expected: PASS (rename only, zero behavior change).
- [ ] **Step 2: Grep guard** — `grep -ri fee_transparency programs/ docs/ README.md artifacts/` returns hits only inside `src/custody/`-to-be files, `docs/custody.md` dormant section, and git history.
- [ ] **Step 3: Commit** — `refactor: rename fee_transparency to sacco_ledger (fixes #3 in part)` with Changes/Verification.

### Task 3: Core domain types

**Files:**
- Modify: `programs/sacco_ledger/core/src/lib.rs` (new types + `pub mod custody` holding old ones)
- Create: `programs/sacco_ledger/core/src/custody.rs` (moved `Institution`, `FeeCategory`, `Payment`, `FeeAllocation`, custody PDA helpers stay callable)
- Test: `programs/sacco_ledger/core/tests/types.rs` (new integration test target)

**Interfaces:**
- Produces (exact names later tasks rely on):
  - `pub enum DepositType { Savings = 0, ShareCapital = 1 }`
  - `pub enum LoanStatus { Applied = 0, Approved = 1, Disbursed = 2, Repaid = 3, Rejected = 4 }`
  - `pub enum ExpenseStatus { Proposed = 0, Approved = 1, Rejected = 2, Executed = 3 }`
  - `#[account_type] pub struct Sacco { name, registration_number, admin, is_verified, is_active, total_deposits_savings: u128, total_deposits_share_capital: u128, total_loans_disbursed: u128, total_loans_repaid: u128, total_expenses: u128, required_approvals: u8, approvers: Vec<AccountId>, supervisory_committee: Vec<AccountId>, member_ids: Vec<AccountId> }`
  - `#[account_type] pub struct Member { sacco_id: AccountId, identity: AccountId, savings_balance: u128, share_capital_balance: u128, loan_ids: Vec<AccountId> }`
  - `#[account_type] pub struct Deposit { sacco_id, member_id, amount: u128, deposit_type: DepositType, reference: String, recorded_by: AccountId, confirmed_by_member: bool }`
  - `#[account_type] pub struct Loan { sacco_id, member_id, principal: u128, purpose: String, status: LoanStatus, approvals: Vec<AccountId>, rejectors: Vec<AccountId>, evidence_cid: Option<String>, repaid_amount: u128 }`
  - `#[account_type] pub struct Expense { sacco_id, amount: u128, purpose: String, evidence_cid: String, requested_by: AccountId, status: ExpenseStatus, approvers: Vec<AccountId>, rejectors: Vec<AccountId>, executed_evidence: Option<String> }`
  - `#[account_type] pub struct PeriodSummary { sacco_id, label: String, deposits_savings: u128, deposits_share_capital: u128, loans_disbursed: u128, loans_repaid: u128, expenses: u128 }`

- [ ] **Step 1: Write failing tests** in `core/tests/types.rs`: `deposit_type_roundtrips`, `loan_status_discriminants`, `expense_status_discriminants`, `sacco_account_borsh_roundtrip`, `deposit_account_serde_roundtrip`, `period_summary_borsh_roundtrip` (assert exact field values through the codec, mirroring `impl_data_codec` usage).
- [ ] **Step 2: Run** `RISC0_DEV_MODE=1 cargo test -p sacco_ledger_core` — Expected: FAIL (types missing).
- [ ] **Step 3: Implement** the types in `core/src/lib.rs` with `impl_data_codec!` for each account; move old types to `core/src/custody.rs` inside `pub mod custody { ... }` and re-point the old custody handler files' `use`s. Keep `ProgramConfig` (drop `token_program_id` per design — custody module keeps its own config needs if it compiled against it; if only custody uses it, move it into `custody` module).
- [ ] **Step 4: Run** — Expected: PASS, and full workspace still compiles (old handlers now reference `custody::` types).
- [ ] **Step 5: Commit** — `feat: sacco_ledger core domain types (fixes #3 in part)`.

### Task 4: New `Instruction` enum (serde shape only)

**Files:**
- Modify: `programs/sacco_ledger/core/src/lib.rs`
- Test: `programs/sacco_ledger/core/tests/instruction_enum.rs`

**Interfaces:**
- Produces: `pub enum Instruction` with exactly these 18 variants (field names pinned): `Initialize {}`, `RegisterSacco { name, registration_number, treasury }`, `VerifySacco`, `AddApprover { approver }`, `AddSupervisor { supervisor }`, `AddMember { identity }`, `RecordDeposit { member, amount, deposit_type, reference }`, `ConfirmDeposit`, `ApplyLoan { amount, purpose }`, `ApproveLoan`, `RejectLoan`, `DisburseLoan { evidence_cid }`, `RecordRepayment { loan, amount, reference }`, `ProposeExpense { amount, purpose, evidence_cid }`, `ApproveExpense`, `RejectExpense`, `ExecuteExpense { executed_evidence }`, `PostPeriodSummary { label }`. Old custody variants (`InitializeInstitutionVault`, `PayFees`, `RequestRelease`, `ApproveRelease`, `RejectRelease`, `ExecuteRelease`, `CreateFeeCategory`, `RegisterInstitution`, `VerifyInstitution`) move to `custody::LegacyInstruction` in the custody module.

- [ ] **Step 1: Failing test** `instruction_enum_roundtrips_all_variants` — serialize/deserialize one instance of each of the 18 variants, assert identity.
- [ ] **Step 2: Run** — Expected: FAIL. **Step 3: Implement** enum (doc comments per variant listing its accounts, mirroring existing style). **Step 4: Run** — PASS.
- [ ] **Step 5: Commit** — `feat: sacco_ledger instruction set (fixes #3 in part)`.

### Task 5: SACCO setup handlers

**Files:**
- Create: `programs/sacco_ledger/src/{initialize,register_sacco,verify_sacco,add_approver,add_supervisor,add_member}.rs`
- Modify: `src/lib.rs` (modules + remove old module list into custody), old handler files move to `src/custody/`
- Test: `programs/sacco_ledger/tests/setup.rs`

**Interfaces:**
- Consumes: Task 3 account types, Task 4 enum variants.
- Produces: handler signature pattern all later tasks copy: `pub fn handle_<name>(...)` pure state-transition functions following the existing `src/*.rs` style (accounts + instruction data in, `Result` with the program's existing error type out). Helper `fn is_approver(sacco: &Sacco, id: &AccountId) -> bool` lives in `src/lib.rs`.

- [ ] **Step 1: Failing tests** in `tests/setup.rs`: `initialize_creates_config`, `register_sacco_records_admin_and_defaults` (required_approvals = 1, empty approvers/committee, totals zero), `register_sacco_rejects_bad_threshold` (0 rejected), `verify_sacco_requires_registrar` (non-registrar signer rejected), `add_approver_admin_only` + `add_approver_rejects_duplicate`, `add_supervisor_admin_only`, `add_member_admin_only` + `add_member_rejects_duplicate_identity`.
- [ ] **Step 2: Run** — FAIL. **Step 3: Implement** the six handlers. **Step 4: Run** — PASS.
- [ ] **Step 5: Commit** — `feat: sacco setup instructions (fixes #3 in part)`.

### Task 6: Deposits

**Files:**
- Create: `programs/sacco_ledger/src/{record_deposit,confirm_deposit}.rs`
- Test: `programs/sacco_ledger/tests/deposits.rs`

**Interfaces:**
- Consumes: `DepositType`, `Deposit`, `Sacco` totals fields, `is_approver`.

- [ ] **Step 1: Failing tests**: `record_deposit_updates_member_and_sacco_totals` (savings case), `record_deposit_splits_totals_by_type` (Savings → `savings_balance`/`total_deposits_savings` only; ShareCapital → the pair only — the Review Focus #4 pin), `record_deposit_rejected_for_unverified_sacco` (#1 pin), `record_deposit_requires_approver`, `record_deposit_rejects_empty_reference`, `confirm_deposit_member_only`, `confirm_deposit_rejects_double_confirm`.
- [ ] **Step 2–4:** FAIL → implement (treasurer = any approver records; member balance and both totals move per type; `confirmed_by_member` flips only with the member's signature) → PASS.
- [ ] **Step 5: Commit** — `feat: record and confirm deposits with typed totals (fixes #3 in part)`.

### Task 7: Loans

**Files:**
- Create: `programs/sacco_ledger/src/{apply_loan,approve_loan,reject_loan,disburse_loan,record_repayment}.rs`
- Test: `programs/sacco_ledger/tests/loans.rs`

**Interfaces:**
- Consumes: `LoanStatus`, `Loan`, `is_approver`; produces the multi-sig helper `fn approval_count(loan: &Loan) -> usize` pattern reused in Task 8 by analogy.

- [ ] **Step 1: Failing tests**: `apply_loan_creates_applied_loan` (member applies; `loan_ids` grows), `apply_loan_rejected_for_unverified_sacco` (#1), `approve_loan_reaches_threshold_at_exact_count` (#2: with `required_approvals = 2`, two distinct approvers → `Approved`), `approve_loan_duplicate_approver_rejected` (#3), `approve_loan_non_approver_rejected`, `reject_loan_approver_only`, `disburse_loan_admin_only_sets_evidence_and_status` (`evidence_cid` non-empty enforced; totals/member updates), `record_repayment_updates_outstanding_and_totals`, `record_repayment_rejects_overpay_and_closed_loan` (#5).
- [ ] **Step 2–4:** FAIL → implement (status machine strictly `Applied → Approved/Rejected → Disbursed → Repaid`; repayments accumulate `repaid_amount`, cap at principal) → PASS.
- [ ] **Step 5: Commit** — `feat: loan lifecycle with multi-sig approval (fixes #3 in part)`.

### Task 8: Expenses

**Files:**
- Create: `programs/sacco_ledger/src/{propose_expense,approve_expense,reject_expense,execute_expense}.rs`
- Test: `programs/sacco_ledger/tests/expenses.rs`

**Interfaces:**
- Consumes: `ExpenseStatus`, `Expense`, `is_approver`; adapts the proven release-flow logic from the dormant custody `request/approve/reject/execute_release` handlers.

- [ ] **Step 1: Failing tests**: `propose_expense_rejected_for_unverified_sacco` (#1), `propose_expense_requires_approver_and_non_empty_purpose`, `approve_expense_below_threshold_stays_pending` (#2), `approve_expense_executes_at_exact_threshold` — approval flips status to `Approved`, **execution is a separate instruction** (#2), `approve_expense_duplicate_approver_rejected` (#3), `reject_expense_approver_only`, `execute_expense_records_off_chain_release` (status → `Executed`, `executed_evidence` required, `total_expenses` grows, **no token/vault accounts touched**).
- [ ] **Step 2–4:** FAIL → implement → PASS.
- [ ] **Step 5: Commit** — `feat: expense proposal, multi-sig approval and recorded execution (fixes #3 in part)`.

### Task 9: Period summary + guest dispatch

**Files:**
- Create: `programs/sacco_ledger/src/post_period_summary.rs`
- Modify: `programs/sacco_ledger/methods/guest/src/bin/fee_transparency.rs` → renamed `sacco_ledger.rs` (if not already in Task 2): dispatch match wiring all 18 instructions to handlers
- Test: `programs/sacco_ledger/tests/period_summary.rs`

**Interfaces:**
- Consumes: all handlers (Tasks 5–8), guest dispatch pattern from the current guest bin.

- [ ] **Step 1: Failing tests**: `post_period_summary_snapshots_totals` (seeded Sacco totals copied into `PeriodSummary` per label), `post_period_summary_rejects_duplicate_label`.
- [ ] **Step 2–4:** FAIL → implement handler → PASS; then wire the guest `match` for all 18 variants, `RISC0_DEV_MODE=1 cargo test --workspace` green.
- [ ] **Step 5: Commit** — `feat: period summaries and guest dispatch (fixes #3 in part)`.

### Task 10: CI workflow

**Files:**
- Create: `.github/workflows/lez-program.yml`

- [ ] **Step 1: Write the workflow** — on push/PR touching `programs/**` or the workflow; `dtolnay/rust-toolchain@1.94.0`; steps: `cargo fmt --check` (if fmt config exists; else skip), `RISC0_DEV_MODE=1 cargo test --workspace`, `cargo clippy --workspace -- -D warnings`.
- [ ] **Step 2: Verify** — `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/lez-program.yml'))"` parses; job step names/shell lines reviewed.
- [ ] **Step 3: Commit** — `ci: run LEZ program tests and clippy on RISC0 dev mode (fixes #3 in part)`.

### Task 11: IDL artifact, docs, final sweep

**Files:**
- Modify: `artifacts/sacco_ledger-idl.json` (hand-updated to the 18-instruction shape, mirroring the existing JSON structure), `docs/idl.md`, `docs/custody.md` (dormant module map), `docs/architecture.md` if stale names remain.

- [ ] **Step 1: Update IDL + docs; `python3 -m json.tool artifacts/sacco_ledger-idl.json` parses; grep guards: no `fee_transparency`/`Institution`/`PayFees` outside custody + history; no schedule language in touched docs.
- [ ] **Step 2: Full suite** — `RISC0_DEV_MODE=1 cargo test --workspace` green.
- [ ] **Step 3: Commit** — `docs: regenerate sacco_ledger IDL and update program docs (fixes #3 in part)`.

### Task 12: Review and land the PR

- [ ] **Step 1:** Run `superpowers:requesting-code-review` on the branch (fresh reviewer; include the spec + this plan + Review Focus list).
- [ ] **Step 2:** Fix findings TDD-style; full suite green.
- [ ] **Step 3:** Land per `issues-to-pr`: push via `push-and-commit-changes-to-github`, open PR `Closes #3` (body: what/why/verification per the design), comment on #3 linking the PR. Stop — merge is a human call.
