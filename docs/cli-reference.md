# CLI Reference

This document provides a complete reference for the DTVM_SolSDK command-line tool yul2wasm, including detailed descriptions of all available parameters and options.

## Basic Usage

The basic usage of yul2wasm is as follows:

```
yul2wasm --input <input_file> --output <output_file> [options]
```

## Required Parameters

| Parameter | Description |
|------|------|
| `--input <file>` | Specifies the path to the input Yul file |
| `--output <file>` | Specifies the path to the output WebAssembly file |

## Optional Parameters

### Basic Options

| Option | Default | Description |
|------|--------|------|
| `--verbose` | No | Enables verbose output mode, displaying detailed information about the compilation process |
| `--debug` | No | Enables debug mode, generating debug information and intermediate files |
| `--opt-level <level>` | default | Sets the LLVM optimization level (available values: default, none, less, more, aggressive) |

### Contract-Related Options

| Option | Default | Description |
|------|--------|------|
| `--main-contract <name>` | (Auto-detect) | Specifies the main contract name; if not specified, uses the top-level object name in the file |
| `--symbol <path=address>` | None | Defines symbol path and address mapping; can be used multiple times to define multiple mappings |
| `--default_ret_type <type>` | u256 | Sets the default return type, available values: u256, bytes32 |

### Optimization Options

| Option | Default | Description |
|------|--------|------|
| `--disable-all-optimizers` | No | Disables all optimizers, used for debugging purposes |
| `--enable-all-optimizers` | No | Enables all possible optimizations to maximize performance |
| `--no-binaryen-optimize` | Yes | Disables Binaryen optimization |
| `--minify-wasm-size` | No | Enables additional WebAssembly size optimizations |

### Platform-Specific Options

| Option | Default | Description |
|------|--------|------|
| `--enable-little-endian-storage-load-store` | No | Enables little-endian storage load/store functionality |
| `--ignore-unknown-linker-library` | No | Ignores unknown linker library errors |

### Other Options

| Option | Description |
|------|------|
| `-h, --help` | Displays help information |
| `-V, --version` | Displays version information |

## Example Usage

### Basic Compilation

Compile a Solidity contract to WebAssembly:

```sh
# First use solc to compile Solidity to Yul
solc --ir --optimize-yul -o output_dir --overwrite MyContract.sol

# Then use yul2wasm to compile Yul to WebAssembly
yul2wasm --input output_dir/MyContract.yul --output MyContract.wasm
```

### Enable Verbose Output and Debug Information

```sh
yul2wasm --input MyContract.yul --output MyContract.wasm --verbose --debug
```

### Adjust Optimization Level

```sh
yul2wasm --input MyContract.yul --output MyContract.wasm --opt-level aggressive
```

### Specify Main Contract Name

```sh
yul2wasm --input MultipleContracts.yul --output MainContract.wasm --main-contract MainContract
```

### Define Symbol Mappings

```sh
yul2wasm --input MyContract.yul --output MyContract.wasm --symbol "lib.sol=0x1234..." --symbol "utils.sol=0xabcd..."
```

### Enable All Optimizations

```sh
yul2wasm --input MyContract.yul --output MyContract.wasm --enable-all-optimizers
```

### Minimize WebAssembly Size

```sh
yul2wasm --input MyContract.yul --output MyContract.wasm --minify-wasm-size
```

## Output Files

When yul2wasm compiles successfully, it generates the following files:

1. **Main WebAssembly Binary File** (file name specified by `--output`)
   - Contains the compiled WebAssembly code

2. **Contract Binary File** (`.cbin` extension)
   - Contains the complete contract code for deployment, including the Wasm length prefix

3. **Contract Hexadecimal File** (`.cbin.hex` extension)
   - Hexadecimal representation of the contract binary file

4. **Additional Debug Mode Files** (generated only when using the `--debug` option)
   - LLVM IR files (`.ll`)
   - Assembly files (`.s`)
   - Other intermediate files

## Return Codes

| Code | Description |
|------|------|
| 0 | Success |
| 1 | General error (file read/write errors, parameter parsing errors, etc.) |
| > 1 | Specific error codes, see error message for details |

## Environment Variables

DTVM_SolSDK does not depend on any specific environment variables, but it requires that related tools (such as LLVM) are available in the system PATH. 
