#!/bin/bash
set -e

source ../scripts/common.sh

rm -f test.db

echo 'test deploy TokenFactory contract $wasm_file'
# deploy contract (arg total supply(uint256))
/opt/chain_mockcli -f out/TokenFactory.wasm --action deploy -s 0x9988776655443322119900112233445566778899 -i 0x
# Call createToken(uint256 initialSupply=0x68656c6c6f000000000000000000000000000000000000000000000000000000)
# to deploy new MyToken contract and return token contract address
# The output will contain "evm finish with result hex: <32-byte hex contract address>",
# which needs to be extracted using grep

echo 'test createToken function'
output=$(/opt/chain_mockcli -f out/TokenFactory.wasm --action call --print-time -s 0x9988776655443322119900112233445566778899 -i 0x2d571cc468656c6c6f000000000000000000000000000000000000000000000000000000)
run_cmd_and_grep "$output" 'evm finish with result hex:'

token_contract_address=$(grep_output_last_result_address "$output")
factory_contract_address="aabbccddeeffaabbccddeeffaabbccddeeffaabb"

echo "deployed ERC20 token contract address: $token_contract_address"

# Test the generated token contract address
../scripts/test_erc20_by_address.sh $token_contract_address '9988776655443322119900112233445566778899'
