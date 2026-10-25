use sacco_ledger_core::custody::Institution;
use nssa_core::{
    account::{AccountWithMetadata, BalanceDiff},
    program::AccountStateDiff,
};

/// Mark institution verified. Scaffold: any authorized registrar; tighten auth later.
pub fn verify_institution(
    mut institution_account: AccountWithMetadata,
    registrar: AccountWithMetadata,
) -> Vec<AccountStateDiff> {
    assert!(registrar.is_authorized, "Registrar must authorize verification");

    let mut institution = Institution::try_from(&institution_account.account.data)
        .expect("Institution account must hold valid Institution data");
    assert!(institution.is_active, "Institution must be active");
    institution.is_verified = true;

    let pre = institution_account.clone();
    institution_account.account.data = (&institution).into();
    let post_data = institution_account.account.data;

    vec![
        AccountStateDiff::new(pre, BalanceDiff::Add(0), post_data),
        AccountStateDiff::unchanged(registrar),
    ]
}
