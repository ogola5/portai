// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract PortfolioTrader {
    // Events for tracking trades
    event TradeExecuted(address indexed user, uint256 amount, string asset, string action);
    event DepositReceived(address indexed user, uint256 amount);
    event Withdrawal(address indexed user, uint256 amount);

    // Mapping to track user balances (in wei-like units, matching NEAR's Balance)
    mapping(address => uint256) public balances;
    // Mapping to track executed trades per user
    mapping(address => Trade[]) public tradeHistory;

    // Struct to match NEAR's TradeRequest (without timestamp, as it's on-chain)
    struct Trade {
        string asset;
        uint256 amount;
        string action; // "buy" or "sell"
    }

    // Modifier to ensure only NEAR bridge or authorized calls
    modifier onlyAuthorized() {
        // Placeholder: In production, restrict to NEAR bridge or owner
        _;
    }

    // Fallback to receive ETH (Aurora's native token)
    receive() external payable {
        balances[msg.sender] += msg.value;
        emit DepositReceived(msg.sender, msg.value);
    }

    // Deposit function for users to fund trades
    function deposit() external payable {
        require(msg.value > 0, "Deposit must be greater than 0");
        balances[msg.sender] += msg.value;
        emit DepositReceived(msg.sender, msg.value);
    }

    // Execute trade (called by NEAR via bridge)
    function execute_trade(string memory user, uint256 amount, string memory asset) external onlyAuthorized {
        // Convert NEAR AccountId (string) to Ethereum address (simplified for demo)
        address userAddr = address(bytes20(keccak256(abi.encodePacked(user))));
        require(amount > 0, "Amount must be greater than 0");

        // Determine action from NEAR context (simplified; assume "buy" or "sell" passed)
        string memory action = bytes(asset).length > 0 ? "buy" : "sell"; // Placeholder logic

        // Record trade
        tradeHistory[userAddr].push(Trade({
            asset: asset,
            amount: amount,
            action: action
        }));

        // Simulate trade execution (e.g., interact with a DEX like Trisolaris)
        if (keccak256(abi.encodePacked(action)) == keccak256(abi.encodePacked("buy"))) {
            require(balances[userAddr] >= amount, "Insufficient balance for buy");
            balances[userAddr] -= amount;
            // In reality: Call DEX to swap ETH for asset
        } else if (keccak256(abi.encodePacked(action)) == keccak256(abi.encodePacked("sell"))) {
            // In reality: Call DEX to swap asset for ETH, then credit balance
            balances[userAddr] += amount; // Simulated
        }

        emit TradeExecuted(userAddr, amount, asset, action);
    }

    // Withdraw funds
    function withdraw(uint256 amount) external {
        require(balances[msg.sender] >= amount, "Insufficient balance");
        balances[msg.sender] -= amount;
        (bool sent, ) = msg.sender.call{value: amount}("");
        require(sent, "Withdrawal failed");
        emit Withdrawal(msg.sender, amount);
    }

    // Get user's trade history
    function getTradeHistory(address user) external view returns (Trade[] memory) {
        return tradeHistory[user];
    }
}