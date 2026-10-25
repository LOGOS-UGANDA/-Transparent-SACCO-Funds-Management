//! DORMANT custody model — the earlier university fee-transparency program
//! (institution vault PDA + category ledger + token transfers).
//!
//! Kept as reference per the pivot review decision: not dispatched by the active
//! instruction set, not wired into the `sacco_ledger` guest. Value never moves
//! through these paths in the SACCO attestation model.

#![allow(dead_code)]

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
#[borsh(use_discriminant = true)]
pub enum InstitutionType {
    University = 0,
    Sacco = 1,
    Cooperative = 2,
    School = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[borsh(use_discriminant = true)]
pub enum FeeStatus {
    Locked = 0,
    PartiallyReleased = 1,
    FullyReleased = 2,
    Refunded = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[borsh(use_discriminant = true)]
pub enum ReleaseStatus {
    Pending = 0,
    Approved = 1,
    Rejected = 2,
    Executed = 3,
}

/// Dormant custody instructions (the earlier fee-transparency program).
#[derive(Serialize, Deserialize)]
pub enum LegacyInstruction {
    Initialize { token_program_id: ProgramId },
    RegisterInstitution {
        name: String,
        registration_number: String,
        institution_type: InstitutionType,
        treasury: AccountId,
        required_approvals: u8,
    },
    VerifyInstitution,
    InitializeInstitutionVault,
    AddApprover { approver: AccountId },
    CreateFeeCategory {
        name: String,
        description: String,
        allocated_percentage_bps: u16,
    },
    PayFees {
        payer_name: String,
        reference_number: String,
        amount: u128,
    },
    RequestRelease {
        amount: u128,
        purpose: String,
        evidence_hash: String,
    },
    ApproveRelease,
    RejectRelease,
    ExecuteRelease,
}

#[account_type]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ProgramConfig {
    pub token_program_id: AccountId,
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
    pub token_definition_id: Option<AccountId>,
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
pub fn compute_config_pda(program_account_id: AccountId) -> AccountId {
    AccountId::for_public_pda(&program_account_id, &compute_config_pda_seed())
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
    program_account_id: AccountId,
    institution_id: AccountId,
    token_definition_id: AccountId,
) -> AccountId {
    AccountId::for_public_pda(
        &program_account_id,
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
