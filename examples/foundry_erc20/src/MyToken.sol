// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC20/extensions/ERC20Burnable.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract MyToken is ERC20Burnable, Ownable {
    // Constant configuration
    uint8 private constant TOKEN_DECIMALS = 18;
    string private constant TOKEN_NAME = "My First Token";
    string private constant TOKEN_SYMBOL = "MFT";

    // Constructor
    constructor(uint256 initialSupply) ERC20(TOKEN_NAME, TOKEN_SYMBOL) Ownable(msg.sender) {
        // Initialize token, mint initial supply
        _mint(msg.sender, initialSupply);
    }

    // Optional mint function (inherited from ERC20Burnable)
    function mint(address to, uint256 amount) public onlyOwner {
        _mint(to, amount);
    }
}

