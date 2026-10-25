use sacco_ledger_core::{PeriodSummary, Sacco};
use crate::is_approver;
use nssa_core::{
    account::{Account, AccountWithMetadata, BalanceDiff},
    program::AccountStateDiff,
};

/// Post a public period snapshot of SACCO totals. Labels are unique per SACCO.
pub fn post_period_summary(
    sacco_account: AccountWithMetadata,
    mut summary_account: AccountWithMetadata,
    poster: AccountWithMetadata,
    label: String,
) -> Vec<AccountStateDiff> {
    assert!(poster.is_authorized, "Poster must authorize");
    assert!(
        summary_account.account == Account::default(),
        "Summary account must be uninitialized"
    );
    assert!(
        summary_account.is_authorized,
        "Summary account must be authorized"
    );
    assert!(!label.is_empty(), "Period label must not be empty");

    let sacco = Sacco::try_from(&sacco_account.account.data)
        .expect("Sacco account must hold valid Sacco data");
    assert!(
        is_approver(&sacco, &poster.account_id),
        "Not a SACCO approver"
    );

    let summary = PeriodSummary {
        sacco_id: sacco_account.account_id,
        label,
        deposits_savings: sacco.total_deposits_savings,
        deposits_share_capital: sacco.total_deposits_share_capital,
        loans_disbursed: sacco.total_loans_disbursed,
        loans_repaid: sacco.total_loans_repaid,
        expenses: sacco.total_expenses,
    };

    let pre = summary_account.clone();
    summary_account.account.data = (&summary).into();
    let post_data = summary_account.account.data;

    vec![
        AccountStateDiff::unchanged(sacco_account),
        AccountStateDiff::new(pre, BalanceDiff::Add(0), post_data),
        AccountStateDiff::unchanged(poster),
    ]
}
