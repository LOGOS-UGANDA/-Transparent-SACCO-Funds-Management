//! Every instruction has an IDL entry whose args match the Instruction enum fields.

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
        Instruction::SetRequiredApprovals {
            required_approvals: 2,
        },
    ]
}

/// Serde field names for a sample instruction (order is not stable under serde_json).
fn serde_field_names(instruction: &Instruction) -> Vec<String> {
    match serde_json::to_value(instruction).expect("Instruction serializes") {
        // Unit variants serialize as a bare string tag.
        serde_json::Value::String(_) => Vec::new(),
        serde_json::Value::Object(map) => {
            assert_eq!(map.len(), 1, "one variant tag");
            match map.values().next().expect("variant payload") {
                serde_json::Value::Object(fields) => fields.keys().cloned().collect(),
                other => panic!("unexpected variant payload: {other}"),
            }
        }
        other => panic!("unexpected Instruction serialization: {other}"),
    }
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
fn arg_names_match_the_instruction_enum_fields() {
    for (index, instruction) in samples().into_iter().enumerate() {
        let name = variant_name(&instruction);
        let encoded = borsh::to_vec(&instruction).expect("instruction encodes");
        assert_eq!(
            usize::from(encoded[0]),
            index,
            "{name}: Borsh variant index"
        );
        assert_eq!(INSTRUCTIONS[index].name, name, "IDL variant order");
        let spec = INSTRUCTIONS
            .iter()
            .find(|spec| spec.name == name)
            .unwrap_or_else(|| panic!("{name} missing from IDL table"));
        let mut idl_args: Vec<&str> = spec.args.iter().map(|arg| arg.name).collect();
        let idl_args_in_order = idl_args.clone();
        let mut enum_fields = serde_field_names(&instruction);
        idl_args.sort_unstable();
        enum_fields.sort_unstable();
        assert_eq!(
            idl_args, enum_fields,
            "{name}: IDL args must match Instruction serde field names"
        );
        let serialized = serde_json::to_string(&instruction).expect("instruction serializes");
        let mut previous = 0;
        for arg in &idl_args_in_order {
            let marker = format!("\"{arg}\":");
            let position = serialized
                .find(&marker)
                .unwrap_or_else(|| panic!("{name}: serialized field {arg} missing"));
            assert!(
                position >= previous,
                "{name}: IDL arg order differs from enum"
            );
            previous = position;
        }
    }
}

#[test]
fn init_accounts_require_signers_except_the_derived_config_pda() {
    for spec in INSTRUCTIONS {
        for account in spec.accounts {
            if account.init {
                if spec.name == "initialize" && account.name == "config" {
                    assert!(
                        !account.signer,
                        "the deterministic config PDA must not require a wallet signer"
                    );
                } else {
                    assert!(
                        account.signer,
                        "{}: init account '{}' must be signer/authorized so plan can claim it",
                        spec.name, account.name
                    );
                }
            }
        }
    }
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
