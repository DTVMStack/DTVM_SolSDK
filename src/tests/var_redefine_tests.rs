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
    fn test_yul_var_redefine1() {
        let mut runtime = TestRuntime::new("VarRedefineTest1", "target/test_yul_var_redefine1");
        let result = runtime.compile_test_yul(
            r#"
            object "VarRedefineTest1" {
                code {
                }
                object "VarRedefineTest1_deployed" {
                    code {
                        function test_byte() -> r {
                            let index := 30
                            let value := 23
                            let value := 256 // error: value is redefined in the same scope
                            r := byte(index, value)
                        }

                        let r := test_byte()
                        mstore(0x00, r)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
        );
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Variable 'value' is already defined in this scope"));
    }

    #[test]
    fn test_yul_var_redefine2() {
        let mut runtime = TestRuntime::new("VarRedefineTest2", "target/test_yul_var_redefine2");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "VarRedefineTest2" {
                code {
                }
                object "VarRedefineTest2_deployed" {
                    code {
                        function test_byte() -> r {
                            let index := 30
                            let value := 23
                            value := 256 // ok: value is reassigned in the same scope
                            r := byte(index, value)
                        }

                        let r := test_byte()
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
            .call(&solidity_selector("test_byte()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000001");
    }

    #[test]
    fn test_yul_var_redefine3() {
        let mut runtime = TestRuntime::new("VarRedefineTest3", "target/test_yul_var_redefine3");
        let result = runtime.compile_test_yul(
            r#"
            object "VarRedefineTest3" {
                code {
                }
                object "VarRedefineTest3_deployed" {
                    code {
                        function test_byte() -> r {
                            let index := 30
                            let r := 256 // error: r is redefined in the same scope
                            r := byte(index, r)
                        }

                        let r := test_byte()
                        mstore(0x00, r)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
        );
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Variable 'r' is already defined in this scope"));
    }

    #[test]
    fn test_yul_var_redefine4() {
        let mut runtime = TestRuntime::new("VarRedefineTest4", "target/test_yul_var_redefine4");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "VarRedefineTest4" {
                code {
                }
                object "VarRedefineTest4_deployed" {
                    code {
                        function test_byte() -> r {
                            let index := 30
                            r := 256            // ok: r is assigned in the same scope
                            r := byte(index, r) // ok: r is reassigned in the same scope
                        }

                        let r := test_byte()
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
            .call(&solidity_selector("test_byte()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000001");
    }

    #[test]
    fn test_yul_var_redefine5() {
        let mut runtime = TestRuntime::new("VarRedefineTest5", "target/test_yul_var_redefine5");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "VarRedefineTest5" {
                code {
                }
                object "VarRedefineTest5_deployed" {
                    code {
                        function test_func1() -> r {
                            let index := 30
                            let value := 256
                            r := byte(index, value)
                        }

                        function test_func2() -> r { // ok: r is defined in different scope
                            let index := 31          // ok: index is defined in different scope
                            let value := 3           // ok: value is defined in different scope
                            r := byte(index, value)
                        }

                        let r := test_func2()
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
            .call(&solidity_selector("test_func2()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000003");
    }

    #[test]
    fn test_yul_var_redefine6() {
        let mut runtime = TestRuntime::new("VarRedefineTest6", "target/test_yul_var_redefine6");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "VarRedefineTest6" {
                code {
                }
                object "VarRedefineTest6_deployed" {
                    code {
                        function test_func() -> value1, value2 {
                            {
                                let srcVal := 10
                                value1 := srcVal
                            }
                            {
                                let srcVal := 20 // ok: srcVal is defined in different scope
                                value2 := srcVal
                            }
                        }
                        let r1, r2 := test_func()
                        mstore(0x00, r2)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime
            .call(&solidity_selector("test_func()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000014");
    }

    #[test]
    fn test_yul_var_redefine7() {
        let mut runtime = TestRuntime::new("VarRedefineTest7", "target/test_yul_var_redefine7");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "VarRedefineTest7" {
                code {
                }
                object "VarRedefineTest7_deployed" {
                    code {
                        function test_func() -> result {
                            result := 1
                            let offset := 1
                            {
                                let offset := 5  // ok: offset is defined in different scope
                                result := add(result, offset)
                            }
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
        runtime.deploy(&[]).unwrap();
        runtime
            .call(&solidity_selector("test_func()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000006");
    }

    #[test]
    fn test_yul_var_redefine8() {
        let mut runtime = TestRuntime::new("VarRedefineTest8", "target/test_yul_var_redefine8");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "VarRedefineTest8" {
                code {
                }
                object "VarRedefineTest8_deployed" {
                    code {
                        function power(base, exponent) -> result {
                            result := 1
                            for { let i := 0 } lt(i, exponent) { i := add(i, 1) }
                            {
                                result := mul(result, base)
                            }
                        }
                        let r := power(2, 2)
                        mstore(0x00, r)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("power()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000004");
    }

    #[test]
    fn test_yul_var_redefine9() {
        let mut runtime = TestRuntime::new("VarRedefineTest9", "target/test_yul_var_redefine9");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "VarRedefineTest9" {
                code {
                }
                object "VarRedefineTest9_deployed" {
                    code {
                        function test_func() -> value1, value2 {
                            if gt(1, 2) { revert(0, 0) }
                            {
                                let srcVal := 10
                                value1 := srcVal
                            }
                            {
                                let srcVal := 20 // ok: srcVal is defined in different scope
                                value2 := srcVal
                            }
                        }
                        let r1, r2 := test_func()
                        mstore(0x00, r2)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime
            .call(&solidity_selector("test_func()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000014");
    }
}
