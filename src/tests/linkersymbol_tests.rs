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
    fn test_linker_symbol_1() {
        let mut runtime = TestRuntime::new("test_linker_symbol_1", "target/test_linker_symbol_1");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "test_linker_symbol_1" {
                code {
                }
                object "test_linker_symbol_1_deployed" {
                    code {
                        function test_linker_symbol() -> r {
                            let a := 3
                            let b := 5
                            r := lt(a, b)
                        }
                        // not used linkersymbol variable
                        let l1 := linkersymbol("l1")
                        let r := test_linker_symbol()
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
            .call(&solidity_selector("test_linker_symbol()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000001");
    }
}
