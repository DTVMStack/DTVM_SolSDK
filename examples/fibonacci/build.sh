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

solc --ir --optimize-yul -o . --overwrite fib.sol
solc --ir --optimize-yul -o . --overwrite fib_recur.sol

$YUL2WASM_PATH --input FibonacciTest.yul --output fib.wasm $YUL2WASM_EXTRA_ARGS
wasm2wat -o fib.wat fib.wasm

$YUL2WASM_PATH --input FibonacciRecurTest.yul --output fib_recur.wasm $YUL2WASM_EXTRA_ARGS
wasm2wat -o fib_recur.wat fib_recur.wasm
