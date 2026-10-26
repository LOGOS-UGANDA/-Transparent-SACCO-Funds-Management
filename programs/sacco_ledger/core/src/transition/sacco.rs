//! The SACCO shard: registration, verification and role composition.

use crate::codec::{decode, encode};
use crate::effect::SaccoEffect;
use crate::{is_approver, DepositType, Sacco};

/// Applies a SACCO effect to the SACCO shard.
#[must_use]
pub fn apply(effect: SaccoEffect, pre: &[u8]) -> Option<Vec<u8>> {
    match effect {
        SaccoEffect::Register {
            name,
            registration_number,
            treasury,
            admin,
        } => {
            assert!(pre.is_empty(), "Sacco account must be uninitialized");
            assert!(!name.is_empty(), "Sacco name must not be empty");
            assert!(
                !registration_number.is_empty(),
                "Registration number must not be empty"
            );
            Some(encode(&Sacco::register(
                name,
                registration_number,
                treasury,
                admin,
            )))
        }
        SaccoEffect::Verify { registrar } => {
            let mut sacco = decode_sacco(pre);
            assert!(sacco.is_active, "Sacco must be active");
            assert_eq!(
                sacco.admin, registrar,
                "Only the SACCO admin can verify in v1"
            );
            sacco.is_verified = true;
            Some(encode(&sacco))
        }
        SaccoEffect::AddApprover { admin, approver } => {
            let mut sacco = decode_sacco(pre);
            assert_eq!(sacco.admin, admin, "Only the SACCO admin can add approvers");
            assert!(
                !sacco.approvers.contains(&approver),
                "Approver already registered"
            );
            assert!(
                !sacco.supervisory_committee.contains(&approver),
                "Supervisory Committee members cannot also be approvers"
            );
            sacco.approvers.push(approver);
            Some(encode(&sacco))
        }
        SaccoEffect::AddSupervisor { admin, supervisor } => {
            let mut sacco = decode_sacco(pre);
            assert_eq!(
                sacco.admin, admin,
                "Only the SACCO admin can add supervisors"
            );
            assert!(
                !sacco.supervisory_committee.contains(&supervisor),
                "Supervisor already registered"
            );
            assert!(
                !sacco.approvers.contains(&supervisor),
                "Approvers cannot also sit on the Supervisory Committee"
            );
            sacco.supervisory_committee.push(supervisor);
            Some(encode(&sacco))
        }
        SaccoEffect::AddMember { admin, identity } => {
            let mut sacco = decode_sacco(pre);
            assert_eq!(sacco.admin, admin, "Only the SACCO admin can add members");
            assert!(
                !sacco.member_ids.contains(&identity),
                "Member identity already registered"
            );
            sacco.member_ids.push(identity);
            Some(encode(&sacco))
        }
        SaccoEffect::RecordDeposit {
            recorder,
            amount,
            deposit_type,
        } => {
            let mut sacco = decode_sacco(pre);
            assert!(sacco.is_verified, "Sacco is not verified");
            assert!(
                is_approver(&sacco, &recorder),
                "Only SACCO approvers can record deposits"
            );
            assert!(amount > 0, "Deposit amount must be greater than 0");
            match deposit_type {
                DepositType::Savings => {
                    sacco.total_deposits_savings = sacco
                        .total_deposits_savings
                        .checked_add(amount)
                        .expect("savings total overflow");
                }
                DepositType::ShareCapital => {
                    sacco.total_deposits_share_capital = sacco
                        .total_deposits_share_capital
                        .checked_add(amount)
                        .expect("share capital total overflow");
                }
            }
            Some(encode(&sacco))
        }
        SaccoEffect::AuthorizeProposal { required_approvals } => {
            let sacco = decode_sacco(pre);
            assert!(sacco.is_verified, "Sacco is not verified");
            assert_eq!(
                required_approvals, sacco.required_approvals,
                "Approver threshold does not match the SACCO configuration"
            );
            None
        }
        SaccoEffect::AuthorizeApprover { approver } => {
            let sacco = decode_sacco(pre);
            assert!(sacco.is_verified, "Sacco is not verified");
            assert!(is_approver(&sacco, &approver), "Not a SACCO approver");
            None
        }
        SaccoEffect::DisburseLoan {
            approver,
            principal,
        } => {
            let mut sacco = decode_sacco(pre);
            assert!(sacco.is_verified, "Sacco is not verified");
            assert!(is_approver(&sacco, &approver), "Not a SACCO approver");
            sacco.total_loans_disbursed = sacco
                .total_loans_disbursed
                .checked_add(principal)
                .expect("loans disbursed overflow");
            Some(encode(&sacco))
        }
        SaccoEffect::RecordRepayment { approver, amount } => {
            let mut sacco = decode_sacco(pre);
            assert!(sacco.is_verified, "Sacco is not verified");
            assert!(is_approver(&sacco, &approver), "Not a SACCO approver");
            sacco.total_loans_repaid = sacco
                .total_loans_repaid
                .checked_add(amount)
                .expect("loans repaid overflow");
            Some(encode(&sacco))
        }
    }
}

fn decode_sacco(pre: &[u8]) -> Sacco {
    decode(pre).expect("Sacco shard must hold valid Sacco data")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AccountId;

    fn registered() -> Sacco {
        Sacco::register(
            "Kampala Traders SACCO".into(),
            "UG-SACCO-001".into(),
            AccountId::new([9; 32]),
            AccountId::new([2; 32]),
        )
    }

    #[test]
    #[should_panic(expected = "Sacco account must be uninitialized")]
    fn register_is_write_once() {
        let pre = encode(&registered());
        let effect = SaccoEffect::Register {
            name: "Other".into(),
            registration_number: "UG-2".into(),
            treasury: AccountId::new([8; 32]),
            admin: AccountId::new([3; 32]),
        };
        let _ = apply(effect, &pre);
    }

    #[test]
    #[should_panic(expected = "Only the SACCO admin can add approvers")]
    fn only_the_admin_adds_approvers() {
        let pre = encode(&registered());
        let _ = apply(
            SaccoEffect::AddApprover {
                admin: AccountId::new([7; 32]),
                approver: AccountId::new([3; 32]),
            },
            &pre,
        );
    }

    #[test]
    fn adding_an_approver_keeps_the_shard_readable() {
        let pre = encode(&registered());
        let post = apply(
            SaccoEffect::AddApprover {
                admin: AccountId::new([2; 32]),
                approver: AccountId::new([3; 32]),
            },
            &pre,
        )
        .expect("writes");
        let sacco: Sacco = decode(&post).expect("sacco data");
        assert_eq!(
            sacco.approvers,
            vec![AccountId::new([2; 32]), AccountId::new([3; 32])]
        );
    }
}
