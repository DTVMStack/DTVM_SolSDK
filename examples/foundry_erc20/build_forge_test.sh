#!/bin/bash
set -e

source ../../tools/build_utils.sh

# install solidity
# https://docs.soliditylang.org/en/latest/installing-solidity.html

# install foundry
# curl -L https://foundry.paradigm.xyz | bash

setup_build_mode ${1:-release}

forge clean
cp ../scripts/WasmTestVM.sol src/WasmTestVM.sol
forge test --extra-output-files ir-optimized
rm src/WasmTestVM.sol

YUL_IR_PATH="out"

# contracts to compile
CONTRACTS=(
    "WasmTestVM"
    "TestContract"
)

compile_all_contracts CONTRACTS[@] "$YUL_IR_PATH"
