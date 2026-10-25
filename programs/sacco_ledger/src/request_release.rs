use sacco_ledger_core::{FeeCategory, Institution, ReleaseRequest, ReleaseStatus};
use nssa_core::{
    account::{Account, AccountWithMetadata, BalanceDiff},
    program::AccountStateDiff,
};

pub fn request_release(
    institution_account: AccountWithMetadata,
    category_account: AccountWithMetadata,
    mut release_account: AccountWithMetadata,
    admin: AccountWithMetadata,
    amount: u128,
    purpose: String,
    evidence_hash: String,
) -> Vec<AccountStateDiff> {
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

    let pre = release_account.clone();
    release_account.account.data = (&release).into();
    let post_data = release_account.account.data;

    vec![
        AccountStateDiff::unchanged(institution_account),
        AccountStateDiff::unchanged(category_account),
        AccountStateDiff::new(pre, BalanceDiff::Add(0), post_data),
        AccountStateDiff::unchanged(admin),
    ]
}
