use lee::program::Program;

pub fn initialize_instruction_data(
) -> Result<nssa_core::program::InstructionData, lee::error::LeeError> {
    Program::serialize_instruction(sacco_ledger_core::Instruction::Initialize)
}

#[cfg(test)]
mod tests {
    use super::initialize_instruction_data;

    #[test]
    fn initialize_instruction_uses_the_first_borsh_enum_variant() {
        assert_eq!(
            initialize_instruction_data().map(|data| data.to_vec()).ok(),
            Some(vec![0]),
        );
    }
}
