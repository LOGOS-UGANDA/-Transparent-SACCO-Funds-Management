use sacco_ledger_core::Sacco;
use nssa_core::{
    account::{Account, AccountId, AccountWithMetadata, BalanceDiff},
    program::AccountStateDiff,
};

/// Register a SACCO: public name, registration number and treasury contact.
/// Starts with one approver (the admin) and the per-SACCO multi-sig threshold at 1.
pub fn register_sacco(
    mut sacco_account: AccountWithMetadata,
    admin: AccountWithMetadata,
    name: String,
    registration_number: String,
    treasury: AccountId,
) -> Vec<AccountStateDiff> {
    assert!(
        sacco_account.is_authorized,
        "Sacco account must be authorized"
    );
    assert!(admin.is_authorized, "Admin must authorize registration");
    assert!(
        sacco_account.account == Account::default(),
        "Sacco account must be uninitialized"
    );
    assert!(!name.is_empty(), "Sacco name must not be empty");
    assert!(
        !registration_number.is_empty(),
        "Registration number must not be empty"
    );

    let sacco = Sacco {
        name,
        registration_number,
        admin: admin.account_id,
        treasury,
        is_verified: false,
        is_active: true,
        total_deposits_savings: 0,
        total_deposits_share_capital: 0,
        total_loans_disbursed: 0,
        total_loans_repaid: 0,
        total_expenses: 0,
        required_approvals: 1,
        approvers: vec![admin.account_id],
        supervisory_committee: vec![],
        member_ids: vec![],
    };

    let pre = sacco_account.clone();
    sacco_account.account.data = (&sacco).into();
    let post_data = sacco_account.account.data;

    vec![
        AccountStateDiff::new(pre, BalanceDiff::Add(0), post_data),
        AccountStateDiff::unchanged(admin),
    ]
}
