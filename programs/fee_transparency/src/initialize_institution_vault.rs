use fee_transparency_core::{
    compute_config_pda, compute_vault_pda, compute_vault_pda_seed, Institution, ProgramConfig,
};
use nssa_core::{
    account::{AccountId, AccountWithMetadata, BalanceDiff},
    program::AccountStateDiff,
};
use token_core::TokenDefinition;

/// Bind `token_definition_id` and vault PDA id on the institution.
///
/// The vault token holding may still be uninitialized; the first `PayFees` transfer can
/// materialize it under PDA authority (same pattern as AMM vault funding).
pub fn initialize_institution_vault(
    config_account: AccountWithMetadata,
    mut institution_account: AccountWithMetadata,
    token_definition: AccountWithMetadata,
    vault: AccountWithMetadata,
    admin: AccountWithMetadata,
    program_account_id: AccountId,
) -> Vec<AccountStateDiff> {
    assert!(admin.is_authorized, "Admin must authorize vault init");
    assert_eq!(
        config_account.account_id,
        compute_config_pda(program_account_id),
        "Config account must be the CONFIG PDA"
    );

    let config = ProgramConfig::try_from(&config_account.account.data)
        .expect("Program must be initialized");
    assert_eq!(
        token_definition.account.program_owner, config.token_program_id,
        "Token definition must be owned by the configured Token Program"
    );
    let _definition = TokenDefinition::try_from(&token_definition.account.data)
        .expect("Token definition account must be valid");

    let mut institution = Institution::try_from(&institution_account.account.data)
        .expect("Institution account must hold valid Institution data");
    assert_eq!(
        institution.admin, admin.account_id,
        "Only institution admin can initialize the vault"
    );
    assert!(
        institution.vault_id.is_none(),
        "Institution vault already initialized"
    );

    let expected_vault = compute_vault_pda(
        program_account_id,
        institution_account.account_id,
        token_definition.account_id,
    );
    assert_eq!(
        vault.account_id, expected_vault,
        "Vault account must be the institution vault PDA"
    );
    // Touch seed helper so seed derivation stays covered by the same inputs.
    let _seed = compute_vault_pda_seed(institution_account.account_id, token_definition.account_id);

    institution.token_definition_id = Some(token_definition.account_id);
    institution.vault_id = Some(vault.account_id);

    let pre = institution_account.clone();
    institution_account.account.data = (&institution).into();
    let post_data = institution_account.account.data;

    vec![
        AccountStateDiff::unchanged(config_account),
        AccountStateDiff::new(pre, BalanceDiff::Add(0), post_data),
        AccountStateDiff::unchanged(token_definition),
        AccountStateDiff::unchanged(vault),
        AccountStateDiff::unchanged(admin),
    ]
}
