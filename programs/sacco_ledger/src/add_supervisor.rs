use sacco_ledger_core::Sacco;
use nssa_core::{
    account::{AccountId, AccountWithMetadata, BalanceDiff},
    program::AccountStateDiff,
};

/// Add a Supervisory Committee member (oversight, separate from approvers). Admin only.
pub fn add_supervisor(
    mut sacco_account: AccountWithMetadata,
    admin: AccountWithMetadata,
    supervisor: AccountId,
) -> Vec<AccountStateDiff> {
    assert!(admin.is_authorized, "Admin must authorize");

    let mut sacco = Sacco::try_from(&sacco_account.account.data)
        .expect("Sacco account must hold valid Sacco data");
    assert_eq!(
        sacco.admin, admin.account_id,
        "Only the SACCO admin can add supervisors"
    );
    assert!(
        !sacco.supervisory_committee.contains(&supervisor),
        "Supervisor already registered"
    );
    assert!(
        !sacco.approvers.contains(&supervisor),
        "Approvers cannot also sit on the Supervisory Committee"
    );

    sacco.supervisory_committee.push(supervisor);
    let pre = sacco_account.clone();
    sacco_account.account.data = (&sacco).into();
    let post_data = sacco_account.account.data;

    vec![
        AccountStateDiff::new(pre, BalanceDiff::Add(0), post_data),
        AccountStateDiff::unchanged(admin),
    ]
}
