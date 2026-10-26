//! Instruction planning: assert what metadata can prove, then name one effect per shard.

use nssa_core::program::{AccountMeta, Plan, PlanInput};
use sacco_ledger_core::effect::{
    ConfigEffect, DepositEffect, Effect, ExpenseEffect, LoanEffect, MemberEffect, SaccoEffect,
};
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
        Instruction::ApplyLoan {
            amount,
            purpose,
            required_approvals,
        } => {
            let [sacco, member, loan, member_signer] =
                <&[_; 4]>::try_from(input.accounts.as_slice())
                    .expect("ApplyLoan requires sacco, member, loan and member signer");
            assert!(loan.is_authorized, "Loan account must be authorized");
            let signer = signer(member_signer, "Member must authorize the application");
            plan.effect(
                sacco,
                &Effect::Sacco(SaccoEffect::AuthorizeProposal { required_approvals }),
            );
            plan.effect(
                member,
                &Effect::Member(MemberEffect::AttachLoan {
                    sacco: sacco.account_id,
                    loan: loan.account_id,
                    signer,
                }),
            );
            plan.effect(
                loan,
                &Effect::Loan(LoanEffect::Init {
                    sacco: sacco.account_id,
                    member: member.account_id,
                    principal: amount,
                    purpose,
                    required_approvals,
                }),
            );
        }
        Instruction::ApproveLoan => {
            let [sacco, loan, approver] = <&[_; 3]>::try_from(input.accounts.as_slice())
                .expect("ApproveLoan requires sacco, loan and approver");
            let approver = signer(approver, "Approver must authorize");
            plan.effect(
                sacco,
                &Effect::Sacco(SaccoEffect::AuthorizeApprover { approver }),
            );
            plan.effect(loan, &Effect::Loan(LoanEffect::Approve { approver }));
        }
        Instruction::RejectLoan => {
            let [sacco, loan, approver] = <&[_; 3]>::try_from(input.accounts.as_slice())
                .expect("RejectLoan requires sacco, loan and approver");
            let approver = signer(approver, "Approver must authorize");
            plan.effect(
                sacco,
                &Effect::Sacco(SaccoEffect::AuthorizeApprover { approver }),
            );
            plan.effect(loan, &Effect::Loan(LoanEffect::Reject { approver }));
        }
        Instruction::DisburseLoan {
            evidence_cid,
            principal,
        } => {
            let [sacco, _member, loan, approver] = <&[_; 4]>::try_from(input.accounts.as_slice())
                .expect("DisburseLoan requires sacco, member, loan and approver");
            let approver = signer(approver, "Approver must authorize");
            plan.effect(
                sacco,
                &Effect::Sacco(SaccoEffect::DisburseLoan {
                    approver,
                    principal,
                }),
            );
            plan.effect(
                loan,
                &Effect::Loan(LoanEffect::Disburse {
                    evidence_cid,
                    principal,
                }),
            );
        }
        Instruction::RecordRepayment {
            loan,
            amount,
            reference,
        } => {
            let [sacco, _member, loan_account, approver] =
                <&[_; 4]>::try_from(input.accounts.as_slice())
                    .expect("RecordRepayment requires sacco, member, loan and approver");
            assert_eq!(
                loan_account.account_id, loan,
                "Loan account does not match the loan id argument"
            );
            let approver = signer(approver, "Approver must authorize");
            plan.effect(
                sacco,
                &Effect::Sacco(SaccoEffect::RecordRepayment { approver, amount }),
            );
            plan.effect(
                loan_account,
                &Effect::Loan(LoanEffect::Repay { amount, reference }),
            );
        }
        Instruction::ProposeExpense {
            amount,
            purpose,
            evidence_cid,
            required_approvals,
        } => {
            let [sacco, expense, proposer] = <&[_; 3]>::try_from(input.accounts.as_slice())
                .expect("ProposeExpense requires sacco, expense and proposer");
            assert!(expense.is_authorized, "Expense account must be authorized");
            let proposer = signer(proposer, "Proposer must authorize");
            plan.effect(
                sacco,
                &Effect::Sacco(SaccoEffect::ProposeExpense {
                    proposer,
                    amount,
                    required_approvals,
                }),
            );
            plan.effect(
                expense,
                &Effect::Expense(ExpenseEffect::Init {
                    sacco: sacco.account_id,
                    amount,
                    purpose,
                    evidence_cid,
                    requested_by: proposer,
                    required_approvals,
                }),
            );
        }
        Instruction::ApproveExpense => {
            let [sacco, expense, approver] = <&[_; 3]>::try_from(input.accounts.as_slice())
                .expect("ApproveExpense requires sacco, expense and approver");
            let approver = signer(approver, "Approver must authorize");
            plan.effect(
                sacco,
                &Effect::Sacco(SaccoEffect::AuthorizeApprover { approver }),
            );
            plan.effect(
                expense,
                &Effect::Expense(ExpenseEffect::Approve { approver }),
            );
        }
        Instruction::RejectExpense => {
            let [sacco, expense, approver] = <&[_; 3]>::try_from(input.accounts.as_slice())
                .expect("RejectExpense requires sacco, expense and approver");
            let approver = signer(approver, "Approver must authorize");
            plan.effect(
                sacco,
                &Effect::Sacco(SaccoEffect::AuthorizeApprover { approver }),
            );
            plan.effect(
                expense,
                &Effect::Expense(ExpenseEffect::Reject { approver }),
            );
        }
        Instruction::ExecuteExpense {
            executed_evidence,
            amount,
        } => {
            let [sacco, expense, approver] = <&[_; 3]>::try_from(input.accounts.as_slice())
                .expect("ExecuteExpense requires sacco, expense and approver");
            let approver = signer(approver, "Approver must authorize");
            plan.effect(
                sacco,
                &Effect::Sacco(SaccoEffect::ExecuteExpense { approver, amount }),
            );
            plan.effect(
                expense,
                &Effect::Expense(ExpenseEffect::Execute {
                    executed_evidence,
                    amount,
                }),
            );
        }
        other => panic!("instruction not ported yet: {other:?}"),
    }
    plan
}
