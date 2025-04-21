#!/bin/bash
set -e

# read input yul file of erc20, compile it to wasm, then execute test
yul_file=$1
BUILD_MODE=${2:-release}

echo "Building in $BUILD_MODE mode"

# Set the yul2wasm path based on the build mode
if [ "$BUILD_MODE" == "release" ]; then
    YUL2WASM_PATH="../../target/release/yul2wasm"
else
    YUL2WASM_PATH="../../target/debug/yul2wasm"
fi

# if yul_file is not provided, or file not exist, then error
if [ -z "$yul_file" ] || [ ! -f "$yul_file" ]; then
    echo "Usage: $0 <path_to_yul_file> <mode>"
    exit 1
fi
output_wasm_file="${yul_file}.wasm"

# compile yul file
$YUL2WASM_PATH --input $yul_file --output $output_wasm_file --verbose --debug
echo "WASM file compiled to $output_wasm_file"

rm -f test.db

# SCRIPTS_DIR is the directory where the current script is located
SCRIPTS_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)

# execute test
$SCRIPTS_DIR/test_erc20.sh $output_wasm_file
