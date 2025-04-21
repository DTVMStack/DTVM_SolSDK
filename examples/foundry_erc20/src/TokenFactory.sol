// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "./MyToken.sol";

contract TokenFactory {
    // 存储已部署的代币地址
    address[] public deployedTokens;

    // 事件：新代币部署
    event TokenDeployed(
        address indexed tokenAddress, 
        address indexed owner, 
        uint256 initialSupply
    );

    // 部署新代币的函数
    function createToken(uint256 initialSupply) public returns (address) {
        // 创建新的MyToken实例
        MyToken newToken = new MyToken(initialSupply);
        
        // 将所有权转移给创建者
        newToken.transferOwnership(msg.sender);
        
        // 记录部署的代币地址
        deployedTokens.push(address(newToken));
        
        // 触发事件
        emit TokenDeployed(address(newToken), msg.sender, initialSupply);
        
        return address(newToken);
    }

    // 获取所有已部署代币的数量
    function getDeployedTokensCount() public view returns (uint256) {
        return deployedTokens.length;
    }

    // 获取特定索引的代币地址
    function getTokenAt(uint256 index) public view returns (address) {
        require(index < deployedTokens.length, "Invalid token index");
        return deployedTokens[index];
    }
}

