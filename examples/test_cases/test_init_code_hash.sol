pragma solidity ^0.8.0;

contract TestInitCodeHashChild {
    function add(uint256 a, uint256 b) public pure returns (uint256) {
        return a + b;
    }
}

contract TestInitCodeHashParent {
    
    function calculate_create2_addr() public returns (address) {
        bytes32 childCodeHashFromCreationCode = keccak256(type(TestInitCodeHashChild).creationCode);
        emit emitBytes32(childCodeHashFromCreationCode);
        bytes32 salt = keccak256(abi.encodePacked(msg.sender));
        address childAddress = address(
            uint160(
                uint256(
                    keccak256(
                        abi.encodePacked(
                            bytes1(0xff),
                            address(this),
                            salt,
                            childCodeHashFromCreationCode
                        )
                    )
                )
            )
        );
        return childAddress;
    }

    event emitAddress(address value);
    event emitBytes32(bytes32 value);

    function create_child_by_create2() public returns (bool) {
        bytes32 salt = keccak256(abi.encodePacked(msg.sender));
        address child = address(new TestInitCodeHashChild{salt: salt}());
        bytes32 childCodeHashFromExtcodehash;
        assembly {
            childCodeHashFromExtcodehash := extcodehash(child)
        }
        emit emitBytes32(childCodeHashFromExtcodehash);
        bytes32 childCodeHashFromCreationCode = keccak256(type(TestInitCodeHashChild).creationCode);
        emit emitBytes32(childCodeHashFromCreationCode);
        emit emitAddress(child);
        return childCodeHashFromExtcodehash == childCodeHashFromCreationCode;
    }
}
