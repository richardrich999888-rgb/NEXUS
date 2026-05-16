// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";

/**
 * @title ReputationBridge
 * @dev Cross-chain bridge for reputation tokens with message passing
 */
contract ReputationBridge is AccessControl, Pausable, ReentrancyGuard {
    using SafeERC20 for IERC20;

    bytes32 public constant RELAYER_ROLE = keccak256("RELAYER_ROLE");
    bytes32 public constant PAUSER_ROLE = keccak256("PAUSER_ROLE");

    struct BridgeRequest {
        address sender;
        address recipient;
        uint256 amount;
        uint256 targetChainId;
        bytes32 messageHash;
        uint256 timestamp;
        bool processed;
    }

    IERC20 public token;
    uint256 public localChainId;
    uint256 public nonce;
    uint256 public bridgeFee = 0.001 ether;
    uint256 public minBridgeAmount = 100 * 1e18;
    uint256 public maxBridgeAmount = 1000000 * 1e18;

    mapping(bytes32 => BridgeRequest) public requests;
    mapping(bytes32 => bool) public processedMessages;
    mapping(uint256 => address) public remoteBridges;
    mapping(uint256 => bool) public supportedChains;

    event BridgeInitiated(
        bytes32 indexed requestId,
        address indexed sender,
        address recipient,
        uint256 amount,
        uint256 targetChainId
    );
    event BridgeCompleted(bytes32 indexed requestId, address indexed recipient, uint256 amount);
    event ChainSupported(uint256 indexed chainId, address bridgeAddress);
    event FeeUpdated(uint256 newFee);

    constructor(address _token, uint256 _chainId) {
        token = IERC20(_token);
        localChainId = _chainId;
        
        _setupRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _setupRole(RELAYER_ROLE, msg.sender);
        _setupRole(PAUSER_ROLE, msg.sender);
    }

    function bridge(
        address recipient,
        uint256 amount,
        uint256 targetChainId
    ) external payable nonReentrant whenNotPaused returns (bytes32) {
        require(msg.value >= bridgeFee, "Insufficient fee");
        require(amount >= minBridgeAmount, "Below minimum");
        require(amount <= maxBridgeAmount, "Above maximum");
        require(supportedChains[targetChainId], "Chain not supported");
        require(recipient != address(0), "Invalid recipient");

        bytes32 requestId = keccak256(abi.encodePacked(
            msg.sender,
            recipient,
            amount,
            targetChainId,
            nonce++,
            block.timestamp
        ));

        bytes32 messageHash = keccak256(abi.encodePacked(
            requestId,
            localChainId,
            targetChainId,
            amount
        ));

        requests[requestId] = BridgeRequest({
            sender: msg.sender,
            recipient: recipient,
            amount: amount,
            targetChainId: targetChainId,
            messageHash: messageHash,
            timestamp: block.timestamp,
            processed: false
        });

        token.safeTransferFrom(msg.sender, address(this), amount);

        emit BridgeInitiated(requestId, msg.sender, recipient, amount, targetChainId);
        return requestId;
    }

    function completeBridge(
        bytes32 requestId,
        address recipient,
        uint256 amount,
        uint256 sourceChainId,
        bytes calldata signature
    ) external onlyRole(RELAYER_ROLE) nonReentrant whenNotPaused {
        require(!processedMessages[requestId], "Already processed");
        require(supportedChains[sourceChainId], "Invalid source chain");

        bytes32 messageHash = keccak256(abi.encodePacked(
            requestId,
            sourceChainId,
            localChainId,
            amount
        ));

        require(_verifySignature(messageHash, signature), "Invalid signature");

        processedMessages[requestId] = true;
        token.safeTransfer(recipient, amount);

        emit BridgeCompleted(requestId, recipient, amount);
    }

    function addSupportedChain(uint256 chainId, address bridgeAddress) external onlyRole(DEFAULT_ADMIN_ROLE) {
        supportedChains[chainId] = true;
        remoteBridges[chainId] = bridgeAddress;
        emit ChainSupported(chainId, bridgeAddress);
    }

    function removeSupportedChain(uint256 chainId) external onlyRole(DEFAULT_ADMIN_ROLE) {
        supportedChains[chainId] = false;
        remoteBridges[chainId] = address(0);
    }

    function updateFee(uint256 newFee) external onlyRole(DEFAULT_ADMIN_ROLE) {
        bridgeFee = newFee;
        emit FeeUpdated(newFee);
    }

    function updateLimits(uint256 _min, uint256 _max) external onlyRole(DEFAULT_ADMIN_ROLE) {
        minBridgeAmount = _min;
        maxBridgeAmount = _max;
    }

    function pause() external onlyRole(PAUSER_ROLE) {
        _pause();
    }

    function unpause() external onlyRole(PAUSER_ROLE) {
        _unpause();
    }

    function withdrawFees(address to) external onlyRole(DEFAULT_ADMIN_ROLE) {
        payable(to).transfer(address(this).balance);
    }

    function _verifySignature(bytes32 messageHash, bytes calldata signature) internal view returns (bool) {
        bytes32 ethSignedHash = keccak256(abi.encodePacked(
            "\x19Ethereum Signed Message:\n32",
            messageHash
        ));
        
        (bytes32 r, bytes32 s, uint8 v) = _splitSignature(signature);
        address signer = ecrecover(ethSignedHash, v, r, s);
        
        return hasRole(RELAYER_ROLE, signer);
    }

    function _splitSignature(bytes calldata sig) internal pure returns (bytes32 r, bytes32 s, uint8 v) {
        require(sig.length == 65, "Invalid signature length");
        
        assembly {
            r := calldataload(sig.offset)
            s := calldataload(add(sig.offset, 32))
            v := byte(0, calldataload(add(sig.offset, 64)))
        }
    }
}
