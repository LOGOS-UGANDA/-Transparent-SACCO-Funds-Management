# sacco_ui

Logos **UI module scaffold** (QML) for Transparent SACCO.

The product records off-chain deposits, loans, and expenses on LEZ. SACCO
money is not deposited into or transferred by this application.

## Role

Desktop UI hosted by Basecamp or `logos-standalone-app`. Calls `sacco_core` through the Logos bridge (`logos.callModule`) — QML stays sandboxed (no direct network).

## Screens (scaffold)

Current `Main.qml` is a placeholder shell. Its fee-payment and payment-lookup buttons are legacy demos that return scaffold JSON; they do not represent the SACCO ledger flow and submit no LEZ transactions. Replace them with screens for recording and reviewing off-chain activity and its on-chain approvals. The React/Hardhat `prototype/` is a superseded custody demo, not a source for current product behavior.

## Build / preview

```bash
cd modules/sacco_ui
# After nix flake lock resolves local sacco_core:
nix run                 # preview via logos-standalone-app
nix build .#lgx         # package for Basecamp
```

Declare `sacco_core` as a dependency in `metadata.json` (already set). Point the flake input at a published GitHub URL when you stop using `path:../sacco_core`.
