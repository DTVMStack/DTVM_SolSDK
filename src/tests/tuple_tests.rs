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
    fn test_return_and_assign_to_multiple_non_ret_vars() {
        let mut runtime = TestRuntime::new(
            "test_return_and_assign_to_multiple_non_ret_vars",
            "target/test_return_and_assign_to_multiple_non_ret_vars",
        );
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "test_return_and_assign_to_multiple_non_ret_vars" {
                code {
                }
                object "test_return_and_assign_to_multiple_non_ret_vars_deployed" {
                    code {

                        function test_f1() -> r1, r2 {
                            let a := 3
                            let b := 2
                            r1 := add(a, b)
                            debug_print(r1)

                            r2 := sub(a, b)
                            debug_print(r2)
                        }

                        function test_add() -> r1, r2 {
                            let a := 1
                            let b := 2
                            a, b := test_f1()
                            r1 := a
                            r2 := b
                            debug_print(r1)
                            debug_print(r2)
                        }

                        let r1, r2 := test_add()
                        debug_print(r1)
                        debug_print(r2)
                        mstore(0x00, r1)
                        mstore(0x20, r2)
                        return(0x00, 0x40)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_add()"), &[]).unwrap();
        runtime.assert_result("00000000000000000000000000000000000000000000000000000000000000050000000000000000000000000000000000000000000000000000000000000001");
    }

    #[test]
    fn test_return_and_assign_to_multiple_ret_vars() {
        let mut runtime = TestRuntime::new(
            "test_return_and_assign_to_multiple_ret_vars",
            "target/test_return_and_assign_to_multiple_ret_vars",
        );
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "test_return_and_assign_to_multiple_ret_vars" {
                code {
                }
                object "test_return_and_assign_to_multiple_ret_vars_deployed" {
                    code {

                        function test_f1() -> r1, r2 {
                            let a := 3
                            let b := 2
                            r1 := add(a, b)
                            debug_print(r1)

                            r2 := sub(a, b)
                            debug_print(r2)
                        }

                        function test_add() -> r1, r2 {
                            r1, r2 := test_f1()
                            debug_print(r1)
                            debug_print(r2)
                        }

                        let r1, r2 := test_add()
                        debug_print(r1)
                        debug_print(r2)
                        mstore(0x00, r1)
                        mstore(0x20, r2)
                        return(0x00, 0x40)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_add()"), &[]).unwrap();
        runtime.assert_result("00000000000000000000000000000000000000000000000000000000000000050000000000000000000000000000000000000000000000000000000000000001");
    }

    #[test]
    fn test_return_and_assign_to_ret_var() {
        let mut runtime = TestRuntime::new(
            "test_return_and_assign_to_ret_var",
            "target/test_return_and_assign_to_ret_var",
        );
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "test_return_and_assign_to_ret_var" {
                code {
                }
                object "test_return_and_assign_to_ret_var_deployed" {
                    code {

                        function test_f1() -> r {
                            let a := 3
                            let b := 2
                            r := add(a, b)
                            debug_print(r)
                        }

                        function test_add() -> r1 {
                            r1 := test_f1()
                            debug_print(r1)
                        }

                        let r1 := test_add()
                        debug_print(r1)
                        mstore(0x00, r1)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_add()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000005");
    }

    #[test]
    fn test_tuple_return1() {
        let mut runtime = TestRuntime::new("test_tuple_return1", "target/test_tuple_return1");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "test_tuple_return1" {
                code {
                }
                object "test_tuple_return1_deployed" {
                    code {
                        function test_add() -> r1, r2 {
                            let a := 3
                            let b := 2
                            r1 := add(a, b)
                            debug_print(r1)

                            r2 := sub(a, b)
                            debug_print(r2)
                        }

                        let r1, r2 := test_add()
                        debug_print(r1)
                        debug_print(r2)
                        mstore(0x00, r1)
                        mstore(0x20, r2)
                        return(0x00, 0x40)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_add()"), &[]).unwrap();
        runtime.assert_result("00000000000000000000000000000000000000000000000000000000000000050000000000000000000000000000000000000000000000000000000000000001");
    }
}
