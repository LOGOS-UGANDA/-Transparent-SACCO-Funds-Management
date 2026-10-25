use sacco_ledger_core::Institution;
use nssa_core::{
    account::{AccountId, AccountWithMetadata, BalanceDiff},
    program::AccountStateDiff,
};

pub fn add_approver(
    mut institution_account: AccountWithMetadata,
    admin: AccountWithMetadata,
    approver: AccountId,
) -> Vec<AccountStateDiff> {
    assert!(admin.is_authorized, "Admin must authorize");

    let mut institution = Institution::try_from(&institution_account.account.data)
        .expect("Institution account must hold valid Institution data");
    assert_eq!(
        institution.admin, admin.account_id,
        "Only institution admin can add approvers"
    );
    assert!(
        !institution.approvers.contains(&approver),
        "Approver already registered"
    );

    institution.approvers.push(approver);
    let pre = institution_account.clone();
    institution_account.account.data = (&institution).into();
    let post_data = institution_account.account.data;

    vec![
        AccountStateDiff::new(pre, BalanceDiff::Add(0), post_data),
        AccountStateDiff::unchanged(admin),
    ]
}
