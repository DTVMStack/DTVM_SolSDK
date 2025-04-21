#!/bin/bash

set -e

source ../scripts/common.sh

# Clean up previous test database
rm -f test.db

# Set contract address and test accounts
CONTRACT="0x1010000000000000000000000000000000000101"
USER1="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
USER2="bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
USER3="cccccccccccccccccccccccccccccccccccccccc"

# Deploy contract
echo "Deploying ERC1155 contract..."
/opt/chain_mockcli --action deploy -t $CONTRACT -f out/MyERC1155.wasm -i 0x

echo "Contract deployed at: $CONTRACT"

# Function: mint(address account, uint256 id, uint256 amount, bytes memory data)
# Parameters: account = $USER1, id = 1, amount = 100, data = 0xdddd
echo "Testing mint..."
output=$(/opt/chain_mockcli --action call --print-time -t $CONTRACT -i 0x731133e9000000000000000000000000${USER1}0000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000002dddd000000000000000000000000000000000000000000000000000000000000)
run_cmd_and_grep "$output" 'evm finish with result hex: \ngas used'

# Function: balanceOf(address account, uint256 id)
# Parameters: account = $USER1, id = 1
echo "Checking balance..."
output=$(/opt/chain_mockcli --action call --print-time -t $CONTRACT -i 0x00fdd58e000000000000000000000000${USER1}0000000000000000000000000000000000000000000000000000000000000001)
run_cmd_and_grep "$output" 'evm finish with result hex: 0000000000000000000000000000000000000000000000000000000000000064\ngas used'

# Function: setApprovalForAll(address operator, bool approved)
# sender = $USER1
# Parameters: operator = $USER2, approved = true
echo "Testing setApprovalForAll..."
output=$(/opt/chain_mockcli --action call --print-time -t $CONTRACT -s 0x$USER1 -i 0xa22cb465000000000000000000000000${USER2}0000000000000000000000000000000000000000000000000000000000000001)
run_cmd_and_grep "$output" 'evm finish with result hex: \ngas used'

# Function: safeTransferFrom(address from, address to, uint256 id, uint256 amount, bytes memory data)
# sender = $USER2
# Parameters: from = $USER1, to = $USER3, id = 1, amount = 50, data = 0x
echo "Testing safeTransferFrom..."
output=$(/opt/chain_mockcli --action call --print-time -t $CONTRACT -s 0x$USER2 -i 0xf242432a000000000000000000000000${USER1}000000000000000000000000${USER3}0000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000003200000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000002dddd000000000000000000000000000000000000000000000000000000000000)
run_cmd_and_grep "$output" 'evm finish with result hex: \ngas used'

# Check final balances
echo "Verifying final balances..."
# Check USER1's balance
output=$(/opt/chain_mockcli --action call --print-time -t $CONTRACT -i 0x00fdd58e000000000000000000000000${USER1}0000000000000000000000000000000000000000000000000000000000000001)
run_cmd_and_grep "$output" 'evm finish with result hex: 0000000000000000000000000000000000000000000000000000000000000032\ngas used'

# Check USER3's balance
output=$(/opt/chain_mockcli --action call --print-time -t $CONTRACT -i 0x00fdd58e000000000000000000000000${USER3}0000000000000000000000000000000000000000000000000000000000000001)
run_cmd_and_grep "$output" 'evm finish with result hex: 0000000000000000000000000000000000000000000000000000000000000032\ngas used'

echo "All tests passed!"
