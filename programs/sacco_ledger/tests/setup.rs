//! Setup-handler integration tests: initialize, register_sacco, verify_sacco,
//! add_approver, add_supervisor, add_member.

use nssa_core::account::{Account, AccountId, AccountWithMetadata};
use sacco_ledger_program::{
    add_approver::add_approver, add_member::add_member, add_supervisor::add_supervisor,
    initialize::initialize, register_sacco::register_sacco, verify_sacco::verify_sacco,
};
use sacco_ledger_core::{Member, Sacco};

fn blank_account(id: u8) -> AccountWithMetadata {
    AccountWithMetadata {
        account_id: AccountId::new([id; 32]),
        account: Account::default(),
        is_authorized: true,
    }
}

fn aid(n: u8) -> AccountId {
    AccountId::new([n; 32])
}

fn sacco_account_from(diffs: Vec<nssa_core::program::AccountStateDiff>) -> AccountWithMetadata {
    let mut sacco_account = blank_account(1);
    sacco_account.account.data = diffs[0].post_data.as_ref().expect("sacco data").clone();
    sacco_account
}

fn registered_sacco() -> (AccountWithMetadata, AccountWithMetadata) {
    let mut sacco_account = blank_account(1);
    let admin = blank_account(2);
    let diffs = register_sacco(
        sacco_account.clone(),
        admin.clone(),
        "Kampala Traders SACCO".into(),
        "UG-SACCO-001".into(),
        aid(9),
    );
    let post = diffs[0]
        .post_data
        .as_ref()
        .expect("sacco data")
        .clone();
    sacco_account.account.data = post;
    (sacco_account, admin)
}

#[test]
fn initialize_creates_config() {
    let program = aid(7);
    let config_id = sacco_ledger_core::compute_config_pda(program);
    let config = AccountWithMetadata {
        account_id: config_id,
        account: Account::default(),
        is_authorized: true,
    };
    let diffs = initialize(config, program);
    assert_eq!(diffs.len(), 1);
    assert!(diffs[0].post_data.is_some(), "config data written");
}

#[test]
fn register_sacco_records_admin_and_defaults() {
    let (sacco_account, _admin) = registered_sacco();
    let sacco = Sacco::try_from(&sacco_account.account.data).expect("sacco data");
    assert_eq!(sacco.name, "Kampala Traders SACCO");
    assert_eq!(sacco.required_approvals, 1);
    assert_eq!(sacco.approvers.len(), 1);
    assert!(sacco.supervisory_committee.is_empty());
    assert!(sacco.member_ids.is_empty());
    assert_eq!(sacco.total_deposits_savings, 0);
    assert_eq!(sacco.total_deposits_share_capital, 0);
    assert_eq!(sacco.total_loans_disbursed, 0);
    assert_eq!(sacco.total_loans_repaid, 0);
    assert_eq!(sacco.total_expenses, 0);
    assert!(!sacco.is_verified);
}

#[test]
#[should_panic(expected = "Sacco name must not be empty")]
fn register_sacco_rejects_empty_name() {
    let sacco_account = blank_account(1);
    let admin = blank_account(2);
    let _ = register_sacco(sacco_account, admin, String::new(), "UG-SACCO-001".into(), aid(9));
}

#[test]
fn verify_sacco_by_admin() {
    let (mut sacco_account, admin) = registered_sacco();
    let diffs = verify_sacco(sacco_account.clone(), admin);
    let post = diffs[0].post_data.as_ref().expect("sacco data");
    let sacco = Sacco::try_from(post).expect("sacco");
    assert!(sacco.is_verified);
    let _ = sacco_account;
}

#[test]
#[should_panic(expected = "Only the SACCO admin can verify in v1")]
fn verify_sacco_requires_registrar() {
    let (sacco_account, _admin) = registered_sacco();
    let outsider = blank_account(3);
    let _ = verify_sacco(sacco_account, outsider);
}

#[test]
fn add_approver_admin_only_and_rejects_duplicate() {
    let (mut sacco_account, admin) = registered_sacco();
    let approver = aid(4);
    let diffs = add_approver(sacco_account.clone(), admin, approver);
    let post = diffs[0].post_data.as_ref().expect("sacco data");
    let sacco = Sacco::try_from(post).expect("sacco");
    assert!(sacco.approvers.contains(&approver));
    assert_eq!(sacco.approvers.len(), 2);
}

#[test]
#[should_panic(expected = "Only the SACCO admin can add approvers")]
fn add_approver_admin_only() {
    let (sacco_account, _admin) = registered_sacco();
    let outsider = blank_account(3);
    let _ = add_approver(sacco_account, outsider, aid(4));
}

#[test]
#[should_panic(expected = "Approver already registered")]
fn add_approver_rejects_duplicate() {
    let (sacco_account, admin) = registered_sacco();
    let _ = add_approver(sacco_account, admin, aid(2)); // admin is already an approver
}

#[test]
fn add_supervisor_admin_only() {
    let (sacco_account, admin) = registered_sacco();
    let supervisor = aid(5);
    let diffs = add_supervisor(sacco_account, admin, supervisor);
    let post = diffs[0].post_data.as_ref().expect("sacco data");
    let sacco = Sacco::try_from(post).expect("sacco");
    assert!(sacco.supervisory_committee.contains(&supervisor));
}

#[test]
#[should_panic(expected = "Approvers cannot also sit on the Supervisory Committee")]
fn add_supervisor_rejects_current_approver() {
    let (sacco_account, admin) = registered_sacco();
    let _ = add_supervisor(sacco_account, admin, aid(2)); // admin is an approver
}

#[test]
fn add_member_creates_member_account() {
    let (mut sacco_account, admin) = registered_sacco();
    let member_account = blank_account(6);
    let diffs = add_member(sacco_account.clone(), member_account, admin, aid(6));
    assert_eq!(diffs.len(), 3);
    let sacco_post = diffs[0].post_data.as_ref().expect("sacco data");
    let sacco = Sacco::try_from(sacco_post).expect("sacco");
    assert!(sacco.member_ids.contains(&aid(6)));
    let member_post = diffs[1].post_data.as_ref().expect("member data");
    let member = Member::try_from(member_post).expect("member");
    assert_eq!(member.identity, aid(6));
    assert_eq!(member.savings_balance, 0);
}

#[test]
#[should_panic(expected = "Member identity already registered")]
fn add_member_rejects_duplicate_identity() {
    let (sacco_account, admin) = registered_sacco();
    let member_account = blank_account(6);
    let diffs = add_member(sacco_account, member_account, admin.clone(), aid(6));
    // Apply the first registration's post-state, then re-register the same identity.
    let sacco_account = sacco_account_from(diffs);
    let member_account2 = blank_account(7);
    let _ = add_member(sacco_account, member_account2, admin, aid(6));
}
