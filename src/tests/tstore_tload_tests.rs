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
    fn test_tstore_tload() {
        let mut runtime = TestRuntime::new("test_tstore_tload", "target/test_tstore_tload");
        runtime.clear_testdata();
        let emited_bc = runtime
            .compile_test_yul(
                r#"
            object "test_tstore_tload" {
                code {
                }

                object "test_tstore_tload_deployed" {
                    code {
                        function test_tstore_tload() -> result {
                            // Use tstore for temporary storage
                            let temp_key := 0x02  // Key for temporary storage
                            let temp_value := 0xdeadbeef  // Value to store temporarily

                            tstore(temp_key, temp_value)  // Store temp_value using temp_key
                            let loaded_value := tload(temp_key)  // Load value using temp_key

                            // Revert if loaded value doesn't match stored value
                            if iszero(eq(loaded_value, temp_value)) {
                                let _ptr := mload(64)
                                mstore(_ptr, shl(224, 0xec442f05))  // Write revert error identifier (e.g. function selector)
                                mstore(add(_ptr, 4), 0x00)          // Error data
                                revert(_ptr, 36)
                            }

                            // Return loaded value if values match
                            result := loaded_value
                        }

                        let test_result := test_tstore_tload()
                        mstore(0x00, test_result)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        std::fs::write("target/test_tstore_tload/test_tstore_tload.wasm", emited_bc).unwrap();
        runtime.wasm2wat(
            "target/test_tstore_tload/test_tstore_tload.wasm",
            "target/test_tstore_tload/test_tstore_tload.wat",
        );

        runtime.deploy(&[]).unwrap();
        runtime
            .call(&solidity_selector("test_tstore_tload()"), &[])
            .unwrap();
        runtime.assert_result("00000000000000000000000000000000000000000000000000000000deadbeef");
    }
}
