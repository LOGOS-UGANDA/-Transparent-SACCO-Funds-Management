//! Instruction planning: assert what metadata can prove, then name one effect per shard.

use nssa_core::program::{AccountMeta, Plan, PlanInput};
use sacco_ledger_core::effect::{ConfigEffect, Effect};
use sacco_ledger_core::Instruction;

/// The registry layout version this build writes and expects.
pub const CONFIG_VERSION: u8 = 1;

/// Reads an account's signer flag, panicking with the instruction's own rule text.
#[must_use]
pub fn signer(account: &AccountMeta, rule: &str) -> nssa_core::account::AccountId {
    assert!(account.is_authorized, "{rule}");
    account.account_id
}

/// Builds the effect plan for one instruction.
pub fn plan(input: &PlanInput, instruction: Instruction) -> Plan {
    let mut plan = Plan::new(input);
    match instruction {
        Instruction::Initialize => {
            let [config] = <&[_; 1]>::try_from(input.accounts.as_slice())
                .expect("Initialize requires exactly 1 account");
            assert!(config.is_authorized, "Config account must be authorized");
            plan.effect(
                config,
                &Effect::Config(ConfigEffect::Init {
                    version: CONFIG_VERSION,
                }),
            );
        }
        other => panic!("instruction not ported yet: {other:?}"),
    }
    plan
}
