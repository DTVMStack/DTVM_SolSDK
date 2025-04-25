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
    fn test_yul_mod0() {
        let mut runtime = TestRuntime::new("ModTest0", "target/test_yul_mod0");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "ModTest0" {
                    code {
                    }
                    object "ModTest0_deployed" {
                        code {
                            function test_mod() -> r {
                                let a := 10
                                let b := 0
                                r := mod(a, b)
                            }

                            let r := test_mod()
                            mstore(0x00, r)
                            return(0x00, 0x20)
                        }
                    }
                }
                "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_mod()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_yul_mod1() {
        let mut runtime = TestRuntime::new("ModTest1", "target/test_yul_mod1");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "ModTest1" {
                    code {
                    }
                    object "ModTest1_deployed" {
                        code {
                            function test_mod() -> r {
                                let a := 10
                                let b := 3
                                r := mod(a, b)
                            }

                            let r := test_mod()
                            mstore(0x00, r)
                            return(0x00, 0x20)
                        }
                    }
                }
                "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime.call(&solidity_selector("test_mod()"), &[]).unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000001");
    }

    #[test]
    fn test_yul_addmod0() {
        let mut runtime = TestRuntime::new("AddmodTest0", "target/test_yul_addmod0");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "AddmodTest0" {
                    code {
                    }
                    object "AddmodTest0_deployed" {
                        code {
                            function test_addmod0() -> r {
                                // addmod(a, b, n) = (a + b) % n, returns 0 when n is 0
                                let a := 1
                                let b := 2
                                let n := 0
                                r := addmod(a, b, n)
                            }

                            let r := test_addmod0()
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
            .call(&solidity_selector("test_addmod0()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_yul_addmod1() {
        let mut runtime = TestRuntime::new("AddmodTest1", "target/test_yul_addmod1");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "AddmodTest1" {
                    code {
                    }
                    object "AddmodTest1_deployed" {
                        code {
                            function test_addmod1() -> r {
                                // addmod(a, b, n) = (a + b) % n, returns 0 when n is 0
                                let a := 10
                                let b := 20
                                let n := 4
                                r := addmod(a, b, n)
                            }

                            let r := test_addmod1()
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
            .call(&solidity_selector("test_addmod1()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000002");
    }

    #[test]
    fn test_yul_addmod_large1() {
        let mut runtime = TestRuntime::new("AddmodTestLarge1", "target/test_yul_addmod_large1");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "AddmodTestLarge1" {
                    code {
                    }
                    object "AddmodTestLarge1_deployed" {
                        code {
                            function test_addmod_large() -> r {
                                // addmod(a, b, n) = (a + b) % n, returns 0 when n is 0
                                let a := 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
                                let b := 2
                                let n := 2
                                r := addmod(a, b, n)
                            }

                            let r := test_addmod_large()
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
            .call(&solidity_selector("test_addmod_large()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000001");
    }

    #[test]
    fn test_yul_addmod_large2() {
        let mut runtime = TestRuntime::new("AddmodTestLarge2", "target/test_yul_addmod_large2");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "AddmodTestLarge2" {
                    code {
                    }
                    object "AddmodTestLarge2_deployed" {
                        code {
                            function test_addmod_large() -> r {
                                // addmod(a, b, n) = (a + b) % n, returns 0 when n is 0
                                let a := 0x60fed4ba255a9d31c961eb74c6356d68c049b8923b61fa6ce669622e60f29fb6 // 43872280807156713839160376167191808430140484563252114113014272064716834774966
                                let b := 0x60fed4ba255a9d31c961eb74c6356d68c049b8923b61fa6ce669622e60f29fb6 // 43872280807156713839160376167191808430140484563252114113014272064716834774966
                                let n := 115792089210356248762697446949407573530086143415290314195533631308867097853951 // 0xffffffff00000001000000000000000000000000ffffffffffffffffffffffff
                                r := addmod(a, b, n) // 87744561614313427678320752334383616860280969126504228226028544129433669549932 0xc1fda9744ab53a6392c3d6e98c6adad18093712476c3f4d9ccd2c45cc1e53f6c
                            }

                            let r := test_addmod_large()
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
            .call(&solidity_selector("test_addmod_large()"), &[])
            .unwrap();
        runtime.assert_result("c1fda9744ab53a6392c3d6e98c6adad18093712476c3f4d9ccd2c45cc1e53f6c");
    }

    #[test]
    fn test_yul_mulmod0() {
        let mut runtime = TestRuntime::new("MulmodTest0", "target/test_yul_mulmod0");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "MulmodTest0" {
                    code {
                    }
                    object "MulmodTest0_deployed" {
                        code {
                            function test_mulmod0() -> r {
                                // mulmod(a, b, n) = (a * b) % n, returns 0 when n is 0
                                let a := 1
                                let b := 2
                                let n := 0
                                r := mulmod(a, b, n)
                            }

                            let r := test_mulmod0()
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
            .call(&solidity_selector("test_mulmod0()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_yul_mulmod1() {
        let mut runtime = TestRuntime::new("MulmodTest1", "target/test_yul_mulmod1");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "MulmodTest1" {
                    code {
                    }
                    object "MulmodTest1_deployed" {
                        code {
                            function test_mulmod1() -> r {
                                // mulmod(a, b, n) = (a * b) % n, returns 0 when n is 0
                                let a := 2
                                let b := 9
                                let n := 4
                                r := mulmod(a, b, n)
                            }

                            let r := test_mulmod1()
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
            .call(&solidity_selector("test_mulmod1()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000002");
    }

    #[test]
    fn test_yul_mulmod_large1() {
        let mut runtime = TestRuntime::new("MulmodTestLarge1", "target/test_yul_mulmod_large1");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "MulmodTestLarge1" {
                    code {
                    }
                    object "MulmodTestLarge1_deployed" {
                        code {
                            function test_mulmod_large() -> r {
                                // mulmod(a, b, n) = (a * b) % n, returns 0 when n is 0
                                let a := 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
                                let b := 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
                                let n := 12
                                r := mulmod(a, b, n)
                            }

                            let r := test_mulmod_large()
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
            .call(&solidity_selector("test_mulmod_large()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000009");
    }

    #[test]
    fn test_yul_mulmod_large2() {
        let mut runtime = TestRuntime::new("MulmodTestLarge2", "target/test_yul_mulmod_large2");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "MulmodTestLarge2" {
                    code {
                    }
                    object "MulmodTestLarge2_deployed" {
                        code {
                            function test_mulmod_large() -> r {
                                // mulmod(a, b, n) = (a * b) % n, returns 0 when n is 0
                                let a := 0x60fed4ba255a9d31c961eb74c6356d68c049b8923b61fa6ce669622e60f29fb6 // 43872280807156713839160376167191808430140484563252114113014272064716834774966
                                let b := 0x60fed4ba255a9d31c961eb74c6356d68c049b8923b61fa6ce669622e60f29fb6 // 43872280807156713839160376167191808430140484563252114113014272064716834774966
                                let n := 115792089210356248762697446949407573530086143415290314195533631308867097853951 // 0xffffffff00000001000000000000000000000000ffffffffffffffffffffffff
                                r := mulmod(a, b, n) // 73229330830612387938861257136077804672149607818474777466957281554347071022933, 0xa1e6551b46bc92098f9b4ec9b88377e83109548a6b01b3105552ed819336a755
                            }

                            let r := test_mulmod_large()
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
            .call(&solidity_selector("test_mulmod_large()"), &[])
            .unwrap();
        runtime.assert_result("a1e6551b46bc92098f9b4ec9b88377e83109548a6b01b3105552ed819336a755");
    }

    #[test]
    fn test_yul_mix_addmod_mulmod() {
        let mut runtime = TestRuntime::new("MixAddmodMulmod", "target/test_yul_mix_addmod_mulmod");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "MixAddmodMulmod" {
                    code {
                    }
                    object "MixAddmodMulmod_deployed" {
                        code {
                            function test_mix_addmod_mulmod() -> r {
                                let usr$p := 115792089210356248762697446949407573530086143415290314195533631308867097853951
                                // let usr$p := 0xffffffff00000001000000000000000000000000ffffffffffffffffffffffff
                                let var_y_2186 := 0x7903fe1008b8bc99a41ae9e95628bc64f2f1b20c2d7e9f5177a3c294d4462299
                                let usr$lhs := mulmod(var_y_2186, var_y_2186, usr$p)
                                debug_print(usr$lhs)
                                let var_x_2184 := 0x60fed4ba255a9d31c961eb74c6356d68c049b8923b61fa6ce669622e60f29fb6
                                let usr$rhs := addmod(mulmod(addmod(mulmod(var_x_2184, var_x_2184, usr$p), 115792089210356248762697446949407573530086143415290314195533631308867097853948, usr$p), var_x_2184, usr$p), 41058363725152142129326129780047268409114441015993725554835256314039467401291, usr$p)
                                r := and(and(lt(var_x_2184, usr$p), lt(var_y_2186, usr$p)), eq(usr$lhs, usr$rhs))
                            }

                            let r := test_mix_addmod_mulmod()
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
            .call(&solidity_selector("test_mix_addmod_mulmod()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000001");
    }
}
