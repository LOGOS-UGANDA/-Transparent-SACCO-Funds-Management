//! The expense shard: proposal, approvals and the execution record.

use crate::codec::{decode, encode};
use crate::effect::ExpenseEffect;
use crate::{Expense, ExpenseStatus, MAX_APPROVERS};

/// Applies an expense effect to the expense shard.
///
/// Like the loan shard, every follow-up effect names the SACCO it belongs to
/// and the shard refuses effects aimed at another SACCO's expense.
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
        ExpenseEffect::Approve { sacco, approver } => {
            let mut expense = decode_expense(pre);
            assert_eq!(
                expense.sacco_id, sacco,
                "Expense belongs to a different SACCO"
            );
            assert_eq!(
                expense.status,
                ExpenseStatus::Proposed,
                "Expense is not proposed"
            );
            assert_ne!(
                approver, expense.requested_by,
                "Proposer cannot approve their own expense"
            );
            assert!(
                !expense.approvers.contains(&approver),
                "Already approved by this approver"
            );
            assert!(
                !expense.rejectors.contains(&approver),
                "Already rejected by this approver"
            );
            assert!(
                expense.approvers.len() < MAX_APPROVERS,
                "Approval limit reached"
            );
            expense.approvers.push(approver);
            if expense.approvers.len() >= usize::from(expense.required_approvals) {
                expense.status = ExpenseStatus::Approved;
            }
            Some(encode(&expense))
        }
        ExpenseEffect::Reject { sacco, approver } => {
            let mut expense = decode_expense(pre);
            assert_eq!(
                expense.sacco_id, sacco,
                "Expense belongs to a different SACCO"
            );
            assert_eq!(
                expense.status,
                ExpenseStatus::Proposed,
                "Expense is not proposed"
            );
            assert_ne!(
                approver, expense.requested_by,
                "Proposer cannot reject their own expense"
            );
            assert!(
                !expense.rejectors.contains(&approver),
                "Already rejected by this approver"
            );
            assert!(
                !expense.approvers.contains(&approver),
                "Approver already approved this expense"
            );
            assert!(
                expense.rejectors.len() < MAX_APPROVERS,
                "Rejection limit reached"
            );
            expense.rejectors.push(approver);
            expense.status = ExpenseStatus::Rejected;
            Some(encode(&expense))
        }
        ExpenseEffect::Execute {
            sacco,
            executed_evidence,
            amount,
        } => {
            let mut expense = decode_expense(pre);
            assert_eq!(
                expense.sacco_id, sacco,
                "Expense belongs to a different SACCO"
            );
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
        approvers: vec![AccountId::new([3; 32])],
        rejectors: vec![],
        executed_evidence: None,
        required_approvals: 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AccountId;

    fn sacco() -> AccountId {
        AccountId::new([1; 32])
    }

    #[test]
    #[should_panic(expected = "Amount does not match the expense")]
    fn the_amount_argument_must_match_the_expense() {
        let pre = encode(&approved(100));
        let _ = apply(
            ExpenseEffect::Execute {
                sacco: sacco(),
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
                sacco: sacco(),
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

    #[test]
    #[should_panic(expected = "Expense belongs to a different SACCO")]
    fn another_saccos_approver_cannot_execute_this_expense() {
        let pre = encode(&approved(100));
        let _ = apply(
            ExpenseEffect::Execute {
                sacco: AccountId::new([9; 32]),
                executed_evidence: "receipt".into(),
                amount: 100,
            },
            &pre,
        );
    }

    #[test]
    #[should_panic(expected = "Proposer cannot approve their own expense")]
    fn the_proposer_cannot_approve_their_own_expense() {
        let expense = Expense {
            status: ExpenseStatus::Proposed,
            approvers: vec![],
            ..approved(100)
        };
        let _ = apply(
            ExpenseEffect::Approve {
                sacco: sacco(),
                approver: AccountId::new([2; 32]),
            },
            &encode(&expense),
        );
    }

    #[test]
    #[should_panic(expected = "Approver already approved this expense")]
    fn an_approver_cannot_reject_after_approving() {
        let expense = Expense {
            status: ExpenseStatus::Proposed,
            approvers: vec![AccountId::new([3; 32])],
            ..approved(100)
        };
        let _ = apply(
            ExpenseEffect::Reject {
                sacco: sacco(),
                approver: AccountId::new([3; 32]),
            },
            &encode(&expense),
        );
    }

    #[test]
    fn reaching_the_threshold_approves_the_expense() {
        let expense = Expense {
            status: ExpenseStatus::Proposed,
            approvers: vec![],
            required_approvals: 2,
            ..approved(100)
        };
        let once = apply(
            ExpenseEffect::Approve {
                sacco: sacco(),
                approver: AccountId::new([3; 32]),
            },
            &encode(&expense),
        )
        .expect("writes");
        assert_eq!(decode_expense(&once).status, ExpenseStatus::Proposed);
        let twice = apply(
            ExpenseEffect::Approve {
                sacco: sacco(),
                approver: AccountId::new([4; 32]),
            },
            &once,
        )
        .expect("writes");
        assert_eq!(decode_expense(&twice).status, ExpenseStatus::Approved);
    }
}
