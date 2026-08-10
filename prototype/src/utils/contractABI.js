export const CONTRACT_ADDRESS = import.meta.env.VITE_CONTRACT_ADDRESS || '0x0000000000000000000000000000000000000000';

export const FEE_TRANSPARENCY_ABI = [
  "function registerInstitution(string _name, string _registrationNumber, uint8 _type, address _treasury) external returns (uint256)",
  "function verifyInstitution(uint256 _institutionId) external",
  "function addApprover(uint256 _institutionId, address _approver) external",
  "function createFeeCategory(uint256 _institutionId, string _name, string _description, uint256 _allocatedPercentage) external returns (uint256)",
  "function payFees(uint256 _institutionId, string _payerName, string _referenceNumber) external payable returns (uint256)",
  "function requestRelease(uint256 _institutionId, uint256 _categoryId, uint256 _amount, string _purpose, string _evidenceHash) external returns (uint256)",
  "function approveRelease(uint256 _releaseId) external",
  "function rejectRelease(uint256 _releaseId) external",
  "function getInstitution(uint256 _id) external view returns (tuple(uint256 id, string name, string registrationNumber, uint8 institutionType, address admin, address treasury, bool isVerified, bool isActive, uint256 totalFeesCollected, uint256 totalFeesReleased, uint256 createdAt))",
  "function getPayment(uint256 _id) external view returns (tuple(uint256 id, uint256 institutionId, address payer, string payerName, string referenceNumber, uint256 amount, uint256 timestamp, uint8 status, bool isVerified))",
  "function getPaymentAllocations(uint256 _paymentId) external view returns (tuple(uint256 paymentId, uint256 categoryId, uint256 amount, bool released, uint256 releasedAt)[])",
  "function getInstitutionCategories(uint256 _institutionId) external view returns (uint256[])",
  "function getPayerPayments(address _payer) external view returns (uint256[])",
  "function getAllInstitutionIds() external view returns (uint256[])",
  "function getAllPaymentIds() external view returns (uint256[])",
  "function verifyPaymentByRef(uint256 _paymentId) external view returns (tuple(uint256 id, uint256 institutionId, address payer, string payerName, string referenceNumber, uint256 amount, uint256 timestamp, uint8 status, bool isVerified), tuple(uint256 paymentId, uint256 categoryId, uint256 amount, bool released, uint256 releasedAt)[], tuple(uint256 id, string name, string registrationNumber, uint8 institutionType, address admin, address treasury, bool isVerified, bool isActive, uint256 totalFeesCollected, uint256 totalFeesReleased, uint256 createdAt))",
  "function getTotalInstitutions() external view returns (uint256)",
  "function getTotalPayments() external view returns (uint256)",
  "function getContractBalance() external view returns (uint256)",
  "function feeCategories(uint256) external view returns (uint256 id, uint256 institutionId, string name, string description, uint256 allocatedPercentage, uint256 totalCollected, uint256 totalReleased, bool isActive)",
  "function getReleaseRequest(uint256 _id) external view returns (tuple(uint256 id, uint256 institutionId, uint256 categoryId, uint256 amount, string purpose, string evidenceHash, address requestedBy, uint8 status, uint256 approvalCount, uint256 rejectionCount, uint256 createdAt, uint256 executedAt))",
  "event InstitutionRegistered(uint256 indexed id, string name, uint8 institutionType)",
  "event PaymentReceived(uint256 indexed id, uint256 indexed institutionId, address indexed payer, uint256 amount)",
  "event FundsReleased(uint256 indexed releaseId, uint256 indexed institutionId, uint256 amount)",
  "event ReleaseRequested(uint256 indexed id, uint256 indexed institutionId, uint256 amount, string purpose)"
];
