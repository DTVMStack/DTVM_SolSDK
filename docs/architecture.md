# Architecture Document

This document describes the architecture design, compilation process, and core components of DTVM_SolSDK, helping developers understand the internal workings of the project.

## Project Overview

yul2wasm is the core tool of DTVM_SolSDK, used to compile Ethereum Solidity smart contracts into WebAssembly, enabling them to run on Wasm-based blockchain platforms. The project maintains compatibility with the Ethereum smart contract model while leveraging WebAssembly's high performance and portability features.

## Compilation Process

The complete compilation process from Solidity to WebAssembly is as follows:

```
Solidity Source → Yul IR → LLVM IR → LLVM Optimization → WebAssembly → (Optional) Wasm Optimization
```

Detailed steps:

1. **Solidity to Yul IR**
   - Use Solidity compiler (solc) to compile Solidity code into Yul intermediate representation
     - Command: `solc --ir --optimize-yul -o output_directory --overwrite your_contract.sol`
   
   - Or use the Foundry framework for compilation (recommended for complex projects)
     - Configure foundry.toml:
       ```toml
       [profile.default]
       optimizer = true
       yul = true
       via_ir = true
       ```
     - Compilation command: `forge build --extra-output-files ir-optimized`
     - The generated IR file is located at `out/ContractName.sol/ContractName.iropt`

2. **Yul IR to LLVM IR**
   - Parse Yul IR and convert it to LLVM IR
   - This step is the core functionality of yul2wasm

3. **LLVM Optimization**
   - Apply various optimizations to LLVM IR
   - Use `-O2` optimization level by default

4. **LLVM IR to WebAssembly**
   - Generate WebAssembly binary code
   - Support various WebAssembly features and limitations

5. **Wasm Optimization (Optional)**
   - Use Binaryen tools to further optimize WebAssembly code
   - Reduce code size and improve execution efficiency

## System Architecture

yul2wasm consists of the following main components:

### 1. Yul Parser

- Located in `src/yul.lalrpop`
- Built using the LALRPOP parser generator
- Parses Yul IR text into an in-memory Abstract Syntax Tree (AST)

### 2. Yul2IR Module

- Located in `src/yul2ir/`
- Responsible for converting Yul AST to LLVM IR
- Main components:
  - `context.rs`: Maintains compilation context
  - `config.rs`: Handles compilation configuration options
  - `yul_instruction.rs`: Processes Yul instructions and types

### 3. Code Generator

- Utilizes the inkwell library (a Rust wrapper for LLVM)
- Generates optimized LLVM IR
- Applies various LLVM optimizations

### 4. Standard Library

- Located in the `stdlib/` directory
- Provides Ethereum-compatible runtime functionality
- Includes memory management, hash functions, EVM opcode simulations, and more

## Data Flow

During the compilation process, data flows in the following forms:

1. Yul IR text → Yul AST (via parser)
2. Yul AST → LLVM IR (via Yul2IR module)
3. LLVM IR → Optimized LLVM IR (via LLVM optimization)
4. Optimized LLVM IR → WebAssembly binary (via LLVM backend)

## Key Design Decisions

### 1. Memory Model

yul2wasm uses a linear memory model, consistent with WebAssembly's memory model. It includes:

- Using WebAssembly linear memory to store dynamic data
- A compatibility layer with Ethereum's EVM memory model
- Memory management to ensure proper memory allocation and deallocation

### 2. Type System

To bridge the gap between Yul and WebAssembly type systems:

- Yul operations primarily use 256-bit integers (U256)
- WebAssembly only natively supports 32-bit and 64-bit integers
- DTVM_SolSDK implements 256-bit operations through simulation

### 3. Optimization Strategy

The project employs multi-level optimization:

- Yul-level optimization (via Solidity compiler)
- LLVM optimization (via LLVM optimization pipeline)
- WebAssembly-specific optimization (via Binaryen)
- Custom optimization passes targeting common smart contract patterns

### 4. ABI Compatibility

To ensure compatibility with Ethereum ABI:

- Support for Ethereum ABI encoding and decoding standards
- Compatibility with Ethereum function calling conventions and slot rules
- Support for event logging functionality
