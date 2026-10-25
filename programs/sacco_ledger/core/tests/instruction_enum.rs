//! Instruction-enum contract test: all variants serde-roundtrip.

use nssa_core::account::AccountId;
use sacco_ledger_core::{DepositType, Instruction};

fn aid(n: u8) -> AccountId {
    AccountId::new([n; 32])
}

fn roundtrip(instruction: Instruction) -> Instruction {
    let json = serde_json::to_string(&instruction).expect("serialize");
    serde_json::from_str(&json).expect("deserialize")
}

#[test]
fn instruction_enum_roundtrips_all_variants() {
    let cases = vec![
        Instruction::Initialize,
        Instruction::RegisterSacco {
            name: "Kampala Traders SACCO".into(),
            registration_number: "UG-SACCO-001".into(),
            treasury: aid(1),
        },
        Instruction::VerifySacco,
        Instruction::AddApprover { approver: aid(2) },
        Instruction::AddSupervisor { supervisor: aid(3) },
        Instruction::AddMember { identity: aid(4) },
        Instruction::RecordDeposit {
            member: aid(5),
            amount: 250_000,
            deposit_type: DepositType::Savings,
            reference: "MPZA-99182".into(),
        },
        Instruction::ConfirmDeposit,
        Instruction::ApplyLoan {
            amount: 150_000,
            purpose: "School fees".into(),
        },
        Instruction::ApproveLoan,
        Instruction::RejectLoan,
        Instruction::DisburseLoan {
            evidence_cid: "bafy-disburse".into(),
        },
        Instruction::RecordRepayment {
            loan: aid(6),
            amount: 50_000,
            reference: "MPZA-99200".into(),
        },
        Instruction::ProposeExpense {
            amount: 75_000,
            purpose: "Stationery for the AGM".into(),
            evidence_cid: "bafy-expense".into(),
        },
        Instruction::ApproveExpense,
        Instruction::RejectExpense,
        Instruction::ExecuteExpense {
            executed_evidence: "bafy-execution".into(),
        },
        Instruction::PostPeriodSummary {
            label: "2026-11".into(),
        },
    ];

    assert_eq!(cases.len(), 18, "the instruction set is pinned at 18 variants");
    for instruction in cases {
        assert_eq!(roundtrip(instruction.clone()), instruction);
    }
}
