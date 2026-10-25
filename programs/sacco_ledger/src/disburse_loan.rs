use sacco_ledger_core::{Loan, LoanStatus, Sacco};
use crate::is_approver;
use nssa_core::{
    account::{AccountWithMetadata, BalanceDiff},
    program::AccountStateDiff,
};

/// Record loan disbursement (an off-chain transfer) with storage evidence CID.
pub fn disburse_loan(
    mut sacco_account: AccountWithMetadata,
    member_account: AccountWithMetadata,
    mut loan_account: AccountWithMetadata,
    approver: AccountWithMetadata,
    evidence_cid: String,
) -> Vec<AccountStateDiff> {
    assert!(approver.is_authorized, "Approver must authorize");

    let mut sacco = Sacco::try_from(&sacco_account.account.data)
        .expect("Sacco account must hold valid Sacco data");
    assert!(
        is_approver(&sacco, &approver.account_id),
        "Not a SACCO approver"
    );

    let mut loan = Loan::try_from(&loan_account.account.data)
        .expect("Loan account must hold valid Loan data");
    assert_eq!(
        loan.status,
        LoanStatus::Approved,
        "Loan must be approved before disbursement"
    );
    assert!(
        !evidence_cid.is_empty(),
        "Disbursement evidence must not be empty"
    );

    loan.status = LoanStatus::Disbursed;
    loan.evidence_cid = Some(evidence_cid);
    sacco.total_loans_disbursed = sacco
        .total_loans_disbursed
        .checked_add(loan.principal)
        .expect("loans disbursed overflow");

    let sacco_pre = sacco_account.clone();
    sacco_account.account.data = (&sacco).into();
    let sacco_data = sacco_account.account.data;

    let loan_pre = loan_account.clone();
    loan_account.account.data = (&loan).into();
    let loan_data = loan_account.account.data;

    vec![
        AccountStateDiff::new(sacco_pre, BalanceDiff::Add(0), sacco_data),
        AccountStateDiff::unchanged(member_account),
        AccountStateDiff::new(loan_pre, BalanceDiff::Add(0), loan_data),
        AccountStateDiff::unchanged(approver),
    ]
}
