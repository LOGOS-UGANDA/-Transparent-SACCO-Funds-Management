use fee_transparency_core::{compute_config_pda, compute_config_pda_seed, ProgramConfig};
use nssa_core::{
    account::{Account, AccountWithMetadata},
    program::{AccountPostState, Claim, ProgramId},
};

/// Create the singleton config PDA and record the Token Program id for chained custody calls.
pub fn initialize(
    mut config_account: AccountWithMetadata,
    token_program_id: ProgramId,
    self_program_id: ProgramId,
) -> Vec<AccountPostState> {
    assert_eq!(
        config_account.account_id,
        compute_config_pda(self_program_id),
        "Config account must be the CONFIG PDA"
    );
    assert!(
        config_account.account == Account::default(),
        "Config account must be uninitialized"
    );

    let config = ProgramConfig { token_program_id };
    config_account.account.data = (&config).into();

    vec![AccountPostState::new_claimed(
        config_account.account,
        Claim::Pda(compute_config_pda_seed()),
    )]
}
