#!/bin/bash

set -e

CUR_PATH=`pwd`
cd ../../stdlib
make clean
make release
cd $CUR_PATH
cargo build --release

cd $CUR_PATH

# build in release mode
./build.sh release

# test generated released mode wasm files
echo 'testing generated release mode wasm files...'
./test_gldtoken.sh
./test_erc721.sh
./test_erc1155.sh
./test_fib.sh
echo 'tests done'

# package wasm files and source files, test scripts to tar.gz
tar czf test_perf_token_wasm.tar.gz out/MyERC721.wasm out/MyERC1155.wasm out/counter.wasm out/fib_recur.wasm out/GLDToken.wasm test_erc721.sh test_erc1155.sh test_gldtoken.sh test_fib.sh src/MyERC721.sol src/MyERC1155.sol src/counter.sol src/fib_recur.sol src/GLDToken.sol
echo "test_perf_token_wasm.tar.gz created"
