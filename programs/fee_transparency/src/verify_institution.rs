use fee_transparency_core::Institution;
use nssa_core::{
    account::AccountWithMetadata,
    program::AccountPostState,
};

/// Mark institution verified. Scaffold: any authorized registrar; tighten auth later.
pub fn verify_institution(
    mut institution_account: AccountWithMetadata,
    registrar: AccountWithMetadata,
) -> Vec<AccountPostState> {
    assert!(registrar.is_authorized, "Registrar must authorize verification");

    let mut institution = Institution::try_from(&institution_account.account.data)
        .expect("Institution account must hold valid Institution data");
    assert!(institution.is_active, "Institution must be active");
    institution.is_verified = true;
    institution_account.account.data = (&institution).into();

    vec![
        AccountPostState::new(institution_account.account),
        AccountPostState::new(registrar.account),
    ]
}
