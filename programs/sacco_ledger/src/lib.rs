//! Transparent SACCO fee-transparency program logic.
//!
//! Pure state-transition helpers — no I/O. Guest entrypoints live under `methods/guest`.
//! Custody design: repository `docs/custody.md`.

pub use sacco_ledger_core as core;

pub mod add_approver;
pub mod approve_release;
pub mod create_fee_category;
pub mod execute_release;
pub mod initialize;
pub mod initialize_institution_vault;
pub mod pay_fees;
pub mod register_institution;
pub mod reject_release;
pub mod request_release;
pub mod verify_institution;

#[cfg(test)]
mod tests;
