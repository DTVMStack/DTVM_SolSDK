#!/bin/bash
set -e

source ../scripts/common.sh
# Read input ERC20 contract address and execute ERC20 test cases similar to test_my_token.sh
erc20_contract_addr=$1
# Need to pass in the token owner contract address which has permission to mint tokens
# We use this address to test minting to bypass permission checks
token_owner_addr=$2

# Exit with error if contract address or owner address is not provided
if [ -z "$erc20_contract_addr" ] || [ -z "$token_owner_addr" ]; then
    echo "Usage: $0 <ERC20-contract-address> <token-owner-address>"
    exit 1
fi

# This assumes a fixed amount of tokens were minted during deployment
echo 'test totalSupply after deploy erc20'
# query totalSupply()
output=$(/opt/chain_mockcli -t $erc20_contract_addr --action call --print-time -i 0x18160ddd)
run_cmd_and_grep "$output" 'evm finish with result hex: 68656c6c6f000000000000000000000000000000000000000000000000000000'

echo 'test mint'
# mint(token_owner_addr,amount)
output=$(/opt/chain_mockcli -t $erc20_contract_addr --action call --print-time -s 0x$token_owner_addr -i 0x40c10f1900000000000000000000000000112233445566778899001122334455667788990000000000000000000000000000000000000000000000000000000000000007)
run_cmd_and_grep "$output" 'evm finish with result hex:'

echo 'test balanceOf after mint'
# balanceOf(address) after mint 68656c6c6f000000000000000000000000000000000000000000000000000007 when has total_supply big
output=$(/opt/chain_mockcli -t $erc20_contract_addr --action call --print-time -i 0x70a082310000000000000000000000000011223344556677889900112233445566778899)
run_cmd_and_grep "$output" 'evm finish with result hex: 0000000000000000000000000000000000000000000000000000000000000007'

echo 'test transfer from owner to user2'
# transfer from owner to user2
output=$(/opt/chain_mockcli -t $erc20_contract_addr --action call --print-time -i 0xa9059cbb0000000000000000000000005b38da6a701c568545dcfcb03fcb875f56beddc40000000000000000000000000000000000000000000000000000000000000005)
run_cmd_and_grep "$output" 'evm finish with result hex:'

echo 'test query balanceOf after transfer'
# balanceOf(address)
output=$(/opt/chain_mockcli -t $erc20_contract_addr --action call --print-time -i 0x70a082310000000000000000000000005b38da6a701c568545dcfcb03fcb875f56beddc4)
run_cmd_and_grep "$output" 'evm finish with result hex: 0000000000000000000000000000000000000000000000000000000000000005'

# test approve, allowance, transferFrom
echo 'test approve, allowance, transferFrom'

# approve to user2
output=$(/opt/chain_mockcli -t $erc20_contract_addr --action call --print-time -i 0x095ea7b30000000000000000000000005b38da6a701c568545dcfcb03fcb875f56beddc40000000000000000000000000000000000000000000000000000000000000001)
run_cmd_and_grep "$output" 'evm finish with result hex:'

# query allowance to user2 (sender is 0x0011223344556677889900112233445566778899)
output=$(/opt/chain_mockcli -t $erc20_contract_addr --action call --print-time -i 0xdd62ed3e00000000000000000000000000112233445566778899001122334455667788990000000000000000000000005b38da6a701c568545dcfcb03fcb875f56beddc4)
run_cmd_and_grep "$output" 'evm finish with result hex: 0000000000000000000000000000000000000000000000000000000000000001'

# transferFrom from 0x0011223344556677889900112233445566778899 to user3 (send by user2)
output=$(/opt/chain_mockcli -t $erc20_contract_addr --action call --print-time --sender-address-hex 0x5b38da6a701c568545dcfcb03fcb875f56beddc4 -i 0x23b872dd00000000000000000000000000112233445566778899001122334455667788990000000000000000000000005b38da6a701c568545dcfcb03fcb875f56beddc50000000000000000000000000000000000000000000000000000000000000001)
run_cmd_and_grep "$output" 'evm finish with result hex: 0000000000000000000000000000000000000000000000000000000000000001'

# query balanceOf user3
output=$(/opt/chain_mockcli -t $erc20_contract_addr --action call --print-time -i 0x70a082310000000000000000000000005b38da6a701c568545dcfcb03fcb875f56beddc5)
run_cmd_and_grep "$output" 'evm finish with result hex: 0000000000000000000000000000000000000000000000000000000000000001'

echo 'all ERC20 tests success'
