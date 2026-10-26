//! The deposit shard: one attested off-chain payment.

use crate::codec::{decode, encode};
use crate::effect::DepositEffect;
use crate::Deposit;

/// Applies a deposit effect to the deposit shard.
#[must_use]
pub fn apply(effect: DepositEffect, pre: &[u8]) -> Option<Vec<u8>> {
    match effect {
        DepositEffect::Init(deposit) => {
            assert!(pre.is_empty(), "Deposit account must be uninitialized");
            assert!(
                !deposit.reference.is_empty(),
                "Deposit reference must not be empty"
            );
            assert!(deposit.amount > 0, "Deposit amount must be greater than 0");
            Some(encode(&deposit))
        }
        DepositEffect::Confirm { member } => {
            let mut deposit: Deposit =
                decode(pre).expect("Deposit shard must hold valid Deposit data");
            assert_eq!(
                deposit.member_id, member,
                "Only the deposit's member can confirm"
            );
            assert!(!deposit.confirmed_by_member, "Deposit already confirmed");
            deposit.confirmed_by_member = true;
            Some(encode(&deposit))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AccountId;

    fn deposit() -> Deposit {
        Deposit {
            sacco_id: AccountId::new([1; 32]),
            member_id: AccountId::new([6; 32]),
            amount: 100,
            deposit_type: crate::DepositType::Savings,
            reference: "MPZA-1".into(),
            recorded_by: AccountId::new([2; 32]),
            confirmed_by_member: false,
        }
    }

    #[test]
    #[should_panic(expected = "Only the deposit's member can confirm")]
    fn only_the_member_confirms() {
        let pre = encode(&deposit());
        let _ = apply(
            DepositEffect::Confirm {
                member: AccountId::new([3; 32]),
            },
            &pre,
        );
    }

    #[test]
    fn a_member_can_confirm_a_deposit() {
        let pre = encode(&deposit());
        let post = apply(
            DepositEffect::Confirm {
                member: AccountId::new([6; 32]),
            },
            &pre,
        )
        .expect("writes");
        assert!(
            decode::<Deposit>(&post)
                .expect("deposit data")
                .confirmed_by_member
        );
    }
}
