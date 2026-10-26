//! Instruction planning: assert what metadata can prove, then name one effect per shard.

use nssa_core::program::{AccountMeta, Plan, PlanInput};
use sacco_ledger_core::effect::{ConfigEffect, DepositEffect, Effect, MemberEffect, SaccoEffect};
use sacco_ledger_core::{Deposit, Instruction};

/// The registry layout version this build writes and expects.
pub const CONFIG_VERSION: u8 = 1;

/// Reads an account's signer flag, panicking with the instruction's own rule text.
#[must_use]
pub fn signer(account: &AccountMeta, rule: &str) -> nssa_core::account::AccountId {
    assert!(account.is_authorized, "{rule}");
    account.account_id
}

/// Builds the effect plan for one instruction.
pub fn plan(input: &PlanInput, instruction: Instruction) -> Plan {
    let mut plan = Plan::new(input);
    match instruction {
        Instruction::Initialize => {
            let [config] = <&[_; 1]>::try_from(input.accounts.as_slice())
                .expect("Initialize requires exactly 1 account");
            assert!(config.is_authorized, "Config account must be authorized");
            plan.effect(
                config,
                &Effect::Config(ConfigEffect::Init {
                    version: CONFIG_VERSION,
                }),
            );
        }
        Instruction::RegisterSacco {
            name,
            registration_number,
            treasury,
        } => {
            let [sacco, admin] = <&[_; 2]>::try_from(input.accounts.as_slice())
                .expect("RegisterSacco requires sacco and admin");
            assert!(sacco.is_authorized, "Sacco account must be authorized");
            let admin = signer(admin, "Admin must authorize registration");
            plan.effect(
                sacco,
                &Effect::Sacco(SaccoEffect::Register {
                    name,
                    registration_number,
                    treasury,
                    admin,
                }),
            );
        }
        Instruction::VerifySacco => {
            let [sacco, registrar] = <&[_; 2]>::try_from(input.accounts.as_slice())
                .expect("VerifySacco requires sacco and registrar");
            let registrar = signer(registrar, "Registrar must authorize verification");
            plan.effect(sacco, &Effect::Sacco(SaccoEffect::Verify { registrar }));
        }
        Instruction::AddApprover { approver } => {
            let [sacco, admin] = <&[_; 2]>::try_from(input.accounts.as_slice())
                .expect("AddApprover requires sacco and admin");
            let admin = signer(admin, "Admin must authorize");
            plan.effect(
                sacco,
                &Effect::Sacco(SaccoEffect::AddApprover { admin, approver }),
            );
        }
        Instruction::AddSupervisor { supervisor } => {
            let [sacco, admin] = <&[_; 2]>::try_from(input.accounts.as_slice())
                .expect("AddSupervisor requires sacco and admin");
            let admin = signer(admin, "Admin must authorize");
            plan.effect(
                sacco,
                &Effect::Sacco(SaccoEffect::AddSupervisor { admin, supervisor }),
            );
        }
        Instruction::AddMember { identity } => {
            let [sacco, member, admin] = <&[_; 3]>::try_from(input.accounts.as_slice())
                .expect("AddMember requires sacco, member and admin");
            assert!(member.is_authorized, "Member account must be authorized");
            let admin = signer(admin, "Admin must authorize");
            plan.effect(
                sacco,
                &Effect::Sacco(SaccoEffect::AddMember { admin, identity }),
            );
            plan.effect(
                member,
                &Effect::Member(MemberEffect::Init {
                    sacco: sacco.account_id,
                    identity,
                }),
            );
        }
        Instruction::RecordDeposit {
            member,
            amount,
            deposit_type,
            reference,
        } => {
            let [sacco, member_account, deposit, recorder] =
                <&[_; 4]>::try_from(input.accounts.as_slice())
                    .expect("RecordDeposit requires sacco, member, deposit and recorder");
            assert!(deposit.is_authorized, "Deposit account must be authorized");
            assert_eq!(
                member_account.account_id, member,
                "Member account does not match the member id argument"
            );
            let recorder = signer(recorder, "Recorder must authorize");

            plan.effect(
                sacco,
                &Effect::Sacco(SaccoEffect::RecordDeposit {
                    recorder,
                    amount,
                    deposit_type,
                }),
            );
            plan.effect(
                member_account,
                &Effect::Member(MemberEffect::RecordDeposit {
                    sacco: sacco.account_id,
                    amount,
                    deposit_type,
                }),
            );
            plan.effect(
                deposit,
                &Effect::Deposit(DepositEffect::Init(Deposit {
                    sacco_id: sacco.account_id,
                    member_id: member,
                    amount,
                    deposit_type,
                    reference,
                    recorded_by: recorder,
                    confirmed_by_member: false,
                })),
            );
        }
        Instruction::ConfirmDeposit => {
            let [deposit, member] = <&[_; 2]>::try_from(input.accounts.as_slice())
                .expect("ConfirmDeposit requires deposit and member");
            let member = signer(member, "Member must authorize confirmation");
            plan.effect(deposit, &Effect::Deposit(DepositEffect::Confirm { member }));
        }
        other => panic!("instruction not ported yet: {other:?}"),
    }
    plan
}
