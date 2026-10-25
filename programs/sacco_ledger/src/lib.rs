//! Transparent SACCO ledger program logic — the attestation model.
//!
//! Pure state-transition handlers over `AccountStateDiff`s — no I/O. Guest entrypoints
//! live under `methods/guest`. Value never moves on chain: deposits, loans and expenses
//! are recorded and approved; the chain is the tamper-evident record.
//!
//! The earlier custody handlers live (dormant) under [`custody`].

pub use sacco_ledger_core as core;

pub mod add_approver;
pub mod add_member;
pub mod add_supervisor;
pub mod apply_loan;
pub mod approve_loan;
pub mod confirm_deposit;
pub mod disburse_loan;
pub mod record_deposit;
pub mod record_repayment;
pub mod reject_loan;
pub mod custody;
pub mod initialize;
pub mod register_sacco;
pub mod verify_sacco;

use core::Sacco;
use nssa_core::account::AccountId;

/// True when `id` is one of the SACCO's multi-sig approvers.
#[must_use]
pub fn is_approver(sacco: &Sacco, id: &AccountId) -> bool {
    sacco.approvers.contains(id)
}
