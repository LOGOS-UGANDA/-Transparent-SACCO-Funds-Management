//! Foundation: config initialisation and the harness itself.

mod support;

use sacco_ledger_core::Instruction;
use support::{aid, foreign, meta, Ledger};

#[test]
fn initialize_creates_config() {
    let config = aid(1);
    let mut ledger = Ledger::new();
    ledger.execute(&[meta(config)], Instruction::Initialize);
    assert_eq!(ledger.config(config).version, 1);
}

#[test]
#[should_panic(expected = "Config account is already initialized")]
fn initialize_is_write_once() {
    let config = aid(1);
    let mut ledger = Ledger::new();
    ledger.execute(&[meta(config)], Instruction::Initialize);
    ledger.execute(&[meta(config)], Instruction::Initialize);
}

#[test]
#[should_panic(expected = "names the shard of")]
fn a_foreign_shard_is_refused() {
    let config = aid(1);
    let mut ledger = Ledger::new();
    ledger.execute(&[foreign(config)], Instruction::Initialize);
}
