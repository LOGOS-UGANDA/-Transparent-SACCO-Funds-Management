//! Expenses: proposal, multi-sig approval, execution evidence.

mod support;

use sacco_ledger_core::{ExpenseStatus, Instruction};
use support::{aid, meta, proposed_expense, two_approver_sacco, Ledger};

#[test]
fn propose_expense_creates_proposed_expense() {
    let mut ledger = Ledger::new();
    two_approver_sacco(&mut ledger);
    proposed_expense(&mut ledger, aid(8));
    let expense = ledger.expense(aid(8));
    assert_eq!(expense.sacco_id, aid(1));
    assert_eq!(expense.amount, 120_000);
    assert_eq!(expense.requested_by, aid(2));
    assert_eq!(expense.status, ExpenseStatus::Proposed);
    assert_eq!(expense.required_approvals, 2);
}

#[test]
#[should_panic(expected = "Sacco is not verified")]
fn propose_expense_rejected_for_unverified_sacco() {
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
        &[meta(aid(1)), meta(aid(8)), meta(aid(2))],
        Instruction::ProposeExpense {
            amount: 1_000,
            purpose: "desk".into(),
            evidence_cid: "cid".into(),
            required_approvals: 1,
        },
    );
}

#[test]
#[should_panic(expected = "Not a SACCO approver")]
fn propose_expense_requires_approver() {
    let mut ledger = Ledger::new();
    two_approver_sacco(&mut ledger);
    ledger.execute(
        &[meta(aid(1)), meta(aid(8)), meta(aid(5))],
        Instruction::ProposeExpense {
            amount: 1_000,
            purpose: "desk".into(),
            evidence_cid: "cid".into(),
            required_approvals: 2,
        },
    );
}

#[test]
#[should_panic(expected = "Expense purpose must not be empty")]
fn propose_expense_rejects_empty_purpose() {
    let mut ledger = Ledger::new();
    two_approver_sacco(&mut ledger);
    ledger.execute(
        &[meta(aid(1)), meta(aid(8)), meta(aid(2))],
        Instruction::ProposeExpense {
            amount: 1_000,
            purpose: String::new(),
            evidence_cid: "cid".into(),
            required_approvals: 2,
        },
    );
}

#[test]
fn approve_expense_below_threshold_stays_pending() {
    let mut ledger = Ledger::new();
    two_approver_sacco(&mut ledger);
    proposed_expense(&mut ledger, aid(8));
    ledger.execute(
        &[meta(aid(1)), meta(aid(8)), meta(aid(2))],
        Instruction::ApproveExpense,
    );
    assert_eq!(ledger.expense(aid(8)).status, ExpenseStatus::Proposed);
    assert_eq!(ledger.expense(aid(8)).approvers, vec![aid(2)]);
}

#[test]
fn approve_expense_executes_at_exact_threshold() {
    let mut ledger = Ledger::new();
    two_approver_sacco(&mut ledger);
    proposed_expense(&mut ledger, aid(8));
    ledger.execute(
        &[meta(aid(1)), meta(aid(8)), meta(aid(2))],
        Instruction::ApproveExpense,
    );
    ledger.execute(
        &[meta(aid(1)), meta(aid(8)), meta(aid(3))],
        Instruction::ApproveExpense,
    );
    assert_eq!(ledger.expense(aid(8)).status, ExpenseStatus::Approved);
    assert_eq!(ledger.expense(aid(8)).approvers, vec![aid(2), aid(3)]);
}

#[test]
#[should_panic(expected = "Already approved by this approver")]
fn approve_expense_duplicate_approver_rejected() {
    let mut ledger = Ledger::new();
    two_approver_sacco(&mut ledger);
    proposed_expense(&mut ledger, aid(8));
    ledger.execute(
        &[meta(aid(1)), meta(aid(8)), meta(aid(2))],
        Instruction::ApproveExpense,
    );
    ledger.execute(
        &[meta(aid(1)), meta(aid(8)), meta(aid(2))],
        Instruction::ApproveExpense,
    );
}

#[test]
fn reject_expense_approver_only() {
    let mut ledger = Ledger::new();
    two_approver_sacco(&mut ledger);
    proposed_expense(&mut ledger, aid(8));
    ledger.execute(
        &[meta(aid(1)), meta(aid(8)), meta(aid(2))],
        Instruction::RejectExpense,
    );
    let expense = ledger.expense(aid(8));
    assert_eq!(expense.status, ExpenseStatus::Rejected);
    assert_eq!(expense.rejectors, vec![aid(2)]);
}

#[test]
fn execute_expense_records_off_chain_release() {
    let mut ledger = Ledger::new();
    two_approver_sacco(&mut ledger);
    proposed_expense(&mut ledger, aid(8));
    for approver in [aid(2), aid(3)] {
        ledger.execute(
            &[meta(aid(1)), meta(aid(8)), meta(approver)],
            Instruction::ApproveExpense,
        );
    }
    ledger.execute(
        &[meta(aid(1)), meta(aid(8)), meta(aid(2))],
        Instruction::ExecuteExpense {
            executed_evidence: "bafy-receipt".into(),
            amount: 120_000,
        },
    );
    let expense = ledger.expense(aid(8));
    assert_eq!(expense.status, ExpenseStatus::Executed);
    assert_eq!(expense.executed_evidence.as_deref(), Some("bafy-receipt"));
    assert_eq!(ledger.sacco(aid(1)).total_expenses, 120_000);
}
