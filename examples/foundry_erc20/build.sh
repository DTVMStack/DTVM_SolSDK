#!/bin/bash
set -e

# install solidity
# https://docs.soliditylang.org/en/latest/installing-solidity.html

# install foundry
# curl -L https://foundry.paradigm.xyz | bash

# Determine the build mode
BUILD_MODE=${1:-release}

echo "Building in $BUILD_MODE mode"

# --enable-little-endian-storage-load-store
YUL2WASM_EXTRA_ARGS="--verbose"

# if env ENABLE_LITTLE_ENDIAN_STORAGE == "ON", then add --enable-little-endian-storage-load-store
if [ "$ENABLE_LITTLE_ENDIAN_STORAGE" == "ON" ]; then
    YUL2WASM_EXTRA_ARGS="$YUL2WASM_EXTRA_ARGS --enable-little-endian-storage-load-store"
fi

# Set the yul2wasm path based on the build mode
if [ "$BUILD_MODE" == "release" ]; then
    YUL2WASM_PATH="../../target/release/yul2wasm"
else
    YUL2WASM_PATH="../../target/debug/yul2wasm"
    YUL2WASM_EXTRA_ARGS="$YUL2WASM_EXTRA_ARGS --debug"
fi

# npm install @openzeppelin/contracts
# solc --ir --optimize-yul -o . --overwrite TokenFactory.sol MyToken.sol

forge build --extra-output-files ir-optimized
# for debug: forge build --extra-output-files ir
# ir generated in out/TokenFactory.sol/TokenFactory.ir

$YUL2WASM_PATH --input out/MyToken.sol/MyToken.iropt --output out/MyToken.wasm $YUL2WASM_EXTRA_ARGS
wasm2wat -o out/MyToken.wat out/MyToken.wasm
echo 'MyToken compiled to wasm in out/MyToken.wasm'

# Compile TokenFactory, which will be used to deploy MyToken
$YUL2WASM_PATH --input out/TokenFactory.sol/TokenFactory.iropt --output out/TokenFactory.wasm $YUL2WASM_EXTRA_ARGS
wasm2wat -o out/TokenFactory.wat out/TokenFactory.wasm
echo 'TokenFactory compiled to wasm in out/TokenFactory.wasm'
