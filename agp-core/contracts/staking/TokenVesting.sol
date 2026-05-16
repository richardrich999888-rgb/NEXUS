// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";

/**
 * @title TokenVesting
 * @dev Token vesting with cliff and linear release for team, investors, and ecosystem
 */
contract TokenVesting is Ownable, ReentrancyGuard {
    using SafeERC20 for IERC20;

    struct VestingSchedule {
        uint256 totalAmount;
        uint256 released;
        uint256 startTime;
        uint256 cliffDuration;
        uint256 vestingDuration;
        bool revocable;
        bool revoked;
    }

    IERC20 public immutable token;
    
    mapping(address => VestingSchedule) public schedules;
    address[] public beneficiaries;
    
    uint256 public totalAllocated;
    uint256 public totalReleased;

    event VestingCreated(address indexed beneficiary, uint256 amount, uint256 cliff, uint256 duration);
    event TokensReleased(address indexed beneficiary, uint256 amount);
    event VestingRevoked(address indexed beneficiary, uint256 unvested);

    constructor(address _token) {
        require(_token != address(0), "Invalid token");
        token = IERC20(_token);
    }

    function createVesting(
        address beneficiary,
        uint256 amount,
        uint256 startTime,
        uint256 cliffDuration,
        uint256 vestingDuration,
        bool revocable
    ) external onlyOwner {
        require(beneficiary != address(0), "Invalid beneficiary");
        require(amount > 0, "Amount must be > 0");
        require(schedules[beneficiary].totalAmount == 0, "Schedule exists");
        require(vestingDuration > cliffDuration, "Duration must exceed cliff");

        schedules[beneficiary] = VestingSchedule({
            totalAmount: amount,
            released: 0,
            startTime: startTime,
            cliffDuration: cliffDuration,
            vestingDuration: vestingDuration,
            revocable: revocable,
            revoked: false
        });

        beneficiaries.push(beneficiary);
        totalAllocated += amount;

        token.safeTransferFrom(msg.sender, address(this), amount);
        emit VestingCreated(beneficiary, amount, cliffDuration, vestingDuration);
    }

    function release() external nonReentrant {
        VestingSchedule storage schedule = schedules[msg.sender];
        require(schedule.totalAmount > 0, "No vesting");
        require(!schedule.revoked, "Vesting revoked");

        uint256 releasable = _releasableAmount(msg.sender);
        require(releasable > 0, "Nothing to release");

        schedule.released += releasable;
        totalReleased += releasable;

        token.safeTransfer(msg.sender, releasable);
        emit TokensReleased(msg.sender, releasable);
    }

    function revoke(address beneficiary) external onlyOwner {
        VestingSchedule storage schedule = schedules[beneficiary];
        require(schedule.revocable, "Not revocable");
        require(!schedule.revoked, "Already revoked");

        uint256 releasable = _releasableAmount(beneficiary);
        uint256 unvested = schedule.totalAmount - schedule.released - releasable;

        schedule.revoked = true;
        
        if (releasable > 0) {
            schedule.released += releasable;
            token.safeTransfer(beneficiary, releasable);
        }
        
        if (unvested > 0) {
            token.safeTransfer(owner(), unvested);
        }

        emit VestingRevoked(beneficiary, unvested);
    }

    function _releasableAmount(address beneficiary) internal view returns (uint256) {
        VestingSchedule memory schedule = schedules[beneficiary];
        return _vestedAmount(schedule) - schedule.released;
    }

    function _vestedAmount(VestingSchedule memory schedule) internal view returns (uint256) {
        if (block.timestamp < schedule.startTime + schedule.cliffDuration) {
            return 0;
        }
        
        uint256 elapsed = block.timestamp - schedule.startTime;
        
        if (elapsed >= schedule.vestingDuration) {
            return schedule.totalAmount;
        }
        
        return (schedule.totalAmount * elapsed) / schedule.vestingDuration;
    }

    function getVestedAmount(address beneficiary) external view returns (uint256) {
        return _vestedAmount(schedules[beneficiary]);
    }

    function getReleasableAmount(address beneficiary) external view returns (uint256) {
        return _releasableAmount(beneficiary);
    }

    function getBeneficiaryCount() external view returns (uint256) {
        return beneficiaries.length;
    }
}
