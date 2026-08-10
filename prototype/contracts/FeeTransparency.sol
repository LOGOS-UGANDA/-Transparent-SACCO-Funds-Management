// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title FeeTransparency - Logos Blockchain Fee Accountability System
 * @notice Locks institutional fees so they are only released for their stated purpose.
 *         Designed for universities (Makerere, Victoria), SACCOs, and cooperatives.
 */
contract FeeTransparency {

    // ──────────────────────────── Types ────────────────────────────

    enum InstitutionType { University, SACCO, Cooperative, School }
    enum FeeStatus       { Locked, PartiallyReleased, FullyReleased, Refunded }
    enum ReleaseStatus   { Pending, Approved, Rejected, Executed }

    struct Institution {
        uint256 id;
        string  name;
        string  registrationNumber;
        InstitutionType institutionType;
        address admin;
        address treasury;
        bool    isVerified;
        bool    isActive;
        uint256 totalFeesCollected;
        uint256 totalFeesReleased;
        uint256 createdAt;
    }

    struct FeeCategory {
        uint256 id;
        uint256 institutionId;
        string  name;
        string  description;
        uint256 allocatedPercentage; // basis points (100 = 1%)
        uint256 totalCollected;
        uint256 totalReleased;
        bool    isActive;
    }

    struct Payment {
        uint256 id;
        uint256 institutionId;
        address payer;
        string  payerName;
        string  referenceNumber;
        uint256 amount;
        uint256 timestamp;
        FeeStatus status;
        bool    isVerified;
    }

    struct FeeAllocation {
        uint256 paymentId;
        uint256 categoryId;
        uint256 amount;
        bool    released;
        uint256 releasedAt;
    }

    struct ReleaseRequest {
        uint256 id;
        uint256 institutionId;
        uint256 categoryId;
        uint256 amount;
        string  purpose;
        string  evidenceHash;
        address requestedBy;
        ReleaseStatus status;
        uint256 approvalCount;
        uint256 rejectionCount;
        uint256 createdAt;
        uint256 executedAt;
    }

    // ──────────────────────────── State ────────────────────────────

    address public owner;
    uint256 public requiredApprovals = 2;

    uint256 private _institutionCounter;
    uint256 private _categoryCounter;
    uint256 private _paymentCounter;
    uint256 private _releaseCounter;

    mapping(uint256 => Institution)    public institutions;
    mapping(uint256 => FeeCategory)    public feeCategories;
    mapping(uint256 => Payment)        public payments;
    mapping(uint256 => ReleaseRequest) public releaseRequests;

    mapping(uint256 => uint256[])      public institutionCategories;
    mapping(uint256 => FeeAllocation[]) public paymentAllocations;
    mapping(address => uint256[])      public payerPayments;
    mapping(uint256 => mapping(address => bool)) public releaseApprovals;
    mapping(uint256 => address[])      public institutionApprovers;

    uint256[] public allInstitutionIds;
    uint256[] public allPaymentIds;

    // ──────────────────────────── Events ────────────────────────────

    event InstitutionRegistered(uint256 indexed id, string name, InstitutionType institutionType);
    event InstitutionVerified(uint256 indexed id);
    event FeeCategoryCreated(uint256 indexed id, uint256 indexed institutionId, string name, uint256 percentage);
    event PaymentReceived(uint256 indexed id, uint256 indexed institutionId, address indexed payer, uint256 amount);
    event FeeAllocated(uint256 indexed paymentId, uint256 indexed categoryId, uint256 amount);
    event ReleaseRequested(uint256 indexed id, uint256 indexed institutionId, uint256 amount, string purpose);
    event ReleaseApproved(uint256 indexed id, address indexed approver);
    event ReleaseRejected(uint256 indexed id, address indexed approver);
    event FundsReleased(uint256 indexed releaseId, uint256 indexed institutionId, uint256 amount);
    event RefundIssued(uint256 indexed paymentId, address indexed payer, uint256 amount);
    event ApproverAdded(uint256 indexed institutionId, address indexed approver);

    // ──────────────────────────── Modifiers ────────────────────────────

    modifier onlyOwner() {
        require(msg.sender == owner, "Only contract owner");
        _;
    }

    modifier onlyInstitutionAdmin(uint256 _institutionId) {
        require(institutions[_institutionId].admin == msg.sender, "Only institution admin");
        _;
    }

    modifier institutionExists(uint256 _institutionId) {
        require(institutions[_institutionId].id != 0, "Institution not found");
        _;
    }

    modifier institutionActive(uint256 _institutionId) {
        require(institutions[_institutionId].isActive, "Institution not active");
        _;
    }

    // ──────────────────────────── Constructor ────────────────────────────

    constructor() {
        owner = msg.sender;
    }

    // ──────────────────────────── Institution Management ────────────────────────────

    function registerInstitution(
        string memory _name,
        string memory _registrationNumber,
        InstitutionType _type,
        address _treasury
    ) external returns (uint256) {
        _institutionCounter++;
        uint256 id = _institutionCounter;

        institutions[id] = Institution({
            id: id,
            name: _name,
            registrationNumber: _registrationNumber,
            institutionType: _type,
            admin: msg.sender,
            treasury: _treasury,
            isVerified: false,
            isActive: true,
            totalFeesCollected: 0,
            totalFeesReleased: 0,
            createdAt: block.timestamp
        });

        allInstitutionIds.push(id);
        institutionApprovers[id].push(msg.sender);

        emit InstitutionRegistered(id, _name, _type);
        return id;
    }

    function verifyInstitution(uint256 _institutionId)
        external
        onlyOwner
        institutionExists(_institutionId)
    {
        institutions[_institutionId].isVerified = true;
        emit InstitutionVerified(_institutionId);
    }

    function addApprover(uint256 _institutionId, address _approver)
        external
        onlyInstitutionAdmin(_institutionId)
    {
        institutionApprovers[_institutionId].push(_approver);
        emit ApproverAdded(_institutionId, _approver);
    }

    // ──────────────────────────── Fee Category Management ────────────────────────────

    function createFeeCategory(
        uint256 _institutionId,
        string memory _name,
        string memory _description,
        uint256 _allocatedPercentage
    )
        external
        onlyInstitutionAdmin(_institutionId)
        institutionActive(_institutionId)
        returns (uint256)
    {
        uint256 totalPercentage = _allocatedPercentage;
        uint256[] memory catIds = institutionCategories[_institutionId];
        for (uint256 i = 0; i < catIds.length; i++) {
            if (feeCategories[catIds[i]].isActive) {
                totalPercentage += feeCategories[catIds[i]].allocatedPercentage;
            }
        }
        require(totalPercentage <= 10000, "Total allocation exceeds 100%");

        _categoryCounter++;
        uint256 id = _categoryCounter;

        feeCategories[id] = FeeCategory({
            id: id,
            institutionId: _institutionId,
            name: _name,
            description: _description,
            allocatedPercentage: _allocatedPercentage,
            totalCollected: 0,
            totalReleased: 0,
            isActive: true
        });

        institutionCategories[_institutionId].push(id);

        emit FeeCategoryCreated(id, _institutionId, _name, _allocatedPercentage);
        return id;
    }

    // ──────────────────────────── Payment Processing ────────────────────────────

    function payFees(
        uint256 _institutionId,
        string memory _payerName,
        string memory _referenceNumber
    )
        external
        payable
        institutionExists(_institutionId)
        institutionActive(_institutionId)
        returns (uint256)
    {
        require(msg.value > 0, "Payment must be greater than 0");
        require(institutions[_institutionId].isVerified, "Institution not verified");

        _paymentCounter++;
        uint256 id = _paymentCounter;

        payments[id] = Payment({
            id: id,
            institutionId: _institutionId,
            payer: msg.sender,
            payerName: _payerName,
            referenceNumber: _referenceNumber,
            amount: msg.value,
            timestamp: block.timestamp,
            status: FeeStatus.Locked,
            isVerified: true
        });

        institutions[_institutionId].totalFeesCollected += msg.value;
        payerPayments[msg.sender].push(id);
        allPaymentIds.push(id);

        _allocateToCategories(id, _institutionId, msg.value);

        emit PaymentReceived(id, _institutionId, msg.sender, msg.value);
        return id;
    }

    function _allocateToCategories(
        uint256 _paymentId,
        uint256 _institutionId,
        uint256 _amount
    ) internal {
        uint256[] memory catIds = institutionCategories[_institutionId];

        for (uint256 i = 0; i < catIds.length; i++) {
            if (feeCategories[catIds[i]].isActive) {
                uint256 allocation = (_amount * feeCategories[catIds[i]].allocatedPercentage) / 10000;

                paymentAllocations[_paymentId].push(FeeAllocation({
                    paymentId: _paymentId,
                    categoryId: catIds[i],
                    amount: allocation,
                    released: false,
                    releasedAt: 0
                }));

                feeCategories[catIds[i]].totalCollected += allocation;
                emit FeeAllocated(_paymentId, catIds[i], allocation);
            }
        }
    }

    // ──────────────────────────── Fund Release (Multi-sig) ────────────────────────────

    function requestRelease(
        uint256 _institutionId,
        uint256 _categoryId,
        uint256 _amount,
        string memory _purpose,
        string memory _evidenceHash
    )
        external
        onlyInstitutionAdmin(_institutionId)
        returns (uint256)
    {
        require(feeCategories[_categoryId].institutionId == _institutionId, "Category mismatch");
        uint256 available = feeCategories[_categoryId].totalCollected - feeCategories[_categoryId].totalReleased;
        require(_amount <= available, "Insufficient locked funds");

        _releaseCounter++;
        uint256 id = _releaseCounter;

        releaseRequests[id] = ReleaseRequest({
            id: id,
            institutionId: _institutionId,
            categoryId: _categoryId,
            amount: _amount,
            purpose: _purpose,
            evidenceHash: _evidenceHash,
            requestedBy: msg.sender,
            status: ReleaseStatus.Pending,
            approvalCount: 0,
            rejectionCount: 0,
            createdAt: block.timestamp,
            executedAt: 0
        });

        emit ReleaseRequested(id, _institutionId, _amount, _purpose);
        return id;
    }

    function approveRelease(uint256 _releaseId) external {
        ReleaseRequest storage req = releaseRequests[_releaseId];
        require(req.id != 0, "Release request not found");
        require(req.status == ReleaseStatus.Pending, "Not pending");
        require(!releaseApprovals[_releaseId][msg.sender], "Already voted");
        require(_isApprover(req.institutionId, msg.sender), "Not an approver");

        releaseApprovals[_releaseId][msg.sender] = true;
        req.approvalCount++;

        emit ReleaseApproved(_releaseId, msg.sender);

        if (req.approvalCount >= requiredApprovals) {
            _executeRelease(_releaseId);
        }
    }

    function rejectRelease(uint256 _releaseId) external {
        ReleaseRequest storage req = releaseRequests[_releaseId];
        require(req.id != 0, "Release request not found");
        require(req.status == ReleaseStatus.Pending, "Not pending");
        require(!releaseApprovals[_releaseId][msg.sender], "Already voted");
        require(_isApprover(req.institutionId, msg.sender), "Not an approver");

        releaseApprovals[_releaseId][msg.sender] = true;
        req.rejectionCount++;

        emit ReleaseRejected(_releaseId, msg.sender);

        if (req.rejectionCount >= requiredApprovals) {
            req.status = ReleaseStatus.Rejected;
        }
    }

    function _executeRelease(uint256 _releaseId) internal {
        ReleaseRequest storage req = releaseRequests[_releaseId];
        req.status = ReleaseStatus.Executed;
        req.executedAt = block.timestamp;

        feeCategories[req.categoryId].totalReleased += req.amount;
        institutions[req.institutionId].totalFeesReleased += req.amount;

        address treasury = institutions[req.institutionId].treasury;
        (bool sent, ) = treasury.call{value: req.amount}("");
        require(sent, "Transfer failed");

        emit FundsReleased(_releaseId, req.institutionId, req.amount);
    }

    function _isApprover(uint256 _institutionId, address _addr) internal view returns (bool) {
        address[] memory approvers = institutionApprovers[_institutionId];
        for (uint256 i = 0; i < approvers.length; i++) {
            if (approvers[i] == _addr) return true;
        }
        return _addr == owner;
    }

    // ──────────────────────────── View Functions ────────────────────────────

    function getInstitution(uint256 _id) external view returns (Institution memory) {
        return institutions[_id];
    }

    function getPayment(uint256 _id) external view returns (Payment memory) {
        return payments[_id];
    }

    function getPaymentAllocations(uint256 _paymentId) external view returns (FeeAllocation[] memory) {
        return paymentAllocations[_paymentId];
    }

    function getInstitutionCategories(uint256 _institutionId) external view returns (uint256[] memory) {
        return institutionCategories[_institutionId];
    }

    function getPayerPayments(address _payer) external view returns (uint256[] memory) {
        return payerPayments[_payer];
    }

    function getAllInstitutionIds() external view returns (uint256[] memory) {
        return allInstitutionIds;
    }

    function getAllPaymentIds() external view returns (uint256[] memory) {
        return allPaymentIds;
    }

    function getInstitutionApprovers(uint256 _institutionId) external view returns (address[] memory) {
        return institutionApprovers[_institutionId];
    }

    function getReleaseRequest(uint256 _id) external view returns (ReleaseRequest memory) {
        return releaseRequests[_id];
    }

    function getTotalInstitutions() external view returns (uint256) {
        return _institutionCounter;
    }

    function getTotalPayments() external view returns (uint256) {
        return _paymentCounter;
    }

    function getContractBalance() external view returns (uint256) {
        return address(this).balance;
    }

    /**
     * @notice Verify a payment by its reference number — the core transparency feature.
     *         Any payer can call this to see exactly where their money is allocated.
     */
    function verifyPaymentByRef(uint256 _paymentId)
        external
        view
        returns (
            Payment memory payment,
            FeeAllocation[] memory allocations,
            Institution memory institution
        )
    {
        payment = payments[_paymentId];
        require(payment.id != 0, "Payment not found");
        allocations = paymentAllocations[_paymentId];
        institution = institutions[payment.institutionId];
    }

    // ──────────────────────────── Admin ────────────────────────────

    function setRequiredApprovals(uint256 _count) external onlyOwner {
        require(_count > 0, "Must require at least 1 approval");
        requiredApprovals = _count;
    }

    receive() external payable {}
}
