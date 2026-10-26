//! Effect application: decode the shard, run the transition, hand back the new bytes.

use sacco_ledger_core::effect::Effect;
use sacco_ledger_core::transition;

/// Applies one effect to one shard's pre-data. `None` keeps the shard unchanged.
#[must_use]
pub fn apply(effect: Effect, pre: &[u8]) -> Option<Vec<u8>> {
    match effect {
        Effect::Config(effect) => transition::config::apply(effect, pre),
    }
}
