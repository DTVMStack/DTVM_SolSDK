#!/bin/bash
set -e

source ../scripts/common.sh

# Read input ERC20 wasm file and execute ERC20 test cases similar to test_my_token.sh
wasm_file=$1
# Check if wasm file path is provided and file exists
if [ -z "$wasm_file" ] || [ ! -f "$wasm_file" ]; then
    echo "Usage: $0 <path_to_wasm_file>"
    exit 1
fi

echo 'test deploy ERC20 contract $wasm_file'
# deploy contract (arg total supply(uint256))
/opt/chain_mockcli -f $wasm_file --action deploy --print-time --enable-gas-meter -s 0x9988776655443322119900112233445566778899 -i 0x68656c6c6f000000000000000000000000000000000000000000000000000000
# total supply is optional here
# /opt/chain_mockcli -f $wasm_file --action deploy -i 0x
echo 'test totalSupply after deploy erc20'
# query totalSupply()
output=$(/opt/chain_mockcli -f $wasm_file --action call --print-time --enable-gas-meter -i 0x18160ddd)
run_cmd_and_grep "$output" 'evm finish with result hex: 68656c6c6f000000000000000000000000000000000000000000000000000000'

echo 'test mint'
# mint(owner,amount)
output=$(/opt/chain_mockcli -f $wasm_file --action call --print-time --enable-gas-meter -s 0x9988776655443322119900112233445566778899 -i 0x40c10f1900000000000000000000000000112233445566778899001122334455667788990000000000000000000000000000000000000000000000000000000000000007)
run_cmd_and_grep "$output" 'evm finish with result hex: \ngas used'

echo 'test balanceOf after mint'
# balanceOf(address) after mint 68656c6c6f000000000000000000000000000000000000000000000000000007 when has total_supply big
output=$(/opt/chain_mockcli -f $wasm_file --action call --print-time --enable-gas-meter -i 0x70a082310000000000000000000000000011223344556677889900112233445566778899)
run_cmd_and_grep "$output" 'evm finish with result hex: 0000000000000000000000000000000000000000000000000000000000000007'

echo 'test transfer from owner to user2'
# transfer from owner to user2
output=$(/opt/chain_mockcli -f $wasm_file --action call --print-time --enable-gas-meter -i 0xa9059cbb0000000000000000000000005b38da6a701c568545dcfcb03fcb875f56beddc40000000000000000000000000000000000000000000000000000000000000005)
run_cmd_and_grep "$output" 'evm finish with result hex:'

echo 'test query balanceOf after transfer'
# balanceOf(address)
output=$(/opt/chain_mockcli -f $wasm_file --action call --print-time --enable-gas-meter -i 0x70a082310000000000000000000000005b38da6a701c568545dcfcb03fcb875f56beddc4)
run_cmd_and_grep "$output" 'evm finish with result hex: 0000000000000000000000000000000000000000000000000000000000000005'

# test approve, allowance, transferFrom
echo 'test approve, allowance, transferFrom'

# approve to user2
output=$(/opt/chain_mockcli -f $wasm_file --action call --print-time --enable-gas-meter -i 0x095ea7b30000000000000000000000005b38da6a701c568545dcfcb03fcb875f56beddc40000000000000000000000000000000000000000000000000000000000000001)
run_cmd_and_grep "$output" 'evm finish with result hex:'

# query allowance to user2 (sender is 0x0011223344556677889900112233445566778899)
output=$(/opt/chain_mockcli -f $wasm_file --action call --print-time --enable-gas-meter -i 0xdd62ed3e00000000000000000000000000112233445566778899001122334455667788990000000000000000000000005b38da6a701c568545dcfcb03fcb875f56beddc4)
run_cmd_and_grep "$output" 'evm finish with result hex: 0000000000000000000000000000000000000000000000000000000000000001'

# transferFrom from 0x0011223344556677889900112233445566778899 to user3 (send by user2)
output=$(/opt/chain_mockcli -f $wasm_file --action call --print-time --enable-gas-meter --sender-address-hex 0x5b38da6a701c568545dcfcb03fcb875f56beddc4 -i 0x23b872dd00000000000000000000000000112233445566778899001122334455667788990000000000000000000000005b38da6a701c568545dcfcb03fcb875f56beddc50000000000000000000000000000000000000000000000000000000000000001)
run_cmd_and_grep "$output" 'evm finish with result hex: 0000000000000000000000000000000000000000000000000000000000000001'

# query balanceOf user3
output=$(/opt/chain_mockcli -f $wasm_file --action call --print-time --enable-gas-meter -i 0x70a082310000000000000000000000005b38da6a701c568545dcfcb03fcb875f56beddc5)
run_cmd_and_grep "$output" 'evm finish with result hex: 0000000000000000000000000000000000000000000000000000000000000001'

echo 'all ERC20 tests success'
