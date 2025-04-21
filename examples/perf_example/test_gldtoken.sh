#!/bin/bash
set -e

rm -f test.db

../scripts/test_erc20.sh out/GLDToken.wasm
