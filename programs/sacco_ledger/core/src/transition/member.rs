//! The member shard: identity and balances.

use crate::codec::{decode, encode};
use crate::effect::MemberEffect;
use crate::{AccountId, Member, MAX_MEMBER_LOANS};

/// Applies a member effect to the member shard.
#[must_use]
pub fn apply(effect: MemberEffect, pre: &[u8]) -> Option<Vec<u8>> {
    match effect {
        MemberEffect::Init { sacco, identity } => {
            assert!(pre.is_empty(), "Member account must be uninitialized");
            Some(encode(&Member {
                sacco_id: sacco,
                identity,
                savings_balance: 0,
                share_capital_balance: 0,
                loan_ids: vec![],
            }))
        }
        MemberEffect::RecordDeposit {
            sacco,
            recorder,
            amount,
            deposit_type,
        } => {
            let mut member = decode_member(pre, sacco);
            assert_ne!(
                member.identity, recorder,
                "A member cannot record their own deposit"
            );
            match deposit_type {
                crate::DepositType::Savings => {
                    member.savings_balance = member
                        .savings_balance
                        .checked_add(amount)
                        .expect("savings balance overflow");
                }
                crate::DepositType::ShareCapital => {
                    member.share_capital_balance = member
                        .share_capital_balance
                        .checked_add(amount)
                        .expect("share capital balance overflow");
                }
            }
            Some(encode(&member))
        }
        MemberEffect::AttachLoan {
            sacco,
            loan,
            signer,
        } => {
            let mut member = decode_member(pre, sacco);
            assert_eq!(
                member.identity, signer,
                "Only the member can apply for their loan"
            );
            assert!(
                !member.loan_ids.contains(&loan),
                "Loan already attached to this member"
            );
            assert!(
                member.loan_ids.len() < MAX_MEMBER_LOANS,
                "Member loan limit reached"
            );
            member.loan_ids.push(loan);
            Some(encode(&member))
        }
        MemberEffect::AuthorizeIdentity { signer } => {
            let member: Member = decode(pre).expect("Member shard must hold valid Member data");
            assert_eq!(
                member.identity, signer,
                "Only the member can confirm their deposit"
            );
            None
        }
    }
}

/// Decodes a member shard and asserts it belongs to `sacco`.
#[must_use]
pub fn decode_member(pre: &[u8], sacco: AccountId) -> Member {
    let member: Member = decode(pre).expect("Member shard must hold valid Member data");
    assert_eq!(
        member.sacco_id, sacco,
        "Member belongs to a different SACCO"
    );
    member
}
