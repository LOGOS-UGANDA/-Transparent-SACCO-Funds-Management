use sacco_ledger_core::Sacco;
use nssa_core::{
    account::{AccountWithMetadata, BalanceDiff},
    program::AccountStateDiff,
};

/// Mark a SACCO verified. v1: the SACCO admin acts as registrar; protocol-level
/// registrar authority is tightened when the registrar account exists.
pub fn verify_sacco(
    mut sacco_account: AccountWithMetadata,
    registrar: AccountWithMetadata,
) -> Vec<AccountStateDiff> {
    assert!(registrar.is_authorized, "Registrar must authorize verification");

    let mut sacco = Sacco::try_from(&sacco_account.account.data)
        .expect("Sacco account must hold valid Sacco data");
    assert!(sacco.is_active, "Sacco must be active");
    assert_eq!(
        sacco.admin, registrar.account_id,
        "Only the SACCO admin can verify in v1"
    );
    sacco.is_verified = true;

    let pre = sacco_account.clone();
    sacco_account.account.data = (&sacco).into();
    let post_data = sacco_account.account.data;

    vec![AccountStateDiff::new(pre, BalanceDiff::Add(0), post_data)]
}
