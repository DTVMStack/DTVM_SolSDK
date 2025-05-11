// Copyright (C) 2024-2025 the DTVM authors. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

#[allow(unused)]
use super::test_helper::solidity_selector;
#[allow(unused)]
use super::test_helper::TestRuntime;
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yul_calldatasize() {
        let mut runtime = TestRuntime::new("CalldatasizeTest", "target/test_yul_calldatasize");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "CalldatasizeTest" {
                code {
                }
                object "CalldatasizeTest_deployed" {
                    code {
                        function test_calldatasize() -> r {
                            r := calldatasize()
                        }

                        let r := test_calldatasize()
                        mstore(0x00, r)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime
            .call(&solidity_selector("test_calldatasize()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000004");
    }

    #[test]
    fn test_yul_origin0() {
        let mut runtime = TestRuntime::new("Origin0Test", "target/test_yul_origin0");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "Origin0Test" {
                code {
                }
                object "Origin0Test_deployed" {
                    code {
                        function test_origin() -> r {
                            r := origin()
                        }

                        let r := test_origin()
                        mstore(0x00, r)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime
            .call(&solidity_selector("test_origin()"), &[])
            .unwrap();
        // DEFAULT_SENDER_ADDRESS_HEX: 0x0011223344556677889900112233445566778899
        runtime.assert_result("0000000000000000000000000011223344556677889900112233445566778899");
    }

    #[test]
    fn test_yul_origin1() {
        let mut runtime = TestRuntime::new("Origin1Test", "target/test_yul_origin1");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "Origin1Test" {
                code {
                }
                object "Origin1Test_deployed" {
                    code {
                        function test_origin() -> r {
                            r := origin()
                        }

                        let r := test_origin()
                        mstore(0x00, r)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.set_sender(Some(
            "0x1234567890123456789012345678901234567890".to_string(),
        ));
        runtime.deploy(&[]).unwrap();
        runtime
            .call(&solidity_selector("test_origin()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000001234567890123456789012345678901234567890");
    }

    #[test]
    fn test_yul_origin2() {
        let mut runtime = TestRuntime::new("Origin2Test", "target/test_yul_origin2");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "OriginTest" {
                code {
                    let _datasize := datasize("OriginTest_deployed")
                    datacopy(0x00, dataoffset("OriginTest_deployed"), _datasize)
                    return(0x00, _datasize)
                }

                object "OriginTest_deployed" {
                    code {
                        datacopy(0x00, dataoffset("ContractA"), datasize("ContractA"))
                        let addrContractA := create(0, 0x00, datasize("ContractA"))
                        mstore(0x00, 0xb7cdb9f0)
                        let success := call(gas(), addrContractA, 0, 0x00, 0x04, 0x00, 0x20)
                        if iszero(success) { revert(0x00, 0x00) } // Revert if failed

                        return(0x00, 0x20)
                    }

                    object "ContractA" {
                        code {
                            let _aDatasize := datasize("ContractA_deployed")
                            datacopy(0x00, dataoffset("ContractA_deployed"), _aDatasize)
                            return(0x00, _aDatasize)
                        }

                        object "ContractA_deployed" {
                            code {
                                let _bDatasize := datasize("ContractB")
                                datacopy(0x00, dataoffset("ContractB"), _bDatasize)
                                let addrContractB := create(0, 0x00, _bDatasize)
                                mstore(0x00, 0xb7cdb9f0)
                                let success := call(gas(), addrContractB, 0, 0x00, 0x04, 0x00, 0x20)
                                if iszero(success) { revert(0x00, 0x00) }

                                return(0x00, 0x20)
                            }

                            object "ContractB" {
                                code {
                                }

                                object "ContractB_deployed" {
                                    code {
                                        function test_origin() -> result {
                                            result := origin()
                                        }
                                        mstore(0x00, test_origin())
                                        return(0x00, 0x20)
                                    }
                                }
                            }
                        }
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.set_sender(Some(
            "0x1234567890123456789012345678901234567890".to_string(),
        ));
        runtime.deploy(&[]).unwrap();
        runtime
            .call(&solidity_selector("test_origin()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000001234567890123456789012345678901234567890");
    }

    #[test]
    fn test_yul_address() {
        let mut runtime = TestRuntime::new("AddressTest", "target/test_yul_address");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "AddressTest" {
                code {
                }
                object "AddressTest_deployed" {
                    code {
                        function test_address() -> r {
                            r := address()
                        }

                        let r := test_address()
                        mstore(0x00, r)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime
            .call(&solidity_selector("test_address()"), &[])
            .unwrap();
        // DEFAULT_RECEIVER_ADDRESS_HEX: 0xaabbccddeeffaabbccddeeffaabbccddeeffaabb
        runtime.assert_result("000000000000000000000000aabbccddeeffaabbccddeeffaabbccddeeffaabb");
    }

    #[test]
    fn test_yul_gaslimit() {
        let mut runtime = TestRuntime::new("GaslimitTest", "target/test_yul_gaslimit");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "GaslimitTest" {
                code {
                }
                object "GaslimitTest_deployed" {
                    code {
                        function test_gaslimit() -> r {
                            r := gaslimit()
                        }

                        let r := test_gaslimit()
                        mstore(0x00, r)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime
            .call(&solidity_selector("test_gaslimit()"), &[])
            .unwrap();
        // DEFAULT_GAS_LIMIT: 10000000
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000989680");
    }

    #[test]
    fn test_callvalue_not_zero() {
        let mut runtime =
            TestRuntime::new("test_callvalue_not_zero", "target/test_callvalue_not_zero");
        runtime.clear_testdata();
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "test_callvalue_not_zero" {
                code {
                }

                object "test_callvalue_not_zero_deployed" {
                    code {
                        function test_callvalue() -> r {
                           if callvalue() { revert(0, 0) }
                           r := 123
                        }

                        let result := test_callvalue()
                        mstore(0x00, result)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime
            .call(&solidity_selector("test_callvalue()"), &[])
            .unwrap();
        runtime.assert_result("000000000000000000000000000000000000000000000000000000000000007b");
    }

    #[test]
    fn test_mstore_memory_guard() {
        let mut runtime = TestRuntime::new(
            "test_mstore_memory_guard",
            "target/test_mstore_memory_guard",
        );
        runtime.clear_testdata();
        let emited_bc = runtime
            .compile_test_yul(
                r#"
            object "test_mstore_memory_guard" {
                code {
                }

                object "test_mstore_memory_guard_deployed" {
                    code {
                        let _1 := memoryguard(0x80)
                        let _2 := 64
                        mstore(_2, _1)
                        return(_2, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        std::fs::write(
            "target/test_mstore_memory_guard/test_mstore_memory_guard.wasm",
            emited_bc,
        )
        .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime
            .call(&solidity_selector("test_mstore_memory_guard()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000060");
    }

    #[test]
    fn test_call_ec_pair_failed() {
        let mut runtime = TestRuntime::new(
            "test_call_ec_pair_failed",
            "target/test_call_ec_pair_failed",
        );
        runtime.clear_testdata();
        let emited_bc = runtime
            .compile_test_yul(
                r#"
            object "test_call_ec_pair_failed" {
                code {
                }

                object "test_call_ec_pair_failed_deployed" {
                    code {
                        let size := calldatasize()
                        calldatacopy(0, 0, size)
                        let status := staticcall(0xffffffff, 8, 0, size, 0, 0x20)
                        let result := 0xfe
                        if status {
                            result := mload(0)
                        }
                        sstore(1, result)
                        return(0, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        std::fs::write(
            "target/test_call_ec_pair_failed/test_call_ec_pair_failed.wasm",
            emited_bc,
        )
        .unwrap();
        runtime.set_enable_gas_meter(false);
        runtime.deploy(&[]).unwrap();
        runtime
            .call(&solidity_selector("test_call_ec_pair_failed()"), &[])
            .unwrap();
        runtime.assert_result("f248a1c000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_solidity_revert_short_string() {
        let mut runtime = TestRuntime::new(
            "test_solidity_revert_short_string",
            "target/test_solidity_revert_short_string",
        );
        runtime.clear_testdata();
        let yul_code = runtime.compile_solidity_to_yul(
            r#"
        pragma solidity ^0.8.0;
 
        contract TestContract {
            function test() public {
                revert("helloworld");
            }
        }
        
        "#,
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
        runtime.assert_revert("08c379a00000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000a68656c6c6f776f726c6400000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_solidity_revert_long_string() {
        let mut runtime = TestRuntime::new(
            "test_solidity_revert_long_string",
            "target/test_solidity_revert_long_string",
        );
        runtime.clear_testdata();
        let yul_code = runtime.compile_solidity_to_yul(
            r#"
        pragma solidity ^0.8.0;
 
        contract TestContract {
            function test() public {
                revert("helloworld. This is a long revert string longer than 32bytes");
            }
        }
        
        "#,
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
        runtime.assert_revert("08c379a00000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000003c68656c6c6f776f726c642e20546869732069732061206c6f6e672072657665727420737472696e67206c6f6e676572207468616e203332627974657300000000");
    }
}
