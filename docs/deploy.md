# Phase 2 deployment runbook

Phase 2 starts with local development. The SACCO program records off-chain
deposits, loan activity, and expenses; it does not receive, hold, or transfer
SACCO money. This runbook does not require a public testnet wallet, token
top-up, or fee payment.

## Current status

- `logos-scaffold` 0.4.0 manages the project-local LEZ sequencer and guest
  deployment.
- The project is pinned to LEZ `1a003bfbfb89fcf987f62975cd08a401169e1b8c` and
  circuits `0.5.7` in `scaffold.toml`.
- There is no checked-in host runner for submitting `sacco_ledger` IDL
  instructions. `logosctl call` invokes Logos Core modules; it does not call an
  LEZ guest program. Do not use the old `logosctl call sacco_ledger ...`
  examples as deploy commands.
- The prior local deployment recorded in
  [`programs/sacco_ledger/README.md`](../programs/sacco_ledger/README.md) has
  not been re-verified against a freshly built v0.3.1 local sequencer.

## 1. Build the local tooling and guest

Run from the repository root. `setup` syncs the configured LEZ and circuits
versions and builds the local sequencer, wallet, and `spel` binaries. If no
prebuilt sequencer exists for the pin, it falls back to a source build.

```bash
logos-scaffold setup --prebuilt
logos-scaffold build
```

The source build may take a long time on a cold cache. It is local compilation;
it does not submit a public transaction or charge a deployment fee.

## 2. Deploy to the local sequencer

```bash
logos-scaffold localnet start
logos-scaffold localnet status --json
logos-scaffold deploy --program-path \
  programs/target/riscv-guest/sacco_ledger-methods/sacco_ledger-guest/riscv32im-risc0-zkvm-elf/release/sacco_ledger.bin \
  --json
```

The deploy output includes the RISC Zero program ID. Compare it with the ID
recorded in the program README and update that value only after confirming a
successful deployment of the current guest binary.

## 3. Submit a ledger instruction

This step is not available yet. The repository has an IDL, but no host-side
runner that builds, signs, and submits a `PublicTransaction` to the local
sequencer. Add and verify that runner before claiming the Phase 2 working-call
milestone complete. The call should record a SACCO event and its approval; it
must not transfer or custody SACCO funds.

## 4. Public testnet

Deferred. Public testnet deployment and calls are not part of the current local
workflow or this preparation PR. Issue #4 remains open until its remaining
deployment and working-call criteria are completed. Do not top up a wallet or
submit testnet transactions as part of this runbook.

## Related documentation

- [Program instructions, local deployment, and known gaps](../programs/sacco_ledger/README.md)
- [Attestation architecture](./architecture.md)
- [Testing guide](./testing.md)
