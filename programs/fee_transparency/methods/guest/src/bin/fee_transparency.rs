#![cfg_attr(not(test), no_main)]

use fee_transparency_core::InstitutionType;
use nssa_core::{
    account::{Account, AccountId, AccountWithMetadata},
    program::ProgramId,
};
use spel_framework::context::ProgramContext;
use spel_framework::prelude::*;

#[cfg(not(test))]
risc0_zkvm::guest::entry!(main);

fn collect_categories(
    category_0: AccountWithMetadata,
    category_1: AccountWithMetadata,
    category_2: AccountWithMetadata,
    category_3: AccountWithMetadata,
    category_4: AccountWithMetadata,
    category_5: AccountWithMetadata,
    category_6: AccountWithMetadata,
    category_7: AccountWithMetadata,
) -> Vec<AccountWithMetadata> {
    [
        category_0, category_1, category_2, category_3, category_4, category_5, category_6,
        category_7,
    ]
    .into_iter()
    .filter(|account| account.account != Account::default())
    .collect()
}

#[lez_program(instruction = "fee_transparency_core::Instruction")]
mod fee_transparency {
    #[expect(
        unused_imports,
        reason = "SPEL instruction macro requires importing parent-scope handler types"
    )]
    use super::*;

    #[instruction]
    pub fn initialize(
        ctx: ProgramContext,
        #[account(mut)]
        config: AccountWithMetadata,
        token_program_id: ProgramId,
    ) -> SpelResult {
        let post_states = fee_transparency_program::initialize::initialize(
            config,
            token_program_id,
            ctx.self_program_id,
        );
        Ok(spel_framework::SpelOutput::execute(post_states, vec![]))
    }

    #[instruction]
    pub fn register_institution(
        _ctx: ProgramContext,
        #[account(mut)]
        institution: AccountWithMetadata,
        #[account(signer)]
        admin: AccountWithMetadata,
        name: String,
        registration_number: String,
        institution_type: InstitutionType,
        treasury: AccountId,
        required_approvals: u8,
    ) -> SpelResult {
        let post_states = fee_transparency_program::register_institution::register_institution(
            institution,
            admin,
            name,
            registration_number,
            institution_type,
            treasury,
            required_approvals,
        );
        Ok(spel_framework::SpelOutput::execute(post_states, vec![]))
    }

    #[instruction]
    pub fn verify_institution(
        _ctx: ProgramContext,
        #[account(mut)]
        institution: AccountWithMetadata,
        #[account(signer)]
        registrar: AccountWithMetadata,
    ) -> SpelResult {
        let post_states = fee_transparency_program::verify_institution::verify_institution(
            institution,
            registrar,
        );
        Ok(spel_framework::SpelOutput::execute(post_states, vec![]))
    }

    #[instruction]
    pub fn initialize_institution_vault(
        ctx: ProgramContext,
        config: AccountWithMetadata,
        #[account(mut)]
        institution: AccountWithMetadata,
        token_definition: AccountWithMetadata,
        #[account(mut)]
        vault: AccountWithMetadata,
        #[account(signer)]
        admin: AccountWithMetadata,
    ) -> SpelResult {
        let post_states =
            fee_transparency_program::initialize_institution_vault::initialize_institution_vault(
                config,
                institution,
                token_definition,
                vault,
                admin,
                ctx.self_program_id,
            );
        Ok(spel_framework::SpelOutput::execute(post_states, vec![]))
    }

    #[instruction]
    pub fn add_approver(
        _ctx: ProgramContext,
        #[account(mut)]
        institution: AccountWithMetadata,
        #[account(signer)]
        admin: AccountWithMetadata,
        approver: AccountId,
    ) -> SpelResult {
        let post_states = fee_transparency_program::add_approver::add_approver(
            institution,
            admin,
            approver,
        );
        Ok(spel_framework::SpelOutput::execute(post_states, vec![]))
    }

    #[instruction]
    pub fn create_fee_category(
        _ctx: ProgramContext,
        #[account(mut)]
        institution: AccountWithMetadata,
        #[account(mut)]
        category: AccountWithMetadata,
        #[account(signer)]
        admin: AccountWithMetadata,
        name: String,
        description: String,
        allocated_percentage_bps: u16,
    ) -> SpelResult {
        let existing = vec![];
        let post_states = fee_transparency_program::create_fee_category::create_fee_category(
            institution,
            category,
            admin,
            name,
            description,
            allocated_percentage_bps,
            &existing,
        );
        Ok(spel_framework::SpelOutput::execute(post_states, vec![]))
    }

    #[instruction]
    pub fn pay_fees(
        ctx: ProgramContext,
        config: AccountWithMetadata,
        #[account(mut)]
        institution: AccountWithMetadata,
        #[account(mut)]
        payment: AccountWithMetadata,
        #[account(signer)]
        payer: AccountWithMetadata,
        #[account(mut)]
        payer_holding: AccountWithMetadata,
        #[account(mut)]
        vault: AccountWithMetadata,
        #[account(mut)]
        category_0: AccountWithMetadata,
        #[account(mut)]
        category_1: AccountWithMetadata,
        #[account(mut)]
        category_2: AccountWithMetadata,
        #[account(mut)]
        category_3: AccountWithMetadata,
        #[account(mut)]
        category_4: AccountWithMetadata,
        #[account(mut)]
        category_5: AccountWithMetadata,
        #[account(mut)]
        category_6: AccountWithMetadata,
        #[account(mut)]
        category_7: AccountWithMetadata,
        payer_name: String,
        reference_number: String,
        amount: u128,
    ) -> SpelResult {
        let categories = collect_categories(
            category_0,
            category_1,
            category_2,
            category_3,
            category_4,
            category_5,
            category_6,
            category_7,
        );
        let (post_states, chained_calls) = fee_transparency_program::pay_fees::pay_fees(
            config,
            institution,
            payment,
            payer,
            payer_holding,
            vault,
            categories,
            payer_name,
            reference_number,
            amount,
            ctx.self_program_id,
        );
        Ok(spel_framework::SpelOutput::execute(post_states, chained_calls))
    }

    #[instruction]
    pub fn request_release(
        _ctx: ProgramContext,
        institution: AccountWithMetadata,
        category: AccountWithMetadata,
        #[account(mut)]
        release: AccountWithMetadata,
        #[account(signer)]
        admin: AccountWithMetadata,
        amount: u128,
        purpose: String,
        evidence_hash: String,
    ) -> SpelResult {
        let post_states = fee_transparency_program::request_release::request_release(
            institution,
            category,
            release,
            admin,
            amount,
            purpose,
            evidence_hash,
        );
        Ok(spel_framework::SpelOutput::execute(post_states, vec![]))
    }

    #[instruction]
    pub fn approve_release(
        _ctx: ProgramContext,
        institution: AccountWithMetadata,
        #[account(mut)]
        release: AccountWithMetadata,
        #[account(signer)]
        approver: AccountWithMetadata,
    ) -> SpelResult {
        let post_states = fee_transparency_program::approve_release::approve_release(
            institution,
            release,
            approver,
        );
        Ok(spel_framework::SpelOutput::execute(post_states, vec![]))
    }

    #[instruction]
    pub fn reject_release(
        _ctx: ProgramContext,
        institution: AccountWithMetadata,
        #[account(mut)]
        release: AccountWithMetadata,
        #[account(signer)]
        approver: AccountWithMetadata,
    ) -> SpelResult {
        let post_states = fee_transparency_program::reject_release::reject_release(
            institution,
            release,
            approver,
        );
        Ok(spel_framework::SpelOutput::execute(post_states, vec![]))
    }

    #[instruction]
    pub fn execute_release(
        ctx: ProgramContext,
        config: AccountWithMetadata,
        #[account(mut)]
        institution: AccountWithMetadata,
        #[account(mut)]
        category: AccountWithMetadata,
        #[account(mut)]
        release: AccountWithMetadata,
        #[account(mut)]
        vault: AccountWithMetadata,
        #[account(mut)]
        treasury_holding: AccountWithMetadata,
    ) -> SpelResult {
        let (post_states, chained_calls) =
            fee_transparency_program::execute_release::execute_release(
                config,
                institution,
                category,
                release,
                vault,
                treasury_holding,
                ctx.self_program_id,
            );
        Ok(spel_framework::SpelOutput::execute(post_states, chained_calls))
    }
}
