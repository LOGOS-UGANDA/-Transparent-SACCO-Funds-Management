# SACCO Pivot: Roadmap PR + Phase Issues — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Land the SACCO-pivot roadmap as a docs-only PR and open 8 unassigned phase issues on the repo so the team can pick up work.

**Architecture:** No code changes. One new roadmap doc plus a README note go on branch `sacco-pivot-plan` → PR to `main`. In parallel, enable Issues on the repo, add 4 workstream labels, create 8 issues (one per phase), then link them from the PR body.

**Tech Stack:** Markdown, git, GitHub CLI (`gh`).

**Spec:** `docs/superpowers/specs/2026-10-25-sacco-pivot-plan-and-issues-design.md`

## Global Constraints

- **No schedule language anywhere** (roadmap doc, issues, README note): no "week(s)", no dates. The plan is worded as **12 steps**. Existing source phrasing is reworded, e.g. "Weeks 2–4" → "Steps 2–4", "12 weeks, from 26 Oct 2026" → "12 steps", "Run 2 to 4 weeks with real members" → "Run the pilot with real members", "the 24 Oct SACCO Build Sprint" → "the recent SACCO Build Sprint", "Ticks show status as of 24 Oct 2026" → "Ticks show work already done", done-item metas "July 2026 report"/"August 2026 report" → "field research report"/"workstreams report". Version numbers (v0.2.0, v0.3) are not dates and stay.
- Branch: `sacco-pivot-plan` off `origin/main`. PR targets `main`. Title: `SACCO pivot: roadmap and 12-step plan`.
- All `gh` commands run with `GH_TOKEN`, `GITHUB_TOKEN`, `GITHUB_PERSONAL_ACCESS_TOKEN` unset (stored `YodHeVauHe` credentials; plain `gh` in this session inherits a stale env token).
- Repo: `LOGOS-UGANDA/-Transparent-SACCO-Funds-Management`.
- Labels (name → color): `lez` → `1D76DB`, `backend` → `0E8A16`, `frontend` → `5319E7`, `community` → `D93F0B`.
- Issues are created **unassigned**. Titles: `Phase N: <name> (Steps X–Y)`.
- Phase→steps mapping (authoritative, totals 12; overlaps are faithful to source): P0=Step 1, P1=Steps 2–4, P2=Step 4, P3=Steps 5–7, P4=Steps 7–8, P5=Steps 8–9, P6=Step 9, P7=Steps 10–12.
- Docs only: nothing under `programs/`, `modules/`, `prototype/`, `artifacts/` changes.

## Review Focus

Failure modes for this work, most likely first. Each is pinned by the verification step of the owning task.

1. **Leftover schedule language** ("week", "Oct 2026", "2026-") surviving in the roadmap or issues → Task 1 Step 4 grep must return nothing (except the no-dates rule's own mention in the spec/plan files, which are not the deliverable doc).
2. **Step mis-numbering** (mapping not summing to 12 or phases labeled with wrong ranges) → Task 1 Step 5 grep must print exactly the 8 mappings above.
3. **Done-ticks lost** (the 6 already-done items must remain `[x]`: 5 phase tasks — field research, workstreams formed, vault+multi-sig exists, IDL exported, core+UI scaffolds exist — plus 1 winnability item) → Task 1 Step 6 grep count must equal 6.
4. **Wrong repo / wrong labels / accidental assignment** on issues → Task 5 Step 3 verifies 8 issues, all on the LOGOS-UGANDA repo, label sets per mapping, `assignees: []`.
5. **PR carrying unintended files** → Task 3 Step 2 verifies the PR diff touches exactly 4 paths (`docs/roadmap-sacco-pivot.md`, `README.md`, the spec, this plan).

---

### Task 1: Branch + roadmap doc

**Files:**
- Create: `docs/roadmap-sacco-pivot.md`
- Commit alongside (already written, currently untracked): `docs/superpowers/specs/2026-10-25-sacco-pivot-plan-and-issues-design.md`, `docs/superpowers/plans/2026-10-25-sacco-pivot-plan-and-issues.md`

**Interfaces:**
- Produces: the roadmap doc that Task 2's README note links to, Task 5's issue bodies quote, and Task 3's PR describes.

- [ ] **Step 1: Create the branch**

```bash
git checkout -b sacco-pivot-plan origin/main
```

Expected: `Switched to a new branch 'sacco-pivot-plan'` (untracked `docs/superpowers/**` files carry over in the working tree).

- [ ] **Step 2: Write `docs/roadmap-sacco-pivot.md`**

Structure (sections in order) and pinned content:

1. **Title + tagline**: `# Transparent SACCO on Logos — Pivot Roadmap`; one line: "The start-to-finish map for Logos Circle Kampala's winnable issue: what we build, who owns which piece, the order we do it in, and how we know we won. **12 steps, 8 phases.**"
2. **The project at a glance** — properties list: Problem ("SACCO members cannot see how savings, loans and expenses are handled, so trust is low."), Solution ("A Logos app where every deposit, loan and expense is recorded and approved on chain, with members' personal finances kept private."), Circle ("Logos Circle Kampala"), Repo (`https://github.com/LOGOS-UGANDA/-Transparent-SACCO-Funds-Management`), Plan ("12 steps"), Status ("Phase 0, scoping"). Callout **What winning means**: "A real SACCO using the tool, members checking it, and measured results reported publicly. Strong code alone does not win a Circles issue."
3. **Architecture** — mermaid `flowchart LR`: Members/Treasurer and committee/Auditor → sacco_ui (QML in Basecamp) → sacco_core (Logos Core module) → sacco_ledger (LEZ program, public and private accounts); sacco_core → Logos Storage (Receipts and minutes by CID, label "upload"); sacco_core → Logos Messaging (Alerts and committee chat, label "notify"). Closing line: "Real money still moves through mobile money or cash. The chain is the shared, tamper-evident record of it, with approvals enforced by the program."
4. **Phases and tasks** — one subsection per phase (H3), each with a small table (Steps / Lead workstream / Logos pieces / Done when) and a task checklist. Caption: "Ticks show work already done." Verbatim task lists:

   - **Phase 0: Scope and validate** — Step 1 · Community and pilot · Circles method · Done when: One pilot SACCO has signed off on three flows and the metrics.
     - [x] Field research at SACCOs around Kampala *(field research report)*
     - [x] Workstreams formed *(workstreams report)*
     - [ ] Pick one pilot SACCO and get written agreement
     - [ ] Lock scope to three flows: deposits, loans, expenses
     - [ ] Agree 2–3 metrics and run a baseline trust survey
     - [ ] Name an owner for each workstream
   - **Phase 1: Domain model and LEZ program** — Steps 2–4 · Logos Blockchain integration · LEZ (Rust, RISC Zero) · Done when: `sacco_ledger` passes tests for every instruction.
     - [x] Vault, multi-sig release flow and approver checks exist (`fee_transparency`)
     - [x] Program IDL exported (`artifacts/`)
     - [ ] Rename `fee_transparency` to `sacco_ledger` and drop university fee concepts
     - [ ] Add SACCO, Member and Committee accounts
     - [ ] Add `RecordDeposit` with treasurer record and member confirmation
     - [ ] Add loans: `ApplyLoan`, `ApproveLoan`, `DisburseLoan`, `RecordRepayment`
     - [ ] Adapt release flow into `ProposeExpense`, approve, `ExecuteExpense`
     - [ ] Add `PostPeriodSummary` for period totals
     - [ ] Unit tests per instruction plus CI running `RISC0_DEV_MODE=1 cargo test`
   - **Phase 2: Upgrade and deploy** — Step 4 · Logos Blockchain integration · Testnet v0.3, `logos-scaffold` · Done when: Program deployed on local sequencer and testnet with a working call.
     - [ ] Bump `lee_core` from v0.2.0 to the v0.3 release and fix breakages
     - [x] Switch docs and scripts from `logoscore`/`lgpm` to `logosctl`
     - [ ] Deploy to local sequencer with `logos-scaffold`
     - [ ] Deploy to testnet and record the program ID in the repo
   - **Phase 3: Core and UI wiring** — Steps 5–7 · Backend and Frontend · Logos Core module, QML UI, Basecamp · Done when: One full flow works end to end inside Basecamp.
     - [x] `sacco_core` and `sacco_ui` scaffolds exist *(placeholder JSON only)*
     - [ ] Wire `sacco_core` to LEZ through the IDL
     - [ ] Member view: my deposits, my loan, SACCO totals
     - [ ] Treasurer and committee view: record, propose, approve
     - [ ] Auditor view: full trail with evidence links
     - [ ] Demo flow: deposit, member views it, expense passes multi-sig
   - **Phase 4: Privacy** — Steps 7–8 · Logos Blockchain integration · LEZ private accounts · Done when: Individual balances are private; SACCO totals stay public.
     - [ ] Move member balances and loans to private accounts
     - [ ] Give auditor a view path for private data
     - [ ] Keep SACCO totals, loan-book totals and expenses public
     - [ ] Write a one-page privacy explainer for members
   - **Phase 5: Storage and Messaging** — Steps 8–9 · Backend · Logos Storage, Logos Messaging · Done when: Receipts stored by CID and members get alerts.
     - [ ] Upload receipts, loan agreements and minutes to Logos Storage
     - [ ] Replace the free-text `evidenceHash` with the stored CID
     - [ ] Member alerts: deposit recorded, loan approved, expense proposed
     - [ ] Committee group chat for approvals
   - **Phase 6: Package and publish** — Step 9 · Frontend · `.lgx` packages, `logos-repo.json` · Done when: Anyone can install the app from Basecamp.
     - [ ] Build signed `.lgx` packages
     - [ ] Fill `indexUrl` and fix the homepage in `logos-repo.json`
     - [ ] Post the repo link on the Basecamp module forum thread
   - **Phase 7: Pilot and report** — Steps 10–12 · Community and pilot · Circles reporting · Done when: Public report with metrics posted on the Logos forum.
     - [ ] Train treasurer and committee at the pilot SACCO
     - [ ] Run the pilot with real members
     - [ ] Repeat the trust survey and collect metrics
     - [ ] Publish results on the Logos forum and X
5. **Step order** — table: Phase | Steps | Lead | Done when (one row per phase, using the mapping above).
6. **Workstreams** — table: UI/UX ("Member, committee and auditor screens" · Basecamp UX · "Wireframes tested with pilot members"), Frontend ("`sacco_ui` and packaging" · QML UI module, `.lgx` · "Member view on real data"), Backend ("`sacco_core`, Storage, Messaging" · Logos Core module · "Core calls LEZ through the IDL"), Logos Blockchain integration ("`sacco_ledger`, deploys, privacy" · LEZ · "SACCO program with tests"), Community and pilot ("Partner SACCO, training, metrics" · Circles method · "Signed pilot agreement"). Closing note: "The earlier report names five workstreams; the fifth is shown here as Community and pilot — rename it to match what the Circle agreed."
7. **Data model** — table (Account or instruction | Kind | Visibility | Notes): SACCO / Account / Public / "Name, registration, totals"; Committee / Account / Public / "Approvers and threshold"; Member / Account / Private / "Savings, shares, loan balance"; RecordDeposit / Instruction / "Member private, total public" / "Treasurer records, member confirms"; ApplyLoan / ApproveLoan / Instruction / "Private with public totals" / "Committee multi-sig"; DisburseLoan / RecordRepayment / Instruction / "Private with public totals" / "Evidence CID attached"; ProposeExpense / ExecuteExpense / Instruction / Public / "Reuses current release flow"; PostPeriodSummary / Instruction / Public / "Period totals for all members".
8. **Metrics** — table (Metric | How we measure it | Phase): "Members who check their record" / "Unique member logins during the pilot" / Phase 7; "Share of activity on chain" / "Deposits, loans and expenses recorded, against the paper books" / Phase 7; "Member trust" / "Same short survey before and after the pilot" / Phases 0 and 7.
9. **Winnability checklist** — caption "From the Logos Circles guide; about 80 percent ticked means the issue is winnable.": [x] Real local problem confirmed by field research; [ ] Narrow, clear scope with one pilot SACCO; [ ] Validated with the people affected; [ ] Built with the community through a small pilot; [ ] Shareable result: working tool or public demo; [ ] 2–3 simple metrics tracked; [ ] Fits Logos values: transparency and privacy; [ ] Finished in about three months and reported publicly.
10. **Open questions** (numbered): Which SACCO is the pilot partner, and who signs for it? · Does the v0.2.0 LEZ code compile against testnet v0.3? · Which is the main repo: upstream `makmot256`, the LOGOS-UGANDA fork, or `kaksv/Transparent-Sacco-on-logos`? · What came out of the recent SACCO Build Sprint at Makerere? · Do we hold tokens on chain at all, or only attest mobile money and cash records?
11. **How to start building** — numbered steps: Install Basecamp, `logosctl` and Nix with flakes · Use `logos-scaffold` to start a local sequencer and deploy `programs/fee_transparency` · `nix flake init -t github:logos-co/logos-module-builder/0.3.2`, then `nix build` · `nix build .#lgx-portable`, then `logosctl package install`.
12. **References** — links: Project repo (github.com/LOGOS-UGANDA/-Transparent-SACCO-Funds-Management), Logos docs (docs.logos.co), Write and deploy an LEZ program, Build a Logos Core module, What is a winnable issue (circles.logos.co), How Logos works (logos.co/field-guide/how-logos-works), Basecamp module repository thread (forum.logos.co/t/basecamp-module-repository-links/1843).

- [ ] **Step 3: Commit**

```bash
git add docs/roadmap-sacco-pivot.md docs/superpowers/
git commit -m "docs: add SACCO pivot roadmap (12 steps)"
```

- [ ] **Step 4: Verify no schedule language**

Run: `grep -niE 'week|oct 2026|, 2026' docs/roadmap-sacco-pivot.md`
Expected: no output.

- [ ] **Step 5: Verify step mapping**

Run: `grep -E '^### Phase|Steps [0-9]' docs/roadmap-sacco-pivot.md`
Expected: 8 phase headings, each paired with the exact step range from the Global Constraints mapping.

- [ ] **Step 6: Verify done-ticks preserved**

Run: `grep -c '\- \[x\]' docs/roadmap-sacco-pivot.md`
Expected: `6` (5 phase tasks + 1 winnability item).

### Task 2: README pivot note

**Files:**
- Modify: `README.md` (insert after the intro line "Built for universities, SACCOs…", before `---`)

**Interfaces:**
- Consumes: `docs/roadmap-sacco-pivot.md` from Task 1 (relative link).

- [ ] **Step 1: Insert the note**

Exact copy:

```markdown
> **Pivot:** the product is pivoting from university fee transparency to a **SACCO ledger** — deposits, loans and expenses recorded and approved on chain, with member balances private. The university-fee model below is the earlier concept and stays for reference. See the [pivot roadmap](./docs/roadmap-sacco-pivot.md) (12 steps, 8 phases).
```

- [ ] **Step 2: Verify link resolves and no schedule language**

Run: `test -f docs/roadmap-sacco-pivot.md && grep -niE 'week|oct 2026|, 2026' README.md`
Expected: exit 0 from `test`, grep returns nothing.

- [ ] **Step 3: Commit**

```bash
git add README.md
git commit -m "docs: add SACCO pivot note to README"
```

### Task 3: Push and open the PR

**Files:**
- none (git/gh operations)

**Interfaces:**
- Consumes: branch `sacco-pivot-plan` from Tasks 1–2.
- Produces: PR number used by Task 5 issue bodies and Task 6 PR-body edit.

- [ ] **Step 1: Push and open PR**

```bash
git push -u origin sacco-pivot-plan
env -u GH_TOKEN -u GITHUB_TOKEN -u GITHUB_PERSONAL_ACCESS_TOKEN gh pr create \
  --base main --head sacco-pivot-plan \
  --title "SACCO pivot: roadmap and 12-step plan" \
  --body "Pivots the product from university fee transparency to a SACCO ledger. Docs only — no code changes.\n\n- New: \`docs/roadmap-sacco-pivot.md\` — the full roadmap, worded as 12 steps (8 phases): problem, architecture, phases with tasks and exit criteria, data model, metrics, workstreams, open questions.\n- README: short pivot note linking the roadmap.\n- Phase issues will follow (one per phase, unassigned) and be linked here.\n\nDecisions in \`docs/superpowers/specs/2026-10-25-sacco-pivot-plan-and-issues-design.md\`."
```

Expected: PR URL printed; record the PR number.

- [ ] **Step 2: Verify PR contains exactly the 4 intended files**

Run: `env -u GH_TOKEN -u GITHUB_TOKEN -u GITHUB_PERSONAL_ACCESS_TOKEN gh pr diff --name-only`
Expected exactly: `README.md`, `docs/roadmap-sacco-pivot.md`, `docs/superpowers/plans/2026-10-25-sacco-pivot-plan-and-issues.md`, `docs/superpowers/specs/2026-10-25-sacco-pivot-plan-and-issues-design.md`.

### Task 4: Enable Issues and create labels

**Files:**
- none (repo settings + labels via API)

**Interfaces:**
- Produces: labels `lez`, `backend`, `frontend`, `community` used by Task 5.

- [ ] **Step 1: Enable Issues**

```bash
env -u GH_TOKEN -u GITHUB_TOKEN -u GITHUB_PERSONAL_ACCESS_TOKEN \
  gh api -X PATCH repos/LOGOS-UGANDA/-Transparent-SACCO-Funds-Management -f has_issues=true
```

Expected: `"has_issues": true` in the response. If HTTP 403 (needs admin): stop and ask YodHeVauHe to flip Settings → General → Features → Issues, then re-verify.

- [ ] **Step 2: Create the 4 labels**

```bash
for l in lez:1D76DB backend:0E8A16 frontend:5319E7 community:D93F0B; do
  env -u GH_TOKEN -u GITHUB_TOKEN -u GITHUB_PERSONAL_ACCESS_TOKEN \
    gh label create "${l%%:*}" --color "${l##*:}" --repo LOGOS-UGANDA/-Transparent-SACCO-Funds-Management
done
```

Expected: `✓ Label "..." created` ×4 (`--force` if any pre-exists).

- [ ] **Step 3: Verify**

Run: `env -u GH_TOKEN -u GITHUB_TOKEN -u GITHUB_PERSONAL_ACCESS_TOKEN gh label list`
Expected: includes all 4 names with the pinned colors.

### Task 5: Create the 8 phase issues

**Files:**
- none (issues via `gh issue create`)

**Interfaces:**
- Consumes: roadmap doc content (Task 1), labels (Task 4), PR number (Task 3).
- Produces: 8 issue numbers/URLs used by Task 6.

- [ ] **Step 1: Create issues Phase 0–7**

One `gh issue create` per phase with `--title "Phase N: <name> (Steps X–Y)"` (singular "Step 1"/"Step 4"/"Step 9" for single-step phases), labels per this mapping: P0 `community` · P1 `lez` · P2 `lez` · P3 `backend,frontend` · P4 `lez` · P5 `backend` · P6 `frontend` · P7 `community`.

Body template (fill per phase; content verbatim from the roadmap doc's phase subsection — same table rows as Markdown, same checklist items, `[x]` ticks preserved). Link the roadmap by default-branch path so it resolves after merge; the PR number in the same line covers the pre-merge window:

```markdown
Part of the [SACCO pivot roadmap](https://github.com/LOGOS-UGANDA/-Transparent-SACCO-Funds-Management/blob/main/docs/roadmap-sacco-pivot.md) (PR #<PR-NUMBER>).

| | |
|---|---|
| Steps | X–Y |
| Lead workstream | <workstream> |
| Logos pieces | <logos pieces> |

**Done when:** <exit criterion>

## Tasks
- [ ] <task> *(…one line per task, verbatim from the roadmap)*
```

- [ ] **Step 2: Verify all 8 exist**

Run: `env -u GH_TOKEN -u GITHUB_TOKEN -u GITHUB_PERSONAL_ACCESS_TOKEN gh issue list --limit 20 --json number,title,labels,assignees`
Expected: 8 issues, titles matching `Phase N: <name> (Steps …)` with the label mapping above, every `assignees` array empty.

- [ ] **Step 3: Verify no schedule language in issue bodies**

Run: `env -u GH_TOKEN -u GITHUB_TOKEN -u GITHUB_PERSONAL_ACCESS_TOKEN gh issue list --limit 20 --json body -q '.[].body' | grep -niE 'week|oct 2026|, 2026'`
Expected: no output.

### Task 6: Link the issues from the PR

**Files:**
- none (PR body edit via `gh pr edit`)

**Interfaces:**
- Consumes: issue numbers (Task 5), PR number (Task 3).

- [ ] **Step 1: Append the issue index to the PR body**

Append to the existing PR body:

```markdown

## Phase issues
- Phase 0 … #<n>
- Phase 1 … #<n>
- Phase 2 … #<n>
- Phase 3 … #<n>
- Phase 4 … #<n>
- Phase 5 … #<n>
- Phase 6 … #<n>
- Phase 7 … #<n>

All unassigned — pick your phase. Phase order is the build order.
```

- [ ] **Step 2: Verify**

Run: `env -u GH_TOKEN -u GITHUB_TOKEN -u GITHUB_PERSONAL_ACCESS_TOKEN gh pr view --json body -q .body`
Expected: contains `## Phase issues` and 8 `#<number>` links.
