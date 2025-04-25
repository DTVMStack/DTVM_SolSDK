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
    fn test_mstore_after_arith_1() {
        let mut runtime = TestRuntime::new(
            "test_mstore_after_arith_1",
            "target/test_mstore_after_arith_1",
        );
        runtime.clear_testdata();
        let emited_bc = runtime
            .compile_test_yul(
                r#"
            object "test_mstore_after_arith_1" {
                code {
                }

                object "test_mstore_after_arith_1_deployed" {
                    code {
                        function test_mstore_after_arith() -> r {
                            let var_to := 0x1234567890abcdef
                            let _1 := sub(shl(160, 1), 1)
                            let _2 := and(var_to, _1)

                            if iszero(_2)
                            {
                                let _5 := mload(64)
                                mstore(_5, shl(224, 0xec442f05))
                                mstore(add(_5, 4), 0x00)
                                revert(_5, 36)
                            }
                            mstore(0, _2)
                            mstore(0x20, 0)
                            r := _2
                        }

                        let result := test_mstore_after_arith()
                        mstore(0x00, result)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        std::fs::write(
            "target/test_mstore_after_arith_1/test_mstore_after_arith_1.wasm",
            emited_bc,
        )
        .unwrap();
        runtime.wasm2wat(
            "target/test_mstore_after_arith_1/test_mstore_after_arith_1.wasm",
            "target/test_mstore_after_arith_1/test_mstore_after_arith_1.wat",
        );

        runtime.deploy(&[]).unwrap();
        runtime
            .call(&solidity_selector("test_mstore_after_arith()"), &[])
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000001234567890abcdef");
    }

    #[test]
    fn test_yul_mstore8() {
        let mut runtime = TestRuntime::new("Mstore8Test", "target/test_yul_mstore8");
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
                object "Mstore8Test" {
                    code {
                    }
                    object "Mstore8Test_deployed" {
                        code {
                            // mstore8(p, v): mem[p] := v & 0xff (only modifies a single byte)
                            function test_mstore8() -> r {
                                mstore8(0, 0xFFFF)
                                mstore8(1, 0xAB)
                                r := mload(0)
                            }

                            let r := test_mstore8()
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
            .call(&solidity_selector("test_mstore8()"), &[])
            .unwrap();
        runtime.assert_result("ffab000000000000000000000000000000000000000000000000000000000000");
    }
}
