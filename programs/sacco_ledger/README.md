# sacco_ledger (LEZ program)

Transparent SACCO attestation ledger on the Logos Execution Zone. Deposits,
loans and expenses are recorded and approved on chain; **value stays off chain**,
so the program holds no tokens and issues no transfers.

Runs on `lee_core` v0.3.1 using the sharded `plan` / `apply` ABI: `plan` maps an
instruction to one typed effect per mutated shard, and `apply` is the pure
per-shard transition. No effect can read another shard, so every value that
crosses a shard boundary is passed as an argument and validated by the shard
that owns the data.

## Layout

```text
sacco_ledger/
├── core/           # Instruction, Effect, record types, PDA helpers
├── src/            # plan() + apply() shims and per-shard transitions
├── idl/            # IDL table, generator and drift check
└── methods/        # risc0 guest binary
```

## Instructions

| Instruction | Purpose |
|-------------|---------|
| `Initialize` | Program config shard |
| `RegisterSacco` | Register a SACCO with its treasury |
| `VerifySacco` | Admin self-verifies the SACCO (v1) |
| `AddApprover` / `AddSupervisor` | Governance roles |
| `SetRequiredApprovals` | Admin sets a threshold from 2 through one less than the approver count |
| `AddMember` | Member account shard |
| `RecordDeposit` / `ConfirmDeposit` | Savings or share-capital deposit, then member confirmation |
| `ApplyLoan` / `ApproveLoan` / `RejectLoan` | Loan application with pinned approval threshold |
| `DisburseLoan` | Marks the loan disbursed, carrying the principal for cross-checking |
| `RecordRepayment` | Repayment against a loan |
| `ProposeExpense` / `ApproveExpense` / `RejectExpense` | Expense proposal with pinned approval threshold |
| `ExecuteExpense` | Marks the expense executed, carrying the amount for cross-checking |
| `PostPeriodSummary` | Writes a period summary from caller-supplied totals |

Approval thresholds are pinned into `Loan` / `Expense` when they are created and
validated against the live SACCO config at that moment, because the approval
transition runs in a different shard from the config.

New SACCOs start with a threshold of one for setup. Verification is blocked
until the admin adds at least two more approvers and sets the threshold to at
least two. The upper bound leaves enough approvers to approve an expense after
excluding its proposer. Any non-proposer approver rejection is an immediate
veto. Period summary labels must use the canonical
`YYYY-MM` calendar-month format, and a month cannot be posted twice.
To keep Borsh shard sizes bounded in this release, each SACCO supports at most
128 members, 32 approvers and 32 supervisors, and each member can reference at
most 128 loans. Each SACCO can retain 128 unique period summaries. Approval and
rejection lists are bounded by the 32-approver limit. Larger deployments need
registry shards before these limits are raised.

## IDL

The JSON artifact is generated from the hand-maintained instruction table in
`idl/src/lib.rs`; it is not derived automatically from the Rust enum. CI fails
when the committed artifact is stale, and tests compare the table with the enum
and planner.

```bash
cd programs
cargo run -p sacco_ledger_idl -- --check
```

## Build / test

```bash
cd programs
RISC0_DEV_MODE=1 cargo test -p sacco_ledger_program
cargo test -p sacco_ledger_core -p sacco_ledger_idl
```

Guest ELF without Docker (the `risc0` rustup toolchain carries the target):

```bash
cd programs/sacco_ledger/methods/guest
cargo +risc0 build --release --target riscv32im-risc0-zkvm-elf
```

## Local deploy

The following are the local deployment commands for `logos-scaffold` 0.4.0.
They have not been rerun against a sequencer built from the current pins. The
valid RISC Zero `ProgramBinary` is produced by building the methods crate:

```bash
cargo build --release --manifest-path programs/sacco_ledger/methods/Cargo.toml
logos-scaffold localnet start
logos-scaffold deploy --program-path \
  programs/target/riscv-guest/sacco_ledger-methods/sacco_ledger-guest/riscv32im-risc0-zkvm-elf/release/sacco_ledger.bin --json
```

Previously recorded local sequencer program ID:
`e45a5879831de87447a44046150d37d2036db0c44fc3cc519c6da7536a1422a7`. It was
extracted with `logos-scaffold spel -- program-id <ProgramBinary>` and included
in a local deployment block. Verify it again after rebuilding the local
sequencer and guest. Public testnet deployment remains deferred in issue #4;
it is not needed for local development.

## Known gaps

1. The ignored `.scaffold` cache was built with the previous LEZ and circuits
   pins. Run `logos-scaffold setup` to rebuild from the v0.3.1 / circuits 0.5.7
   pins above, then rerun the local `initialize` call before treating it as
   verified.
2. `sacco_core` does not submit IDL transactions yet.
3. Refunds are handled off chain; the current ledger has no instruction to record a refund yet.
