// Copyright (C) 2024-2025 the DTVM authors. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

#[allow(unused)]
use super::test_helper::solidity_selector;
#[allow(unused)]
use super::test_helper::TestRuntime;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yul2ir::transform::UNIFIED_REVERT_ERROR_ZERO;
    use std::io::{BufRead, BufReader};

    fn contains_string(file_path: &str, target: &str) -> std::io::Result<bool> {
        let file = std::fs::File::open(file_path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            if line?.contains(target) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    #[test]
    fn test_yul_opt_revert_zero1() {
        let mut runtime = TestRuntime::new("RevertZeroTest1", "target/test_revert_zero1");
        let emited_bc = runtime
            .compile_test_yul(
                r#"
            object "RevertZeroTest1" {
                code {
                }
                object "RevertZeroTest1_deployed" {
                    code {
                        function test_revert_error_1() {
                            revert(0, 0)
                        }
                        function test_revert_error_2() {
                            revert(0, 0)
                        }
                        function test_func() -> r {
                            r := 0
                            if callvalue() {
                                test_revert_error_1()
                            }
                            if iszero(lt(calldatasize(), 4)) {
                                r := 1
                            }
                            test_revert_error_2()
                        }

                        let r := test_func()
                        mstore(0x00, r)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        std::fs::write("target/test_revert_zero1/test_revert_zero1.wasm", emited_bc).unwrap();
        runtime.wasm2wat(
            "target/test_revert_zero1/test_revert_zero1.wasm",
            "target/test_revert_zero1/test_revert_zero1.wat",
        );
        assert!(contains_string(
            "target/test_revert_zero1/test_revert_zero1.wat",
            UNIFIED_REVERT_ERROR_ZERO
        )
        .unwrap());
        assert!(!contains_string(
            "target/test_revert_zero1/test_revert_zero1.wat",
            "test_revert_error_1"
        )
        .unwrap());
        assert!(!contains_string(
            "target/test_revert_zero1/test_revert_zero1.wat",
            "test_revert_error_2"
        )
        .unwrap());
        runtime.deploy(&[]).unwrap();
        runtime
            .call(&solidity_selector("test_func()"), &[])
            .unwrap();
    }

    #[test]
    fn test_yul_opt_revert_zero2() {
        let mut runtime = TestRuntime::new("RevertZeroTest2", "target/test_revert_zero2");
        let emited_bc = runtime
            .compile_test_yul(
                r#"
            object "RevertZeroTest2" {
                code {
                }
                object "RevertZeroTest2_deployed" {
                    code {
                        let r := test_func()
                        mstore(0x00, r)
                        return(0x00, 0x20)

                        function test_revert_error_1() {
                            revert(0, 0)
                        }
                        function test_revert_error_2() {
                            revert(0, 0)
                        }
                        function test_func() -> r {
                            r := 0
                            if callvalue() {
                                test_revert_error_1()
                            }
                            if iszero(lt(calldatasize(), 4)) {
                                r := 1
                            }
                            test_revert_error_2()
                        }
                    }
                }
            }
            "#,
            )
            .unwrap();
        std::fs::write("target/test_revert_zero2/test_revert_zero2.wasm", emited_bc).unwrap();
        runtime.wasm2wat(
            "target/test_revert_zero2/test_revert_zero2.wasm",
            "target/test_revert_zero2/test_revert_zero2.wat",
        );
        assert!(contains_string(
            "target/test_revert_zero2/test_revert_zero2.wat",
            UNIFIED_REVERT_ERROR_ZERO
        )
        .unwrap());
        assert!(!contains_string(
            "target/test_revert_zero2/test_revert_zero2.wat",
            "test_revert_error_1"
        )
        .unwrap());
        assert!(!contains_string(
            "target/test_revert_zero2/test_revert_zero2.wat",
            "test_revert_error_2"
        )
        .unwrap());
        runtime.deploy(&[]).unwrap();
        runtime
            .call(&solidity_selector("test_func()"), &[])
            .unwrap();
    }

    #[test]
    fn test_yul_opt_revert_zero3() {
        let mut runtime = TestRuntime::new("RevertZeroTest3", "target/test_revert_zero3");
        let emited_bc = runtime
            .compile_test_yul(
                r#"
            object "RevertZeroTest3" {
                code {
                }
                object "RevertZeroTest3_deployed" {
                    code {
                        let r := test_func()
                        mstore(0x00, r)
                        return(0x00, 0x20)

                        function test_func() -> r {
                            r := 0
                            if callvalue() {
                                test_revert_error_1()
                            }
                            if iszero(lt(calldatasize(), 4)) {
                                r := 1
                            }
                            test_revert_error_2()
                        }
                        function test_revert_error_1() {
                            revert(0, 0)
                        }
                        function test_revert_error_2() {
                            revert(0, 0)
                        }
                    }
                }
            }
            "#,
            )
            .unwrap();
        std::fs::write("target/test_revert_zero3/test_revert_zero3.wasm", emited_bc).unwrap();
        runtime.wasm2wat(
            "target/test_revert_zero3/test_revert_zero3.wasm",
            "target/test_revert_zero3/test_revert_zero3.wat",
        );
        assert!(contains_string(
            "target/test_revert_zero3/test_revert_zero3.wat",
            UNIFIED_REVERT_ERROR_ZERO
        )
        .unwrap());
        assert!(!contains_string(
            "target/test_revert_zero3/test_revert_zero3.wat",
            "test_revert_error_1"
        )
        .unwrap());
        assert!(!contains_string(
            "target/test_revert_zero3/test_revert_zero3.wat",
            "test_revert_error_2"
        )
        .unwrap());
        runtime.deploy(&[]).unwrap();
        runtime
            .call(&solidity_selector("test_func()"), &[])
            .unwrap();
    }
}
