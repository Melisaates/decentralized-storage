// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract StorageStaking is ReentrancyGuard, Ownable {
    IERC20 public stakingToken;
    
    struct Stake {
        uint256 amount;
        uint256 timestamp;
        uint256 storageLimit;
        bool active;
    }
    
    mapping(address => Stake) public stakes;
    
    // Storage conversion rate (1 token = X bytes)
    uint256 public constant BYTES_PER_TOKEN = 1_000_000; // 1MB per token
    
    // Minimum staking period
    uint256 public constant MIN_STAKE_PERIOD = 30 days;
    
    event Staked(address indexed user, uint256 amount, uint256 storageLimit);
    event Unstaked(address indexed user, uint256 amount);
    event StorageLimitUpdated(address indexed user, uint256 newLimit);
    
    constructor(address _stakingToken) {
        stakingToken = IERC20(_stakingToken);
    }
    
    function stake(uint256 amount) external nonReentrant {
        require(amount > 0, "Cannot stake 0 tokens");
        require(stakingToken.transferFrom(msg.sender, address(this), amount), "Transfer failed");
        
        uint256 storageLimit = amount * BYTES_PER_TOKEN;
        
        stakes[msg.sender] = Stake({
            amount: amount,
            timestamp: block.timestamp,
            storageLimit: storageLimit,
            active: true
        });
        
        emit Staked(msg.sender, amount, storageLimit);
    }
    
    function unstake() external nonReentrant {
        Stake storage userStake = stakes[msg.sender];
        require(userStake.active, "No active stake found");
        require(block.timestamp >= userStake.timestamp + MIN_STAKE_PERIOD, "Staking period not completed");
        
        uint256 amount = userStake.amount;
        userStake.active = false;
        
        require(stakingToken.transfer(msg.sender, amount), "Transfer failed");
        
        emit Unstaked(msg.sender, amount);
    }
    
    function getStorageLimit(address user) external view returns (uint256) {
        Stake memory userStake = stakes[user];
        if (!userStake.active) return 0;
        return userStake.storageLimit;
    }
    
    function isStakeActive(address user) external view returns (bool) {
        return stakes[user].active;
    }
    
    function getStakeInfo(address user) external view returns (
        uint256 amount,
        uint256 timestamp,
        uint256 storageLimit,
        bool active
    ) {
        Stake memory userStake = stakes[user];
        return (
            userStake.amount,
            userStake.timestamp,
            userStake.storageLimit,
            userStake.active
        );
    }
}