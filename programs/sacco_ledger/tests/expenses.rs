//! Expense integration tests: propose, multi-sig approve, recorded execution.

use nssa_core::account::{Account, AccountId, AccountWithMetadata};
use sacco_ledger_program::{
    add_approver::add_approver, approve_expense::approve_expense, execute_expense::execute_expense,
    propose_expense::propose_expense, register_sacco::register_sacco,
    reject_expense::reject_expense, verify_sacco::verify_sacco,
};
use sacco_ledger_core::{Expense, ExpenseStatus, Sacco};

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

/// Verified SACCO (threshold 2, approvers = admin(2) + second(3)).
fn verified_two_approver_sacco() -> (AccountWithMetadata, AccountWithMetadata, AccountWithMetadata) {
    let mut sacco_account = blank_account(1);
    let admin = blank_account(2);
    let second = blank_account(3);
    let diffs = register_sacco(
        sacco_account.clone(),
        admin.clone(),
        "Kampala Traders SACCO".into(),
        "UG-SACCO-001".into(),
        aid(9),
    );
    sacco_account.account.data = post_of(&diffs, 0);
    let diffs = add_approver(sacco_account.clone(), admin.clone(), aid(3));
    sacco_account.account.data = post_of(&diffs, 0);
    let mut sacco = Sacco::try_from(&sacco_account.account.data).expect("sacco");
    sacco.required_approvals = 2;
    sacco_account.account.data = (&sacco).into();
    let diffs = verify_sacco(sacco_account.clone(), admin.clone());
    sacco_account.account.data = post_of(&diffs, 0);
    (sacco_account, admin, second)
}

fn approved_expense() -> (AccountWithMetadata, AccountWithMetadata, AccountWithMetadata, AccountWithMetadata) {
    let (sacco_account, admin, second) = verified_two_approver_sacco();
    let expense_account = blank_account(4);
    let diffs = propose_expense(
        sacco_account.clone(),
        expense_account,
        admin.clone(),
        75_000,
        "Stationery for the AGM".into(),
        "bafy-expense".into(),
    );
    let mut expense_account = blank_account(4);
    expense_account.account.data = post_of(&diffs, 1);
    let diffs = approve_expense(sacco_account.clone(), expense_account.clone(), admin.clone());
    expense_account.account.data = post_of(&diffs, 1);
    let diffs = approve_expense(sacco_account.clone(), expense_account.clone(), second.clone());
    expense_account.account.data = post_of(&diffs, 1);
    (sacco_account, expense_account, admin, second)
}

#[test]
#[should_panic(expected = "Sacco is not verified")]
fn propose_expense_rejected_for_unverified_sacco() {
    let mut sacco_account = blank_account(1);
    let admin = blank_account(2);
    let diffs = register_sacco(
        sacco_account.clone(),
        admin,
        "Kampala Traders SACCO".into(),
        "UG-SACCO-001".into(),
        aid(9),
    );
    sacco_account.account.data = post_of(&diffs, 0);
    let _ = propose_expense(
        sacco_account,
        blank_account(4),
        blank_account(2),
        75_000,
        "Stationery".into(),
        "bafy".into(),
    );
}

#[test]
#[should_panic(expected = "Not a SACCO approver")]
fn propose_expense_requires_approver() {
    let (sacco_account, _admin, _second) = verified_two_approver_sacco();
    let _ = propose_expense(
        sacco_account,
        blank_account(4),
        blank_account(8),
        75_000,
        "Stationery".into(),
        "bafy".into(),
    );
}

#[test]
#[should_panic(expected = "Expense purpose must not be empty")]
fn propose_expense_rejects_empty_purpose() {
    let (sacco_account, admin, _second) = verified_two_approver_sacco();
    let _ = propose_expense(
        sacco_account,
        blank_account(4),
        admin,
        75_000,
        String::new(),
        "bafy".into(),
    );
}

#[test]
fn approve_expense_below_threshold_stays_pending() {
    let (sacco_account, admin, _second) = verified_two_approver_sacco();
    let expense_account = blank_account(4);
    let diffs = propose_expense(
        sacco_account.clone(),
        expense_account,
        admin.clone(),
        75_000,
        "Stationery for the AGM".into(),
        "bafy-expense".into(),
    );
    let mut expense_account = blank_account(4);
    expense_account.account.data = post_of(&diffs, 1);
    let diffs = approve_expense(sacco_account, expense_account, admin);
    let expense = Expense::try_from(diffs[1].post_data.as_ref().expect("expense")).expect("exp");
    assert_eq!(expense.status, ExpenseStatus::Proposed, "below threshold stays proposed");
    assert_eq!(expense.approvers.len(), 1);
}

#[test]
fn approve_expense_executes_at_exact_threshold() {
    let (sacco_account, admin, second) = verified_two_approver_sacco();
    let expense_account = blank_account(4);
    let diffs = propose_expense(
        sacco_account.clone(),
        expense_account,
        admin.clone(),
        75_000,
        "Stationery for the AGM".into(),
        "bafy-expense".into(),
    );
    let mut expense_account = blank_account(4);
    expense_account.account.data = post_of(&diffs, 1);
    let diffs = approve_expense(sacco_account.clone(), expense_account.clone(), admin.clone());
    expense_account.account.data = post_of(&diffs, 1);
    let diffs = approve_expense(sacco_account, expense_account, second);
    let expense = Expense::try_from(diffs[1].post_data.as_ref().expect("expense")).expect("exp");
    assert_eq!(expense.status, ExpenseStatus::Approved);
    assert_eq!(expense.approvers.len(), 2);
}

#[test]
#[should_panic(expected = "Already approved by this approver")]
fn approve_expense_duplicate_approver_rejected() {
    let (sacco_account, admin, _second) = verified_two_approver_sacco();
    let expense_account = blank_account(4);
    let diffs = propose_expense(
        sacco_account.clone(),
        expense_account,
        admin.clone(),
        75_000,
        "Stationery for the AGM".into(),
        "bafy-expense".into(),
    );
    let mut expense_account = blank_account(4);
    expense_account.account.data = post_of(&diffs, 1);
    let diffs = approve_expense(sacco_account.clone(), expense_account.clone(), admin.clone());
    expense_account.account.data = post_of(&diffs, 1);
    let _ = approve_expense(sacco_account, expense_account, admin);
}

#[test]
fn reject_expense_approver_only() {
    let (sacco_account, admin, second) = verified_two_approver_sacco();
    let expense_account = blank_account(4);
    let diffs = propose_expense(
        sacco_account.clone(),
        expense_account,
        admin,
        75_000,
        "Stationery for the AGM".into(),
        "bafy-expense".into(),
    );
    let mut expense_account = blank_account(4);
    expense_account.account.data = post_of(&diffs, 1);
    let diffs = reject_expense(sacco_account, expense_account, second);
    let expense = Expense::try_from(diffs[1].post_data.as_ref().expect("expense")).expect("exp");
    assert_eq!(expense.status, ExpenseStatus::Rejected);
}

#[test]
fn execute_expense_records_off_chain_release() {
    let (mut sacco_account, expense_account, admin, _second) = approved_expense();
    let before = Sacco::try_from(&sacco_account.account.data).expect("sacco");
    let diffs = execute_expense(
        sacco_account.clone(),
        expense_account,
        admin,
        "bafy-payment-proof".into(),
    );
    let after = Sacco::try_from(diffs[0].post_data.as_ref().expect("sacco")).expect("sacco");
    assert_eq!(after.total_expenses, before.total_expenses + 75_000);
    let expense = Expense::try_from(diffs[1].post_data.as_ref().expect("expense")).expect("exp");
    assert_eq!(expense.status, ExpenseStatus::Executed);
    assert_eq!(expense.executed_evidence.as_deref(), Some("bafy-payment-proof"));
}
