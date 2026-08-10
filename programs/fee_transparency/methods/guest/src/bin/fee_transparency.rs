#![cfg_attr(not(test), no_main)]

use fee_transparency_core::InstitutionType;
use nssa_core::{
    account::{AccountId, AccountWithMetadata},
};
use spel_framework::context::ProgramContext;
use spel_framework::prelude::*;

#[cfg(not(test))]
risc0_zkvm::guest::entry!(main);

#[lez_program(instruction = "fee_transparency_core::Instruction")]
mod fee_transparency {
    #[expect(
        unused_imports,
        reason = "SPEL instruction macro requires importing parent-scope handler types"
    )]
    use super::*;

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
        // v1: enforce the 100% cap using only categories already listed on the
        // institution account once category account fetching is wired in the client.
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
        _ctx: ProgramContext,
        #[account(mut)]
        institution: AccountWithMetadata,
        #[account(mut)]
        payment: AccountWithMetadata,
        #[account(signer)]
        payer: AccountWithMetadata,
        payer_name: String,
        reference_number: String,
        amount: u128,
    ) -> SpelResult {
        // Category account list is supplied by richer client txs in a follow-up;
        // scaffold records the payment with an empty live category set.
        let categories = vec![];
        let post_states = fee_transparency_program::pay_fees::pay_fees(
            institution,
            payment,
            payer,
            categories,
            payer_name,
            reference_number,
            amount,
        );
        Ok(spel_framework::SpelOutput::execute(post_states, vec![]))
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
        _ctx: ProgramContext,
        #[account(mut)]
        institution: AccountWithMetadata,
        #[account(mut)]
        category: AccountWithMetadata,
        #[account(mut)]
        release: AccountWithMetadata,
        #[account(mut)]
        treasury: AccountWithMetadata,
    ) -> SpelResult {
        let post_states = fee_transparency_program::execute_release::execute_release(
            institution,
            category,
            release,
            treasury,
        );
        Ok(spel_framework::SpelOutput::execute(post_states, vec![]))
    }
}
