//! The member shard: identity and balances.

use crate::codec::{decode, encode};
use crate::effect::MemberEffect;
use crate::{AccountId, Member};

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
