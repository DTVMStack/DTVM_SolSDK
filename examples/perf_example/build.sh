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

forge build --extra-output-files ir-optimized

# build MyERC721
$YUL2WASM_PATH --input out/MyERC721.sol/MyERC721.iropt --output out/MyERC721.wasm $YUL2WASM_EXTRA_ARGS
wasm2wat -o out/MyERC721.wat out/MyERC721.wasm
echo 'MyERC721 compiled to wasm in out/MyERC721.wasm'

# build MyERC1155
$YUL2WASM_PATH --input out/MyERC1155.sol/MyERC1155.iropt --output out/MyERC1155.wasm $YUL2WASM_EXTRA_ARGS
wasm2wat -o out/MyERC1155.wat out/MyERC1155.wasm
echo 'MyERC1155 compiled to wasm in out/MyERC1155.wasm'

# build GLDToken
$YUL2WASM_PATH --input out/GLDToken.sol/GLDToken.iropt --output out/GLDToken.wasm $YUL2WASM_EXTRA_ARGS --enable-all-optimizers --default_ret_type u256
wasm2wat -o out/GLDToken.wat out/GLDToken.wasm
echo 'GLDToken compiled to wasm in out/GLDToken.wasm'

# build fib_recur
$YUL2WASM_PATH --input out/fib_recur.sol/FibonacciRecurTest.iropt --output out/fib_recur.wasm $YUL2WASM_EXTRA_ARGS
wasm2wat -o out/fib_recur.wat out/fib_recur.wasm
echo 'fib_recur compiled to wasm in out/fib_recur.wasm'

# build counter
echo "building counter contract"
$YUL2WASM_PATH --input out/counter.sol/counter.iropt --output out/counter.wasm $YUL2WASM_EXTRA_ARGS
wasm2wat -o out/counter.wat out/counter.wasm
echo 'counter compiled to wasm in out/counter.wasm'

# build TestToStringStore to test_to_string_store.wasm
$YUL2WASM_PATH --input out/test_to_string_store.sol/TestToStringStore.iropt --output out/test_to_string_store.wasm $YUL2WASM_EXTRA_ARGS
wasm2wat -o out/test_to_string_store.wat out/test_to_string_store.wasm
echo 'test_to_string_store compiled to wasm in out/test_to_string_store.wasm'