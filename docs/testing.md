# Testing Transparent SACCO

What you can verify today, and how Basecamp fits in.

## Reality check

| Layer | Testable now? | What “working” means |
|-------|---------------|----------------------|
| UX prototype (`prototype/`) | Yes | Demo flows with Hardhat/demo data |
| LEZ program (host handlers) | Partially | `cargo test` once LEZ deps resolve |
| LEZ program on-chain | Not yet | Needs guest build + `wallet deploy-program` |
| `sacco_core` / `sacco_ui` in Basecamp | Not yet packaged | Need Nix → `.lgx`, then install/load |
| Full pay → multi-sig → release | Not yet | Needs core ↔ LEZ wallet wiring |

Basecamp is the right place to **eventually** exercise the product UI + modules. It is also useful **today** to learn the stack (wallet, LEZ, Package Manager) even before our modules ship.

---

## A. Use Basecamp now (stack rehearsal)

1. Install from [logos.co/basecamp](https://logos.co/basecamp).
2. Open Basecamp and confirm bundled modules load (Package Manager / Modules).
3. Exercise the **LEZ wallet**: create accounts, get test tokens (Piñata/faucet if available), transfer, optionally create a custom token.
4. Open the **LEZ explorer / blockchain dashboard** if present — this is the environment `fee_transparency` will target.
5. Optionally try a third-party module from a community repo:  
   **Settings → Package Repositories → Add** a URL from the [forum thread](https://forum.logos.co/t/basecamp-module-repository-links/1843), then install from Applications.

This does **not** prove Transparent SACCO yet; it proves your Basecamp + LEZ environment works.

---

## B. Test the web prototype (product UX)

```bash
cd prototype
npm install
npm run dev
```

Walk through Dashboard → Institutions → Pay Fees → Verify → Releases. This validates UX only, not Logos custody.

---

## C. Test the LEZ program (CLI / host)

Requires Rust + Logos LEZ dependencies (see [lez-programs](https://github.com/logos-blockchain/lez-programs)).

```bash
cd programs
RISC0_DEV_MODE=1 cargo test -p fee_transparency_program
```

Later (on-chain):

```bash
# after guest build
wallet deploy-program target/guest/fee_transparency.bin
spel --idl ../artifacts/fee_transparency-idl.json initialize ...
# then register_institution → initialize_institution_vault → pay_fees → ...
```

See [custody.md](./custody.md) and [idl.md](./idl.md).

---

## D. Test modules outside Basecamp (recommended first)

### UI preview (standalone)

```bash
cd modules/sacco_ui
nix run    # logos-standalone-app preview
```

Expect: shell UI, status/demo buttons. Until `sacco_core` is installed alongside, core calls may fail — that is expected until both modules are packaged together.

### Core via logosctl

```bash
cd modules/sacco_core
nix build .#lgx
# install with logosctl into a local modules/ dir, then:
logosctl daemon -D -m ./modules
logosctl package install sacco_core
logosctl call sacco_core getStatus
logosctl call sacco_core recordDeposit sacco-id member-id 1000000 savings MPZA-REF
logosctl stop
```

Today these return **scaffold JSON**, not chain txs. Still useful to prove the module loads and the API surface works.

Official walkthrough: [Build and run a Logos core module](https://docs.logos.co/core/build-modules/build-and-run-a-logos-core-module).

---

## E. Test in Basecamp (when `.lgx` builds succeed)

This is the procedure you asked for.

### Prerequisites

- Nix with flakes (`experimental-features = nix-command flakes`)
- Basecamp installed (portable download or Nix build)
- Matching LGX variant:
  - Dev Basecamp → need `-dev` variant (often via dual bundler)
  - Portable Basecamp → portable `.lgx`

### 1. Build packages

```bash
cd modules/sacco_core
nix build .#lgx
# Prefer dual bundle for Basecamp flexibility:
# nix bundle --bundler github:logos-co/nix-bundle-lgx/tutorial-v3#dual .#lib

cd ../sacco_ui
nix build .#lgx
```

### 2. Install into Basecamp’s data dir

```bash
# macOS
BASECAMP_DIR="$HOME/Library/Application Support/Logos/LogosBasecamp"
# Linux
# BASECAMP_DIR="$HOME/.config/Logos/LogosBasecamp"

logosctl package install --file /path/to/sacco_core.lgx
# UI modules may use plugins dir depending on package type:
logosctl package install --file /path/to/sacco_ui.lgx
```

Or in Basecamp UI: **Package Manager → Install LGX Package** and select the `.lgx` files.

### 3. Load and smoke-test

1. Restart Basecamp (or use Modules view → Load).
2. Confirm `sacco_core` under Core Modules and `sacco_ui` under UI Modules / Applications.
3. Open Transparent SACCO UI.
4. Click **Refresh status** → should show scaffold ready string.
5. Click **Demo: pay fees** → should show scaffold JSON in “Last core response”.

### 4. Pass criteria (current scaffold)

- Modules install without variant errors  
- UI opens  
- `getStatus` / demo buttons return scaffold payloads (not crashes)

### 5. Pass criteria (after LEZ wiring — future)

- `Initialize` + vault + `PayFees` show vault balance change in LEZ explorer  
- Multi-sig approve → `ExecuteRelease` moves tokens to treasury  
- Payment verifiable via public accounts / explorer  

---

## F. Discovery test (published repo — later)

When `logos-repo.json` has a real `indexUrl`:

1. Basecamp → **Settings → Package Repositories → Add** your repo URL  
2. Install Transparent SACCO from Applications  
3. Same smoke tests as section E  

---

## Suggested order

1. Basecamp stack rehearsal (A)  
2. Prototype UX (B) if you care about screens  
3. Package + install modules in Basecamp (E) for shell/API smoke  
4. LEZ program tests + deploy (C)  
5. Wire `sacco_core` → IDL/wallet, then re-run E with real custody  
