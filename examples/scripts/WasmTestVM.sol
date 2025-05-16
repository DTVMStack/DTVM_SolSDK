// SPDX-License-Identifier: Unlicense
pragma solidity ^0.8.0;

contract WasmTestVM {
    /// Asserts that two `int256` values are equal and includes error message into revert string on failure.
    function assertEq(uint256 left, uint256 right, string memory err) public pure {
        require(left == right, err);
    }
}
