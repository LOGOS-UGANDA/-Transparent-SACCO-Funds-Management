# `sacco_ledger` on `lee_core` v0.3 — design

Issue: [#3 Phase 1: Domain model and LEZ program](https://github.com/LOGOS-UGANDA/transparent-sacco/issues/3), remaining task
"Bump `lee_core` to v0.3, fix breakages and deploy a hello-world program".

Base branch: `issue-3-sacco-ledger-pr` (PR #18, the v0.2.5 green base). This work is stacked on it so
#18 stays reviewable on its own; the port lands as a second PR against that branch.

## Context

The program builds on `lee_core` v0.2.5-rc2 through the SPEL framework. v0.3.1 of
`logos-execution-zone` replaces that programming model:

| v0.2.5 (SPEL) | v0.3.1 (`lee_core`) |
|---|---|
| `#[lez_program]` module of `#[instruction]` fns | `fn main() { run_program(plan, apply) }` |
| `ProgramContext`, `AccountWithMetadata`, `AccountStateDiff` | `PlanInput`, `AccountMeta`, `ApplyInput`/`ApplyOutput` |
| one handler mutates every account in the instruction | `plan` emits one `effect` per shard; `apply` runs once per shard |
| `nssa_core::account::Data` fixed-size bytes | sharded `AccountData { shards: BTreeMap<AccountId, ShardData> }` |
| `spel` generates the program IDL | no IDL generation |
| `#[account_type]` derive | plain Borsh over shard bytes |

`plan` sees account metadata and instruction arguments but **never account data**. `apply` sees one
shard's pre-data and the effect, but **not** the signer, the sibling accounts, or the instruction.

Nothing is deployed and no accounts exist on any network, so the data layout and the instruction ABI
may change freely.

## Goals

1. All 18 instructions execute on `lee_core` v0.3.1 with the same business rules as today.
2. `RISC0_DEV_MODE=1 cargo test` green in `programs/`, with the existing rule coverage preserved.
3. `artifacts/sacco_ledger-idl.json` regenerated and kept honest by CI.
4. The program deployed and one call executed on a local sequencer with `logos-scaffold`.

## Non-goals

- No custody/token movement. The attestation model is unchanged; nothing moved value before, and
  nothing does now.
- No new domain features, no private accounts, no UI work (that is #5).
- No testnet deploy (that is #4).
- The dormant `custody` module is deleted rather than ported — it is unreachable from the active
  instruction set and is preserved in git history.

## Execution model

```
plan(&PlanInput, Instruction) -> Plan           // asserts shape, ids, signers; emits effects
apply(Effect, pre_shard_data) -> Option<bytes>  // asserts data rules; returns new shard
```

- Signer checks live in `plan`, which is the only place that can see `AccountMeta::is_authorized`.
- Data-dependent authority lives in the shard transition that can see the data: the SACCO shard
  checks approver membership and verification; the member shard checks SACCO ownership.
- `plan.effect(account, &effect)` names the account's shard for this program, so an account whose
  `program_account_id` is another program fails before execution.
- Creation effects use `write_once(pre, data)` so a second initialisation of the same account panics.

## Code layout

```
programs/sacco_ledger/
  core/src/lib.rs            domain structs, Instruction, Borsh encode/decode over shard bytes
  core/src/effect.rs         Effect enums (one variant group per shard kind)
  core/src/transition/       pure per-shard transitions (one file per account type)
  src/plan.rs                Instruction -> Plan
  src/apply.rs               Effect -> shard transition
  src/lib.rs                 re-exports, is_approver
  methods/guest/...          fn main() { run_program(plan, apply) }
  idl/                       IDL generator crate (host only)
  tests/                     harness + rule tests
```

## Effects

```rust
pub enum Effect {
    Config(ConfigEffect),   // Init
    Sacco(SaccoEffect),
    Member(MemberEffect),
    Deposit(DepositEffect),
    Loan(LoanEffect),
    Expense(ExpenseEffect),
    Summary(SummaryEffect),
}
```

Every variant carries only what its own transition needs. The full set, by shard:

**SaccoEffect**

| Variant | Transition |
|---|---|
| `Register { name, registration_number, treasury, admin }` | `write_once` new `Sacco`: admin = signer, zero totals, `required_approvals = 1`, `is_verified = false`, `is_active = true` |
| `Verify { registrar }` | assert `is_active`, registrar equals `admin` (v1 rule, unchanged); set `is_verified` |
| `AddApprover { admin, approver }` | assert `sacco.admin == admin`; push to approvers, reject duplicates |
| `AddSupervisor { admin, supervisor }` | same for `supervisory_committee` |
| `AddMember { admin, identity }` | assert `sacco.admin == admin`; push to `member_ids`, reject duplicates |
| `RecordDeposit { recorder, amount, deposit_type }` | assert verified, recorder is approver, amount > 0; add to the matching total |
| `RequireVerified` | inspect-only (`None`): assert `is_verified` (the SACCO shard stays unchanged for instructions that only read it) |
| `AuthorizeApprover { approver }` | inspect-only (`None`): assert verified and approver membership |
| `DisburseLoan { approver, principal }` | assert verified + approver; add to `total_loans_disbursed` |
| `RecordRepayment { approver, amount }` | assert verified + approver; add to `total_loans_repaid` |
| `ProposeExpense { proposer, amount }` | assert verified + approver + amount > 0 |
| `ExecuteExpense { approver, amount }` | assert verified + approver; add to `total_expenses` |
| `HoldTotals(Totals)` | inspect-only (`None`): assert the five live totals equal the snapshot |

**MemberEffect**: `Init { sacco, identity }`, `RecordDeposit { sacco, amount, deposit_type }`,
`AttachLoan { sacco, loan, signer }` (asserts `signer == member.identity`) — each also asserts
`member.sacco_id == sacco`. Disbursement and repayment do not touch the member account, exactly as
today.

**DepositEffect**: `Init(Deposit)`, `Confirm { member }` (asserts the confirming signer is the
member identity, and the deposit is not already confirmed).

**LoanEffect**: `Init { sacco, member, principal, purpose }`, `Approve { approver }`,
`Reject { approver }`, `Disburse { evidence_cid }`, `Repay { amount, reference }` — status
transitions and the repayment cap (`repaid_amount + amount <= principal`) live here, since the loan
shard owns them.

**ExpenseEffect**: `Init { sacco, amount, purpose, evidence_cid, requested_by }`,
`Approve { approver }`, `Reject { approver }`, `Execute { executed_evidence }`.

**SummaryEffect**: `Init { sacco, label, totals }`.

`AuthorizeApprover` exists because approver membership lives on the SACCO shard while the gated
change lives on the loan/expense shard. A separate inspect effect keeps the rule where its data is.

## Per-instruction mapping

| Instruction | Effects (in order) |
|---|---|
| `Initialize` | Config(Init) |
| `RegisterSacco` | Sacco(Register) |
| `VerifySacco` | Sacco(Verify) |
| `AddApprover` | Sacco(AddApprover) |
| `AddSupervisor` | Sacco(AddSupervisor) |
| `AddMember` | Sacco(AddMember), Member(Init) |
| `RecordDeposit` | Sacco(RecordDeposit), Member(RecordDeposit), Deposit(Init) |
| `ConfirmDeposit` | Deposit(Confirm) |
| `ApplyLoan` | Sacco(RequireVerified), Member(AttachLoan), Loan(Init) |
| `ApproveLoan` | Sacco(AuthorizeApprover), Loan(Approve) |
| `RejectLoan` | Sacco(AuthorizeApprover), Loan(Reject) |
| `DisburseLoan` | Sacco(DisburseLoan), Loan(Disburse) |
| `RecordRepayment` | Sacco(RecordRepayment), Loan(Repay) |
| `ProposeExpense` | Sacco(ProposeExpense), Expense(Init) |
| `ApproveExpense` | Sacco(AuthorizeApprover), Expense(Approve) |
| `RejectExpense` | Sacco(AuthorizeApprover), Expense(Reject) |
| `ExecuteExpense` | Sacco(ExecuteExpense), Expense(Execute) |
| `PostPeriodSummary` | Sacco(HoldTotals), Summary(Init) |

Account order and the signer/writable flags are the same as the current IDL, with one exception
below.

## The `PostPeriodSummary` ABI change

Today the handler copies the SACCO's live totals into the new summary account. In v0.3 neither side
can read the other's shard, so the totals must arrive as an argument. The client reads the public
SACCO account and passes:

```rust
PostPeriodSummary { label: String, totals: Totals }   // Totals: 5 x u128
```

`plan` emits `Sacco(HoldTotals(totals))` (read-only; panics unless the live totals match) and
`Summary(Init { .. })` writing the snapshot. This mirrors upstream's `InboxHolds` guard in the fee
program: the writer writes, the owner-shard validates. The IDL entry gains the `totals` argument;
`sacco_core` (#5) will need the same argument on its method in a later slice.

## IDL

New host-only crate `programs/sacco_ledger/idl`:

- `INSTRUCTIONS: &[InstructionSpec]` — name, accounts `(name, writable, signer, init)`, args
  `(name, type)`.
- A coverage guard: an exhaustive `match` over `Instruction` returning the variant name, with a test
  asserting the table covers every variant. Adding an instruction without its IDL entry fails to
  compile or fails the test.
- `cargo run -p sacco_ledger_idl -- --out artifacts/sacco_ledger-idl.json` writes the file;
  `--check` regenerates in memory, compares bytes, and exits non-zero naming the stale instructions.
- JSON schema unchanged (`version`, `name`, `description`, `instructions[]`) so the `sacco_core`
  contract parser keeps working.

## Tests

`tests/support/mod.rs` provides the harness: an in-memory `AccountId -> ShardData` map, `execute(
program_id, accounts, instruction)` that runs `plan`, asserts each effect names a planned account
with the matching selector, then applies each effect and stores post data. `#[should_panic]` still
works, so rule violations are asserted by message.

Coverage:

- The 52 existing tests, rewritten onto the harness, same names and same assertions where the rule
  is unchanged.
- Plan-level: unsigned signer panics; account with a foreign `program_account_id` panics;
  misordered or missing accounts panic.
- Summary: mismatched totals panic, matching totals write the snapshot.
- Pure-transition unit tests in `core` for arithmetic-only rules (repayment cap, threshold,
  deposit-type split) so those do not need the harness.
- IDL: table covers the enum; committed artifact is current.

## Toolchain, dependencies, CI

- `programs/rust-toolchain.toml` → `1.98.1` (upstream v0.3.1's channel). Crates stay edition 2021.
- `lee_core` → git tag `v0.3.1`, `features = ["host"]`, still aliased as `nssa_core` where the code
  already does. `borsh` → `1.5.7`. Drop `spel-framework`, `spel-framework-macros`, `token_core`.
- `risc0-zkvm` / `risc0-build` stay `=3.0.5` (upstream uses the same).
- `.github/workflows/lez-program.yml`: toolchain 1.98.1, plus an IDL `--check` step.

## Deploy

After tests are green: `logos-scaffold init`, `logos-scaffold setup`, `localnet start`, build the
guest, `deploy sacco_ledger --json`, then an `initialize` call. The program id is recorded in
`programs/sacco_ledger/README.md` (`docs/deploy.md` belongs to PR #14 and is left alone). If the
local network cannot start on this machine, the task stays open and the failure is reported with the
exact step that failed.

## Risks

- **Local sequencer provisioning**: `logos-scaffold setup` fetches/builds sequencer binaries. If it
  fails offline or out of disk, the deploy task cannot be ticked here.
- **Effect size**: effects carry `String` fields (purpose, reference, evidence CID). `ShardData` has
  a hard size cap; the port keeps argument sizes as they are today and the existing caps are not
  expected to be approached.
- **Stacked branch**: if #18 changes under review, this branch rebases onto it before the port PR is
  opened.
- **`HoldTotals` trust**: the summary is only as honest as the SACCO shard check, which is exactly
  the guarantee the old handler gave (it read the same live data).

## Acceptance

- `cd programs && RISC0_DEV_MODE=1 cargo test` green, including the IDL check.
- Every instruction in the issue's checklist has a passing test through `plan` + `apply`.
- `artifacts/sacco_ledger-idl.json` regenerated by the generator and current.
- Local deploy performed and its program id recorded, or the blocking step reported.
