use sacco_ledger_core::{Loan, LoanStatus, Sacco};
use crate::is_approver;
use nssa_core::{
    account::{AccountId, AccountWithMetadata, BalanceDiff},
    program::AccountStateDiff,
};

/// Record a loan repayment (an off-chain transfer) against a disbursed loan.
/// Repayments accumulate; the loan closes at exactly the principal.
#[allow(clippy::too_many_arguments)]
pub fn record_repayment(
    mut sacco_account: AccountWithMetadata,
    member_account: AccountWithMetadata,
    mut loan_account: AccountWithMetadata,
    approver: AccountWithMetadata,
    loan: AccountId,
    amount: u128,
    reference: String,
) -> Vec<AccountStateDiff> {
    assert!(approver.is_authorized, "Approver must authorize");

    let mut sacco = Sacco::try_from(&sacco_account.account.data)
        .expect("Sacco account must hold valid Sacco data");
    assert!(
        is_approver(&sacco, &approver.account_id),
        "Not a SACCO approver"
    );

    let mut loan_state = Loan::try_from(&loan_account.account.data)
        .expect("Loan account must hold valid Loan data");
    assert_eq!(
        loan_account.account_id, loan,
        "Loan account does not match the loan id argument"
    );
    assert!(amount > 0, "Repayment must be greater than 0");
    assert!(!reference.is_empty(), "Repayment reference must not be empty");
    assert!(
        loan_state.status == LoanStatus::Disbursed || loan_state.status == LoanStatus::Repaid,
        "Loan is not in Disbursed state"
    );
    assert!(
        loan_state.status != LoanStatus::Repaid || loan_state.repaid_amount < loan_state.principal,
        "Loan is already fully repaid"
    );

    let outstanding = loan_state
        .principal
        .checked_sub(loan_state.repaid_amount)
        .expect("repaid exceeds principal");
    assert!(amount <= outstanding, "Repayment exceeds outstanding principal");

    loan_state.repaid_amount = loan_state
        .repaid_amount
        .checked_add(amount)
        .expect("repaid overflow");
    if loan_state.repaid_amount == loan_state.principal {
        loan_state.status = LoanStatus::Repaid;
    }
    sacco.total_loans_repaid = sacco
        .total_loans_repaid
        .checked_add(amount)
        .expect("loans repaid overflow");

    let sacco_pre = sacco_account.clone();
    sacco_account.account.data = (&sacco).into();
    let sacco_data = sacco_account.account.data;

    let loan_pre = loan_account.clone();
    loan_account.account.data = (&loan_state).into();
    let loan_data = loan_account.account.data;

    vec![
        AccountStateDiff::new(sacco_pre, BalanceDiff::Add(0), sacco_data),
        AccountStateDiff::unchanged(member_account),
        AccountStateDiff::new(loan_pre, BalanceDiff::Add(0), loan_data),
        AccountStateDiff::unchanged(approver),
    ]
}
