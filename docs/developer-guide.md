# Developer Guide for DTVM_SolSDK

## Project Overview
DTVM_SolSDK is a compiler that translates Solidity's Yul intermediate representation into WebAssembly (Wasm), enabling Ethereum smart contracts to run in Wasm environments.

## Development Environment Setup

### Prerequisites
- Rust 1.81 or newer
- LLVM 16
- Solidity Compiler 0.8.25
- Binaryen (for wasm tools like wasm2wat)

### Installation Instructions

#### Ubuntu/Debian
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

#### macOS
```sh
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install LLVM 16
brew install llvm@16

# Install Solidity Compiler
brew tap ethereum/ethereum
brew install solidity

# Install Binaryen
brew install binaryen
```

## Project Structure
- `docker/` - Dockerfile and related scripts for building the compiler
- `src/` - Core compiler source code
  - `tests/` - Compiler test suite
  - `yul2ir/` - Yul IR parser and LLVM IR generator
- `stdlib/` - Standard library modules that provide common functionality and utilities
- `lib` - Additional libraries or external dependencies
- `tools` - Scripts for various tasks like formatting
- `examples/` - Example Solidity contracts and compilation scripts
- `docs/` - Project documentation

## Build Process

1. **Compilation Pipeline:**
   - Solidity source → Yul IR (using solc compiler)
   - Yul IR → WebAssembly (using yul2wasm)

2. **Compilation Stages:**
   - Parsing Yul IR
   - Generating LLVM IR
   - Optimizing LLVM IR
   - Generating WebAssembly binary

## Adding Features

When adding new features to yul2wasm:

1. **Understanding Yul IR**:
   - Familiarize yourself with Solidity's Yul Intermediate Representation syntax and semantics
   - Reference: https://docs.soliditylang.org/en/latest/yul.html

2. **Making Compiler Changes**:
   - Modify the parser for new Yul syntax support
   - Update LLVM IR generation for new features
   - Add appropriate tests to verify the functionality

3. **Testing**:
   - Unit tests: `cargo test`
   - Integration tests: Use example contracts in `examples/` directory

## Debugging

1. **Debug Logging**:
   - Use the `--verbose` flag when running yul2wasm
   - For advanced debugging, use the `--debug` flag to generate additional debug information

2. **Examining LLVM IR and Assembly**:
   - Intermediate files are generated during compilation with proper flags
   - Check `.ll` files for LLVM IR and `.s` files for assembly output

3. **Testing with Example Contracts**:
   - The `examples/` directory contains various smart contracts to test against
   - Use these as benchmarks for your changes

## Contributing Guidelines

1. **Code Style**:
   - Follow Rust code conventions
   - Run `cargo fmt` before submitting code
   - Use `cargo clippy` for linting

2. **Pull Request Process**:
   - Create a new branch for your feature/fix
   - Include tests for new functionality
   - Update documentation as needed
   - Submit a PR with a clear description of changes

3. **Documentation**:
   - Update relevant documentation in the `docs/` directory
   - Include examples for new features
   - Comment complex code sections

## Performance Considerations

When optimizing the compiler:

1. **LLVM Optimization Levels**:
   - Default is `-O2` for maximum optimization
   - Consider compilation time vs. performance trade-offs

2. **Wasm Size Optimization**:
   - Binaryen's `wasm-opt` can further optimize the generated Wasm

## Common Issues and Solutions

- **Memory Management**: WebAssembly has specific memory models; ensure proper memory handling
- **Stack Limitations**: Watch for stack usage in complex contracts
- **ABI Compatibility**: Ensure compatibility with Ethereum ABI encoding/decoding
