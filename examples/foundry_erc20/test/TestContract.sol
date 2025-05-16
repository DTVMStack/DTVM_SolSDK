// SPDX-License-Identifier: Unlicense
pragma solidity ^0.8.0;

import "forge-std/Test.sol";

import "src/MyToken.sol";

contract TestContract is Test {
    MyToken c;

    address owner = address(this);
    address user2 = address(0x5B38Da6a701c568545dCfcB03FcB875f56beddC4);
    address user3 = address(0x5B38Da6a701C568545DcFCb03fcb875F56BEDDc5);

    uint256 constant INITIAL_SUPPLY = 1000;
    uint256 constant MINT_AMOUNT = 7;
    uint256 constant TRANSFER_AMOUNT = 5;
    uint256 constant APPROVE_AMOUNT = 10;

    function setUp() public {
        c = new MyToken(INITIAL_SUPPLY);
    }

    function testDeployAndTotalSupply() public {
        assertEq(c.totalSupply(), INITIAL_SUPPLY, "Initial total supply should match");
    }

    function testMint() public {
        uint256 user2_balance = c.balanceOf(user2);
        c.mint(user2, MINT_AMOUNT);
        assertEq(c.balanceOf(user2), user2_balance + MINT_AMOUNT, "User2 balance should be correct after mint");
    }

    function testApproveAndAllowance() public {
        uint256 user2_allowance = c.allowance(owner, user2);
        c.approve(user2, APPROVE_AMOUNT);
        assertEq(c.allowance(owner, user2), user2_allowance + APPROVE_AMOUNT, "Allowance should be set correctly");
    }

    function testTransfer() public {
        assertEq(c.balanceOf(address(this)), INITIAL_SUPPLY, "Owner balance should be 0 before testTransfer");
        c.mint(address(this), MINT_AMOUNT);
        assertEq(c.balanceOf(address(this)), INITIAL_SUPPLY + MINT_AMOUNT, "Owner balance should be 5 before transfer");

        uint256 user2_balance = c.balanceOf(user2);
        c.transfer(user2, TRANSFER_AMOUNT);

        assertEq(c.balanceOf(user2), user2_balance + TRANSFER_AMOUNT, "User2 should receive correct amount");
        assertEq(c.balanceOf(address(this)), INITIAL_SUPPLY + MINT_AMOUNT - TRANSFER_AMOUNT, "Owner balance should be decreased");
    }

    event ValueLogged(uint256 value);

    event AddressLogged(address value);

    function testTransferFrom() public {
        emit AddressLogged(address(this));
        uint256 user2_allowance = c.allowance(address(this), user2);
        vm.startPrank(owner);
        uint256 this_balance = c.balanceOf(address(this));
        emit ValueLogged(this_balance);
        c.mint(address(this), MINT_AMOUNT);
        emit ValueLogged(c.balanceOf(address(this)));
        c.approve(user2, APPROVE_AMOUNT);
        vm.stopPrank();

        vm.startPrank(user2);
        uint256 user3_balance = c.balanceOf(user3);
        emit ValueLogged(user3_balance);
        c.transferFrom(address(this), user3, APPROVE_AMOUNT);
        vm.stopPrank();
        emit ValueLogged(c.balanceOf(address(this)));

        assertEq(c.balanceOf(user3), user3_balance + APPROVE_AMOUNT, "User3 should receive correct amount");
        assertEq(c.balanceOf(address(this)), this_balance + MINT_AMOUNT - APPROVE_AMOUNT, "Owner balance should be decreased");
        assertEq(c.allowance(address(this), user2), user2_allowance + APPROVE_AMOUNT - APPROVE_AMOUNT, "Allowance should be decreased");
    }

    // CompleteFlow: deploy -> mint -> transfer -> approve -> transferFrom
    function testCompleteFlow() public {
        uint256 owner_balance = c.balanceOf(owner);
        uint256 user2_balance = c.balanceOf(user2);
        uint256 user2_allowance_before_approve = c.allowance(owner, user2);
        uint256 user3_balance = c.balanceOf(user3);

        vm.startPrank(owner);

        assertEq(c.totalSupply(), INITIAL_SUPPLY, "Initial total supply should match");

        c.mint(owner, MINT_AMOUNT);
        assertEq(c.balanceOf(owner), owner_balance + MINT_AMOUNT, "Owner should receive minted tokens");

        c.transfer(user2, TRANSFER_AMOUNT);
        assertEq(c.balanceOf(user2), user2_balance + TRANSFER_AMOUNT, "User2 should receive transferred tokens");

        c.approve(user2, APPROVE_AMOUNT);
        uint256 user2_allowance_after_approve = c.allowance(owner, user2);
        assertEq(c.allowance(owner, user2), user2_allowance_before_approve + APPROVE_AMOUNT, "Allowance should be set correctly");

        vm.stopPrank();

        vm.startPrank(user2);
        c.transferFrom(owner, user3, APPROVE_AMOUNT);
        vm.stopPrank();

        assertEq(c.balanceOf(user3), user3_balance + APPROVE_AMOUNT, "User3 should receive correct amount");
        assertEq(c.allowance(owner, user2), user2_allowance_after_approve - APPROVE_AMOUNT, "Allowance should be decreased");
    }
}
