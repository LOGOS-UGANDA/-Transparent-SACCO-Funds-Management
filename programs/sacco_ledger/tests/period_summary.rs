//! Period snapshots: a public, validated copy of the SACCO totals.

mod support;

use sacco_ledger_core::{DepositType, Instruction};
use support::{add_verified_member, aid, meta, totals, verified_sacco, Ledger};

fn funded(ledger: &mut Ledger) {
    verified_sacco(ledger);
    add_verified_member(ledger, aid(6));
    ledger.execute(
        &[meta(aid(1)), meta(aid(6)), meta(aid(7)), meta(aid(2))],
        Instruction::RecordDeposit {
            member: aid(6),
            amount: 75_000,
            deposit_type: DepositType::Savings,
            reference: "MPZA-1".into(),
        },
    );
}

#[test]
fn post_period_summary_snapshots_totals() {
    let mut ledger = Ledger::new();
    funded(&mut ledger);
    let snapshot = totals(&ledger);
    ledger.execute(
        &[meta(aid(1)), meta(aid(10)), meta(aid(2))],
        Instruction::PostPeriodSummary {
            label: "2026-11".into(),
            totals: snapshot,
        },
    );
    let summary = ledger.summary(aid(10));
    assert_eq!(summary.sacco_id, aid(1));
    assert_eq!(summary.label, "2026-11");
    assert_eq!(summary.deposits_savings, 75_000);
    assert_eq!(summary.deposits_share_capital, 0);
}

#[test]
#[should_panic(expected = "Not a SACCO approver")]
fn post_period_summary_rejects_non_approver() {
    let mut ledger = Ledger::new();
    funded(&mut ledger);
    let snapshot = totals(&ledger);
    ledger.execute(
        &[meta(aid(1)), meta(aid(10)), meta(aid(5))],
        Instruction::PostPeriodSummary {
            label: "2026-11".into(),
            totals: snapshot,
        },
    );
}

#[test]
#[should_panic(expected = "Totals do not match the SACCO's live totals")]
fn post_period_summary_rejects_a_stale_snapshot() {
    let mut ledger = Ledger::new();
    funded(&mut ledger);
    let stale = totals(&ledger);
    ledger.execute(
        &[meta(aid(1)), meta(aid(6)), meta(aid(11)), meta(aid(2))],
        Instruction::RecordDeposit {
            member: aid(6),
            amount: 1_000,
            deposit_type: DepositType::Savings,
            reference: "MPZA-2".into(),
        },
    );
    ledger.execute(
        &[meta(aid(1)), meta(aid(10)), meta(aid(2))],
        Instruction::PostPeriodSummary {
            label: "2026-11".into(),
            totals: stale,
        },
    );
}

#[test]
#[should_panic(expected = "Period label must use YYYY-MM format")]
fn post_period_summary_rejects_invalid_label() {
    let mut ledger = Ledger::new();
    funded(&mut ledger);
    let snapshot = totals(&ledger);
    ledger.execute(
        &[meta(aid(1)), meta(aid(10)), meta(aid(2))],
        Instruction::PostPeriodSummary {
            label: "2026-13".into(),
            totals: snapshot,
        },
    );
}

#[test]
fn period_labels_can_be_posted_out_of_order() {
    let mut ledger = Ledger::new();
    funded(&mut ledger);
    let snapshot = totals(&ledger);
    ledger.execute(
        &[meta(aid(1)), meta(aid(10)), meta(aid(2))],
        Instruction::PostPeriodSummary {
            label: "2026-11".into(),
            totals: snapshot,
        },
    );
    ledger.execute(
        &[meta(aid(1)), meta(aid(11)), meta(aid(2))],
        Instruction::PostPeriodSummary {
            label: "2026-10".into(),
            totals: snapshot,
        },
    );
    assert_eq!(ledger.summary(aid(11)).label, "2026-10");
}

#[test]
#[should_panic(expected = "Period label was already posted")]
fn a_period_label_cannot_be_reposted_after_a_newer_label() {
    let mut ledger = Ledger::new();
    funded(&mut ledger);
    let snapshot = totals(&ledger);
    for (summary, label) in [
        (aid(10), "2026-11"),
        (aid(11), "2026-12"),
        (aid(12), "2026-11"),
    ] {
        ledger.execute(
            &[meta(aid(1)), meta(summary), meta(aid(2))],
            Instruction::PostPeriodSummary {
                label: label.into(),
                totals: snapshot,
            },
        );
    }
}

#[test]
#[should_panic(expected = "Period label was already posted")]
fn a_period_label_cannot_be_posted_twice() {
    let mut ledger = Ledger::new();
    funded(&mut ledger);
    let snapshot = totals(&ledger);
    for summary in [aid(10), aid(11)] {
        ledger.execute(
            &[meta(aid(1)), meta(summary), meta(aid(2))],
            Instruction::PostPeriodSummary {
                label: "2026-11".into(),
                totals: snapshot,
            },
        );
    }
}

#[test]
#[should_panic(expected = "Period summary limit reached")]
fn summary_history_has_a_fixed_limit() {
    let mut ledger = Ledger::new();
    funded(&mut ledger);
    let mut sacco = ledger.sacco(aid(1));
    sacco.period_labels = vec!["2000-01".into(); sacco_ledger_core::MAX_PERIOD_SUMMARIES];
    ledger.put(aid(1), &sacco);
    let snapshot = totals(&ledger);
    ledger.execute(
        &[meta(aid(1)), meta(aid(200)), meta(aid(2))],
        Instruction::PostPeriodSummary {
            label: "2026-11".into(),
            totals: snapshot,
        },
    );
}
