# sacco_core

Headless Logos core module for the Transparent SACCO attestation ledger. The product records off-chain deposits, loan activity, and expenses; it never receives, holds, or transfers SACCO money.

The intended module contract is documented in [API.md](API.md). The Rust local client now submits IDL-shaped public instructions through the pinned LEZ wallet and reads known account shards. The C++ module's `dispatchInstruction()` and query methods are not wired to this client yet, so the Basecamp UI still does not submit transactions or display live ledger state.

## Build

Requires Nix + [`logos-module-builder`](https://github.com/logos-co/logos-module-builder).

```bash
cd modules/sacco_core
nix build .#lgx
nix build .#lgx-portable
```

Install and preview steps: [docs/basecamp-publishing.md](../../docs/basecamp-publishing.md).
