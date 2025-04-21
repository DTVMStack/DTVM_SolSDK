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
    fn test_yul_byte_0() {
        let mut runtime = TestRuntime::new("ByteTest0", "target/test_yul_byte_0");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "ByteTest0" {
                    code {
                    }
                    object "ByteTest0_deployed" {
                        code {
                            // byte(index, value): the indicated byte at the least significant position, returns 0 if the byte offset is out of range
                            function test_byte_0() -> r {
                                let index := 33
                                let value := 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFD
                                r := byte(index, value)
                            }

                            let r := test_byte_0()
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
            .call(&solidity_selector("test_byte_0()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_yul_byte_1() {
        let mut runtime = TestRuntime::new("ByteTest1", "target/test_yul_byte_1");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "ByteTest1" {
                    code {
                    }
                    object "ByteTest1_deployed" {
                        code {
                            function test_byte_1() -> r {
                                let index := 30
                                let value := 256 // 0x0000000000000000000000000000000000000000000000000000000000000100
                                r := byte(index, value)
                            }

                            let r := test_byte_1()
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
            .call(&solidity_selector("test_byte_1()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000001");
    }

    #[test]
    fn test_byte_2() {
        let mut runtime = TestRuntime::new("ByteTest2", "target/test_yul_byte_2");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "ByteTest2" {
                    code {
                    }
                    object "ByteTest2_deployed" {
                        code {
                            function test_byte_2() -> r {
                                let index := 30
                                let value := 0x00000000000000000000000000000000000000000000000000000000000012c1 // 4801
                                r := byte(index, value)
                            }

                            let r := test_byte_2()
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
            .call(&solidity_selector("test_byte_2()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000012");
    }

    #[test]
    fn test_byte_3() {
        let mut runtime = TestRuntime::new("ByteTest3", "target/test_yul_byte_3");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "ByteTest3" {
                    code {
                    }
                    object "ByteTest3_deployed" {
                        code {
                            function test_byte_3() -> r {
                                let index := 2
                                let value := "test" // 0x7465737400000000000000000000000000000000000000000000000000000000
                                r := byte(index, value)
                            }

                            let r := test_byte_3()
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
            .call(&solidity_selector("test_byte_3()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000073");
    }

    #[test]
    fn test_byte_4() {
        let mut runtime = TestRuntime::new("ByteTest4", "target/test_yul_byte_4");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "ByteTest4" {
                    code {
                    }
                    object "ByteTest4_deployed" {
                        code {
                            function test_byte_4() -> a {
                                let x := 0x80000000000000000000000000000000
                                let r := 0

                                if gt(x, 0xffffffffffffffffffffffffffffffff) {
                                    r := shl(7, 1)
                                }
                                // debug_print(r) // 0x0000000000000000000000000000000000000000000000000000000000000000

                                if gt(shr(r, x), 0xffffffffffffffff) {
                                    r := or(r, shl(6, 1))
                                }
                                // debug_print(r) // 0x0000000000000000000000000000000000000000000000000000000000000040

                                if gt(shr(r, x), 0xffffffff) {
                                    r := or(r, shl(5, 1))
                                }
                                // debug_print(r) // 0x0000000000000000000000000000000000000000000000000000000000000060

                                if gt(shr(r, x), 0xffff) {
                                    r := or(r, shl(4, 1))
                                }
                                // debug_print(r) // 0x0000000000000000000000000000000000000000000000000000000000000070

                                if gt(shr(r, x), 0xff) {
                                    r := or(r, shl(3, 1))
                                }
                                // debug_print(r) // 0x0000000000000000000000000000000000000000000000000000000000000078

                                if gt(shr(r, x), 0xf) {
                                    r := or(r, shl(2, 1))
                                }
                                // debug_print(r) // 0x000000000000000000000000000000000000000000000000000000000000007c
                                a := or(r, byte(shr(r, x), 0x0000010102020202030303030303030300000000000000000000000000000000))
                                // debug_print(a) // 0x000000000000000000000000000000000000000000000000000000000000007f
                            }

                            let r := test_byte_4()
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
            .call(&solidity_selector("test_byte_4()"), &[])
            .unwrap();
        runtime.assert_result("000000000000000000000000000000000000000000000000000000000000007f");
    }
}
