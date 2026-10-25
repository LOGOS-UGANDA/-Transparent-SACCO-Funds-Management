use sacco_ledger_core::Deposit;
use nssa_core::{
    account::{AccountWithMetadata, BalanceDiff},
    program::AccountStateDiff,
};

/// Member confirmation of a recorded deposit. Only the deposit's member may confirm.
pub fn confirm_deposit(
    mut deposit_account: AccountWithMetadata,
    member: AccountWithMetadata,
) -> Vec<AccountStateDiff> {
    assert!(member.is_authorized, "Member must authorize confirmation");

    let mut deposit = Deposit::try_from(&deposit_account.account.data)
        .expect("Deposit account must hold valid Deposit data");
    assert_eq!(
        deposit.member_id, member.account_id,
        "Only the deposit's member can confirm"
    );
    assert!(
        !deposit.confirmed_by_member,
        "Deposit already confirmed"
    );

    deposit.confirmed_by_member = true;
    let pre = deposit_account.clone();
    deposit_account.account.data = (&deposit).into();
    let post_data = deposit_account.account.data;

    vec![AccountStateDiff::new(pre, BalanceDiff::Add(0), post_data)]
}
