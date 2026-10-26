#pragma once

#include <string>
#include "logos_module_context.h"

/**
 * Transparent SACCO core module — wiring for the `sacco_ledger` LEZ program.
 *
 * One method per instruction of `artifacts/sacco_ledger-idl.json` (plus the
 * read-only queries the UI views use). Each mutation method builds the exact
 * instruction payload (field names match the `Instruction` enum) and hands it
 * to dispatchInstruction(); the LEZ wallet transport plugs in there once the
 * deployed program id is recorded in docs/deploy.md.
 */
class SaccoCoreImpl : public LogosModuleContext
{
public:
    /// Health check for Basecamp / standalone load verification.
    std::string getStatus();

    // ── Setup ────────────────────────────────────────────────────────────
    std::string initialize();
    std::string registerSacco(const std::string& name,
                              const std::string& registrationNumber,
                              const std::string& treasury);
    std::string verifySacco();
    std::string addApprover(const std::string& approver);
    std::string addSupervisor(const std::string& supervisor);
    std::string addMember(const std::string& memberAccount, const std::string& identity);

    // ── Deposits ─────────────────────────────────────────────────────────
    std::string recordDeposit(const std::string& memberAccount,
                              const std::string& amount,
                              const std::string& depositType,   // "savings" | "shareCapital"
                              const std::string& reference);
    std::string confirmDeposit(const std::string& depositAccount);

    // ── Loans ────────────────────────────────────────────────────────────
    std::string applyLoan(const std::string& amount, const std::string& purpose);
    std::string approveLoan(const std::string& loanAccount);
    std::string rejectLoan(const std::string& loanAccount);
    std::string disburseLoan(const std::string& loanAccount, const std::string& evidenceCid);
    std::string recordRepayment(const std::string& loanAccount, const std::string& amount);

    // ── Expenses ─────────────────────────────────────────────────────────
    std::string proposeExpense(const std::string& amount,
                               const std::string& purpose,
                               const std::string& evidenceCid);
    std::string approveExpense(const std::string& expenseAccount);
    std::string rejectExpense(const std::string& expenseAccount);
    std::string executeExpense(const std::string& expenseAccount);

    // ── Reporting ────────────────────────────────────────────────────────
    std::string postPeriodSummary(const std::string& label);

    // ── Read-only queries (module-local; used by the QML views) ─────────
    std::string getSacco();
    std::string getMember(const std::string& memberAccount);
    std::string getDeposit(const std::string& depositAccount);
    std::string getLoan(const std::string& loanAccount);
    std::string getExpense(const std::string& expenseAccount);
    std::string listMyDeposits(const std::string& memberAccount);
    std::string listMyLoans(const std::string& memberAccount);
    std::string listPeriodSummaries();
    std::string getAuditTrail();

logos_events:
    /// Emitted on every dispatched instruction (UI feedback / audit trail feed).
    void operationQueued(const std::string& operation, const std::string& detail);

private:
    /// Single dispatch point: builds the LEZ call for the named instruction with
    /// the given borsh-shaped field map. Returns the transaction result JSON.
    std::string dispatchInstruction(const std::string& instruction,
                                    const std::string& fieldsJson);
};
