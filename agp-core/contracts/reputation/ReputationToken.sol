// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/ERC20Votes.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/security/Pausable.sol";

/**
 * @title ReputationToken
 * @dev ERC20 token representing reputation in the AGP-CORE ecosystem
 * Includes snapshots for governance voting power.
 */
contract ReputationToken is ERC20, ERC20Votes, AccessControl, Pausable {
    bytes32 public constant MINTER_ROLE = keccak256("MINTER_ROLE");
    bytes32 public constant BURNER_ROLE = keccak256("BURNER_ROLE");
    bytes32 public constant PAUSER_ROLE = keccak256("PAUSER_ROLE");
    
    // Maximum total supply (optional cap)
    uint256 public maxSupply;
    
    // Address of the protocol this token belongs to
    address public protocolAddress;
    
    // Staking information (simplified tracking)
    mapping(address => uint256) public stakedBalance;
    
    // Events
    event TokensMinted(address indexed to, uint256 amount, string reason);
    event TokensBurned(address indexed from, uint256 amount, string reason);
    event TokensStaked(address indexed user, uint256 amount);
    event TokensUnstaked(address indexed user, uint256 amount);
    
    constructor(
        string memory _name,
        string memory _symbol,
        uint256 _initialSupply,
        address _protocol,
        uint256 _maxSupply
    ) ERC20(_name, _symbol) ERC20Permit(_name) {
        require(_protocol != address(0), "Invalid protocol address");
        
        protocolAddress = _protocol;
        maxSupply = _maxSupply;
        
        _setupRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _setupRole(MINTER_ROLE, msg.sender);
        _setupRole(BURNER_ROLE, msg.sender);
        _setupRole(PAUSER_ROLE, msg.sender);
        
        if (_initialSupply > 0) {
            _mint(_protocol, _initialSupply);
        }
    }
    
    function mint(address to, uint256 amount, string memory reason) external onlyRole(MINTER_ROLE) {
        if (maxSupply > 0) {
            require(totalSupply() + amount <= maxSupply, "Exceeds max supply");
        }
        _mint(to, amount);
        emit TokensMinted(to, amount, reason);
    }
    
    function burn(address from, uint256 amount, string memory reason) external onlyRole(BURNER_ROLE) {
        _burn(from, amount);
        emit TokensBurned(from, amount, reason);
    }
    
    function pause() external onlyRole(PAUSER_ROLE) {
        _pause();
    }
    
    function unpause() external onlyRole(PAUSER_ROLE) {
        _unpause();
    }
    
    // Overrides for ERC20Votes
    function _afterTokenTransfer(address from, address to, uint256 amount) internal override(ERC20, ERC20Votes) {
        super._afterTokenTransfer(from, to, amount);
    }
    
    function _mint(address to, uint256 amount) internal override(ERC20, ERC20Votes) {
        super._mint(to, amount);
    }
    
    function _burn(address account, uint256 amount) internal override(ERC20, ERC20Votes) {
        super._burn(account, amount);
    }
}
