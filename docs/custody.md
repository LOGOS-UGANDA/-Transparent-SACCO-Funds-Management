# Superseded fee-custody proposal (historical)

> **Not the current product design.** This file records an earlier proposal to hold and transfer fee tokens. That proposal was superseded by the SACCO attestation ledger described in the [main README](../README.md) and [architecture](./architecture.md). The current program does not receive, hold, or transfer SACCO money. Deposits, loan disbursements, repayments, and expense payments stay off chain; LEZ stores the associated records and approvals.

The details below are retained only as project history. They do not describe implemented or planned `sacco_ledger` behavior.

## Historical proposal

## Goal

Payers’ funds must be **actually locked** on LEZ (not only recorded in a ledger), and releases must move tokens only after multi-sig approval for a stated purpose.

## Chosen model: institution vault + category ledger

```text
Payer token holding
        │  PayFees (Token::Transfer chained call)
        ▼
Institution vault (PDA token holding)
        │  ledger split across FeeCategory accounts (basis points)
        │  ExecuteRelease (Token::Transfer + vault PDA authority)
        ▼
Treasury token holding
```

| Piece | Role |
|-------|------|
| `ProgramConfig` (PDA `"CONFIG"`) | Stores the Token Program id used for chained calls |
| Institution vault (PDA) | Single token holding per `(institution, token_definition)` that custodies locked fees |
| `FeeCategory` accounts | Public ledger: allocation %, collected, released |
| `Payment` / `ReleaseRequest` | Public audit trail |

### Why not per-category vaults (v1)?

Per-category vaults give stronger physical segregation, but SPEL guest instructions need a fixed account list. An institution vault plus enforced category ledgers still prevents over-release per category, keeps one custody transfer per pay/release, and matches AMM-style PDA vault patterns on Logos.

Per-category vaults remain a possible v2 upgrade.

## PDA derivation

Seeds are hashed with RISC Zero SHA-256 (same pattern as ATA / AMM):

| PDA | Seed material |
|-----|----------------|
| Config | `b"CONFIG"` |
| Institution vault | `institution_id \|\| token_definition_id` (32 + 32 bytes) |

Vault address: `AccountId::for_public_pda(sacco_ledger_program_id, vault_seed)`.

## Instructions (custody-related)

1. **`Initialize`** — create config PDA; set `token_program_id`.
2. **`InitializeInstitutionVault`** — bind `token_definition_id` + `vault_id` on the institution; vault holding may be created on first inbound transfer (token program + PDA auth), same idea as AMM pool vaults.
3. **`PayFees`** — update institution/category/payment ledgers; chain `Token::Transfer` from payer holding → vault for the full payment amount.
4. **`ExecuteRelease`** — after multi-sig approval, debit category/institution ledgers; chain `Token::Transfer` from vault → treasury holding, authorized with vault PDA seeds.

## Guest account limits (v1)

`PayFees` guest entry accepts up to **8** category accounts (`category_0` … `category_7`). Unused slots must be default accounts and are skipped. Institutions with more than eight active categories need a later protocol change (or batched allocation).

## Invariants

- Cannot `PayFees` until the institution is verified and has an initialized vault.
- Category basis points across active categories must not exceed 10_000.
- `ExecuteRelease` amount ≤ category `total_collected - total_released`.
- Vault account id must always match the PDA for `(institution, token_definition)`.
- Token Program id always comes from `ProgramConfig`, never from a caller-trusted holding alone.

## Out of scope (next)

- Refund path with reverse transfer
- Multi-token institutions (one vault per token definition is supported; UI may start with one fee token)
- IDL clients / `sacco_core` wallet wiring (see `docs/idl.md`)
