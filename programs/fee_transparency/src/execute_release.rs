use fee_transparency_core::{FeeCategory, Institution, ReleaseRequest, ReleaseStatus};
use nssa_core::{
    account::AccountWithMetadata,
    program::AccountPostState,
};

/// Mark an approved release as executed and update category/institution totals.
/// Scaffold: does not yet transfer tokens to treasury.
pub fn execute_release(
    mut institution_account: AccountWithMetadata,
    mut category_account: AccountWithMetadata,
    mut release_account: AccountWithMetadata,
    treasury: AccountWithMetadata,
) -> Vec<AccountPostState> {
    let mut institution = Institution::try_from(&institution_account.account.data)
        .expect("Institution account must hold valid Institution data");
    assert_eq!(
        institution.treasury, treasury.account_id,
        "Treasury account mismatch"
    );

    let mut release = ReleaseRequest::try_from(&release_account.account.data)
        .expect("Release account must hold valid ReleaseRequest data");
    assert_eq!(
        release.status,
        ReleaseStatus::Approved,
        "Release must be approved before execution"
    );
    assert_eq!(
        release.institution_id, institution_account.account_id,
        "Release institution mismatch"
    );
    assert_eq!(
        release.category_id, category_account.account_id,
        "Release category mismatch"
    );

    let mut category = FeeCategory::try_from(&category_account.account.data)
        .expect("Category account must hold valid FeeCategory data");
    let available = category
        .total_collected
        .checked_sub(category.total_released)
        .expect("released exceeds collected");
    assert!(release.amount <= available, "Insufficient locked funds");

    category.total_released = category
        .total_released
        .checked_add(release.amount)
        .expect("category released overflow");
    institution.total_fees_released = institution
        .total_fees_released
        .checked_add(release.amount)
        .expect("institution released overflow");
    release.status = ReleaseStatus::Executed;

    institution_account.account.data = (&institution).into();
    category_account.account.data = (&category).into();
    release_account.account.data = (&release).into();

    vec![
        AccountPostState::new(institution_account.account),
        AccountPostState::new(category_account.account),
        AccountPostState::new(release_account.account),
        AccountPostState::new(treasury.account),
    ]
}
