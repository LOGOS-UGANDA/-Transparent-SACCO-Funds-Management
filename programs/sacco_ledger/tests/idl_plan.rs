//! IDL signer declarations must match the plan's authorization checks.

use nssa_core::account::AccountId;
use nssa_core::program::{AccountMeta, PlanInput};
use sacco_ledger_core::{DepositType, Instruction, Totals};
use sacco_ledger_idl::INSTRUCTIONS;
use sacco_ledger_program::plan;

const PROGRAM: AccountId = AccountId::new([0xAA; 32]);

fn samples() -> Vec<Instruction> {
    let id = AccountId::new([7; 32]);
    vec![
        Instruction::Initialize,
        Instruction::RegisterSacco {
            name: "n".into(),
            registration_number: "r".into(),
            treasury: id,
        },
        Instruction::VerifySacco,
        Instruction::AddApprover { approver: id },
        Instruction::AddSupervisor { supervisor: id },
        Instruction::AddMember { identity: id },
        Instruction::RecordDeposit {
            member: id,
            amount: 1,
            deposit_type: DepositType::Savings,
            reference: "r".into(),
        },
        Instruction::ConfirmDeposit,
        Instruction::ApplyLoan {
            amount: 1,
            purpose: "p".into(),
            required_approvals: 1,
        },
        Instruction::ApproveLoan,
        Instruction::RejectLoan,
        Instruction::DisburseLoan {
            evidence_cid: "c".into(),
            principal: 1,
        },
        Instruction::RecordRepayment {
            loan: id,
            amount: 1,
            reference: "r".into(),
        },
        Instruction::ProposeExpense {
            amount: 1,
            purpose: "p".into(),
            evidence_cid: "c".into(),
            required_approvals: 1,
        },
        Instruction::ApproveExpense,
        Instruction::RejectExpense,
        Instruction::ExecuteExpense {
            executed_evidence: "e".into(),
            amount: 1,
        },
        Instruction::PostPeriodSummary {
            label: "l".into(),
            totals: Totals::default(),
        },
        Instruction::SetRequiredApprovals {
            required_approvals: 2,
        },
    ]
}

#[test]
fn idl_signer_flags_match_plan_authorization() {
    let samples = samples();
    assert_eq!(samples.len(), INSTRUCTIONS.len());

    for (index, (instruction, spec)) in samples.iter().zip(INSTRUCTIONS).enumerate() {
        let accounts: Vec<_> = spec
            .accounts
            .iter()
            .enumerate()
            .map(|(slot, account)| {
                let account_id = if spec.name == "record_deposit"
                    && account.name == "member_account"
                    || spec.name == "record_repayment" && account.name == "loan_account"
                {
                    AccountId::new([7; 32])
                } else {
                    AccountId::new([(index * 16 + slot + 1) as u8; 32])
                };
                AccountMeta::new(account_id, account.signer, PROGRAM)
            })
            .collect();
        let input = PlanInput {
            self_account_id: PROGRAM,
            caller_account_id: None,
            accounts: accounts.clone(),
            instruction_data: borsh::to_vec(instruction).expect("instruction encodes"),
        };
        let _planned = plan(&input, instruction.clone());

        for (slot, account) in spec.accounts.iter().enumerate() {
            if !account.signer {
                continue;
            }
            let mut unsigned_accounts = accounts.clone();
            unsigned_accounts[slot].is_authorized = false;
            let unsigned_input = PlanInput {
                accounts: unsigned_accounts,
                ..input.clone()
            };
            assert!(
                std::panic::catch_unwind(|| plan(&unsigned_input, instruction.clone())).is_err(),
                "{} account {} is declared signer but plan accepts it unsigned",
                spec.name,
                account.name
            );
        }
    }
}
