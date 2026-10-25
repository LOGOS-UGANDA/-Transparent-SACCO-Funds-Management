use sacco_ledger_core::{Loan, LoanStatus, Sacco};
use crate::is_approver;
use nssa_core::{
    account::{AccountWithMetadata, BalanceDiff},
    program::AccountStateDiff,
};

/// Committee multi-sig approval of an applied loan. Approval at exactly the
/// SACCO's `required_approvals` flips the loan to `Approved`.
pub fn approve_loan(
    sacco_account: AccountWithMetadata,
    mut loan_account: AccountWithMetadata,
    approver: AccountWithMetadata,
) -> Vec<AccountStateDiff> {
    assert!(approver.is_authorized, "Approver must authorize");

    let sacco = Sacco::try_from(&sacco_account.account.data)
        .expect("Sacco account must hold valid Sacco data");
    assert!(
        is_approver(&sacco, &approver.account_id),
        "Not a SACCO approver"
    );

    let mut loan = Loan::try_from(&loan_account.account.data)
        .expect("Loan account must hold valid Loan data");
    assert_eq!(loan.status, LoanStatus::Applied, "Loan is not in Applied state");
    assert!(
        !loan.approvals.contains(&approver.account_id),
        "Already approved by this approver"
    );
    assert!(
        !loan.rejectors.contains(&approver.account_id),
        "Already rejected by this approver"
    );

    loan.approvals.push(approver.account_id);
    if loan.approvals.len() >= usize::from(sacco.required_approvals) {
        loan.status = LoanStatus::Approved;
    }

    let pre = loan_account.clone();
    loan_account.account.data = (&loan).into();
    let post_data = loan_account.account.data;

    vec![
        AccountStateDiff::unchanged(sacco_account),
        AccountStateDiff::new(pre, BalanceDiff::Add(0), post_data),
        AccountStateDiff::unchanged(approver),
    ]
}
