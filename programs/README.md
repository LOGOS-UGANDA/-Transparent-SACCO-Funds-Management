# Programs (LEZ)

Rust workspace for Logos Execution Zone programs.

The current `sacco_ledger` program records and approves SACCO activity that
happens off chain. It does not custody or transfer savings, loan principal, or
expense funds. The former fee-custody design is historical only; see
[`docs/custody.md`](../docs/custody.md).

```bash
cd programs
RISC0_DEV_MODE=1 cargo test -p sacco_ledger_program
```

- Program details: [sacco_ledger/README.md](./sacco_ledger/README.md)
- Attestation model: [architecture](../docs/architecture.md)
- IDL: [docs/idl.md](../docs/idl.md)
- Architecture: [docs/architecture.md](../docs/architecture.md)
