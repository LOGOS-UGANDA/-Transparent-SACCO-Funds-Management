//! Effects: one per mutated shard, Borsh-encoded inside `ShardEffect`.

use borsh::{BorshDeserialize, BorshSerialize};

use crate::{AccountId, Deposit, DepositType, ProgramConfigVersion, Totals};

/// Every effect a `sacco_ledger` instruction can emit.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum Effect {
    Config(ConfigEffect),
    Sacco(SaccoEffect),
    Member(MemberEffect),
    Deposit(DepositEffect),
    Loan(LoanEffect),
    Expense(ExpenseEffect),
    Summary(SummaryEffect),
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
    AuthorizeProposal {
        required_approvals: u8,
    },
    AuthorizeApprover {
        approver: AccountId,
    },
    DisburseLoan {
        approver: AccountId,
        principal: u128,
    },
    RecordRepayment {
        approver: AccountId,
        amount: u128,
    },
    ProposeExpense {
        proposer: AccountId,
        amount: u128,
        required_approvals: u8,
    },
    ExecuteExpense {
        approver: AccountId,
        amount: u128,
    },
    HoldTotals {
        totals: Totals,
        poster: AccountId,
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
    AttachLoan {
        sacco: AccountId,
        loan: AccountId,
        signer: AccountId,
    },
}

/// Effects on a deposit shard.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum DepositEffect {
    Init(Deposit),
    Confirm { member: AccountId },
}

/// Effects on a loan shard.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum LoanEffect {
    Init {
        sacco: AccountId,
        member: AccountId,
        principal: u128,
        purpose: String,
        required_approvals: u8,
    },
    Approve {
        approver: AccountId,
    },
    Reject {
        approver: AccountId,
    },
    Disburse {
        evidence_cid: String,
        principal: u128,
    },
    Repay {
        amount: u128,
        reference: String,
    },
}

/// Effects on an expense shard.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum ExpenseEffect {
    Init {
        sacco: AccountId,
        amount: u128,
        purpose: String,
        evidence_cid: String,
        requested_by: AccountId,
        required_approvals: u8,
    },
    Approve {
        approver: AccountId,
    },
    Reject {
        approver: AccountId,
    },
    Execute {
        executed_evidence: String,
        amount: u128,
    },
}

/// Effects on a period-summary shard.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum SummaryEffect {
    Init {
        sacco: AccountId,
        label: String,
        totals: Totals,
    },
}
