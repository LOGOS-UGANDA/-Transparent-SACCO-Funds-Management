use fee_transparency_core::{FeeCategory, Institution, BPS_DENOMINATOR};
use nssa_core::{
    account::{Account, AccountId, AccountWithMetadata},
    program::{AccountPostState, Claim},
};

/// Create a fee category and register its id on the institution.
pub fn create_fee_category(
    mut institution_account: AccountWithMetadata,
    mut category_account: AccountWithMetadata,
    admin: AccountWithMetadata,
    name: String,
    description: String,
    allocated_percentage_bps: u16,
    /// Currently active categories (client-supplied) used to enforce the 100% cap.
    existing_categories: &[FeeCategory],
) -> Vec<AccountPostState> {
    assert!(admin.is_authorized, "Admin must authorize category creation");
    assert!(
        category_account.account == Account::default(),
        "Category account must be uninitialized"
    );
    assert!(
        category_account.is_authorized,
        "Category account must be authorized"
    );

    let mut institution = Institution::try_from(&institution_account.account.data)
        .expect("Institution account must hold valid Institution data");
    assert_eq!(
        institution.admin, admin.account_id,
        "Only institution admin can create categories"
    );
    assert!(institution.is_active, "Institution must be active");

    let mut total = allocated_percentage_bps;
    for category in existing_categories {
        if category.is_active {
            total = total
                .checked_add(category.allocated_percentage_bps)
                .expect("percentage overflow");
        }
    }
    assert!(
        total <= BPS_DENOMINATOR,
        "Total allocation exceeds 100%"
    );

    let category_id: AccountId = category_account.account_id;
    let category = FeeCategory {
        institution_id: institution_account.account_id,
        name,
        description,
        allocated_percentage_bps,
        total_collected: 0,
        total_released: 0,
        is_active: true,
    };

    institution.category_ids.push(category_id);
    institution_account.account.data = (&institution).into();
    category_account.account.data = (&category).into();

    vec![
        AccountPostState::new(institution_account.account),
        AccountPostState::new_claimed_if_default(category_account.account, Claim::Authorized),
        AccountPostState::new(admin.account),
    ]
}
