//! Expenses: proposal, multi-sig approval, execution evidence.

mod support;

use sacco_ledger_core::{ExpenseStatus, Instruction};
use support::{aid, meta, proposed_expense, three_approver_sacco, Ledger};

#[test]
fn propose_expense_creates_proposed_expense() {
    let mut ledger = Ledger::new();
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
    three_approver_sacco(&mut ledger);
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
#[should_panic(expected = "Approver threshold does not match the SACCO configuration")]
fn expense_threshold_must_match_sacco_configuration() {
    let mut ledger = Ledger::new();
    three_approver_sacco(&mut ledger);
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
#[should_panic(expected = "Expense purpose must not be empty")]
fn propose_expense_rejects_empty_purpose() {
    let mut ledger = Ledger::new();
    three_approver_sacco(&mut ledger);
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
    proposed_expense(&mut ledger, aid(8));
    // Proposer is aid(2); a different approver must sign.
    ledger.execute(
        &[meta(aid(1)), meta(aid(8)), meta(aid(3))],
        Instruction::ApproveExpense,
    );
    assert_eq!(ledger.expense(aid(8)).status, ExpenseStatus::Proposed);
    assert_eq!(ledger.expense(aid(8)).approvers, vec![aid(3)]);
}

#[test]
fn approve_expense_executes_at_exact_threshold() {
    let mut ledger = Ledger::new();
    proposed_expense(&mut ledger, aid(8));
    ledger.execute(
        &[meta(aid(1)), meta(aid(8)), meta(aid(3))],
        Instruction::ApproveExpense,
    );
    ledger.execute(
        &[meta(aid(1)), meta(aid(8)), meta(aid(4))],
        Instruction::ApproveExpense,
    );
    assert_eq!(ledger.expense(aid(8)).status, ExpenseStatus::Approved);
    assert_eq!(ledger.expense(aid(8)).approvers, vec![aid(3), aid(4)]);
}

#[test]
#[should_panic(expected = "Proposer cannot approve their own expense")]
fn proposer_cannot_approve_their_own_expense() {
    let mut ledger = Ledger::new();
    proposed_expense(&mut ledger, aid(8));
    ledger.execute(
        &[meta(aid(1)), meta(aid(8)), meta(aid(2))],
        Instruction::ApproveExpense,
    );
}

#[test]
#[should_panic(expected = "Already approved by this approver")]
fn approve_expense_duplicate_approver_rejected() {
    let mut ledger = Ledger::new();
    proposed_expense(&mut ledger, aid(8));
    ledger.execute(
        &[meta(aid(1)), meta(aid(8)), meta(aid(3))],
        Instruction::ApproveExpense,
    );
    ledger.execute(
        &[meta(aid(1)), meta(aid(8)), meta(aid(3))],
        Instruction::ApproveExpense,
    );
}

#[test]
fn reject_expense_approver_only() {
    let mut ledger = Ledger::new();
    proposed_expense(&mut ledger, aid(8));
    ledger.execute(
        &[meta(aid(1)), meta(aid(8)), meta(aid(3))],
        Instruction::RejectExpense,
    );
    let expense = ledger.expense(aid(8));
    assert_eq!(expense.status, ExpenseStatus::Rejected);
    assert_eq!(expense.rejectors, vec![aid(3)]);
}

#[test]
#[should_panic(expected = "Proposer cannot reject their own expense")]
fn proposer_cannot_reject_their_own_expense() {
    let mut ledger = Ledger::new();
    proposed_expense(&mut ledger, aid(8));
    ledger.execute(
        &[meta(aid(1)), meta(aid(8)), meta(aid(2))],
        Instruction::RejectExpense,
    );
}

#[test]
fn execute_expense_records_off_chain_release() {
    let mut ledger = Ledger::new();
    proposed_expense(&mut ledger, aid(8));
    for approver in [aid(3), aid(4)] {
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

#[test]
#[should_panic(expected = "Expense must be approved before execution")]
fn execute_expense_must_be_approved_first() {
    let mut ledger = Ledger::new();
    proposed_expense(&mut ledger, aid(8));
    ledger.execute(
        &[meta(aid(1)), meta(aid(8)), meta(aid(2))],
        Instruction::ExecuteExpense {
            executed_evidence: "bafy-receipt".into(),
            amount: 120_000,
        },
    );
}

#[test]
#[should_panic(expected = "Expense belongs to a different SACCO")]
fn another_sacco_cannot_execute_this_expense() {
    let mut ledger = Ledger::new();
    proposed_expense(&mut ledger, aid(8));
    for approver in [aid(3), aid(4)] {
        ledger.execute(
            &[meta(aid(1)), meta(aid(8)), meta(approver)],
            Instruction::ApproveExpense,
        );
    }
    // SACCO B: admin aid(5), then try to execute A's expense under B.
    ledger.verified_sacco(aid(20), aid(5));
    ledger.set_approvers(aid(20), &[aid(5)], 1);
    ledger.execute(
        &[meta(aid(20)), meta(aid(8)), meta(aid(5))],
        Instruction::ExecuteExpense {
            executed_evidence: "bafy-receipt".into(),
            amount: 120_000,
        },
    );
}

#[test]
#[should_panic(expected = "Expense belongs to a different SACCO")]
fn another_sacco_cannot_approve_this_expense() {
    let mut ledger = Ledger::new();
    proposed_expense(&mut ledger, aid(8));
    ledger.verified_sacco(aid(20), aid(5));
    ledger.set_approvers(aid(20), &[aid(5)], 1);
    ledger.execute(
        &[meta(aid(20)), meta(aid(8)), meta(aid(5))],
        Instruction::ApproveExpense,
    );
}

#[test]
#[should_panic(expected = "Expense belongs to a different SACCO")]
fn another_sacco_cannot_reject_this_expense() {
    let mut ledger = Ledger::new();
    proposed_expense(&mut ledger, aid(8));
    ledger.verified_sacco(aid(20), aid(5));
    ledger.set_approvers(aid(20), &[aid(5)], 1);
    ledger.execute(
        &[meta(aid(20)), meta(aid(8)), meta(aid(5))],
        Instruction::RejectExpense,
    );
}
