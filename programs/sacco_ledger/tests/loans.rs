//! Loan lifecycle: application, multi-sig approval, disbursement, repayment.

mod support;

use sacco_ledger_core::{Instruction, LoanStatus};
use support::{aid, applied_loan, disbursed_loan, meta, two_approver_sacco, Ledger};

#[test]
fn apply_loan_creates_applied_loan() {
    let mut ledger = Ledger::new();
    two_approver_sacco(&mut ledger);
    applied_loan(&mut ledger, aid(7));
    let loan = ledger.loan(aid(7));
    assert_eq!(loan.sacco_id, aid(1));
    assert_eq!(loan.member_id, aid(6));
    assert_eq!(loan.principal, 200_000);
    assert_eq!(loan.status, LoanStatus::Applied);
    assert_eq!(loan.required_approvals, 2);
    assert!(loan.approvals.is_empty());
    assert!(ledger.member(aid(6)).loan_ids.contains(&aid(7)));
}

#[test]
#[should_panic(expected = "Sacco is not verified")]
fn apply_loan_rejected_for_unverified_sacco() {
    let mut ledger = Ledger::new();
    ledger.execute(
        &[meta(aid(1)), meta(aid(2))],
        Instruction::RegisterSacco {
            name: "Kampala Traders SACCO".into(),
            registration_number: "UG-SACCO-001".into(),
            treasury: aid(9),
        },
    );
    ledger.execute(
        &[meta(aid(1)), meta(aid(6)), meta(aid(2))],
        Instruction::AddMember { identity: aid(6) },
    );
    applied_loan(&mut ledger, aid(7));
}

#[test]
fn approve_loan_reaches_threshold_at_exact_count() {
    let mut ledger = Ledger::new();
    two_approver_sacco(&mut ledger);
    applied_loan(&mut ledger, aid(7));
    ledger.execute(
        &[meta(aid(1)), meta(aid(7)), meta(aid(2))],
        Instruction::ApproveLoan,
    );
    assert_eq!(ledger.loan(aid(7)).status, LoanStatus::Applied);
    ledger.execute(
        &[meta(aid(1)), meta(aid(7)), meta(aid(3))],
        Instruction::ApproveLoan,
    );
    assert_eq!(ledger.loan(aid(7)).status, LoanStatus::Approved);
    assert_eq!(ledger.loan(aid(7)).approvals, vec![aid(2), aid(3)]);
}

#[test]
#[should_panic(expected = "Already approved by this approver")]
fn approve_loan_duplicate_approver_rejected() {
    let mut ledger = Ledger::new();
    two_approver_sacco(&mut ledger);
    applied_loan(&mut ledger, aid(7));
    ledger.execute(
        &[meta(aid(1)), meta(aid(7)), meta(aid(2))],
        Instruction::ApproveLoan,
    );
    ledger.execute(
        &[meta(aid(1)), meta(aid(7)), meta(aid(2))],
        Instruction::ApproveLoan,
    );
}

#[test]
#[should_panic(expected = "Not a SACCO approver")]
fn approve_loan_non_approver_rejected() {
    let mut ledger = Ledger::new();
    two_approver_sacco(&mut ledger);
    applied_loan(&mut ledger, aid(7));
    ledger.execute(
        &[meta(aid(1)), meta(aid(7)), meta(aid(5))],
        Instruction::ApproveLoan,
    );
}

#[test]
fn reject_loan_approver_only() {
    let mut ledger = Ledger::new();
    two_approver_sacco(&mut ledger);
    applied_loan(&mut ledger, aid(7));
    ledger.execute(
        &[meta(aid(1)), meta(aid(7)), meta(aid(2))],
        Instruction::RejectLoan,
    );
    let loan = ledger.loan(aid(7));
    assert_eq!(loan.status, LoanStatus::Rejected);
    assert_eq!(loan.rejectors, vec![aid(2)]);
}

#[test]
fn disburse_loan_sets_evidence_status_and_totals() {
    let mut ledger = Ledger::new();
    two_approver_sacco(&mut ledger);
    disbursed_loan(&mut ledger, aid(7));
    let loan = ledger.loan(aid(7));
    assert_eq!(loan.status, LoanStatus::Disbursed);
    assert_eq!(loan.evidence_cid.as_deref(), Some("bafy-disbursement"));
    assert_eq!(ledger.sacco(aid(1)).total_loans_disbursed, 200_000);
}

#[test]
#[should_panic(expected = "Disbursement evidence must not be empty")]
fn disburse_loan_requires_evidence() {
    let mut ledger = Ledger::new();
    two_approver_sacco(&mut ledger);
    applied_loan(&mut ledger, aid(7));
    for approver in [aid(2), aid(3)] {
        ledger.execute(
            &[meta(aid(1)), meta(aid(7)), meta(approver)],
            Instruction::ApproveLoan,
        );
    }
    ledger.execute(
        &[meta(aid(1)), meta(aid(6)), meta(aid(7)), meta(aid(2))],
        Instruction::DisburseLoan {
            evidence_cid: String::new(),
            principal: 200_000,
        },
    );
}

#[test]
fn record_repayment_updates_outstanding_and_totals() {
    let mut ledger = Ledger::new();
    two_approver_sacco(&mut ledger);
    disbursed_loan(&mut ledger, aid(7));
    ledger.execute(
        &[meta(aid(1)), meta(aid(6)), meta(aid(7)), meta(aid(2))],
        Instruction::RecordRepayment {
            loan: aid(7),
            amount: 50_000,
            reference: "MPZA-REPAY-1".into(),
        },
    );
    assert_eq!(ledger.loan(aid(7)).repaid_amount, 50_000);
    assert_eq!(ledger.loan(aid(7)).status, LoanStatus::Disbursed);
    assert_eq!(ledger.sacco(aid(1)).total_loans_repaid, 50_000);
}

#[test]
#[should_panic(expected = "Repayment exceeds outstanding principal")]
fn record_repayment_rejects_overpay() {
    let mut ledger = Ledger::new();
    two_approver_sacco(&mut ledger);
    disbursed_loan(&mut ledger, aid(7));
    ledger.execute(
        &[meta(aid(1)), meta(aid(6)), meta(aid(7)), meta(aid(2))],
        Instruction::RecordRepayment {
            loan: aid(7),
            amount: 200_001,
            reference: "X".into(),
        },
    );
}

#[test]
#[should_panic(expected = "Loan is already fully repaid")]
fn record_repayment_rejects_closed_loan() {
    let mut ledger = Ledger::new();
    two_approver_sacco(&mut ledger);
    disbursed_loan(&mut ledger, aid(7));
    ledger.execute(
        &[meta(aid(1)), meta(aid(6)), meta(aid(7)), meta(aid(2))],
        Instruction::RecordRepayment {
            loan: aid(7),
            amount: 200_000,
            reference: "X".into(),
        },
    );
    ledger.execute(
        &[meta(aid(1)), meta(aid(6)), meta(aid(7)), meta(aid(2))],
        Instruction::RecordRepayment {
            loan: aid(7),
            amount: 1,
            reference: "Y".into(),
        },
    );
}
