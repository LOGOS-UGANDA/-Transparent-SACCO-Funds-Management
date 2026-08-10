# sacco_ui

Logos **UI module** (QML) for Transparent SACCO.

## Role

User-facing screens (mirroring the prototype flows):

- Dashboard
- Institutions / institution detail
- Pay fees
- Verify payment
- Releases

Runs in Basecamp or `logos-standalone-app`. Chain and network access go through `sacco_core`, not direct HTTP from QML.

## Status

Scaffold only. Implement with the [C++ / QML UI module guide](https://docs.logos.co/core/build-modules/build-a-logos-cpp-ui-module). UX reference: `prototype/src/pages/`.
