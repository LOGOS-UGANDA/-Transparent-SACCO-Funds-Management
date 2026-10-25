use fee_transparency_core::{FeeCategory, Institution, ReleaseRequest, ReleaseStatus};
use nssa_core::{
    account::{Account, AccountWithMetadata},
    program::{AccountPostState, Claim},
};

pub fn request_release(
    institution_account: AccountWithMetadata,
    category_account: AccountWithMetadata,
    mut release_account: AccountWithMetadata,
    admin: AccountWithMetadata,
    amount: u128,
    purpose: String,
    evidence_hash: String,
) -> Vec<AccountPostState> {
    assert!(admin.is_authorized, "Admin must authorize release request");
    assert!(
        release_account.account == Account::default(),
        "Release account must be uninitialized"
    );
    assert!(
        release_account.is_authorized,
        "Release account must be authorized"
    );

    let institution = Institution::try_from(&institution_account.account.data)
        .expect("Institution account must hold valid Institution data");
    assert_eq!(
        institution.admin, admin.account_id,
        "Only institution admin can request release"
    );

    let category = FeeCategory::try_from(&category_account.account.data)
        .expect("Category account must hold valid FeeCategory data");
    assert_eq!(
        category.institution_id, institution_account.account_id,
        "Category institution mismatch"
    );
    let available = category
        .total_collected
        .checked_sub(category.total_released)
        .expect("released exceeds collected");
    assert!(amount <= available, "Insufficient locked funds");

    let release = ReleaseRequest {
        institution_id: institution_account.account_id,
        category_id: category_account.account_id,
        amount,
        purpose,
        evidence_hash,
        requested_by: admin.account_id,
        status: ReleaseStatus::Pending,
        approvers: vec![],
        rejectors: vec![],
    };
    release_account.account.data = (&release).into();

    vec![
        AccountPostState::new(institution_account.account),
        AccountPostState::new(category_account.account),
        AccountPostState::new_claimed_if_default(release_account.account, Claim::Authorized),
        AccountPostState::new(admin.account),
    ]
}
