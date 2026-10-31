# Issue #5: Basecamp SACCO Ledger Flow

**Status:** Design approved in chat on 2026-10-26; awaiting spec review.

## Intent

Complete issue #5 with one local, end-to-end SACCO flow inside Logos Basecamp. A user records an off-chain deposit attestation, the member sees and confirms it, an expense passes a multi-signature threshold, and the auditor view reads the resulting public ledger history.

The SACCO ledger records attestations; it does not hold or transfer member funds. The demo uses test accounts and a local sequencer. It does not submit transactions to a public testnet or move real value.

## Current State and Dependencies

- PR #20 contains the role views and a `sacco_core` API surface, but `dispatchInstruction()` returns a queued placeholder and the query methods return unavailable placeholders.
- PR #20 is based on `main`, which has no `programs/sacco_ledger` tree or `artifacts/sacco_ledger-idl.json`.
- PR #17 contains the first API-contract document for issue #5. It references the ledger IDL and deployment documentation that are not present on its base branch.
- PR #18 adds the `sacco_ledger` program on `lee_core` v0.2.5-rc2. PR #21 ports that program to v0.3 and is based on PR #18. PR #21 remains open with a changes-requested review.
- PR #21 currently marks the `Initialize` config account as a signer even though it derives that account as a PDA. A PDA has no wallet signing key; initialization must validate the exact derived PDA without requiring its signature.
- The baseline on `main` fails before tests start: `programs/fee_transparency/methods/Cargo.toml` has no target. PR #18 replaces that scaffold with the ledger guest library and guest binary.
- The local sequencer is already healthy on port 3040. Basecamp has not been set up. No local Bedrock endpoint is listening on port 8080. The in-process `lez_indexer_module` exposes queries through Logos calls; its standalone port 8779 is not required.
- The current program and API contract depend on a deployed program ID recorded in `docs/deploy.md`, which does not yet exist on `main`.

**Dependency order:** PR #18 must land before PR #21; both ledger changes must land before the issue #5 integration is built and verified against `main`. This work will not merge either dependency PR. Update the existing issue #5 branch after those dependencies are merged, rather than copying the program/IDL into a second branch.

## Goals

1. Submit real `sacco_ledger` instructions through the Logos LEZ wallet module, with the wallet signing only for the selected account identities.
2. Read current account state and transaction history from the LEZ indexer, so each view displays chain data rather than local placeholder data.
3. Keep instruction encoding, account ordering, signer flags, and account decoding tied to the Rust ledger types and generated IDL.
4. Fix the member view’s missing account selection, the mismatch between query response shapes and QML models, and invalid JSON construction from unvalidated inputs.
5. Demonstrate the deposit/member-confirmation/multi-signature-expense flow in Basecamp on a local test network.
6. Make program initialization possible by requiring the exact config PDA while treating it as a non-signer.

## Non-goals

- Public testnet deployment, repository publication, or production credentials.
- Token custody or transfer. Deposits and expenses remain off-chain actions represented by ledger attestations.
- A generic wallet or indexer framework beyond what the SACCO flow needs.
- Closing or merging PRs #17, #18, #20, or #21 as part of this design.

## Selected Architecture

### Module calls

Keep `sacco_core` as the module-facing API used by `sacco_ui`.

- Add the official `lez_core` module as a typed dependency. Use its `send_generic_public_transaction` API for signed ledger writes and its account APIs for wallet account discovery. Do not shell out to `spel` from a Basecamp module.
- Add the official `lez_indexer_module` as a typed dependency for finalized account state and transaction history. Start it with an explicit local indexer config and show a clear unavailable/syncing state in the UI until it is ready.
- Pin both module inputs in the Nix flakes and declare both dependencies in module metadata so standalone and Basecamp builds resolve the same runtime modules.

### IDL and binary data

Add a small native Rust client crate under `programs/sacco_ledger/` that reuses `sacco_ledger_core` and `sacco_ledger_idl`. Expose a narrow C ABI for the C++ module to:

- Encode an instruction from validated JSON into the exact Borsh enum bytes.
- Return the ordered account names and signer/writable flags from the Rust IDL table, validating that every required account was supplied once.
- Decode a fetched account shard into the JSON object expected by the views.
- Decode indexed `instruction_data` for SACCO ledger transactions so the auditor can label and group history entries.

Build and link this client crate as a native static library in the `sacco_core` Nix build. The C++ side will not duplicate enum indexes, Borsh layouts, or account metadata. Rust tests will pin encoded bytes and account order against `sacco_ledger_idl`; C++ tests will cover the FFI boundary and returned error values.

### Signer and account handling

- Populate signer choices from the connected LEZ wallet’s account list. The UI passes a selected account ID; it never receives or stores private keys.
- `sacco_core` builds account metas and signer requirements from the IDL, not from UI-provided boolean flags.
- Use the `sacco_ledger` client helpers for config-PDA calculation and account-ID conversion. Generate new public account IDs through `lez_core` for the SACCO, member, deposit, loan, expense, and summary records that need initialized accounts.
- `Initialize` must reject a config account whose ID is not `compute_config_pda(program_id)` and must not require a signature from the PDA.
- Persist only non-secret demo conveniences (program ID, active SACCO ID, created record IDs) under the module instance persistence path. The indexer remains the source of truth for chain state and audit history.
- Return a consistent JSON envelope from module methods: success/error status, transaction hash when applicable, decoded data, and a user-readable error. Do not report a queued transaction as successful before the wallet and sequencer accept it.

### Indexed queries and audit trail

- Use `lez_indexer_module.getAccount()` to fetch current public account state.
- Page through `getTransactionsByAccount(sacco_id, offset, limit)` to discover ledger transactions touching the SACCO account. Decode the transaction’s ordered account IDs and instruction bytes through the Rust client crate, then fetch the affected deposit, loan, expense, and summary accounts.
- Deposit confirmation does not touch the SACCO account. For each discovered record account, query its transaction history too, then merge transactions by hash. This includes member confirmations and avoids implying those transactions appear in SACCO history.
- Show the transaction hash, instruction kind, account IDs, ledger status, and evidence CID in the auditor view. Render missing/not-finalized data as a loading or error state, never as an empty successful result.

### QML behavior

- `MemberView` receives a selected wallet identity from `Main.qml`; it must not default to a permanently empty identity. Provide account selection and refresh behavior.
- The member view shows the SACCO totals, that member’s balances, deposits, and loans from decoded chain data. Deposit confirmation submits the member identity as the required signer.
- The committee view validates required fields before enabling actions and chooses the current signer identity from the wallet account list.
- The approval demo uses three distinct SACCO approvers and threshold 2: the proposer cannot approve their own expense, and the other two approvers each sign once before execution.
- The auditor view uses the indexed transaction history and links evidence CIDs as displayable text/URLs where supported.
- Keep the existing Member / Committee / Auditor tabs, but put the longer views in scrollable layouts so all controls remain reachable at the module’s target window size.

## Data Flow for the Demo

1. Start the local sequencer, compatible local Bedrock/indexer services, and Basecamp.
2. Deploy the `sacco_ledger` guest to the local sequencer; record its program ID and local connection details in `docs/deploy.md`.
3. Create/select test wallet accounts for one admin/proposer, two additional approvers, and one member identity.
4. Initialize the program, register and verify a SACCO, add the two approvers and member account, then set the required approval threshold to 2.
5. Record a savings deposit for the member. The member view refreshes from the indexer and the member identity confirms the deposit.
6. The proposer creates an expense with an evidence CID. The other two approvers approve using distinct accounts. An approver executes the approved expense with execution evidence.
7. The auditor view refreshes from indexed account/transaction data and shows the deposit, confirmation, expense approvals, execution, and evidence references.

## Verification

- Rust client tests compare Borsh bytes, enum indexes, account order, signer flags, and account decoding with the Rust IDL/core types.
- C++ module tests cover validation, wallet/indexer failures, successful transaction responses, and JSON output shapes.
- QML integration tests cover account selection, data rendering, disabled invalid actions, member confirmation, multi-signer expense approval, and auditor history.
- Local integration test deploys the guest, initializes a fresh SACCO, executes the demo sequence, and verifies final indexed state and transaction history.
- Build and install both `.lgx` modules through the project’s Basecamp workflow, then launch the app and complete the same flow in Basecamp.
- Update `docs/testing.md`, module READMEs, the API contract, and `docs/deploy.md` to distinguish the local end-to-end flow from any future testnet work.

## Operational Constraints

- Basecamp profile setup is not present. Use project-local `.scaffold/basecamp/` state only.
- The indexer module needs a reachable Bedrock endpoint at port 8080 and local indexer configuration. The standalone JSON-RPC listener on port 8779 is not required when queries use the in-process module.
- Do not reuse or expose wallet keys. Use fresh local test accounts and the local wallet home for the demo.
- If a required upstream dependency or service cannot be built or started, report the exact failing command and leave the acceptance criterion visibly incomplete rather than replacing it with a mock response.

## References

- [Logos C++ SDK](https://github.com/logos-co/logos-cpp-sdk) — typed calls between universal modules.
- [LEZ Core module](https://github.com/logos-blockchain/logos-execution-zone-module) — wallet transaction and account APIs.
- [LEZ Indexer module](https://github.com/logos-blockchain/lez-indexer-module) — finalized account and transaction-history APIs.
- [`docs/testing.md`](../../testing.md) and [Basecamp publishing](../../basecamp-publishing.md) — project verification and module install workflows.
