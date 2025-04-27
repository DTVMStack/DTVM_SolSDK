# Quick Start Guide for DTVM_SolSDK

## Prerequisites

The fastest way to set up the compilation environment is to use a Docker image or build it based on the provided Dockerfile:

```bash
docker pull dtvmdev1/dtvm-sol-dev-x64:main
```

Before using DTVM_SolSDK, ensure the following dependencies are installed on your system:

- **solc** (Solidity compiler) or **Foundry**
- **Binaryen** (Optional)

### Installing Solidity Compiler (solc)

Download the Solidity compiler from:
[https://github.com/ethereum/solidity/releases](https://github.com/ethereum/solidity/releases)

### Installing Foundry

Install Foundry from:
[https://getfoundry.sh/](https://getfoundry.sh/)

## Basic Usage

### Compiling a Solidity Contract to WebAssembly

The compilation process involves two main steps:

1. Compile Solidity to Yul IR using the Solidity compiler
2. Compile Yul IR to WebAssembly using yul2wasm

Here's a basic example:

```bash
# Step 1: Compile Solidity to Yul IR
solc --ir --optimize-yul -o output_directory --overwrite your_contract.sol

# Step 2: Compile Yul IR to WebAssembly
yul2wasm --input output_directory/ContractName.yul --output your_contract.wasm
```

If you're using Foundry for your project, here's how to compile your Solidity contract to WebAssembly:

```bash
# Step1: Compile foundry project to Yul IR
forge build --extra-output-files ir-optimized

# If your solidity filename and contract name is ExampleContract

# Step2: Compile Yul IR to WebAssembly
yul2wasm  --input out/ExampleContract.sol/ExampleContract.iropt --output out/ExampleContract.wasm
```

### Command Line Options

yul2wasm provides several command-line options:

| Option | Description |
|--------|-------------|
| `--input <file>` | Specify the input Yul file (required) |
| `--output <file>` | Specify the output WebAssembly file (required) |
| `--verbose` | Enable verbose output for debugging |
| `--debug` | Generate debug information |
| `--opt-level <level>` | Set LLVM optimization level (0-3, default: 3) |

### Output File Types

When working with yul2wasm, you'll encounter several file types:

- `.wasm`: WebAssembly binary format - the final compiled contract that can be deployed on Wasm-based blockchains
- `.cbin`: Contract binary format - contains the compiled bytecode of the contract
- `.cbin.hex`: Hexadecimal representation of the contract binary - useful for deployment and verification

### Converting WebAssembly to Text Format (WAT)

For inspection or debugging, you can convert the binary WebAssembly to text format:

```bash
wasm2wat -o your_contract.wat your_contract.wasm
```

## Troubleshooting

For common issues, security best practices, and more detailed information, please contact us through:
[https://github.com/DTVMStack/DTVM_SolSDK/issues](https://github.com/DTVMStack/DTVM_SolSDK/issues)
