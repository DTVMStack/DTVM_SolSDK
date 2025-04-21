# Compilation Guide

This guide provides detailed steps for compiling DTVM_SolSDK from source, including all necessary dependencies and environment setup.

## System Requirements

DTVM_SolSDK can be built on the following operating systems:
- Linux (Ubuntu/Debian recommended)
- macOS
- Windows (via WSL)

## Dependencies

Before compiling DTVM_SolSDK, you need to install the following dependencies:

### Required Dependencies

1. **Rust 1.81+**
   - Rust language and its package manager Cargo
   
2. **LLVM 16**
   - Used for compilation and optimization

3. **Solidity Compiler 0.8.25+**
   - Used to compile Solidity code to Yul intermediate representation

## Installing Dependencies

### Ubuntu/Debian

```sh
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install LLVM 16
apt update
apt install -y llvm-16 llvm-16-dev

# Install Solidity Compiler
apt-get install software-properties-common
add-apt-repository ppa:ethereum/ethereum
apt-get update
apt-get install solc

# Install Binaryen
apt install -y binaryen
```

### macOS

```sh
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install LLVM 16
brew install llvm@16
# Add LLVM to PATH
echo 'export LLVM_SYS_160_PREFIX=/opt/homebrew/opt/llvm@16' >> ~/.bash_profile
echo 'export PATH="$LLVM_SYS_160_PREFIX/bin:$PATH"' >> ~/.bash_profile

# Install Solidity Compiler
brew tap ethereum/ethereum
brew install solidity

# Install Binaryen
brew install binaryen
```

## Download Source Code

Clone the repository from GitHub:

```sh
git clone https://github.com/DTVMStack/DTVM_SolSDK.git
cd DTVM_SolSDK
```

## Compile the Project

### Development Mode Compilation

```sh
make -f dev.makefile debug
```

The compiled binary will be located at `target/debug/yul2wasm`.

### Release Mode Compilation

```sh
make -f dev.makefile release
```

The compiled binary will be located at `target/release/yul2wasm`.

## Verify Installation

Verify that yul2wasm is compiled correctly:

```sh
./target/release/yul2wasm --help
```

You should see output similar to the following:

```
Compile Yul source to wasm

Usage: yul2wasm [OPTIONS] --input <INPUT> --output <OUTPUT>

Options:
      --input <INPUT>                  Input file path
      --output <OUTPUT>                Output wasm path
      --verbose                        Verbose output
      --debug                          Debug output
      --opt-level <OPT_LEVEL>          Optimization level [default: default]
      --main-contract <MAIN_CONTRACT>  Main contract name
      --symbol <PATH=ADDRESS>          Symbol path=address
      --ignore-unknown-linker-library  Ignore unknown linker library
      --no-binaryen-optimize           No binaryen optimize [default: true]
      --minify-wasm-size               Minify wasm size
      --disable-all-optimizers         Disable all optimizers
      --enable-all-optimizers          Enable all optimizers
      --enable-little-endian-storage-load-store  Enable little endian storage load/store
      --default_ret_type <DEFAULT_RET_TYPE>  Default return type [default: u256]
  -h, --help                           Print help
  -V, --version                        Print version
```

## Run Tests

Ensure all tests pass:

```sh
cargo test
```

## Compile Example Project

yul2wasm provides multiple examples, you can familiarize yourself with the tool's usage by compiling these examples:

```sh
cd examples/erc20
./build_erc20.sh
```

If everything goes well, you should see the generated WebAssembly file (`my_erc20.wasm`) and the readable text format (`my_erc20.wat`).

## Troubleshooting

### Common Issues

1. **LLVM Not Found**
   - Ensure LLVM 16 is correctly installed and added to PATH
   - For macOS: You can find the path using the command `brew --prefix llvm@16` and make sure the llvm16 bin directory is added to the PATH environment variable
   - For Linux: Check if the llvm-16-dev package is installed

2. **Solidity Compiler Issues**
   - Verify solc version: `solc --version` (should be 0.8.25 or higher)
   - If the version is too low, update to the latest version

3. **Dependency Conflicts**
   - Try cleaning the build: `cargo clean` and then rebuild

### Submitting Issues

If you encounter problems that you cannot resolve, please submit a detailed issue report on GitHub, including:
- Operating system and version
- Dependency versions (Rust, LLVM, Solidity)
- Error messages and logs
- Steps to reproduce the issue
