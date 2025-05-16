#!/bin/bash
set -e

cd ..
source ../tools/forge_test_utils.sh
cd foundry_erc20

YUL_IR_PATH="out"
wasm_vm_file="$YUL_IR_PATH/WasmTestVM.wasm"
contract_file="$YUL_IR_PATH/TestContract.wasm"
# deploy WasmTestVM, Cheat code address from: abstract contract CommonBase, 0x7109709ECfa91a80626fF3989D68f67F5b1DD12D
WASM_TEST_VM_DEPLOY_ADDR=0x7109709ecfa91a80626ff3989d68f67f5b1dd12d
DEPLOYER_INITIALIZER_ADDR=0x11bbccddeeffaabbccddeeffaabbccddeeffaa11

run_single_test() {
    init_test "$wasm_vm_file" "$WASM_TEST_VM_DEPLOY_ADDR" "$contract_file" "$DEPLOYER_INITIALIZER_ADDR"

    local wasm_file=$1
    local function_name=$2
    local expected_result=$3
    call_contract_function "$wasm_file" "$DEPLOYER_INITIALIZER_ADDR" "$function_name" "$expected_output"
    echo "Test success: $function_name"
}

# testDeployAndTotalSupply() - 0x39eb0c5c
run_single_test $contract_file "testDeployAndTotalSupply()" 'evm finish with result hex: 00000000000000000000000000000000000000000000000000000000000003e8'
# testMint() - 0x9642ddaf
run_single_test $contract_file "testMint()" 'evm finish with result hex: 0000000000000000000000000000000000000000000000000000000000000007'
# testApproveAndAllowance() - 0xba5af22d
run_single_test $contract_file "testApproveAndAllowance()" 'evm finish with result hex: 0000000000000000000000000000000000000000000000000000000000000001'
# testTransfer() - 0xd591221f
run_single_test $contract_file "testTransfer()" 'evm finish with result hex: 00000000000000000000000000000000000000000000000000000000000003ea'
# testTransferFrom() - 0x70557298
run_single_test $contract_file "testTransferFrom()" 'evm finish with result hex:'
# testCompleteFlow() - 0xe44962e7
run_single_test $contract_file "testCompleteFlow()" 'evm finish with result hex:'

echo "All tests success!"
