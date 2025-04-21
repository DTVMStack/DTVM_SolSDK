#!/bin/bash
set -e

# install solidity
# https://docs.soliditylang.org/en/latest/installing-solidity.html

# Determine the build mode
BUILD_MODE=${1:-release}

echo "Building in $BUILD_MODE mode"

YUL2WASM_EXTRA_ARGS="--verbose"

# Set the yul2wasm path based on the build mode
if [ "$BUILD_MODE" == "release" ]; then
    YUL2WASM_PATH="../../target/release/yul2wasm"
else
    YUL2WASM_PATH="../../target/debug/yul2wasm"
    YUL2WASM_EXTRA_ARGS="--verbose --debug"
fi

solc --ir --optimize-yul -o . --overwrite test_init_code_hash.sol

$YUL2WASM_PATH --input TestInitCodeHashParent.yul --output test_init_code_hash.wasm $YUL2WASM_EXTRA_ARGS
wasm2wat -o test_init_code_hash.wat test_init_code_hash.wasm
