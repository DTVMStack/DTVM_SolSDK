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
    fn test_i32_cast_to_u256() {
        let mut runtime = TestRuntime::new("test_i32_cast_to_u256", "target/test_i32_cast_to_u256");
        runtime.clear_testdata();
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "test_i32_cast_to_u256" {
                code {
                }

                object "test_i32_cast_to_u256_deployed" {
                    code {
                        function test_cast(a) -> r {
                            debug_print(a)
                            r := a
                        }

                        let result := test_cast(calldatasize())
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
            .call(&solidity_selector("test_cast()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000004");
    }
}
