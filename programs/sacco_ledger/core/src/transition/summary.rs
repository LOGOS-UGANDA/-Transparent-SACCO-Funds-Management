//! The period-summary shard: a validated copy of the SACCO's public totals.

use crate::codec::encode;
use crate::effect::SummaryEffect;
use crate::PeriodSummary;

/// Applies a summary effect to the summary shard.
#[must_use]
pub fn apply(effect: SummaryEffect, pre: &[u8]) -> Option<Vec<u8>> {
    match effect {
        SummaryEffect::Init {
            sacco,
            label,
            totals,
        } => {
            assert!(pre.is_empty(), "Summary account must be uninitialized");
            assert!(!label.is_empty(), "Period label must not be empty");
            Some(encode(&PeriodSummary::new(sacco, label, totals)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AccountId, PeriodSummary, Totals};

    fn effect() -> SummaryEffect {
        SummaryEffect::Init {
            sacco: AccountId::new([1; 32]),
            label: "2026-11".into(),
            totals: Totals::default(),
        }
    }

    #[test]
    fn a_summary_copies_the_totals() {
        use crate::codec::decode;

        let post = apply(effect(), &[]).expect("writes");
        let summary: PeriodSummary = decode(&post).expect("summary data");
        assert_eq!(summary.label, "2026-11");
    }

    #[test]
    #[should_panic(expected = "Summary account must be uninitialized")]
    fn a_summary_is_write_once() {
        let first = apply(effect(), &[]).expect("writes");
        let _ = apply(effect(), &first);
    }
}
