// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC20/extensions/ERC20Burnable.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract MyToken is ERC20Burnable, Ownable {
    // 常量配置
    uint8 private constant TOKEN_DECIMALS = 18;
    string private constant TOKEN_NAME = "My First Token";
    string private constant TOKEN_SYMBOL = "MFT";

    // 构造函数
    constructor(uint256 initialSupply) ERC20(TOKEN_NAME, TOKEN_SYMBOL) Ownable(msg.sender) {
        // 初始化代币，铸造初始供应量
        _mint(msg.sender, initialSupply);
    }

    // 增加可选的铸造函数（继承自ERC20Burnable）
    function mint(address to, uint256 amount) public onlyOwner {
        _mint(to, amount);
    }
}

