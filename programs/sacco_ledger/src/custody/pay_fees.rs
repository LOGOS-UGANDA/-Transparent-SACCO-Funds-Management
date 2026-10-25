use sacco_ledger_core::custody::{compute_config_pda, compute_vault_pda, compute_vault_pda_seed, FeeAllocation, FeeCategory, FeeStatus, Institution, Payment, ProgramConfig, BPS_DENOMINATOR, MAX_PAY_FEE_CATEGORIES};
use nssa_core::{
    account::{Account, AccountId, AccountWithMetadata, BalanceDiff},
    program::{AccountStateDiff, ChainedCall},
};
use token_core::TokenHolding;

/// Lock a payment into the institution vault and allocate amounts across active categories.
///
/// Returns state diffs plus a chained `Token::Transfer` (payer holding → vault).
pub fn pay_fees(
    config_account: AccountWithMetadata,
    mut institution_account: AccountWithMetadata,
    mut payment_account: AccountWithMetadata,
    payer: AccountWithMetadata,
    payer_holding: AccountWithMetadata,
    vault: AccountWithMetadata,
    mut categories: Vec<AccountWithMetadata>,
    payer_name: String,
    reference_number: String,
    amount: u128,
    program_account_id: AccountId,
) -> (Vec<AccountStateDiff>, Vec<ChainedCall>) {
    assert!(payer.is_authorized, "Payer must authorize payment");
    assert!(
        payer_holding.is_authorized,
        "Payer token holding must be authorized"
    );
    assert!(amount > 0, "Payment must be greater than 0");
    assert!(
        payment_account.account == Account::default(),
        "Payment account must be uninitialized"
    );
    assert!(
        payment_account.is_authorized,
        "Payment account must be authorized"
    );
    assert!(
        categories.len() <= MAX_PAY_FEE_CATEGORIES,
        "Too many category accounts for v1 PayFees"
    );
    assert_eq!(
        config_account.account_id,
        compute_config_pda(program_account_id),
        "Config account must be the CONFIG PDA"
    );

    let config = ProgramConfig::try_from(&config_account.account.data)
        .expect("Program must be initialized");
    assert_eq!(
        payer_holding.account.program_owner, config.token_program_id,
        "Payer holding must be owned by the configured Token Program"
    );

    let payer_token = TokenHolding::try_from(&payer_holding.account.data)
        .expect("Payer holding must be a valid token holding");
    let token_definition_id = payer_token.definition_id();

    let mut institution = Institution::try_from(&institution_account.account.data)
        .expect("Institution account must hold valid Institution data");
    assert!(institution.is_verified, "Institution not verified");
    assert!(institution.is_active, "Institution not active");
    let vault_id = institution
        .vault_id
        .expect("Institution vault not initialized");
    let expected_definition = institution
        .token_definition_id
        .expect("Institution token definition not set");
    assert_eq!(
        token_definition_id, expected_definition,
        "Payer token does not match institution fee token"
    );
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

    let mut allocations = Vec::new();
    let mut category_diffs = Vec::new();
    for category_account in &mut categories {
        let mut category = FeeCategory::try_from(&category_account.account.data)
            .expect("Category account must hold valid FeeCategory data");
        assert_eq!(
            category.institution_id, institution_account.account_id,
            "Category institution mismatch"
        );
        assert!(
            institution.category_ids.contains(&category_account.account_id),
            "Category not registered on institution"
        );
        if !category.is_active {
            category_diffs.push(AccountStateDiff::unchanged(category_account.clone()));
            continue;
        }
        let share = amount
            .checked_mul(u128::from(category.allocated_percentage_bps))
            .expect("allocation overflow")
            / u128::from(BPS_DENOMINATOR);
        category.total_collected = category
            .total_collected
            .checked_add(share)
            .expect("collected overflow");
        let category_pre = category_account.clone();
        category_account.account.data = (&category).into();
        let category_data = category_account.account.data.clone();
        category_diffs.push(AccountStateDiff::new(
            category_pre,
            BalanceDiff::Add(0),
            category_data,
        ));
        allocations.push(FeeAllocation {
            category_id: category_account.account_id,
            amount: share,
            released: false,
        });
    }

    let payment = Payment {
        institution_id: institution_account.account_id,
        payer: payer.account_id,
        payer_name,
        reference_number,
        amount,
        status: FeeStatus::Locked,
        allocations,
    };

    institution.total_fees_collected = institution
        .total_fees_collected
        .checked_add(amount)
        .expect("institution collected overflow");

    let institution_pre = institution_account.clone();
    institution_account.account.data = (&institution).into();
    let institution_data = institution_account.account.data;

    let payment_pre = payment_account.clone();
    payment_account.account.data = (&payment).into();
    let payment_data = payment_account.account.data;

    let transfer_call = ChainedCall::new(
        config.token_program_id,
        vec![payer_holding.account_id, vault.account_id],
        &token_core::Instruction::Transfer {
            amount_to_transfer: amount,
        },
    )
    .with_pda_seeds(vec![compute_vault_pda_seed(
        institution_account.account_id,
        token_definition_id,
    )]);

    let mut diffs = vec![
        AccountStateDiff::unchanged(config_account),
        AccountStateDiff::new(institution_pre, BalanceDiff::Add(0), institution_data),
        AccountStateDiff::new(payment_pre, BalanceDiff::Add(0), payment_data),
        AccountStateDiff::unchanged(payer),
        AccountStateDiff::unchanged(payer_holding),
        AccountStateDiff::unchanged(vault),
    ];
    diffs.extend(category_diffs);

    (diffs, vec![transfer_call])
}
