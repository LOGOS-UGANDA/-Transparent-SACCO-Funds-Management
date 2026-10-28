# Phase 2 deployment runbook

Phase 2 starts with local development. The SACCO program records off-chain
deposits, loan activity, and expenses; it does not receive, hold, or transfer
SACCO money. This runbook does not require a public testnet wallet, token
top-up, or fee payment.

## Current status

- `scaffold.toml` pins the project runtime to LEZ `v0.3.1`, SPEL
  `v0.7.0-rc.3`, and circuits `0.5.7`. SPEL is a separate setup tool and has
  its own LEZ dependency; the local Initialize client and program use LEZ
  `v0.3.1`.
- The repository contains a small local client that sends `Initialize` with
  the pinned LEZ v0.3.1 wallet API and confirms the resulting config state.
  `logosctl call` invokes Logos Core modules; it does not call an LEZ guest
  program.
- Program account IDs are deployment outputs and can change when the guest
  binary changes. Record the full ID from each deployment alongside the guest
  commit; do not treat an old ID as canonical.
- Scaffold CLI 0.4.0 and 0.4.1 generate an 8 MiB sequencer block limit, while
  the pinned LEZ accepts at most 1.75 MiB. The checked-in preparation script
  changes the ignored local sequencer config to 1 MiB after setup.

## 1. Build the local tooling and guest

Run from the repository root. `setup` syncs the configured LEZ, SPEL, and
circuits versions and builds the local sequencer, wallet, and `spel` binaries.
If no prebuilt sequencer exists for the pin, it falls back to a source build.

```bash
logos-scaffold setup --prebuilt
logos-scaffold build
python3 scripts/prepare-localnet.py
logos-scaffold localnet start
logos-scaffold localnet status --json
```

Run `prepare-localnet.py` after setup, which may regenerate the sequencer
config. This changes only ignored local state. Local compilation and localnet
transactions do not submit anything to a public network or require a real token
top-up. A submitted call uses the local sequencer's valueless genesis test
units as its fee balance.

## 2. Deploy to the local sequencer

```bash
logos-scaffold deploy --program-path \
  programs/target/riscv-guest/sacco_ledger-methods/sacco_ledger-guest/riscv32im-risc0-zkvm-elf/release/sacco_ledger.bin \
  --json
```

Capture the full `program_account_id` from the JSON output. It is the deployed
program account used by the call below; it can differ between guest builds.

## 3. Submit a ledger instruction

Choose the local fee payer from `logos-scaffold wallet list --long`; setup
seeds local accounts for the sequencer's genesis state. Use the deployed
program account ID and submit the call with the repository's local client:

```bash
PROGRAM_ACCOUNT_ID='<program_account_id from deploy --json>'
LOCAL_FEE_PAYER_ID='<seeded local account ID>'
CARGO_NET_OFFLINE=true cargo +1.98.1 run --release \
  --manifest-path programs/Cargo.toml -p sacco_ledger_local_client -- \
  "$PROGRAM_ACCOUNT_ID" "$LOCAL_FEE_PAYER_ID"
```

The client prints the config PDA and transaction hash, then exits successfully
only after it reads `ProgramConfig.version = 1` from the local sequencer. This
initializes only the program registry; it does not record or transfer SACCO
funds.

## 4. Public testnet

Deferred. Public testnet deployment and calls are not part of this local
workflow or this preparation PR. Do not top up a wallet or submit public
transactions as part of this runbook.

## Related documentation

- [Program instructions, local deployment, and known gaps](../programs/sacco_ledger/README.md)
- [Attestation architecture](./architecture.md)
- [Testing guide](./testing.md)
