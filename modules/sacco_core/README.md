# sacco_core

Headless Logos **core module** for Transparent SACCO.

## Role

- Expose SACCO ledger operations to `sacco_ui` (and CLI via `logosctl`)
- Later: call the deployed `sacco_ledger` LEZ program / wallet APIs
- Optional: Logos Storage for release evidence CIDs

## API (scaffold)

| Method | Purpose |
|--------|---------|
| `getStatus` | Load / health check |
| `registerInstitution` | Queue institution registration |
| `payFees` | Queue locked fee payment |
| `verifyPaymentByRef` | Queue payment lookup |
| `requestRelease` | Queue multi-sig release request |
| `approveRelease` | Queue release approval |

Current implementations return JSON-shaped scaffold payloads and emit `operationQueued`. They do **not** yet submit LEZ transactions.

Next: wire these methods to wallet/`spel` using [`artifacts/fee_transparency-idl.json`](../../artifacts/fee_transparency-idl.json) and the custody flow in [`docs/custody.md`](../../docs/custody.md) (`Initialize` → vault → `PayFees` → multi-sig → `ExecuteRelease`).

## Build

Requires Nix + [`logos-module-builder`](https://github.com/logos-co/logos-module-builder).

```bash
cd modules/sacco_core
nix build .#lgx          # package as .lgx for Basecamp / logosctl
# or follow the core-module tutorial for logosctl install
```

Preview / install steps: [docs/basecamp-publishing.md](../../docs/basecamp-publishing.md).
