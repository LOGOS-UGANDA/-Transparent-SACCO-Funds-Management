# fee_transparency (LEZ program)

LEZ program for Transparent SACCO fee locking, category allocation, multi-sig release, and **token custody**.

Product model from `prototype/contracts/FeeTransparency.sol`, adapted to LEZ (Rust / RISC Zero / SPEL). Custody design: [`docs/custody.md`](../../docs/custody.md).

## Layout

```text
fee_transparency/
├── core/                 # Instruction enum, account types, PDA helpers
├── src/                  # Pure state-transition handlers (+ chained Token calls)
└── methods/
    ├── build.rs
    └── guest/            # #[lez_program] guest binary
```

## Instructions

| Instruction | Purpose |
|-------------|---------|
| `Initialize` | Config PDA + Token Program id |
| `RegisterInstitution` | Create institution registry account |
| `VerifyInstitution` | Mark institution verified |
| `InitializeInstitutionVault` | Bind fee token + vault PDA |
| `AddApprover` | Register a multi-sig approver |
| `CreateFeeCategory` | Category with basis-point allocation |
| `PayFees` | Ledger + transfer payer holding → vault |
| `RequestRelease` | Propose release with purpose/evidence |
| `ApproveRelease` / `RejectRelease` | Multi-sig votes |
| `ExecuteRelease` | Ledger + transfer vault → treasury holding |

Prefer **public** LEZ accounts for payer-visible audit trails.

## Custody (v1)

- One **institution vault PDA** per `(institution, token_definition)`
- Categories keep a **ledger** (collected / released)
- `PayFees` / `ExecuteRelease` issue `Token::Transfer` chained calls
- Guest `PayFees` accepts up to **8** category account slots

## IDL

Checked-in client contract: [`artifacts/fee_transparency-idl.json`](../../artifacts/fee_transparency-idl.json). Regeneration notes: [`docs/idl.md`](../../docs/idl.md).

## Build / test

Requires the Logos LEZ + SPEL toolchains (see [lez-programs](https://github.com/logos-blockchain/lez-programs)).

```bash
cd programs
RISC0_DEV_MODE=1 cargo test -p fee_transparency_program

cargo risczero build --manifest-path fee_transparency/methods/guest/Cargo.toml
```

Deploy later with:

```bash
wallet deploy-program target/guest/fee_transparency.bin
```

## Known gaps

1. `CreateFeeCategory` guest does not yet load existing categories for the 100% BPS cap (host API accepts `existing_categories`; wire in guest next).
2. Refund path not implemented.
3. `sacco_core` not yet submitting IDL txs (scaffold JSON only).
4. Regenerate IDL with `spel`/`idl-gen` once the toolchain is installed and replace the hand-authored artifact if it drifts.
