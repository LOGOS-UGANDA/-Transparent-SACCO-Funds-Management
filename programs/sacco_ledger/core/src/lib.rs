//! Core types and instructions for the Transparent SACCO (`sacco_ledger`) LEZ program.
//!
//! The program is the record, not the custodian: value stays in mobile money and bank
//! accounts, and the chain attests to deposits, loans and expenses with multi-signature
//! approval. Account data uses Borsh in per-account shards. Instructions use serde for
//! the wallet JSON path and Borsh for the guest ABI.

pub mod codec;
pub mod effect;
pub mod transition;

use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

/// Registry layout version written by `Initialize`.
pub type ProgramConfigVersion = u8;

pub use nssa_core::account::AccountId;

/// Deposit record type: withdrawable savings vs member ownership (share capital).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize,
)]
#[borsh(use_discriminant = true)]
pub enum DepositType {
    Savings = 0,
    ShareCapital = 1,
}

/// Loan lifecycle state: `Applied -> Approved/Rejected -> Disbursed -> Repaid`.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize,
)]
#[borsh(use_discriminant = true)]
pub enum LoanStatus {
    Applied = 0,
    Approved = 1,
    Disbursed = 2,
    Repaid = 3,
    Rejected = 4,
}

/// Expense lifecycle state: `Proposed -> Approved/Rejected -> Executed`.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize,
)]
#[borsh(use_discriminant = true)]
pub enum ExpenseStatus {
    Proposed = 0,
    Approved = 1,
    Rejected = 2,
    Executed = 3,
}

/// A SACCO institution: public registration, totals and per-SACCO approver config.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Sacco {
    pub name: String,
    pub registration_number: String,
    pub admin: AccountId,
    /// Treasury contact for recorded disbursements (attestation reference).
    pub treasury: AccountId,
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

impl Sacco {
    /// A newly registered SACCO: one approver (the admin) and threshold 1.
    #[must_use]
    pub fn register(
        name: String,
        registration_number: String,
        treasury: AccountId,
        admin: AccountId,
    ) -> Self {
        Self {
            name,
            registration_number,
            admin,
            treasury,
            is_verified: false,
            is_active: true,
            total_deposits_savings: 0,
            total_deposits_share_capital: 0,
            total_loans_disbursed: 0,
            total_loans_repaid: 0,
            total_expenses: 0,
            required_approvals: 1,
            approvers: vec![admin],
            supervisory_committee: vec![],
            member_ids: vec![],
        }
    }
}

/// True when `id` is one of the SACCO's multi-sig approvers.
#[must_use]
pub fn is_approver(sacco: &Sacco, id: &AccountId) -> bool {
    sacco.approvers.contains(id)
}

/// A SACCO member: balances are public in Phase 1 and move to private accounts in the
/// privacy phase; the type is shaped so member data can move behind private accounts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Member {
    pub sacco_id: AccountId,
    pub identity: AccountId,
    pub savings_balance: u128,
    pub share_capital_balance: u128,
    pub loan_ids: Vec<AccountId>,
}

/// A recorded deposit: attests to off-chain money received for a member.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Loan {
    pub sacco_id: AccountId,
    pub member_id: AccountId,
    /// Identity that signed the application; cannot approve or reject its own loan.
    pub applicant: AccountId,
    pub principal: u128,
    pub purpose: String,
    pub status: LoanStatus,
    pub approvals: Vec<AccountId>,
    pub rejectors: Vec<AccountId>,
    /// Evidence CID (Logos Storage) attached at disbursement.
    pub evidence_cid: Option<String>,
    /// Accumulated repayments; capped at principal.
    pub repaid_amount: u128,
    /// Multi-sig threshold pinned when the loan was applied.
    pub required_approvals: u8,
}

/// A SACCO expense: proposed, multi-sig approved, executed by recording the off-chain release.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
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
    /// Multi-sig threshold pinned when the expense was proposed.
    pub required_approvals: u8,
}

/// Period snapshot of SACCO totals, posted by the treasurer and public for members.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
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

/// The SACCO's five public aggregate totals, as one portable value.
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    BorshSerialize,
    BorshDeserialize,
)]
pub struct Totals {
    pub deposits_savings: u128,
    pub deposits_share_capital: u128,
    pub loans_disbursed: u128,
    pub loans_repaid: u128,
    pub expenses: u128,
}

impl Totals {
    /// Reads the totals out of a SACCO account.
    #[must_use]
    pub fn of(sacco: &Sacco) -> Self {
        Self {
            deposits_savings: sacco.total_deposits_savings,
            deposits_share_capital: sacco.total_deposits_share_capital,
            loans_disbursed: sacco.total_loans_disbursed,
            loans_repaid: sacco.total_loans_repaid,
            expenses: sacco.total_expenses,
        }
    }
}

impl PeriodSummary {
    /// A snapshot of `totals` for `sacco` under `label`.
    #[must_use]
    pub fn new(sacco_id: AccountId, label: String, totals: Totals) -> Self {
        Self {
            sacco_id,
            label,
            deposits_savings: totals.deposits_savings,
            deposits_share_capital: totals.deposits_share_capital,
            loans_disbursed: totals.loans_disbursed,
            loans_repaid: totals.loans_repaid,
            expenses: totals.expenses,
        }
    }
}

/// SACCO ledger instructions. Account lists are documented per variant; handlers live
/// in `sacco_ledger_program`. The attestation model: value never moves on chain —
/// instructions record and approve off-chain money events.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum Instruction {
    /// Create the singleton SACCO registry config.
    ///
    /// Accounts:
    /// - Config account (uninitialized PDA)
    Initialize,

    /// Register a SACCO: public name, registration number and treasury contact.
    ///
    /// Accounts:
    /// - Sacco account (uninitialized, authorized)
    /// - Admin account (signer)
    RegisterSacco {
        name: String,
        registration_number: String,
        treasury: AccountId,
    },

    /// Mark a SACCO verified. In v1 the SACCO admin self-verifies.
    ///
    /// Accounts:
    /// - Sacco account (mut)
    /// - Admin account (signer)
    VerifySacco,

    /// Add a multi-sig approver (treasurer or committee member).
    ///
    /// Accounts:
    /// - Sacco account (mut)
    /// - Admin account (signer)
    AddApprover { approver: AccountId },

    /// Add a Supervisory Committee member (oversight, distinct from approvers).
    ///
    /// Accounts:
    /// - Sacco account (mut)
    /// - Admin account (signer)
    AddSupervisor { supervisor: AccountId },

    /// Register a member identity on the SACCO.
    ///
    /// Accounts:
    /// - Sacco account (mut)
    /// - Member account (uninitialized, authorized)
    /// - Admin account (signer)
    AddMember { identity: AccountId },

    /// Record an off-chain deposit for a member (treasurer attests, member confirms).
    ///
    /// Accounts:
    /// - Sacco account (mut)
    /// - Member account (mut)
    /// - Deposit account (uninitialized, authorized)
    /// - Approver account (signer)
    RecordDeposit {
        member: AccountId,
        amount: u128,
        deposit_type: DepositType,
        reference: String,
    },

    /// Member confirmation of a recorded deposit.
    ///
    /// Accounts:
    /// - Sacco account
    /// - Deposit account (mut)
    /// - Member account (signer)
    ConfirmDeposit,

    /// Member applies for a loan.
    ///
    /// Accounts:
    /// - Sacco account (mut)
    /// - Member account (mut)
    /// - Loan account (uninitialized, authorized)
    /// - Member identity account (signer)
    ApplyLoan {
        amount: u128,
        purpose: String,
        /// The threshold the SACCO had when the loan was applied.
        required_approvals: u8,
    },

    /// Committee multi-sig approval of a loan.
    ///
    /// Accounts:
    /// - Sacco account
    /// - Loan account (mut)
    /// - Approver account (signer)
    ApproveLoan,

    /// Committee rejection of a loan.
    ///
    /// Accounts:
    /// - Sacco account
    /// - Loan account (mut)
    /// - Approver account (signer)
    RejectLoan,

    /// Record loan disbursement (off-chain transfer) with storage evidence.
    ///
    /// Accounts:
    /// - Sacco account (mut)
    /// - Member account (mut)
    /// - Loan account (mut)
    /// - Approver account (signer)
    DisburseLoan {
        evidence_cid: String,
        /// The loan principal, re-asserted so the owning shard can validate it.
        principal: u128,
    },

    /// Record a loan repayment (off-chain transfer) against a disbursed loan.
    ///
    /// Accounts:
    /// - Sacco account (mut)
    /// - Member account (mut)
    /// - Loan account (mut)
    /// - Approver account (signer)
    RecordRepayment {
        loan: AccountId,
        amount: u128,
        reference: String,
    },

    /// Propose an expense against SACCO funds (treasurer).
    ///
    /// Accounts:
    /// - Sacco account (mut)
    /// - Expense account (uninitialized, authorized)
    /// - Approver account (signer)
    ProposeExpense {
        amount: u128,
        purpose: String,
        evidence_cid: String,
        /// The threshold the SACCO had when the expense was proposed.
        required_approvals: u8,
    },

    /// Committee multi-sig approval of a proposed expense.
    ///
    /// Accounts:
    /// - Sacco account
    /// - Expense account (mut)
    /// - Approver account (signer)
    ApproveExpense,

    /// Committee rejection of a proposed expense.
    ///
    /// Accounts:
    /// - Sacco account
    /// - Expense account (mut)
    /// - Approver account (signer)
    RejectExpense,

    /// Record execution of an approved expense: the off-chain payment reference.
    ///
    /// Accounts:
    /// - Sacco account (mut)
    /// - Expense account (mut)
    /// - Approver account (signer)
    ExecuteExpense {
        executed_evidence: String,
        /// The expense amount, re-asserted so the owning shard can validate it.
        amount: u128,
    },

    /// Post a period summary snapshot of SACCO totals (public for members).
    ///
    /// Accounts:
    /// - Sacco account
    /// - Period summary account (uninitialized, authorized)
    /// - Approver account (signer)
    PostPeriodSummary {
        label: String,
        /// The totals snapshot, re-asserted so the SACCO shard can validate it.
        totals: Totals,
    },
}

/// Seed for the singleton config PDA.
const CONFIG_PDA_SEED: &[u8] = b"CONFIG";

/// Singleton config stored at the `"CONFIG"` PDA.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ProgramConfig {
    /// Bumped when the registry layout changes; lets clients detect version.
    pub version: u8,
}

/// Derives the config PDA seed.
#[must_use]
pub fn compute_config_pda_seed() -> nssa_core::program::PdaSeed {
    use risc0_zkvm::sha::{Impl, Sha256};
    nssa_core::program::PdaSeed::new(
        Impl::hash_bytes(CONFIG_PDA_SEED)
            .as_bytes()
            .try_into()
            .expect("Hash output must be exactly 32 bytes long"),
    )
}

/// Derives the config PDA for this program's account id.
#[must_use]
pub fn compute_config_pda(program_account_id: AccountId) -> AccountId {
    AccountId::for_public_pda(&program_account_id, &compute_config_pda_seed())
}
