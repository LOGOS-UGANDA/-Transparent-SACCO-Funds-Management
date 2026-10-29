#pragma once

#include <string>
#include "logos_module_context.h"

/**
 * Transparent SACCO core module — wiring for the `sacco_ledger` LEZ program.
 *
 * One method per instruction of `artifacts/sacco_ledger-idl.json` (plus the
 * read-only queries the UI views use). Each mutation method builds the exact
 * instruction payload (field names match the `Instruction` enum) and hands it
 * to dispatchInstruction(). The ledger attests to off-chain activity; it
 * never holds or transfers SACCO funds.
 */
class SaccoCoreImpl : public LogosModuleContext
{
public:
    /// Health check for Basecamp / standalone load verification.
    std::string getStatus();
    std::string setLedgerContext(const std::string& programId,
                                 const std::string& saccoAccount,
                                 const std::string& feePayerId,
                                 const std::string& signerId);
    std::string createPublicAccount();

    // ── Setup ────────────────────────────────────────────────────────────
    std::string initialize();
    std::string registerSacco(const std::string& name,
                              const std::string& registrationNumber,
                              const std::string& treasury,
                              const std::string& saccoAccount);
    std::string verifySacco();
    std::string addApprover(const std::string& approver);
    std::string setRequiredApprovals(const std::string& requiredApprovals);
    std::string addSupervisor(const std::string& supervisor);
    std::string addMember(const std::string& memberAccount, const std::string& identity);

    // ── Deposits ─────────────────────────────────────────────────────────
    std::string recordDeposit(const std::string& memberAccount,
                              const std::string& depositAccount,
                              const std::string& amount,
                              const std::string& depositType,
                              const std::string& reference);
    std::string confirmDeposit(const std::string& depositAccount,
                               const std::string& memberAccount);

    // ── Loans ────────────────────────────────────────────────────────────
    std::string applyLoan(const std::string& memberAccount,
                          const std::string& loanAccount,
                          const std::string& amount,
                          const std::string& purpose);
    std::string approveLoan(const std::string& loanAccount);
    std::string rejectLoan(const std::string& loanAccount);
    std::string disburseLoan(const std::string& memberAccount,
                             const std::string& loanAccount,
                             const std::string& evidenceCid,
                             const std::string& principal);
    std::string recordRepayment(const std::string& memberAccount,
                                const std::string& loanAccount,
                                const std::string& amount,
                                const std::string& reference);

    // ── Expenses ─────────────────────────────────────────────────────────
    std::string proposeExpense(const std::string& expenseAccount,
                               const std::string& amount,
                               const std::string& purpose,
                               const std::string& evidenceCid,
                               const std::string& requiredApprovals);
    std::string approveExpense(const std::string& expenseAccount);
    std::string rejectExpense(const std::string& expenseAccount);
    std::string executeExpense(const std::string& expenseAccount,
                               const std::string& amount,
                               const std::string& evidenceReference);

    // ── Reporting ────────────────────────────────────────────────────────
    std::string postPeriodSummary(const std::string& summaryAccount,
                                  const std::string& label,
                                  const std::string& totalsJson);

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
    /// Emitted when an instruction is queued (UI feedback).
    void operationSubmitted(const std::string& operation, const std::string& detail);

private:
    /// Single dispatch point: builds the LEZ call for the named instruction with
    /// the given borsh-shaped field map. Returns the transaction result JSON.
    std::string dispatchInstruction(const std::string& instruction,
                                    const std::string& fieldsJson);
    std::string readIndexedAccount(const std::string& kind,
                                   const std::string& accountId);
    std::string programId_;
    std::string saccoAccount_;
    std::string feePayerId_;
    std::string signerId_;
};
