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
    fn test_yul_add_overflow_u32() {
        let mut runtime = TestRuntime::new(
            "test_yul_add_overflow_u32",
            "target/test_yul_add_overflow_u32",
        );
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "test_yul_add_overflow_u32" {
                code {
                }
                object "test_yul_add_overflow_u32_deployed" {
                    code {
                        function test_add() -> r {
                            let a := 0xFFFFFFFF
                            let b := 0xFFFFFFFF
                            // when arithmetic calculate, all the numbers are converted to u256
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
        runtime.assert_result("00000000000000000000000000000000000000000000000000000001fffffffe");
    }

    #[test]
    fn test_yul_add_overflow() {
        let mut runtime = TestRuntime::new("AddOverflowTest", "target/test_yul_add_overflow");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "AddOverflowTest" {
                code {
                }
                object "AddOverflowTest_deployed" {
                    code {
                        function test_add() -> r {
                            let a := 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
                            let b := 3
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
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000002");
    }

    #[test]
    fn test_yul_add_large_numbers() {
        let mut runtime = TestRuntime::new("AddLargeTest", "target/test_yul_add_large");
        runtime.clear_testdata();
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
        object "AddLargeTest" {
            code {
            }
            object "AddLargeTest_deployed" {
                code {
                    function test_add() -> r {
                        let a := 0xffffffffffffffff
                        let b := 0x123456789abcdef
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
        runtime.assert_result("0000000000000000000000000000000000000000000000010123456789abcdee");
    }

    #[test]
    fn test_yul_add_overflow2() {
        let mut runtime =
            TestRuntime::new("test_yul_add_overflow2", "target/test_yul_add_overflow");
        runtime.clear_testdata();
        let _emited_bc = runtime
        .compile_test_yul(
            r#"
        object "test_yul_add_overflow2" {
            code {
            }
            object "test_yul_add_overflow2_deployed" {
                code {
                    function test_add() -> r {
                        let max := 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
                        let one := 1
                        r := add(max, one)  // Should wrap around to 0
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
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_yul_add_multiple() {
        let mut runtime = TestRuntime::new("AddMultipleTest", "target/test_yul_add_multiple");
        runtime.clear_testdata();
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
        object "AddMultipleTest" {
            code {
            }
            object "AddMultipleTest_deployed" {
                code {
                    function test_add() -> r {
                        let a := 0x100000000
                        let b := 0x200000000
                        let c := 0x300000000
                        let temp := add(a, b)
                        r := add(temp, c)
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
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000600000000");
    }

    #[test]
    fn test_yul_sub() {
        let mut runtime = TestRuntime::new("SubTest", "target/test_yul_sub");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "SubTest" {
                code {
                }
                object "SubTest_deployed" {
                    code {
                        function test_sub() -> r {
                            let a := 5
                            let b := 3
                            r := sub(a, b)
                        }
                        let r := test_sub()
                        mstore(0x00, r)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_sub()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000002");
    }

    #[test]
    fn test_sub_overflow1() {
        let mut runtime = TestRuntime::new("SubOverflowTest1", "target/test_yul_sub_overflow1");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "SubOverflowTest1" {
                    code {
                    }
                    object "SubOverflowTest1_deployed" {
                        code {
                            function test_sub_overflow() -> r {
                                let a := 0
                                let b := 1
                                r := sub(a, b)
                            }

                            let r := test_sub_overflow()
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
            .call(&solidity_selector("test_sub_overflow()"), &[])
            .unwrap();
        runtime.assert_result("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
    }

    #[test]
    fn test_sub_overflow2() {
        let mut runtime = TestRuntime::new("SubOverflowTest2", "target/test_yul_sub_overflow2");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "SubOverflowTest2" {
                    code {
                    }
                    object "SubOverflowTest2_deployed" {
                        code {
                            function test_sub_overflow() -> r {
                                let a := 0x123456789abcdef
                                let b := 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
                                r := sub(a, b)
                            }

                            let r := test_sub_overflow()
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
            .call(&solidity_selector("test_sub_overflow()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000123456789abcdf0");
    }

    #[test]
    fn test_yul_mul() {
        let mut runtime = TestRuntime::new("MulTest", "target/test_yul_mul");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "MulTest" {
                    code {
                    }
                    object "MulTest_deployed" {
                        code {
                            function test_mul() -> r {
                                let a := 3
                                let b := 4
                                r := mul(a, b)
                            }

                            let r := test_mul()
                            mstore(0x00, r)
                            return(0x00, 0x20)
                        }
                    }
                }
                "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_mul()"), &[]).unwrap();
        runtime.assert_result("000000000000000000000000000000000000000000000000000000000000000c");
    }

    #[test]
    fn test_yul_mul_overflow() {
        let mut runtime = TestRuntime::new("MulOverflowTest", "target/test_yul_mul_overflow");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "MulOverflowTest" {
                    code {
                    }
                    object "MulOverflowTest_deployed" {
                        code {
                            function test_mul() -> r {
                                let a := 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
                                let b := 2
                                r := mul(a, b)
                            }

                            let r := test_mul()
                            mstore(0x00, r)
                            return(0x00, 0x20)
                        }
                    }
                }
                "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_mul()"), &[]).unwrap();
        runtime.assert_result("fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe");
    }

    #[test]
    fn test_yul_div0() {
        let mut runtime = TestRuntime::new("DivTest0", "target/test_yul_div0");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "DivTest0" {
                    code {
                    }
                    object "DivTest0_deployed" {
                        code {
                            function test_div() -> r {
                                let a := 10
                                let b := 0
                                r := div(a, b)
                            }

                            let r := test_div()
                            mstore(0x00, r)
                            return(0x00, 0x20)
                        }
                    }
                }
                "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_div()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_yul_div1() {
        let mut runtime = TestRuntime::new("DivTest1", "target/test_yul_div1");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "DivTest1" {
                    code {
                    }
                    object "DivTest1_deployed" {
                        code {
                            function test_div() -> r {
                                let a := 10
                                let b := 2
                                r := div(a, b)
                            }

                            let r := test_div()
                            mstore(0x00, r)
                            return(0x00, 0x20)
                        }
                    }
                }
                "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_div()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000005");
    }

    #[test]
    fn test_yul_div2() {
        let mut runtime = TestRuntime::new("DivTest2", "target/test_yul_div2");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "DivTest2" {
                    code {
                    }
                    object "DivTest2_deployed" {
                        code {
                            function test_div() -> r {
                                let a := 1
                                let b := 2
                                r := div(a, b)
                            }

                            let r := test_div()
                            mstore(0x00, r)
                            return(0x00, 0x20)
                        }
                    }
                }
                "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_div()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_yul_shl() {
        let mut runtime = TestRuntime::new("ShlTest", "target/test_yul_shl");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "ShlTest" {
                code {
                }
                object "ShlTest_deployed" {
                    code {
                        function test_shl() -> r {
                            // shl(shift, value) shifts value left by shift bits
                            let shift := 2
                            let value := 10
                            r := shl(shift, value)
                            debug_print(r)
                        }
                        let r := test_shl()
                        debug_print(r)
                        mstore(0x00, r)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_shl()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000028");
    }

    #[test]
    fn test_yul_shl_const() {
        let mut runtime = TestRuntime::new("ShlConstTest", "target/test_yul_shl_const");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "ShlConstTest" {
                code {
                }
                object "ShlConstTest_deployed" {
                    code {
                        function test_shl_const() -> r {
                            // shl(shift, value) shifts value left by shift bits
                            r := sub(shl(160, 1), 1)
                        }
                        let r := test_shl_const()
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
            .call(&solidity_selector("test_shl_const()"), &[])
            .unwrap();
        runtime.assert_result("000000000000000000000000ffffffffffffffffffffffffffffffffffffffff");
    }

    #[test]
    fn test_yul_shl_large() {
        let mut runtime = TestRuntime::new("ShlLargeTest", "target/test_yul_shl_large");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "ShlLargeTest" {
                    code {
                    }
                    object "ShlLargeTest_deployed" {
                        code {
                            function test_shl_large() -> r {
                                // shl(shift, value) shifts value left by shift bits, returns 0 if shift exceeds 255
                                let shift := 256
                                let value := 1
                                r := shl(shift, value)
                            }

                            let r := test_shl_large()
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
            .call(&solidity_selector("test_shl_large()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_yul_shl_constant_large() {
        let mut runtime = TestRuntime::new(
            "test_yul_shl_constant_large",
            "target/test_yul_shl_constant_large",
        );
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "test_yul_shl_constant_large" {
                    code {
                    }
                    object "test_yul_shl_constant_large_deployed" {
                        code {
                            function test_shl_large() -> r {
                                // shl(shift, value) shifts value left by shift bits, returns 0 if shift exceeds 255
                                r := shl(256, 1)
                            }

                            let r := test_shl_large()
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
            .call(&solidity_selector("test_shl_large()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_yul_shl_large2() {
        let mut runtime = TestRuntime::new("ShlLargeTest2", "target/test_yul_shl_large2");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "ShlLargeTest2" {
                    code {
                    }
                    object "ShlLargeTest2_deployed" {
                        code {
                            function test_shl_large2() -> r {
                                // shl(shift, value) shifts value left by shift bits, returns 0 if shift exceeds 255
                                let shift := 257
                                let value := 1
                                r := shl(shift, value)
                            }

                            let r := test_shl_large2()
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
            .call(&solidity_selector("test_shl_large2()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_yul_shl_constant_large2() {
        let mut runtime = TestRuntime::new(
            "test_yul_shl_constant_large2",
            "target/test_yul_shl_constant_large2",
        );
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "test_yul_shl_constant_large2" {
                    code {
                    }
                    object "test_yul_shl_constant_large2_deployed" {
                        code {
                            function test_shl_large2() -> r {
                                // shl(shift, value) shifts value left by shift bits, returns 0 if shift exceeds 255
                                r := shl(257, 1)
                            }

                            let r := test_shl_large2()
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
            .call(&solidity_selector("test_shl_large2()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_yul_shr() {
        let mut runtime = TestRuntime::new("ShrTest", "target/test_yul_shr");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "ShrTest" {
                code {
                }
                object "ShrTest_deployed" {
                    code {
                        // shr(shift, value) shifts value right by shift bits
                        function test_shr() -> r {
                            let shift := 2
                            let value := 10
                            r := shr(shift, value)
                        }
                        let r := test_shr()
                        mstore(0x00, r)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_shr()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000002");
    }

    #[test]
    fn test_yul_exp0() {
        let mut runtime = TestRuntime::new("ExpTest0", "target/test_yul_exp0");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "ExpTest0" {
                    code {
                    }
                    object "ExpTest0_deployed" {
                        code {
                            function test_exp() -> r {
                                let a := 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
                                let b := 0
                                r := exp(a, b)
                            }

                            let r := test_exp()
                            mstore(0x00, r)
                            return(0x00, 0x20)
                        }
                    }
                }
                "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_exp()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000001");
    }

    #[test]
    fn test_yul_exp1() {
        let mut runtime = TestRuntime::new("ExpTest1", "target/test_yul_exp1");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "ExpTest1" {
                    code {
                    }
                    object "ExpTest1_deployed" {
                        code {
                            function test_exp() -> r {
                                let a := 10
                                let b := 2
                                r := exp(a, b)
                            }

                            let r := test_exp()
                            mstore(0x00, r)
                            return(0x00, 0x20)
                        }
                    }
                }
                "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_exp()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000064");
    }

    #[test]
    fn test_yul_exp2() {
        let mut runtime = TestRuntime::new("ExpTest2", "target/test_yul_exp2");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "ExpTest2" {
                    code {
                    }
                    object "ExpTest2_deployed" {
                        code {
                            function test_exp() -> r {
                                let a := 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
                                let b := 2
                                r := exp(a, b)
                            }

                            let r := test_exp()
                            mstore(0x00, r)
                            return(0x00, 0x20)
                        }
                    }
                }
                "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_exp()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000001");
    }

    #[test]
    fn test_yul_exp3() {
        let mut runtime = TestRuntime::new("ExpTest3", "target/test_yul_exp3");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "ExpTest3" {
                    code {
                    }
                    object "ExpTest3_deployed" {
                        code {
                            function test_exp() -> r {
                                let a := 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
                                let b := 3
                                r := exp(a, b)
                            }

                            let r := test_exp()
                            mstore(0x00, r)
                            return(0x00, 0x20)
                        }
                    }
                }
                "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_exp()"), &[]).unwrap();
        runtime.assert_result("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
    }

    #[test]
    fn test_yul_keccak256() {
        let mut runtime = TestRuntime::new("KeccakTest", "target/test_yul_keccak256");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "KeccakTest" {
                    code {
                    }
                    object "KeccakTest_deployed" {
                        code {
                            function test_keccak256() -> r {
                                // keccak256(p, n): computes the Keccak-256 hash of the data located at memory address p and spanning n bytes
                                let p := 0
                                let n := 4
                                r := keccak256(p, n)
                            }

                            let r := test_keccak256()
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
            .call(&solidity_selector("test_keccak256()"), &[])
            .unwrap();
        runtime.assert_result("e8e77626586f73b955364c7b4bbf0bb7f7685ebd40e852b164633a4acbd3244c");
    }
}
