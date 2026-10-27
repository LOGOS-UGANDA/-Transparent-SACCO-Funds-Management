# IDL

The Interface Definition Language (IDL) describes `sacco_ledger` instructions, accounts, and types for wallet / `spel` / `sacco_core` clients.

## Artifact

| File | Meaning |
|------|---------|
| [`artifacts/sacco_ledger-idl.json`](../artifacts/sacco_ledger-idl.json) | Generated JSON artifact from the declarative Rust table in `programs/sacco_ledger/idl/src/lib.rs` |

The Rust table is hand-maintained and tested against the instruction enum and
planner. The JSON is generated from that table. When an instruction, account,
or argument changes, update the Rust table and its consistency tests in the
same change.

## Regenerate (when toolchain is installed)

From the repository root, render the artifact with:

```bash
cd programs
cargo run -p sacco_ledger_idl -- --out ../artifacts/sacco_ledger-idl.json
cargo run -p sacco_ledger_idl -- --check
```

## Client usage (next)

The Basecamp `sacco_core` module uses the Logos wallet APIs and the Rust client
helpers generated from the same ledger types; it does not shell out to `spel`.
