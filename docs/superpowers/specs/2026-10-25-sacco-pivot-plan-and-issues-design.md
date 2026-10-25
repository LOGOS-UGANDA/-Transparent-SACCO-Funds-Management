# Design — SACCO Pivot: Roadmap Pull Request and Phase Issues

| | |
|---|---|
| **Date** | 2026-10-25 |
| **Status** | Approved, with amendments of 2026-10-25 recorded under *Decisions* |
| **Classification** | Small change (documentation and project management only) |
| **Selected approach** | A — documentation-only pull request, one issue per phase |

## 1. Background and intent

Logos Circle Kampala has decided to pivot this repository's product from university fee transparency to a SACCO ledger: deposits, loans, and expenses recorded and approved on chain, with individual member balances kept private and institutional totals public. The pivot roadmap currently exists only as an externally generated web page; it carries no authority inside the repository and offers the team no way to claim or track work.

This change establishes the pivot inside the repository. It lands the roadmap as reviewed documentation and opens structured, labeled issues so that contributors can take ownership of specific phases.

The roadmap is expressed as **twelve steps across eight phases**. Sequencing — not scheduling — carries the plan: the documents deliberately contain no durations, calendar dates, or other time commitments.

## 2. Objectives

1. Publish the complete pivot roadmap at `docs/roadmap-sacco-pivot.md`: problem statement, architecture, the eight phases with tasks and exit criteria, data model, metrics, workstreams, and open questions.
2. Announce the pivot in the README with a link to the roadmap, keeping the earlier university-fee material as historical reference.
3. Enable the GitHub issue tracker and establish four workstream labels: `lez`, `backend`, `frontend`, `community`.
4. Open issues for the first three phases, assigned to the repository lead, with the remaining phases deferred until requested.

## 3. Scope

**In scope.** New roadmap document; README pivot notice; issue-tracker enablement; four workstream labels; issues for Phases 0–2.

**Out of scope.** Any change to program code, modules, artifacts, or tooling. In particular, the rename of `fee_transparency` to `sacco_ledger` is Phase 1 work for the team, not part of this change.

**Explicitly avoided.** Front-loading Phase 1 implementation into the documentation pull request; fragmenting the roadmap into per-task issues.

## 4. Constraints

- Schedule language is prohibited in all deliverables: no durations or dates. Existing phrasing is reworded during adaptation (for example, "Weeks 2–4" becomes "Steps 2–4"; "12 weeks" becomes "12 steps").
- All GitHub CLI operations run against the stored credentials of `YodHeVauHe`, with conflicting `GH_TOKEN` environment variables unset.
- Enabling the issue tracker requires repository administrator rights, which the stored token does not possess; the setting must be switched on by an administrator.

## 5. Approaches considered

| Approach | Description | Assessment |
|---|---|---|
| **A — documentation-only pull request plus per-phase issues** | Roadmap and README on a dedicated branch; one issue per phase with checklists, exit criteria, and workstream labels. | **Selected.** Minimal, reviewable diff; every unit of implementation work reaches the team as an assignable issue. |
| B — plan and program rename in one pull request | Approach A plus renaming `programs/fee_transparency` to `programs/sacco_ledger` and updating references. | Rejected. Anticipates Phase 1, inflates the diff, and removes work intended for the team. |
| C — specification set plus per-workstream issues | Fuller design and planning documents; five issues organized by workstream rather than phase. | Rejected. Phases span workstreams, so workstream-level issues blur ownership and sequencing. |

## 6. Decisions

Decisions below marked *amendment* were made by the repository lead after initial approval and supersede the corresponding earlier choices.

1. Issue titles follow `Phase N: <name> (Steps X–Y)`, singular for single-step phases.
2. Roadmap statuses are preserved as checkbox states, with provenance notes stripped of dates.
3. The design and plan documents travel on the same branch as the roadmap.
4. *Amendment:* Issues for **Phases 0–2 are assigned to `YodHeVauHe`** and created first; issues for **Phases 3–7 are deferred** until the lead requests them. The original design created all eight unassigned.
5. *Amendment (PR review, 2026-10-25):* oversight is separated from approval — the Phase 1 data model gains a **Supervisory Committee** account alongside the Committee, and approver thresholds and role composition become **per-SACCO configuration**.
6. *Amendment (PR review, 2026-10-25):* the custody question is settled toward **attestation**: the program is the record, not a custodian. Value stays off chain in mobile money and bank accounts; the vault-custody code remains in the repository but dormant. `ProposeExpense` / `ExecuteExpense` are documented as recording approval and release of off-chain funds.
7. *Amendment (PR review, 2026-10-25):* the `lee_core` v0.3 bump and a hello-world deploy move to the **front of Phase 1**, so version incompatibility surfaces before domain modelling; Phase 2 retains the tooling switch and the sequencer/testnet deploys.
8. *Amendment (PR review, 2026-10-25):* investments are stated **out of scope for v1**; five reviewer questions are adopted as roadmap open questions (deposit types at Phase 1 kickoff; member-exit and beneficiary recovery at Phase 4; pilot reachability; URSB record-keeping obligations; auditor and Supervisory Committee visibility into private balances).

## 7. Deliverables

| Deliverable | Location |
|---|---|
| Pivot roadmap | `docs/roadmap-sacco-pivot.md` |
| README pivot notice | `README.md` |
| Pull request | `sacco-pivot-plan` → `main`, titled "SACCO pivot: roadmap and 12-step plan" |
| Workstream labels | `lez` `1D76DB` · `backend` `0E8A16` · `frontend` `5319E7` · `community` `D93F0B` |
| Phase issues | Phases 0–2 assigned to `YodHeVauHe`; Phases 3–7 deferred |

## 8. Reference

- Implementation plan: `docs/superpowers/plans/2026-10-25-sacco-pivot-plan-and-issues.md`
