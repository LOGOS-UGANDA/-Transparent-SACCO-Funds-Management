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

/// Structured argument types referenced by name from instruction args.
const TYPE_DEFS: &[(&str, &[(&str, &str)])] = &[(
    "totals",
    &[
        ("deposits_savings", "u128"),
        ("deposits_share_capital", "u128"),
        ("loans_disbursed", "u128"),
        ("loans_repaid", "u128"),
        ("expenses", "u128"),
    ],
)];

/// Notes copied into the artifact for client authors.
const NOTES: &[&str] = &[
    "Attestation model: the program records and approves off-chain money events; it holds no value and no token transfers occur.",
    "Unit variants (verify_sacco, confirm_deposit, approve/reject loan and expense) carry no args; arg names match the Instruction enum fields exactly.",
    "Approval thresholds are pinned into loans and expenses at creation and validated against the live SACCO config, because an effect cannot read another shard.",
    "Accounts with init: true require authorization (signer: true) so plan can claim the empty shard, except config: initialize validates its deterministic program-derived address instead.",
    "In v1 verify_sacco is self-verification by the SACCO admin.",
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
                signer: true,
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
                name: "admin",
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
        args: &[ArgSpec {
            name: "approver",
            ty: ArgType::Scalar("account_id"),
        }],
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
        args: &[ArgSpec {
            name: "supervisor",
            ty: ArgType::Scalar("account_id"),
        }],
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
                signer: true,
                init: true,
            },
            AccountSpec {
                name: "admin",
                writable: false,
                signer: true,
                init: false,
            },
        ],
        args: &[ArgSpec {
            name: "identity",
            ty: ArgType::Scalar("account_id"),
        }],
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
                signer: true,
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
                name: "member_account",
                writable: false,
                signer: false,
                init: false,
            },
            AccountSpec {
                name: "member_signer",
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
                writable: false,
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
                signer: true,
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
                writable: false,
                signer: false,
                init: false,
            },
            AccountSpec {
                name: "expense",
                writable: true,
                signer: true,
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
                writable: true,
                signer: false,
                init: false,
            },
            AccountSpec {
                name: "summary",
                writable: true,
                signer: true,
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
    InstructionSpec {
        name: "set_required_approvals",
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
        args: &[ArgSpec {
            name: "required_approvals",
            ty: ArgType::Scalar("u8"),
        }],
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
        Instruction::SetRequiredApprovals { .. } => "set_required_approvals",
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

    let types: serde_json::Map<String, serde_json::Value> = TYPE_DEFS
        .iter()
        .map(|(name, fields)| {
            (
                (*name).to_string(),
                serde_json::json!({
                    "kind": "struct",
                    "fields": fields.iter().map(|(field, ty)| serde_json::json!({
                        "name": field,
                        "type": ty,
                    })).collect::<Vec<_>>(),
                }),
            )
        })
        .collect();

    let document = serde_json::json!({
        "version": "0.3.0",
        "name": "sacco_ledger",
        "description": "Transparent SACCO attestation ledger: deposits, loans and expenses recorded and approved on chain; value stays off chain. Hand-maintained IDL table kept in lockstep with sacco_ledger_core::Instruction (see idl coverage tests).",
        "instructions": instructions,
        "account_types": ACCOUNT_TYPES,
        "types": types,
        "notes": NOTES,
    });

    let mut rendered = serde_json::to_string_pretty(&document).expect("IDL serializes");
    rendered.push('\n');
    rendered
}
