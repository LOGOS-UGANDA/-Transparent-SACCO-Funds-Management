# Issue #5 Basecamp SACCO Ledger Flow Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete issue #5 with a working local Basecamp flow that writes SACCO ledger instructions, reads finalized indexer state, supports member confirmation and a two-of-three expense approval, and presents the public audit trail.

**Architecture:** Extend the existing `sacco_core` universal module with typed `lez_core` and `lez_indexer_module` dependencies, and bind it to the ledger’s Rust core and IDL through a narrow native C ABI client. Keep account selection and workflow controls in QML, with structured module responses and indexer-backed state. First repair the upstream config PDA initialization contract; build against the merged PR #18 and PR #21 implementation rather than duplicating the ledger.

**Tech Stack:** Rust (`sacco_ledger_core`, `sacco_ledger_idl`, Borsh, C ABI), C++ Logos module SDK, Qt/QML, Nix flakes, Basecamp, local LEZ sequencer and Bedrock indexer.

**Spec:** `docs/superpowers/specs/2026-10-26-issue-5-basecamp-ledger-flow-design.md`

## Global Constraints

- PR #18 must land before PR #21; both ledger changes must land before integration is built and verified against `main`.
- Do not merge dependency PRs or close PRs #17, #18, #20, or #21 as part of this work.
- The ledger records attestations; it does not hold or transfer member funds.
- Use `lez_core.send_generic_public_transaction` and typed Logos calls; do not shell out to `spel`.
- The `Initialize` config account is a non-signer and must equal `compute_config_pda(program_id)`.
- The indexer module requires a reachable Bedrock endpoint at port 8080; its standalone JSON-RPC port 8779 is not required.
- Keep private keys out of UI and module persistence; persist only non-secret demo IDs and connection data.
- Do not report queued or pending transactions as successful; expose the actual wallet/sequencer result and finalized indexer state distinctly.
- Use only fresh local test accounts and local services for the demonstration.

## Review Focus

- Malformed JSON, control characters, numeric overflow, and invalid C ABI input must return a structured error without invalid JSON or process failure.
- Missing, duplicate, extra, or incorrectly ordered IDL account mappings must be rejected before wallet submission.
- A wrong config PDA or attempted PDA signature must fail safely; the exact config PDA must initialize without a signer requirement.
- Wallet rejection, sequencer failure, unavailable Bedrock/indexer, and indexer sync lag must remain distinct user-visible states.
- Pagination overlap, duplicate transaction hashes, and confirmations that only touch a member record must still produce one complete auditor history.

---

## Dependency gate

- [ ] Confirm PR #18 is merged into `main`, then confirm PR #21 is merged on top and includes the PDA signer correction from Task 1. Do not start integration tasks until both are present.
- [ ] Update the isolated issue #5 branch from the new `main` state without merging PRs on the user’s behalf; preserve the existing issue #20 work as the issue #5 implementation base.
- [ ] Re-run the repository’s documented baseline build and record the exact command/result before changing integration code.

## File map

| Area | Files | Responsibility |
|---|---|---|
| Ledger contract | `programs/sacco_ledger/src/plan.rs`, `programs/sacco_ledger/idl/src/lib.rs`, associated Rust tests and generated IDL artifact | Validate config PDA, signer metadata, and generated artifact consistency. These files arrive through PR #21. |
| Native client | `programs/sacco_ledger/client/` (crate manifest, `src/lib.rs`, tests; final path adjusted to repository conventions after dependency merge) | Convert validated JSON to Borsh instruction bytes, expose IDL account metadata, derive config PDA, decode account shards and indexed instructions. |
| Core module | `modules/sacco_core/src/sacco_core_impl.{h,cpp}`, `modules/sacco_core/CMakeLists.txt`, `modules/sacco_core/flake.nix`, `modules/sacco_core/metadata.json` | Typed wallet/indexer calls, validation, transaction submission, persistence, structured JSON APIs. |
| UI module | `modules/sacco_ui/Main.qml`, role view QML files (discover exact current names after dependencies land), `modules/sacco_ui/flake.nix`, `modules/sacco_ui/metadata.json` | Account selection, form validation, data models, role workflows, audit display and scrolling. |
| Verification/docs | Existing Rust/C++/QML test directories, `docs/testing.md`, `docs/deploy.md`, `modules/sacco_core/README.md`, `modules/sacco_ui/README.md`, issue #5 API contract doc from PR #17 | Automated and local integration verification; document local setup and API behavior. |

## Task 1: Repair config PDA initialization contract

**Files:**
- Modify: `programs/sacco_ledger/src/plan.rs`
- Modify: `programs/sacco_ledger/idl/src/lib.rs`
- Test: associated `sacco_ledger` planner and IDL tests
- Regenerate: `artifacts/sacco_ledger-idl.json` using the repository’s established IDL generation command

**Interfaces:** `compute_config_pda(program_id)` supplies the only valid config account ID. `Initialize` takes that config account as writable/non-signer according to the existing IDL representation.

- [ ] Write tests proving the correct config PDA initializes without authorization/signature and a wrong account ID is rejected.
- [ ] Add an IDL assertion that `initialize.config.signer == false` and account order is unchanged.
- [ ] Run the focused tests and confirm they fail for the existing signer/PDA behavior.
- [ ] Remove the impossible signer assertion, validate the supplied config ID against `compute_config_pda(input.self_account_id)`, and update signer metadata.
- [ ] Regenerate the IDL artifact and run focused Rust tests plus the IDL drift check.
- [ ] Commit the minimal contract correction on the issue #5 integration branch after PRs #18 and #21 have landed; do not rewrite either open dependency branch.

## Task 2: Build the Rust client ABI and pin binary contracts

**Files:**
- Create: `programs/sacco_ledger/client/Cargo.toml` and Rust sources/tests (or the exact workspace location selected from post-merge conventions)
- Modify: `programs/Cargo.toml` if workspace membership is needed

**Interfaces:** Export fallible C ABI functions for instruction encode, IDL account metadata, config PDA derivation, shard decode, and indexed instruction decode. Each function accepts explicit pointer/length inputs and returns an owned result plus an error code; provide one matching free function. JSON results use `{ "ok": true, "data": ... }` or `{ "ok": false, "error": { "code": ..., "message": ... } }`.

- [ ] Write Rust tests for Borsh bytes and instruction enum discriminants across supported instructions.
- [ ] Write tests for IDL account ordering, signer/writable flags, missing and duplicate account names, and PDA derivation.
- [ ] Write shard and instruction-data decode tests using existing core fixtures; include malformed/truncated bytes.
- [ ] Run the new tests and confirm the assertions fail before implementing the ABI.
- [ ] Implement the smallest Rust wrapper around existing core/IDL functions; do not duplicate layouts or enum constants.
- [ ] Run crate tests, formatting, and clippy for the client crate; document exact ABI ownership and error semantics in its module header.
- [ ] Commit the client crate independently.

## Task 3: Connect sacco_core to wallet, client ABI, and indexer

**Files:**
- Modify: `modules/sacco_core/src/sacco_core_impl.h`
- Modify: `modules/sacco_core/src/sacco_core_impl.cpp`
- Modify: `modules/sacco_core/CMakeLists.txt`
- Modify: `modules/sacco_core/flake.nix`
- Modify: `modules/sacco_core/metadata.json`
- Create or modify: focused `sacco_core` unit tests following the repository’s existing harness

**Interfaces:** Core methods accept validated operation JSON and selected identity ID; return the structured envelope defined in the spec with `status`, optional `transaction_hash`, decoded `data`, and `error`. Query methods return stable named collections for balances, deposits, loans, expenses, and audit events. The client ABI owns all Borsh and IDL conversions.

- [ ] Add failing tests for JSON escaping/control characters, invalid numeric fields, missing signer identity, wallet rejection, sequencer error, and indexer unavailable/syncing states.
- [ ] Add tests that success is returned only after wallet/sequencer acceptance and that the transaction hash is preserved.
- [ ] Implement typed Logos dependencies to `lez_core` and `lez_indexer_module`; pin both module inputs and metadata dependencies.
- [ ] Link the native Rust static library through CMake/Nix and check ABI error paths and buffer release.
- [ ] Implement instruction dispatch from Rust-validated bytes and IDL-derived account metas; never accept UI-supplied signer flags.
- [ ] Implement indexer account reads and paginated transaction history with explicit loading/error states.
- [ ] Persist only program ID, active SACCO ID, and generated demo record IDs under the module instance path.
- [ ] Run focused C++ tests and module build; commit the core integration.

## Task 4: Make member, committee, and auditor views functional

**Files:**
- Modify: `modules/sacco_ui/Main.qml`
- Modify: member, committee, and auditor QML view files identified in the current module
- Add or modify: QML test files in the existing UI test harness

**Interfaces:** `Main.qml` owns selected wallet identity and passes it to role views. Core query methods supply the named model shapes from Task 3. UI methods submit operation payloads without signer booleans and render the returned transaction state.

- [ ] Add QML tests for loading wallet accounts and selecting a member identity; verify refresh works after selection.
- [ ] Add tests for rendering actual balances/deposits/loans and for disabling empty, malformed, or out-of-range actions.
- [ ] Add tests for three distinct approvers, self-approval rejection, duplicate approval prevention, and threshold-two execution.
- [ ] Add auditor tests confirming one merged entry per hash, including member confirmation discovered from member-record history.
- [ ] Implement account selection and thread the selected identity through member and committee actions.
- [ ] Bind view models to decoded indexer data and use explicit loading, syncing, empty, and error states.
- [ ] Make member confirmation and multi-signature expense controls submit real core operations.
- [ ] Make role views scrollable at the module’s target window size; show evidence CIDs as safe text/links.
- [ ] Run the UI test harness and build the UI module; commit the UI changes.

## Task 5: Complete auditor history discovery and consistency

**Files:**
- Modify: `modules/sacco_core/src/sacco_core_impl.cpp` and focused core tests
- Modify: auditor QML and its tests

**Interfaces:** `getAuditHistory(sacco_id, offset, limit)` returns decoded entries with hash, instruction kind, account IDs, status, and evidence CID. It queries SACCO history, follows decoded record IDs to their histories, deduplicates by transaction hash, and preserves stable ordering/pagination metadata.

- [ ] Add tests for overlapping pages, repeated hashes, a confirmation on a member record only, and a record created outside the first SACCO page.
- [ ] Implement record-history expansion with bounded pagination and a deterministic hash-based merge.
- [ ] Ensure indexer lag and partial fetch errors are reported instead of returning a misleading complete empty history.
- [ ] Run focused core/UI auditor tests and commit the history fix.

## Task 6: Document local setup and API contract

**Files:**
- Modify: `docs/testing.md`
- Create or modify: `docs/deploy.md`
- Modify: `modules/sacco_core/README.md`
- Modify: `modules/sacco_ui/README.md`
- Modify: issue #5 API contract doc supplied by PR #17 after its dependency is available

- [ ] Document sequencer, Bedrock/indexer, Basecamp profile setup, module build/install/launch commands, local deployment, test account roles, and reset procedure.
- [ ] Document method names, payload validation, response envelope, signer selection, pending/finalized semantics, and auditor history discovery.
- [ ] Clearly state that local deposits/expenses are attestations and no member funds move.
- [ ] Check documentation commands against the actual `lgs basecamp` command surface and run markdown/lint checks available in the repository.
- [ ] Commit documentation and API-contract updates.

## Task 7: Run the local end-to-end acceptance flow

**Files:** Existing integration workflow/test harness and any generated local deployment notes.

- [ ] Start the local sequencer, matching Bedrock endpoint on 8080, configured indexer module, and project-local Basecamp profile.
- [ ] Build, install, and launch both `.lgx` modules through the Basecamp workflow.
- [ ] Deploy the ledger guest locally and initialize the validated config PDA.
- [ ] Create/select one proposer, two additional approvers, and one member; register/verify the SACCO and member.
- [ ] Record a deposit, confirm it as that member, propose an evidenced expense, collect two distinct valid approvals, and execute it.
- [ ] Verify finalized balances and that auditor history includes record, confirmation, approvals, execution, hashes, and evidence references.
- [ ] Capture exact successful commands and any environment limitations in `docs/testing.md`; do not mark unavailable acceptance steps complete.

## Task 8: Self-review and prepare the issue #5 pull request

- [ ] Review the full diff against every spec goal, non-goal, failure mode, and operational constraint.
- [ ] Search for placeholder responses, shell-outs to `spel`, duplicated Borsh/IDL constants, unsafe JSON concatenation, secret persistence, and optimistic success states; remove any matches.
- [ ] Run the complete Rust/C++/QML checks and Basecamp build available in the environment; report exact skipped checks and reasons.
- [ ] Request a fresh code review, fix valid findings, and rerun affected checks.
- [ ] Commit and push through the repository’s `push-and-commit-changes-to-github` workflow, update the existing issue #5 PR/branch, and provide a PR description referencing issue #5. Do not merge.

## Self-review

- **Spec coverage:** Goals 1–6 are handled by Tasks 1–5; local demo and documentation by Tasks 6–7; final review/push by Task 8. Non-goals and operational constraints are carried in Global Constraints and tested/documented in Tasks 1–7.
- **Placeholder scan:** No queued/unavailable mock behavior is accepted as completion; Task 8 explicitly searches for placeholders and shell-outs.
- **Type consistency:** The Rust client ABI returns owned fallible results; C++ core converts these into the one documented JSON envelope; QML consumes named collection fields and passes a selected identity ID only.
- **Review Focus:** All five input/failure classes have explicit tests in Tasks 1–5. Pagination and confirmation coverage is assigned to Task 5; missing/duplicate account handling to Task 2; signer/PDA correctness to Task 1.
