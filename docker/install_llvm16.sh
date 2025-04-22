#!/bin/bash
set -e
cd /opt
ls
tar -xvf clang+llvm-16.0.4-x86_64-linux-gnu-ubuntu-22.04.tar.xz
rm -rf clang+llvm-16.0.4-x86_64-linux-gnu-ubuntu-22.04.tar.xz
mv clang+llvm-16.0.4-x86_64-linux-gnu-ubuntu-22.04 llvm16
