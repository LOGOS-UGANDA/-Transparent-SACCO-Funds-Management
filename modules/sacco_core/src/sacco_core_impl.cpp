#include "sacco_core_impl.h"

#include <sstream>

namespace {

/// JSON-escape a string value for payload construction.
std::string jstr(const std::string& value) {
    std::ostringstream out;
    out << '"';
    for (char c : value) {
        switch (c) {
            case '"': out << "\\\""; break;
            case '\\': out << "\\\\"; break;
            case '\n': out << "\\n"; break;
            default: out << c;
        }
    }
    out << '"';
    return out.str();
}

std::string ok(const std::string& instruction, const std::string& fields) {
    std::ostringstream out;
    out << "{\"status\":\"queued\",\"instruction\":" << jstr(instruction)
        << ",\"fields\":" << fields << "}";
    return out.str();
}

std::string ok(const std::string& instruction, const std::string& fields,
               const std::string& programStatus) {
    std::ostringstream out;
    out << "{\"status\":" << jstr(programStatus)
        << ",\"instruction\":" << jstr(instruction)
        << ",\"fields\":" << fields << "}";
    return out.str();
}

}  // namespace

std::string SaccoCoreImpl::getStatus() {
    return "{\"module\":\"sacco_core\",\"api\":\"sacco_ledger\",\"instructions\":18}";
}

// ── Setup ────────────────────────────────────────────────────────────────

std::string SaccoCoreImpl::initialize() {
    emit operationQueued("initialize", "{}");
    return dispatchInstruction("Initialize", "{}");
}

std::string SaccoCoreImpl::registerSacco(const std::string& name,
                                         const std::string& registrationNumber,
                                         const std::string& treasury) {
    const std::string fields = "{\"name\":" + jstr(name) +
                               ",\"registration_number\":" + jstr(registrationNumber) +
                               ",\"treasury\":" + jstr(treasury) + "}";
    emit operationQueued("register_sacco", fields);
    return dispatchInstruction("RegisterSacco", fields);
}

std::string SaccoCoreImpl::verifySacco() {
    emit operationQueued("verify_sacco", "{}");
    return dispatchInstruction("VerifySacco", "{}");
}

std::string SaccoCoreImpl::addApprover(const std::string& approver) {
    const std::string fields = "{\"approver\":" + jstr(approver) + "}";
    emit operationQueued("add_approver", fields);
    return dispatchInstruction("AddApprover", fields);
}

std::string SaccoCoreImpl::addSupervisor(const std::string& supervisor) {
    const std::string fields = "{\"supervisor\":" + jstr(supervisor) + "}";
    emit operationQueued("add_supervisor", fields);
    return dispatchInstruction("AddSupervisor", fields);
}

std::string SaccoCoreImpl::addMember(const std::string& memberAccount,
                                     const std::string& identity) {
    const std::string fields = "{\"identity\":" + jstr(identity) +
                               ",\"member_account\":" + jstr(memberAccount) + "}";
    emit operationQueued("add_member", fields);
    return dispatchInstruction("AddMember", fields);
}

// ── Deposits ─────────────────────────────────────────────────────────────

std::string SaccoCoreImpl::recordDeposit(const std::string& memberAccount,
                                         const std::string& amount,
                                         const std::string& depositType,
                                         const std::string& reference) {
    const std::string fields = "{\"member\":" + jstr(memberAccount) +
                               ",\"amount\":" + amount +
                               ",\"deposit_type\":" + jstr(depositType) +
                               ",\"reference\":" + jstr(reference) + "}";
    emit operationQueued("record_deposit", fields);
    return dispatchInstruction("RecordDeposit", fields);
}

std::string SaccoCoreImpl::confirmDeposit(const std::string& depositAccount) {
    const std::string fields = "{\"deposit_account\":" + jstr(depositAccount) + "}";
    emit operationQueued("confirm_deposit", fields);
    return dispatchInstruction("ConfirmDeposit", fields);
}

// ── Loans ────────────────────────────────────────────────────────────────

std::string SaccoCoreImpl::applyLoan(const std::string& amount, const std::string& purpose) {
    const std::string fields = "{\"amount\":" + amount +
                               ",\"purpose\":" + jstr(purpose) + "}";
    emit operationQueued("apply_loan", fields);
    return dispatchInstruction("ApplyLoan", fields);
}

std::string SaccoCoreImpl::approveLoan(const std::string& loanAccount) {
    const std::string fields = "{\"loan_account\":" + jstr(loanAccount) + "}";
    emit operationQueued("approve_loan", fields);
    return dispatchInstruction("ApproveLoan", fields);
}

std::string SaccoCoreImpl::rejectLoan(const std::string& loanAccount) {
    const std::string fields = "{\"loan_account\":" + jstr(loanAccount) + "}";
    emit operationQueued("reject_loan", fields);
    return dispatchInstruction("RejectLoan", fields);
}

std::string SaccoCoreImpl::disburseLoan(const std::string& loanAccount,
                                        const std::string& evidenceCid) {
    const std::string fields = "{\"loan_account\":" + jstr(loanAccount) +
                               ",\"evidence_cid\":" + jstr(evidenceCid) + "}";
    emit operationQueued("disburse_loan", fields);
    return dispatchInstruction("DisburseLoan", fields);
}

std::string SaccoCoreImpl::recordRepayment(const std::string& loanAccount,
                                           const std::string& amount) {
    const std::string fields = "{\"loan_account\":" + jstr(loanAccount) +
                               ",\"amount\":" + amount + "}";
    emit operationQueued("record_repayment", fields);
    return dispatchInstruction("RecordRepayment", fields);
}

// ── Expenses ─────────────────────────────────────────────────────────────

std::string SaccoCoreImpl::proposeExpense(const std::string& amount,
                                          const std::string& purpose,
                                          const std::string& evidenceCid) {
    const std::string fields = "{\"amount\":" + amount +
                               ",\"purpose\":" + jstr(purpose) +
                               ",\"evidence_cid\":" + jstr(evidenceCid) + "}";
    emit operationQueued("propose_expense", fields);
    return dispatchInstruction("ProposeExpense", fields);
}

std::string SaccoCoreImpl::approveExpense(const std::string& expenseAccount) {
    const std::string fields = "{\"expense_account\":" + jstr(expenseAccount) + "}";
    emit operationQueued("approve_expense", fields);
    return dispatchInstruction("ApproveExpense", fields);
}

std::string SaccoCoreImpl::rejectExpense(const std::string& expenseAccount) {
    const std::string fields = "{\"expense_account\":" + jstr(expenseAccount) + "}";
    emit operationQueued("reject_expense", fields);
    return dispatchInstruction("RejectExpense", fields);
}

std::string SaccoCoreImpl::executeExpense(const std::string& expenseAccount) {
    const std::string fields = "{\"expense_account\":" + jstr(expenseAccount) + "}";
    emit operationQueued("execute_expense", fields);
    return dispatchInstruction("ExecuteExpense", fields);
}

// ── Reporting ────────────────────────────────────────────────────────────

std::string SaccoCoreImpl::postPeriodSummary(const std::string& label) {
    const std::string fields = "{\"label\":" + jstr(label) + "}";
    emit operationQueued("post_period_summary", fields);
    return dispatchInstruction("PostPeriodSummary", fields);
}

// ── Read-only queries ────────────────────────────────────────────────────
// Backed by the LEZ indexer once the program id is recorded; until then each
// query reports the account it would read so the views stay testable.

std::string SaccoCoreImpl::getSacco() {
    return ok("Query::Sacco", "{}", "unavailable_pending_deploy");
}

std::string SaccoCoreImpl::getMember(const std::string& memberAccount) {
    return ok("Query::Member", "{\"account\":" + jstr(memberAccount) + "}", "unavailable_pending_deploy");
}

std::string SaccoCoreImpl::getDeposit(const std::string& depositAccount) {
    return ok("Query::Deposit", "{\"account\":" + jstr(depositAccount) + "}", "unavailable_pending_deploy");
}

std::string SaccoCoreImpl::getLoan(const std::string& loanAccount) {
    return ok("Query::Loan", "{\"account\":" + jstr(loanAccount) + "}", "unavailable_pending_deploy");
}

std::string SaccoCoreImpl::getExpense(const std::string& expenseAccount) {
    return ok("Query::Expense", "{\"account\":" + jstr(expenseAccount) + "}", "unavailable_pending_deploy");
}

std::string SaccoCoreImpl::listMyDeposits(const std::string& memberAccount) {
    return ok("Query::MyDeposits", "{\"member\":" + jstr(memberAccount) + "}", "unavailable_pending_deploy");
}

std::string SaccoCoreImpl::listMyLoans(const std::string& memberAccount) {
    return ok("Query::MyLoans", "{\"member\":" + jstr(memberAccount) + "}", "unavailable_pending_deploy");
}

std::string SaccoCoreImpl::listPeriodSummaries() {
    return ok("Query::PeriodSummaries", "{}", "unavailable_pending_deploy");
}

std::string SaccoCoreImpl::getAuditTrail() {
    return ok("Query::AuditTrail", "{}", "unavailable_pending_deploy");
}

// ── Dispatch ─────────────────────────────────────────────────────────────

std::string SaccoCoreImpl::dispatchInstruction(const std::string& instruction,
                                               const std::string& fieldsJson) {
    // Single wiring point for the LEZ wallet transport: send (program_id,
    // instruction, fieldsJson) once the deployed program id is recorded in
    // docs/deploy.md. Until then the call is queued, not sent — money never
    // moves through this module in any state.
    return ok(instruction, fieldsJson, "queued_pending_deploy");
}
