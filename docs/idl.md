# IDL

The Interface Definition Language (IDL) describes `sacco_ledger` instructions, accounts, and types for wallet / `spel` / `sacco_core` clients.

## Artifact

| File | Meaning |
|------|---------|
| [`artifacts/sacco_ledger-idl.json`](../artifacts/sacco_ledger-idl.json) | Current IDL (hand-authored to match the SPEL guest) |

Treat this file as the client contract. When guest account lists or args change, update the IDL in the same change.

## Regenerate (when toolchain is installed)

From [lez-programs](https://github.com/logos-blockchain/lez-programs) style tooling:

```bash
# Prefer the repo's guest source as the source of truth
spel generate-idl programs/sacco_ledger/methods/guest/src/bin/sacco_ledger.rs \
  > artifacts/sacco_ledger-idl.json
```

Or, if using `idl-gen` from a Logos programs workspace:

```bash
cargo run -p idl-gen -- programs/sacco_ledger/methods/guest/src/bin/sacco_ledger.rs \
  > artifacts/sacco_ledger-idl.json
```

Diff the generated file against the checked-in artifact and commit both guest + IDL together.

## Client usage (next)

`modules/sacco_core` will call instructions via wallet/`spel --idl artifacts/sacco_ledger-idl.json ...` once custody flows are exercised on testnet. Until then, core methods return scaffold JSON only.
