use fee_transparency_core::Institution;
use nssa_core::{
    account::{AccountId, AccountWithMetadata},
    program::AccountPostState,
};

pub fn add_approver(
    mut institution_account: AccountWithMetadata,
    admin: AccountWithMetadata,
    approver: AccountId,
) -> Vec<AccountPostState> {
    assert!(admin.is_authorized, "Admin must authorize");

    let mut institution = Institution::try_from(&institution_account.account.data)
        .expect("Institution account must hold valid Institution data");
    assert_eq!(
        institution.admin, admin.account_id,
        "Only institution admin can add approvers"
    );
    assert!(
        !institution.approvers.contains(&approver),
        "Approver already registered"
    );

    institution.approvers.push(approver);
    institution_account.account.data = (&institution).into();

    vec![
        AccountPostState::new(institution_account.account),
        AccountPostState::new(admin.account),
    ]
}
