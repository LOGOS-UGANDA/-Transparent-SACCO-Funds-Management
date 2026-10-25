//! Domain type contract tests for sacco_ledger_core.

use borsh::BorshDeserialize;
use nssa_core::account::AccountId;
use sacco_ledger_core::{
    Deposit, DepositType, Expense, ExpenseStatus, Loan, LoanStatus, Member, PeriodSummary, Sacco,
};

fn aid(n: u8) -> AccountId {
    AccountId::new([n; 32])
}

fn sacco_fixture() -> Sacco {
    Sacco {
        name: "Kampala Traders SACCO".into(),
        registration_number: "UG-SACCO-001".into(),
        admin: aid(1),
        treasury: aid(9),
        is_verified: false,
        is_active: true,
        total_deposits_savings: 1_000_000,
        total_deposits_share_capital: 500_000,
        total_loans_disbursed: 300_000,
        total_loans_repaid: 120_000,
        total_expenses: 45_000,
        required_approvals: 2,
        approvers: vec![aid(2), aid(3)],
        supervisory_committee: vec![aid(4)],
        member_ids: vec![aid(5)],
    }
}

#[test]
fn deposit_type_roundtrips() {
    for t in [DepositType::Savings, DepositType::ShareCapital] {
        let bytes = borsh::to_vec(&t).expect("serialize");
        let back = DepositType::try_from_slice(&bytes).expect("deserialize");
        assert_eq!(t, back);
    }
}

#[test]
fn loan_status_discriminants() {
    let cases = [
        (LoanStatus::Applied, 0_u8),
        (LoanStatus::Approved, 1),
        (LoanStatus::Disbursed, 2),
        (LoanStatus::Repaid, 3),
        (LoanStatus::Rejected, 4),
    ];
    for (status, expected) in cases {
        let bytes = borsh::to_vec(&status).expect("serialize");
        assert_eq!(bytes[0], expected, "borsh discriminant for {expected}");
        let back = LoanStatus::try_from_slice(&bytes).expect("deserialize");
        assert_eq!(status, back);
    }
}

#[test]
fn expense_status_discriminants() {
    let cases = [
        (ExpenseStatus::Proposed, 0_u8),
        (ExpenseStatus::Approved, 1),
        (ExpenseStatus::Rejected, 2),
        (ExpenseStatus::Executed, 3),
    ];
    for (status, expected) in cases {
        let bytes = borsh::to_vec(&status).expect("serialize");
        assert_eq!(bytes[0], expected, "borsh discriminant for {expected}");
        let back = ExpenseStatus::try_from_slice(&bytes).expect("deserialize");
        assert_eq!(status, back);
    }
}

#[test]
fn sacco_account_borsh_roundtrip() {
    let original = sacco_fixture();
    let bytes = borsh::to_vec(&original).expect("serialize");
    let back = Sacco::try_from_slice(&bytes).expect("deserialize");
    assert_eq!(original, back);
}

#[test]
fn deposit_account_serde_roundtrip() {
    let original = Deposit {
        sacco_id: aid(1),
        member_id: aid(2),
        amount: 250_000,
        deposit_type: DepositType::ShareCapital,
        reference: "MPZA-99182".into(),
        recorded_by: aid(3),
        confirmed_by_member: false,
    };
    let json = serde_json::to_string(&original).expect("serde serialize");
    let back: Deposit = serde_json::from_str(&json).expect("serde deserialize");
    assert_eq!(original, back);
}

#[test]
fn member_account_borsh_roundtrip() {
    let original = Member {
        sacco_id: aid(1),
        identity: aid(2),
        savings_balance: 80_000,
        share_capital_balance: 20_000,
        loan_ids: vec![aid(3)],
    };
    let bytes = borsh::to_vec(&original).expect("serialize");
    let back = Member::try_from_slice(&bytes).expect("deserialize");
    assert_eq!(original, back);
}

#[test]
fn loan_account_borsh_roundtrip() {
    let original = Loan {
        sacco_id: aid(1),
        member_id: aid(2),
        principal: 150_000,
        purpose: "School fees".into(),
        status: LoanStatus::Disbursed,
        approvals: vec![aid(3), aid(4)],
        rejectors: vec![],
        evidence_cid: Some("bafy-test-cid".into()),
        repaid_amount: 50_000,
    };
    let bytes = borsh::to_vec(&original).expect("serialize");
    let back = Loan::try_from_slice(&bytes).expect("deserialize");
    assert_eq!(original, back);
}

#[test]
fn expense_account_borsh_roundtrip() {
    let original = Expense {
        sacco_id: aid(1),
        amount: 75_000,
        purpose: "Stationery for the AGM".into(),
        evidence_cid: "bafy-expense-cid".into(),
        requested_by: aid(2),
        status: ExpenseStatus::Proposed,
        approvers: vec![],
        rejectors: vec![],
        executed_evidence: None,
    };
    let bytes = borsh::to_vec(&original).expect("serialize");
    let back = Expense::try_from_slice(&bytes).expect("deserialize");
    assert_eq!(original, back);
}

#[test]
fn period_summary_borsh_roundtrip() {
    let original = PeriodSummary {
        sacco_id: aid(1),
        label: "2026-11".into(),
        deposits_savings: 400_000,
        deposits_share_capital: 100_000,
        loans_disbursed: 250_000,
        loans_repaid: 90_000,
        expenses: 30_000,
    };
    let bytes = borsh::to_vec(&original).expect("serialize");
    let back = PeriodSummary::try_from_slice(&bytes).expect("deserialize");
    assert_eq!(original, back);
}
