#!/bin/bash
set -e

rm -f test.db

../scripts/test_erc20.sh my_erc20.wasm
