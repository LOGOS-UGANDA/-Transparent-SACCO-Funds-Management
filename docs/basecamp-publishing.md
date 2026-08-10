# Publishing for Basecamp discovery

Basecamp finds third-party apps through **module repositories** (catalogues), not a single curated store.

## End-to-end path

1. Build core and UI modules with `logos-module-builder`.
2. Package each module as `.lgx` (for example `nix build .#lgx` or the dual bundler documented in Logos Core guides).
3. Publish signed packages and an `index.json` (typically via GitHub Releases).
4. Fill in `logos-repo.json` at the repo root:
   - `indexUrl` — URL of your release index
   - `trustedSigners` — signer DIDs for package verification
5. Host `logos-repo.json` at a stable HTTPS URL (GitHub Pages, raw release asset, or your domain).
6. Tell users to add that URL in Basecamp: **Settings → Package Repositories → Add**, then install from Applications / Package Manager.
7. Optionally list the URL on the [Logos forum module-repo thread](https://forum.logos.co/t/basecamp-module-repository-links/1843) so others can find it.

## Local install (during development)

- Install a local `.lgx` in Basecamp via Package Manager → Install LGX Package, or with `lgpm` into a modules directory.
- Preview UI modules with `logos-standalone-app` (`nix run` in the UI module flake).

## References

- [Install and load a module in Basecamp](https://docs.logos.co/core/build-modules/install-and-load-a-module-in-logos-basecamp)
- [Build and run a Logos core module](https://docs.logos.co/core/build-modules/build-and-run-a-logos-core-module)
- [Build a Logos C++ UI module](https://docs.logos.co/core/build-modules/build-a-logos-cpp-ui-module)
- [Basecamp](https://logos.co/basecamp)
