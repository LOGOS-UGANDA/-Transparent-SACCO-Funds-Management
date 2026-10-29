# sacco_core

Headless Logos core module for the Transparent SACCO attestation ledger. The product records off-chain deposits, loan activity, and expenses; it never receives, holds, or transfers SACCO money.

The module exposes the instruction and query surface in [API.md](API.md). The current Phase 3 implementation is a UI and API scaffold. LEZ transaction submission and real account-state queries still require the wallet transport integration described in issue #5.

## Build

Requires Nix + [`logos-module-builder`](https://github.com/logos-co/logos-module-builder).

```bash
cd modules/sacco_core
nix build .#lgx
nix build .#lgx-portable
```

Install and preview steps: [docs/basecamp-publishing.md](../../docs/basecamp-publishing.md).
