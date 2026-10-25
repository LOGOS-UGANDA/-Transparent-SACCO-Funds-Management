use sacco_ledger_core::{Expense, ExpenseStatus, Sacco};
use crate::is_approver;
use nssa_core::{
    account::{AccountWithMetadata, BalanceDiff},
    program::AccountStateDiff,
};

/// Committee multi-sig approval of a proposed expense. At exactly the SACCO's
/// `required_approvals`, the expense flips to `Approved`.
pub fn approve_expense(
    sacco_account: AccountWithMetadata,
    mut expense_account: AccountWithMetadata,
    approver: AccountWithMetadata,
) -> Vec<AccountStateDiff> {
    assert!(approver.is_authorized, "Approver must authorize");

    let sacco = Sacco::try_from(&sacco_account.account.data)
        .expect("Sacco account must hold valid Sacco data");
    assert!(
        is_approver(&sacco, &approver.account_id),
        "Not a SACCO approver"
    );

    let mut expense = Expense::try_from(&expense_account.account.data)
        .expect("Expense account must hold valid Expense data");
    assert_eq!(
        expense.status,
        ExpenseStatus::Proposed,
        "Expense is not proposed"
    );
    assert!(
        !expense.approvers.contains(&approver.account_id),
        "Already approved by this approver"
    );
    assert!(
        !expense.rejectors.contains(&approver.account_id),
        "Already rejected by this approver"
    );

    expense.approvers.push(approver.account_id);
    if expense.approvers.len() >= usize::from(sacco.required_approvals) {
        expense.status = ExpenseStatus::Approved;
    }

    let pre = expense_account.clone();
    expense_account.account.data = (&expense).into();
    let post_data = expense_account.account.data;

    vec![
        AccountStateDiff::unchanged(sacco_account),
        AccountStateDiff::new(pre, BalanceDiff::Add(0), post_data),
        AccountStateDiff::unchanged(approver),
    ]
}
