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

## IDL

Generated, not hand-maintained: `artifacts/sacco_ledger-idl.json` is rendered
from the single instruction table in `idl/src/lib.rs`, and CI fails when the
committed artifact is stale.

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

Built and deployed against a `logos-scaffold` 0.4.0 local sequencer on
`lee_core` v0.3.1:

```bash
logos-scaffold localnet start
logos-scaffold deploy --program-path \
  programs/sacco_ledger/methods/guest/target/riscv32im-risc0-zkvm-elf/release/sacco_ledger --json
```

The deploy prints the risc0 image id for the guest ELF; record it here when it
changes. Testnet deployment stays with issue #4 and
[`docs/deploy.md`](../../docs/deploy.md).

## Known gaps

1. `logos-scaffold` 0.4.0 exposes no transaction subcommand, so the `initialize`
   call is deferred to the wallet/Basecamp path in `docs/deploy.md`.
2. `sacco_core` does not submit IDL transactions yet.
3. No refund path.
