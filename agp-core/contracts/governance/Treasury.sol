// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";

/**
 * @title Treasury
 * @dev Multi-sig treasury management for protocol funds
 */
contract Treasury is AccessControl, ReentrancyGuard {
    using SafeERC20 for IERC20;

    bytes32 public constant EXECUTOR_ROLE = keccak256("EXECUTOR_ROLE");
    bytes32 public constant PROPOSER_ROLE = keccak256("PROPOSER_ROLE");

    struct Proposal {
        address target;
        uint256 value;
        bytes data;
        string description;
        uint256 approvals;
        uint256 deadline;
        bool executed;
        mapping(address => bool) hasApproved;
    }

    uint256 public proposalCount;
    uint256 public requiredApprovals;
    uint256 public proposalDuration = 7 days;
    
    mapping(uint256 => Proposal) public proposals;
    mapping(address => uint256) public tokenBudgets;

    event ProposalCreated(uint256 indexed id, address target, uint256 value, string description);
    event ProposalApproved(uint256 indexed id, address approver);
    event ProposalExecuted(uint256 indexed id);
    event FundsReceived(address indexed token, uint256 amount);
    event BudgetSet(address indexed token, uint256 amount);

    constructor(address[] memory executors, uint256 _requiredApprovals) {
        require(executors.length >= _requiredApprovals, "Invalid threshold");
        
        _setupRole(DEFAULT_ADMIN_ROLE, msg.sender);
        
        for (uint256 i = 0; i < executors.length; i++) {
            _setupRole(EXECUTOR_ROLE, executors[i]);
            _setupRole(PROPOSER_ROLE, executors[i]);
        }
        
        requiredApprovals = _requiredApprovals;
    }

    receive() external payable {
        emit FundsReceived(address(0), msg.value);
    }

    function depositToken(address token, uint256 amount) external {
        IERC20(token).safeTransferFrom(msg.sender, address(this), amount);
        emit FundsReceived(token, amount);
    }

    function propose(
        address target,
        uint256 value,
        bytes calldata data,
        string calldata description
    ) external onlyRole(PROPOSER_ROLE) returns (uint256) {
        uint256 id = proposalCount++;
        
        Proposal storage p = proposals[id];
        p.target = target;
        p.value = value;
        p.data = data;
        p.description = description;
        p.deadline = block.timestamp + proposalDuration;
        
        emit ProposalCreated(id, target, value, description);
        return id;
    }

    function approve(uint256 proposalId) external onlyRole(EXECUTOR_ROLE) {
        Proposal storage p = proposals[proposalId];
        
        require(block.timestamp < p.deadline, "Proposal expired");
        require(!p.executed, "Already executed");
        require(!p.hasApproved[msg.sender], "Already approved");

        p.hasApproved[msg.sender] = true;
        p.approvals++;

        emit ProposalApproved(proposalId, msg.sender);
    }

    function execute(uint256 proposalId) external nonReentrant onlyRole(EXECUTOR_ROLE) {
        Proposal storage p = proposals[proposalId];
        
        require(p.approvals >= requiredApprovals, "Insufficient approvals");
        require(!p.executed, "Already executed");

        p.executed = true;

        (bool success, ) = p.target.call{value: p.value}(p.data);
        require(success, "Execution failed");

        emit ProposalExecuted(proposalId);
    }

    function setBudget(address token, uint256 amount) external onlyRole(DEFAULT_ADMIN_ROLE) {
        tokenBudgets[token] = amount;
        emit BudgetSet(token, amount);
    }

    function transferToken(address token, address to, uint256 amount) external onlyRole(EXECUTOR_ROLE) {
        require(amount <= tokenBudgets[token], "Exceeds budget");
        tokenBudgets[token] -= amount;
        IERC20(token).safeTransfer(to, amount);
    }

    function getBalance(address token) external view returns (uint256) {
        if (token == address(0)) {
            return address(this).balance;
        }
        return IERC20(token).balanceOf(address(this));
    }

    function updateRequiredApprovals(uint256 _required) external onlyRole(DEFAULT_ADMIN_ROLE) {
        requiredApprovals = _required;
    }
}
