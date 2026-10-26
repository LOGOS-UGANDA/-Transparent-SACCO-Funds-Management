//! Effects: one per mutated shard, Borsh-encoded inside `ShardEffect`.

use borsh::{BorshDeserialize, BorshSerialize};

use crate::ProgramConfigVersion;

/// Every effect a `sacco_ledger` instruction can emit.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum Effect {
    Config(ConfigEffect),
}

/// Effects on the singleton config shard.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum ConfigEffect {
    Init { version: ProgramConfigVersion },
}
