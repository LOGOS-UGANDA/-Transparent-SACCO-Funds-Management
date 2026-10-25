use crate::register_institution::register_institution;
use sacco_ledger_core::{Institution, InstitutionType};
use nssa_core::account::{Account, AccountId, AccountWithMetadata};

fn blank_account(id: u8) -> AccountWithMetadata {
    AccountWithMetadata {
        account_id: AccountId::new([id; 32]),
        account: Account::default(),
        is_authorized: true,
    }
}

#[test]
fn register_institution_writes_initial_state() {
    let institution_account = blank_account(1);
    let admin = blank_account(2);
    let treasury = AccountId::new([3; 32]);

    let diffs = register_institution(
        institution_account,
        admin,
        "Makerere University".into(),
        "MAK-001".into(),
        InstitutionType::University,
        treasury,
        2,
    );

    assert_eq!(diffs.len(), 2);
    let post_data = diffs[0]
        .post_data
        .as_ref()
        .expect("institution diff must carry new data");
    let institution = Institution::try_from(post_data).expect("institution data");
    assert_eq!(institution.name, "Makerere University");
    assert!(!institution.is_verified);
    assert_eq!(institution.required_approvals, 2);
    assert_eq!(institution.approvers.len(), 1);
    assert!(institution.vault_id.is_none());
    assert!(institution.token_definition_id.is_none());
}
