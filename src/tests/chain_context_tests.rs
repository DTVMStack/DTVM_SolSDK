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
    fn test_yul_gas() {
        let mut runtime = TestRuntime::new("GasTest", "target/test_yul_gas");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "GasTest" {
                code {
                }
                object "GasTest_deployed" {
                    code {
                        function test_gas() -> r {
                            r := gas()
                        }

                        let r := test_gas()
                        mstore(0x00, r)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_gas()"), &[]).unwrap();
        // DEFAULT_GAS_LIMIT: 10000000; DELTA_GAS: 100;
        runtime.assert_result("000000000000000000000000000000000000000000000000000000000098961c");
        // 9999900
    }

    #[test]
    fn test_yul_chainid() {
        let mut runtime = TestRuntime::new("ChainidTest", "target/test_yul_chainid");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "ChainidTest" {
                code {
                }
                object "ChainidTest_deployed" {
                    code {
                        function test_chainid() -> r {
                            r := chainid()
                        }

                        let r := test_chainid()
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
            .call(&solidity_selector("test_chainid()"), &[])
            .unwrap();
        // DEFAULT_CHAIN_ID: 1234
        runtime.assert_result("00000000000000000000000000000000000000000000000000000000000004d2");
    }

    #[test]
    fn test_yul_gasprice() {
        let mut runtime = TestRuntime::new("GaspriceTest", "target/test_yul_gasprice");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "GaspriceTest" {
                code {
                }
                object "GaspriceTest_deployed" {
                    code {
                        function test_gasprice() -> r {
                            r := gasprice()
                        }

                        let r := test_gasprice()
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
            .call(&solidity_selector("test_gasprice()"), &[])
            .unwrap();
        // DEFAULT_GAS_PRICE: 1
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000001");
    }

    #[test]
    fn test_yul_basefee() {
        let mut runtime = TestRuntime::new("BasefeeTest", "target/test_yul_basefee");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "BasefeeTest" {
                code {
                }
                object "BasefeeTest_deployed" {
                    code {
                        function test_basefee() -> r {
                            r := basefee()
                        }

                        let r := test_basefee()
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
            .call(&solidity_selector("test_basefee()"), &[])
            .unwrap();
        // DEFAULT_BASE_FEE: 0
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_yul_blobbasefee() {
        let mut runtime = TestRuntime::new("BolbbasefeeTest", "target/test_yul_blobbasefee");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "BolbbasefeeTest" {
                code {
                }
                object "BolbbasefeeTest_deployed" {
                    code {
                        function test_blobbasefee() ->r {
                            r := blobbasefee()
                        }

                        let r := test_blobbasefee()
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
            .call(&solidity_selector("test_blobbasefee()"), &[])
            .unwrap();
        // DEFAULT_BLOB_BASE_FEE: 0
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_yul_coinbase() {
        let mut runtime = TestRuntime::new("CoinbaseTest", "target/test_yul_coinbase");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "CoinbaseTest" {
                code {
                }
                object "CoinbaseTest_deployed" {
                    code {
                        function test_coinbase() ->r {
                            r := coinbase()
                        }

                        let r := test_coinbase()
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
            .call(&solidity_selector("test_coinbase()"), &[])
            .unwrap();
        // DEFAULT_COINBASE: 0000000000000000000000000303030303030303030303030303030303030303
        runtime.assert_result("0000000000000000000000000303030303030303030303030303030303030303");
    }

    #[test]
    fn test_yul_prevrandao() {
        let mut runtime = TestRuntime::new("PrevrandaoTest", "target/test_yul_prevrandao");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "PrevrandaoTest" {
                code {
                }
                object "PrevrandaoTest_deployed" {
                    code {
                        function test_prevrandao() ->r {
                            r := prevrandao()
                        }

                        let r := test_prevrandao()
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
            .call(&solidity_selector("test_prevrandao()"), &[])
            .unwrap();
        // DEFAULT_PREVRANDAO: 0000000000000000000000000505050505050505050505050505050505050505
        runtime.assert_result("0000000000000000000000000505050505050505050505050505050505050505");
    }

    #[test]
    fn test_yul_difficulty() {
        let mut runtime = TestRuntime::new("DifficultyTest", "target/test_yul_difficulty");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "DifficultyTest" {
                code {
                }
                object "DifficultyTest_deployed" {
                    code {
                        function test_difficulty() ->r {
                            r := difficulty() // same method as prevrandao
                        }

                        let r := test_difficulty()
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
            .call(&solidity_selector("test_difficulty()"), &[])
            .unwrap();
        // DEFAULT_PREVRANDAO: 0000000000000000000000000505050505050505050505050505050505050505
        runtime.assert_result("0000000000000000000000000505050505050505050505050505050505050505");
    }

    #[test]
    fn test_yul_timestamp() {
        let mut runtime = TestRuntime::new("TimestampTest", "target/test_yul_timestamp");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "TimestampTest" {
                code {
                }
                object "TimestampTest_deployed" {
                    code {
                        function test_timestamp() ->r {
                            r := timestamp()
                        }

                        let r := test_timestamp()
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
            .call(&solidity_selector("test_timestamp()"), &[])
            .unwrap();
        // DEFAULT_TIMESTAMP: 1234567890
        runtime.assert_result("00000000000000000000000000000000000000000000000000000000499602d2");
    }

    #[test]
    fn test_yul_number() {
        let mut runtime = TestRuntime::new("NumberTest", "target/test_yul_number");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "NumberTest" {
                code {
                }
                object "NumberTest_deployed" {
                    code {
                        function test_number() ->r {
                            r := number()
                        }

                        let r := test_number()
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
            .call(&solidity_selector("test_number()"), &[])
            .unwrap();
        // DEFAULT_NUMBER: 12345
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000003039");
    }

    #[test]
    fn test_yul_caller0() {
        let mut runtime = TestRuntime::new("Caller0Test", "target/test_yul_caller0");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "Caller0Test" {
                code {
                }
                object "Caller0Test_deployed" {
                    code {
                        function test_caller() -> r {
                            r := caller()
                        }

                        let r := test_caller()
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
            .call(&solidity_selector("test_caller()"), &[])
            .unwrap();
        // DEFAULT_SENDER_ADDRESS_HEX: 0x0011223344556677889900112233445566778899
        runtime.assert_result("0000000000000000000000000011223344556677889900112233445566778899");
    }

    #[test]
    fn test_yul_caller1() {
        let mut runtime = TestRuntime::new("Caller1Test", "target/test_yul_caller1");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "Caller1Test" {
                code {
                }
                object "Caller1Test_deployed" {
                    code {
                        function test_caller() -> r {
                            r := caller()
                        }

                        let r := test_caller()
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
            .call(&solidity_selector("test_caller()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000001234567890123456789012345678901234567890");
    }

    #[test]
    fn test_yul_caller2() {
        let mut runtime = TestRuntime::new("Caller2Test", "target/test_yul_caller2");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "Caller2Test" {
                code {
                    let _datasize := datasize("Caller2Test_deployed")
                    datacopy(0x00, dataoffset("Caller2Test_deployed"), _datasize)
                    return(0x00, _datasize)
                }

                object "Caller2Test_deployed" {
                    code {
                        // Deploy "ContractA" and test if `caller` opcode is correctly resolved in nested calls.
                        datacopy(0x00, dataoffset("ContractA"), datasize("ContractA"))
                        let addrContractA := create(0, 0x00, datasize("ContractA"))
                        mstore(0x00, 0x4d49c1ea) // selector for "test_caller()"
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
                                // Deploy "ContractB", call `test_caller()`, and verify returned `caller` address.
                                let _bDatasize := datasize("ContractB")
                                datacopy(0x00, dataoffset("ContractB"), _bDatasize)
                                let addrContractB := create(0, 0x00, _bDatasize)
                                mstore(0x00, 0x4d49c1ea) // selector for "test_caller()"
                                // Call "test_caller" in ContractB
                                let success := call(gas(), addrContractB, 0, 0x00, 0x04, 0x00, 0x20)
                                if iszero(success) { revert(0x00, 0x00) }

                                let callerFromB := mload(0x00)
                                mstore(0x00, 0) // Default output
                                // Check if caller value is the deployment address
                                if eq(callerFromB, address()) {
                                    mstore(0x00, 1) // Set output to 1 if match
                                }

                                return(0x00, 0x20)
                            }

                            object "ContractB" {
                                code {
                                }

                                object "ContractB_deployed" {
                                    code {
                                        function test_caller() -> result {
                                            result := caller()
                                        }
                                        mstore(0x00, test_caller())
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
            .call(&solidity_selector("test_caller()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000001");
    }
}
