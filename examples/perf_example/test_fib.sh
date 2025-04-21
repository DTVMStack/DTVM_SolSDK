#!/bin/bash
set -e

source ../scripts/common.sh

ABI_ENCODE="../scripts/abi_encode.py"

rm -rf test.db

echo 'test deploy fib_recur contract'
# deploy contract
/opt/chain_mockcli -f out/fib_recur.wasm --action deploy --print-time --enable-gas-meter -i 0x

echo 'test fibonacci(30)'
# query fibonacci(uint256)
FIB1_ABI_DATA=$($ABI_ENCODE "fibonacciTailOptimized(uint256)" "30")
output=$(/opt/chain_mockcli -f out/fib_recur.wasm --action call --print-time --enable-gas-meter -i $FIB1_ABI_DATA)
run_cmd_and_grep "$output" 'evm finish with result hex: 00000000000000000000000000000000000000000000000000000000000cb228'
