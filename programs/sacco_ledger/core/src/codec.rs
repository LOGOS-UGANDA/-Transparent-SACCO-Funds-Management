//! Borsh helpers for shard-encoded account data.
//!
//! A shard holds the Borsh encoding of exactly one account type. `decode` returns
//! `Result` so the caller names the rule that the data must satisfy.

use borsh::{BorshDeserialize, BorshSerialize};

/// Encodes an account value for storage in its shard.
#[must_use]
pub fn encode<T: BorshSerialize>(value: &T) -> Vec<u8> {
    borsh::to_vec(value).expect("borsh serialization is infallible")
}

/// Decodes a shard's bytes into an account value.
///
/// # Errors
/// Returns the Borsh error when the shard does not hold a valid `T`.
pub fn decode<T: BorshDeserialize>(data: &[u8]) -> Result<T, std::io::Error> {
    borsh::from_slice(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AccountId;
    use crate::Sacco;

    #[test]
    fn a_sacco_round_trips_through_its_shard() {
        let sacco = Sacco::register(
            "Kampala Traders SACCO".into(),
            "UG-SACCO-001".into(),
            AccountId::new([9; 32]),
            AccountId::new([2; 32]),
        );
        let decoded: Sacco = decode(&encode(&sacco)).expect("round trip");
        assert_eq!(decoded, sacco);
    }

    #[test]
    fn garbage_shard_data_is_an_error_not_a_panic() {
        assert!(decode::<Sacco>(&[0xFF; 3]).is_err());
    }
}
