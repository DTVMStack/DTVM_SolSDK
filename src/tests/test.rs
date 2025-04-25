// Copyright (C) 2024-2025 the DTVM authors, Ltd. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

#[allow(unused)]
use super::test_helper::solidity_selector;
#[allow(unused)]
use super::test_helper::TestRuntime;

#[test]
fn test_yul_add() {
    let mut runtime = TestRuntime::new("AddTest", "target/test_yul_add");
    runtime.clear_testdata();
    let _emited_bc = runtime
        .compile_test_yul(
            r#"
        object "AddTest" {
            code {
            }
            object "AddTest_deployed" {
                code {
                    function test_add() -> r {
                        let a := 1
                        let b := 2
                        r := add(a, b)
                    }

                    let r := test_add()
                    mstore(0x00, r)
                    return(0x00, 0x20)
                }
            }
        }
        "#,
        )
        .unwrap();
    runtime.deploy(&[]).unwrap();
    runtime.call(&solidity_selector("test_add()"), &[]).unwrap();
    runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000003");
}
