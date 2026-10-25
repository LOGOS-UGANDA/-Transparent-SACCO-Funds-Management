//! Period summary integration tests.

use nssa_core::account::{Account, AccountId, AccountWithMetadata};
use sacco_ledger_program::{
    add_member::add_member, post_period_summary::post_period_summary,
    record_deposit::record_deposit, register_sacco::register_sacco, verify_sacco::verify_sacco,
};
use sacco_ledger_core::{DepositType, PeriodSummary, Sacco};

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
    diffs[i]
        .post_data
        .as_ref()
        .unwrap_or_else(|| panic!("post data missing at index {i}"))
        .clone()
}

/// Verified SACCO (admin approver) with a savings deposit of 100_000 recorded.
fn sacco_with_deposit() -> (AccountWithMetadata, AccountWithMetadata) {
    let mut sacco_account = blank_account(1);
    let admin = blank_account(2);
    let diffs = register_sacco(
        sacco_account.clone(),
        admin.clone(),
        "Kampala Traders SACCO".into(),
        "UG-SACCO-001".into(),
        aid(9),
    );
    sacco_account.account.data = post_of(&diffs, 0);
    let diffs = verify_sacco(sacco_account.clone(), admin.clone());
    sacco_account.account.data = post_of(&diffs, 0);

    // Register the member, then record a deposit against it.
    let member_account = blank_account(6);
    let diffs = add_member(sacco_account.clone(), member_account, admin.clone(), aid(6));
    sacco_account.account.data = post_of(&diffs, 0);

    let member_account = blank_account(6);
    let mut member_account2 = blank_account(6);
    member_account2.account.data = post_of(&diffs, 1);
    let deposit_account = blank_account(7);
    let diffs = record_deposit(
        sacco_account.clone(),
        member_account2,
        deposit_account,
        admin.clone(),
        aid(6),
        100_000,
        DepositType::Savings,
        "MPZA-3001".into(),
    );
    sacco_account.account.data = post_of(&diffs, 0);
    let _ = member_account;
    (sacco_account, admin)
}

#[test]
fn post_period_summary_snapshots_totals() {
    let (sacco_account, admin) = sacco_with_deposit();
    let diffs = post_period_summary(
        sacco_account.clone(),
        blank_account(8),
        admin,
        "2026-11".into(),
    );
    let summary =
        PeriodSummary::try_from(diffs[1].post_data.as_ref().expect("summary")).expect("summary");
    assert_eq!(summary.label, "2026-11");
    assert_eq!(summary.deposits_savings, 100_000);
    assert_eq!(summary.deposits_share_capital, 0);
    assert_eq!(summary.loans_disbursed, 0);
    assert_eq!(summary.expenses, 0);
}

#[test]
#[should_panic(expected = "Not a SACCO approver")]
fn post_period_summary_rejects_non_approver() {
    let (sacco_account, _admin) = sacco_with_deposit();
    let _ = post_period_summary(sacco_account, blank_account(8), blank_account(8), "2026-11".into());
}
