# sacco_core

Headless Logos **core module** for Transparent SACCO.

## Role

- Expose fee-transparency operations to `sacco_ui` (and CLI via `logoscore`)
- Later: call the deployed `fee_transparency` LEZ program / wallet APIs
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

## Build

Requires Nix + [`logos-module-builder`](https://github.com/logos-co/logos-module-builder).

```bash
cd modules/sacco_core
nix build .#lgx          # package as .lgx for Basecamp / lgpm
# or follow the core-module tutorial for logoscore install
```

Preview / install steps: [docs/basecamp-publishing.md](../../docs/basecamp-publishing.md).
