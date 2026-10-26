//! Transparent SACCO ledger program logic — the attestation model.
//!
//! Pure state transitions over shard bytes, no I/O. Value never moves on chain:
//! deposits, loans and expenses are recorded and approved; the chain is the
//! tamper-evident record.

pub mod apply;
pub mod plan;

pub use apply::apply;
pub use plan::plan;
pub use sacco_ledger_core as core;
