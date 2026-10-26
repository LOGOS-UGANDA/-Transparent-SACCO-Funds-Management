//! Foundation: config initialisation and the harness itself.

mod support;

use sacco_ledger_core::{compute_config_pda, Instruction};
use support::{aid, foreign, meta, unsigned, Ledger, PROGRAM};

#[test]
fn initialize_creates_config() {
    let config = compute_config_pda(PROGRAM);
    let mut ledger = Ledger::new();
    ledger.execute(&[unsigned(config)], Instruction::Initialize);
    assert_eq!(ledger.config(config).version, 1);
}

#[test]
#[should_panic(expected = "Config account must match the program-derived address")]
fn initialize_rejects_a_non_pda_config_account() {
    let mut ledger = Ledger::new();
    ledger.execute(&[meta(aid(1))], Instruction::Initialize);
}

#[test]
#[should_panic(expected = "Config account is already initialized")]
fn initialize_is_write_once() {
    let config = compute_config_pda(PROGRAM);
    let mut ledger = Ledger::new();
    ledger.execute(&[unsigned(config)], Instruction::Initialize);
    ledger.execute(&[unsigned(config)], Instruction::Initialize);
}

#[test]
#[should_panic(expected = "names the shard of")]
fn a_foreign_shard_is_refused() {
    let config = compute_config_pda(PROGRAM);
    let mut ledger = Ledger::new();
    ledger.execute(&[foreign(config)], Instruction::Initialize);
}
