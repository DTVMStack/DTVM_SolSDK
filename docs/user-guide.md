# User Guide for DTVM_SolSDK

## Introduction

DTVM_SolSDK is a tool that compiles Ethereum Solidity smart contracts to WebAssembly (Wasm), enabling deployment on  Wasm-based blockchains. This guide will walk you through how to use DTVM_SolSDK to compile your Solidity contracts into WebAssembly.

Now the core tool of DTVM_SolSDK is yul2wasm.

## Installation

Before using DTVM_SolSDK, ensure you have the following prerequisites installed:

- Solidity Compiler 0.8.29(0.8.25+)
- LLVM 16
- Rust 1.81 or later
- Binaryen (for wasm tools)

For detailed installation instructions, refer to the [Developer Guide](developer-guide.md).

## Basic Usage

### Compiling a Solidity Contract to WebAssembly

The compilation process involves two main steps:

1. Compile Solidity to Yul IR using the Solidity compiler
2. Compile Yul IR to WebAssembly using yul2wasm

Here's a basic example:

```sh
# Step 1: Compile Solidity to Yul IR
solc --ir --optimize-yul -o output_directory --overwrite your_contract.sol

# Step 2: Compile Yul IR to WebAssembly
yul2wasm --input output_directory/ContractName.yul --output your_contract.wasm
```

### Command Line Options

yul2wasm provides several command-line options:

- `--input <file>`: Specify the input Yul file (required)
- `--output <file>`: Specify the output WebAssembly file (required)
- `--verbose`: Enable verbose output for debugging
- `--debug`: Generate debug information
- `--opt-level <level>`: Set LLVM optimization level (0-3, default: 3)

### Converting WebAssembly to Text Format (WAT)

For inspection or debugging, you can convert the binary WebAssembly to text format:

```sh
wasm2wat -o your_contract.wat your_contract.wasm
```

## Working with Examples

The `examples/` directory contains various Solidity contracts and scripts to help you understand how to use yul2wasm.

### ERC20 Token Example

To compile and test the ERC20 token example:

```sh
cd examples/erc20
./build_erc20.sh          # Compile the contract
./test_simple_token.sh    # Run tests
```

### Other Examples

The `examples/` directory includes several other examples:

- `foundry_erc20/`: ERC20 token using Foundry framework
- `nft/`: NFT (ERC721) implementation

## Integration with Development Workflows

### Automating Compilation

You can create a script similar to `examples/erc20/build_erc20.sh` to automate the compilation process:

```sh
#!/bin/bash
set -e

# Compile Solidity to Yul IR
solc --ir --optimize-yul -o . --overwrite your_contract.sol

# Compile Yul IR to WebAssembly
yul2wasm --input ContractName.yul --output your_contract.wasm

# Optional: Convert to WAT format for inspection
wasm2wat -o your_contract.wat your_contract.wasm
```

### Testing Compiled Contracts

The `examples/scripts/` directory contains various testing scripts that you can use as templates for testing your compiled contracts.

## Troubleshooting

### Common Issues

1. **Missing Dependencies**
   - Ensure all prerequisites are installed correctly.
   - Check version compatibility (especially for Solidity and LLVM).

2. **Compilation Errors**
   - Check Solidity syntax and version compatibility.
   - Examine verbose output with `--verbose` flag.
   - For complex errors, use `--debug` flag to generate more information.

3. **Execution Errors**
   - Verify memory management in your contract.
   - Check for stack overflow in complex functions.
   - Ensure proper ABI encoding/decoding for function calls.

### Getting Help

If you encounter issues not covered in this guide:

1. Check the GitHub repository issues section
2. Examine the test cases in the `examples/` directory
3. Refer to the developer guide for more technical details

### Security Best Practices

When developing contracts for deployment:

1. Follow standard Solidity security practices
2. Test extensively before deployment
3. Consider formal verification for critical contracts
