use lee::program::Program;

pub fn initialize_instruction_data(
) -> Result<nssa_core::program::InstructionData, lee::error::LeeError> {
    Program::serialize_instruction(sacco_ledger_core::Instruction::Initialize)
}

pub fn instruction_data_from_json(
    json: &str,
) -> anyhow::Result<nssa_core::program::InstructionData> {
    let instruction: sacco_ledger_core::Instruction = serde_json::from_str(json)?;
    Ok(Program::serialize_instruction(instruction)?)
}

#[cfg(test)]
mod tests {
    use super::{initialize_instruction_data, instruction_data_from_json};

    #[test]
    fn initialize_instruction_uses_the_first_borsh_enum_variant() {
        assert_eq!(
            initialize_instruction_data().map(|data| data.to_vec()).ok(),
            Some(vec![0]),
        );
    }

    #[test]
    fn idl_shaped_deposit_json_serializes_to_record_deposit() {
        let json = r#"{"RecordDeposit":{"member":"11111111111111111111111111111111","amount":100,"deposit_type":"Savings","reference":"DEMO-1"}}"#;
        assert_eq!(
            instruction_data_from_json(json)
                .map(|data| data.to_vec())
                .ok()
                .and_then(|data| data.first().copied()),
            Some(6)
        );
    }

    #[test]
    fn idl_shaped_expense_json_serializes_to_propose_expense() {
        let json = r#"{"ProposeExpense":{"amount":100,"purpose":"supplies","evidence_cid":"cid:test","required_approvals":2}}"#;
        assert_eq!(
            instruction_data_from_json(json)
                .map(|data| data.to_vec())
                .ok()
                .and_then(|data| data.first().copied()),
            Some(13)
        );
    }
}
