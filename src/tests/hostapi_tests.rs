// Copyright (C) 2024-2025 Ant Group Co., Ltd. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

#[allow(unused)]
use super::test_helper::solidity_selector;
#[allow(unused)]
use super::test_helper::TestRuntime;
#[cfg(test)]
mod tests {
    use super::*;

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
            "target/test_mstore_memory_guard/test_mstore_memory_guard.wasm",
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
}
