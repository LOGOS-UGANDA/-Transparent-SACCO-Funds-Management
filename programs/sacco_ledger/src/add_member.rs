use sacco_ledger_core::{Member, Sacco};
use nssa_core::{
    account::{Account, AccountId, AccountWithMetadata, BalanceDiff},
    program::AccountStateDiff,
};

/// Register a member identity on the SACCO. Admin only; duplicate identities rejected.
pub fn add_member(
    mut sacco_account: AccountWithMetadata,
    mut member_account: AccountWithMetadata,
    admin: AccountWithMetadata,
    identity: AccountId,
) -> Vec<AccountStateDiff> {
    assert!(admin.is_authorized, "Admin must authorize");
    assert!(
        member_account.account == Account::default(),
        "Member account must be uninitialized"
    );
    assert!(
        member_account.is_authorized,
        "Member account must be authorized"
    );

    let mut sacco = Sacco::try_from(&sacco_account.account.data)
        .expect("Sacco account must hold valid Sacco data");
    assert_eq!(
        sacco.admin, admin.account_id,
        "Only the SACCO admin can add members"
    );
    assert!(
        !sacco.member_ids.contains(&identity),
        "Member identity already registered"
    );

    sacco.member_ids.push(identity);

    let sacco_pre = sacco_account.clone();
    sacco_account.account.data = (&sacco).into();
    let sacco_data = sacco_account.account.data;

    let member = Member {
        sacco_id: sacco_pre.account_id,
        identity,
        savings_balance: 0,
        share_capital_balance: 0,
        loan_ids: vec![],
    };
    let member_pre = member_account.clone();
    member_account.account.data = (&member).into();
    let member_data = member_account.account.data;

    vec![
        AccountStateDiff::new(sacco_pre, BalanceDiff::Add(0), sacco_data),
        AccountStateDiff::new(member_pre, BalanceDiff::Add(0), member_data),
        AccountStateDiff::unchanged(admin),
    ]
}
