# sacco_ui

Logos **UI module** (QML) for Transparent SACCO.

## Role

Desktop UI hosted by Basecamp or `logos-standalone-app`. Calls `sacco_core` through the Logos bridge (`logos.callModule`) — QML stays sandboxed (no direct network).

## Screens (scaffold)

Current `Main.qml` is a thin shell: status, demo pay/verify buttons, last core response. Full product screens (dashboard, institutions, releases) should follow `prototype/src/pages/` once core ↔ LEZ is live.

## Build / preview

```bash
cd modules/sacco_ui
# After nix flake lock resolves local sacco_core:
nix run                 # preview via logos-standalone-app
nix build .#lgx         # package for Basecamp
```

Declare `sacco_core` as a dependency in `metadata.json` (already set). Point the flake input at a published GitHub URL when you stop using `path:../sacco_core`.
