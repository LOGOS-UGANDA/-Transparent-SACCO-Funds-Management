# sacco_core

Headless Logos **core module scaffold** for Transparent SACCO.

The product records and approves off-chain deposits, loan activity, and
expenses. It does not receive, hold, or transfer SACCO money.

## Role

- Planned: expose `sacco_ledger` attestation operations to `sacco_ui` (and CLI via `logosctl`)
- Later: submit IDL instructions that record off-chain events and approvals
- Optional: Logos Storage for release evidence CIDs

## API (scaffold)

| Method | Purpose |
|--------|---------|
| `getStatus` | Load / health check |
| `registerInstitution`, `payFees`, `verifyPaymentByRef` | Legacy fee-transparency stubs; return scaffold JSON only |
| `requestRelease`, `approveRelease` | Legacy custody stubs; return scaffold JSON only and move no funds |

Current implementations return JSON-shaped scaffold payloads and emit `operationQueued`. They do **not** yet submit LEZ transactions.

Next: replace the legacy stubs with `sacco_ledger` IDL operations for recording deposits, member confirmations, loan approvals, and expense approvals, with evidence of off-chain payments. No SACCO funds or tokens are sent to the program.

## Build

Requires Nix + [`logos-module-builder`](https://github.com/logos-co/logos-module-builder).

```bash
cd modules/sacco_core
nix build .#lgx          # development package for local Basecamp use
nix build .#lgx-portable # portable package for logosctl
```

Install and preview steps: [docs/basecamp-publishing.md](../../docs/basecamp-publishing.md).
