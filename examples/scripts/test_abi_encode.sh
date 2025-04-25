#!/bin/bash
# Copyright (c) the DTVM authors Core Contributors
# SPDX-License-Identifier: Apache-2.0

# This script is used to test the ABI encoding of ERC20 functions using the abi_encode.py script.
# It requires the abi_encode.py script to be in the same directory as this script.
# The script will test the encoding of various ERC20 functions and print the results to the console.

set -e

source ../scripts/common.sh

# Make script executable
chmod +x abi_encode.py

echo "Testing ERC20 function encodings..."

# Test address and values
ADDR1="0x1122334455667788990011223344556677889900"
ADDR2="0x2233445566778899001122334455667788990011"
AMOUNT="1000000000000000000" # 1 token with 18 decimals

echo -e "\n1. Testing transfer(address,uint256)"
output=$(./abi_encode.py "transfer(address,uint256)" $ADDR1 $AMOUNT)
run_cmd_and_grep "$output" '0xa9059cbb00000000000000000000000011223344556677889900112233445566778899000000000000000000000000000000000000000000000000000de0b6b3a7640000' # Check for transfer function selector

echo -e "\n2. Testing balanceOf(address)"
output=$(./abi_encode.py "balanceOf(address)" $ADDR1)
run_cmd_and_grep "$output" '0x70a082310000000000000000000000001122334455667788990011223344556677889900'

echo -e "\n3. Testing totalSupply()"
output=$(./abi_encode.py "totalSupply()")
run_cmd_and_grep "$output" '0x18160ddd'

echo -e "\n4. Testing approve(address,uint256)"
output=$(./abi_encode.py "approve(address,uint256)" $ADDR2 $AMOUNT)
run_cmd_and_grep "$output" '0x095ea7b300000000000000000000000022334455667788990011223344556677889900110000000000000000000000000000000000000000000000000de0b6b3a7640000'

echo -e "\n5. Testing allowance(address,address)"
output=$(./abi_encode.py "allowance(address,address)" $ADDR1 $ADDR2)
run_cmd_and_grep "$output" '0xdd62ed3e00000000000000000000000011223344556677889900112233445566778899000000000000000000000000002233445566778899001122334455667788990011'

echo -e "\n6. Testing swapExactTokensForTokens(uint256,uint256,address[],address,uint256)"
output=$(./abi_encode.py "swapExactTokensForTokens(uint256,uint256,address[],address,uint256)" "80" "0" "0x5100000000000000000000000000000000000005,0x5100000000000000000000000000000000000006" "0x5100000000000000000000000000000000000006" "99999999999999999")
run_cmd_and_grep "$output" '0x38ed17390000000000000000000000000000000000000000000000000000000000000050000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000005100000000000000000000000000000000000006000000000000000000000000000000000000000000000000016345785d89ffff000000000000000000000000000000000000000000000000000000000000000200000000000000000000000051000000000000000000000000000000000000050000000000000000000000005100000000000000000000000000000000000006'

echo -e "\nAll tests completed."
