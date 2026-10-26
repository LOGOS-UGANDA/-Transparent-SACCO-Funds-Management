//! The loan shard: lifecycle and the repayment cap.

use crate::codec::{decode, encode};
use crate::effect::LoanEffect;
use crate::{Loan, LoanStatus};

/// Applies a loan effect to the loan shard.
#[must_use]
pub fn apply(effect: LoanEffect, pre: &[u8]) -> Option<Vec<u8>> {
    match effect {
        LoanEffect::Init {
            sacco,
            member,
            principal,
            purpose,
            required_approvals,
        } => {
            assert!(pre.is_empty(), "Loan account must be uninitialized");
            assert!(principal > 0, "Loan amount must be greater than 0");
            assert!(!purpose.is_empty(), "Loan purpose must not be empty");
            Some(encode(&Loan {
                sacco_id: sacco,
                member_id: member,
                principal,
                purpose,
                status: LoanStatus::Applied,
                approvals: vec![],
                rejectors: vec![],
                evidence_cid: None,
                repaid_amount: 0,
                required_approvals,
            }))
        }
        LoanEffect::Approve { approver } => {
            let mut loan = decode_loan(pre);
            assert_eq!(
                loan.status,
                LoanStatus::Applied,
                "Loan is not in Applied state"
            );
            assert!(
                !loan.approvals.contains(&approver),
                "Already approved by this approver"
            );
            assert!(
                !loan.rejectors.contains(&approver),
                "Already rejected by this approver"
            );
            loan.approvals.push(approver);
            if loan.approvals.len() >= usize::from(loan.required_approvals) {
                loan.status = LoanStatus::Approved;
            }
            Some(encode(&loan))
        }
        LoanEffect::Reject { approver } => {
            let mut loan = decode_loan(pre);
            assert_eq!(
                loan.status,
                LoanStatus::Applied,
                "Loan is not in Applied state"
            );
            assert!(
                !loan.rejectors.contains(&approver),
                "Already rejected by this approver"
            );
            loan.rejectors.push(approver);
            loan.status = LoanStatus::Rejected;
            Some(encode(&loan))
        }
        LoanEffect::Disburse {
            evidence_cid,
            principal,
        } => {
            let mut loan = decode_loan(pre);
            assert_eq!(
                loan.status,
                LoanStatus::Approved,
                "Loan must be approved before disbursement"
            );
            assert!(
                !evidence_cid.is_empty(),
                "Disbursement evidence must not be empty"
            );
            assert_eq!(
                principal, loan.principal,
                "Principal does not match the loan"
            );
            loan.status = LoanStatus::Disbursed;
            loan.evidence_cid = Some(evidence_cid);
            Some(encode(&loan))
        }
        LoanEffect::Repay { amount, reference } => {
            let mut loan = decode_loan(pre);
            assert!(amount > 0, "Repayment must be greater than 0");
            assert!(
                !reference.is_empty(),
                "Repayment reference must not be empty"
            );
            assert!(
                loan.status == LoanStatus::Disbursed || loan.status == LoanStatus::Repaid,
                "Loan is not in Disbursed state"
            );
            assert!(
                loan.status != LoanStatus::Repaid || loan.repaid_amount < loan.principal,
                "Loan is already fully repaid"
            );
            let outstanding = loan
                .principal
                .checked_sub(loan.repaid_amount)
                .expect("repaid exceeds principal");
            assert!(
                amount <= outstanding,
                "Repayment exceeds outstanding principal"
            );
            loan.repaid_amount = loan
                .repaid_amount
                .checked_add(amount)
                .expect("repaid overflow");
            if loan.repaid_amount == loan.principal {
                loan.status = LoanStatus::Repaid;
            }
            Some(encode(&loan))
        }
    }
}

fn decode_loan(pre: &[u8]) -> Loan {
    decode(pre).expect("Loan shard must hold valid Loan data")
}

/// A disbursed loan for tests in this module.
#[cfg(test)]
pub fn disbursed(principal: u128) -> Loan {
    use crate::AccountId;

    Loan {
        sacco_id: AccountId::new([1; 32]),
        member_id: AccountId::new([6; 32]),
        principal,
        purpose: "school fees".into(),
        status: LoanStatus::Disbursed,
        approvals: vec![AccountId::new([2; 32])],
        rejectors: vec![],
        evidence_cid: Some("cid".into()),
        repaid_amount: 0,
        required_approvals: 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cap_is_the_outstanding_principal() {
        let pre = encode(&disbursed(100));
        assert!(apply(
            LoanEffect::Repay {
                amount: 100,
                reference: "ok".into()
            },
            &pre
        )
        .is_some());
    }

    #[test]
    #[should_panic(expected = "Repayment exceeds outstanding principal")]
    fn overpaying_the_outstanding_principal_is_refused() {
        let pre = encode(&disbursed(100));
        let _ = apply(
            LoanEffect::Repay {
                amount: 101,
                reference: "no".into(),
            },
            &pre,
        );
    }

    #[test]
    fn the_final_repayment_closes_the_loan() {
        let pre = encode(&disbursed(100));
        let post = apply(
            LoanEffect::Repay {
                amount: 100,
                reference: "ok".into(),
            },
            &pre,
        )
        .expect("writes");
        assert_eq!(decode_loan(&post).status, LoanStatus::Repaid);
    }
}
