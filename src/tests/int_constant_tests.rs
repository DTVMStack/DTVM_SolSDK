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
    fn test_simple_u256_decimal() {
        let mut runtime = TestRuntime::new(
            "test_simple_u256_decimal",
            "target/test_simple_u256_decimal",
        );
        runtime.clear_testdata();
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "test_simple_u256_decimal" {
                code {
                }

                object "test_simple_u256_decimal_deployed" {
                    code {
                        function test_int() -> r {
                            let a := 123
                            r := a
                        }

                        let result := test_int()
                        mstore(0x00, result)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_int()"), &[]).unwrap();
        runtime.assert_result("000000000000000000000000000000000000000000000000000000000000007b");
    }

    #[test]
    fn test_simple_u256_hex() {
        let mut runtime = TestRuntime::new("test_simple_u256_hex", "target/test_simple_u256_hex");
        runtime.clear_testdata();
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "test_simple_u256_hex" {
                code {
                }

                object "test_simple_u256_hex_deployed" {
                    code {
                        function test_int() -> r {
                            let a := 0x7b
                            r := a
                        }

                        let result := test_int()
                        mstore(0x00, result)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_int()"), &[]).unwrap();
        runtime.assert_result("000000000000000000000000000000000000000000000000000000000000007b");
    }

    #[test]
    fn test_very_big_u256_decimal() {
        let mut runtime = TestRuntime::new(
            "test_very_big_u256_decimal",
            "target/test_very_big_u256_decimal",
        );
        runtime.clear_testdata();
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "test_very_big_u256_decimal" {
                code {
                }

                object "test_very_big_u256_decimal_deployed" {
                    code {
                        function test_int() -> r {
                            /// let a := 0xffffffffffffffffffffffffffff123fffffffffffffffffffffffffffffffff
                            let a := 115792089237316195423570985008687887142324004389641923804625477280772707581951
                            r := a
                        }

                        let result := test_int()
                        mstore(0x00, result)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_int()"), &[]).unwrap();
        runtime.assert_result("ffffffffffffffffffffffffffff123fffffffffffffffffffffffffffffffff");
    }

    #[test]
    fn test_very_big_u256_hex() {
        let mut runtime =
            TestRuntime::new("test_very_big_u256_hex", "target/test_very_big_u256_hex");
        runtime.clear_testdata();
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "test_very_big_u256_hex" {
                code {
                }

                object "test_very_big_u256_hex_deployed" {
                    code {
                        function test_int() -> r {
                            let a := 0xffffffffffffffffffffffffffff123fffffffffffffffffffffffffffffffff
                            r := a
                        }

                        let result := test_int()
                        mstore(0x00, result)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_int()"), &[]).unwrap();
        runtime.assert_result("ffffffffffffffffffffffffffff123fffffffffffffffffffffffffffffffff");
    }

    #[test]
    fn test_max_u256() {
        let mut runtime = TestRuntime::new("test_max_u256", "target/test_max_u256");
        runtime.clear_testdata();
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "test_max_u256" {
                code {
                }

                object "test_max_u256_deployed" {
                    code {
                        function test_int() -> r {
                            let a := 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
                            r := a
                        }

                        let result := test_int()
                        mstore(0x00, result)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_int()"), &[]).unwrap();
        runtime.assert_result("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
    }

    #[test]
    fn test_arithmetic_operations() {
        let mut runtime = TestRuntime::new(
            "test_arithmetic_operations",
            "target/test_arithmetic_operations",
        );
        runtime.clear_testdata();
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "test_arithmetic_operations" {
                code {
                }

                object "test_arithmetic_operations_deployed" {
                    code {
                        function test_int() -> r {
                            let a := 123
                            let b := 456
                            let c := add(a, b)
                            r := c
                        }

                        let result := test_int()
                        mstore(0x00, result)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_int()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000243");
    }

    #[test]
    fn test_bitwise_operations1() {
        let mut runtime = TestRuntime::new(
            "test_bitwise_operations1",
            "target/test_bitwise_operations1",
        );
        runtime.clear_testdata();
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "test_bitwise_operations1" {
                code {
                }

                object "test_bitwise_operations1_deployed" {
                    code {
                        function test_int() -> r {
                            let a := 0x0f0f0f0f
                            let b := 0xf0f0f0f0
                            let c := and(a, b)
                            r := c
                        }

                        let result := test_int()
                        mstore(0x00, result)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_int()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_bitwise_operations2() {
        let mut runtime = TestRuntime::new(
            "test_bitwise_operations2",
            "target/test_bitwise_operations2",
        );
        runtime.clear_testdata();
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "test_bitwise_operations2" {
                code {
                }

                object "test_bitwise_operations2_deployed" {
                    code {
                        function test_int() -> r {
                            let a := 0x0f1f0f0f
                            let b := 0xf0f0f0f0
                            let c := and(a, b)
                            r := c
                        }

                        let result := test_int()
                        mstore(0x00, result)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_int()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000100000");
    }
}
