// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "./MyToken.sol";

contract TokenFactory {
    // Store addresses of deployed tokens
    address[] public deployedTokens;

    // Event: New token deployment
    event TokenDeployed(
        address indexed tokenAddress, 
        address indexed owner, 
        uint256 initialSupply
    );

    // Function to deploy a new token
    function createToken(uint256 initialSupply) public returns (address) {
        // Create new MyToken instance
        MyToken newToken = new MyToken(initialSupply);
        
        // Transfer ownership to creator
        newToken.transferOwnership(msg.sender);
        
        // Record deployed token address
        deployedTokens.push(address(newToken));
        
        // Emit event
        emit TokenDeployed(address(newToken), msg.sender, initialSupply);
        
        return address(newToken);
    }

    // Get total count of deployed tokens
    function getDeployedTokensCount() public view returns (uint256) {
        return deployedTokens.length;
    }

    // Get token address at specific index
    function getTokenAt(uint256 index) public view returns (address) {
        require(index < deployedTokens.length, "Invalid token index");
        return deployedTokens[index];
    }
}

