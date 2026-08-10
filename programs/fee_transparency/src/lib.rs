//! Transparent SACCO fee-transparency program logic.
//!
//! Pure state-transition helpers — no I/O. Guest entrypoints live under `methods/guest`.

pub use fee_transparency_core as core;

pub mod add_approver;
pub mod approve_release;
pub mod create_fee_category;
pub mod execute_release;
pub mod pay_fees;
pub mod register_institution;
pub mod reject_release;
pub mod request_release;
pub mod verify_institution;

#[cfg(test)]
mod tests;
