#!/bin/bash
set -e

setup_build_mode() {
    local BUILD_MODE=${1:-release}
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

    export YUL2WASM_PATH
    export YUL2WASM_EXTRA_ARGS
}

compile_contract() {
    local contract=$1
    local YUL_IR_PATH=$2
    echo "Compiling $contract..."

    $YUL2WASM_PATH --input $YUL_IR_PATH/$contract.sol/$contract.iropt \
                   --output $YUL_IR_PATH/$contract.wasm \
                   $YUL2WASM_EXTRA_ARGS

    wasm2wat -o $YUL_IR_PATH/$contract.wat $YUL_IR_PATH/$contract.wasm

    if [ -f "$YUL_IR_PATH/$contract.wasm" ]; then
        echo "Successfully compiled $contract to $YUL_IR_PATH/$contract.wasm"
    else
        echo "Error: Failed to compile $contract" >&2
        exit 1
    fi
}

compile_all_contracts() {
    local contracts=("${!1}")
    local YUL_IR_PATH=$2

    for contract in "${contracts[@]}"; do
        compile_contract "$contract" "$YUL_IR_PATH"
    done

    echo "All contracts compiled successfully"
}
