//! In-memory sequencer: runs `plan`, then applies each effect in order.
#![allow(
    dead_code,
    reason = "each integration test binary uses a subset of the harness"
)]

use std::collections::BTreeMap;

use borsh::{BorshDeserialize, BorshSerialize};
use nssa_core::account::AccountId;
use nssa_core::program::{AccountMeta, PlanInput};
use sacco_ledger_core::codec::{decode, encode};
use sacco_ledger_core::effect::Effect;
use sacco_ledger_core::{
    Deposit, Expense, Instruction, Loan, Member, PeriodSummary, ProgramConfig, Sacco, Totals,
};
use sacco_ledger_program::{apply, plan};

/// The program's own account id in every test.
pub const PROGRAM: AccountId = AccountId::new([0xAA; 32]);
/// A different program, used to prove shard ownership is enforced.
pub const OTHER_PROGRAM: AccountId = AccountId::new([0xBB; 32]);

/// An account id derived from a single byte, e.g. `aid(6)` is the member.
#[must_use]
pub fn aid(n: u8) -> AccountId {
    AccountId::new([n; 32])
}

/// A writable, authorized, program-owned account handle.
#[must_use]
pub fn meta(id: AccountId) -> AccountMeta {
    AccountMeta::new(id, true, PROGRAM)
}

/// A read-only, authorized account handle.
#[must_use]
pub fn readonly(id: AccountId) -> AccountMeta {
    AccountMeta::new(id, false, PROGRAM)
}

/// An account handle whose shard belongs to a different program.
#[must_use]
pub fn foreign(id: AccountId) -> AccountMeta {
    AccountMeta::new(id, true, OTHER_PROGRAM)
}

/// An account handle whose signer flag is false.
#[must_use]
pub fn unsigned(id: AccountId) -> AccountMeta {
    AccountMeta::new(id, false, PROGRAM)
}

/// Registers and verifies a SACCO owned by `aid(2)`.
pub fn verified_sacco(ledger: &mut Ledger) -> AccountId {
    ledger.verified_sacco(aid(1), aid(2));
    aid(1)
}

/// Adds `member` to the fixture SACCO.
pub fn add_verified_member(ledger: &mut Ledger, member: AccountId) {
    ledger.add_member(aid(1), aid(2), member);
}

/// A verified SACCO with approvers `aid(2)` and `aid(3)` and threshold 2.
pub fn two_approver_sacco(ledger: &mut Ledger) -> AccountId {
    verified_sacco(ledger);
    add_verified_member(ledger, aid(6));
    ledger.set_approvers(aid(1), &[aid(2), aid(3)], 2);
    aid(1)
}

/// Applies for a 200_000 loan for member `aid(6)`, threshold 2.
pub fn applied_loan(ledger: &mut Ledger, loan: AccountId) {
    ledger.execute(
        &[meta(aid(1)), meta(aid(6)), meta(loan), meta(aid(6))],
        Instruction::ApplyLoan {
            amount: 200_000,
            purpose: "school fees".into(),
            required_approvals: 2,
        },
    );
}

/// A fully disbursed 200_000 loan.
pub fn disbursed_loan(ledger: &mut Ledger, loan: AccountId) {
    applied_loan(ledger, loan);
    for approver in [aid(2), aid(3)] {
        ledger.execute(
            &[meta(aid(1)), meta(loan), meta(approver)],
            Instruction::ApproveLoan,
        );
    }
    ledger.execute(
        &[meta(aid(1)), meta(aid(6)), meta(loan), meta(aid(2))],
        Instruction::DisburseLoan {
            evidence_cid: "bafy-disbursement".into(),
            principal: 200_000,
        },
    );
}

/// The SACCO's live totals, as a client would read them.
pub fn totals(ledger: &Ledger) -> Totals {
    Totals::of(&ledger.sacco(aid(1)))
}

/// A verified SACCO with approvers `aid(2)`, `aid(3)`, `aid(4)` and threshold 2.
///
/// Three approvers are required for expenses: the proposer cannot approve their
/// own proposal, so a threshold of 2 needs a third signer.
pub fn three_approver_sacco(ledger: &mut Ledger) -> AccountId {
    verified_sacco(ledger);
    add_verified_member(ledger, aid(6));
    ledger.set_approvers(aid(1), &[aid(2), aid(3), aid(4)], 2);
    aid(1)
}

/// Proposes a 120_000 expense (proposer `aid(2)`) on the three-approver SACCO.
pub fn proposed_expense(ledger: &mut Ledger, expense: AccountId) {
    three_approver_sacco(ledger);
    ledger.execute(
        &[meta(aid(1)), meta(expense), meta(aid(2))],
        Instruction::ProposeExpense {
            amount: 120_000,
            purpose: "office chairs".into(),
            evidence_cid: "bafy-purpose".into(),
            required_approvals: 2,
        },
    );
}

/// Shard state after executing instructions.
#[derive(Default)]
pub struct Ledger {
    shards: BTreeMap<AccountId, Vec<u8>>,
}

impl Ledger {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Overwrites a shard directly, for seeding governance configuration.
    pub fn put<T: BorshSerialize>(&mut self, id: AccountId, value: &T) {
        self.shards.insert(id, encode(value));
    }

    /// Sets the SACCO's approver list and threshold.
    pub fn set_approvers(&mut self, sacco: AccountId, approvers: &[AccountId], threshold: u8) {
        let mut state = self.sacco(sacco);
        state.approvers = approvers.to_vec();
        state.required_approvals = threshold;
        self.put(sacco, &state);
    }

    /// Runs `plan`, then applies every emitted effect, exactly as the sequencer does.
    pub fn execute(&mut self, accounts: &[AccountMeta], instruction: Instruction) {
        let input = PlanInput {
            self_account_id: PROGRAM,
            caller_account_id: None,
            accounts: accounts.to_vec(),
            instruction_data: encode(&instruction),
        };
        let planned = plan(&input, instruction);
        for shard_effect in planned.output().effects.clone() {
            let selector = shard_effect.selector;
            assert_eq!(
                selector.program_account_id, PROGRAM,
                "effect names a shard this program does not own"
            );
            assert!(
                accounts.iter().any(|a| a.account_id == selector.account_id
                    && a.program_account_id == selector.program_account_id),
                "effect names an account the plan did not receive"
            );
            let effect: Effect =
                decode(&shard_effect.data).expect("planned effect must Borsh-decode");
            let pre = self
                .shards
                .get(&selector.account_id)
                .cloned()
                .unwrap_or_default();
            if let Some(post) = apply(effect, &pre) {
                self.shards.insert(selector.account_id, post);
            }
        }
    }

    /// A verified SACCO with `admin` as its sole approver.
    pub fn verified_sacco(&mut self, sacco: AccountId, admin: AccountId) {
        self.execute(
            &[meta(sacco), meta(admin)],
            Instruction::RegisterSacco {
                name: "Kampala Traders SACCO".into(),
                registration_number: "UG-SACCO-001".into(),
                treasury: aid(9),
            },
        );
        for approver in [aid(250), aid(251)] {
            self.execute(
                &[meta(sacco), meta(admin)],
                Instruction::AddApprover { approver },
            );
        }
        self.execute(
            &[meta(sacco), meta(admin)],
            Instruction::SetRequiredApprovals {
                required_approvals: 2,
            },
        );
        self.execute(&[meta(sacco), meta(admin)], Instruction::VerifySacco);
    }

    /// A member of `sacco`, created by `admin`.
    pub fn add_member(&mut self, sacco: AccountId, admin: AccountId, member: AccountId) {
        self.execute(
            &[meta(sacco), meta(member), meta(admin)],
            Instruction::AddMember { identity: member },
        );
    }

    #[must_use]
    pub fn raw(&self, id: AccountId) -> Vec<u8> {
        self.shards.get(&id).cloned().unwrap_or_default()
    }

    #[must_use]
    pub fn decode<T: BorshDeserialize>(&self, id: AccountId) -> Option<T> {
        self.shards.get(&id).map(|bytes| {
            decode(bytes).unwrap_or_else(|_| panic!("account {id} does not hold the expected data"))
        })
    }

    #[must_use]
    pub fn config(&self, id: AccountId) -> ProgramConfig {
        self.decode(id).expect("config account")
    }

    #[must_use]
    pub fn sacco(&self, id: AccountId) -> Sacco {
        self.decode(id).expect("sacco account")
    }

    #[must_use]
    pub fn member(&self, id: AccountId) -> Member {
        self.decode(id).expect("member account")
    }

    #[must_use]
    pub fn deposit(&self, id: AccountId) -> Deposit {
        self.decode(id).expect("deposit account")
    }

    #[must_use]
    pub fn loan(&self, id: AccountId) -> Loan {
        self.decode(id).expect("loan account")
    }

    #[must_use]
    pub fn expense(&self, id: AccountId) -> Expense {
        self.decode(id).expect("expense account")
    }

    #[must_use]
    pub fn summary(&self, id: AccountId) -> PeriodSummary {
        self.decode(id).expect("summary account")
    }
}
