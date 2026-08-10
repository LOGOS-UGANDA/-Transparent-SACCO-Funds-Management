#pragma once

#include <string>
#include "logos_module_context.h"

/**
 * Transparent SACCO core module.
 *
 * Exposes product operations to UI modules / CLI. LEZ wallet / program calls
 * will be wired here once the fee_transparency program is deployed.
 */
class SaccoCoreImpl : public LogosModuleContext
{
public:
    /// Health check for Basecamp / standalone load verification.
    std::string getStatus();

    /// Scaffold: returns a JSON-shaped summary describing the planned LEZ call.
    std::string registerInstitution(
        const std::string& name,
        const std::string& registrationNumber,
        const std::string& institutionType,
        const std::string& treasuryAccountId);

    /// Scaffold: describe a pay-fees LEZ call.
    std::string payFees(
        const std::string& institutionAccountId,
        const std::string& payerName,
        const std::string& referenceNumber,
        const std::string& amount);

    /// Scaffold: look up / verify a payment by reference (client-side index later).
    std::string verifyPaymentByRef(const std::string& referenceNumber);

    /// Scaffold: request a multi-sig release.
    std::string requestRelease(
        const std::string& institutionAccountId,
        const std::string& categoryAccountId,
        const std::string& amount,
        const std::string& purpose,
        const std::string& evidenceHash);

    /// Scaffold: approve a pending release.
    std::string approveRelease(const std::string& releaseAccountId);

logos_events:
    /// Emitted when a scaffold API is invoked (useful for UI feedback).
    void operationQueued(const std::string& operation, const std::string& detail);
};
