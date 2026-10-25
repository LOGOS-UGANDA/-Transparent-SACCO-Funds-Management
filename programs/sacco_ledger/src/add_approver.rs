use sacco_ledger_core::Sacco;
use nssa_core::{
    account::{AccountId, AccountWithMetadata, BalanceDiff},
    program::AccountStateDiff,
};

/// Add a multi-sig approver. Admin only; duplicates rejected.
pub fn add_approver(
    mut sacco_account: AccountWithMetadata,
    admin: AccountWithMetadata,
    approver: AccountId,
) -> Vec<AccountStateDiff> {
    assert!(admin.is_authorized, "Admin must authorize");

    let mut sacco = Sacco::try_from(&sacco_account.account.data)
        .expect("Sacco account must hold valid Sacco data");
    assert_eq!(
        sacco.admin, admin.account_id,
        "Only the SACCO admin can add approvers"
    );
    assert!(
        !sacco.approvers.contains(&approver),
        "Approver already registered"
    );
    assert!(
        !sacco.supervisory_committee.contains(&approver),
        "Supervisory Committee members cannot also be approvers"
    );

    sacco.approvers.push(approver);
    let pre = sacco_account.clone();
    sacco_account.account.data = (&sacco).into();
    let post_data = sacco_account.account.data;

    vec![
        AccountStateDiff::new(pre, BalanceDiff::Add(0), post_data),
        AccountStateDiff::unchanged(admin),
    ]
}
