//! SACCO registration, verification and role composition.

mod support;

use nssa_core::account::AccountId;
use sacco_ledger_core::Instruction;
use support::{aid, meta, unsigned, verified_sacco, Ledger};

fn registered(ledger: &mut Ledger) -> (AccountId, AccountId) {
    let (sacco, admin) = (aid(1), aid(2));
    ledger.execute(
        &[meta(sacco), meta(admin)],
        Instruction::RegisterSacco {
            name: "Kampala Traders SACCO".into(),
            registration_number: "UG-SACCO-001".into(),
            treasury: aid(9),
        },
    );
    (sacco, admin)
}

#[test]
fn register_sacco_records_admin_and_defaults() {
    let (sacco, admin) = (aid(1), aid(2));
    let mut ledger = Ledger::new();
    ledger.execute(
        &[meta(sacco), meta(admin)],
        Instruction::RegisterSacco {
            name: "Kampala Traders SACCO".into(),
            registration_number: "UG-SACCO-001".into(),
            treasury: aid(9),
        },
    );
    let registered = ledger.sacco(sacco);
    assert_eq!(registered.admin, admin);
    assert_eq!(registered.treasury, aid(9));
    assert_eq!(registered.required_approvals, 1);
    assert_eq!(registered.approvers, vec![admin]);
    assert!(registered.is_active);
    assert!(!registered.is_verified);
    assert_eq!(registered.total_deposits_savings, 0);
}

#[test]
#[should_panic(expected = "Sacco name must not be empty")]
fn register_sacco_rejects_empty_name() {
    let mut ledger = Ledger::new();
    ledger.execute(
        &[meta(aid(1)), meta(aid(2))],
        Instruction::RegisterSacco {
            name: String::new(),
            registration_number: "UG-SACCO-001".into(),
            treasury: aid(9),
        },
    );
}

#[test]
#[should_panic(expected = "Admin must authorize")]
fn an_unsigned_signer_is_refused() {
    let mut ledger = Ledger::new();
    ledger.execute(
        &[meta(aid(1)), unsigned(aid(2))],
        Instruction::RegisterSacco {
            name: "Kampala Traders SACCO".into(),
            registration_number: "UG-SACCO-001".into(),
            treasury: aid(9),
        },
    );
}

#[test]
#[should_panic(expected = "names the shard of")]
fn a_foreign_shard_is_refused() {
    let mut ledger = Ledger::new();
    ledger.execute(
        &[support::foreign(aid(1)), meta(aid(2))],
        Instruction::RegisterSacco {
            name: "Kampala Traders SACCO".into(),
            registration_number: "UG-SACCO-001".into(),
            treasury: aid(9),
        },
    );
}

#[test]
fn verify_sacco_by_admin() {
    let mut ledger = Ledger::new();
    let (sacco, admin) = registered(&mut ledger);
    for approver in [aid(3), aid(4)] {
        ledger.execute(
            &[meta(sacco), meta(admin)],
            Instruction::AddApprover { approver },
        );
    }
    ledger.execute(
        &[meta(sacco), meta(admin)],
        Instruction::SetRequiredApprovals {
            required_approvals: 2,
        },
    );
    ledger.execute(&[meta(sacco), meta(admin)], Instruction::VerifySacco);
    assert!(ledger.sacco(sacco).is_verified);
}

#[test]
#[should_panic(expected = "Approval threshold must be at least 2 before verification")]
fn verify_sacco_requires_a_multi_approver_threshold() {
    let mut ledger = Ledger::new();
    let (sacco, admin) = registered(&mut ledger);
    ledger.execute(&[meta(sacco), meta(admin)], Instruction::VerifySacco);
}

#[test]
#[should_panic(expected = "Only the SACCO admin can verify in v1")]
fn verify_sacco_requires_admin() {
    let mut ledger = Ledger::new();
    let (sacco, _admin) = registered(&mut ledger);
    ledger.execute(&[meta(sacco), meta(aid(3))], Instruction::VerifySacco);
}

#[test]
fn add_approver_admin_only() {
    let mut ledger = Ledger::new();
    let (sacco, admin) = registered(&mut ledger);
    ledger.execute(
        &[meta(sacco), meta(admin)],
        Instruction::AddApprover { approver: aid(3) },
    );
    assert!(ledger.sacco(sacco).approvers.contains(&aid(3)));
}

#[test]
fn admin_can_set_threshold_within_approver_count() {
    let mut ledger = Ledger::new();
    let (sacco, admin) = registered(&mut ledger);
    ledger.execute(
        &[meta(sacco), meta(admin)],
        Instruction::AddApprover { approver: aid(3) },
    );
    ledger.execute(
        &[meta(sacco), meta(admin)],
        Instruction::AddApprover { approver: aid(4) },
    );
    ledger.execute(
        &[meta(sacco), meta(admin)],
        Instruction::SetRequiredApprovals {
            required_approvals: 2,
        },
    );
    assert_eq!(ledger.sacco(sacco).required_approvals, 2);
}

#[test]
#[should_panic(expected = "Only the SACCO admin can set approvals")]
fn only_admin_can_change_approval_threshold() {
    let mut ledger = Ledger::new();
    let (sacco, admin) = registered(&mut ledger);
    ledger.execute(
        &[meta(sacco), meta(admin)],
        Instruction::AddApprover { approver: aid(3) },
    );
    ledger.execute(
        &[meta(sacco), meta(admin)],
        Instruction::AddApprover { approver: aid(4) },
    );
    ledger.execute(
        &[meta(sacco), meta(aid(3))],
        Instruction::SetRequiredApprovals {
            required_approvals: 2,
        },
    );
}

#[test]
#[should_panic(expected = "Threshold must be between 2 and one less than the approver count")]
fn threshold_cannot_exceed_approver_count() {
    let mut ledger = Ledger::new();
    let (sacco, admin) = registered(&mut ledger);
    ledger.execute(
        &[meta(sacco), meta(admin)],
        Instruction::SetRequiredApprovals {
            required_approvals: 2,
        },
    );
}

#[test]
#[should_panic(expected = "Threshold must be between 2 and one less than the approver count")]
fn configured_threshold_must_have_multiple_approvers() {
    let mut ledger = Ledger::new();
    let (sacco, admin) = registered(&mut ledger);
    for approver in [aid(3), aid(4)] {
        ledger.execute(
            &[meta(sacco), meta(admin)],
            Instruction::AddApprover { approver },
        );
    }
    ledger.execute(
        &[meta(sacco), meta(admin)],
        Instruction::SetRequiredApprovals {
            required_approvals: 1,
        },
    );
}

#[test]
#[should_panic(expected = "Approver limit reached")]
fn approver_list_has_a_fixed_limit() {
    let mut ledger = Ledger::new();
    let (sacco_id, admin) = registered(&mut ledger);
    let mut sacco = ledger.sacco(sacco_id);
    sacco.approvers = vec![aid(2); sacco_ledger_core::MAX_APPROVERS];
    ledger.put(sacco_id, &sacco);
    ledger.execute(
        &[meta(sacco_id), meta(admin)],
        Instruction::AddApprover { approver: aid(3) },
    );
}

#[test]
#[should_panic(expected = "Member limit reached")]
fn member_list_has_a_fixed_limit() {
    let mut ledger = Ledger::new();
    let (sacco_id, admin) = registered(&mut ledger);
    let mut sacco = ledger.sacco(sacco_id);
    sacco.member_ids = vec![aid(10); sacco_ledger_core::MAX_SACCO_MEMBERS];
    ledger.put(sacco_id, &sacco);
    ledger.execute(
        &[meta(sacco_id), meta(aid(16)), meta(admin)],
        Instruction::AddMember { identity: aid(16) },
    );
}

#[test]
#[should_panic(expected = "Instruction cannot apply multiple effects to the same account")]
fn one_account_cannot_fill_two_effected_roles() {
    let mut ledger = Ledger::new();
    verified_sacco(&mut ledger);
    ledger.execute(
        &[meta(aid(1)), meta(aid(1)), meta(aid(7)), meta(aid(2))],
        Instruction::RecordDeposit {
            member: aid(1),
            amount: 1_000,
            deposit_type: sacco_ledger_core::DepositType::Savings,
            reference: "MPZA-alias".into(),
        },
    );
}

#[test]
#[should_panic(expected = "Approver already registered")]
fn add_approver_rejects_duplicate() {
    let mut ledger = Ledger::new();
    let (sacco, admin) = registered(&mut ledger);
    ledger.execute(
        &[meta(sacco), meta(admin)],
        Instruction::AddApprover { approver: admin },
    );
}

#[test]
fn add_supervisor_admin_only() {
    let mut ledger = Ledger::new();
    let (sacco, admin) = registered(&mut ledger);
    ledger.execute(
        &[meta(sacco), meta(admin)],
        Instruction::AddSupervisor { supervisor: aid(4) },
    );
    assert!(ledger.sacco(sacco).supervisory_committee.contains(&aid(4)));
}

#[test]
#[should_panic(expected = "Approvers cannot also sit on the Supervisory Committee")]
fn add_supervisor_rejects_current_approver() {
    let mut ledger = Ledger::new();
    let (sacco, admin) = registered(&mut ledger);
    ledger.execute(
        &[meta(sacco), meta(admin)],
        Instruction::AddSupervisor { supervisor: admin },
    );
}

#[test]
fn add_member_creates_member_account() {
    let mut ledger = Ledger::new();
    let (sacco, admin) = registered(&mut ledger);
    ledger.execute(
        &[meta(sacco), meta(aid(6)), meta(admin)],
        Instruction::AddMember { identity: aid(6) },
    );
    let member = ledger.member(aid(6));
    assert_eq!(member.sacco_id, sacco);
    assert_eq!(member.identity, aid(6));
    assert_eq!(member.savings_balance, 0);
    assert!(ledger.sacco(sacco).member_ids.contains(&aid(6)));
}

#[test]
#[should_panic(expected = "Member identity already registered")]
fn add_member_rejects_duplicate_identity() {
    let mut ledger = Ledger::new();
    let (sacco, admin) = registered(&mut ledger);
    ledger.execute(
        &[meta(sacco), meta(aid(6)), meta(admin)],
        Instruction::AddMember { identity: aid(6) },
    );
    ledger.execute(
        &[meta(sacco), meta(aid(6)), meta(admin)],
        Instruction::AddMember { identity: aid(6) },
    );
}
