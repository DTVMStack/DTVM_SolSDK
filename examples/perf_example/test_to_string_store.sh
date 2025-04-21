#!/bin/bash
set -e

source ../scripts/common.sh

rm -f test.db

echo 'test deploy test_to_string_store contract'
# deploy contract (arg total supply(uint256))
/opt/chain_mockcli -f out/test_to_string_store.wasm --action deploy -i 0x

echo 'test tester(100)'
output=$(/opt/chain_mockcli -f out/test_to_string_store.wasm --action call --print-time -i 0xa667472f0000000000000000000000000000000000000000000000000000000000000064)
run_cmd_and_grep "$output" 'log data: 000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000053132333435000000000000000000000000000000000000000000000000000000'
