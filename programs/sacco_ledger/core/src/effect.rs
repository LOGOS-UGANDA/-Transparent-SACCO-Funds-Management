//! Effects: one per mutated shard, Borsh-encoded inside `ShardEffect`.

use borsh::{BorshDeserialize, BorshSerialize};

use crate::{AccountId, Deposit, DepositType, ProgramConfigVersion};

/// Every effect a `sacco_ledger` instruction can emit.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum Effect {
    Config(ConfigEffect),
    Sacco(SaccoEffect),
    Member(MemberEffect),
    Deposit(DepositEffect),
}

/// Effects on the singleton config shard.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum ConfigEffect {
    Init { version: ProgramConfigVersion },
}

/// Effects on a SACCO shard.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum SaccoEffect {
    Register {
        name: String,
        registration_number: String,
        treasury: AccountId,
        admin: AccountId,
    },
    Verify {
        registrar: AccountId,
    },
    AddApprover {
        admin: AccountId,
        approver: AccountId,
    },
    AddSupervisor {
        admin: AccountId,
        supervisor: AccountId,
    },
    AddMember {
        admin: AccountId,
        identity: AccountId,
    },
    RecordDeposit {
        recorder: AccountId,
        amount: u128,
        deposit_type: DepositType,
    },
}

/// Effects on a member shard.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum MemberEffect {
    Init {
        sacco: AccountId,
        identity: AccountId,
    },
    RecordDeposit {
        sacco: AccountId,
        amount: u128,
        deposit_type: DepositType,
    },
}

/// Effects on a deposit shard.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum DepositEffect {
    Init(Deposit),
    Confirm { member: AccountId },
}
