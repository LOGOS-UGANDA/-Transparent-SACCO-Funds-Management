#![cfg_attr(not(test), no_main)]

use nssa_core::account::{AccountId, AccountWithMetadata};
use sacco_ledger_core::DepositType;
use spel_framework::context::ProgramContext;
use spel_framework::prelude::*;

#[cfg(not(test))]
risc0_zkvm::guest::entry!(main);

#[lez_program(instruction = "sacco_ledger_core::Instruction")]
mod sacco_ledger {
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
    ) -> SpelResult {
        let post_states = sacco_ledger_program::initialize::initialize(config, ctx.self_account_id);
        Ok(spel_framework::SpelOutput::execute(post_states, vec![]))
    }

    #[instruction]
    pub fn register_sacco(
        _ctx: ProgramContext,
        #[account(mut)]
        sacco: AccountWithMetadata,
        #[account(signer)]
        admin: AccountWithMetadata,
        name: String,
        registration_number: String,
        treasury: AccountId,
    ) -> SpelResult {
        let post_states = sacco_ledger_program::register_sacco::register_sacco(
            sacco,
            admin,
            name,
            registration_number,
            treasury,
        );
        Ok(spel_framework::SpelOutput::execute(post_states, vec![]))
    }

    #[instruction]
    pub fn verify_sacco(
        _ctx: ProgramContext,
        #[account(mut)]
        sacco: AccountWithMetadata,
        #[account(signer)]
        registrar: AccountWithMetadata,
    ) -> SpelResult {
        let post_states = sacco_ledger_program::verify_sacco::verify_sacco(sacco, registrar);
        Ok(spel_framework::SpelOutput::execute(post_states, vec![]))
    }

    #[instruction]
    pub fn add_approver(
        _ctx: ProgramContext,
        #[account(mut)]
        sacco: AccountWithMetadata,
        #[account(signer)]
        admin: AccountWithMetadata,
        approver: AccountId,
    ) -> SpelResult {
        let post_states =
            sacco_ledger_program::add_approver::add_approver(sacco, admin, approver);
        Ok(spel_framework::SpelOutput::execute(post_states, vec![]))
    }

    #[instruction]
    pub fn add_supervisor(
        _ctx: ProgramContext,
        #[account(mut)]
        sacco: AccountWithMetadata,
        #[account(signer)]
        admin: AccountWithMetadata,
        supervisor: AccountId,
    ) -> SpelResult {
        let post_states =
            sacco_ledger_program::add_supervisor::add_supervisor(sacco, admin, supervisor);
        Ok(spel_framework::SpelOutput::execute(post_states, vec![]))
    }

    #[instruction]
    pub fn add_member(
        _ctx: ProgramContext,
        #[account(mut)]
        sacco: AccountWithMetadata,
        #[account(mut)]
        member: AccountWithMetadata,
        #[account(signer)]
        admin: AccountWithMetadata,
        identity: AccountId,
    ) -> SpelResult {
        let post_states =
            sacco_ledger_program::add_member::add_member(sacco, member, admin, identity);
        Ok(spel_framework::SpelOutput::execute(post_states, vec![]))
    }

    #[instruction]
    pub fn record_deposit(
        _ctx: ProgramContext,
        #[account(mut)]
        sacco: AccountWithMetadata,
        #[account(mut)]
        member_account: AccountWithMetadata,
        #[account(mut)]
        deposit: AccountWithMetadata,
        #[account(signer)]
        recorder: AccountWithMetadata,
        member: AccountId,
        amount: u128,
        deposit_type: DepositType,
        reference: String,
    ) -> SpelResult {
        let post_states = sacco_ledger_program::record_deposit::record_deposit(
            member,
            sacco,
            member,
            deposit,
            recorder,
            amount,
            deposit_type,
            reference,
        );
        Ok(spel_framework::SpelOutput::execute(post_states, vec![]))
    }

    #[instruction]
    pub fn confirm_deposit(
        _ctx: ProgramContext,
        #[account(mut)]
        deposit: AccountWithMetadata,
        #[account(signer)]
        member: AccountWithMetadata,
    ) -> SpelResult {
        let post_states = sacco_ledger_program::confirm_deposit::confirm_deposit(deposit, member);
        Ok(spel_framework::SpelOutput::execute(post_states, vec![]))
    }

    #[instruction]
    pub fn apply_loan(
        _ctx: ProgramContext,
        #[account(mut)]
        sacco: AccountWithMetadata,
        #[account(mut)]
        member: AccountWithMetadata,
        #[account(mut)]
        loan: AccountWithMetadata,
        #[account(signer)]
        member_signer: AccountWithMetadata,
        amount: u128,
        purpose: String,
    ) -> SpelResult {
        let post_states = sacco_ledger_program::apply_loan::apply_loan(
            sacco,
            member,
            loan,
            member_signer,
            amount,
            purpose,
        );
        Ok(spel_framework::SpelOutput::execute(post_states, vec![]))
    }

    #[instruction]
    pub fn approve_loan(
        _ctx: ProgramContext,
        sacco: AccountWithMetadata,
        #[account(mut)]
        loan: AccountWithMetadata,
        #[account(signer)]
        approver: AccountWithMetadata,
    ) -> SpelResult {
        let post_states = sacco_ledger_program::approve_loan::approve_loan(sacco, loan, approver);
        Ok(spel_framework::SpelOutput::execute(post_states, vec![]))
    }

    #[instruction]
    pub fn reject_loan(
        _ctx: ProgramContext,
        sacco: AccountWithMetadata,
        #[account(mut)]
        loan: AccountWithMetadata,
        #[account(signer)]
        approver: AccountWithMetadata,
    ) -> SpelResult {
        let post_states = sacco_ledger_program::reject_loan::reject_loan(sacco, loan, approver);
        Ok(spel_framework::SpelOutput::execute(post_states, vec![]))
    }

    #[instruction]
    pub fn disburse_loan(
        _ctx: ProgramContext,
        #[account(mut)]
        sacco: AccountWithMetadata,
        member: AccountWithMetadata,
        #[account(mut)]
        loan: AccountWithMetadata,
        #[account(signer)]
        approver: AccountWithMetadata,
        evidence_cid: String,
    ) -> SpelResult {
        let post_states = sacco_ledger_program::disburse_loan::disburse_loan(
            sacco,
            member,
            loan,
            approver,
            evidence_cid,
        );
        Ok(spel_framework::SpelOutput::execute(post_states, vec![]))
    }

    #[instruction]
    pub fn record_repayment(
        _ctx: ProgramContext,
        #[account(mut)]
        sacco: AccountWithMetadata,
        member: AccountWithMetadata,
        #[account(mut)]
        loan: AccountWithMetadata,
        #[account(signer)]
        approver: AccountWithMetadata,
        loan: AccountId,
        amount: u128,
        reference: String,
    ) -> SpelResult {
        let post_states = sacco_ledger_program::record_repayment::record_repayment(
            sacco,
            member,
            loan,
            approver,
            loan,
            amount,
            reference,
        );
        Ok(spel_framework::SpelOutput::execute(post_states, vec![]))
    }

    #[instruction]
    pub fn propose_expense(
        _ctx: ProgramContext,
        sacco: AccountWithMetadata,
        #[account(mut)]
        expense: AccountWithMetadata,
        #[account(signer)]
        proposer: AccountWithMetadata,
        amount: u128,
        purpose: String,
        evidence_cid: String,
    ) -> SpelResult {
        let post_states = sacco_ledger_program::propose_expense::propose_expense(
            sacco,
            expense,
            proposer,
            amount,
            purpose,
            evidence_cid,
        );
        Ok(spel_framework::SpelOutput::execute(post_states, vec![]))
    }

    #[instruction]
    pub fn approve_expense(
        _ctx: ProgramContext,
        sacco: AccountWithMetadata,
        #[account(mut)]
        expense: AccountWithMetadata,
        #[account(signer)]
        approver: AccountWithMetadata,
    ) -> SpelResult {
        let post_states =
            sacco_ledger_program::approve_expense::approve_expense(sacco, expense, approver);
        Ok(spel_framework::SpelOutput::execute(post_states, vec![]))
    }

    #[instruction]
    pub fn reject_expense(
        _ctx: ProgramContext,
        sacco: AccountWithMetadata,
        #[account(mut)]
        expense: AccountWithMetadata,
        #[account(signer)]
        approver: AccountWithMetadata,
    ) -> SpelResult {
        let post_states =
            sacco_ledger_program::reject_expense::reject_expense(sacco, expense, approver);
        Ok(spel_framework::SpelOutput::execute(post_states, vec![]))
    }

    #[instruction]
    pub fn execute_expense(
        _ctx: ProgramContext,
        #[account(mut)]
        sacco: AccountWithMetadata,
        #[account(mut)]
        expense: AccountWithMetadata,
        #[account(signer)]
        approver: AccountWithMetadata,
        executed_evidence: String,
    ) -> SpelResult {
        let post_states = sacco_ledger_program::execute_expense::execute_expense(
            sacco,
            expense,
            approver,
            executed_evidence,
        );
        Ok(spel_framework::SpelOutput::execute(post_states, vec![]))
    }

    #[instruction]
    pub fn post_period_summary(
        _ctx: ProgramContext,
        sacco: AccountWithMetadata,
        #[account(mut)]
        summary: AccountWithMetadata,
        #[account(signer)]
        poster: AccountWithMetadata,
        label: String,
    ) -> SpelResult {
        let post_states = sacco_ledger_program::post_period_summary::post_period_summary(
            sacco,
            summary,
            poster,
            label,
        );
        Ok(spel_framework::SpelOutput::execute(post_states, vec![]))
    }
}

