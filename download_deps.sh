#!/bin/bash
set -e

# get libclang_rt.builtins-wasm32.a from https://github.com/WebAssembly/wasi-sdk/releases/tag/wasi-sdk-12
wget https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-12/libclang_rt.builtins-wasm32-wasi-12.0.tar.gz
tar -xzf libclang_rt.builtins-wasm32-wasi-12.0.tar.gz
rm libclang_rt.builtins-wasm32-wasi-12.0.tar.gz
