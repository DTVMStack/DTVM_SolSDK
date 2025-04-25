// Copyright (C) 2024-2025 the DTVM authors, Ltd. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

#[allow(unused)]
use super::test_helper::solidity_selector;
#[allow(unused)]
use super::test_helper::TestRuntime;
#[cfg(test)]
mod tests {
    use ethabi::ParamType;

    use crate::tests::test_helper::encode_abi_parameters;

    use super::*;

    #[test]
    fn test_transfer_load_address_1() {
        let mut runtime = TestRuntime::new(
            "test_transfer_load_address_1",
            "target/test_transfer_load_address_1",
        );
        runtime.clear_testdata();
        let emited_bc = runtime
            .compile_test_yul(
                r#"
            object "test_transfer_load_address_1" {
                code {
                }

                object "test_transfer_load_address_1_deployed" {
                    code {
                        function fun_transfer(a, b, value) -> r {
                           sstore(a, value)
                           sstore(b, value)
                           r := 1
                        }

                        function abi_decode_address1() -> value
                        {
                            value := calldataload(4)
                            if iszero(eq(value, and(value, sub(shl(160, 1), 1)))) { revert(0, 0) }
                        }

                        let a := caller()
                        let b := abi_decode_address1()
                        let value := calldataload(36)

                        let result := fun_transfer(a, b, value)
                        mstore(0x00, result)
                        return(0x00, 0x20)
                    }
                }
            }
            "#,
            )
            .unwrap();
        std::fs::write(
            "target/test_transfer_load_address_1/test_transfer_load_address_1.wasm",
            emited_bc,
        )
        .unwrap();
        runtime.wasm2wat(
            "target/test_transfer_load_address_1/test_transfer_load_address_1.wasm",
            "target/test_transfer_load_address_1/test_transfer_load_address_1.wat",
        );

        runtime.deploy(&[]).unwrap();
        runtime
            .call(
                &solidity_selector("transfer(address,uint256)"),
                &encode_abi_parameters(&[
                    (
                        "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
                        ParamType::Address,
                    ),
                    ("100".to_string(), ParamType::Uint(256)),
                ]),
            )
            .unwrap();
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000001");
    }

    #[test]
    fn test_token_iropt_fun_transfer() {
        let mut runtime = TestRuntime::new(
            "test_token_iropt_fun_transfer",
            "target/test_token_iropt_fun_transfer",
        );
        runtime.clear_testdata();
        runtime.set_enable_all_optimizers(true);
        let emited_bc = runtime
            .compile_test_yul(
                r#"
                object "test_token_iropt_fun_transfer_Token" {
                    code {
                        // add init balance for 0x1111111111111111111111111111111111111111
                        mstore(0, 0x1111111111111111111111111111111111111111)
                        mstore(32, 0)
                        sstore(keccak256(0, 0x40), 1000000000000000000000000000000000000000000000000000000000000000)
                    }

                    object "test_token_iropt_fun_transfer_Token_deployed" {
                        code {
                            function fun_transfer(var_from, var_to, var_value)
                            {
                                let _1 := sub(shl(160, 1), 1)
                                let _2 := and(var_from, _1)
                                if iszero(_2)
                                {
                                    let _3 := mload(64)
                                    mstore(_3, shl(225, 0x4b637e8f))
                                    mstore(add(_3, 4), 0x00)
                                    revert(_3, 36)
                                }
                                let _4 := and(var_to, _1)
                                if iszero(_4)
                                {
                                    let _5 := mload(64)
                                    mstore(_5, shl(224, 0xec442f05))
                                    mstore(add(_5, 4), 0x00)
                                    revert(_5, 36)
                                }
                                mstore(0, _2)
                                mstore(0x20, 0)
                                let _6 := sload(keccak256(0, 0x40))
                                if lt(_6, var_value)
                                {
                                    revert(0, 0)
                                }
                                mstore(0, _2)
                                mstore(0x20, 0)
                                sstore(keccak256(0, 0x40), sub(_6, var_value))
                                mstore(0, _4)
                                let dataSlot := keccak256(0, 0x40)
                                sstore(dataSlot, add(sload(dataSlot), var_value))
                                let _8 := mload(0x40)
                                mstore(_8, var_value)
                                log3(_8, 0x20, 0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef, _2, _4)
                            }

                            function abi_decode_address(offset) -> value {
                                value := calldataload(offset)
                            }

                            function abi_decode_uint256(offset) -> value {
                                value := calldataload(offset)
                            }

                            // Main entry point logic: decode params and call fun_transfer
                            let param_from := abi_decode_address(4)
                            let param_to := abi_decode_address(36)
                            let param_value := abi_decode_uint256(68)

                            fun_transfer(param_from, param_to, param_value)
                            let result := 1
                            mstore(0x00, result)
                            return(0x00, 0x20)
                        }
                    }
                }
            "#,
            )
            .unwrap();
        std::fs::write(
            "target/test_token_iropt_fun_transfer/test_token_iropt_fun_transfer.wasm",
            emited_bc,
        )
        .unwrap();
        runtime.wasm2wat(
            "target/test_token_iropt_fun_transfer/test_token_iropt_fun_transfer.wasm",
            "target/test_token_iropt_fun_transfer/test_token_iropt_fun_transfer.wat",
        );

        runtime.deploy(&[]).unwrap();

        // Note: This test assumes the initial balance of the 'from' address is sufficient.
        // A real test might need to set the initial balance using sstore in the deploy phase or via runtime setup.
        runtime
            .call(
                &solidity_selector("transfer(address,address,uint256)"), // Dummy selector, not used by Yul entry logic
                &encode_abi_parameters(&[
                    (
                        "0x1111111111111111111111111111111111111111".to_string(), // from
                        ParamType::Address,
                    ),
                    (
                        "0x2222222222222222222222222222222222222222".to_string(), // to
                        ParamType::Address,
                    ),
                    ("500".to_string(), ParamType::Uint(256)), // value
                ]),
            )
            .unwrap();
        // Expect success (1)
        runtime.assert_result("0000000000000000000000000000000000000000000000000000000000000001");
    }
}
