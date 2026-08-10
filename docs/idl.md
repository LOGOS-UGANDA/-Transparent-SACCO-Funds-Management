# IDL

The Interface Definition Language (IDL) describes `fee_transparency` instructions, accounts, and types for wallet / `spel` / `sacco_core` clients.

## Artifact

| File | Meaning |
|------|---------|
| [`artifacts/fee_transparency-idl.json`](../artifacts/fee_transparency-idl.json) | Current IDL (hand-authored to match the SPEL guest) |

Treat this file as the client contract. When guest account lists or args change, update the IDL in the same change.

## Regenerate (when toolchain is installed)

From [lez-programs](https://github.com/logos-blockchain/lez-programs) style tooling:

```bash
# Prefer the repo's guest source as the source of truth
spel generate-idl programs/fee_transparency/methods/guest/src/bin/fee_transparency.rs \
  > artifacts/fee_transparency-idl.json
```

Or, if using `idl-gen` from a Logos programs workspace:

```bash
cargo run -p idl-gen -- programs/fee_transparency/methods/guest/src/bin/fee_transparency.rs \
  > artifacts/fee_transparency-idl.json
```

Diff the generated file against the checked-in artifact and commit both guest + IDL together.

## Client usage (next)

`modules/sacco_core` will call instructions via wallet/`spel --idl artifacts/fee_transparency-idl.json ...` once custody flows are exercised on testnet. Until then, core methods return scaffold JSON only.
