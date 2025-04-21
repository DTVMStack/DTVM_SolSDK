#!/bin/bash
set -e

source ../scripts/common.sh

ABI_ENCODE="../scripts/abi_encode.py"

rm -rf test.db

echo 'test deploy contract'
# deploy contract
/opt/chain_mockcli -f test_init_code_hash.wasm --action deploy --print-time --enable-gas-meter -i 0x

echo 'test calculate_create2_addr()'
# query fibonacci(uint256)
CALCULATE_CREATE2_ADDR_ABI_DATA=$($ABI_ENCODE "calculate_create2_addr()")
output=$(/opt/chain_mockcli -f test_init_code_hash.wasm --action call --print-time --enable-gas-meter -i $CALCULATE_CREATE2_ADDR_ABI_DATA)
run_cmd_and_grep "$output" 'evm finish with result hex: '
echo 'test calculate_create2_addr() end'

echo 'test create_child_by_create2()'
# query fibonacci(uint256)
CREATE_CHILD_BY_CREATE2_ABI_DATA=$($ABI_ENCODE "create_child_by_create2()")
output=$(/opt/chain_mockcli -f test_init_code_hash.wasm --action call --print-time --enable-gas-meter -i $CREATE_CHILD_BY_CREATE2_ABI_DATA)
run_cmd_and_grep "$output" 'evm finish with result hex: 0000000000000000000000000000000000000000000000000000000000000001'
echo 'test create_child_by_create2() end'
