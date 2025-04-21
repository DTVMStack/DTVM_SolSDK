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
    fn test_yul_sdiv0() {
        let mut runtime = TestRuntime::new("SdivTest0", "target/test_yul_sdiv0");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "SdivTest0" {
                    code {
                    }
                    object "SdivTest0_deployed" {
                        code {
                            // sdiv(x, y): x / y, for signed two's complement, returns 0 if y == 0
                            function test_sdiv() -> r {
                                let x := 10
                                let y := 0
                                r := sdiv(x, y)
                            }

                            let r := test_sdiv()
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
            .call(&solidity_selector("test_sdiv()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_yul_sdiv1() {
        let mut runtime = TestRuntime::new("SdivTest1", "target/test_yul_sdiv1");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "SdivTest1" {
                    code {
                    }
                    object "SdivTest1_deployed" {
                        code {
                            function test_sdiv() -> r {
                                let x := 10
                                let y := 10
                                r := sdiv(x, y)
                            }

                            let r := test_sdiv()
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
            .call(&solidity_selector("test_sdiv()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000001");
    }

    #[test]
    fn test_yul_sdiv2() {
        let mut runtime = TestRuntime::new("SdivTest2", "target/test_yul_sdiv2");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "SdivTest2" {
                    code {
                    }
                    object "SdivTest2_deployed" {
                        code {
                            function test_sdiv() -> r {
                                let x := 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFE
                                let y := 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
                                r := sdiv(x, y)
                            }

                            let r := test_sdiv()
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
            .call(&solidity_selector("test_sdiv()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000002");
    }

    #[test]
    fn test_yul_smod0() {
        let mut runtime = TestRuntime::new("SmodTest0", "target/test_yul_smod0");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "SmodTest0" {
                    code {
                    }
                    object "SmodTest0_deployed" {
                        code {
                            // smod(x, y): x % y, for signed two's complement, returns 0 if y == 0
                            function test_smod() -> r {
                                let x := 2
                                let y := 0
                                r := smod(x, y)
                            }

                            let r := test_smod()
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
            .call(&solidity_selector("test_smod()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_yul_smod1() {
        let mut runtime = TestRuntime::new("SmodTest1", "target/test_yul_smod1");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "SmodTest1" {
                    code {
                    }
                    object "SmodTest1_deployed" {
                        code {
                            // smod(x, y): x % y, for signed two's complement, returns 0 if y == 0
                            function test_smod() -> r {
                                let x := 10
                                let y := 3
                                r := smod(x, y)
                            }

                            let r := test_smod()
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
            .call(&solidity_selector("test_smod()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000001");
    }

    #[test]
    fn test_yul_smod2() {
        let mut runtime = TestRuntime::new("SmodTest2", "target/test_yul_smod2");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "SmodTest2" {
                    code {
                    }
                    object "SmodTest2_deployed" {
                        code {
                            // smod(x, y): x % y, for signed two's complement, returns 0 if y == 0
                            function test_smod() -> r {
                                let x := 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF8
                                let y := 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFD
                                r := smod(x, y)
                            }

                            let r := test_smod()
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
            .call(&solidity_selector("test_smod()"), &[])
            .unwrap();
        runtime.assert_result("fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe");
    }

    #[test]
    fn test_yul_signextend0() {
        let mut runtime = TestRuntime::new("SignextendTest0", "target/test_yul_signextend0");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "SignextendTest0" {
                    code {
                    }
                    object "SignextendTest0_deployed" {
                        code {
                            // signextend(n, value) counts from the least significant bit and performs sign extension from the (n*8+7)th bit
                            function test_signextend() -> r {
                                let n := 0
                                let value := 0xFF
                                r := signextend(n, value)
                            }

                            let r := test_signextend()
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
            .call(&solidity_selector("test_signextend()"), &[])
            .unwrap();
        runtime.assert_result("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
    }

    #[test]
    fn test_yul_signextend1() {
        let mut runtime = TestRuntime::new("SignextendTest1", "target/test_yul_signextend1");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "SignextendTest1" {
                    code {
                    }
                    object "SignextendTest1_deployed" {
                        code {
                            // signextend(n, value) counts from the least significant bit and performs sign extension from the (n*8+7)th bit
                            function test_signextend() -> r {
                                let n := 0
                                let value := 0x7f
                                r := signextend(n, value)
                            }

                            let r := test_signextend()
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
            .call(&solidity_selector("test_signextend()"), &[])
            .unwrap();
        runtime.assert_result("000000000000000000000000000000000000000000000000000000000000007f");
    }

    #[test]
    fn test_yul_signextend2() {
        let mut runtime = TestRuntime::new("SignextendTest2", "target/test_yul_signextend2");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "SignextendTest2" {
                    code {
                    }
                    object "SignextendTest2_deployed" {
                        code {
                            // signextend(n, value) counts from the least significant bit and performs sign extension from the (n*8+7)th bit
                            function test_signextend() -> r {
                                let n := 1
                                let value := 255
                                r := signextend(n, value)
                            }

                            let r := test_signextend()
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
            .call(&solidity_selector("test_signextend()"), &[])
            .unwrap();
        runtime.assert_result("00000000000000000000000000000000000000000000000000000000000000ff");
    }
}
