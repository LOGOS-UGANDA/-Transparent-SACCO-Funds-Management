#pragma once

#include <string>
#include "logos_module_context.h"

/**
 * Transparent SACCO core module.
 *
 * Scaffold APIs for UI modules / CLI. These placeholder methods are not yet
 * wired to the sacco_ledger IDL; the product records off-chain SACCO activity
 * and does not custody or transfer SACCO funds.
 */
class SaccoCoreImpl : public LogosModuleContext
{
public:
    /// Health check for Basecamp / standalone load verification.
    std::string getStatus();

    /// Legacy fee-transparency scaffold; not part of the current ledger API.
    std::string registerInstitution(
        const std::string& name,
        const std::string& registrationNumber,
        const std::string& institutionType,
        const std::string& treasuryAccountId);

    /// Legacy fee-transparency scaffold; does not submit a transfer or LEZ call.
    std::string payFees(
        const std::string& institutionAccountId,
        const std::string& payerName,
        const std::string& referenceNumber,
        const std::string& amount);

    /// Legacy fee-transparency scaffold; not part of the current ledger API.
    std::string verifyPaymentByRef(const std::string& referenceNumber);

    /// Legacy fee-transparency scaffold; does not release funds.
    std::string requestRelease(
        const std::string& institutionAccountId,
        const std::string& categoryAccountId,
        const std::string& amount,
        const std::string& purpose,
        const std::string& evidenceHash);

    /// Legacy fee-transparency scaffold; does not release funds.
    std::string approveRelease(const std::string& releaseAccountId);

logos_events:
    /// Emitted when a scaffold API is invoked (useful for UI feedback).
    void operationQueued(const std::string& operation, const std::string& detail);
};
