//! Core types and instructions for the Transparent SACCO (`sacco_ledger`) LEZ program.
//!
//! The program is the record, not the custodian: value stays in mobile money and bank
//! accounts, and the chain attests to deposits, loans and expenses with multi-signature
//! approval. Account data uses Borsh. Instructions use serde (SPEL / wallet path).
//!
//! The earlier university fee-transparency custody model (vault PDA + category ledger)
//! is kept dormant in [`custody`] — reference code, not dispatched by the active
//! instruction set.

pub mod custody;

use borsh::{BorshDeserialize, BorshSerialize};
use nssa_core::account::AccountId;
use serde::{Deserialize, Serialize};
use spel_framework_macros::account_type;

/// Deposit record type: withdrawable savings vs member ownership (share capital).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[borsh(use_discriminant = true)]
pub enum DepositType {
    Savings = 0,
    ShareCapital = 1,
}

/// Loan lifecycle state: `Applied -> Approved/Rejected -> Disbursed -> Repaid`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[borsh(use_discriminant = true)]
pub enum LoanStatus {
    Applied = 0,
    Approved = 1,
    Disbursed = 2,
    Repaid = 3,
    Rejected = 4,
}

/// Expense lifecycle state: `Proposed -> Approved/Rejected -> Executed`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[borsh(use_discriminant = true)]
pub enum ExpenseStatus {
    Proposed = 0,
    Approved = 1,
    Rejected = 2,
    Executed = 3,
}

/// A SACCO institution: public registration, totals and per-SACCO approver config.
#[account_type]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Sacco {
    pub name: String,
    pub registration_number: String,
    pub admin: AccountId,
    pub is_verified: bool,
    pub is_active: bool,
    /// Total recorded savings deposits (attested, off-chain value).
    pub total_deposits_savings: u128,
    /// Total recorded share-capital deposits.
    pub total_deposits_share_capital: u128,
    /// Total loan principal disbursed.
    pub total_loans_disbursed: u128,
    /// Total loan repayments recorded.
    pub total_loans_repaid: u128,
    /// Total expenses executed (recorded release of off-chain funds).
    pub total_expenses: u128,
    /// Multi-sig threshold for loans and expenses, configured per SACCO.
    pub required_approvals: u8,
    /// Approvers (treasurer and committee members) allowed to record and approve.
    pub approvers: Vec<AccountId>,
    /// Supervisory Committee: oversight role, separate from approvers.
    pub supervisory_committee: Vec<AccountId>,
    pub member_ids: Vec<AccountId>,
}

/// A SACCO member: balances are public in Phase 1 and move to private accounts in the
/// privacy phase; the type is shaped so member data can move behind private accounts.
#[account_type]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Member {
    pub sacco_id: AccountId,
    pub identity: AccountId,
    pub savings_balance: u128,
    pub share_capital_balance: u128,
    pub loan_ids: Vec<AccountId>,
}

/// A recorded deposit: attests to off-chain money received for a member.
#[account_type]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Deposit {
    pub sacco_id: AccountId,
    pub member_id: AccountId,
    pub amount: u128,
    pub deposit_type: DepositType,
    /// Mobile-money / bank reference for the off-chain transfer.
    pub reference: String,
    /// Approver (treasurer) who recorded the deposit.
    pub recorded_by: AccountId,
    /// Set when the member confirms the record.
    pub confirmed_by_member: bool,
}

/// A member loan: applied, multi-sig approved, disbursed with evidence, repaid.
#[account_type]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Loan {
    pub sacco_id: AccountId,
    pub member_id: AccountId,
    pub principal: u128,
    pub purpose: String,
    pub status: LoanStatus,
    pub approvals: Vec<AccountId>,
    pub rejectors: Vec<AccountId>,
    /// Evidence CID (Logos Storage) attached at disbursement.
    pub evidence_cid: Option<String>,
    /// Accumulated repayments; capped at principal.
    pub repaid_amount: u128,
}

/// A SACCO expense: proposed, multi-sig approved, executed by recording the off-chain release.
#[account_type]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Expense {
    pub sacco_id: AccountId,
    pub amount: u128,
    pub purpose: String,
    /// Evidence CID (Logos Storage) for the stated purpose.
    pub evidence_cid: String,
    pub requested_by: AccountId,
    pub status: ExpenseStatus,
    pub approvers: Vec<AccountId>,
    pub rejectors: Vec<AccountId>,
    /// Evidence recorded at execution: proof the off-chain payment happened.
    pub executed_evidence: Option<String>,
}

/// Period snapshot of SACCO totals, posted by the treasurer and public for members.
#[account_type]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct PeriodSummary {
    pub sacco_id: AccountId,
    /// Period label, e.g. `2026-11`.
    pub label: String,
    pub deposits_savings: u128,
    pub deposits_share_capital: u128,
    pub loans_disbursed: u128,
    pub loans_repaid: u128,
    pub expenses: u128,
}

macro_rules! impl_data_codec {
    ($ty:ty) => {
        impl TryFrom<&nssa_core::account::Data> for $ty {
            type Error = std::io::Error;

            fn try_from(data: &nssa_core::account::Data) -> Result<Self, Self::Error> {
                <$ty>::try_from_slice(data.as_ref())
            }
        }

        impl From<&$ty> for nssa_core::account::Data {
            fn from(value: &$ty) -> Self {
                let mut data = Vec::with_capacity(std::mem::size_of_val(value));
                BorshSerialize::serialize(value, &mut data)
                    .expect("Serialization to Vec should not fail");
                nssa_core::account::Data::try_from(data)
                    .expect("encoded account data should fit into Data")
            }
        }
    };
}

impl_data_codec!(Sacco);
impl_data_codec!(Member);
impl_data_codec!(Deposit);
impl_data_codec!(Loan);
impl_data_codec!(Expense);
impl_data_codec!(PeriodSummary);
