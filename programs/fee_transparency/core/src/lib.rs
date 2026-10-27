//! Core types and instructions for the Transparent SACCO fee-transparency LEZ program.
//!
//! Account data uses Borsh. Instructions use serde (SPEL / wallet path).
//! Prefer public LEZ accounts so payers can audit allocations.
//!
//! Custody model: see `docs/custody.md` — institution vault PDA + category ledger.

use borsh::{BorshDeserialize, BorshSerialize};
use nssa_core::{
    account::{AccountId, Data},
    program::{PdaSeed, ProgramId},
};
use serde::{Deserialize, Serialize};
use spel_framework_macros::account_type;

/// Basis points denominator: 10_000 = 100%.
pub const BPS_DENOMINATOR: u16 = 10_000;

/// Max category accounts accepted by the `PayFees` guest entry (v1).
pub const MAX_PAY_FEE_CATEGORIES: usize = 8;

const CONFIG_PDA_SEED: &[u8] = b"CONFIG";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum InstitutionType {
    University = 0,
    Sacco = 1,
    Cooperative = 2,
    School = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum FeeStatus {
    Locked = 0,
    PartiallyReleased = 1,
    FullyReleased = 2,
    Refunded = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum ReleaseStatus {
    Pending = 0,
    Approved = 1,
    Rejected = 2,
    Executed = 3,
}

/// Program instructions. Account lists are documented per variant; handlers live in
/// `fee_transparency_program`.
#[derive(Serialize, Deserialize)]
pub enum Instruction {
    /// Create the singleton program config PDA and store the Token Program id.
    ///
    /// Accounts:
    /// - Config account (uninitialized PDA for `"CONFIG"`)
    Initialize { token_program_id: ProgramId },

    /// Create a new institution registry account.
    ///
    /// Accounts:
    /// - Institution account (uninitialized, authorized)
    /// - Admin account (signer)
    RegisterInstitution {
        name: String,
        registration_number: String,
        institution_type: InstitutionType,
        treasury: AccountId,
        required_approvals: u8,
    },

    /// Mark an institution as verified (protocol / registrar authority).
    ///
    /// Accounts:
    /// - Institution account (mut)
    /// - Registrar account (signer)
    VerifyInstitution,

    /// Bind a fee-token definition and custody vault PDA to the institution.
    ///
    /// Accounts:
    /// - Config account
    /// - Institution account (mut)
    /// - Token definition account (owned by Token Program)
    /// - Vault account (must be the institution vault PDA; may be uninitialized)
    /// - Admin account (signer)
    InitializeInstitutionVault,

    /// Add a multi-sig approver for release requests.
    ///
    /// Accounts:
    /// - Institution account (mut)
    /// - Admin account (signer)
    AddApprover { approver: AccountId },

    /// Create a fee category under an institution. Percentages are basis points.
    ///
    /// Accounts:
    /// - Institution account (mut)
    /// - Category account (uninitialized, authorized)
    /// - Admin account (signer)
    CreateFeeCategory {
        name: String,
        description: String,
        allocated_percentage_bps: u16,
    },

    /// Lock a fee payment into the institution vault and allocate to categories.
    ///
    /// Accounts:
    /// - Config account
    /// - Institution account (mut)
    /// - Payment account (uninitialized, authorized)
    /// - Payer account (signer)
    /// - Payer token holding (mut, authorized)
    /// - Institution vault token holding (mut, PDA)
    /// - Up to 8 category accounts (mut); unused slots must be default accounts
    PayFees {
        payer_name: String,
        reference_number: String,
        amount: u128,
    },

    /// Request release of locked funds from a category.
    ///
    /// Accounts:
    /// - Institution account
    /// - Category account
    /// - Release request account (uninitialized, authorized)
    /// - Admin account (signer)
    RequestRelease {
        amount: u128,
        purpose: String,
        evidence_hash: String,
    },

    /// Approve a pending release request (multi-sig).
    ///
    /// Accounts:
    /// - Institution account
    /// - Release request account (mut)
    /// - Approver account (signer)
    ApproveRelease,

    /// Reject a pending release request.
    ///
    /// Accounts:
    /// - Institution account
    /// - Release request account (mut)
    /// - Approver account (signer)
    RejectRelease,

    /// Execute an approved release: debit ledgers and transfer vault → treasury holding.
    ///
    /// Accounts:
    /// - Config account
    /// - Institution account (mut)
    /// - Category account (mut)
    /// - Release request account (mut)
    /// - Institution vault token holding (mut, PDA)
    /// - Treasury token holding (mut)
    ExecuteRelease,
}

/// Singleton config stored at the `"CONFIG"` PDA.
#[account_type]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ProgramConfig {
    pub token_program_id: ProgramId,
}

#[account_type]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Institution {
    pub name: String,
    pub registration_number: String,
    pub institution_type: InstitutionType,
    pub admin: AccountId,
    pub treasury: AccountId,
    pub is_verified: bool,
    pub is_active: bool,
    pub total_fees_collected: u128,
    pub total_fees_released: u128,
    pub required_approvals: u8,
    pub approvers: Vec<AccountId>,
    pub category_ids: Vec<AccountId>,
    /// Fee token definition once the vault is initialized.
    pub token_definition_id: Option<AccountId>,
    /// Custody vault PDA once initialized.
    pub vault_id: Option<AccountId>,
}

#[account_type]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct FeeCategory {
    pub institution_id: AccountId,
    pub name: String,
    pub description: String,
    /// Basis points (100 = 1%).
    pub allocated_percentage_bps: u16,
    pub total_collected: u128,
    pub total_released: u128,
    pub is_active: bool,
}

#[account_type]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Payment {
    pub institution_id: AccountId,
    pub payer: AccountId,
    pub payer_name: String,
    pub reference_number: String,
    pub amount: u128,
    pub status: FeeStatus,
    pub allocations: Vec<FeeAllocation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct FeeAllocation {
    pub category_id: AccountId,
    pub amount: u128,
    pub released: bool,
}

#[account_type]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ReleaseRequest {
    pub institution_id: AccountId,
    pub category_id: AccountId,
    pub amount: u128,
    pub purpose: String,
    pub evidence_hash: String,
    pub requested_by: AccountId,
    pub status: ReleaseStatus,
    pub approvers: Vec<AccountId>,
    pub rejectors: Vec<AccountId>,
}

#[must_use]
pub fn compute_config_pda_seed() -> PdaSeed {
    use risc0_zkvm::sha::{Impl, Sha256};
    PdaSeed::new(
        Impl::hash_bytes(CONFIG_PDA_SEED)
            .as_bytes()
            .try_into()
            .expect("Hash output must be exactly 32 bytes long"),
    )
}

#[must_use]
pub fn compute_config_pda(program_id: ProgramId) -> AccountId {
    AccountId::for_public_pda(&program_id, &compute_config_pda_seed())
}

#[must_use]
pub fn compute_vault_pda_seed(
    institution_id: AccountId,
    token_definition_id: AccountId,
) -> PdaSeed {
    use risc0_zkvm::sha::{Impl, Sha256};
    let mut bytes = [0_u8; 64];
    let (institution_bytes, definition_bytes) = bytes.split_at_mut(32);
    institution_bytes.copy_from_slice(&institution_id.to_bytes());
    definition_bytes.copy_from_slice(&token_definition_id.to_bytes());
    PdaSeed::new(
        Impl::hash_bytes(&bytes)
            .as_bytes()
            .try_into()
            .expect("Hash output must be exactly 32 bytes long"),
    )
}

#[must_use]
pub fn compute_vault_pda(
    program_id: ProgramId,
    institution_id: AccountId,
    token_definition_id: AccountId,
) -> AccountId {
    AccountId::for_public_pda(
        &program_id,
        &compute_vault_pda_seed(institution_id, token_definition_id),
    )
}

macro_rules! impl_data_codec {
    ($ty:ty) => {
        impl TryFrom<&Data> for $ty {
            type Error = std::io::Error;

            fn try_from(data: &Data) -> Result<Self, Self::Error> {
                <$ty>::try_from_slice(data.as_ref())
            }
        }

        impl From<&$ty> for Data {
            fn from(value: &$ty) -> Self {
                let mut data = Vec::with_capacity(std::mem::size_of_val(value));
                BorshSerialize::serialize(value, &mut data)
                    .expect("Serialization to Vec should not fail");
                Data::try_from(data).expect("encoded account data should fit into Data")
            }
        }
    };
}

impl_data_codec!(ProgramConfig);
impl_data_codec!(Institution);
impl_data_codec!(FeeCategory);
impl_data_codec!(Payment);
impl_data_codec!(ReleaseRequest);
