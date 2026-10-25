use sacco_ledger_core::{Loan, LoanStatus, Member, Sacco};
use nssa_core::{
    account::{Account, AccountWithMetadata, BalanceDiff},
    program::AccountStateDiff,
};

/// Member applies for a loan on their own account. Records intent; committee
/// approval follows via `ApproveLoan` multi-sig.
pub fn apply_loan(
    mut sacco_account: AccountWithMetadata,
    mut member_account: AccountWithMetadata,
    mut loan_account: AccountWithMetadata,
    member_signer: AccountWithMetadata,
    amount: u128,
    purpose: String,
) -> Vec<AccountStateDiff> {
    assert!(
        member_signer.is_authorized,
        "Member must authorize the application"
    );
    assert!(loan_account.account == Account::default(), "Loan account must be uninitialized");
    assert!(loan_account.is_authorized, "Loan account must be authorized");

    let sacco = Sacco::try_from(&sacco_account.account.data)
        .expect("Sacco account must hold valid Sacco data");
    assert!(sacco.is_verified, "Sacco is not verified");
    let mut member = Member::try_from(&member_account.account.data)
        .expect("Member account must hold valid Member data");
    assert_eq!(
        member.sacco_id, sacco_account.account_id,
        "Member belongs to a different SACCO"
    );
    assert_eq!(
        member.identity, member_signer.account_id,
        "Only the member can apply for their loan"
    );
    assert!(amount > 0, "Loan amount must be greater than 0");
    assert!(!purpose.is_empty(), "Loan purpose must not be empty");

    let loan = Loan {
        sacco_id: sacco_account.account_id,
        member_id: member.identity,
        principal: amount,
        purpose,
        status: LoanStatus::Applied,
        approvals: vec![],
        rejectors: vec![],
        evidence_cid: None,
        repaid_amount: 0,
    };
    let loan_id = loan_account.account_id;
    member.loan_ids.push(loan_id);

    let sacco_pre = sacco_account.clone();
    sacco_account.account.data = (&sacco).into();
    let sacco_data = sacco_account.account.data;

    let member_pre = member_account.clone();
    member_account.account.data = (&member).into();
    let member_data = member_account.account.data;

    let loan_pre = loan_account.clone();
    loan_account.account.data = (&loan).into();
    let loan_data = loan_account.account.data;

    vec![
        AccountStateDiff::new(sacco_pre, BalanceDiff::Add(0), sacco_data),
        AccountStateDiff::new(member_pre, BalanceDiff::Add(0), member_data),
        AccountStateDiff::new(loan_pre, BalanceDiff::Add(0), loan_data),
        AccountStateDiff::unchanged(member_signer),
    ]
}
