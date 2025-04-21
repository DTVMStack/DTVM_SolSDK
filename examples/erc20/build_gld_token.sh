#!/bin/bash
set -e

# install solidity 0.8.29
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

solc --ir --via-ir --optimize-yul --optimize-runs 200 --optimize -o ./out --overwrite --bin @openzeppelin/contracts=./node_modules/@openzeppelin/contracts/ GLDToken.sol

$YUL2WASM_PATH --input ./out/GLDToken.yul --output GLDToken.wasm $YUL2WASM_EXTRA_ARGS
wasm2wat -o GLDToken.wat GLDToken.wasm
