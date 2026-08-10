use fee_transparency_core::{Institution, InstitutionType};
use nssa_core::{
    account::{Account, AccountWithMetadata},
    program::{AccountPostState, Claim},
};

/// Initialize an institution account. Scaffold: validates shape and writes initial state.
pub fn register_institution(
    mut institution_account: AccountWithMetadata,
    admin: AccountWithMetadata,
    name: String,
    registration_number: String,
    institution_type: InstitutionType,
    treasury: nssa_core::account::AccountId,
    required_approvals: u8,
) -> Vec<AccountPostState> {
    assert!(
        institution_account.is_authorized,
        "Institution account must be authorized"
    );
    assert!(admin.is_authorized, "Admin must authorize registration");
    assert!(
        institution_account.account == Account::default(),
        "Institution account must be uninitialized"
    );
    assert!(required_approvals >= 1, "At least one approval required");

    let institution = Institution {
        name,
        registration_number,
        institution_type,
        admin: admin.account_id,
        treasury,
        is_verified: false,
        is_active: true,
        total_fees_collected: 0,
        total_fees_released: 0,
        required_approvals,
        approvers: vec![admin.account_id],
        category_ids: vec![],
    };

    institution_account.account.data = (&institution).into();

    vec![
        AccountPostState::new_claimed_if_default(
            institution_account.account,
            Claim::Authorized,
        ),
        AccountPostState::new_claimed_if_default(admin.account, Claim::Authorized),
    ]
}
