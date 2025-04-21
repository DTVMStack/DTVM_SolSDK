#!/bin/bash

set -e

source ../scripts/common.sh

# Clean up previous test database
rm -f test.db

# Set contract address
CONTRACT="0x1010000000000000000000000000000000000101"
USER1="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
USER2="bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
USER3="cccccccccccccccccccccccccccccccccccccccc"

# Deploy contract using --action deploy
echo "Deploying ERC721 contract..."
/opt/chain_mockcli --action deploy -t $CONTRACT -f out/MyERC721.wasm -i 0x

echo "Contract deployed at: $CONTRACT"

# Function: mint(address to)
# Parameters: to = $USER1
echo "Testing mint..."
output=$(/opt/chain_mockcli --action call --print-time -t $CONTRACT -i 0x6a627842000000000000000000000000$USER1)
run_cmd_and_grep "$output" 'evm finish with result hex: 0000000000000000000000000000000000000000000000000000000000000000\ngas used'

# Function: ownerOf(uint256 tokenId)
# Parameters: tokenId = 0
echo "Checking ownership..."
output=$(/opt/chain_mockcli --action call --print-time -t $CONTRACT -i 0x6352211e0000000000000000000000000000000000000000000000000000000000000000)
run_cmd_and_grep "$output" 'evm finish with result hex: 000000000000000000000000aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\ngas used'

# Function: balanceOf(address owner)
# Parameters: owner = 0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
echo "Checking balance..."
output=$(/opt/chain_mockcli --action call --print-time -t $CONTRACT -i 0x70a08231000000000000000000000000aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa)
run_cmd_and_grep "$output" 'evm finish with result hex: 0000000000000000000000000000000000000000000000000000000000000001\ngas used'

# Function: approve(address to, uint256 tokenId)
# sender = $USER1
# Parameters: 
#   to = $USER2
#   tokenId = 0
echo "Testing approve..."
output=$(/opt/chain_mockcli --action call --print-time -t $CONTRACT -s 0x$USER1 -i 0x095ea7b3000000000000000000000000bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb0000000000000000000000000000000000000000000000000000000000000000)
run_cmd_and_grep "$output" 'evm finish with result hex: \ngas used'

# Function: transferFrom(address from, address to, uint256 tokenId)
# sender = USER2
# Parameters:
#   from = 0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
#   to = $USER3
#   tokenId = 0
echo "Testing transfer..."
output=$(/opt/chain_mockcli --action call --print-time -t $CONTRACT -s 0x$USER2 -i 0x23b872dd000000000000000000000000aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa000000000000000000000000cccccccccccccccccccccccccccccccccccccccc0000000000000000000000000000000000000000000000000000000000000000)
run_cmd_and_grep "$output" 'evm finish with result hex: \ngas used'

# Function: ownerOf(uint256 tokenId)
# Parameters: tokenId = 0
echo "Verifying token final owner..."
output=$(/opt/chain_mockcli --action call --print-time -t $CONTRACT -i 0x6352211e0000000000000000000000000000000000000000000000000000000000000000)
run_cmd_and_grep "$output" 'evm finish with result hex: 000000000000000000000000cccccccccccccccccccccccccccccccccccccccc\ngas used'

echo "All tests passed!"
