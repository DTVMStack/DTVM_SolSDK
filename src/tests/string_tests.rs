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
    fn test_string_constant() {
        let mut runtime = TestRuntime::new("test_string_constant", "target/test_string_constant");
        runtime.clear_testdata();
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "test_string_constant" {
                code {
                }

                object "test_string_constant_deployed" {
                    code {
                        function test_string() -> r {
                            mstore(200, "hello")
                            r := mload(200)
                        }

                        let result := test_string()
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
            .call(&solidity_selector("test_string()"), &[])
            .unwrap();
        runtime.assert_result("68656c6c6f000000000000000000000000000000000000000000000000000000");
    }
}
