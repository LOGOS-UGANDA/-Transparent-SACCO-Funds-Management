//! Core types and instructions for the Transparent SACCO fee-transparency LEZ program.
//!
//! Account data uses Borsh. Instructions use serde (SPEL / wallet path).
//! Prefer public LEZ accounts so payers can audit allocations.

use borsh::{BorshDeserialize, BorshSerialize};
use nssa_core::account::{AccountId, Data};
use serde::{Deserialize, Serialize};
use spel_framework_macros::account_type;

/// Basis points denominator: 10_000 = 100%.
pub const BPS_DENOMINATOR: u16 = 10_000;

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

    /// Add a multi-sig approver for release requests.
    ///
    /// Accounts:
    /// - Institution account (mut)
    /// - Admin account (signer)
    AddApprover { approver: AccountId },

    /// Create a fee category under an institution. Percentages are basis points.
    ///
    /// Accounts:
    /// - Institution account
    /// - Category account (uninitialized, authorized)
    /// - Admin account (signer)
    CreateFeeCategory {
        name: String,
        description: String,
        allocated_percentage_bps: u16,
    },

    /// Record a locked fee payment and allocate to active categories.
    /// Token movement / custody will be wired via chained token calls in a later iteration.
    ///
    /// Accounts:
    /// - Institution account (mut)
    /// - Payment account (uninitialized, authorized)
    /// - Payer account (signer)
    /// - Category accounts referenced by the institution (mut) — count is dynamic; v1 handlers
    ///   accept a fixed category list passed by the client.
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

    /// Execute an approved release once `required_approvals` is met.
    ///
    /// Accounts:
    /// - Institution account (mut)
    /// - Category account (mut)
    /// - Release request account (mut)
    /// - Treasury / destination account (mut)
    ExecuteRelease,
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

impl_data_codec!(Institution);
impl_data_codec!(FeeCategory);
impl_data_codec!(Payment);
impl_data_codec!(ReleaseRequest);
