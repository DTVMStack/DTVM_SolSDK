// Copyright (C) 2024-2025 the DTVM authors. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

/// This file contains test cases for the solidity strings library usage.
///
/// The `Strings.sol` library is imported from the `@openzeppelin/contracts` package.
/// Url is https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/utils/Strings.sol.
/// The library provides functions for string operations, such as converting a `uint256` to a string.
/// The test cases in this file are used to test the correctness of the library functions.

#[allow(unused)]
use super::test_helper::solidity_selector;
#[allow(unused)]
use super::test_helper::TestRuntime;
#[cfg(test)]
mod tests {
    use super::*;

    // Embed the content of openzepplin_strings_full.sol file in the same directory
    // into the OPEN_ZEPPLIN_STRINGS_SOL_CODE global variable.
    const OPEN_ZEPPLIN_STRINGS_SOL_CODE: &str = include_str!("openzepplin_strings_full.sol");

    #[test]
    fn test_solidity_strings_to_string_1() {
        let mut runtime = TestRuntime::new(
            "test_solidity_strings_to_string_1",
            "target/test_solidity_strings_to_string_1",
        );
        runtime.clear_testdata();
        let yul_code = runtime.compile_solidity_to_yul(
            &format!(
                r#"
        pragma solidity ^0.8.0;
        {}
        
        contract TestContract {{
            function test() public returns (string memory) {{
                // test Strings.toString(uint256)
                return Strings.toString(123456789);
            }}
        }}
        "#,
                OPEN_ZEPPLIN_STRINGS_SOL_CODE
            ),
            "TestContract",
        );
        if let Err(err) = &yul_code {
            eprintln!("compile to yul error: {err}");
        }
        assert!(yul_code.is_ok());
        let yul_code = yul_code.unwrap();
        let _emited_bc = runtime.compile_test_yul(&yul_code).unwrap();
        runtime.set_enable_gas_meter(false);
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test()"), &[]).unwrap();

        // The string encoding of 123456789
        runtime.assert_result("000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000093132333435363738390000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_solidity_strings_to_hex_string_uint() {
        let mut runtime = TestRuntime::new(
            "test_solidity_strings_to_hex_string_uint",
            "target/test_solidity_strings_to_hex_string_uint",
        );
        runtime.clear_testdata();
        let yul_code = runtime.compile_solidity_to_yul(
            &format!(
                r#"
        pragma solidity ^0.8.0;
        {}
        contract TestContract {{
            function test() public pure returns (string memory) {{
                return Strings.toHexString(0xDEADBEEF);
            }}
        }}
        "#,
                OPEN_ZEPPLIN_STRINGS_SOL_CODE
            ),
            "TestContract",
        );
        assert!(yul_code.is_ok());
        let yul_code = yul_code.unwrap();
        let _emited_bc = runtime.compile_test_yul(&yul_code).unwrap();
        runtime.set_enable_gas_meter(false);
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test()"), &[]).unwrap();
        // The string encoding of "0xDEADBEEF"
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000a3078646561646265656600000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_solidity_strings_to_hex_string_address() {
        let mut runtime = TestRuntime::new(
            "test_solidity_strings_to_hex_string_address",
            "target/test_solidity_strings_to_hex_string_address",
        );
        runtime.clear_testdata();
        let yul_code = runtime.compile_solidity_to_yul(
            &format!(
                r#"
        pragma solidity ^0.8.0;
        {}
        contract TestContract {{
            function test() public pure returns (string memory) {{
                return Strings.toHexString(0x1234567890123456789012345678901234567890);
            }}
        }}
        "#,
                OPEN_ZEPPLIN_STRINGS_SOL_CODE
            ),
            "TestContract",
        );
        assert!(yul_code.is_ok());
        let yul_code = yul_code.unwrap();
        let _emited_bc = runtime.compile_test_yul(&yul_code).unwrap();
        runtime.set_enable_gas_meter(false);
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test()"), &[]).unwrap();
        // The string encoding of "0x1234567890123456789012345678901234567890"
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002a30783132333435363738393031323334353637383930313233343536373839303132333435363738393000000000000000000000000000000000000000000000");
    }
}
