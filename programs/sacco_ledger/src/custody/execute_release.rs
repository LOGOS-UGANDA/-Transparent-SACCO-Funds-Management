use sacco_ledger_core::custody::{compute_config_pda, compute_vault_pda, compute_vault_pda_seed, FeeCategory, Institution, ProgramConfig, ReleaseRequest, ReleaseStatus};
use nssa_core::{
    account::{AccountId, AccountWithMetadata, BalanceDiff},
    program::{AccountStateDiff, ChainedCall},
};
use token_core::TokenHolding;

/// Mark an approved release as executed, update ledgers, and transfer vault → treasury holding.
pub fn execute_release(
    config_account: AccountWithMetadata,
    mut institution_account: AccountWithMetadata,
    mut category_account: AccountWithMetadata,
    mut release_account: AccountWithMetadata,
    vault: AccountWithMetadata,
    treasury_holding: AccountWithMetadata,
    program_account_id: AccountId,
) -> (Vec<AccountStateDiff>, Vec<ChainedCall>) {
    assert_eq!(
        config_account.account_id,
        compute_config_pda(program_account_id),
        "Config account must be the CONFIG PDA"
    );
    let config = ProgramConfig::try_from(&config_account.account.data)
        .expect("Program must be initialized");

    let mut institution = Institution::try_from(&institution_account.account.data)
        .expect("Institution account must hold valid Institution data");
    let vault_id = institution
        .vault_id
        .expect("Institution vault not initialized");
    let token_definition_id = institution
        .token_definition_id
        .expect("Institution token definition not set");
    assert_eq!(vault.account_id, vault_id, "Vault account mismatch");
    assert_eq!(
        vault.account_id,
        compute_vault_pda(
            program_account_id,
            institution_account.account_id,
            token_definition_id
        ),
        "Vault account must be the institution vault PDA"
    );
    assert_eq!(
        vault.account.program_owner, config.token_program_id,
        "Vault must be owned by the configured Token Program"
    );
    assert_eq!(
        treasury_holding.account.program_owner, config.token_program_id,
        "Treasury holding must be owned by the configured Token Program"
    );

    let treasury_token = TokenHolding::try_from(&treasury_holding.account.data)
        .expect("Treasury holding must be a valid token holding");
    assert_eq!(
        treasury_token.definition_id(),
        token_definition_id,
        "Treasury holding token definition mismatch"
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

    let institution_pre = institution_account.clone();
    institution_account.account.data = (&institution).into();
    let institution_data = institution_account.account.data;

    let category_pre = category_account.clone();
    category_account.account.data = (&category).into();
    let category_data = category_account.account.data;

    let release_pre = release_account.clone();
    release_account.account.data = (&release).into();
    let release_data = release_account.account.data;

    let transfer_call = ChainedCall::new(
        config.token_program_id,
        vec![vault.account_id, treasury_holding.account_id],
        &token_core::Instruction::Transfer {
            amount_to_transfer: release.amount,
        },
    )
    .with_pda_seeds(vec![compute_vault_pda_seed(
        institution_account.account_id,
        token_definition_id,
    )]);

    let diffs = vec![
        AccountStateDiff::unchanged(config_account),
        AccountStateDiff::new(institution_pre, BalanceDiff::Add(0), institution_data),
        AccountStateDiff::new(category_pre, BalanceDiff::Add(0), category_data),
        AccountStateDiff::new(release_pre, BalanceDiff::Add(0), release_data),
        AccountStateDiff::unchanged(vault),
        AccountStateDiff::unchanged(treasury_holding),
    ];

    (diffs, vec![transfer_call])
}
