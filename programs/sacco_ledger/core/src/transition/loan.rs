//! The loan shard: lifecycle, ownership and the repayment cap.

use crate::codec::{decode, encode};
use crate::effect::LoanEffect;
use crate::{Loan, LoanStatus};

/// Applies a loan effect to the loan shard.
///
/// The SACCO shard authorizes approvers, but it cannot read this account, so
/// every transition re-checks that the effect names the SACCO that owns the
/// loan and the member account it was opened for.
#[must_use]
pub fn apply(effect: LoanEffect, pre: &[u8]) -> Option<Vec<u8>> {
    match effect {
        LoanEffect::Init {
            sacco,
            member,
            applicant,
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
                applicant,
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
        LoanEffect::Approve { sacco, approver } => {
            let mut loan = decode_loan(pre);
            assert_eq!(loan.sacco_id, sacco, "Loan belongs to a different SACCO");
            assert_eq!(
                loan.status,
                LoanStatus::Applied,
                "Loan is not in Applied state"
            );
            assert_ne!(
                approver, loan.applicant,
                "Applicant cannot approve their own loan"
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
        LoanEffect::Reject { sacco, approver } => {
            let mut loan = decode_loan(pre);
            assert_eq!(loan.sacco_id, sacco, "Loan belongs to a different SACCO");
            assert_eq!(
                loan.status,
                LoanStatus::Applied,
                "Loan is not in Applied state"
            );
            assert_ne!(
                approver, loan.applicant,
                "Applicant cannot reject their own loan"
            );
            assert!(
                !loan.rejectors.contains(&approver),
                "Already rejected by this approver"
            );
            assert!(
                !loan.approvals.contains(&approver),
                "Approver already approved this loan"
            );
            loan.rejectors.push(approver);
            loan.status = LoanStatus::Rejected;
            Some(encode(&loan))
        }
        LoanEffect::Disburse {
            sacco,
            member,
            evidence_cid,
            principal,
        } => {
            let mut loan = decode_loan(pre);
            assert_eq!(loan.sacco_id, sacco, "Loan belongs to a different SACCO");
            assert_eq!(
                loan.member_id, member,
                "Member account does not match the loan"
            );
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
        LoanEffect::Repay {
            sacco,
            member,
            amount,
            reference,
        } => {
            let mut loan = decode_loan(pre);
            assert_eq!(loan.sacco_id, sacco, "Loan belongs to a different SACCO");
            assert_eq!(
                loan.member_id, member,
                "Member account does not match the loan"
            );
            assert!(amount > 0, "Repayment must be greater than 0");
            assert!(
                !reference.is_empty(),
                "Repayment reference must not be empty"
            );
            assert_eq!(
                loan.status,
                LoanStatus::Disbursed,
                "Loan is not in Disbursed state"
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
        applicant: AccountId::new([6; 32]),
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
    use crate::AccountId;

    fn sacco() -> AccountId {
        AccountId::new([1; 32])
    }

    fn member() -> AccountId {
        AccountId::new([6; 32])
    }

    #[test]
    fn the_cap_is_the_outstanding_principal() {
        let pre = encode(&disbursed(100));
        assert!(apply(
            LoanEffect::Repay {
                sacco: sacco(),
                member: member(),
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
                sacco: sacco(),
                member: member(),
                amount: 101,
                reference: "no".into(),
            },
            &pre,
        );
    }

    #[test]
    #[should_panic(expected = "Loan belongs to a different SACCO")]
    fn a_loan_cannot_be_repaid_through_another_sacco() {
        let pre = encode(&disbursed(100));
        let _ = apply(
            LoanEffect::Repay {
                sacco: AccountId::new([9; 32]),
                member: member(),
                amount: 1,
                reference: "no".into(),
            },
            &pre,
        );
    }

    #[test]
    #[should_panic(expected = "Member account does not match the loan")]
    fn a_loan_cannot_be_repaid_for_another_member() {
        let pre = encode(&disbursed(100));
        let _ = apply(
            LoanEffect::Repay {
                sacco: sacco(),
                member: AccountId::new([7; 32]),
                amount: 1,
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
                sacco: sacco(),
                member: member(),
                amount: 100,
                reference: "ok".into(),
            },
            &pre,
        )
        .expect("writes");
        assert_eq!(decode_loan(&post).status, LoanStatus::Repaid);
    }

    #[test]
    #[should_panic(expected = "Approver already approved this loan")]
    fn an_approver_cannot_reject_after_approving() {
        let loan = Loan {
            status: LoanStatus::Applied,
            approvals: vec![AccountId::new([2; 32])],
            ..disbursed(100)
        };
        let _ = apply(
            LoanEffect::Reject {
                sacco: sacco(),
                approver: AccountId::new([2; 32]),
            },
            &encode(&loan),
        );
    }

    #[test]
    #[should_panic(expected = "Applicant cannot approve their own loan")]
    fn an_applicant_cannot_approve_their_own_loan() {
        let loan = Loan {
            status: LoanStatus::Applied,
            approvals: vec![],
            ..disbursed(100)
        };
        let _ = apply(
            LoanEffect::Approve {
                sacco: sacco(),
                approver: member(),
            },
            &encode(&loan),
        );
    }

    #[test]
    fn reaching_the_threshold_approves_the_loan() {
        let loan = Loan {
            status: LoanStatus::Applied,
            approvals: vec![],
            required_approvals: 2,
            ..disbursed(100)
        };
        let once = apply(
            LoanEffect::Approve {
                sacco: sacco(),
                approver: AccountId::new([2; 32]),
            },
            &encode(&loan),
        )
        .expect("writes");
        assert_eq!(decode_loan(&once).status, LoanStatus::Applied);
        let twice = apply(
            LoanEffect::Approve {
                sacco: sacco(),
                approver: AccountId::new([3; 32]),
            },
            &once,
        )
        .expect("writes");
        assert_eq!(decode_loan(&twice).status, LoanStatus::Approved);
    }
}
