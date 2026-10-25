use crate::is_approver;
use sacco_ledger_core::{Deposit, DepositType, Member, Sacco};
use nssa_core::{
    account::{Account, AccountId, AccountWithMetadata, BalanceDiff},
    program::AccountStateDiff,
};

/// Record an off-chain deposit for a member (approver attests; the member confirms later).
/// Totals and balances move strictly within the deposit's type.
#[allow(clippy::too_many_arguments)]
pub fn record_deposit(
    mut sacco_account: AccountWithMetadata,
    mut member_account: AccountWithMetadata,
    mut deposit_account: AccountWithMetadata,
    recorder: AccountWithMetadata,
    amount: u128,
    deposit_type: DepositType,
    reference: String,
) -> Vec<AccountStateDiff> {
    assert!(recorder.is_authorized, "Recorder must authorize");
    assert!(
        deposit_account.account == Account::default(),
        "Deposit account must be uninitialized"
    );
    assert!(deposit_account.is_authorized, "Deposit account must be authorized");

    let mut sacco = Sacco::try_from(&sacco_account.account.data)
        .expect("Sacco account must hold valid Sacco data");
    assert!(sacco.is_verified, "Sacco is not verified");
    assert!(
        is_approver(&sacco, &recorder.account_id),
        "Only SACCO approvers can record deposits"
    );
    assert!(amount > 0, "Deposit amount must be greater than 0");
    assert!(
        !reference.is_empty(),
        "Deposit reference must not be empty"
    );

    let mut member = Member::try_from(&member_account.account.data)
        .expect("Member account must hold valid Member data");
    assert_eq!(
        member.sacco_id, sacco_account.account_id,
        "Member belongs to a different SACCO"
    );

    let deposit = Deposit {
        sacco_id: sacco_account.account_id,
        member_id: member.identity,
        amount,
        deposit_type,
        reference,
        recorded_by: recorder.account_id,
        confirmed_by_member: false,
    };

    match deposit_type {
        DepositType::Savings => {
            member.savings_balance = member
                .savings_balance
                .checked_add(amount)
                .expect("savings balance overflow");
            sacco.total_deposits_savings = sacco
                .total_deposits_savings
                .checked_add(amount)
                .expect("savings total overflow");
        }
        DepositType::ShareCapital => {
            member.share_capital_balance = member
                .share_capital_balance
                .checked_add(amount)
                .expect("share capital balance overflow");
            sacco.total_deposits_share_capital = sacco
                .total_deposits_share_capital
                .checked_add(amount)
                .expect("share capital total overflow");
        }
    }

    let sacco_pre = sacco_account.clone();
    sacco_account.account.data = (&sacco).into();
    let sacco_data = sacco_account.account.data;

    let member_pre = member_account.clone();
    member_account.account.data = (&member).into();
    let member_data = member_account.account.data;

    let deposit_pre = deposit_account.clone();
    deposit_account.account.data = (&deposit).into();
    let deposit_data = deposit_account.account.data;

    vec![
        AccountStateDiff::new(sacco_pre, BalanceDiff::Add(0), sacco_data),
        AccountStateDiff::new(member_pre, BalanceDiff::Add(0), member_data),
        AccountStateDiff::new(deposit_pre, BalanceDiff::Add(0), deposit_data),
        AccountStateDiff::unchanged(recorder),
    ]
}
