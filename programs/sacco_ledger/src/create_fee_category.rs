use sacco_ledger_core::{FeeCategory, Institution, BPS_DENOMINATOR};
use nssa_core::{
    account::{Account, AccountWithMetadata, BalanceDiff},
    program::AccountStateDiff,
};

/// Create a fee category and register its id on the institution.
pub fn create_fee_category(
    mut institution_account: AccountWithMetadata,
    mut category_account: AccountWithMetadata,
    admin: AccountWithMetadata,
    name: String,
    description: String,
    allocated_percentage_bps: u16,
    // Currently active categories (client-supplied) used to enforce the 100% cap.
    existing_categories: &[FeeCategory],
) -> Vec<AccountStateDiff> {
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
                .expect("bps addition overflow");
        }
    }
    assert!(
        total <= BPS_DENOMINATOR,
        "Category allocation would exceed 100%"
    );

    let category = FeeCategory {
        institution_id: institution_account.account_id,
        name,
        description,
        allocated_percentage_bps,
        total_collected: 0,
        total_released: 0,
        is_active: true,
    };

    let category_id = category_account.account_id;
    institution.category_ids.push(category_id);

    let institution_pre = institution_account.clone();
    institution_account.account.data = (&institution).into();
    let institution_data = institution_account.account.data;

    let category_pre = category_account.clone();
    category_account.account.data = (&category).into();
    let category_data = category_account.account.data;

    vec![
        AccountStateDiff::new(institution_pre, BalanceDiff::Add(0), institution_data),
        AccountStateDiff::new(category_pre, BalanceDiff::Add(0), category_data),
        AccountStateDiff::unchanged(admin),
    ]
}
