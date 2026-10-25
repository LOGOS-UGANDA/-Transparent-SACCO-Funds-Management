//! Deposit integration tests: typed totals, verification gating, member confirmation.

use nssa_core::account::{Account, AccountId, AccountWithMetadata};
use sacco_ledger_program::{
    add_member::add_member, confirm_deposit::confirm_deposit, register_sacco::register_sacco,
    verify_sacco::verify_sacco,
};
use sacco_ledger_core::{Deposit, DepositType, Member, Sacco};

fn blank_account(id: u8) -> AccountWithMetadata {
    AccountWithMetadata {
        account_id: AccountId::new([id; 32]),
        account: Account::default(),
        is_authorized: true,
    }
}

fn aid(n: u8) -> AccountId {
    AccountId::new([n; 32])
}

fn post_of(diffs: &[nssa_core::program::AccountStateDiff], i: usize) -> nssa_core::account::Data {
    diffs[i].post_data.as_ref().expect("post data").clone()
}

/// A verified SACCO with one member (identity = aid(6), member account id 6).
fn verified_sacco_with_member() -> (AccountWithMetadata, AccountWithMetadata, AccountWithMetadata) {
    let mut sacco_account = blank_account(1);
    let admin = blank_account(2);
    let member_account = blank_account(6);
    let diffs = register_sacco(
        sacco_account.clone(),
        admin.clone(),
        "Kampala Traders SACCO".into(),
        "UG-SACCO-001".into(),
        aid(9),
    );
    sacco_account.account.data = post_of(&diffs, 0);
    let diffs = add_member(sacco_account.clone(), member_account, admin, aid(6));
    sacco_account.account.data = post_of(&diffs, 0);
    let mut member_account = blank_account(6);
    member_account.account.data = post_of(&diffs, 1);
    // verify (admin is the v1 registrar)
    let admin = blank_account(2);
    let diffs = verify_sacco(sacco_account.clone(), admin);
    sacco_account.account.data = post_of(&diffs, 0);
    (sacco_account, member_account, blank_account(2))
}

#[test]
fn record_deposit_updates_member_and_sacco_totals() {
    let (sacco_account, member_account, treasurer) = verified_sacco_with_member();
    let deposit_account = blank_account(7);
    let diffs = sacco_ledger_program::record_deposit::record_deposit(
        sacco_account.clone(),
        member_account.clone(),
        deposit_account,
        treasurer,
        aid(6),
        100_000,
        DepositType::Savings,
        "MPZA-1001".into(),
    );
    let sacco = Sacco::try_from(diffs[0].post_data.as_ref().expect("sacco")).expect("sacco");
    assert_eq!(sacco.total_deposits_savings, 100_000);
    assert_eq!(sacco.total_deposits_share_capital, 0);
    let member = Member::try_from(diffs[1].post_data.as_ref().expect("member")).expect("member");
    assert_eq!(member.savings_balance, 100_000);
    assert_eq!(member.share_capital_balance, 0);
    let deposit = Deposit::try_from(diffs[2].post_data.as_ref().expect("deposit")).expect("dep");
    assert_eq!(deposit.amount, 100_000);
    assert_eq!(deposit.deposit_type, DepositType::Savings);
    assert_eq!(deposit.reference, "MPZA-1001");
    assert!(!deposit.confirmed_by_member);
}

#[test]
fn record_deposit_splits_totals_by_type() {
    let (sacco_account, member_account, treasurer) = verified_sacco_with_member();
    let deposit_account = blank_account(7);
    let diffs = sacco_ledger_program::record_deposit::record_deposit(
        sacco_account.clone(),
        member_account.clone(),
        deposit_account,
        treasurer,
        aid(6),
        40_000,
        DepositType::ShareCapital,
        "MPZA-1002".into(),
    );
    let sacco = Sacco::try_from(diffs[0].post_data.as_ref().expect("sacco")).expect("sacco");
    assert_eq!(sacco.total_deposits_share_capital, 40_000, "share capital moves only its own total");
    assert_eq!(sacco.total_deposits_savings, 0, "savings total untouched");
    let member = Member::try_from(diffs[1].post_data.as_ref().expect("member")).expect("member");
    assert_eq!(member.share_capital_balance, 40_000);
    assert_eq!(member.savings_balance, 0);
}

#[test]
#[should_panic(expected = "Sacco is not verified")]
fn record_deposit_rejected_for_unverified_sacco() {
    let mut sacco_account = blank_account(1);
    let admin = blank_account(2);
    let member_account = blank_account(6);
    let diffs = register_sacco(
        sacco_account.clone(),
        admin,
        "Kampala Traders SACCO".into(),
        "UG-SACCO-001".into(),
        aid(9),
    );
    sacco_account.account.data = post_of(&diffs, 0);
    let _ = sacco_ledger_program::record_deposit::record_deposit(
        sacco_account,
        member_account,
        blank_account(7),
        blank_account(3),
        aid(6),
        100_000,
        DepositType::Savings,
        "MPZA-1003".into(),
    );
}

#[test]
#[should_panic(expected = "Only SACCO approvers can record deposits")]
fn record_deposit_requires_approver() {
    let (sacco_account, member_account, _treasurer) = verified_sacco_with_member();
    let outsider = blank_account(8); // not in approvers
    let _ = sacco_ledger_program::record_deposit::record_deposit(
        sacco_account,
        member_account,
        blank_account(7),
        outsider,
        aid(6),
        100_000,
        DepositType::Savings,
        "MPZA-1004".into(),
    );
}

#[test]
#[should_panic(expected = "Deposit reference must not be empty")]
fn record_deposit_rejects_empty_reference() {
    let (sacco_account, member_account, treasurer) = verified_sacco_with_member();
    let _ = sacco_ledger_program::record_deposit::record_deposit(
        sacco_account,
        member_account,
        blank_account(7),
        treasurer,
        aid(6),
        100_000,
        DepositType::Savings,
        String::new(),
    );
}

#[test]
fn confirm_deposit_flips_member_confirmation() {
    let (sacco_account, member_account, treasurer) = verified_sacco_with_member();
    let diffs = sacco_ledger_program::record_deposit::record_deposit(
        sacco_account,
        member_account,
        blank_account(7),
        treasurer,
        aid(6),
        100_000,
        DepositType::Savings,
        "MPZA-1005".into(),
    );
    let mut deposit_account = blank_account(7);
    deposit_account.account.data = post_of(&diffs, 2);
    let member_signer = blank_account(6); // identity matches deposit.member_id
    let confirm_diffs = confirm_deposit(deposit_account, member_signer);
    let deposit =
        Deposit::try_from(confirm_diffs[0].post_data.as_ref().expect("deposit")).expect("dep");
    assert!(deposit.confirmed_by_member);
}

#[test]
#[should_panic(expected = "Only the deposit's member can confirm")]
fn confirm_deposit_member_only() {
    let (sacco_account, member_account, treasurer) = verified_sacco_with_member();
    let diffs = sacco_ledger_program::record_deposit::record_deposit(
        sacco_account,
        member_account,
        blank_account(7),
        treasurer,
        aid(6),
        100_000,
        DepositType::Savings,
        "MPZA-1006".into(),
    );
    let mut deposit_account = blank_account(7);
    deposit_account.account.data = post_of(&diffs, 2);
    let outsider = blank_account(8);
    let _ = confirm_deposit(deposit_account, outsider);
}

#[test]
#[should_panic(expected = "Deposit already confirmed")]
fn confirm_deposit_rejects_double_confirm() {
    let (sacco_account, member_account, treasurer) = verified_sacco_with_member();
    let diffs = sacco_ledger_program::record_deposit::record_deposit(
        sacco_account,
        member_account,
        blank_account(7),
        treasurer,
        aid(6),
        100_000,
        DepositType::Savings,
        "MPZA-1007".into(),
    );
    let mut deposit_account = blank_account(7);
    deposit_account.account.data = post_of(&diffs, 2);
    let member_signer = blank_account(6);
    let confirm_diffs = confirm_deposit(deposit_account.clone(), member_signer.clone());
    deposit_account.account.data = post_of(&confirm_diffs, 0);
    let _ = confirm_deposit(deposit_account, member_signer);
}
