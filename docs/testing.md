# Testing Transparent SACCO

What you can verify today, and how Basecamp fits in.

## Reality check

| Layer | Testable now? | What “working” means |
|-------|---------------|----------------------|
| Legacy UX prototype (`prototype/`) | Yes, as a demo only | Old Hardhat fee-custody flows; does not test the current SACCO ledger |
| LEZ program (host handlers) | Partially | `cargo test` once LEZ deps resolve |
| LEZ program on-chain | Not yet | Needs guest build, deployment, and an IDL transaction runner |
| `sacco_core` / `sacco_ui` in Basecamp | Not yet packaged | Need Nix → `.lgx`, then install/load |
| SACCO ledger flow | Not yet | Needs core ↔ LEZ wiring for attesting off-chain deposits, loans, and expenses |

Basecamp is the right place to **eventually** exercise the product UI + modules. It is also useful **today** to learn the stack (wallet, LEZ, Package Manager) even before our modules ship.

---

## A. Use Basecamp now (stack rehearsal)

1. Install from [logos.co/basecamp](https://logos.co/basecamp).
2. Open Basecamp and confirm bundled modules load (Package Manager / Modules).
3. Optionally create an **LEZ wallet account** to learn how accounts and signing work. Token transfers and custom-token exercises are generic LEZ features, not Transparent SACCO flows: this project records off-chain deposits, loans and expenses and does not hold or transfer SACCO funds.
4. Open the **LEZ explorer / blockchain dashboard** if present — this is the environment `sacco_ledger` will target.
5. Optionally try a third-party module from a community repo:  
   **Settings → Package Repositories → Add** a URL from the [forum thread](https://forum.logos.co/t/basecamp-module-repository-links/1843), then install from Applications.

This does **not** prove Transparent SACCO yet; it proves your Basecamp + LEZ environment works.

---

## B. Explore the legacy web prototype

This React + Hardhat demo predates the SACCO attestation model. It simulates fee custody and transfers and is not the current product flow or a test of `sacco_ledger`.

```bash
cd prototype
npm install
npm run dev
```

Walk through Dashboard → Institutions → Pay Fees → Verify → Releases to inspect the historical demo only.

---

## C. Test the LEZ program (CLI / host)

Requires Rust + Logos LEZ dependencies (see [lez-programs](https://github.com/logos-blockchain/lez-programs)).

```bash
cd programs
RISC0_DEV_MODE=1 cargo test -p sacco_ledger_program
```

The program can be built and deployed using the steps in [`sacco_ledger/README.md`](../programs/sacco_ledger/README.md). A client runner for submitting the current IDL instructions is not yet implemented. Future calls will record and approve off-chain SACCO activity; they will not deposit or transfer SACCO funds.

Use the local sequencer for development. The current local workflow needs no public testnet wallet or test tokens. Public testnet deployment is deferred, is not needed to build the product, and is not part of the current test steps. See [architecture.md](./architecture.md), the [superseded custody proposal](./custody.md), and [idl.md](./idl.md).

---

## D. Test modules outside Basecamp (recommended first)

### UI preview (standalone)

```bash
cd modules/sacco_ui
nix run    # logos-standalone-app preview
```

Expect: shell UI, status/demo buttons. Until `sacco_core` is installed alongside, core calls may fail — that is expected until both modules are packaged together.

### Core via `logosctl`

```bash
cd modules/sacco_core
nix build .#lgx-portable
logosctl daemon start --detach
logosctl package install --file ./result/*.lgx --yes
logosctl module load sacco_core
logosctl call sacco_core getStatus
logosctl daemon stop
```

The `sacco_core` fee-payment and release methods are legacy placeholders and return **scaffold JSON**, not chain transactions. Use `getStatus` to verify module loading; these legacy methods do not represent current product operations.

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

### 2. Install into Basecamp

In Basecamp, use **Package Manager → Install LGX Package** and select the `.lgx` files.

### 3. Load and smoke-test

1. Restart Basecamp (or use Modules view → Load).
2. Confirm `sacco_core` under Core Modules and `sacco_ui` under UI Modules / Applications.
3. Open Transparent SACCO UI.
4. Click **Refresh status** → should show scaffold ready string.
5. The visible legacy demo buttons return scaffold JSON; they do not send transactions or move SACCO funds.

### 4. Pass criteria (current scaffold)

- Modules install without variant errors  
- UI opens  
- `getStatus` works; any legacy demo button returns scaffold JSON only

### 5. Pass criteria (after LEZ wiring — future)

- `RecordDeposit` updates member and SACCO totals (savings vs share capital split)  
- Multi-sig approval → `ExecuteExpense` records evidence that an off-chain payment happened
- Deposits, loans and expenses verifiable via public accounts / explorer  

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
5. Wire `sacco_core` → IDL/wallet for attestation of off-chain activity; do not add SACCO fund custody or transfers
