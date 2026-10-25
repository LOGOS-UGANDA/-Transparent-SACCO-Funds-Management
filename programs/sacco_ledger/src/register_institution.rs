use sacco_ledger_core::{Institution, InstitutionType};
use nssa_core::{
    account::{Account, AccountId, AccountWithMetadata, BalanceDiff},
    program::AccountStateDiff,
};

/// Initialize an institution account. Scaffold: validates shape and writes initial state.
pub fn register_institution(
    mut institution_account: AccountWithMetadata,
    admin: AccountWithMetadata,
    name: String,
    registration_number: String,
    institution_type: InstitutionType,
    treasury: AccountId,
    required_approvals: u8,
) -> Vec<AccountStateDiff> {
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
        token_definition_id: None,
        vault_id: None,
    };

    let institution_pre = institution_account.clone();
    institution_account.account.data = (&institution).into();
    let institution_data = institution_account.account.data;

    vec![
        AccountStateDiff::new(institution_pre, BalanceDiff::Add(0), institution_data),
        AccountStateDiff::unchanged(admin),
    ]
}
