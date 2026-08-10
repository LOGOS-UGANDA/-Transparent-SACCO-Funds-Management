#include "sacco_core_impl.h"

#include <sstream>

namespace {

std::string queued(const std::string& op, const std::string& detail)
{
    std::ostringstream out;
    out << "{\"status\":\"scaffold\",\"operation\":\"" << op
        << "\",\"detail\":\"" << detail << "\"}";
    return out.str();
}

} // namespace

std::string SaccoCoreImpl::getStatus()
{
    return "sacco_core scaffold ready — LEZ fee_transparency wiring pending";
}

std::string SaccoCoreImpl::registerInstitution(
    const std::string& name,
    const std::string& registrationNumber,
    const std::string& institutionType,
    const std::string& treasuryAccountId)
{
    std::ostringstream detail;
    detail << name << "|" << registrationNumber << "|" << institutionType
           << "|" << treasuryAccountId;
    const std::string payload = queued("registerInstitution", detail.str());
    operationQueued("registerInstitution", payload);
    return payload;
}

std::string SaccoCoreImpl::payFees(
    const std::string& institutionAccountId,
    const std::string& payerName,
    const std::string& referenceNumber,
    const std::string& amount)
{
    std::ostringstream detail;
    detail << institutionAccountId << "|" << payerName << "|"
           << referenceNumber << "|" << amount;
    const std::string payload = queued("payFees", detail.str());
    operationQueued("payFees", payload);
    return payload;
}

std::string SaccoCoreImpl::verifyPaymentByRef(const std::string& referenceNumber)
{
    const std::string payload = queued("verifyPaymentByRef", referenceNumber);
    operationQueued("verifyPaymentByRef", payload);
    return payload;
}

std::string SaccoCoreImpl::requestRelease(
    const std::string& institutionAccountId,
    const std::string& categoryAccountId,
    const std::string& amount,
    const std::string& purpose,
    const std::string& evidenceHash)
{
    std::ostringstream detail;
    detail << institutionAccountId << "|" << categoryAccountId << "|"
           << amount << "|" << purpose << "|" << evidenceHash;
    const std::string payload = queued("requestRelease", detail.str());
    operationQueued("requestRelease", payload);
    return payload;
}

std::string SaccoCoreImpl::approveRelease(const std::string& releaseAccountId)
{
    const std::string payload = queued("approveRelease", releaseAccountId);
    operationQueued("approveRelease", payload);
    return payload;
}
