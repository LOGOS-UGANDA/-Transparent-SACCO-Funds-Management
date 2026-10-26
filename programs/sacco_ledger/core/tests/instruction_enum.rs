//! Every instruction survives the Borsh round trip used by the wallet path.

use sacco_ledger_core::{AccountId, DepositType, Instruction, Totals};

fn round_trip(instruction: Instruction) {
    let bytes = borsh::to_vec(&instruction).expect("encode");
    let decoded: Instruction = borsh::from_slice(&bytes).expect("decode");
    assert_eq!(decoded, instruction);
}

#[test]
fn every_variant_round_trips() {
    let id = AccountId::new([7; 32]);
    for instruction in [
        Instruction::Initialize,
        Instruction::RegisterSacco {
            name: "Kampala Traders SACCO".into(),
            registration_number: "UG-SACCO-001".into(),
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
            reference: "MPZA-1".into(),
        },
        Instruction::ConfirmDeposit,
        Instruction::ApplyLoan {
            amount: 1,
            purpose: "school fees".into(),
            required_approvals: 2,
        },
        Instruction::ApproveLoan,
        Instruction::RejectLoan,
        Instruction::DisburseLoan {
            evidence_cid: "cid".into(),
            principal: 1,
        },
        Instruction::RecordRepayment {
            loan: id,
            amount: 1,
            reference: "MPZA-2".into(),
        },
        Instruction::ProposeExpense {
            amount: 1,
            purpose: "desk".into(),
            evidence_cid: "cid".into(),
            required_approvals: 2,
        },
        Instruction::ApproveExpense,
        Instruction::RejectExpense,
        Instruction::ExecuteExpense {
            executed_evidence: "receipt".into(),
            amount: 1,
        },
        Instruction::PostPeriodSummary {
            label: "2026-11".into(),
            totals: Totals::default(),
        },
    ] {
        round_trip(instruction);
    }
}
