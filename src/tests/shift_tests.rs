// Copyright (C) 2024-2025 the DTVM authors, Ltd. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

#[allow(unused)]
use super::test_helper::solidity_selector;
#[allow(unused)]
use super::test_helper::TestRuntime;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yul_shl() {
        let mut runtime = TestRuntime::new("test_yul_shl", "target/test_yul_shl");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "test_yul_shl" {
                code {
                }
                object "test_yul_shl_deployed" {
                    code {
                        function test_shl() -> r {
                            // shl(shift, value) shifts value left by shift bits
                            let shift := 2
                            let value := 10
                            r := shl(shift, value)
                        }
                        let r := test_shl()
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
    fn test_yul_shl_bytes32() {
        let mut runtime = TestRuntime::new("test_yul_shl_bytes32", "target/test_yul_shl_bytes32");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "test_yul_shl_bytes32" {
                code {
                }
                object "test_yul_shl_bytes32_deployed" {
                    code {
                        function test_shl() -> r {
                            // shl(shift, value) shifts value left by shift bits
                            let value := mload(0x40)
                            let shift := 2
                            r := shl(shift, value)
                        }
                        mstore(0x40, 10)
                        let r := test_shl()
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
    fn test_yul_sar() {
        let mut runtime = TestRuntime::new("SarTest", "target/test_yul_sar");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "SarTest" {
                    code {
                    }
                    object "SarTest_deployed" {
                        code {
                            // sar(shift, value) performs arithmetic right shift of value by shift bits
                            function test_sar() -> r {
                                let shift := 2
                                let value := 10
                                r := sar(shift, value)
                            }

                            let r := test_sar()
                            mstore(0x00, r)
                            return(0x00, 0x20)
                        }
                    }
                }
                "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_sar()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000002");
    }

    // --- New tests for large uint256 shifts ---

    const LARGE_VAL: &str = "0x1111111122222222333333334444444455555555666666667777777788888888";

    fn generate_shl_yul(shift: u32) -> String {
        format!(
            r#"
            object "ShlLargeTest_{}" {{
                code {{
                }}
                object "ShlLargeTest_{}_deployed" {{
                    code {{
                        function test_shl_large() -> r {{
                            let shift := {}
                            let value := {}
                            debug_print(value)
                            debug_print(shift)
                            r := shl(shift, value)
                        }}
                        let r := test_shl_large()
                        mstore(0x00, r)
                        return(0x00, 0x20)
                    }}
                }}
            }}
            "#,
            shift, shift, shift, LARGE_VAL
        )
    }

    fn generate_shr_yul(shift: u32) -> String {
        format!(
            r#"
            object "ShrLargeTest_{}" {{
                code {{
                }}
                object "ShrLargeTest_{}_deployed" {{
                    code {{
                        function test_shr_large() -> r {{
                            let shift := {}
                            let value := {}
                            debug_print(value)
                            debug_print(shift)
                            r := shr(shift, value)
                        }}
                        let r := test_shr_large()
                        mstore(0x00, r)
                        return(0x00, 0x20)
                    }}
                }}
            }}
            "#,
            shift, shift, shift, LARGE_VAL
        )
    }

    fn generate_sar_yul(shift: u32) -> String {
        format!(
            r#"
            object "SarLargeTest_{}" {{
                code {{
                }}
                object "SarLargeTest_{}_deployed" {{
                    code {{
                        // sar(shift, value) performs arithmetic right shift of value by shift bits, returns 0 if shift exceeds 255
                        function test_sar_large() -> r {{
                            let shift := {}
                            let value := {}
                            debug_print(value)
                            debug_print(shift)
                            r := sar(shift, value)
                        }}
                        let r := test_sar_large()
                        mstore(0x00, r)
                        return(0x00, 0x20)
                    }}
                }}
            }}
            "#,
            shift, shift, shift, LARGE_VAL
        )
    }

    #[test]
    fn test_yul_shl_large_0() {
        let shift = 0;
        let mut runtime = TestRuntime::new(
            &format!("ShlLargeTest_{}", shift),
            &format!("target/test_yul_shl_large_{}", shift),
        );
        let _emited_bc = runtime.compile_test_yul(&generate_shl_yul(shift)).unwrap();
        runtime.deploy(&[]).unwrap();
        runtime
            .call(&solidity_selector("test_shl_large()"), &[])
            .unwrap();
        runtime.assert_result("1111111122222222333333334444444455555555666666667777777788888888");
    }

    #[test]
    fn test_yul_shl_large_31() {
        let shift = 31;
        let mut runtime = TestRuntime::new(
            &format!("ShlLargeTest_{}", shift),
            &format!("target/test_yul_shl_large_{}", shift),
        );
        let _emited_bc = runtime.compile_test_yul(&generate_shl_yul(shift)).unwrap();
        runtime.deploy(&[]).unwrap();
        runtime
            .call(&solidity_selector("test_shl_large()"), &[])
            .unwrap();
        runtime.assert_result("9111111119999999a22222222aaaaaaab33333333bbbbbbbc444444400000000");
    }

    #[test]
    fn test_yul_shl_large_127() {
        let shift = 127;
        let mut runtime = TestRuntime::new(
            &format!("ShlLargeTest_{}", shift),
            &format!("target/test_yul_shl_large_{}", shift),
        );
        let _emited_bc = runtime.compile_test_yul(&generate_shl_yul(shift)).unwrap();
        runtime.deploy(&[]).unwrap();
        runtime
            .call(&solidity_selector("test_shl_large()"), &[])
            .unwrap();
        runtime.assert_result("2aaaaaaab33333333bbbbbbbc444444400000000000000000000000000000000");
    }

    #[test]
    fn test_yul_shl_large_244() {
        let shift = 244;
        let mut runtime = TestRuntime::new(
            &format!("ShlLargeTest_{}", shift),
            &format!("target/test_yul_shl_large_{}", shift),
        );
        let _emited_bc = runtime.compile_test_yul(&generate_shl_yul(shift)).unwrap();
        runtime.deploy(&[]).unwrap();
        runtime
            .call(&solidity_selector("test_shl_large()"), &[])
            .unwrap();
        runtime.assert_result("8880000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_yul_shl_large_256() {
        let shift = 256;
        let mut runtime = TestRuntime::new(
            &format!("ShlLargeTest_{}", shift),
            &format!("target/test_yul_shl_large_{}", shift),
        );
        let _emited_bc = runtime.compile_test_yul(&generate_shl_yul(shift)).unwrap();
        runtime.deploy(&[]).unwrap();
        runtime
            .call(&solidity_selector("test_shl_large()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_yul_shr_large_0() {
        let shift = 0;
        let mut runtime = TestRuntime::new(
            &format!("ShrLargeTest_{}", shift),
            &format!("target/test_yul_shr_large_{}", shift),
        );
        let _emited_bc = runtime.compile_test_yul(&generate_shr_yul(shift)).unwrap();
        runtime.deploy(&[]).unwrap();
        runtime
            .call(&solidity_selector("test_shr_large()"), &[])
            .unwrap();
        runtime.assert_result("1111111122222222333333334444444455555555666666667777777788888888");
    }

    #[test]
    fn test_yul_shr_large_31() {
        let shift = 31;
        let mut runtime = TestRuntime::new(
            &format!("ShrLargeTest_{}", shift),
            &format!("target/test_yul_shr_large_{}", shift),
        );
        let _emited_bc = runtime.compile_test_yul(&generate_shr_yul(shift)).unwrap();
        runtime.deploy(&[]).unwrap();
        runtime
            .call(&solidity_selector("test_shr_large()"), &[])
            .unwrap();
        runtime.assert_result("0000000022222222444444446666666688888888aaaaaaaacccccccceeeeeeef");
    }

    #[test]
    fn test_yul_shr_large_127() {
        let shift = 127;
        let mut runtime = TestRuntime::new(
            &format!("ShrLargeTest_{}", shift),
            &format!("target/test_yul_shr_large_{}", shift),
        );
        let _emited_bc = runtime.compile_test_yul(&generate_shr_yul(shift)).unwrap();
        runtime.deploy(&[]).unwrap();
        runtime
            .call(&solidity_selector("test_shr_large()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000022222222444444446666666688888888");
    }

    #[test]
    fn test_yul_shr_large_244() {
        let shift = 244;
        let mut runtime = TestRuntime::new(
            &format!("ShrLargeTest_{}", shift),
            &format!("target/test_yul_shr_large_{}", shift),
        );
        let _emited_bc = runtime.compile_test_yul(&generate_shr_yul(shift)).unwrap();
        runtime.deploy(&[]).unwrap();
        runtime
            .call(&solidity_selector("test_shr_large()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000111");
    }

    #[test]
    fn test_yul_shr_large_256() {
        let shift = 256;
        let mut runtime = TestRuntime::new(
            &format!("ShrLargeTest_{}", shift),
            &format!("target/test_yul_shr_large_{}", shift),
        );
        let _emited_bc = runtime.compile_test_yul(&generate_shr_yul(shift)).unwrap();
        runtime.deploy(&[]).unwrap();
        runtime
            .call(&solidity_selector("test_shr_large()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_yul_sar_large_0() {
        let shift = 0;
        let mut runtime = TestRuntime::new(
            &format!("SarLargeTest_{}", shift),
            &format!("target/test_yul_sar_large_{}", shift),
        );
        let _emited_bc = runtime.compile_test_yul(&generate_sar_yul(shift)).unwrap();
        runtime.deploy(&[]).unwrap();
        runtime
            .call(&solidity_selector("test_sar_large()"), &[])
            .unwrap();
        runtime.assert_result("1111111122222222333333334444444455555555666666667777777788888888");
    }

    #[test]
    fn test_yul_sar_large_31() {
        let shift = 31;
        let mut runtime = TestRuntime::new(
            &format!("SarLargeTest_{}", shift),
            &format!("target/test_yul_sar_large_{}", shift),
        );
        let _emited_bc = runtime.compile_test_yul(&generate_sar_yul(shift)).unwrap();
        runtime.deploy(&[]).unwrap();
        runtime
            .call(&solidity_selector("test_sar_large()"), &[])
            .unwrap();
        runtime.assert_result("0000000022222222444444446666666688888888aaaaaaaacccccccceeeeeeef");
    }

    #[test]
    fn test_yul_sar_large_127() {
        let shift = 127;
        let mut runtime = TestRuntime::new(
            &format!("SarLargeTest_{}", shift),
            &format!("target/test_yul_sar_large_{}", shift),
        );
        let _emited_bc = runtime.compile_test_yul(&generate_sar_yul(shift)).unwrap();
        runtime.deploy(&[]).unwrap();
        runtime
            .call(&solidity_selector("test_sar_large()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000022222222444444446666666688888888");
    }

    #[test]
    fn test_yul_sar_large_244() {
        let shift = 244;
        let mut runtime = TestRuntime::new(
            &format!("SarLargeTest_{}", shift),
            &format!("target/test_yul_sar_large_{}", shift),
        );
        let _emited_bc = runtime.compile_test_yul(&generate_sar_yul(shift)).unwrap();
        runtime.deploy(&[]).unwrap();
        runtime
            .call(&solidity_selector("test_sar_large()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000111");
    }

    #[test]
    fn test_yul_sar_large_256() {
        let shift = 256;
        let mut runtime = TestRuntime::new(
            &format!("SarLargeTest_{}", shift),
            &format!("target/test_yul_sar_large_{}", shift),
        );
        let _emited_bc = runtime.compile_test_yul(&generate_sar_yul(shift)).unwrap();
        runtime.deploy(&[]).unwrap();
        runtime
            .call(&solidity_selector("test_sar_large()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000000");
    }
}
