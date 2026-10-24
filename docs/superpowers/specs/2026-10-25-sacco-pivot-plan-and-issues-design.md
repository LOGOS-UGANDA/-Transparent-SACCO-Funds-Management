# SACCO pivot: roadmap PR + phase issues — design

**Date:** 2026-10-25 · **Size:** small change · **Approach:** A (docs-only PR + one issue per phase)

## Intent and why

The pivot from university fee transparency to a SACCO ledger is decided (Logos Circle Kampala roadmap, currently Phase 0). The team needs the plan living in the repo, not on a Grok page, and needs assignable work items. This change lands the roadmap as docs on a branch/PR and opens one issue per phase.

The plan is worded as **12 steps, not a 12-week calendar**: same order, same groupings, no time commitment. The 8 phases (0–7) map onto steps (Phase 0 = Step 1 … Phase 7 = Steps 10–12).

## Goals

- Branch `sacco-pivot-plan` off `origin/main` with a docs-only PR:
  - `docs/roadmap-sacco-pivot.md` — the roadmap adapted for the repo: problem, architecture, the 8 phases (0–7) with tasks and exit criteria, data model, metrics, workstreams, open questions — all worded as 12 steps with no week or date framing.
  - README: short "Pivot to SACCO ledger" note pointing at the roadmap doc.
- Enable Issues on the repo (currently disabled).
- Create 8 issues — one per phase — each with its task checklist, "Done when" exit criterion, and a workstream label.
- New labels: `lez`, `backend`, `frontend`, `community` (only GitHub defaults exist today).

## Non-goals

- No code changes in this PR. The `fee_transparency` → `sacco_ledger` rename and everything downstream stays team work.
- No milestones, no project board, no per-task issues.

## Anti-goals

- Front-running Phase 1 work inside the plan PR.
- Issue spam: ~40 roadmap tasks as ~40 separate issues.

## Constraints

- `gh` CLI: run with `GH_TOKEN` unset; the stored `YodHeVauHe` credentials are valid.
- Enabling Issues needs repo admin. Try via API first; if forbidden, YodHeVauHe flips Settings → General → Features.

## Decisions (mine unless corrected)

- Issues are created **unassigned** — no phase→person mapping was given. Assignment happens in the team.
- Issue titles: `Phase N: <name> (Steps X–Y)`, e.g. `Phase 1: Domain model and LEZ program (Steps 2–4)`.
- Week-based scheduling is replaced by step ordering everywhere: "Weeks 2–4" → "Steps 2–4"; "12 weeks, from 26 Oct 2026" → "12 steps". No dates appear in the roadmap or issues.
- Roadmap task statuses as of 24 Oct 2026 are preserved as ticked checkboxes in issue bodies.
- Design doc rides on the same branch.

## Left to the builder

- Exact wording of the roadmap doc and issue bodies (source: the Grok page content).
- PR title/description and label colors.
