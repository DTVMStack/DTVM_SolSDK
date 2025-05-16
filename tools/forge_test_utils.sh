#!/bin/bash
set -e

source ../examples/scripts/common.sh
ABI_ENCODE="../scripts/abi_encode.py"
MOCKCLI_PATH="/opt/chain_mockcli"

DEPLOYER_SENDER=0x9988776655443322119900112233445566778899

cleanup() {
    if [ -f test.db ]; then
        rm -f test.db
    fi
}

deploy_contract() {
    local wasm_file=$1
    local deploy_addr=$2
    echo "deploy contract: $wasm_file"
    $MOCKCLI_PATH -f $wasm_file --action deploy -s $DEPLOYER_SENDER -t $deploy_addr -i 0x
}

call_contract_function() {
    local wasm_file=$1
    local deploy_addr=$2
    local function_name=$3
    local expected_output=$4
    echo "call contract $wasm_file function $function_name"
    ABI_DATA=$($ABI_ENCODE "$function_name")
    output=$($MOCKCLI_PATH -f $wasm_file -t $deploy_addr --action call --print-time --enable-gas-meter -s $DEPLOYER_SENDER -i $ABI_DATA)
    run_cmd_and_grep "$output" "$expected_output"
}

init_test() {
    cleanup

    local wasm_vm_file=$1
    local WASM_TEST_VM_DEPLOY_ADDR=$2
    local contract_file=$3
    local DEPLOYER_INITIALIZER_ADDR=$4

    deploy_contract "$wasm_vm_file" "$WASM_TEST_VM_DEPLOY_ADDR"
    deploy_contract "$contract_file" "$DEPLOYER_INITIALIZER_ADDR"

    call_contract_function "$contract_file" "$DEPLOYER_INITIALIZER_ADDR" "setUp()" 'evm finish with result hex:'
}
