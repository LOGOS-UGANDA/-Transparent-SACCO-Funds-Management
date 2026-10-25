# Deploying `sacco_ledger`

> Phase 2 runbook (issue #4). The program compiles and tests green (see CI); the deploy steps need the Logos toolchain on a machine with the sequencer/testnet credentials.

## Prerequisites

- Basecamp installed ([logos.co/basecamp](https://logos.co/basecamp)) — gives you the wallet, the local LEZ node and `logosctl`
- `logos-scaffold` for the local sequencer ([docs: write and deploy an LEZ program](https://docs.logos.co/lez/programs/write-and-deploy-lez-program-with-scaffold))
- Nix with flakes (for building the `.lgx` packages)

## 1. Local sequencer

```bash
# start a local sequencer
logos-scaffold up
# deploy the program from this repo
logos-scaffold deploy programs/sacco_ledger
# note the program id it prints — record it below
```

Record the deployed program id here once done: `__________`

Then run a working call against the local node:

```bash
logosctl call sacco_ledger initialize        # config PDA
logosctl call sacco_ledger register_sacco "Kampala Traders SACCO" UG-SACCO-001 <treasury-id>
logosctl call sacco_ledger verify_sacco      # admin signs
```

**Done when:** the initialize/register/verify round-trip returns success on the local sequencer.

## 2. Testnet

```bash
logos-scaffold deploy --network testnet programs/sacco_ledger
```

- Record the testnet program id here: `__________`
- Commit the program id to this file (and `README.md`) in the same PR as the deploy.

**Done when:** one working call (initialize) succeeds on testnet and the program id is recorded.

## 3. Notes on the v0.3 port

This repo runs on `lee_core` v0.2.5-rc2 — the version `token_core` and `spel` 0.7 pair with today. Testnet v0.3 replaces the account model (sharded `AccountData`, diff-only outputs), which is a handler-port, not a tag bump. Budget a focused session for it after the local-sequencer deploy; the unit-test suite (52 tests) is the safety net during that port.
