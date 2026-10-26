//! Every instruction has an IDL entry.

use sacco_ledger_core::{AccountId, DepositType, Instruction, Totals};
use sacco_ledger_idl::{variant_name, INSTRUCTIONS};

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
    ]
}

#[test]
fn the_table_covers_every_instruction() {
    let table: Vec<&str> = INSTRUCTIONS.iter().map(|spec| spec.name).collect();
    for instruction in samples() {
        assert!(
            table.contains(&variant_name(&instruction)),
            "{} is missing from the IDL table",
            variant_name(&instruction)
        );
    }
    assert_eq!(
        table.len(),
        samples().len(),
        "the table has an entry the enum does not use"
    );
}

#[test]
fn the_committed_artifact_is_current() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../artifacts/sacco_ledger-idl.json"
    );
    let committed = std::fs::read_to_string(path).expect("committed IDL artifact");
    assert_eq!(
        committed,
        sacco_ledger_idl::render(),
        "run the generator and commit the result"
    );
}
