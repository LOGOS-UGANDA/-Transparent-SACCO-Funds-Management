# `sacco_ledger` on `lee_core` v0.3 — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port all 18 `sacco_ledger` instructions from the SPEL v0.2.5 ABI to the `lee_core` v0.3.1 `plan`/`apply` sharded-account ABI, keeping every business rule, the IDL artifact, and getting a working local deploy.

**Architecture:** `plan` asserts account shape, ids and signer flags, then emits one typed effect per mutated shard. Each effect's `apply` decodes that shard's bytes, runs a pure transition, and returns the new bytes. Values that live on a different shard travel as instruction arguments validated by the owning shard, or are pinned into the proposal at creation.

**Tech Stack:** Rust 1.98.1, `lee_core` v0.3.1 (git tag), `borsh` 1.5.7, RISC Zero 3.0.5, `logos-scaffold` 0.4.0, GitHub Actions.

**Spec:** `docs/superpowers/specs/2026-10-26-lee-core-v03-port-design.md`

## Global Constraints

- Branch `issue-3-lee-core-v03`, already cut from `issue-3-sacco-ledger-pr`. Never commit on `main` or on `issue-3-sacco-ledger-pr`.
- `programs/rust-toolchain.toml` = `1.98.1`; every command in `programs/` runs under it.
- `nssa_core` is the local alias for `lee_core`, git tag `v0.3.1`, `features = ["host"]` on the workspace dependency.
- `borsh = "1.5.7"`, `risc0-zkvm`/`risc0-build` stay `=3.0.5`. `spel-framework`, `spel-framework-macros`, `token_core` are removed.
- Workspace lints deny `unwrap_used`, `todo`, `unimplemented`, `unsafe_code`. Use `expect`/`panics` with a message that states the rule.
- Panic messages are the rule text; tests assert on them. Do not reword a message without updating its test.
- `ShardData` hard cap is 100 KiB. No instruction may grow a shard beyond it.
- Every commit uses the `Changes:` / `Verification:` body standard and references `#3`.
- `RISC0_DEV_MODE=1` for all test runs.

## Review Focus

1. An account marked `is_authorized: false` must fail even when its data checks would pass — Task 2 pins this.
2. An account whose shard belongs to another program must fail, never silently write — Task 2 pins this.
3. Re-running a creation instruction against an existing account must fail, not overwrite — Task 3 pins this with `write_once`.
4. A member confirming someone else's deposit must fail — Task 3 pins this.
5. A stale `totals` snapshot for `PostPeriodSummary` must fail rather than be recorded — Task 6 pins this.

---

## Task 1: Foundation — v0.3 toolchain, shard codec, `Initialize`, harness

**Files:**
- Modify: `programs/rust-toolchain.toml`
- Modify: `programs/Cargo.toml`
- Modify: `programs/sacco_ledger/core/Cargo.toml`
- Modify: `programs/sacco_ledger/Cargo.toml`
- Modify: `programs/sacco_ledger/methods/guest/Cargo.toml`
- Modify: `programs/sacco_ledger/methods/guest/src/bin/sacco_ledger.rs`
- Modify: `programs/sacco_ledger/core/Cargo.toml`
- Modify: `programs/sacco_ledger/core/src/lib.rs`
- Create: `programs/sacco_ledger/core/src/codec.rs`
- Create: `programs/sacco_ledger/core/src/effect.rs`
- Create: `programs/sacco_ledger/core/src/transition/mod.rs`
- Create: `programs/sacco_ledger/core/src/transition/config.rs`
- Delete: `programs/sacco_ledger/core/src/custody.rs`
- Delete: `programs/sacco_ledger/src/custody/` (11 files + `tests.rs`)
- Delete: `programs/sacco_ledger/src/{add_approver,add_member,add_supervisor,apply_loan,approve_expense,approve_loan,confirm_deposit,disburse_loan,execute_expense,initialize,post_period_summary,propose_expense,record_deposit,record_repayment,reject_expense,reject_loan,register_sacco,verify_sacco}.rs`
- Delete: `programs/sacco_ledger/tests/{deposits,expenses,loans,period_summary,setup}.rs`
- Delete: `programs/sacco_ledger/core/tests/{instruction_enum,types}.rs` (rewritten in Task 2)
- Create: `programs/sacco_ledger/src/plan.rs`
- Create: `programs/sacco_ledger/src/apply.rs`
- Create: `programs/sacco_ledger/tests/support/mod.rs`
- Create: `programs/sacco_ledger/tests/foundation.rs`

**Interfaces:**
- Produces: `sacco_ledger_core::codec::{encode, decode}`, `sacco_ledger_core::effect::{Effect, ConfigEffect}`, `sacco_ledger_core::transition::config::apply`, `sacco_ledger_program::{plan, apply}`, and the test harness `tests/support/mod.rs` (`Ledger`, `meta`, `readonly`, `foreign`, `unsigned`, `aid`).
- Later tasks extend `Effect`, `plan` and `apply`; they never change these signatures.

- [ ] **Step 1: Bump the toolchain and dependencies**

`programs/rust-toolchain.toml`:

```toml
[toolchain]
channel = "1.98.1"
profile = "default"
```

`programs/Cargo.toml` — replace the workspace dependencies block with:

```toml
[workspace]
members = [
    "sacco_ledger/core",
    "sacco_ledger",
    "sacco_ledger/methods",
]
exclude = [
    "sacco_ledger/methods/guest",
]
resolver = "2"

[workspace.package]
edition = "2021"
version = "0.1.0"

[workspace.dependencies]
nssa_core = { git = "https://github.com/logos-blockchain/logos-execution-zone.git", tag = "v0.3.1", package = "lee_core" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
borsh = { version = "1.5.7", features = ["derive"] }
risc0-zkvm = { version = "=3.0.5" }
sacco_ledger_core = { path = "sacco_ledger/core" }
sacco_ledger_program = { path = "sacco_ledger" }

[workspace.lints.rust]
rust_2018_idioms = { level = "deny", priority = -1 }
unsafe_code = "deny"

[workspace.lints.clippy]
correctness = { level = "deny", priority = -1 }
suspicious = { level = "deny", priority = -1 }
perf = { level = "deny", priority = -1 }
style = { level = "warn", priority = -1 }
dbg_macro = "deny"
todo = "deny"
unimplemented = "deny"
unwrap_used = "deny"
```

Keep the lints block exactly as it is today; only the dependency lines change.

Do not enable `lee_core`'s `host` feature: the guest depends on `sacco_ledger_program`, and Cargo unifies
features, so a host-only feature would be compiled into the zkVM guest. Nothing here uses randomness.

`programs/sacco_ledger/core/Cargo.toml`:

```toml
[package]
name = "sacco_ledger_core"
version = "0.1.0"
edition = "2021"

[lints]
workspace = true

[dependencies]
nssa_core = { workspace = true }
borsh = { workspace = true }
serde = { workspace = true }
risc0-zkvm = { workspace = true, default-features = false }

[dev-dependencies]
serde_json = "1.0"
```

`programs/sacco_ledger/Cargo.toml`:

```toml
[package]
name = "sacco_ledger_program"
version = "0.1.0"
edition = "2021"

[lints]
workspace = true

[dependencies]
nssa_core = { workspace = true }
sacco_ledger_core = { workspace = true }

[dev-dependencies]
borsh = { workspace = true }
```

`programs/sacco_ledger/methods/guest/Cargo.toml`:

```toml
[package]
name = "sacco_ledger-guest"
version = "0.1.0"
edition = "2021"

[workspace]

[profile.release]
debug = 0
strip = "symbols"

[lints.rust]
rust_2018_idioms = { level = "deny", priority = -1 }
unsafe_code = "deny"

[[bin]]
name = "sacco_ledger"
path = "src/bin/sacco_ledger.rs"

[dependencies]
nssa_core = { git = "https://github.com/logos-blockchain/logos-execution-zone.git", tag = "v0.3.1", package = "lee_core" }
sacco_ledger_program = { path = "../..", package = "sacco_ledger_program" }
risc0-zkvm = { version = "=3.0.5", default-features = false }
```

- [ ] **Step 2: Install the toolchain and confirm the bump breaks the build**

Run: `rustup toolchain install 1.98.1`
Run: `cd programs && cargo check 2>&1 | head -40`
Expected: FAIL — `spel_framework` / `AccountWithMetadata` unresolved. This is the failing state the port must fix.

- [ ] **Step 3: Delete the custody module and the old handlers**

```bash
cd /home/zasourcecode/Projects/transparent-sacco
git rm -r programs/sacco_ledger/src/custody programs/sacco_ledger/core/src/custody.rs
git rm programs/sacco_ledger/src/{add_approver,add_member,add_supervisor,apply_loan,approve_expense,approve_loan,confirm_deposit,disburse_loan,execute_expense,initialize,post_period_summary,propose_expense,record_deposit,record_repayment,reject_expense,reject_loan,register_sacco,verify_sacco}.rs
git rm programs/sacco_ledger/tests/{deposits,expenses,loans,period_summary,setup}.rs
git rm programs/sacco_ledger/core/tests/{instruction_enum,types}.rs
```

- [ ] **Step 4: Write the failing codec test**

`programs/sacco_ledger/core/src/codec.rs`:

```rust
//! Borsh helpers for shard-encoded account data.
//!
//! A shard holds the Borsh encoding of exactly one account type. `decode` returns
//! `Result` so the caller names the rule that the data must satisfy.

use borsh::{BorshDeserialize, BorshSerialize};

/// Encodes an account value for storage in its shard.
#[must_use]
pub fn encode<T: BorshSerialize>(value: &T) -> Vec<u8> {
    borsh::to_vec(value).expect("borsh serialization is infallible")
}

/// Decodes a shard's bytes into an account value.
///
/// # Errors
/// Returns the Borsh error when the shard does not hold a valid `T`.
pub fn decode<T: BorshDeserialize>(data: &[u8]) -> Result<T, std::io::Error> {
    borsh::from_slice(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Sacco;
    use crate::AccountId;

    #[test]
    fn a_sacco_round_trips_through_its_shard() {
        let sacco = Sacco::register(
            "Kampala Traders SACCO".into(),
            "UG-SACCO-001".into(),
            AccountId::new([9; 32]),
            AccountId::new([2; 32]),
        );
        let decoded: Sacco = decode(&encode(&sacco)).expect("round trip");
        assert_eq!(decoded, sacco);
    }

    #[test]
    fn garbage_shard_data_is_an_error_not_a_panic() {
        assert!(decode::<Sacco>(&[0xFF; 3]).is_err());
    }
}
```

Run: `cd programs && cargo test -p sacco_ledger_core codec 2>&1 | tail -10`
Expected: FAIL — `crate::AccountId` is not re-exported and `Sacco::register` does not exist yet.

- [ ] **Step 5: Rebuild `core/src/lib.rs` on plain Borsh**

Delete the `spel_framework_macros` import, the `pub mod custody;` line, every `#[account_type]` attribute, the private `use nssa_core::account::AccountId;`, and the `impl_data_codec!` macro plus its seven invocations. Add at the top:

```rust
pub mod codec;
pub mod effect;
pub mod transition;

pub use nssa_core::account::AccountId;
```

Keep every domain struct and `Instruction` variant. Add the constructor the codec test needs:

```rust
impl Sacco {
    /// A newly registered SACCO: one approver (the admin) and threshold 1.
    #[must_use]
    pub fn register(
        name: String,
        registration_number: String,
        treasury: AccountId,
        admin: AccountId,
    ) -> Self {
        Self {
            name,
            registration_number,
            admin,
            treasury,
            is_verified: false,
            is_active: true,
            total_deposits_savings: 0,
            total_deposits_share_capital: 0,
            total_loans_disbursed: 0,
            total_loans_repaid: 0,
            total_expenses: 0,
            required_approvals: 1,
            approvers: vec![admin],
            supervisory_committee: vec![],
            member_ids: vec![],
        }
    }
}
```

The structs keep `Serialize`/`Deserialize` (wallet JSON) and `BorshSerialize`/`BorshDeserialize` (shards).

Run: `cd programs && cargo test -p sacco_ledger_core codec 2>&1 | tail -10`
Expected: PASS (2 tests).

- [ ] **Step 6: Write the failing `Initialize` transition test**

`programs/sacco_ledger/core/src/transition/config.rs`:

```rust
//! The singleton registry config shard.

use crate::codec::{decode, encode};
use crate::effect::ConfigEffect;
use crate::{ProgramConfig, ProgramConfigVersion};

/// Applies a config effect to the config shard.
#[must_use]
pub fn apply(effect: ConfigEffect, pre: &[u8]) -> Option<Vec<u8>> {
    match effect {
        ConfigEffect::Init { version } => {
            assert!(pre.is_empty(), "Config account is already initialized");
            Some(encode(&ProgramConfig { version }))
        }
    }
}
```

`programs/sacco_ledger/core/src/transition/mod.rs`:

```rust
//! Pure per-shard transitions. Each module applies one effect kind to one shard.

pub mod config;
```

`programs/sacco_ledger/core/src/effect.rs`:

```rust
//! Effects: one per mutated shard, Borsh-encoded inside `ShardEffect`.

use borsh::{BorshDeserialize, BorshSerialize};

use crate::ProgramConfigVersion;

/// Every effect a `sacco_ledger` instruction can emit.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum Effect {
    Config(ConfigEffect),
}

/// Effects on the singleton config shard.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum ConfigEffect {
    Init { version: ProgramConfigVersion },
}
```

Add `ProgramConfigVersion` to `core/src/lib.rs`:

```rust
/// Registry layout version written by `Initialize`.
pub type ProgramConfigVersion = u8;
```

Add to `config.rs` a test:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_writes_the_version() {
        let post = apply(ConfigEffect::Init { version: 1 }, &[]).expect("writes");
        let decoded: ProgramConfig = decode(&post).expect("config data");
        assert_eq!(decoded.version, 1);
    }

    #[test]
    #[should_panic(expected = "Config account is already initialized")]
    fn init_is_write_once() {
        let pre = encode(&ProgramConfig { version: 1 });
        apply(ConfigEffect::Init { version: 1 }, &pre);
    }
}
```

Run: `cd programs && cargo test -p sacco_ledger_core transition 2>&1 | tail -10`
Expected: PASS.

- [ ] **Step 7: Write the failing plan test**

`programs/sacco_ledger/tests/support/mod.rs`:

```rust
//! In-memory sequencer: runs `plan`, then applies each effect in order.
#![allow(dead_code, reason = "each integration test binary uses a subset of the harness")]

use std::collections::BTreeMap;

use borsh::BorshDeserialize;
use nssa_core::account::AccountId;
use nssa_core::program::{AccountMeta, PlanInput};
use sacco_ledger_core::codec::{decode, encode};
use sacco_ledger_core::{
    Deposit, Effect, Expense, Instruction, Loan, Member, PeriodSummary, ProgramConfig, Sacco,
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
            let pre = self.shards.get(&selector.account_id).cloned().unwrap_or_default();
            if let Some(post) = apply(effect, &pre) {
                self.shards.insert(selector.account_id, post);
            }
        }
    }

    #[must_use]
    pub fn raw(&self, id: AccountId) -> Vec<u8> {
        self.shards.get(&id).cloned().unwrap_or_default()
    }

    #[must_use]
    pub fn decode<T: BorshDeserialize>(&self, id: AccountId) -> Option<T> {
        self.shards.get(&id).map(|bytes| {
            decode(bytes).unwrap_or_else(|_| panic!("account {} does not hold the expected data", id))
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
```

`programs/sacco_ledger/tests/foundation.rs`:

```rust
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
```

Run: `cd programs && cargo test -p sacco_ledger_program --test foundation 2>&1 | tail -20`
Expected: FAIL — `plan`/`apply` do not exist in `sacco_ledger_program` yet.

- [ ] **Step 8: Implement `plan`, `apply` and the library surface**

`programs/sacco_ledger/src/plan.rs`:

```rust
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
#[must_use]
pub fn plan(input: &PlanInput, instruction: Instruction) -> Plan {
    let mut plan = Plan::new(input);
    match instruction {
        Instruction::Initialize => {
            let [config] = <&[_; 1]>::try_from(input.accounts.as_slice())
                .expect("Initialize requires exactly 1 account");
            assert!(config.is_authorized, "Config account must be authorized");
            plan.effect(config, &Effect::Config(ConfigEffect::Init { version: CONFIG_VERSION }));
        }
        other => panic!("instruction not ported yet: {other:?}"),
    }
    plan
}
```

`programs/sacco_ledger/src/apply.rs`:

```rust
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
```

`programs/sacco_ledger/src/lib.rs`:

```rust
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
```

`programs/sacco_ledger/methods/guest/src/bin/sacco_ledger.rs`:

```rust
#![cfg_attr(not(test), no_main)]

#[cfg(not(test))]
risc0_zkvm::guest::entry!(main);

fn main() {
    nssa_core::program::run_program(sacco_ledger_program::plan, sacco_ledger_program::apply);
}
```

Run: `cd programs && RISC0_DEV_MODE=1 cargo test -p sacco_ledger_program --test foundation 2>&1 | tail -20`
Expected: PASS (3 tests) — config initialisation, write-once, and refusal of a foreign shard.

- [ ] **Step 9: Confirm the rest of the workspace still compiles**

Run: `cd programs && RISC0_DEV_MODE=1 cargo test -p sacco_ledger_core 2>&1 | tail -10`
Expected: PASS.

- [ ] **Step 10: Commit**

```bash
git add programs
git commit -F - <<'EOF'
refactor: move sacco_ledger onto the lee_core v0.3 ABI (#3)

Changes:
- Toolchain 1.98.1 and lee_core v0.3.1; drop spel-framework, token_core and
  the account_type/Data codec, replaced by Borsh shard helpers.
- Delete the dormant custody module and the 18 SPEL handlers; the program is
  now plan/apply with one effect per mutated shard.
- Initialize ported through the new Config effect, guest entry rewritten to
  run_program, and an in-memory harness that mirrors the sequencer's plan
  then per-effect apply.

Verification:
- `cargo test -p sacco_ledger_core`: pass
- `RISC0_DEV_MODE=1 cargo test -p sacco_ledger_program --test foundation`: Initialize tests pass
EOF
```

---

## Task 2: SACCO governance — register, verify, approvers, supervisors, members

**Files:**
- Modify: `programs/sacco_ledger/core/src/effect.rs`
- Create: `programs/sacco_ledger/core/src/transition/sacco.rs`
- Create: `programs/sacco_ledger/core/src/transition/member.rs`
- Modify: `programs/sacco_ledger/core/src/transition/mod.rs`
- Modify: `programs/sacco_ledger/core/src/lib.rs` (add `is_approver`, `Totals` later in Task 6)
- Modify: `programs/sacco_ledger/src/plan.rs`
- Modify: `programs/sacco_ledger/src/apply.rs`
- Create: `programs/sacco_ledger/tests/setup.rs`
- Create: `programs/sacco_ledger/core/tests/instruction_enum.rs`

**Interfaces:**
- Consumes: Task 1's `Effect`, `plan`, `apply`, harness.
- Produces: `SaccoEffect::{Register, Verify, AddApprover, AddSupervisor, AddMember}`, `MemberEffect::Init`, `transition::sacco::apply`, `transition::member::apply`, `sacco_ledger_core::is_approver(&Sacco, &AccountId) -> bool`.

- [ ] **Step 1: Write the failing governance tests**

`programs/sacco_ledger/tests/setup.rs`:

```rust
//! SACCO registration, verification and role composition.

mod support;

use nssa_core::account::AccountId;
use sacco_ledger_core::Instruction;
use support::{aid, meta, unsigned, Ledger};

fn registered(ledger: &mut Ledger) -> (AccountId, AccountId) {
    let (sacco, admin) = (aid(1), aid(2));
    ledger.execute(
        &[meta(sacco), meta(admin)],
        Instruction::RegisterSacco {
            name: "Kampala Traders SACCO".into(),
            registration_number: "UG-SACCO-001".into(),
            treasury: aid(9),
        },
    );
    (sacco, admin)
}

#[test]
fn register_sacco_records_admin_and_defaults() {
    let (sacco, admin) = (aid(1), aid(2));
    let mut ledger = Ledger::new();
    ledger.execute(
        &[meta(sacco), meta(admin)],
        Instruction::RegisterSacco {
            name: "Kampala Traders SACCO".into(),
            registration_number: "UG-SACCO-001".into(),
            treasury: aid(9),
        },
    );
    let registered = ledger.sacco(sacco);
    assert_eq!(registered.admin, admin);
    assert_eq!(registered.treasury, aid(9));
    assert_eq!(registered.required_approvals, 1);
    assert_eq!(registered.approvers, vec![admin]);
    assert!(registered.is_active);
    assert!(!registered.is_verified);
    assert_eq!(registered.total_deposits_savings, 0);
}

#[test]
#[should_panic(expected = "Sacco name must not be empty")]
fn register_sacco_rejects_empty_name() {
    let mut ledger = Ledger::new();
    ledger.execute(
        &[meta(aid(1)), meta(aid(2))],
        Instruction::RegisterSacco {
            name: String::new(),
            registration_number: "UG-SACCO-001".into(),
            treasury: aid(9),
        },
    );
}

#[test]
#[should_panic(expected = "Admin must authorize")]
fn an_unsigned_signer_is_refused() {
    let mut ledger = Ledger::new();
    ledger.execute(
        &[meta(aid(1)), unsigned(aid(2))],
        Instruction::RegisterSacco {
            name: "Kampala Traders SACCO".into(),
            registration_number: "UG-SACCO-001".into(),
            treasury: aid(9),
        },
    );
}

#[test]
#[should_panic(expected = "names the shard of")]
fn a_foreign_shard_is_refused() {
    let mut ledger = Ledger::new();
    ledger.execute(
        &[support::foreign(aid(1)), meta(aid(2))],
        Instruction::RegisterSacco {
            name: "Kampala Traders SACCO".into(),
            registration_number: "UG-SACCO-001".into(),
            treasury: aid(9),
        },
    );
}

#[test]
fn verify_sacco_by_admin() {
    let mut ledger = Ledger::new();
    let (sacco, admin) = registered(&mut ledger);
    ledger.execute(&[meta(sacco), meta(admin)], Instruction::VerifySacco);
    assert!(ledger.sacco(sacco).is_verified);
}

#[test]
#[should_panic(expected = "Only the SACCO admin can verify in v1")]
fn verify_sacco_requires_registrar() {
    let mut ledger = Ledger::new();
    let (sacco, _admin) = registered(&mut ledger);
    ledger.execute(&[meta(sacco), meta(aid(3))], Instruction::VerifySacco);
}

#[test]
fn add_approver_admin_only() {
    let mut ledger = Ledger::new();
    let (sacco, admin) = registered(&mut ledger);
    ledger.execute(
        &[meta(sacco), meta(admin)],
        Instruction::AddApprover { approver: aid(3) },
    );
    assert!(ledger.sacco(sacco).approvers.contains(&aid(3)));
}

#[test]
#[should_panic(expected = "Approver already registered")]
fn add_approver_rejects_duplicate() {
    let mut ledger = Ledger::new();
    let (sacco, admin) = registered(&mut ledger);
    ledger.execute(
        &[meta(sacco), meta(admin)],
        Instruction::AddApprover { approver: admin },
    );
}

#[test]
fn add_supervisor_admin_only() {
    let mut ledger = Ledger::new();
    let (sacco, admin) = registered(&mut ledger);
    ledger.execute(
        &[meta(sacco), meta(admin)],
        Instruction::AddSupervisor { supervisor: aid(4) },
    );
    assert!(ledger.sacco(sacco).supervisory_committee.contains(&aid(4)));
}

#[test]
#[should_panic(expected = "Approvers cannot also sit on the Supervisory Committee")]
fn add_supervisor_rejects_current_approver() {
    let mut ledger = Ledger::new();
    let (sacco, admin) = registered(&mut ledger);
    ledger.execute(
        &[meta(sacco), meta(admin)],
        Instruction::AddSupervisor { supervisor: admin },
    );
}

#[test]
fn add_member_creates_member_account() {
    let mut ledger = Ledger::new();
    let (sacco, admin) = registered(&mut ledger);
    ledger.execute(
        &[meta(sacco), meta(aid(6)), meta(admin)],
        Instruction::AddMember { identity: aid(6) },
    );
    let member = ledger.member(aid(6));
    assert_eq!(member.sacco_id, sacco);
    assert_eq!(member.identity, aid(6));
    assert_eq!(member.savings_balance, 0);
    assert!(ledger.sacco(sacco).member_ids.contains(&aid(6)));
}

#[test]
#[should_panic(expected = "Member identity already registered")]
fn add_member_rejects_duplicate_identity() {
    let mut ledger = Ledger::new();
    let (sacco, admin) = registered(&mut ledger);
    ledger.execute(
        &[meta(sacco), meta(aid(6)), meta(admin)],
        Instruction::AddMember { identity: aid(6) },
    );
    ledger.execute(
        &[meta(sacco), meta(aid(6)), meta(admin)],
        Instruction::AddMember { identity: aid(6) },
    );
}
```

Add to `tests/support/mod.rs`:

```rust
impl Ledger {
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
        self.execute(&[meta(sacco), meta(admin)], Instruction::VerifySacco);
    }

    /// A member of `sacco`, created by `admin`.
    pub fn add_member(&mut self, sacco: AccountId, admin: AccountId, member: AccountId) {
        self.execute(
            &[meta(sacco), meta(member), meta(admin)],
            Instruction::AddMember { identity: member },
        );
    }
}
```

Run: `cd programs && cargo test -p sacco_ledger_program --test setup 2>&1 | tail -20`
Expected: FAIL — `SaccoEffect` has only `Init`.

- [ ] **Step 2: Add the governance effects**

In `core/src/effect.rs`:

```rust
/// Effects on a SACCO shard.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum SaccoEffect {
    Register { name: String, registration_number: String, treasury: AccountId, admin: AccountId },
    Verify { registrar: AccountId },
    AddApprover { admin: AccountId, approver: AccountId },
    AddSupervisor { admin: AccountId, supervisor: AccountId },
    AddMember { admin: AccountId, identity: AccountId },
}

/// Effects on a member shard.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum MemberEffect {
    Init { sacco: AccountId, identity: AccountId },
}
```

and extend `Effect`:

```rust
pub enum Effect {
    Config(ConfigEffect),
    Sacco(SaccoEffect),
    Member(MemberEffect),
}
```

- [ ] **Step 3: Write the failing transition tests, then the transitions**

`core/src/transition/sacco.rs`:

```rust
//! The SACCO shard: registration, verification and role composition.

use crate::codec::{decode, encode};
use crate::effect::SaccoEffect;
use crate::{is_approver, Sacco};

#[must_use]
pub fn apply(effect: SaccoEffect, pre: &[u8]) -> Option<Vec<u8>> {
    match effect {
        SaccoEffect::Register { name, registration_number, treasury, admin } => {
            assert!(pre.is_empty(), "Sacco account must be uninitialized");
            assert!(!name.is_empty(), "Sacco name must not be empty");
            assert!(!registration_number.is_empty(), "Registration number must not be empty");
            Some(encode(&Sacco::register(name, registration_number, treasury, admin)))
        }
        SaccoEffect::Verify { registrar } => {
            let mut sacco = decode::<Sacco>(pre).expect("Sacco shard must hold valid Sacco data");
            assert!(sacco.is_active, "Sacco must be active");
            assert_eq!(sacco.admin, registrar, "Only the SACCO admin can verify in v1");
            sacco.is_verified = true;
            Some(encode(&sacco))
        }
        SaccoEffect::AddApprover { admin, approver } => {
            let mut sacco = decode::<Sacco>(pre).expect("Sacco shard must hold valid Sacco data");
            assert_eq!(sacco.admin, admin, "Only the SACCO admin can add approvers");
            assert!(!sacco.approvers.contains(&approver), "Approver already registered");
            assert!(
                !sacco.supervisory_committee.contains(&approver),
                "Supervisory Committee members cannot also be approvers"
            );
            sacco.approvers.push(approver);
            Some(encode(&sacco))
        }
        SaccoEffect::AddSupervisor { admin, supervisor } => {
            let mut sacco = decode::<Sacco>(pre).expect("Sacco shard must hold valid Sacco data");
            assert_eq!(sacco.admin, admin, "Only the SACCO admin can add supervisors");
            assert!(
                !sacco.supervisory_committee.contains(&supervisor),
                "Supervisor already registered"
            );
            assert!(
                !sacco.approvers.contains(&supervisor),
                "Approvers cannot also sit on the Supervisory Committee"
            );
            sacco.supervisory_committee.push(supervisor);
            Some(encode(&sacco))
        }
        SaccoEffect::AddMember { admin, identity } => {
            let mut sacco = decode::<Sacco>(pre).expect("Sacco shard must hold valid Sacco data");
            assert_eq!(sacco.admin, admin, "Only the SACCO admin can add members");
            assert!(!sacco.member_ids.contains(&identity), "Member identity already registered");
            sacco.member_ids.push(identity);
            Some(encode(&sacco))
        }
        _ => panic!("sacco effect not ported yet"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registered() -> Sacco {
        Sacco::register(
            "Kampala Traders SACCO".into(),
            "UG-SACCO-001".into(),
            crate::AccountId::new([9; 32]),
            crate::AccountId::new([2; 32]),
        )
    }

    #[test]
    fn register_is_write_once() {
        let pre = encode(&registered());
        let effect = SaccoEffect::Register {
            name: "Other".into(),
            registration_number: "UG-2".into(),
            treasury: crate::AccountId::new([8; 32]),
            admin: crate::AccountId::new([3; 32]),
        };
        let result = std::panic::catch_unwind(|| apply(effect, &pre));
        assert!(result.is_err());
    }

    #[test]
    fn only_the_admin_adds_approvers() {
        let pre = encode(&registered());
        let result = std::panic::catch_unwind(|| {
            apply(
                SaccoEffect::AddApprover {
                    admin: crate::AccountId::new([7; 32]),
                    approver: crate::AccountId::new([3; 32]),
                },
                &pre,
            )
        });
        assert!(result.is_err());
    }
}
```

`core/src/transition/member.rs`:

```rust
//! The member shard: identity and balances.

use crate::codec::{decode, encode};
use crate::effect::MemberEffect;
use crate::{AccountId, Member};

#[must_use]
pub fn apply(effect: MemberEffect, pre: &[u8]) -> Option<Vec<u8>> {
    match effect {
        MemberEffect::Init { sacco, identity } => {
            assert!(pre.is_empty(), "Member account must be uninitialized");
            Some(encode(&Member {
                sacco_id: sacco,
                identity,
                savings_balance: 0,
                share_capital_balance: 0,
                loan_ids: vec![],
            }))
        }
        _ => panic!("member effect not ported yet"),
    }
}

/// Decodes a member shard and asserts it belongs to `sacco`.
pub fn decode_member(pre: &[u8], sacco: AccountId) -> Member {
    let member: Member = decode(pre).expect("Member shard must hold valid Member data");
    assert_eq!(member.sacco_id, sacco, "Member belongs to a different SACCO");
    member
}
```

Add `is_approver` to `core/src/lib.rs`:

```rust
/// True when `id` is one of the SACCO's multi-sig approvers.
#[must_use]
pub fn is_approver(sacco: &Sacco, id: &AccountId) -> bool {
    sacco.approvers.contains(id)
}
```

Register both modules in `core/src/transition/mod.rs`:

```rust
pub mod config;
pub mod member;
pub mod sacco;
```

Run: `cd programs && cargo test -p sacco_ledger_core --lib 2>&1 | tail -10`
Expected: PASS.

- [ ] **Step 4: Plan and apply the governance instructions**

In `src/plan.rs`, replace the catch-all arm with:

```rust
Instruction::RegisterSacco { name, registration_number, treasury } => {
    let [sacco, admin] = <&[_; 2]>::try_from(input.accounts.as_slice())
        .expect("RegisterSacco requires sacco and admin");
    assert!(sacco.is_authorized, "Sacco account must be authorized");
    let admin = signer(admin, "Admin must authorize registration");
    plan.effect(
        sacco,
        &Effect::Sacco(SaccoEffect::Register { name, registration_number, treasury, admin }),
    );
}
Instruction::VerifySacco => {
    let [sacco, registrar] = <&[_; 2]>::try_from(input.accounts.as_slice())
        .expect("VerifySacco requires sacco and registrar");
    let registrar = signer(registrar, "Registrar must authorize verification");
    plan.effect(sacco, &Effect::Sacco(SaccoEffect::Verify { registrar }));
}
Instruction::AddApprover { approver } => {
    let [sacco, admin] = <&[_; 2]>::try_from(input.accounts.as_slice())
        .expect("AddApprover requires sacco and admin");
    let admin = signer(admin, "Admin must authorize");
    plan.effect(sacco, &Effect::Sacco(SaccoEffect::AddApprover { admin, approver }));
}
Instruction::AddSupervisor { supervisor } => {
    let [sacco, admin] = <&[_; 2]>::try_from(input.accounts.as_slice())
        .expect("AddSupervisor requires sacco and admin");
    let admin = signer(admin, "Admin must authorize");
    plan.effect(sacco, &Effect::Sacco(SaccoEffect::AddSupervisor { admin, supervisor }));
}
Instruction::AddMember { identity } => {
    let [sacco, member, admin] = <&[_; 3]>::try_from(input.accounts.as_slice())
        .expect("AddMember requires sacco, member and admin");
    assert!(member.is_authorized, "Member account must be authorized");
    let admin = signer(admin, "Admin must authorize");
    plan.effect(sacco, &Effect::Sacco(SaccoEffect::AddMember { admin, identity }));
    plan.effect(
        member,
        &Effect::Member(MemberEffect::Init { sacco: sacco.account_id, identity }),
    );
}
```

In `src/apply.rs`:

```rust
Effect::Sacco(effect) => transition::sacco::apply(effect, pre),
Effect::Member(effect) => transition::member::apply(effect, pre),
```

Imports: `use sacco_ledger_core::effect::{ConfigEffect, Effect, MemberEffect, SaccoEffect};`

Run: `cd programs && RISC0_DEV_MODE=1 cargo test -p sacco_ledger_program --test setup 2>&1 | tail -25`
Expected: PASS (12 tests).

- [ ] **Step 5: Rewrite the instruction-enum round-trip test**

`programs/sacco_ledger/core/tests/instruction_enum.rs`:

```rust
//! Every instruction survives the Borsh round trip used by the wallet path.

use sacco_ledger_core::{AccountId, DepositType, Instruction, Totals};

fn round_trip(instruction: Instruction) {
    let bytes = borsh::to_vec(&instruction).expect("encode");
    let decoded: Instruction = borsh::from_slice(&bytes).expect("decode");
    assert_eq!(decoded, instruction);
}

#[test]
fn every_variant_round_trips() {
    let id = AccountId::new([7; 32]);
    for instruction in [
        Instruction::Initialize,
        Instruction::RegisterSacco {
            name: "Kampala Traders SACCO".into(),
            registration_number: "UG-SACCO-001".into(),
            treasury: id,
        },
        Instruction::VerifySacco,
        Instruction::AddApprover { approver: id },
        Instruction::AddSupervisor { supervisor: id },
        Instruction::AddMember { identity: id },
        Instruction::RecordDeposit {
            member: id,
            amount: 1,
            deposit_type: DepositType::Savings,
            reference: "MPZA-1".into(),
        },
        Instruction::ConfirmDeposit,
        Instruction::ApplyLoan { amount: 1, purpose: "school fees".into(), required_approvals: 2 },
        Instruction::ApproveLoan,
        Instruction::RejectLoan,
        Instruction::DisburseLoan { evidence_cid: "cid".into(), principal: 1 },
        Instruction::RecordRepayment { loan: id, amount: 1, reference: "MPZA-2".into() },
        Instruction::ProposeExpense {
            amount: 1,
            purpose: "desk".into(),
            evidence_cid: "cid".into(),
            required_approvals: 2,
        },
        Instruction::ApproveExpense,
        Instruction::RejectExpense,
        Instruction::ExecuteExpense { executed_evidence: "receipt".into(), amount: 1 },
        Instruction::PostPeriodSummary { label: "2026-11".into(), totals: Totals::default() },
    ] {
        round_trip(instruction);
    }
}
```

This test fails until Task 6 adds `Totals` and the new arguments; it is finished then. Commit only once it passes.

- [ ] **Step 6: Commit**

```bash
git add programs
git commit -F - <<'EOF'
feat: port SACCO governance instructions to plan/apply (#3)

Changes:
- SaccoEffect Register/Verify/AddApprover/AddSupervisor/AddMember and
  MemberEffect Init, with the admin, duplicate and role-separation rules
  moved into the transitions that can see the data.
- plan asserts account shape and signer flags and emits one effect per
  shard; tests/setup.rs replaces the SPEL-era setup tests on the harness.
- is_approver moved into core so transitions and plans share one rule.

Verification:
- `RISC0_DEV_MODE=1 cargo test -p sacco_ledger_program --test setup`: 12 pass
- `cargo test -p sacco_ledger_core --lib`: pass
EOF
```

---

## Task 3: Deposits — record and member confirmation

**Files:**
- Modify: `programs/sacco_ledger/core/src/effect.rs`
- Modify: `programs/sacco_ledger/core/src/transition/sacco.rs`
- Modify: `programs/sacco_ledger/core/src/transition/member.rs`
- Create: `programs/sacco_ledger/core/src/transition/deposit.rs`
- Modify: `programs/sacco_ledger/core/src/transition/mod.rs`
- Modify: `programs/sacco_ledger/src/plan.rs`
- Modify: `programs/sacco_ledger/src/apply.rs`
- Create: `programs/sacco_ledger/tests/deposits.rs`

**Interfaces:**
- Consumes: Task 2's `verified_sacco`, `add_member`, `meta`, `Ledger`.
- Produces: `SaccoEffect::RecordDeposit`, `MemberEffect::RecordDeposit`, `DepositEffect::{Init, Confirm}`, `transition::deposit::apply`.

- [ ] **Step 1: Write the failing deposit tests**

`programs/sacco_ledger/tests/deposits.rs`:

```rust
//! Deposits: typed totals, verification gating, member confirmation.

mod support;

use sacco_ledger_core::{DepositType, Instruction};
use support::{add_verified_member, aid, meta, verified_sacco, Ledger};

fn record(ledger: &mut Ledger, deposit: u8) {
    ledger.execute(
        &[meta(aid(1)), meta(aid(6)), meta(aid(deposit)), meta(aid(2))],
        Instruction::RecordDeposit {
            member: aid(6),
            amount: 100_000,
            deposit_type: DepositType::Savings,
            reference: "MPZA-1001".into(),
        },
    );
}

#[test]
fn record_deposit_updates_member_and_sacco_totals() {
    let mut ledger = Ledger::new();
    verified_sacco(&mut ledger);
    add_verified_member(&mut ledger, aid(6));
    record(&mut ledger, 7);

    let sacco = ledger.sacco(aid(1));
    assert_eq!(sacco.total_deposits_savings, 100_000);
    assert_eq!(sacco.total_deposits_share_capital, 0);

    let member = ledger.member(aid(6));
    assert_eq!(member.savings_balance, 100_000);
    assert_eq!(member.share_capital_balance, 0);

    let deposit = ledger.deposit(aid(7));
    assert_eq!(deposit.amount, 100_000);
    assert_eq!(deposit.deposit_type, DepositType::Savings);
    assert_eq!(deposit.reference, "MPZA-1001");
    assert_eq!(deposit.recorded_by, aid(2));
    assert!(!deposit.confirmed_by_member);
}

#[test]
fn record_deposit_splits_totals_by_type() {
    let mut ledger = Ledger::new();
    verified_sacco(&mut ledger);
    add_verified_member(&mut ledger, aid(6));
    ledger.execute(
        &[meta(aid(1)), meta(aid(6)), meta(aid(7)), meta(aid(2))],
        Instruction::RecordDeposit {
            member: aid(6),
            amount: 40_000,
            deposit_type: DepositType::ShareCapital,
            reference: "MPZA-1002".into(),
        },
    );
    assert_eq!(ledger.sacco(aid(1)).total_deposits_savings, 0);
    assert_eq!(ledger.sacco(aid(1)).total_deposits_share_capital, 40_000);
    assert_eq!(ledger.member(aid(6)).savings_balance, 0);
    assert_eq!(ledger.member(aid(6)).share_capital_balance, 40_000);
}

#[test]
#[should_panic(expected = "Sacco is not verified")]
fn record_deposit_rejected_for_unverified_sacco() {
    let mut ledger = Ledger::new();
    let (sacco, admin) = (aid(1), aid(2));
    ledger.execute(
        &[meta(sacco), meta(admin)],
        Instruction::RegisterSacco {
            name: "Kampala Traders SACCO".into(),
            registration_number: "UG-SACCO-001".into(),
            treasury: aid(9),
        },
    );
    ledger.execute(
        &[meta(sacco), meta(aid(6)), meta(admin)],
        Instruction::AddMember { identity: aid(6) },
    );
    record(&mut ledger, 7);
}

#[test]
#[should_panic(expected = "Only SACCO approvers can record deposits")]
fn record_deposit_requires_approver() {
    let mut ledger = Ledger::new();
    verified_sacco(&mut ledger);
    add_verified_member(&mut ledger, aid(6));
    ledger.execute(
        &[meta(aid(1)), meta(aid(6)), meta(aid(7)), meta(aid(3))],
        Instruction::RecordDeposit {
            member: aid(6),
            amount: 1_000,
            deposit_type: DepositType::Savings,
            reference: "MPZA-3".into(),
        },
    );
}

#[test]
#[should_panic(expected = "Deposit reference must not be empty")]
fn record_deposit_rejects_empty_reference() {
    let mut ledger = Ledger::new();
    verified_sacco(&mut ledger);
    add_verified_member(&mut ledger, aid(6));
    ledger.execute(
        &[meta(aid(1)), meta(aid(6)), meta(aid(7)), meta(aid(2))],
        Instruction::RecordDeposit {
            member: aid(6),
            amount: 1_000,
            deposit_type: DepositType::Savings,
            reference: String::new(),
        },
    );
}

#[test]
#[should_panic(expected = "Deposit account must be uninitialized")]
fn record_deposit_cannot_overwrite_a_deposit() {
    let mut ledger = Ledger::new();
    verified_sacco(&mut ledger);
    add_verified_member(&mut ledger, aid(6));
    record(&mut ledger, 7);
    record(&mut ledger, 7);
}

#[test]
fn confirm_deposit_flips_member_confirmation() {
    let mut ledger = Ledger::new();
    verified_sacco(&mut ledger);
    add_verified_member(&mut ledger, aid(6));
    record(&mut ledger, 7);
    ledger.execute(&[meta(aid(7)), meta(aid(6))], Instruction::ConfirmDeposit);
    assert!(ledger.deposit(aid(7)).confirmed_by_member);
}

#[test]
#[should_panic(expected = "Only the deposit's member can confirm")]
fn confirm_deposit_member_only() {
    let mut ledger = Ledger::new();
    verified_sacco(&mut ledger);
    add_verified_member(&mut ledger, aid(6));
    record(&mut ledger, 7);
    ledger.execute(&[meta(aid(7)), meta(aid(3))], Instruction::ConfirmDeposit);
}

#[test]
#[should_panic(expected = "Deposit already confirmed")]
fn confirm_deposit_rejects_double_confirm() {
    let mut ledger = Ledger::new();
    verified_sacco(&mut ledger);
    add_verified_member(&mut ledger, aid(6));
    record(&mut ledger, 7);
    ledger.execute(&[meta(aid(7)), meta(aid(6))], Instruction::ConfirmDeposit);
    ledger.execute(&[meta(aid(7)), meta(aid(6))], Instruction::ConfirmDeposit);
}
```

Add to `tests/support/mod.rs`:

```rust
/// Registers and verifies a SACCO owned by `aid(2)`.
pub fn verified_sacco(ledger: &mut Ledger) -> AccountId {
    ledger.verified_sacco(aid(1), aid(2));
    aid(1)
}

/// Adds `member` to the fixture SACCO.
pub fn add_verified_member(ledger: &mut Ledger, member: AccountId) {
    ledger.add_member(aid(1), aid(2), member);
}
```

Run: `cd programs && cargo test -p sacco_ledger_program --test deposits 2>&1 | tail -20`
Expected: FAIL — `RecordDeposit`/`ConfirmDeposit` are unhandled plan arms.

- [ ] **Step 2: Add the deposit effects**

`core/src/effect.rs`:

```rust
/// Effects on a deposit shard.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum DepositEffect {
    Init(Deposit),
    Confirm { member: AccountId },
}
```

In `SaccoEffect` add:

```rust
RecordDeposit { recorder: AccountId, amount: u128, deposit_type: DepositType },
```

In `MemberEffect` add:

```rust
RecordDeposit { sacco: AccountId, amount: u128, deposit_type: DepositType },
```

and add `Deposit(DepositEffect)` to `Effect`. Import `Deposit`, `DepositType`.

- [ ] **Step 3: Write the failing transition tests, then the transitions**

`core/src/transition/deposit.rs`:

```rust
//! The deposit shard: one attested off-chain payment.

use crate::codec::{decode, encode};
use crate::effect::DepositEffect;
use crate::Deposit;

#[must_use]
pub fn apply(effect: DepositEffect, pre: &[u8]) -> Option<Vec<u8>> {
    match effect {
        DepositEffect::Init(deposit) => {
            assert!(pre.is_empty(), "Deposit account must be uninitialized");
            assert!(!deposit.reference.is_empty(), "Deposit reference must not be empty");
            assert!(deposit.amount > 0, "Deposit amount must be greater than 0");
            Some(encode(&deposit))
        }
        DepositEffect::Confirm { member } => {
            let mut deposit: Deposit =
                decode(pre).expect("Deposit shard must hold valid Deposit data");
            assert_eq!(deposit.member_id, member, "Only the deposit's member can confirm");
            assert!(!deposit.confirmed_by_member, "Deposit already confirmed");
            deposit.confirmed_by_member = true;
            Some(encode(&deposit))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AccountId;

    fn deposit() -> Deposit {
        Deposit {
            sacco_id: AccountId::new([1; 32]),
            member_id: AccountId::new([6; 32]),
            amount: 100,
            deposit_type: crate::DepositType::Savings,
            reference: "MPZA-1".into(),
            recorded_by: AccountId::new([2; 32]),
            confirmed_by_member: false,
        }
    }

    #[test]
    fn only_the_member_confirms() {
        let pre = encode(&deposit());
        let result = std::panic::catch_unwind(|| {
            apply(DepositEffect::Confirm { member: AccountId::new([3; 32]) }, &pre)
        });
        assert!(result.is_err());
    }
}
```

In `core/src/transition/sacco.rs`, replace the `_ => panic!` arm with:

```rust
SaccoEffect::RecordDeposit { recorder, amount, deposit_type } => {
    let mut sacco = decode::<Sacco>(pre).expect("Sacco shard must hold valid Sacco data");
    assert!(sacco.is_verified, "Sacco is not verified");
    assert!(
        is_approver(&sacco, &recorder),
        "Only SACCO approvers can record deposits"
    );
    assert!(amount > 0, "Deposit amount must be greater than 0");
    match deposit_type {
        DepositType::Savings => {
            sacco.total_deposits_savings = sacco
                .total_deposits_savings
                .checked_add(amount)
                .expect("savings total overflow");
        }
        DepositType::ShareCapital => {
            sacco.total_deposits_share_capital = sacco
                .total_deposits_share_capital
                .checked_add(amount)
                .expect("share capital total overflow");
        }
    }
    Some(encode(&sacco))
}
```

In `core/src/transition/member.rs`, replace the `_ => panic!` arm with:

```rust
MemberEffect::RecordDeposit { sacco, amount, deposit_type } => {
    let mut member = decode_member(pre, sacco);
    match deposit_type {
        crate::DepositType::Savings => {
            member.savings_balance = member
                .savings_balance
                .checked_add(amount)
                .expect("savings balance overflow");
        }
        crate::DepositType::ShareCapital => {
            member.share_capital_balance = member
                .share_capital_balance
                .checked_add(amount)
                .expect("share capital balance overflow");
        }
    }
    Some(encode(&member))
}
```

Add `pub mod deposit;` to `core/src/transition/mod.rs`.

- [ ] **Step 4: Plan and apply the deposit instructions**

In `src/plan.rs`:

```rust
Instruction::RecordDeposit { member, amount, deposit_type, reference } => {
    let [sacco, member_account, deposit, recorder] =
        <&[_; 4]>::try_from(input.accounts.as_slice())
            .expect("RecordDeposit requires sacco, member, deposit and recorder");
    assert!(deposit.is_authorized, "Deposit account must be authorized");
    assert_eq!(member_account.account_id, member, "Member account does not match the member id argument");
    let recorder = signer(recorder, "Recorder must authorize");

    plan.effect(
        sacco,
        &Effect::Sacco(SaccoEffect::RecordDeposit { recorder, amount, deposit_type }),
    );
    plan.effect(
        member_account,
        &Effect::Member(MemberEffect::RecordDeposit {
            sacco: sacco.account_id,
            amount,
            deposit_type,
        }),
    );
    plan.effect(
        deposit,
        &Effect::Deposit(DepositEffect::Init(Deposit {
            sacco_id: sacco.account_id,
            member_id: member,
            amount,
            deposit_type,
            reference,
            recorded_by: recorder,
            confirmed_by_member: false,
        })),
    );
}
Instruction::ConfirmDeposit => {
    let [deposit, member] = <&[_; 2]>::try_from(input.accounts.as_slice())
        .expect("ConfirmDeposit requires deposit and member");
    let member = signer(member, "Member must authorize confirmation");
    plan.effect(deposit, &Effect::Deposit(DepositEffect::Confirm { member }));
}
```

In `src/apply.rs`:

```rust
Effect::Deposit(effect) => transition::deposit::apply(effect, pre),
```

Imports add `Deposit`, `DepositEffect`, `DepositType`.

Run: `cd programs && RISC0_DEV_MODE=1 cargo test -p sacco_ledger_program --test deposits 2>&1 | tail -25`
Expected: PASS (9 tests).

- [ ] **Step 5: Commit**

```bash
git add programs
git commit -F - <<'EOF'
feat: port deposit instructions to plan/apply (#3)

Changes:
- RecordDeposit becomes three effects: SACCO totals, member balances and the
  new deposit shard; the verified and approver rules live on the SACCO
  shard, the ownership rule on the member shard, write-once on the deposit.
- ConfirmDeposit is a deposit-local effect asserting the confirming signer is
  the member and that the record is not already confirmed.
- tests/deposits.rs replaces the SPEL-era suite on the harness, including a
  replay test proving a deposit shard cannot be overwritten.

Verification:
- `RISC0_DEV_MODE=1 cargo test -p sacco_ledger_program --test deposits`: 9 pass
- `cargo test -p sacco_ledger_core --lib`: pass
EOF
```

---

## Task 4: Loans — apply, approve, reject, disburse, repay

**Files:**
- Modify: `programs/sacco_ledger/core/src/effect.rs`
- Modify: `programs/sacco_ledger/core/src/transition/sacco.rs`
- Modify: `programs/sacco_ledger/core/src/transition/member.rs`
- Create: `programs/sacco_ledger/core/src/transition/loan.rs`
- Modify: `programs/sacco_ledger/core/src/transition/mod.rs`
- Modify: `programs/sacco_ledger/core/src/lib.rs` (`Loan.required_approvals`)
- Modify: `programs/sacco_ledger/src/plan.rs`
- Modify: `programs/sacco_ledger/src/apply.rs`
- Modify: `programs/sacco_ledger/tests/support/mod.rs` (seeding helper)
- Create: `programs/sacco_ledger/tests/loans.rs`

**Interfaces:**
- Consumes: Task 3's effects and harness.
- Produces: `SaccoEffect::{AuthorizeProposal, AuthorizeApprover, DisburseLoan, RecordRepayment}`, `MemberEffect::AttachLoan`, `LoanEffect::{Init, Approve, Reject, Disburse, Repay}`, `Loan.required_approvals: u8`, `Ledger::set_approvers`.

- [ ] **Step 1: Write the failing loan tests**

Add to `tests/support/mod.rs`:

```rust
use borsh::BorshSerialize;

impl Ledger {
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
        ledger.execute(&[meta(aid(1)), meta(loan), meta(approver)], Instruction::ApproveLoan);
    }
    ledger.execute(
        &[meta(aid(1)), meta(aid(6)), meta(loan), meta(aid(2))],
        Instruction::DisburseLoan { evidence_cid: "bafy-disbursement".into(), principal: 200_000 },
    );
}
```

`programs/sacco_ledger/tests/loans.rs`:

```rust
//! Loan lifecycle: application, multi-sig approval, disbursement, repayment.

mod support;

use sacco_ledger_core::{LoanStatus, Instruction};
use support::{
    aid, applied_loan, disbursed_loan, meta, two_approver_sacco, Ledger,
};

#[test]
fn apply_loan_creates_applied_loan() {
    let mut ledger = Ledger::new();
    two_approver_sacco(&mut ledger);
    applied_loan(&mut ledger, aid(7));
    let loan = ledger.loan(aid(7));
    assert_eq!(loan.sacco_id, aid(1));
    assert_eq!(loan.member_id, aid(6));
    assert_eq!(loan.principal, 200_000);
    assert_eq!(loan.status, LoanStatus::Applied);
    assert_eq!(loan.required_approvals, 2);
    assert!(loan.approvals.is_empty());
    assert!(ledger.member(aid(6)).loan_ids.contains(&aid(7)));
}

#[test]
#[should_panic(expected = "Sacco is not verified")]
fn apply_loan_rejected_for_unverified_sacco() {
    let mut ledger = Ledger::new();
    ledger.execute(
        &[meta(aid(1)), meta(aid(2))],
        Instruction::RegisterSacco {
            name: "Kampala Traders SACCO".into(),
            registration_number: "UG-SACCO-001".into(),
            treasury: aid(9),
        },
    );
    ledger.execute(
        &[meta(aid(1)), meta(aid(6)), meta(aid(2))],
        Instruction::AddMember { identity: aid(6) },
    );
    applied_loan(&mut ledger, aid(7));
}

#[test]
fn approve_loan_reaches_threshold_at_exact_count() {
    let mut ledger = Ledger::new();
    two_approver_sacco(&mut ledger);
    applied_loan(&mut ledger, aid(7));
    ledger.execute(&[meta(aid(1)), meta(aid(7)), meta(aid(2))], Instruction::ApproveLoan);
    assert_eq!(ledger.loan(aid(7)).status, LoanStatus::Applied);
    ledger.execute(&[meta(aid(1)), meta(aid(7)), meta(aid(3))], Instruction::ApproveLoan);
    assert_eq!(ledger.loan(aid(7)).status, LoanStatus::Approved);
    assert_eq!(ledger.loan(aid(7)).approvals, vec![aid(2), aid(3)]);
}

#[test]
#[should_panic(expected = "Already approved by this approver")]
fn approve_loan_duplicate_approver_rejected() {
    let mut ledger = Ledger::new();
    two_approver_sacco(&mut ledger);
    applied_loan(&mut ledger, aid(7));
    ledger.execute(&[meta(aid(1)), meta(aid(7)), meta(aid(2))], Instruction::ApproveLoan);
    ledger.execute(&[meta(aid(1)), meta(aid(7)), meta(aid(2))], Instruction::ApproveLoan);
}

#[test]
#[should_panic(expected = "Not a SACCO approver")]
fn approve_loan_non_approver_rejected() {
    let mut ledger = Ledger::new();
    two_approver_sacco(&mut ledger);
    applied_loan(&mut ledger, aid(7));
    ledger.execute(&[meta(aid(1)), meta(aid(7)), meta(aid(5))], Instruction::ApproveLoan);
}

#[test]
fn reject_loan_approver_only() {
    let mut ledger = Ledger::new();
    two_approver_sacco(&mut ledger);
    applied_loan(&mut ledger, aid(7));
    ledger.execute(&[meta(aid(1)), meta(aid(7)), meta(aid(2))], Instruction::RejectLoan);
    let loan = ledger.loan(aid(7));
    assert_eq!(loan.status, LoanStatus::Rejected);
    assert_eq!(loan.rejectors, vec![aid(2)]);
}

#[test]
fn disburse_loan_sets_evidence_status_and_totals() {
    let mut ledger = Ledger::new();
    two_approver_sacco(&mut ledger);
    disbursed_loan(&mut ledger, aid(7));
    let loan = ledger.loan(aid(7));
    assert_eq!(loan.status, LoanStatus::Disbursed);
    assert_eq!(loan.evidence_cid.as_deref(), Some("bafy-disbursement"));
    assert_eq!(ledger.sacco(aid(1)).total_loans_disbursed, 200_000);
}

#[test]
#[should_panic(expected = "Disbursement evidence must not be empty")]
fn disburse_loan_requires_evidence() {
    let mut ledger = Ledger::new();
    two_approver_sacco(&mut ledger);
    applied_loan(&mut ledger, aid(7));
    for approver in [aid(2), aid(3)] {
        ledger.execute(&[meta(aid(1)), meta(aid(7)), meta(approver)], Instruction::ApproveLoan);
    }
    ledger.execute(
        &[meta(aid(1)), meta(aid(6)), meta(aid(7)), meta(aid(2))],
        Instruction::DisburseLoan { evidence_cid: String::new(), principal: 200_000 },
    );
}

#[test]
fn record_repayment_updates_outstanding_and_totals() {
    let mut ledger = Ledger::new();
    two_approver_sacco(&mut ledger);
    disbursed_loan(&mut ledger, aid(7));
    ledger.execute(
        &[meta(aid(1)), meta(aid(6)), meta(aid(7)), meta(aid(2))],
        Instruction::RecordRepayment {
            loan: aid(7),
            amount: 50_000,
            reference: "MPZA-REPAY-1".into(),
        },
    );
    assert_eq!(ledger.loan(aid(7)).repaid_amount, 50_000);
    assert_eq!(ledger.loan(aid(7)).status, LoanStatus::Disbursed);
    assert_eq!(ledger.sacco(aid(1)).total_loans_repaid, 50_000);
}

#[test]
#[should_panic(expected = "Repayment exceeds outstanding principal")]
fn record_repayment_rejects_overpay() {
    let mut ledger = Ledger::new();
    two_approver_sacco(&mut ledger);
    disbursed_loan(&mut ledger, aid(7));
    ledger.execute(
        &[meta(aid(1)), meta(aid(6)), meta(aid(7)), meta(aid(2))],
        Instruction::RecordRepayment { loan: aid(7), amount: 200_001, reference: "X".into() },
    );
}

#[test]
#[should_panic(expected = "Loan is already fully repaid")]
fn record_repayment_rejects_closed_loan() {
    let mut ledger = Ledger::new();
    two_approver_sacco(&mut ledger);
    disbursed_loan(&mut ledger, aid(7));
    ledger.execute(
        &[meta(aid(1)), meta(aid(6)), meta(aid(7)), meta(aid(2))],
        Instruction::RecordRepayment { loan: aid(7), amount: 200_000, reference: "X".into() },
    );
    ledger.execute(
        &[meta(aid(1)), meta(aid(6)), meta(aid(7)), meta(aid(2))],
        Instruction::RecordRepayment { loan: aid(7), amount: 1, reference: "Y".into() },
    );
}
```

Run: `cd programs && cargo test -p sacco_ledger_program --test loans 2>&1 | tail -20`
Expected: FAIL — loan variants unhandled.

- [ ] **Step 2: Add `required_approvals` to `Loan` and the loan effects**

In `core/src/lib.rs`, add to `Loan`:

```rust
    /// Multi-sig threshold pinned when the loan was applied.
    pub required_approvals: u8,
```

In `core/src/effect.rs`:

```rust
/// Effects on a loan shard.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum LoanEffect {
    Init { sacco: AccountId, member: AccountId, principal: u128, purpose: String, required_approvals: u8 },
    Approve { approver: AccountId },
    Reject { approver: AccountId },
    Disburse { evidence_cid: String, principal: u128 },
    Repay { amount: u128, reference: String },
}
```

Add to `SaccoEffect`:

```rust
AuthorizeProposal { required_approvals: u8 },
AuthorizeApprover { approver: AccountId },
DisburseLoan { approver: AccountId, principal: u128 },
RecordRepayment { approver: AccountId, amount: u128 },
```

Add to `MemberEffect`:

```rust
AttachLoan { sacco: AccountId, loan: AccountId, signer: AccountId },
```

Add `Loan(LoanEffect)` to `Effect`.

- [ ] **Step 3: Write the failing transition tests, then the transitions**

`core/src/transition/loan.rs`:

```rust
//! The loan shard: lifecycle and the repayment cap.

use crate::codec::{decode, encode};
use crate::effect::LoanEffect;
use crate::{AccountId, Loan, LoanStatus};

#[must_use]
pub fn apply(effect: LoanEffect, pre: &[u8]) -> Option<Vec<u8>> {
    match effect {
        LoanEffect::Init { sacco, member, principal, purpose, required_approvals } => {
            assert!(pre.is_empty(), "Loan account must be uninitialized");
            assert!(principal > 0, "Loan amount must be greater than 0");
            assert!(!purpose.is_empty(), "Loan purpose must not be empty");
            Some(encode(&Loan {
                sacco_id: sacco,
                member_id: member,
                principal,
                purpose,
                status: LoanStatus::Applied,
                approvals: vec![],
                rejectors: vec![],
                evidence_cid: None,
                repaid_amount: 0,
                required_approvals,
            }))
        }
        LoanEffect::Approve { approver } => {
            let mut loan: Loan = decode(pre).expect("Loan shard must hold valid Loan data");
            assert_eq!(loan.status, LoanStatus::Applied, "Loan is not in Applied state");
            assert!(!loan.approvals.contains(&approver), "Already approved by this approver");
            assert!(!loan.rejectors.contains(&approver), "Already rejected by this approver");
            loan.approvals.push(approver);
            if loan.approvals.len() >= usize::from(loan.required_approvals) {
                loan.status = LoanStatus::Approved;
            }
            Some(encode(&loan))
        }
        LoanEffect::Reject { approver } => {
            let mut loan: Loan = decode(pre).expect("Loan shard must hold valid Loan data");
            assert_eq!(loan.status, LoanStatus::Applied, "Loan is not in Applied state");
            assert!(!loan.rejectors.contains(&approver), "Already rejected by this approver");
            loan.rejectors.push(approver);
            loan.status = LoanStatus::Rejected;
            Some(encode(&loan))
        }
        LoanEffect::Disburse { evidence_cid, principal } => {
            let mut loan: Loan = decode(pre).expect("Loan shard must hold valid Loan data");
            assert_eq!(
                loan.status,
                LoanStatus::Approved,
                "Loan must be approved before disbursement"
            );
            assert!(!evidence_cid.is_empty(), "Disbursement evidence must not be empty");
            assert_eq!(principal, loan.principal, "Principal does not match the loan");
            loan.status = LoanStatus::Disbursed;
            loan.evidence_cid = Some(evidence_cid);
            Some(encode(&loan))
        }
        LoanEffect::Repay { amount, reference } => {
            let mut loan: Loan = decode(pre).expect("Loan shard must hold valid Loan data");
            assert!(amount > 0, "Repayment must be greater than 0");
            assert!(!reference.is_empty(), "Repayment reference must not be empty");
            assert!(
                loan.status == LoanStatus::Disbursed || loan.status == LoanStatus::Repaid,
                "Loan is not in Disbursed state"
            );
            assert!(
                loan.status != LoanStatus::Repaid || loan.repaid_amount < loan.principal,
                "Loan is already fully repaid"
            );
            let outstanding = loan
                .principal
                .checked_sub(loan.repaid_amount)
                .expect("repaid exceeds principal");
            assert!(amount <= outstanding, "Repayment exceeds outstanding principal");
            loan.repaid_amount = loan
                .repaid_amount
                .checked_add(amount)
                .expect("repaid overflow");
            if loan.repaid_amount == loan.principal {
                loan.status = LoanStatus::Repaid;
            }
            Some(encode(&loan))
        }
    }
}

/// A disbursed loan for tests in this module.
pub fn disbursed(principal: u128) -> Loan {
    Loan {
        sacco_id: AccountId::new([1; 32]),
        member_id: AccountId::new([6; 32]),
        principal,
        purpose: "school fees".into(),
        status: LoanStatus::Disbursed,
        approvals: vec![AccountId::new([2; 32])],
        rejectors: vec![],
        evidence_cid: Some("cid".into()),
        repaid_amount: 0,
        required_approvals: 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cap_is_the_outstanding_principal() {
        let pre = encode(&disbursed(100));
        assert!(apply(LoanEffect::Repay { amount: 100, reference: "ok".into() }, &pre).is_some());
        assert!(std::panic::catch_unwind(|| {
            apply(LoanEffect::Repay { amount: 101, reference: "no".into() }, &pre)
        })
        .is_err());
    }
}
```

In `core/src/transition/sacco.rs`, add arms:

```rust
SaccoEffect::AuthorizeProposal { required_approvals } => {
    let sacco = decode::<Sacco>(pre).expect("Sacco shard must hold valid Sacco data");
    assert!(sacco.is_verified, "Sacco is not verified");
    assert_eq!(
        required_approvals, sacco.required_approvals,
        "Approver threshold does not match the SACCO configuration"
    );
    None
}
SaccoEffect::AuthorizeApprover { approver } => {
    let sacco = decode::<Sacco>(pre).expect("Sacco shard must hold valid Sacco data");
    assert!(sacco.is_verified, "Sacco is not verified");
    assert!(is_approver(&sacco, &approver), "Not a SACCO approver");
    None
}
SaccoEffect::DisburseLoan { approver, principal } => {
    let mut sacco = decode::<Sacco>(pre).expect("Sacco shard must hold valid Sacco data");
    assert!(sacco.is_verified, "Sacco is not verified");
    assert!(is_approver(&sacco, &approver), "Not a SACCO approver");
    sacco.total_loans_disbursed = sacco
        .total_loans_disbursed
        .checked_add(principal)
        .expect("loans disbursed overflow");
    Some(encode(&sacco))
}
SaccoEffect::RecordRepayment { approver, amount } => {
    let mut sacco = decode::<Sacco>(pre).expect("Sacco shard must hold valid Sacco data");
    assert!(sacco.is_verified, "Sacco is not verified");
    assert!(is_approver(&sacco, &approver), "Not a SACCO approver");
    sacco.total_loans_repaid = sacco
        .total_loans_repaid
        .checked_add(amount)
        .expect("loans repaid overflow");
    Some(encode(&sacco))
}
```

Replace the remaining `_ => panic!` arm in `core/src/transition/member.rs` with:

```rust
MemberEffect::AttachLoan { sacco, loan, signer } => {
    let mut member = decode_member(pre, sacco);
    assert_eq!(member.identity, signer, "Only the member can apply for their loan");
    assert!(!member.loan_ids.contains(&loan), "Loan already attached to this member");
    member.loan_ids.push(loan);
    Some(encode(&member))
}
```

Add `pub mod loan;` to `core/src/transition/mod.rs`.

Run: `cd programs && cargo test -p sacco_ledger_core --lib 2>&1 | tail -10`
Expected: PASS.

- [ ] **Step 4: Plan and apply the loan instructions**

In `src/plan.rs`:

```rust
Instruction::ApplyLoan { amount, purpose, required_approvals } => {
    let [sacco, member, loan, member_signer] = <&[_; 4]>::try_from(input.accounts.as_slice())
        .expect("ApplyLoan requires sacco, member, loan and member signer");
    assert!(loan.is_authorized, "Loan account must be authorized");
    let signer = signer(member_signer, "Member must authorize the application");
    plan.effect(
        sacco,
        &Effect::Sacco(SaccoEffect::AuthorizeProposal { required_approvals }),
    );
    plan.effect(
        member,
        &Effect::Member(MemberEffect::AttachLoan {
            sacco: sacco.account_id,
            loan: loan.account_id,
            signer,
        }),
    );
    plan.effect(
        loan,
        &Effect::Loan(LoanEffect::Init {
            sacco: sacco.account_id,
            member: member.account_id,
            principal: amount,
            purpose,
            required_approvals,
        }),
    );
}
Instruction::ApproveLoan => {
    let [sacco, loan, approver] = <&[_; 3]>::try_from(input.accounts.as_slice())
        .expect("ApproveLoan requires sacco, loan and approver");
    let approver = signer(approver, "Approver must authorize");
    plan.effect(sacco, &Effect::Sacco(SaccoEffect::AuthorizeApprover { approver }));
    plan.effect(loan, &Effect::Loan(LoanEffect::Approve { approver }));
}
Instruction::RejectLoan => {
    let [sacco, loan, approver] = <&[_; 3]>::try_from(input.accounts.as_slice())
        .expect("RejectLoan requires sacco, loan and approver");
    let approver = signer(approver, "Approver must authorize");
    plan.effect(sacco, &Effect::Sacco(SaccoEffect::AuthorizeApprover { approver }));
    plan.effect(loan, &Effect::Loan(LoanEffect::Reject { approver }));
}
Instruction::DisburseLoan { evidence_cid, principal } => {
    let [sacco, _member, loan, approver] = <&[_; 4]>::try_from(input.accounts.as_slice())
        .expect("DisburseLoan requires sacco, member, loan and approver");
    let approver = signer(approver, "Approver must authorize");
    plan.effect(
        sacco,
        &Effect::Sacco(SaccoEffect::DisburseLoan { approver, principal }),
    );
    plan.effect(loan, &Effect::Loan(LoanEffect::Disburse { evidence_cid, principal }));
}
Instruction::RecordRepayment { loan, amount, reference } => {
    let [sacco, _member, loan_account, approver] = <&[_; 4]>::try_from(input.accounts.as_slice())
        .expect("RecordRepayment requires sacco, member, loan and approver");
    assert_eq!(
        loan_account.account_id, loan,
        "Loan account does not match the loan id argument"
    );
    let approver = signer(approver, "Approver must authorize");
    plan.effect(
        sacco,
        &Effect::Sacco(SaccoEffect::RecordRepayment { approver, amount }),
    );
    plan.effect(loan_account, &Effect::Loan(LoanEffect::Repay { amount, reference }));
}
```

In `src/apply.rs`: `Effect::Loan(effect) => transition::loan::apply(effect, pre),`

Run: `cd programs && RISC0_DEV_MODE=1 cargo test -p sacco_ledger_program --test loans 2>&1 | tail -25`
Expected: PASS (11 tests).

- [ ] **Step 5: Commit**

```bash
git add programs
git commit -F - <<'EOF'
feat: port loan lifecycle to plan/apply (#3)

Changes:
- Loan carries the approval threshold pinned at application time; the SACCO
  shard validates that threshold (AuthorizeProposal) and approves only its
  own members' actions (AuthorizeApprover), since data cannot cross shards.
- Init/Approve/Reject/Disburse/Repay effects hold the status machine and the
  repayment cap; disbursement and repayment totals update on the SACCO shard
  from the validated principal and amount arguments.
- tests/loans.rs ports the 11 SPEL-era lifecycle tests onto the harness.

Verification:
- `RISC0_DEV_MODE=1 cargo test -p sacco_ledger_program --test loans`: 11 pass
- `cargo test -p sacco_ledger_core --lib`: pass
EOF
```

---

## Task 5: Expenses — propose, approve, reject, execute

**Files:**
- Modify: `programs/sacco_ledger/core/src/effect.rs`
- Modify: `programs/sacco_ledger/core/src/transition/sacco.rs`
- Create: `programs/sacco_ledger/core/src/transition/expense.rs`
- Modify: `programs/sacco_ledger/core/src/transition/mod.rs`
- Modify: `programs/sacco_ledger/core/src/lib.rs` (`Expense.required_approvals`)
- Modify: `programs/sacco_ledger/src/plan.rs`
- Modify: `programs/sacco_ledger/src/apply.rs`
- Modify: `programs/sacco_ledger/tests/support/mod.rs`
- Create: `programs/sacco_ledger/tests/expenses.rs`

**Interfaces:**
- Consumes: Task 4's two-approver fixture.
- Produces: `SaccoEffect::{ProposeExpense, ExecuteExpense}`, `ExpenseEffect::{Init, Approve, Reject, Execute}`, `Expense.required_approvals: u8`, `transition::expense::apply`, fixture `proposed_expense`.

- [ ] **Step 1: Write the failing expense tests**

Add to `tests/support/mod.rs`:

```rust
/// Proposes a 120_000 expense on the two-approver SACCO.
pub fn proposed_expense(ledger: &mut Ledger, expense: AccountId) {
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
```

`programs/sacco_ledger/tests/expenses.rs`:

```rust
//! Expenses: proposal, multi-sig approval, execution evidence.

mod support;

use sacco_ledger_core::{ExpenseStatus, Instruction};
use support::{aid, meta, proposed_expense, two_approver_sacco, Ledger};

#[test]
fn propose_expense_creates_proposed_expense() {
    let mut ledger = Ledger::new();
    two_approver_sacco(&mut ledger);
    proposed_expense(&mut ledger, aid(8));
    let expense = ledger.expense(aid(8));
    assert_eq!(expense.sacco_id, aid(1));
    assert_eq!(expense.amount, 120_000);
    assert_eq!(expense.requested_by, aid(2));
    assert_eq!(expense.status, ExpenseStatus::Proposed);
    assert_eq!(expense.required_approvals, 2);
}

#[test]
#[should_panic(expected = "Sacco is not verified")]
fn propose_expense_rejected_for_unverified_sacco() {
    let mut ledger = Ledger::new();
    ledger.execute(
        &[meta(aid(1)), meta(aid(2))],
        Instruction::RegisterSacco {
            name: "Kampala Traders SACCO".into(),
            registration_number: "UG-SACCO-001".into(),
            treasury: aid(9),
        },
    );
    ledger.execute(
        &[meta(aid(1)), meta(aid(8)), meta(aid(2))],
        Instruction::ProposeExpense {
            amount: 1_000,
            purpose: "desk".into(),
            evidence_cid: "cid".into(),
            required_approvals: 1,
        },
    );
}

#[test]
#[should_panic(expected = "Not a SACCO approver")]
fn propose_expense_requires_approver() {
    let mut ledger = Ledger::new();
    two_approver_sacco(&mut ledger);
    ledger.execute(
        &[meta(aid(1)), meta(aid(8)), meta(aid(5))],
        Instruction::ProposeExpense {
            amount: 1_000,
            purpose: "desk".into(),
            evidence_cid: "cid".into(),
            required_approvals: 2,
        },
    );
}

#[test]
#[should_panic(expected = "Expense purpose must not be empty")]
fn propose_expense_rejects_empty_purpose() {
    let mut ledger = Ledger::new();
    two_approver_sacco(&mut ledger);
    ledger.execute(
        &[meta(aid(1)), meta(aid(8)), meta(aid(2))],
        Instruction::ProposeExpense {
            amount: 1_000,
            purpose: String::new(),
            evidence_cid: "cid".into(),
            required_approvals: 2,
        },
    );
}

#[test]
fn approve_expense_below_threshold_stays_pending() {
    let mut ledger = Ledger::new();
    two_approver_sacco(&mut ledger);
    proposed_expense(&mut ledger, aid(8));
    ledger.execute(&[meta(aid(1)), meta(aid(8)), meta(aid(2))], Instruction::ApproveExpense);
    assert_eq!(ledger.expense(aid(8)).status, ExpenseStatus::Proposed);
    assert_eq!(ledger.expense(aid(8)).approvers, vec![aid(2)]);
}

#[test]
fn approve_expense_executes_at_exact_threshold() {
    let mut ledger = Ledger::new();
    two_approver_sacco(&mut ledger);
    proposed_expense(&mut ledger, aid(8));
    ledger.execute(&[meta(aid(1)), meta(aid(8)), meta(aid(2))], Instruction::ApproveExpense);
    ledger.execute(&[meta(aid(1)), meta(aid(8)), meta(aid(3))], Instruction::ApproveExpense);
    assert_eq!(ledger.expense(aid(8)).status, ExpenseStatus::Approved);
    assert_eq!(ledger.expense(aid(8)).approvers, vec![aid(2), aid(3)]);
}

#[test]
#[should_panic(expected = "Already approved by this approver")]
fn approve_expense_duplicate_approver_rejected() {
    let mut ledger = Ledger::new();
    two_approver_sacco(&mut ledger);
    proposed_expense(&mut ledger, aid(8));
    ledger.execute(&[meta(aid(1)), meta(aid(8)), meta(aid(2))], Instruction::ApproveExpense);
    ledger.execute(&[meta(aid(1)), meta(aid(8)), meta(aid(2))], Instruction::ApproveExpense);
}

#[test]
fn reject_expense_approver_only() {
    let mut ledger = Ledger::new();
    two_approver_sacco(&mut ledger);
    proposed_expense(&mut ledger, aid(8));
    ledger.execute(&[meta(aid(1)), meta(aid(8)), meta(aid(2))], Instruction::RejectExpense);
    let expense = ledger.expense(aid(8));
    assert_eq!(expense.status, ExpenseStatus::Rejected);
    assert_eq!(expense.rejectors, vec![aid(2)]);
}

#[test]
fn execute_expense_records_off_chain_release() {
    let mut ledger = Ledger::new();
    two_approver_sacco(&mut ledger);
    proposed_expense(&mut ledger, aid(8));
    for approver in [aid(2), aid(3)] {
        ledger.execute(&[meta(aid(1)), meta(aid(8)), meta(approver)], Instruction::ApproveExpense);
    }
    ledger.execute(
        &[meta(aid(1)), meta(aid(8)), meta(aid(2))],
        Instruction::ExecuteExpense { executed_evidence: "bafy-receipt".into(), amount: 120_000 },
    );
    let expense = ledger.expense(aid(8));
    assert_eq!(expense.status, ExpenseStatus::Executed);
    assert_eq!(expense.executed_evidence.as_deref(), Some("bafy-receipt"));
    assert_eq!(ledger.sacco(aid(1)).total_expenses, 120_000);
}
```

Run: `cd programs && cargo test -p sacco_ledger_program --test expenses 2>&1 | tail -20`
Expected: FAIL — expense variants unhandled.

- [ ] **Step 2: Add `required_approvals` to `Expense` and the expense effects**

In `core/src/lib.rs`, add to `Expense`:

```rust
    /// Multi-sig threshold pinned when the expense was proposed.
    pub required_approvals: u8,
```

In `core/src/effect.rs`:

```rust
/// Effects on an expense shard.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum ExpenseEffect {
    Init { sacco: AccountId, amount: u128, purpose: String, evidence_cid: String, requested_by: AccountId, required_approvals: u8 },
    Approve { approver: AccountId },
    Reject { approver: AccountId },
    Execute { executed_evidence: String, amount: u128 },
}
```

Add to `SaccoEffect`:

```rust
ProposeExpense { proposer: AccountId, amount: u128, required_approvals: u8 },
ExecuteExpense { approver: AccountId, amount: u128 },
```

Add `Expense(ExpenseEffect)` to `Effect`.

- [ ] **Step 3: Write the failing transition tests, then the transitions**

`core/src/transition/expense.rs`:

```rust
//! The expense shard: proposal, approvals and the execution record.

use crate::codec::{decode, encode};
use crate::effect::ExpenseEffect;
use crate::{AccountId, Expense, ExpenseStatus};

#[must_use]
pub fn apply(effect: ExpenseEffect, pre: &[u8]) -> Option<Vec<u8>> {
    match effect {
        ExpenseEffect::Init {
            sacco,
            amount,
            purpose,
            evidence_cid,
            requested_by,
            required_approvals,
        } => {
            assert!(pre.is_empty(), "Expense account must be uninitialized");
            assert!(amount > 0, "Expense amount must be greater than 0");
            assert!(!purpose.is_empty(), "Expense purpose must not be empty");
            assert!(!evidence_cid.is_empty(), "Expense evidence must not be empty");
            Some(encode(&Expense {
                sacco_id: sacco,
                amount,
                purpose,
                evidence_cid,
                requested_by,
                status: ExpenseStatus::Proposed,
                approvers: vec![],
                rejectors: vec![],
                executed_evidence: None,
                required_approvals,
            }))
        }
        ExpenseEffect::Approve { approver } => {
            let mut expense: Expense =
                decode(pre).expect("Expense shard must hold valid Expense data");
            assert_eq!(expense.status, ExpenseStatus::Proposed, "Expense is not proposed");
            assert!(!expense.approvers.contains(&approver), "Already approved by this approver");
            assert!(!expense.rejectors.contains(&approver), "Already rejected by this approver");
            expense.approvers.push(approver);
            if expense.approvers.len() >= usize::from(expense.required_approvals) {
                expense.status = ExpenseStatus::Approved;
            }
            Some(encode(&expense))
        }
        ExpenseEffect::Reject { approver } => {
            let mut expense: Expense =
                decode(pre).expect("Expense shard must hold valid Expense data");
            assert_eq!(expense.status, ExpenseStatus::Proposed, "Expense is not proposed");
            assert!(!expense.rejectors.contains(&approver), "Already rejected by this approver");
            expense.rejectors.push(approver);
            expense.status = ExpenseStatus::Rejected;
            Some(encode(&expense))
        }
        ExpenseEffect::Execute { executed_evidence, amount } => {
            let mut expense: Expense =
                decode(pre).expect("Expense shard must hold valid Expense data");
            assert_eq!(
                expense.status,
                ExpenseStatus::Approved,
                "Expense must be approved before execution"
            );
            assert!(!executed_evidence.is_empty(), "Execution evidence must not be empty");
            assert_eq!(amount, expense.amount, "Amount does not match the expense");
            expense.status = ExpenseStatus::Executed;
            expense.executed_evidence = Some(executed_evidence);
            Some(encode(&expense))
        }
    }
}

/// An approved expense for tests in this module.
pub fn approved(amount: u128) -> Expense {
    Expense {
        sacco_id: AccountId::new([1; 32]),
        amount,
        purpose: "desk".into(),
        evidence_cid: "cid".into(),
        requested_by: AccountId::new([2; 32]),
        status: ExpenseStatus::Approved,
        approvers: vec![AccountId::new([2; 32])],
        rejectors: vec![],
        executed_evidence: None,
        required_approvals: 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_amount_argument_must_match_the_expense() {
        let pre = encode(&approved(100));
        assert!(std::panic::catch_unwind(|| {
            apply(
                ExpenseEffect::Execute { executed_evidence: "receipt".into(), amount: 101 },
                &pre,
            )
        })
        .is_err());
    }
}
```

In `core/src/transition/sacco.rs`, add:

```rust
SaccoEffect::ProposeExpense { proposer, amount, required_approvals } => {
    let mut sacco = decode::<Sacco>(pre).expect("Sacco shard must hold valid Sacco data");
    assert!(sacco.is_verified, "Sacco is not verified");
    assert!(is_approver(&sacco, &proposer), "Not a SACCO approver");
    assert!(amount > 0, "Expense amount must be greater than 0");
    assert_eq!(
        required_approvals, sacco.required_approvals,
        "Approver threshold does not match the SACCO configuration"
    );
    Some(encode(&sacco))
}
SaccoEffect::ExecuteExpense { approver, amount } => {
    let mut sacco = decode::<Sacco>(pre).expect("Sacco shard must hold valid Sacco data");
    assert!(sacco.is_verified, "Sacco is not verified");
    assert!(is_approver(&sacco, &approver), "Not a SACCO approver");
    sacco.total_expenses = sacco
        .total_expenses
        .checked_add(amount)
        .expect("expense total overflow");
    Some(encode(&sacco))
}
```

`ProposeExpense` writes the SACCO even though it changes nothing, so the threshold check has a shard to live on and the plan never emits two effects for one shard. Add `pub mod expense;` to `core/src/transition/mod.rs`.

Run: `cd programs && cargo test -p sacco_ledger_core --lib 2>&1 | tail -10`
Expected: PASS.

- [ ] **Step 4: Plan and apply the expense instructions**

In `src/plan.rs`:

```rust
Instruction::ProposeExpense { amount, purpose, evidence_cid, required_approvals } => {
    let [sacco, expense, proposer] = <&[_; 3]>::try_from(input.accounts.as_slice())
        .expect("ProposeExpense requires sacco, expense and proposer");
    assert!(expense.is_authorized, "Expense account must be authorized");
    let proposer = signer(proposer, "Proposer must authorize");
    plan.effect(
        sacco,
        &Effect::Sacco(SaccoEffect::ProposeExpense { proposer, amount, required_approvals }),
    );
    plan.effect(
        expense,
        &Effect::Expense(ExpenseEffect::Init {
            sacco: sacco.account_id,
            amount,
            purpose,
            evidence_cid,
            requested_by: proposer,
            required_approvals,
        }),
    );
}
Instruction::ApproveExpense => {
    let [sacco, expense, approver] = <&[_; 3]>::try_from(input.accounts.as_slice())
        .expect("ApproveExpense requires sacco, expense and approver");
    let approver = signer(approver, "Approver must authorize");
    plan.effect(sacco, &Effect::Sacco(SaccoEffect::AuthorizeApprover { approver }));
    plan.effect(expense, &Effect::Expense(ExpenseEffect::Approve { approver }));
}
Instruction::RejectExpense => {
    let [sacco, expense, approver] = <&[_; 3]>::try_from(input.accounts.as_slice())
        .expect("RejectExpense requires sacco, expense and approver");
    let approver = signer(approver, "Approver must authorize");
    plan.effect(sacco, &Effect::Sacco(SaccoEffect::AuthorizeApprover { approver }));
    plan.effect(expense, &Effect::Expense(ExpenseEffect::Reject { approver }));
}
Instruction::ExecuteExpense { executed_evidence, amount } => {
    let [sacco, expense, approver] = <&[_; 3]>::try_from(input.accounts.as_slice())
        .expect("ExecuteExpense requires sacco, expense and approver");
    let approver = signer(approver, "Approver must authorize");
    plan.effect(
        sacco,
        &Effect::Sacco(SaccoEffect::ExecuteExpense { approver, amount }),
    );
    plan.effect(
        expense,
        &Effect::Expense(ExpenseEffect::Execute { executed_evidence, amount }),
    );
}
```

In `src/apply.rs`: `Effect::Expense(effect) => transition::expense::apply(effect, pre),`

Run: `cd programs && RISC0_DEV_MODE=1 cargo test -p sacco_ledger_program --test expenses 2>&1 | tail -25`
Expected: PASS (9 tests).

- [ ] **Step 5: Commit**

```bash
git add programs
git commit -F - <<'EOF'
feat: port multi-sig expenses to plan/apply (#3)

Changes:
- Expense carries the threshold pinned at proposal; ProposeExpense validates
  it against the live SACCO config on the SACCO shard, and approvals compare
  against the pinned value.
- Approve/Reject/Execute effects hold the status machine; Execute validates
  the amount argument against the expense before the SACCO shard adds it to
  the executed total.
- tests/expenses.rs ports the 8 SPEL-era expense tests onto the harness.

Verification:
- `RISC0_DEV_MODE=1 cargo test -p sacco_ledger_program --test expenses`: 9 pass
- `cargo test -p sacco_ledger_core --lib`: pass
EOF
```

---

## Task 6: Period summary and the completed instruction set

**Files:**
- Modify: `programs/sacco_ledger/core/src/lib.rs` (`Totals`, `PeriodSummary::new`)
- Modify: `programs/sacco_ledger/core/src/effect.rs`
- Modify: `programs/sacco_ledger/core/src/transition/sacco.rs`
- Create: `programs/sacco_ledger/core/src/transition/summary.rs`
- Modify: `programs/sacco_ledger/core/src/transition/mod.rs`
- Modify: `programs/sacco_ledger/src/plan.rs` (remove the catch-all)
- Modify: `programs/sacco_ledger/src/apply.rs`
- Create: `programs/sacco_ledger/tests/period_summary.rs`
- Modify: `programs/sacco_ledger/tests/support/mod.rs`

**Interfaces:**
- Consumes: everything above.
- Produces: `Totals`, `PeriodSummary::new`, `SaccoEffect::HoldTotals`, `SummaryEffect::Init`, an exhaustive `plan`/`apply` pair.

- [ ] **Step 1: Write the failing summary tests**

Add to `tests/support/mod.rs`:

```rust
/// The SACCO's live totals, as a client would read them.
pub fn totals(ledger: &Ledger) -> Totals {
    Totals::of(&ledger.sacco(aid(1)))
}
```

`programs/sacco_ledger/tests/period_summary.rs`:

```rust
//! Period snapshots: a public, validated copy of the SACCO totals.

mod support;

use sacco_ledger_core::{Instruction, Totals};
use support::{add_verified_member, aid, meta, totals, verified_sacco, Ledger};

fn funded(ledger: &mut Ledger) {
    verified_sacco(ledger);
    add_verified_member(ledger, aid(6));
    ledger.execute(
        &[meta(aid(1)), meta(aid(6)), meta(aid(7)), meta(aid(2))],
        Instruction::RecordDeposit {
            member: aid(6),
            amount: 75_000,
            deposit_type: sacco_ledger_core::DepositType::Savings,
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
        Instruction::PostPeriodSummary { label: "2026-11".into(), totals: snapshot },
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
        Instruction::PostPeriodSummary { label: "2026-11".into(), totals: snapshot },
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
            deposit_type: sacco_ledger_core::DepositType::Savings,
            reference: "MPZA-2".into(),
        },
    );
    ledger.execute(
        &[meta(aid(1)), meta(aid(10)), meta(aid(2))],
        Instruction::PostPeriodSummary { label: "2026-11".into(), totals: stale },
    );
}

#[test]
#[should_panic(expected = "Period label must not be empty")]
fn post_period_summary_rejects_empty_label() {
    let mut ledger = Ledger::new();
    funded(&mut ledger);
    let snapshot = totals(&ledger);
    ledger.execute(
        &[meta(aid(1)), meta(aid(10)), meta(aid(2))],
        Instruction::PostPeriodSummary { label: String::new(), totals: snapshot },
    );
}
```

Run: `cd programs && cargo test -p sacco_ledger_program --test period_summary 2>&1 | tail -20`
Expected: FAIL — `Totals` does not exist.

- [ ] **Step 2: Add `Totals` and the summary effect**

In `core/src/lib.rs`:

```rust
/// The SACCO's five public aggregate totals, as one portable value.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize,
)]
pub struct Totals {
    pub deposits_savings: u128,
    pub deposits_share_capital: u128,
    pub loans_disbursed: u128,
    pub loans_repaid: u128,
    pub expenses: u128,
}

impl Totals {
    /// Reads the totals out of a SACCO account.
    #[must_use]
    pub fn of(sacco: &Sacco) -> Self {
        Self {
            deposits_savings: sacco.total_deposits_savings,
            deposits_share_capital: sacco.total_deposits_share_capital,
            loans_disbursed: sacco.total_loans_disbursed,
            loans_repaid: sacco.total_loans_repaid,
            expenses: sacco.total_expenses,
        }
    }
}

impl PeriodSummary {
    /// A snapshot of `totals` for `sacco` under `label`.
    #[must_use]
    pub fn new(sacco_id: AccountId, label: String, totals: Totals) -> Self {
        Self {
            sacco_id,
            label,
            deposits_savings: totals.deposits_savings,
            deposits_share_capital: totals.deposits_share_capital,
            loans_disbursed: totals.loans_disbursed,
            loans_repaid: totals.loans_repaid,
            expenses: totals.expenses,
        }
    }
}
```

Change the instruction:

```rust
PostPeriodSummary { label: String, totals: Totals },
```

In `core/src/effect.rs`:

```rust
/// Effects on a period-summary shard.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum SummaryEffect {
    Init { sacco: AccountId, label: String, totals: Totals },
}
```

Add to `SaccoEffect`:

```rust
HoldTotals { totals: Totals, poster: AccountId },
```

Add `Summary(SummaryEffect)` to `Effect`.

- [ ] **Step 3: Write the failing transition tests, then the transitions**

`core/src/transition/summary.rs`:

```rust
//! The period-summary shard: a validated copy of the SACCO's public totals.

use crate::codec::encode;
use crate::effect::SummaryEffect;
use crate::PeriodSummary;

#[must_use]
pub fn apply(effect: SummaryEffect, pre: &[u8]) -> Option<Vec<u8>> {
    match effect {
        SummaryEffect::Init { sacco, label, totals } => {
            assert!(pre.is_empty(), "Summary account must be uninitialized");
            assert!(!label.is_empty(), "Period label must not be empty");
            Some(encode(&PeriodSummary::new(sacco, label, totals)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AccountId;

    #[test]
    fn a_summary_is_write_once() {
        let effect = SummaryEffect::Init {
            sacco: AccountId::new([1; 32]),
            label: "2026-11".into(),
            totals: crate::Totals::default(),
        };
        let first = apply(effect.clone(), &[]).expect("writes");
        assert!(std::panic::catch_unwind(|| apply(effect, &first)).is_err());
    }
}
```

In `core/src/transition/sacco.rs`, replace the `_ => panic!` arm with:

```rust
SaccoEffect::HoldTotals { totals, poster } => {
    let sacco = decode::<Sacco>(pre).expect("Sacco shard must hold valid Sacco data");
    assert!(is_approver(&sacco, &poster), "Not a SACCO approver");
    assert_eq!(
        totals,
        crate::Totals::of(&sacco),
        "Totals do not match the SACCO's live totals"
    );
    None
}
```

Every `SaccoEffect` variant is now handled, so the transitional catch-all is gone and an added
variant fails to compile until its rule is written.

Add `pub mod summary;` to `core/src/transition/mod.rs`.

Run: `cd programs && cargo test -p sacco_ledger_core --lib 2>&1 | tail -10`
Expected: PASS.

- [ ] **Step 4: Plan and apply the summary instruction, and make the matches exhaustive**

In `src/plan.rs`, replace the catch-all with:

```rust
Instruction::PostPeriodSummary { label, totals } => {
    let [sacco, summary, poster] = <&[_; 3]>::try_from(input.accounts.as_slice())
        .expect("PostPeriodSummary requires sacco, summary and poster");
    assert!(summary.is_authorized, "Summary account must be authorized");
    let poster = signer(poster, "Poster must authorize");
    plan.effect(sacco, &Effect::Sacco(SaccoEffect::HoldTotals { totals, poster }));
    plan.effect(
        summary,
        &Effect::Summary(SummaryEffect::Init { sacco: sacco.account_id, label, totals }),
    );
}
```

`HoldTotals` carries both the snapshot and the poster, because approver membership lives on the
SACCO shard and no effect can read another shard.

Update the `Instruction` doc comments for the account lists and arguments that changed:
`ConfirmDeposit` takes deposit and member only; `ApplyLoan`, `ProposeExpense`, `DisburseLoan`,
`ExecuteExpense` and `PostPeriodSummary` gain the new argument.

There is no catch-all left: the `match instruction` covers all 18 variants, so a new instruction
fails to compile until it is planned.

In `src/apply.rs`: `Effect::Summary(effect) => transition::summary::apply(effect, pre),`

Run: `cd programs && RISC0_DEV_MODE=1 cargo test -p sacco_ledger_program --test period_summary 2>&1 | tail -25`
Expected: PASS (4 tests).

- [ ] **Step 5: Run the whole workspace suite and confirm the enum test**

Run: `cd programs && RISC0_DEV_MODE=1 cargo test 2>&1 | tail -30`
Expected: PASS — 41 program tests + core tests, including `every_variant_round_trips` from Task 2.

- [ ] **Step 6: Commit**

```bash
git add programs
git commit -F - <<'EOF'
feat: port period summaries and complete the v0.3 instruction set (#3)

Changes:
- Totals is a portable snapshot; PostPeriodSummary takes it as an argument
  and the SACCO shard validates it against live totals before the summary
  shard is written, because no effect can read another shard.
- HoldTotals also carries the poster so approver membership is checked where
  the approver list lives.
- plan now matches every Instruction variant exhaustively; the transitional
  catch-all is gone.

Verification:
- `RISC0_DEV_MODE=1 cargo test`: pass (program + core, incl. enum round trip)
EOF
```

---

## Task 7: IDL generator and CI drift check

**Files:**
- Create: `programs/sacco_ledger/idl/Cargo.toml`
- Create: `programs/sacco_ledger/idl/src/lib.rs`
- Create: `programs/sacco_ledger/idl/src/main.rs`
- Create: `programs/sacco_ledger/idl/tests/coverage.rs`
- Regenerate: `artifacts/sacco_ledger-idl.json`
- Modify: `.github/workflows/lez-program.yml`

**Interfaces:**
- Consumes: `sacco_ledger_core::Instruction`.
- Produces: `sacco_ledger_idl::{INSTRUCTIONS, render, variant_name}` and a `--check` binary mode.

- [ ] **Step 1: Write the failing coverage test**

Add the crate to the workspace in `programs/Cargo.toml`:

```toml
members = [
    "sacco_ledger/core",
    "sacco_ledger",
    "sacco_ledger/idl",
    "sacco_ledger/methods",
]
```

`programs/sacco_ledger/idl/Cargo.toml`:

```toml
[package]
name = "sacco_ledger_idl"
version = "0.1.0"
edition = "2021"

[lints]
workspace = true

[dependencies]
sacco_ledger_core = { workspace = true }
serde_json = { workspace = true }
```

`programs/sacco_ledger/idl/src/lib.rs`:

```rust
//! The `sacco_ledger` IDL: one declarative table, rendered to JSON.
//!
//! v0.3 removed SPEL's IDL generation, so the artifact is produced here and
//! compared in CI. Adding an instruction without its entry fails to compile
//! (`variant_name`) and fails the coverage test.

use sacco_ledger_core::Instruction;

pub struct AccountSpec {
    pub name: &'static str,
    pub writable: bool,
    pub signer: bool,
    pub init: bool,
}

pub struct ArgSpec {
    pub name: &'static str,
    pub ty: &'static str,
}

pub struct InstructionSpec {
    pub name: &'static str,
    pub accounts: &'static [AccountSpec],
    pub args: &'static [ArgSpec],
}

const NONE: &[ArgSpec] = &[];
const ID: &[ArgSpec] = &[ArgSpec { name: "id", ty: "account_id" }];

pub const INSTRUCTIONS: &[InstructionSpec] = &[
    InstructionSpec {
        name: "initialize",
        accounts: &[AccountSpec { name: "config", writable: true, signer: false, init: true }],
        args: NONE,
    },
    InstructionSpec {
        name: "register_sacco",
        accounts: &[
            AccountSpec { name: "sacco", writable: true, signer: false, init: true },
            AccountSpec { name: "admin", writable: false, signer: true, init: false },
        ],
        args: &[
            ArgSpec { name: "name", ty: "string" },
            ArgSpec { name: "registration_number", ty: "string" },
            ArgSpec { name: "treasury", ty: "account_id" },
        ],
    },
    InstructionSpec {
        name: "verify_sacco",
        accounts: &[
            AccountSpec { name: "sacco", writable: true, signer: false, init: false },
            AccountSpec { name: "registrar", writable: false, signer: true, init: false },
        ],
        args: NONE,
    },
    InstructionSpec {
        name: "add_approver",
        accounts: &[
            AccountSpec { name: "sacco", writable: true, signer: false, init: false },
            AccountSpec { name: "admin", writable: false, signer: true, init: false },
        ],
        args: ID,
    },
    InstructionSpec {
        name: "add_supervisor",
        accounts: &[
            AccountSpec { name: "sacco", writable: true, signer: false, init: false },
            AccountSpec { name: "admin", writable: false, signer: true, init: false },
        ],
        args: ID,
    },
    InstructionSpec {
        name: "add_member",
        accounts: &[
            AccountSpec { name: "sacco", writable: true, signer: false, init: false },
            AccountSpec { name: "member", writable: true, signer: false, init: true },
            AccountSpec { name: "admin", writable: false, signer: true, init: false },
        ],
        args: ID,
    },
    InstructionSpec {
        name: "record_deposit",
        accounts: &[
            AccountSpec { name: "sacco", writable: true, signer: false, init: false },
            AccountSpec { name: "member_account", writable: true, signer: false, init: false },
            AccountSpec { name: "deposit", writable: true, signer: false, init: true },
            AccountSpec { name: "recorder", writable: false, signer: true, init: false },
        ],
        args: &[
            ArgSpec { name: "member", ty: "account_id" },
            ArgSpec { name: "amount", ty: "u128" },
            ArgSpec { name: "deposit_type", ty: "deposit_type" },
            ArgSpec { name: "reference", ty: "string" },
        ],
    },
    InstructionSpec {
        name: "confirm_deposit",
        accounts: &[
            AccountSpec { name: "deposit", writable: true, signer: false, init: false },
            AccountSpec { name: "member", writable: false, signer: true, init: false },
        ],
        args: NONE,
    },
    InstructionSpec {
        name: "apply_loan",
        accounts: &[
            AccountSpec { name: "sacco", writable: true, signer: false, init: false },
            AccountSpec { name: "member_account", writable: true, signer: false, init: false },
            AccountSpec { name: "loan", writable: true, signer: false, init: true },
            AccountSpec { name: "member_signer", writable: false, signer: true, init: false },
        ],
        args: &[
            ArgSpec { name: "amount", ty: "u128" },
            ArgSpec { name: "purpose", ty: "string" },
            ArgSpec { name: "required_approvals", ty: "u8" },
        ],
    },
    InstructionSpec {
        name: "approve_loan",
        accounts: &[
            AccountSpec { name: "sacco", writable: false, signer: false, init: false },
            AccountSpec { name: "loan", writable: true, signer: false, init: false },
            AccountSpec { name: "approver", writable: false, signer: true, init: false },
        ],
        args: NONE,
    },
    InstructionSpec {
        name: "reject_loan",
        accounts: &[
            AccountSpec { name: "sacco", writable: false, signer: false, init: false },
            AccountSpec { name: "loan", writable: true, signer: false, init: false },
            AccountSpec { name: "approver", writable: false, signer: true, init: false },
        ],
        args: NONE,
    },
    InstructionSpec {
        name: "disburse_loan",
        accounts: &[
            AccountSpec { name: "sacco", writable: true, signer: false, init: false },
            AccountSpec { name: "member_account", writable: false, signer: false, init: false },
            AccountSpec { name: "loan", writable: true, signer: false, init: false },
            AccountSpec { name: "approver", writable: false, signer: true, init: false },
        ],
        args: &[
            ArgSpec { name: "evidence_cid", ty: "string" },
            ArgSpec { name: "principal", ty: "u128" },
        ],
    },
    InstructionSpec {
        name: "record_repayment",
        accounts: &[
            AccountSpec { name: "sacco", writable: true, signer: false, init: false },
            AccountSpec { name: "member_account", writable: false, signer: false, init: false },
            AccountSpec { name: "loan_account", writable: true, signer: false, init: false },
            AccountSpec { name: "approver", writable: false, signer: true, init: false },
        ],
        args: &[
            ArgSpec { name: "loan", ty: "account_id" },
            ArgSpec { name: "amount", ty: "u128" },
            ArgSpec { name: "reference", ty: "string" },
        ],
    },
    InstructionSpec {
        name: "propose_expense",
        accounts: &[
            AccountSpec { name: "sacco", writable: true, signer: false, init: false },
            AccountSpec { name: "expense", writable: true, signer: false, init: true },
            AccountSpec { name: "proposer", writable: false, signer: true, init: false },
        ],
        args: &[
            ArgSpec { name: "amount", ty: "u128" },
            ArgSpec { name: "purpose", ty: "string" },
            ArgSpec { name: "evidence_cid", ty: "string" },
            ArgSpec { name: "required_approvals", ty: "u8" },
        ],
    },
    InstructionSpec {
        name: "approve_expense",
        accounts: &[
            AccountSpec { name: "sacco", writable: false, signer: false, init: false },
            AccountSpec { name: "expense", writable: true, signer: false, init: false },
            AccountSpec { name: "approver", writable: false, signer: true, init: false },
        ],
        args: NONE,
    },
    InstructionSpec {
        name: "reject_expense",
        accounts: &[
            AccountSpec { name: "sacco", writable: false, signer: false, init: false },
            AccountSpec { name: "expense", writable: true, signer: false, init: false },
            AccountSpec { name: "approver", writable: false, signer: true, init: false },
        ],
        args: NONE,
    },
    InstructionSpec {
        name: "execute_expense",
        accounts: &[
            AccountSpec { name: "sacco", writable: true, signer: false, init: false },
            AccountSpec { name: "expense", writable: true, signer: false, init: false },
            AccountSpec { name: "approver", writable: false, signer: true, init: false },
        ],
        args: &[
            ArgSpec { name: "executed_evidence", ty: "string" },
            ArgSpec { name: "amount", ty: "u128" },
        ],
    },
    InstructionSpec {
        name: "post_period_summary",
        accounts: &[
            AccountSpec { name: "sacco", writable: false, signer: false, init: false },
            AccountSpec { name: "summary", writable: true, signer: false, init: true },
            AccountSpec { name: "poster", writable: false, signer: true, init: false },
        ],
        args: &[
            ArgSpec { name: "label", ty: "string" },
            ArgSpec { name: "totals", ty: "totals" },
        ],
    },
];

/// The IDL name of an instruction. Exhaustive on purpose: a new variant will
/// not compile until it is named here and added to `INSTRUCTIONS`.
#[must_use]
pub fn variant_name(instruction: &Instruction) -> &'static str {
    match instruction {
        Instruction::Initialize => "initialize",
        Instruction::RegisterSacco { .. } => "register_sacco",
        Instruction::VerifySacco => "verify_sacco",
        Instruction::AddApprover { .. } => "add_approver",
        Instruction::AddSupervisor { .. } => "add_supervisor",
        Instruction::AddMember { .. } => "add_member",
        Instruction::RecordDeposit { .. } => "record_deposit",
        Instruction::ConfirmDeposit => "confirm_deposit",
        Instruction::ApplyLoan { .. } => "apply_loan",
        Instruction::ApproveLoan => "approve_loan",
        Instruction::RejectLoan => "reject_loan",
        Instruction::DisburseLoan { .. } => "disburse_loan",
        Instruction::RecordRepayment { .. } => "record_repayment",
        Instruction::ProposeExpense { .. } => "propose_expense",
        Instruction::ApproveExpense => "approve_expense",
        Instruction::RejectExpense => "reject_expense",
        Instruction::ExecuteExpense { .. } => "execute_expense",
        Instruction::PostPeriodSummary { .. } => "post_period_summary",
    }
}

/// Renders the IDL document exactly as `artifacts/sacco_ledger-idl.json` stores it.
#[must_use]
pub fn render() -> String {
    let instructions: Vec<serde_json::Value> = INSTRUCTIONS
        .iter()
        .map(|spec| {
            serde_json::json!({
                "name": spec.name,
                "accounts": spec.accounts.iter().map(|account| serde_json::json!({
                    "name": account.name,
                    "writable": account.writable,
                    "signer": account.signer,
                    "init": account.init,
                })).collect::<Vec<_>>(),
                "args": spec.args.iter().map(|arg| serde_json::json!({
                    "name": arg.name,
                    "type": arg.ty,
                })).collect::<Vec<_>>(),
            })
        })
        .collect();

    let document = serde_json::json!({
        "version": "0.2.0",
        "name": "sacco_ledger",
        "description": "Transparent SACCO attestation ledger: deposits, loans and expenses recorded and approved on chain; value stays off chain. Regenerated from the sacco_ledger_core::Instruction enum (borsh + serde).",
        "instructions": instructions,
    });

    let mut rendered = serde_json::to_string_pretty(&document).expect("IDL serializes");
    rendered.push('\n');
    rendered
}
```

`programs/sacco_ledger/idl/tests/coverage.rs`:

```rust
//! Every instruction has an IDL entry.

use sacco_ledger_core::{AccountId, DepositType, Instruction, Totals};
use sacco_ledger_idl::{variant_name, INSTRUCTIONS};

fn samples() -> Vec<Instruction> {
    let id = AccountId::new([7; 32]);
    vec![
        Instruction::Initialize,
        Instruction::RegisterSacco {
            name: "n".into(),
            registration_number: "r".into(),
            treasury: id,
        },
        Instruction::VerifySacco,
        Instruction::AddApprover { approver: id },
        Instruction::AddSupervisor { supervisor: id },
        Instruction::AddMember { identity: id },
        Instruction::RecordDeposit {
            member: id,
            amount: 1,
            deposit_type: DepositType::Savings,
            reference: "r".into(),
        },
        Instruction::ConfirmDeposit,
        Instruction::ApplyLoan { amount: 1, purpose: "p".into(), required_approvals: 1 },
        Instruction::ApproveLoan,
        Instruction::RejectLoan,
        Instruction::DisburseLoan { evidence_cid: "c".into(), principal: 1 },
        Instruction::RecordRepayment { loan: id, amount: 1, reference: "r".into() },
        Instruction::ProposeExpense {
            amount: 1,
            purpose: "p".into(),
            evidence_cid: "c".into(),
            required_approvals: 1,
        },
        Instruction::ApproveExpense,
        Instruction::RejectExpense,
        Instruction::ExecuteExpense { executed_evidence: "e".into(), amount: 1 },
        Instruction::PostPeriodSummary { label: "l".into(), totals: Totals::default() },
    ]
}

#[test]
fn the_table_covers_every_instruction() {
    let table: Vec<&str> = INSTRUCTIONS.iter().map(|spec| spec.name).collect();
    for instruction in samples() {
        assert!(
            table.contains(&variant_name(&instruction)),
            "{} is missing from the IDL table",
            variant_name(&instruction)
        );
    }
    assert_eq!(table.len(), samples().len(), "the table has an entry the enum does not use");
}

#[test]
fn the_committed_artifact_is_current() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../artifacts/sacco_ledger-idl.json");
    let committed = std::fs::read_to_string(path).expect("committed IDL artifact");
    assert_eq!(committed, sacco_ledger_idl::render(), "run the generator and commit the result");
}
```

Run: `cd programs && cargo test -p sacco_ledger_idl 2>&1 | tail -20`
Expected: FAIL — the committed artifact still has the old arguments.

- [ ] **Step 2: Regenerate the artifact**

`programs/sacco_ledger/idl/src/main.rs`:

```rust
//! Renders or checks `artifacts/sacco_ledger-idl.json`.

use std::process::ExitCode;

/// Repo-root artifact path, resolved at compile time so the command works from any cwd.
const ARTIFACT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../artifacts/sacco_ledger-idl.json");

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let rendered = sacco_ledger_idl::render();

    match args.as_slice() {
        [] => {
            print!("{rendered}");
            ExitCode::SUCCESS
        }
        [flag, path] if flag == "--out" => match std::fs::write(path, rendered) {
            Ok(()) => {
                println!("wrote {path}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("cannot write {path}: {error}");
                ExitCode::FAILURE
            }
        },
        [flag] if flag == "--check" => {
            let committed = std::fs::read_to_string(ARTIFACT).unwrap_or_default();
            if committed == rendered {
                println!("{ARTIFACT} is current");
                ExitCode::SUCCESS
            } else {
                eprintln!("{ARTIFACT} is stale: run `cargo run -p sacco_ledger_idl -- --out {ARTIFACT}`");
                ExitCode::FAILURE
            }
        }
        _ => {
            eprintln!("usage: sacco_ledger_idl [--out <path> | --check]");
            ExitCode::FAILURE
        }
    }
}
```

Run: `cd programs && cargo run -p sacco_ledger_idl -- --out ../artifacts/sacco_ledger-idl.json`
Note: the binary runs from `programs/`, so `../artifacts/...` is the repo artifact. Then re-run the test:

Run: `cd programs && cargo test -p sacco_ledger_idl 2>&1 | tail -10`
Expected: PASS (2 tests).

- [ ] **Step 3: Add the CI drift check**

In `.github/workflows/lez-program.yml`:

```yaml
      - name: Install Rust 1.98.1
        uses: dtolnay/rust-toolchain@1.98.1
        with:
          components: clippy, rustfmt
```

and, after the test step:

```yaml
      - name: IDL artifact is current
        working-directory: programs
        run: cargo run -p sacco_ledger_idl -- --check
```

- [ ] **Step 4: Commit**

```bash
git add programs artifacts/sacco_ledger-idl.json .github/workflows/lez-program.yml
git commit -F - <<'EOF'
feat: generate the sacco_ledger IDL and check it in CI (#3)

Changes:
- New sacco_ledger_idl crate holds one declarative instruction table and
  renders the same JSON schema sacco_core already parses; variant_name is an
  exhaustive match so a new instruction cannot compile without a name.
- Coverage test proves the table matches the enum and the committed artifact;
  `--check` regenerates in memory and fails CI when the file is stale.
- Artifact regenerated for required_approvals, principal, amount and totals
  arguments; workflow toolchain moved to 1.98.1.

Verification:
- `cargo test -p sacco_ledger_idl`: 2 pass
- `cargo run -p sacco_ledger_idl -- --check`: current
EOF
```

---

## Task 8: Local deploy, program id, branch and PR

**Files:**
- Create: `scaffold.toml` (from `logos-scaffold init`)
- Modify: `.gitignore` (from `logos-scaffold init`)
- Modify: `programs/sacco_ledger/README.md` (deployed program id, v0.3 note)

**Interfaces:**
- Consumes: a green test suite from Tasks 1–7.
- Produces: the local program id recorded in the README, a pushed branch and a PR.

- [ ] **Step 1: Initialize the scaffold project**

Run: `logos-scaffold init`
Run: `git status --short` → expect `scaffold.toml` and a `.gitignore` addition only.

- [ ] **Step 2: Provision the sequencer**

Run: `logos-scaffold setup`
Expected: sequencer, wallet and spel binaries land under `.scaffold/`. This downloads and builds; allow it to run in the background and do not poll. If it fails, record the failing command and stop this task here.

- [ ] **Step 3: Start the local network**

Run: `logos-scaffold localnet start`
Run: `logos-scaffold localnet status`
Expected: running, with a local RPC endpoint.

- [ ] **Step 4: Build the guest ELF**

Run: `cd programs && cargo risczero build --manifest-path sacco_ledger/methods/guest/Cargo.toml`
Expected: a `.bin` under `programs/target/riscv32im-risc0-zkvm-elf/docker/`.

If `cargo risczero build` cannot find the bin, fall back to:

```bash
cd programs/sacco_ledger/methods/guest
cargo build --release --target riscv32im-risc0-zkvm-elf
```

Note the resulting `sacco_ledger.bin` path.

- [ ] **Step 5: Deploy and capture the program id**

Run: `logos-scaffold deploy --program-path <path-to-sacco_ledger.bin> --json`
Expected: JSON containing the deployed program id. Save it.

- [ ] **Step 6: Make one call against the local chain**

Attempt the `initialize` call with the scaffold's spel forwarding:

Run: `logos-scaffold spel --help`
Use the documented transaction subcommand to call `initialize` with one writable account and capture the result.

If the scaffold CLI exposes no transaction path, record in the PR: "deployed program id ✅, initialize call not runnable through logos-scaffold 0.4.0; deferred to the Basecamp/wallet path in `docs/deploy.md`". Do not claim the call ran.

- [ ] **Step 7: Record the program id**

In `programs/sacco_ledger/README.md`, replace the v0.2.5 note with:

```markdown
## Local deploy

Built and deployed on `lee_core` v0.3.1 with `logos-scaffold` 0.4.0:

- Local sequencer program id: `<the id from Step 5>`
- Test: `RISC0_DEV_MODE=1 cargo test` in `programs/`
- IDL: `cargo run -p sacco_ledger_idl -- --check` in `programs/`

Testnet deployment stays with issue #4 and `docs/deploy.md`.
```

- [ ] **Step 8: Verify the whole branch**

Run: `cd programs && RISC0_DEV_MODE=1 cargo test 2>&1 | tail -20`
Run: `cd programs && cargo clippy --all-targets -- -D warnings 2>&1 | tail -20`
Run: `cd programs && cargo fmt --check`
Expected: tests pass, clippy clean, formatting clean.

- [ ] **Step 9: Commit, push, open the PR**

```bash
git add scaffold.toml .gitignore programs/sacco_ledger/README.md
git commit -F - <<'EOF'
feat: deploy sacco_ledger to a local sequencer (#3)

Changes:
- scaffold.toml initialised for logos-scaffold; local network provisioned and
  the v0.3 guest ELF deployed, with the program id recorded in the program
  README alongside the test and IDL commands.
- Notes that testnet deployment remains with issue #4.

Verification:
- `RISC0_DEV_MODE=1 cargo test`: pass
- `cargo clippy --all-targets -- -D warnings`: clean
- `cargo fmt --check`: clean
- `logos-scaffold deploy --program-path <elf> --json`: program id recorded
EOF
git push -u origin issue-3-lee-core-v03
```

Open the PR against the base branch, not `main`:

```bash
gh pr create --base issue-3-sacco-ledger-pr --head issue-3-lee-core-v03 \
  --title "Phase 1: port sacco_ledger to lee_core v0.3 (issue #3)" \
  --body "Closes #3

## What changed
- All 18 instructions now run on the v0.3.1 plan/apply ABI: one typed effect per mutated shard, with data-dependent authority in the shard that owns the data.
- Loan and Expense pin the approval threshold at proposal time; DisburseLoan, ExecuteExpense and PostPeriodSummary carry the mirrored value as an argument validated by the owning shard (no effect can read another shard).
- IDL is generated by a new crate with a CI drift check; the artifact is regenerated.
- The dormant custody module is removed; it could not compile under v0.3 and was unreachable.

## Why
Issue #3's remaining task: bump lee_core to v0.3, fix the breakages, deploy. This is stacked on #18 so the v0.2.5 base stays reviewable on its own.

## Verification
- \`RISC0_DEV_MODE=1 cargo test\`: <paste counts>
- \`cargo clippy --all-targets -- -D warnings\`: clean
- \`cargo run -p sacco_ledger_idl -- --check\`: current
- Local deploy: program id <id>, initialize call <ran|deferred per Step 6>"
```

Then tick the issue task and leave a short note on both #3 and #18:

```bash
gh issue comment 3 --repo LOGOS-UGANDA/transparent-sacco --body "v0.3 port is up in <PR URL>: all 18 instructions on plan/apply, generated IDL, local deploy at <program id>. The 'Bump lee_core to v0.3, fix breakages and deploy a hello-world program' task is done once this merges; #18 remains the v0.2.5 base."
gh pr comment 18 --repo LOGOS-UGANDA/transparent-sacco --body "The v0.3 port is stacked on this branch: <PR URL>. Merge this one first, then the port."
```

- [ ] **Step 10: Report**

Post the PR URL, the program id, and any step that could not be completed with its exact failure. Never merge and never delete branches unbidden.

---

## Self-review

**Spec coverage**

| Spec section | Task |
|---|---|
| Execution model, effect layout | 1–6 |
| Per-instruction mapping (18) | 2 (5 governance), 3 (2 deposits), 4 (5 loans), 5 (4 expenses), 6 (summary) |
| Cross-shard values / ABI changes | 4 (`required_approvals`, `principal`), 5 (`required_approvals`, `amount`), 6 (`totals`) |
| IDL generator + CI | 7 |
| Tests (ported 41 + new) | 2–6, plus Review Focus pins in 2, 3, 6 |
| Toolchain, dependencies, CI | 1, 7 |
| Local deploy + program id | 8 |
| Custody deletion | 1 |

**Placeholder scan:** no `TBD`/`TODO`; every step names its command and expected result. Task 8 Step 6
has a documented fallback rather than a guess, because `logos-scaffold 0.4.0` exposes no transaction
subcommand in its help output.

**Type consistency:** `Effect` variants used in `plan`/`apply` are exactly those declared in
`core/src/effect.rs`; `SaccoEffect::HoldTotals { totals, poster }` matches the Task 6 transition and
plan arm; `Loan.required_approvals` / `Expense.required_approvals` are set at Init and read at
Approve in the same task that introduces them.

**Known sequencing note:** the enum round-trip test written in Task 2 references `Totals` and the
new arguments, so it only passes from Task 6 onward. Task 2's commit therefore runs the `setup` and
core tests, not that file; Task 6 runs the full suite that turns it green. Do not tick Task 2's final
suite until Task 6.

