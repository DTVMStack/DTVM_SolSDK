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
    fn test_yul_lt_true() {
        let mut runtime = TestRuntime::new("LtTestTrue", "target/test_yul_lt_true");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "LtTestTrue" {
                code {
                }
                object "LtTestTrue_deployed" {
                    code {
                        function test_lt() -> r {
                            let a := 3
                            let b := 5
                            r := lt(a, b)
                        }

                        let r := test_lt()
                        mstore(0x00, r)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_lt()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000001");
    }

    #[test]
    fn test_yul_lt_false1() {
        let mut runtime = TestRuntime::new("LtTestFalse1", "target/test_yul_lt_false1");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "LtTestFalse1" {
                code {
                }
                object "LtTestFalse1_deployed" {
                    code {
                        function test_lt() -> r {
                            let a := 5
                            let b := 3
                            r := lt(a, b)
                        }

                        let r := test_lt()
                        mstore(0x00, r)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_lt()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_yul_lt_false2() {
        let mut runtime = TestRuntime::new("LtTestFalse2", "target/test_yul_lt_false2");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "LtTestFalse2" {
                code {
                }
                object "LtTestFalse2_deployed" {
                    code {
                        function test_lt() -> r {
                            let a := 5
                            let b := 5
                            r := lt(a, b)
                        }

                        let r := test_lt()
                        mstore(0x00, r)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_lt()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_yul_gt_true() {
        let mut runtime = TestRuntime::new("GtTestTrue", "target/test_yul_gt_true");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "GtTestTrue" {
                code {
                }
                object "GtTestTrue_deployed" {
                    code {
                        function test_gt() -> r {
                            let a := 6
                            let b := 5
                            r := gt(a, b)
                        }

                        let r := test_gt()
                        mstore(0x00, r)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_gt()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000001");
    }

    #[test]
    fn test_yul_gt_false1() {
        let mut runtime = TestRuntime::new("GtTestFalse1", "target/test_yul_gt_false1");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "GtTestFalse1" {
                code {
                }
                object "GtTestFalse1_deployed" {
                    code {
                        function test_gt() -> r {
                            let a := 5
                            let b := 6
                            r := gt(a, b)
                        }

                        let r := test_gt()
                        mstore(0x00, r)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_gt()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_yul_gt_false2() {
        let mut runtime = TestRuntime::new("GtTestFalse2", "target/test_yul_gt_false2");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "GtTestFalse2" {
                code {
                }
                object "GtTestFalse2_deployed" {
                    code {
                        function test_gt() -> r {
                            let a := 5
                            let b := 5
                            r := gt(a, b)
                        }

                        let r := test_gt()
                        mstore(0x00, r)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_gt()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_yul_eq_true() {
        let mut runtime = TestRuntime::new("EqTestTrue", "target/test_yul_eq_true");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "EqTestTrue" {
                code {
                }
                object "EqTestTrue_deployed" {
                    code {
                        function test_eq() -> r {
                            let a := 5
                            let b := 5
                            r := eq(a, b)
                        }

                        let r := test_eq()
                        mstore(0x00, r)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_eq()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000001");
    }

    #[test]
    fn test_yul_eq_false1() {
        let mut runtime = TestRuntime::new("EqTestFalse1", "target/test_yul_eq_false1");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "EqTestFalse1" {
                code {
                }
                object "EqTestFalse1_deployed" {
                    code {
                        function test_eq() -> r {
                            let a := 5
                            let b := 6
                            r := eq(a, b)
                        }

                        let r := test_eq()
                        mstore(0x00, r)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_eq()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_yul_eq_false2() {
        let mut runtime = TestRuntime::new("EqTestFalse2", "target/test_yul_eq_false2");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "EqTestFalse2" {
                code {
                }
                object "EqTestFalse2_deployed" {
                    code {
                        function test_eq() -> r {
                            let a := 6
                            let b := 5
                            r := eq(a, b)
                        }

                        let r := test_eq()
                        mstore(0x00, r)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_eq()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_yul_slt_true() {
        let mut runtime = TestRuntime::new("SltTestTrue", "target/test_yul_slt_true");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "SltTestTrue" {
                code {
                }
                object "SltTestTrue_deployed" {
                    code {
                        // If signed integer x < y, then it is 1; otherwise, it is 0.
                        function test_slt() -> r {
                            let x := 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
                            let y := 0
                            r := slt(x, y)
                        }

                        let r := test_slt()
                        mstore(0x00, r)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_slt()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000001");
    }

    #[test]
    fn test_yul_slt_false1() {
        let mut runtime = TestRuntime::new("SltTestFalse1", "target/test_yul_slt_false1");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "SltTestFalse1" {
                code {
                }
                object "SltTestFalse1_deployed" {
                    code {
                        // If signed integer x < y, then it is 1; otherwise, it is 0.
                        function test_slt() -> r {
                            let x := 0x0f
                            let y := 1
                            r := slt(x, y)
                        }

                        let r := test_slt()
                        mstore(0x00, r)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_slt()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_yul_slt_false2() {
        let mut runtime = TestRuntime::new("SltTestFalse2", "target/test_yul_slt_false2");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "SltTestFalse2" {
                code {
                }
                object "SltTestFalse2_deployed" {
                    code {
                        // If signed integer x < y, then it is 1; otherwise, it is 0.
                        function test_slt() -> r {
                            let x := 10
                            let y := 10
                            r := slt(x, y)
                        }

                        let r := test_slt()
                        mstore(0x00, r)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_slt()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_yul_slt_false3() {
        let mut runtime = TestRuntime::new("SltTestFalse3", "target/test_yul_slt_false3");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "SltTestFalse3" {
                code {
                }
                object "SltTestFalse3_deployed" {
                    code {
                        // If signed integer x < y, then it is 1; otherwise, it is 0.
                        function test_slt() -> r {
                            let x := 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
                            let y := 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
                            r := slt(x, y)
                        }

                        let r := test_slt()
                        mstore(0x00, r)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_slt()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_yul_sgt_true() {
        let mut runtime = TestRuntime::new("SgtTestTrue", "target/test_yul_sgt_true");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "SgtTestTrue" {
                code {
                }
                object "SgtTestTrue_deployed" {
                    code {
                        // If signed integer x > y, then it is 1; otherwise, it is 0.
                        function test_sgt() -> r {
                            let x := 0
                            let y := 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
                            r := sgt(x, y)
                        }

                        let r := test_sgt()
                        mstore(0x00, r)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_sgt()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000001");
    }

    #[test]
    fn test_yul_sgt_false1() {
        let mut runtime = TestRuntime::new("SgtTestFalse1", "target/test_yul_sgt_false1");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "SgtTestFalse1" {
                code {
                }
                object "SgtTestFalse1_deployed" {
                    code {
                        // If signed integer x > y, then it is 1; otherwise, it is 0.
                        function test_sgt() -> r {
                            let x := 1
                            let y := 0x0f
                            r := sgt(x, y)
                        }

                        let r := test_sgt()
                        mstore(0x00, r)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_sgt()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_yul_sgt_false2() {
        let mut runtime = TestRuntime::new("SgtTestFalse2", "target/test_yul_sgt_false2");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "SgtTestFalse2" {
                code {
                }
                object "SgtTestFalse2_deployed" {
                    code {
                        // If signed integer x > y, then it is 1; otherwise, it is 0.
                        function test_sgt() -> r {
                            let x := 10
                            let y := 10
                            r := sgt(x, y)
                        }

                        let r := test_sgt()
                        mstore(0x00, r)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_sgt()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_yul_sgt_false3() {
        let mut runtime = TestRuntime::new("SgtTestFalse3", "target/test_yul_sgt_false3");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "SgtTestFalse3" {
                code {
                }
                object "SgtTestFalse3_deployed" {
                    code {
                        // If signed integer x > y, then it is 1; otherwise, it is 0.
                        function test_sgt() -> r {
                            let x := 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
                            let y := 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
                            r := sgt(x, y)
                        }

                        let r := test_sgt()
                        mstore(0x00, r)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_sgt()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_yul_and1() {
        let mut runtime = TestRuntime::new("AndTest1", "target/test_yul_and1");
        runtime.clear_testdata();
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "AndTest1" {
                code {
                }

                object "AndTest1_deployed" {
                    code {
                        function test_and() -> r {
                            let a := 1
                            let b := 0
                            r := and(a, b)
                        }

                        let result := test_and()
                        mstore(0x00, result)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_and()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000000");
        // for AND
    }

    #[test]
    fn test_yul_and2() {
        let mut runtime = TestRuntime::new("AndTest2", "target/test_yul_and2");
        runtime.clear_testdata();
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "AndTest2" {
                code {
                }

                object "AndTest2_deployed" {
                    code {
                        function test_and() -> r {
                            let a := 1
                            let b := 1
                            r := and(a, b)
                        }

                        let result := test_and()
                        mstore(0x00, result)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_and()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000001");
        // for AND
    }

    #[test]
    fn test_yul_or1() {
        let mut runtime = TestRuntime::new("OrTest1", "target/test_yul_or1");
        runtime.clear_testdata();
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "OrTest1" {
                code {
                }

                object "OrTest1_deployed" {
                    code {
                        function test_or() -> r {
                            let a := 0
                            let b := 1
                            r := or(a, b)
                        }

                        let result := test_or()
                        mstore(0x00, result)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_or()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000001");
        // for OR
    }

    #[test]
    fn test_yul_or2() {
        let mut runtime = TestRuntime::new("OrTest2", "target/test_yul_or2");
        runtime.clear_testdata();
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "OrTest2" {
                code {
                }

                object "OrTest2_deployed" {
                    code {
                        function test_or() -> r {
                            let a := 0
                            let b := 0
                            r := or(a, b)
                        }

                        let result := test_or()
                        mstore(0x00, result)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_or()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000000");
        // for OR
    }

    #[test]
    fn test_yul_not1() {
        let mut runtime = TestRuntime::new("test_yul_not1", "target/test_yul_not1");
        runtime.clear_testdata();
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "test_yul_not1" {
                code {
                }
                object "test_yul_not1_deployed" {
                    code {
                        function test_not() -> r {
                            let a := 0
                            // NOT 0 = 1
                            r := not(a)
                        }

                        let r_not := test_not()
                        mstore(0x00, r_not)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_not()"), &[]).unwrap();
        runtime.assert_result("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
        // for NOT
    }

    #[test]
    fn test_yul_not2() {
        let mut runtime = TestRuntime::new("test_yul_not2", "target/test_yul_not2");
        runtime.clear_testdata();
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "test_yul_not2" {
                code {
                }
                object "test_yul_not2_deployed" {
                    code {
                        function test_not() -> r {
                            let a := 1
                            // NOT 1 = 0
                            r := not(a)
                        }

                        let r_not := test_not()
                        mstore(0x00, r_not)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_not()"), &[]).unwrap();
        runtime.assert_result("fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe");
        // for NOT
    }

    #[test]
    fn test_yul_xor1() {
        let mut runtime = TestRuntime::new("XorTest", "target/test_yul_xor1");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "XorTest" {
                code {
                }
                object "XorTest_deployed" {
                    code {
                        function test_xor() -> r {
                            let a := 0
                            let b := 0
                            r := xor(a, b)
                        }
                        let r := test_xor()
                        mstore(0x00, r)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_xor()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_yul_xor2() {
        let mut runtime = TestRuntime::new("test_yul_xor2", "target/test_yul_xor2");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "test_yul_xor2" {
                code {
                }
                object "test_yul_xor2_deployed" {
                    code {
                        function test_xor() -> r {
                            let a := 0
                            let b := 1
                            r := xor(a, b)
                        }
                        let r := test_xor()
                        mstore(0x00, r)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_xor()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000001");
    }

    #[test]
    fn test_yul_xor3() {
        let mut runtime = TestRuntime::new("test_yul_xor3", "target/test_yul_xor3");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "test_yul_xor3" {
                code {
                }
                object "test_yul_xor3_deployed" {
                    code {
                        function test_xor() -> r {
                            let a := 1
                            let b := 0
                            r := xor(a, b)
                        }
                        let r := test_xor()
                        mstore(0x00, r)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_xor()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000001");
    }

    #[test]
    fn test_yul_xor4() {
        let mut runtime = TestRuntime::new("test_yul_xor4", "target/test_yul_xor4");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "test_yul_xor4" {
                code {
                }
                object "test_yul_xor4_deployed" {
                    code {
                        function test_xor() -> r {
                            let a := 1
                            let b := 1
                            r := xor(a, b)
                        }
                        let r := test_xor()
                        mstore(0x00, r)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_xor()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_yul_is_zero1() {
        let mut runtime = TestRuntime::new("test_yul_is_zero1", "target/test_yul_is_zero1");
        runtime.clear_testdata();
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "test_yul_is_zero1" {
                code {
                }
                object "test_yul_is_zero1_deployed" {
                    code {
                        function test_is_zero() -> r {
                            let a := 0
                            // iszero(0) = 1
                            r := iszero(a)
                        }

                        let r_is_zero := test_is_zero()
                        mstore(0x00, r_is_zero)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime
            .call(&solidity_selector("test_is_zero()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000001");
        // for is_zero
    }

    #[test]
    fn test_yul_is_zero2() {
        let mut runtime = TestRuntime::new("test_yul_is_zero2", "target/test_yul_is_zero2");
        runtime.clear_testdata();
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "test_yul_is_zero2" {
                code {
                }
                object "test_yul_is_zero2_deployed" {
                    code {
                        function test_is_zero() -> r {
                            let a := 7
                            // iszero(0) = 0
                            r := iszero(a)
                        }

                        let r_is_zero := test_is_zero()
                        mstore(0x00, r_is_zero)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime
            .call(&solidity_selector("test_is_zero()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000000");
        // for is_zero
    }
}
