// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";

/**
 * @title AlignmentStaking
 * @dev Staking contract rewarding agents based on alignment and reputation
 */
contract AlignmentStaking is ReentrancyGuard, AccessControl {
    using SafeERC20 for IERC20;
    
    bytes32 public constant REWARD_MANAGER_ROLE = keccak256("REWARD_MANAGER_ROLE");
    
    IERC20 public stakingToken;
    IERC20 public rewardToken;
    
    uint256 public rewardRate; // per second
    uint256 public lastUpdateTime;
    uint256 public rewardPerTokenStored;
    
    struct Stake {
        uint256 amount;
        uint256 userRewardPerTokenPaid;
        uint256 rewards;
        uint256 alignmentMultiplier; // scaled by 1e18
        uint256 lockUntil;
    }
    
    mapping(address => Stake) public stakes;
    uint256 public totalStaked;
    
    event Staked(address indexed user, uint256 amount);
    event Unstaked(address indexed user, uint256 amount);
    event RewardPaid(address indexed user, uint256 reward);
    
    constructor(address _stakingToken, address _rewardToken) {
        stakingToken = IERC20(_stakingToken);
        rewardToken = IERC20(_rewardToken);
        _setupRole(DEFAULT_ADMIN_ROLE, msg.sender);
    }
    
    function rewardPerToken() public view returns (uint256) {
        if (totalStaked == 0) {
            return rewardPerTokenStored;
        }
        return rewardPerTokenStored + (block.timestamp - lastUpdateTime) * rewardRate * 1e18 / totalStaked;
    }
    
    function earned(address account) public view returns (uint256) {
        Stake storage s = stakes[account];
        uint256 baseEarned = s.amount * (rewardPerToken() - s.userRewardPerTokenPaid) / 1e18 + s.rewards;
        return baseEarned * s.alignmentMultiplier / 1e18;
    }
    
    modifier updateReward(address account) {
        rewardPerTokenStored = rewardPerToken();
        lastUpdateTime = block.timestamp;
        if (account != address(0)) {
            stakes[account].rewards = earned(account);
            stakes[account].userRewardPerTokenPaid = rewardPerTokenStored;
        }
        _;
    }
    
    function stake(uint256 amount, uint256 lockDays, uint256 multiplier) external nonReentrant updateReward(msg.sender) {
        require(amount > 0, "Cannot stake 0");
        stakingToken.safeTransferFrom(msg.sender, address(this), amount);
        totalStaked += amount;
        stakes[msg.sender].amount += amount;
        stakes[msg.sender].alignmentMultiplier = multiplier;
        stakes[msg.sender].lockUntil = block.timestamp + (lockDays * 1 days);
        emit Staked(msg.sender, amount);
    }
    
    function withdraw(uint256 amount) public nonReentrant updateReward(msg.sender) {
        require(amount > 0, "Cannot withdraw 0");
        require(block.timestamp >= stakes[msg.sender].lockUntil, "Still locked");
        totalStaked -= amount;
        stakes[msg.sender].amount -= amount;
        stakingToken.safeTransfer(msg.sender, amount);
        emit Unstaked(msg.sender, amount);
    }
    
    function getReward() public nonReentrant updateReward(msg.sender) {
        uint256 reward = stakes[msg.sender].rewards;
        if (reward > 0) {
            stakes[msg.sender].rewards = 0;
            rewardToken.safeTransfer(msg.sender, reward);
            emit RewardPaid(msg.sender, reward);
        }
    }
}
