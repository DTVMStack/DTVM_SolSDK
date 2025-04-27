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
    fn test_create_2_and_creation_code() {
        let mut runtime = TestRuntime::new(
            "test_create_2_and_creation_code",
            "target/test_create_2_and_creation_code",
        );
        runtime.clear_testdata();
        let _emited_bc = runtime
            .compile_test_yul(
                r#"
            object "test_create_2_and_creation_code" {
                code {
                }

                object "test_create_2_and_creation_code_deployed" {

                    code {
                        function allocate_unbounded() -> memPtr {
                            memPtr := mload(64)
                        }

                        function round_up_to_mul_of_32(value) -> result {
                            result := and(add(value, 31), not(31))
                        }

                        function panic_error_0x41() {
                            mstore(0, 35408467139433450592217433187231851964531694900788300625387963629091585785856)
                            mstore(4, 0x41)
                            revert(0, 0x24)
                        }

                        function finalize_allocation(memPtr, size) {
                            let newFreePtr := add(memPtr, round_up_to_mul_of_32(size))
                            // protect against overflow
                            if or(gt(newFreePtr, 0xffffffffffffffff), lt(newFreePtr, memPtr)) { panic_error_0x41() }
                            mstore(64, newFreePtr)
                        }

                        function allocate_memory(size) -> memPtr {
                            memPtr := allocate_unbounded()
                            finalize_allocation(memPtr, size)
                        }

                        function array_dataslot_t_bytes_memory_ptr(ptr) -> data {
                            data := ptr
                            data := add(ptr, 0x20)
                        }

                        function array_length_t_bytes_memory_ptr(value) -> length {
                            length := mload(value)
                        }

                        function test_create_2_and_creation_code() -> init_code_hash, created_addr {
                            let _1 := datasize("SubContract")
                            let expr_15_mpos := allocate_memory(add(_1, 32))
                            mstore(expr_15_mpos, _1)
                            datacopy(add(expr_15_mpos, 32), dataoffset("SubContract"), _1)
                            /// "keccak256(type(SubContract).creationCode)"
                            init_code_hash := keccak256(array_dataslot_t_bytes_memory_ptr(expr_15_mpos), array_length_t_bytes_memory_ptr(expr_15_mpos))
                            created_addr := create2(0, array_dataslot_t_bytes_memory_ptr(expr_15_mpos), array_length_t_bytes_memory_ptr(expr_15_mpos), 0)
                        }

                        let init_code_hash, created_addr := test_create_2_and_creation_code()
                        mstore(0x00, init_code_hash)
                        mstore(0x20, created_addr)
                        return(0x00, 0x40)
                    }

                    object "SubContract" {
                        code {
                        }
                        object "SubContract_deployed" {
                            code {
                                function test_and() -> r {
                                    let a := 1
                                    let b := 0
                                    r := and(a, b)
                                }
                            }
                        }
                    }

                }
            }
            "#,
            )
            .unwrap();
        runtime.deploy(&[]).unwrap();
        runtime
            .call(&solidity_selector("test_create_2_and_creation_code()"), &[])
            .unwrap();
        // TODO: manualy check the using init code hash: xxx same as the first 32 bytes hex of the last evm finish with result hex:
        runtime.assert_success();
        // for AND
    }
}
