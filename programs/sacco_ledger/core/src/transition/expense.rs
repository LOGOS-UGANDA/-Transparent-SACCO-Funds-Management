//! The expense shard: proposal, approvals and the execution record.

use crate::codec::{decode, encode};
use crate::effect::ExpenseEffect;
use crate::{Expense, ExpenseStatus};

/// Applies an expense effect to the expense shard.
#[must_use]
pub fn apply(effect: ExpenseEffect, pre: &[u8]) -> Option<Vec<u8>> {
    match effect {
        ExpenseEffect::Init {
            sacco,
            amount,
            purpose,
            evidence_cid,
            requested_by,
            required_approvals,
        } => {
            assert!(pre.is_empty(), "Expense account must be uninitialized");
            assert!(amount > 0, "Expense amount must be greater than 0");
            assert!(!purpose.is_empty(), "Expense purpose must not be empty");
            assert!(
                !evidence_cid.is_empty(),
                "Expense evidence must not be empty"
            );
            Some(encode(&Expense {
                sacco_id: sacco,
                amount,
                purpose,
                evidence_cid,
                requested_by,
                status: ExpenseStatus::Proposed,
                approvers: vec![],
                rejectors: vec![],
                executed_evidence: None,
                required_approvals,
            }))
        }
        ExpenseEffect::Approve { approver } => {
            let mut expense = decode_expense(pre);
            assert_eq!(
                expense.status,
                ExpenseStatus::Proposed,
                "Expense is not proposed"
            );
            assert!(
                !expense.approvers.contains(&approver),
                "Already approved by this approver"
            );
            assert!(
                !expense.rejectors.contains(&approver),
                "Already rejected by this approver"
            );
            expense.approvers.push(approver);
            if expense.approvers.len() >= usize::from(expense.required_approvals) {
                expense.status = ExpenseStatus::Approved;
            }
            Some(encode(&expense))
        }
        ExpenseEffect::Reject { approver } => {
            let mut expense = decode_expense(pre);
            assert_eq!(
                expense.status,
                ExpenseStatus::Proposed,
                "Expense is not proposed"
            );
            assert!(
                !expense.rejectors.contains(&approver),
                "Already rejected by this approver"
            );
            expense.rejectors.push(approver);
            expense.status = ExpenseStatus::Rejected;
            Some(encode(&expense))
        }
        ExpenseEffect::Execute {
            executed_evidence,
            amount,
        } => {
            let mut expense = decode_expense(pre);
            assert_eq!(
                expense.status,
                ExpenseStatus::Approved,
                "Expense must be approved before execution"
            );
            assert!(
                !executed_evidence.is_empty(),
                "Execution evidence must not be empty"
            );
            assert_eq!(amount, expense.amount, "Amount does not match the expense");
            expense.status = ExpenseStatus::Executed;
            expense.executed_evidence = Some(executed_evidence);
            Some(encode(&expense))
        }
    }
}

fn decode_expense(pre: &[u8]) -> Expense {
    decode(pre).expect("Expense shard must hold valid Expense data")
}

/// An approved expense for tests in this module.
#[cfg(test)]
pub fn approved(amount: u128) -> Expense {
    use crate::AccountId;

    Expense {
        sacco_id: AccountId::new([1; 32]),
        amount,
        purpose: "desk".into(),
        evidence_cid: "cid".into(),
        requested_by: AccountId::new([2; 32]),
        status: ExpenseStatus::Approved,
        approvers: vec![AccountId::new([2; 32])],
        rejectors: vec![],
        executed_evidence: None,
        required_approvals: 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Amount does not match the expense")]
    fn the_amount_argument_must_match_the_expense() {
        let pre = encode(&approved(100));
        let _ = apply(
            ExpenseEffect::Execute {
                executed_evidence: "receipt".into(),
                amount: 101,
            },
            &pre,
        );
    }

    #[test]
    fn executing_a_matching_expense_records_the_evidence() {
        let pre = encode(&approved(100));
        let post = apply(
            ExpenseEffect::Execute {
                executed_evidence: "receipt".into(),
                amount: 100,
            },
            &pre,
        )
        .expect("writes");
        let expense = decode_expense(&post);
        assert_eq!(expense.status, ExpenseStatus::Executed);
        assert_eq!(expense.executed_evidence.as_deref(), Some("receipt"));
    }
}
