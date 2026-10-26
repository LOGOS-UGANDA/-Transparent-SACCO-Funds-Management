#![cfg_attr(not(test), no_main)]

#[cfg(not(test))]
risc0_zkvm::guest::entry!(main);

fn main() {
    nssa_core::program::run_program(sacco_ledger_program::plan, sacco_ledger_program::apply);
}
