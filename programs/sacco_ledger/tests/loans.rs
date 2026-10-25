//! Loan lifecycle integration tests: apply, multi-sig approve/reject, disburse, repay.

use nssa_core::account::{Account, AccountId, AccountWithMetadata};
use sacco_ledger_program::{
    add_approver::add_approver, add_member::add_member, apply_loan::apply_loan,
    approve_loan::approve_loan, disburse_loan::disburse_loan, record_repayment::record_repayment,
    register_sacco::register_sacco, reject_loan::reject_loan, verify_sacco::verify_sacco,
};
use sacco_ledger_core::{Loan, LoanStatus, Member, Sacco};

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

/// Verified SACCO (threshold 2, approvers = admin(2) + second(3)) with member 6
/// and an applied loan (account 7).
fn fixture_with_applied_loan() -> Fixture {
    let mut sacco_account = blank_account(1);
    let admin = blank_account(2);
    let second = blank_account(3);
    let mut member_account = blank_account(6);

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

    // Raise the per-SACCO threshold to 2 for the multi-sig pins.
    let mut sacco = Sacco::try_from(&sacco_account.account.data).expect("sacco");
    sacco.required_approvals = 2;
    sacco_account.account.data = (&sacco).into();

    let diffs = add_member(sacco_account.clone(), member_account.clone(), admin.clone(), aid(6));
    sacco_account.account.data = post_of(&diffs, 0);
    member_account.account.data = post_of(&diffs, 1);

    let diffs = verify_sacco(sacco_account.clone(), admin.clone());
    sacco_account.account.data = post_of(&diffs, 0);

    let loan_account = blank_account(7);
    let diffs = apply_loan(
        sacco_account.clone(),
        member_account.clone(),
        loan_account.clone(),
        member_account.clone(),
        150_000,
        "School fees".into(),
    );
    member_account.account.data = post_of(&diffs, 1);
    let mut loan_account = blank_account(7);
    loan_account.account.data = post_of(&diffs, 2);

    Fixture {
        sacco_account,
        member_account,
        loan_account,
        admin,
        second,
    }
}

struct Fixture {
    sacco_account: AccountWithMetadata,
    member_account: AccountWithMetadata,
    loan_account: AccountWithMetadata,
    admin: AccountWithMetadata,
    second: AccountWithMetadata,
}

#[test]
fn apply_loan_creates_applied_loan() {
    let f = fixture_with_applied_loan();
    let loan = Loan::try_from(&f.loan_account.account.data).expect("loan");
    assert_eq!(loan.status, LoanStatus::Applied);
    assert_eq!(loan.principal, 150_000);
    assert_eq!(loan.purpose, "School fees");
    assert!(loan.approvals.is_empty());
    assert_eq!(loan.repaid_amount, 0);
    assert!(loan.evidence_cid.is_none());
    let member = Member::try_from(&f.member_account.account.data).expect("member");
    assert_eq!(member.loan_ids, vec![aid(7)]);
}

#[test]
#[should_panic(expected = "Sacco is not verified")]
fn apply_loan_rejected_for_unverified_sacco() {
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
    let _ = apply_loan(
        sacco_account,
        blank_account(6),
        blank_account(7),
        blank_account(6),
        150_000,
        "School fees".into(),
    );
}

#[test]
fn approve_loan_reaches_threshold_at_exact_count() {
    let mut f = fixture_with_applied_loan();
    let sacco = Sacco::try_from(&f.sacco_account.account.data).expect("sacco");
    assert_eq!(sacco.required_approvals, 2);

    let diffs = approve_loan(f.sacco_account.clone(), f.loan_account.clone(), f.admin.clone());
    let after_one = Loan::try_from(diffs[1].post_data.as_ref().expect("loan")).expect("loan");
    assert_eq!(after_one.status, LoanStatus::Applied, "below threshold stays applied");
    assert_eq!(after_one.approvals.len(), 1);

    f.loan_account.account.data = post_of(&diffs, 1);
    let diffs = approve_loan(f.sacco_account.clone(), f.loan_account, f.second);
    let after_two = Loan::try_from(diffs[1].post_data.as_ref().expect("loan")).expect("loan");
    assert_eq!(after_two.status, LoanStatus::Approved, "exact threshold approves");
    assert_eq!(after_two.approvals.len(), 2);
}

#[test]
#[should_panic(expected = "Already approved by this approver")]
fn approve_loan_duplicate_approver_rejected() {
    let f = fixture_with_applied_loan();
    let diffs = approve_loan(f.sacco_account.clone(), f.loan_account.clone(), f.admin.clone());
    let mut loan_account = f.loan_account;
    loan_account.account.data = post_of(&diffs, 1);
    let _ = approve_loan(f.sacco_account, loan_account, f.admin);
}

#[test]
#[should_panic(expected = "Not a SACCO approver")]
fn approve_loan_non_approver_rejected() {
    let f = fixture_with_applied_loan();
    let _ = approve_loan(f.sacco_account, f.loan_account, blank_account(8));
}

#[test]
fn reject_loan_approver_only() {
    let f = fixture_with_applied_loan();
    let diffs = reject_loan(f.sacco_account.clone(), f.loan_account, f.second);
    let loan = Loan::try_from(diffs[1].post_data.as_ref().expect("loan")).expect("loan");
    assert_eq!(loan.status, LoanStatus::Rejected);
}

#[test]
fn disburse_loan_sets_evidence_status_and_totals() {
    let mut f = fixture_with_applied_loan();
    let diffs = approve_loan(f.sacco_account.clone(), f.loan_account.clone(), f.admin.clone());
    f.loan_account.account.data = post_of(&diffs, 1);
    let diffs = approve_loan(f.sacco_account.clone(), f.loan_account.clone(), f.second.clone());
    f.loan_account.account.data = post_of(&diffs, 1);

    let before = Sacco::try_from(&f.sacco_account.account.data).expect("sacco");
    let diffs = disburse_loan(
        f.sacco_account.clone(),
        f.member_account.clone(),
        f.loan_account,
        f.second,
        "bafy-disbursement-evidence".into(),
    );
    let after = Sacco::try_from(diffs[0].post_data.as_ref().expect("sacco")).expect("sacco");
    assert_eq!(
        after.total_loans_disbursed,
        before.total_loans_disbursed + 150_000
    );
    let loan = Loan::try_from(diffs[2].post_data.as_ref().expect("loan")).expect("loan");
    assert_eq!(loan.status, LoanStatus::Disbursed);
    assert_eq!(loan.evidence_cid.as_deref(), Some("bafy-disbursement-evidence"));
}

#[test]
#[should_panic(expected = "Disbursement evidence must not be empty")]
fn disburse_loan_requires_evidence() {
    let mut f = fixture_with_applied_loan();
    let diffs = approve_loan(f.sacco_account.clone(), f.loan_account.clone(), f.admin.clone());
    f.loan_account.account.data = post_of(&diffs, 1);
    let diffs = approve_loan(f.sacco_account.clone(), f.loan_account.clone(), f.second.clone());
    f.loan_account.account.data = post_of(&diffs, 1);
    let _ = disburse_loan(f.sacco_account, f.member_account, f.loan_account, f.second, String::new());
}

#[test]
fn record_repayment_updates_outstanding_and_totals() {
    let mut f = fixture_with_applied_loan();
    let diffs = approve_loan(f.sacco_account.clone(), f.loan_account.clone(), f.admin.clone());
    f.loan_account.account.data = post_of(&diffs, 1);
    let diffs = approve_loan(f.sacco_account.clone(), f.loan_account.clone(), f.second.clone());
    f.loan_account.account.data = post_of(&diffs, 1);
    let diffs = disburse_loan(
        f.sacco_account.clone(),
        f.member_account.clone(),
        f.loan_account.clone(),
        f.second.clone(),
        "bafy-disbursement-evidence".into(),
    );
    f.sacco_account.account.data = post_of(&diffs, 0);
    f.loan_account.account.data = post_of(&diffs, 2);

    let diffs = record_repayment(
        f.sacco_account.clone(),
        f.member_account.clone(),
        f.loan_account.clone(),
        f.admin,
        aid(7),
        50_000,
        "MPZA-2001".into(),
    );
    let loan = Loan::try_from(diffs[2].post_data.as_ref().expect("loan")).expect("loan");
    assert_eq!(loan.repaid_amount, 50_000);
    assert_eq!(loan.status, LoanStatus::Disbursed, "partial repayment keeps status");
    let sacco = Sacco::try_from(diffs[0].post_data.as_ref().expect("sacco")).expect("sacco");
    assert_eq!(sacco.total_loans_repaid, 50_000);
}

#[test]
#[should_panic(expected = "Repayment exceeds outstanding principal")]
fn record_repayment_rejects_overpay() {
    let f = fixture_fully_disbursed();
    let _ = record_repayment(
        f.sacco_account,
        f.member_account,
        f.loan_account,
        f.admin,
        aid(7),
        150_001,
        "MPZA-2002".into(),
    );
}

#[test]
#[should_panic(expected = "Loan is already fully repaid")]
fn record_repayment_rejects_closed_loan() {
    let mut f = fixture_fully_disbursed();
    let diffs = record_repayment(
        f.sacco_account.clone(),
        f.member_account.clone(),
        f.loan_account.clone(),
        f.admin.clone(),
        aid(7),
        150_000,
        "MPZA-2003".into(),
    );
    f.loan_account.account.data = post_of(&diffs, 2);
    let _ = record_repayment(
        f.sacco_account,
        f.member_account,
        f.loan_account,
        f.admin,
        aid(7),
        1,
        "MPZA-2004".into(),
    );
}

/// Fixture with the loan approved, disbursed and ready for repayment.
fn fixture_fully_disbursed() -> Fixture {
    let mut f = fixture_with_applied_loan();
    let diffs = approve_loan(f.sacco_account.clone(), f.loan_account.clone(), f.admin.clone());
    f.loan_account.account.data = post_of(&diffs, 1);
    let diffs = approve_loan(f.sacco_account.clone(), f.loan_account.clone(), f.second.clone());
    f.loan_account.account.data = post_of(&diffs, 1);
    let diffs = disburse_loan(
        f.sacco_account.clone(),
        f.member_account.clone(),
        f.loan_account.clone(),
        f.second.clone(),
        "bafy-disbursement-evidence".into(),
    );
    f.sacco_account.account.data = post_of(&diffs, 0);
    f.loan_account.account.data = post_of(&diffs, 2);
    f
}
