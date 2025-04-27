# DTVM_SolSDK - Solidity to WebAssembly Compiler

![GitHub license](https://img.shields.io/badge/license-Apache--2.0-blue.svg)
![GitHub stars](https://img.shields.io/github/stars/DTVMStack/DTVM_SolSDK?style=social)
![Solidity](https://img.shields.io/badge/solidity-0.8-blue.svg)

DTVM_SolSDK is an open-source compiler that enables Ethereum Solidity smart contracts to run on WebAssembly (Wasm) based blockchains. The compiler translates Solidity's Yul intermediate representation into optimized WebAssembly bytecode, maintaining compatibility with the Ethereum smart contract model.

Please visit the [project documentation](docs/user-guide.md) for more details.

## Background

In the blockchain ecosystem, WebAssembly (Wasm) has emerged as a fast, efficient, and portable execution environment. However, the vast majority of smart contracts are written in Solidity for the Ethereum Virtual Machine (EVM). DTVM_SolSDK bridges this gap by allowing developers to compile Solidity contracts to WebAssembly, enabling them to run on next-generation Wasm-based blockchain platforms while maintaining the familiar Ethereum development experience.

DTVM_SolSDK works by accepting Solidity's Yul intermediate representation (IR) and compiling it to optimized WebAssembly bytecode, ensuring compatibility with the Ethereum smart contract model and preserving all the original contract logic and behavior.

## Features

- Complete Solidity language support (version 0.8)
- LLVM-based optimization pipeline
- Ethereum ABI compatibility
- Support for complex Solidity patterns including DeFi protocols
- Multiple example contracts demonstrating capabilities

## Quick Start

### Dependencies

* Solidity Compiler 0.8.29/(0.8.25+)
* LLVM 16
* Rust 1.81 or later
* Binaryen (`brew install binaryen` on macOS, `apt install -y binaryen` on Ubuntu)

### Building the Project

```sh
cargo build --release
```

### Basic Usage

```sh
# Step 1: Compile Solidity to Yul IR
solc --ir --optimize-yul -o . --overwrite your_contract.sol

# Step 2: Compile Yul IR to WebAssembly
yul2wasm --input ContractName.yul --output your_contract.wasm
```

For complete usage instructions, see the [User Guide](docs/user-guide.md).

## Examples

The `examples/` directory contains various Solidity contracts and compilation scripts:

- [ERC20 Token](examples/erc20/) - Basic ERC20 token implementation
- [NFT Implementation](examples/nft/) - NFT (ERC721) implementation
- [Foundry ERC20](examples/foundry_erc20/) - ERC20 token using the Foundry framework
- [Counter](examples/counter/) - Simple counter contract
- [Fibonacci](examples/fibonacci/) - Fibonacci sequence calculator

### Compiling an Example

```sh
cd examples/erc20
./build_erc20.sh
```

This script will:
- Compile the `simple_erc20.sol` Solidity file to Yul IR
- Use DTVM_SolSDK to compile the Yul file to WebAssembly
- Convert the Wasm file to human-readable WebAssembly text format (WAT)

### Testing an Example

```sh
cd examples/erc20
./test_simple_token.sh
```

## Documentation

- [User Guide](docs/user-guide.md) - Instructions for using DTVM_SolSDK
- [Developer Guide](docs/developer-guide.md) - Information for developers contributing to the project

## Project Status

DTVM_SolSDK is actively developed and supports most Solidity language features. The compiler has been tested with various real-world smart contracts including DeFi protocols like Uniswap.

## Community

* [GitHub Issues](https://github.com/DTVMStack/DTVM_SolSDK/issues) - Bug reports and feature requests
* [Pull Requests](https://github.com/DTVMStack/DTVM_SolSDK/pulls) - Contribute to the project

## Contributing

Contributions are welcome! Please see our [Developer Guide](docs/developer-guide.md) for details on how to contribute.

1. Fork the repository
2. Create a feature branch (`git checkout -b feature-branch`)
3. Commit your changes (`git commit -am 'Add new feature'`)
4. Push to the branch (`git push origin feature-branch`)
5. Create a new Pull Request

Please adhere to our [Commit Convention](docs/COMMIT_CONVENTION.md) when making commits. All PRs are automatically validated against this standard.

## Versioning

This project follows [Semantic Versioning](https://semver.org/). For details on our version numbering scheme and release process, please see our [Versioning Guide](docs/VERSIONING.md).

## License

This project is licensed under the Apache-2.0 License.
