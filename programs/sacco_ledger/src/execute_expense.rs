use sacco_ledger_core::{Expense, ExpenseStatus, Sacco};
use crate::is_approver;
use nssa_core::{
    account::{AccountWithMetadata, BalanceDiff},
    program::AccountStateDiff,
};

/// Record execution of an approved expense: the off-chain payment proof. Value never
/// moves on chain — this instruction attests that the approved payment happened.
pub fn execute_expense(
    mut sacco_account: AccountWithMetadata,
    mut expense_account: AccountWithMetadata,
    approver: AccountWithMetadata,
    executed_evidence: String,
) -> Vec<AccountStateDiff> {
    assert!(approver.is_authorized, "Approver must authorize");

    let mut sacco = Sacco::try_from(&sacco_account.account.data)
        .expect("Sacco account must hold valid Sacco data");
    assert!(
        is_approver(&sacco, &approver.account_id),
        "Not a SACCO approver"
    );

    let mut expense = Expense::try_from(&expense_account.account.data)
        .expect("Expense account must hold valid Expense data");
    assert_eq!(
        expense.status,
        ExpenseStatus::Approved,
        "Expense must be approved before execution"
    );
    assert!(
        !executed_evidence.is_empty(),
        "Execution evidence must not be empty"
    );

    expense.status = ExpenseStatus::Executed;
    expense.executed_evidence = Some(executed_evidence);
    sacco.total_expenses = sacco
        .total_expenses
        .checked_add(expense.amount)
        .expect("expense total overflow");

    let sacco_pre = sacco_account.clone();
    sacco_account.account.data = (&sacco).into();
    let sacco_data = sacco_account.account.data;

    let expense_pre = expense_account.clone();
    expense_account.account.data = (&expense).into();
    let expense_data = expense_account.account.data;

    vec![
        AccountStateDiff::new(sacco_pre, BalanceDiff::Add(0), sacco_data),
        AccountStateDiff::new(expense_pre, BalanceDiff::Add(0), expense_data),
        AccountStateDiff::unchanged(approver),
    ]
}
