# fee_transparency (LEZ program)

LEZ program scaffold for Transparent SACCO fee locking, category allocation, and multi-sig release.

Mirrors the product model in `prototype/contracts/FeeTransparency.sol`, adapted to LEZ account-oriented design (Rust / RISC Zero / SPEL).

## Layout

```text
fee_transparency/
├── core/                 # Instruction enum + Borsh account types
├── src/                  # Pure state-transition handlers
└── methods/
    ├── build.rs          # risc0 embed_methods
    └── guest/            # #[lez_program] guest binary
```

## Instructions

| Instruction | Purpose |
|-------------|---------|
| `RegisterInstitution` | Create institution registry account |
| `VerifyInstitution` | Mark institution verified |
| `AddApprover` | Register a multi-sig approver |
| `CreateFeeCategory` | Category with basis-point allocation |
| `PayFees` | Lock payment + allocate to categories |
| `RequestRelease` | Propose release with purpose/evidence |
| `ApproveRelease` / `RejectRelease` | Multi-sig votes |
| `ExecuteRelease` | Finalize approved release |

Prefer **public** LEZ accounts for payer-visible audit trails.

## Build / test

Requires the Logos LEZ + SPEL toolchains (see [lez-programs](https://github.com/logos-blockchain/lez-programs)).

```bash
cd programs
# Unit-test host-side handlers (skips guest ZK build when deps resolve)
RISC0_DEV_MODE=1 cargo test -p fee_transparency_program

# Guest binary (Docker / risczero toolchain)
cargo risczero build --manifest-path fee_transparency/methods/guest/Cargo.toml
```

Deploy later with:

```bash
wallet deploy-program target/guest/fee_transparency.bin
```

## Scaffold gaps (next)

1. Wire `AddApprover` handler + category list into `PayFees` / `CreateFeeCategory` guest txs.
2. Move value with token-program chained calls (not just ledger fields).
3. Generate IDL (`spel generate-idl` or `idl-gen`) into `artifacts/`.
4. Integration tests through the zkVM.
