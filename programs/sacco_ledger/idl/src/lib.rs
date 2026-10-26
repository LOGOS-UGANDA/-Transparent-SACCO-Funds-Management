//! The `sacco_ledger` IDL: one declarative table, rendered to JSON.
//!
//! v0.3 removed SPEL's IDL generation, so the artifact is produced here and
//! compared in CI. Adding an instruction without its entry fails to compile
//! (`variant_name`) and fails the coverage test.

use sacco_ledger_core::Instruction;

/// One account in an instruction's account list.
pub struct AccountSpec {
    pub name: &'static str,
    pub writable: bool,
    pub signer: bool,
    pub init: bool,
}

/// The declared type of an argument: a scalar name, or a fixed set of values.
pub enum ArgType {
    Scalar(&'static str),
    Enum(&'static [&'static str]),
}

/// One argument in an instruction's argument list.
pub struct ArgSpec {
    pub name: &'static str,
    pub ty: ArgType,
}

/// One instruction in the IDL.
pub struct InstructionSpec {
    pub name: &'static str,
    pub accounts: &'static [AccountSpec],
    pub args: &'static [ArgSpec],
}

const NONE: &[ArgSpec] = &[];
const ID: &[ArgSpec] = &[ArgSpec {
    name: "id",
    ty: ArgType::Scalar("account_id"),
}];

/// The account types the program owns, in shard order.
const ACCOUNT_TYPES: &[&str] = &[
    "Sacco",
    "Member",
    "Deposit",
    "Loan",
    "Expense",
    "PeriodSummary",
    "ProgramConfig",
];

/// Notes copied into the artifact for client authors.
const NOTES: &[&str] = &[
    "Attestation model: the program records and approves off-chain money events; it holds no value and no token transfers occur.",
    "Unit variants (verify_sacco, confirm_deposit, approve/reject loan and expense) carry no args; arg names match the Instruction enum fields exactly.",
    "Approval thresholds are pinned into loans and expenses at creation and validated against the live SACCO config, because an effect cannot read another shard.",
];

/// Every instruction, in enum order.
pub const INSTRUCTIONS: &[InstructionSpec] = &[
    InstructionSpec {
        name: "initialize",
        accounts: &[AccountSpec {
            name: "config",
            writable: true,
            signer: false,
            init: true,
        }],
        args: NONE,
    },
    InstructionSpec {
        name: "register_sacco",
        accounts: &[
            AccountSpec {
                name: "sacco",
                writable: true,
                signer: false,
                init: true,
            },
            AccountSpec {
                name: "admin",
                writable: false,
                signer: true,
                init: false,
            },
        ],
        args: &[
            ArgSpec {
                name: "name",
                ty: ArgType::Scalar("string"),
            },
            ArgSpec {
                name: "registration_number",
                ty: ArgType::Scalar("string"),
            },
            ArgSpec {
                name: "treasury",
                ty: ArgType::Scalar("account_id"),
            },
        ],
    },
    InstructionSpec {
        name: "verify_sacco",
        accounts: &[
            AccountSpec {
                name: "sacco",
                writable: true,
                signer: false,
                init: false,
            },
            AccountSpec {
                name: "registrar",
                writable: false,
                signer: true,
                init: false,
            },
        ],
        args: NONE,
    },
    InstructionSpec {
        name: "add_approver",
        accounts: &[
            AccountSpec {
                name: "sacco",
                writable: true,
                signer: false,
                init: false,
            },
            AccountSpec {
                name: "admin",
                writable: false,
                signer: true,
                init: false,
            },
        ],
        args: ID,
    },
    InstructionSpec {
        name: "add_supervisor",
        accounts: &[
            AccountSpec {
                name: "sacco",
                writable: true,
                signer: false,
                init: false,
            },
            AccountSpec {
                name: "admin",
                writable: false,
                signer: true,
                init: false,
            },
        ],
        args: ID,
    },
    InstructionSpec {
        name: "add_member",
        accounts: &[
            AccountSpec {
                name: "sacco",
                writable: true,
                signer: false,
                init: false,
            },
            AccountSpec {
                name: "member",
                writable: true,
                signer: false,
                init: true,
            },
            AccountSpec {
                name: "admin",
                writable: false,
                signer: true,
                init: false,
            },
        ],
        args: ID,
    },
    InstructionSpec {
        name: "record_deposit",
        accounts: &[
            AccountSpec {
                name: "sacco",
                writable: true,
                signer: false,
                init: false,
            },
            AccountSpec {
                name: "member_account",
                writable: true,
                signer: false,
                init: false,
            },
            AccountSpec {
                name: "deposit",
                writable: true,
                signer: false,
                init: true,
            },
            AccountSpec {
                name: "recorder",
                writable: false,
                signer: true,
                init: false,
            },
        ],
        args: &[
            ArgSpec {
                name: "member",
                ty: ArgType::Scalar("account_id"),
            },
            ArgSpec {
                name: "amount",
                ty: ArgType::Scalar("u128"),
            },
            ArgSpec {
                name: "deposit_type",
                ty: ArgType::Enum(&["savings", "share_capital"]),
            },
            ArgSpec {
                name: "reference",
                ty: ArgType::Scalar("string"),
            },
        ],
    },
    InstructionSpec {
        name: "confirm_deposit",
        accounts: &[
            AccountSpec {
                name: "deposit",
                writable: true,
                signer: false,
                init: false,
            },
            AccountSpec {
                name: "member",
                writable: false,
                signer: true,
                init: false,
            },
        ],
        args: NONE,
    },
    InstructionSpec {
        name: "apply_loan",
        accounts: &[
            AccountSpec {
                name: "sacco",
                writable: true,
                signer: false,
                init: false,
            },
            AccountSpec {
                name: "member_account",
                writable: true,
                signer: false,
                init: false,
            },
            AccountSpec {
                name: "loan",
                writable: true,
                signer: false,
                init: true,
            },
            AccountSpec {
                name: "member_signer",
                writable: false,
                signer: true,
                init: false,
            },
        ],
        args: &[
            ArgSpec {
                name: "amount",
                ty: ArgType::Scalar("u128"),
            },
            ArgSpec {
                name: "purpose",
                ty: ArgType::Scalar("string"),
            },
            ArgSpec {
                name: "required_approvals",
                ty: ArgType::Scalar("u8"),
            },
        ],
    },
    InstructionSpec {
        name: "approve_loan",
        accounts: &[
            AccountSpec {
                name: "sacco",
                writable: false,
                signer: false,
                init: false,
            },
            AccountSpec {
                name: "loan",
                writable: true,
                signer: false,
                init: false,
            },
            AccountSpec {
                name: "approver",
                writable: false,
                signer: true,
                init: false,
            },
        ],
        args: NONE,
    },
    InstructionSpec {
        name: "reject_loan",
        accounts: &[
            AccountSpec {
                name: "sacco",
                writable: false,
                signer: false,
                init: false,
            },
            AccountSpec {
                name: "loan",
                writable: true,
                signer: false,
                init: false,
            },
            AccountSpec {
                name: "approver",
                writable: false,
                signer: true,
                init: false,
            },
        ],
        args: NONE,
    },
    InstructionSpec {
        name: "disburse_loan",
        accounts: &[
            AccountSpec {
                name: "sacco",
                writable: true,
                signer: false,
                init: false,
            },
            AccountSpec {
                name: "member_account",
                writable: false,
                signer: false,
                init: false,
            },
            AccountSpec {
                name: "loan",
                writable: true,
                signer: false,
                init: false,
            },
            AccountSpec {
                name: "approver",
                writable: false,
                signer: true,
                init: false,
            },
        ],
        args: &[
            ArgSpec {
                name: "evidence_cid",
                ty: ArgType::Scalar("string"),
            },
            ArgSpec {
                name: "principal",
                ty: ArgType::Scalar("u128"),
            },
        ],
    },
    InstructionSpec {
        name: "record_repayment",
        accounts: &[
            AccountSpec {
                name: "sacco",
                writable: true,
                signer: false,
                init: false,
            },
            AccountSpec {
                name: "member_account",
                writable: false,
                signer: false,
                init: false,
            },
            AccountSpec {
                name: "loan_account",
                writable: true,
                signer: false,
                init: false,
            },
            AccountSpec {
                name: "approver",
                writable: false,
                signer: true,
                init: false,
            },
        ],
        args: &[
            ArgSpec {
                name: "loan",
                ty: ArgType::Scalar("account_id"),
            },
            ArgSpec {
                name: "amount",
                ty: ArgType::Scalar("u128"),
            },
            ArgSpec {
                name: "reference",
                ty: ArgType::Scalar("string"),
            },
        ],
    },
    InstructionSpec {
        name: "propose_expense",
        accounts: &[
            AccountSpec {
                name: "sacco",
                writable: true,
                signer: false,
                init: false,
            },
            AccountSpec {
                name: "expense",
                writable: true,
                signer: false,
                init: true,
            },
            AccountSpec {
                name: "proposer",
                writable: false,
                signer: true,
                init: false,
            },
        ],
        args: &[
            ArgSpec {
                name: "amount",
                ty: ArgType::Scalar("u128"),
            },
            ArgSpec {
                name: "purpose",
                ty: ArgType::Scalar("string"),
            },
            ArgSpec {
                name: "evidence_cid",
                ty: ArgType::Scalar("string"),
            },
            ArgSpec {
                name: "required_approvals",
                ty: ArgType::Scalar("u8"),
            },
        ],
    },
    InstructionSpec {
        name: "approve_expense",
        accounts: &[
            AccountSpec {
                name: "sacco",
                writable: false,
                signer: false,
                init: false,
            },
            AccountSpec {
                name: "expense",
                writable: true,
                signer: false,
                init: false,
            },
            AccountSpec {
                name: "approver",
                writable: false,
                signer: true,
                init: false,
            },
        ],
        args: NONE,
    },
    InstructionSpec {
        name: "reject_expense",
        accounts: &[
            AccountSpec {
                name: "sacco",
                writable: false,
                signer: false,
                init: false,
            },
            AccountSpec {
                name: "expense",
                writable: true,
                signer: false,
                init: false,
            },
            AccountSpec {
                name: "approver",
                writable: false,
                signer: true,
                init: false,
            },
        ],
        args: NONE,
    },
    InstructionSpec {
        name: "execute_expense",
        accounts: &[
            AccountSpec {
                name: "sacco",
                writable: true,
                signer: false,
                init: false,
            },
            AccountSpec {
                name: "expense",
                writable: true,
                signer: false,
                init: false,
            },
            AccountSpec {
                name: "approver",
                writable: false,
                signer: true,
                init: false,
            },
        ],
        args: &[
            ArgSpec {
                name: "executed_evidence",
                ty: ArgType::Scalar("string"),
            },
            ArgSpec {
                name: "amount",
                ty: ArgType::Scalar("u128"),
            },
        ],
    },
    InstructionSpec {
        name: "post_period_summary",
        accounts: &[
            AccountSpec {
                name: "sacco",
                writable: false,
                signer: false,
                init: false,
            },
            AccountSpec {
                name: "summary",
                writable: true,
                signer: false,
                init: true,
            },
            AccountSpec {
                name: "poster",
                writable: false,
                signer: true,
                init: false,
            },
        ],
        args: &[
            ArgSpec {
                name: "label",
                ty: ArgType::Scalar("string"),
            },
            ArgSpec {
                name: "totals",
                ty: ArgType::Scalar("totals"),
            },
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
                    "type": match arg.ty {
                        ArgType::Scalar(name) => serde_json::json!(name),
                        ArgType::Enum(values) => serde_json::json!({ "enum": values }),
                    },
                })).collect::<Vec<_>>(),
            })
        })
        .collect();

    let document = serde_json::json!({
        "version": "0.2.0",
        "name": "sacco_ledger",
        "description": "Transparent SACCO attestation ledger: deposits, loans and expenses recorded and approved on chain; value stays off chain. Regenerated from the sacco_ledger_core::Instruction enum (borsh + serde).",
        "instructions": instructions,
        "account_types": ACCOUNT_TYPES,
        "notes": NOTES,
    });

    let mut rendered = serde_json::to_string_pretty(&document).expect("IDL serializes");
    rendered.push('\n');
    rendered
}
