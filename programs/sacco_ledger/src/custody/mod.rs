//! DORMANT custody handlers — the earlier fee-transparency program, kept as
//! reference. Not dispatched by the sacco_ledger instruction set.

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
