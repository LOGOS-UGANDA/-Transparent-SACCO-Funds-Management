use sacco_ledger_core::{Expense, ExpenseStatus, Sacco};
use crate::is_approver;
use nssa_core::{
    account::{Account, AccountWithMetadata, BalanceDiff},
    program::AccountStateDiff,
};

/// Propose an expense against SACCO funds (approver attests the stated purpose).
pub fn propose_expense(
    sacco_account: AccountWithMetadata,
    mut expense_account: AccountWithMetadata,
    proposer: AccountWithMetadata,
    amount: u128,
    purpose: String,
    evidence_cid: String,
) -> Vec<AccountStateDiff> {
    assert!(proposer.is_authorized, "Proposer must authorize");
    assert!(
        expense_account.account == Account::default(),
        "Expense account must be uninitialized"
    );
    assert!(
        expense_account.is_authorized,
        "Expense account must be authorized"
    );

    let sacco = Sacco::try_from(&sacco_account.account.data)
        .expect("Sacco account must hold valid Sacco data");
    assert!(sacco.is_verified, "Sacco is not verified");
    assert!(
        is_approver(&sacco, &proposer.account_id),
        "Not a SACCO approver"
    );
    assert!(amount > 0, "Expense amount must be greater than 0");
    assert!(!purpose.is_empty(), "Expense purpose must not be empty");
    assert!(
        !evidence_cid.is_empty(),
        "Expense evidence must not be empty"
    );

    let expense = Expense {
        sacco_id: sacco_account.account_id,
        amount,
        purpose,
        evidence_cid,
        requested_by: proposer.account_id,
        status: ExpenseStatus::Proposed,
        approvers: vec![],
        rejectors: vec![],
        executed_evidence: None,
    };

    let pre = expense_account.clone();
    expense_account.account.data = (&expense).into();
    let post_data = expense_account.account.data;

    vec![
        AccountStateDiff::unchanged(sacco_account),
        AccountStateDiff::new(pre, BalanceDiff::Add(0), post_data),
        AccountStateDiff::unchanged(proposer),
    ]
}
