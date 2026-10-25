use sacco_ledger_core::{compute_config_pda, ProgramConfig};
use nssa_core::{
    account::{Account, AccountId, AccountWithMetadata, BalanceDiff},
    program::AccountStateDiff,
};

/// Create the singleton config PDA for the SACCO registry.
pub fn initialize(
    mut config_account: AccountWithMetadata,
    program_account_id: AccountId,
) -> Vec<AccountStateDiff> {
    assert_eq!(
        config_account.account_id,
        compute_config_pda(program_account_id),
        "Config account must be the CONFIG PDA"
    );
    assert!(
        config_account.account == Account::default(),
        "Config account must be uninitialized"
    );

    let pre = config_account.clone();
    let config = ProgramConfig { version: 1 };
    config_account.account.data = (&config).into();
    let post_data = config_account.account.data;

    vec![AccountStateDiff::new(pre, BalanceDiff::Add(0), post_data)]
}
