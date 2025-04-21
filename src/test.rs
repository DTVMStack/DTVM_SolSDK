// Copyright (C) 2024-2025 Ant Group Co., Ltd. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

#[allow(unused)]
use crate::yul2ir::config::Yul2IROptions;
#[allow(unused)]
use crate::yul2ir::context::Yul2IRContext;
#[allow(unused)]
use crate::yul2ir::yul;
#[allow(unused)]
use inkwell::context::Context;

#[test]
fn test_simple_mstore_memory_guard() {
    let expr = yul::ObjectParser::new()
        .parse(
            r#"
        object "SimpleMstoreMemoryGuard" {
            code {
                let _1 := memoryguard(0x80)
                let _2 := 64
                mstore(_2, _1)
                return(_2, 0x20)
            }
        }
        "#,
        )
        .unwrap();
    println!("{:?}", expr);
    let llvm_context = Context::create();
    let opts = Yul2IROptions::test("SimpleMstoreMemoryGuard");
    let mut context = Yul2IRContext::new_with_object(&llvm_context, &opts, expr);
    let emited_bc = context.emit("SimpleMstoreMemoryGuard").unwrap();
    std::fs::write("test.out.wasm", emited_bc).unwrap();
}

#[test]
fn yul_parser_pop_instruction_test() {
    let expr = yul::ObjectParser::new()
        .parse(
            r#"
        object "PopTest" {
            /* multi line comment
               before code */
            // single line comment before code
            code {
                function test_pop() {
                    let a := 1   // comment after instruction
                    let b := 2
                    // sigle line comment between instructions
                    pop(a)
                    /* multi line comments
                       between instructions*/
                    pop(b)
                }
                // comment inner code
                test_pop()
                // comment in last code
            }

            // comment before data segment
            data ".metadata" hex"aa"
            // single line comment after data segment
            /* multi line comment
               after data segment */
        }
        "#,
        )
        .unwrap();
    println!("{:?}", expr);
    let llvm_context = Context::create();
    let opts = Yul2IROptions::test("PopTest");
    let mut context = Yul2IRContext::new_with_object(&llvm_context, &opts, expr);
    let emited_bc = context.emit("yul_parser_pop_instruction_test").unwrap();
    std::fs::write("test.out.wasm", emited_bc).unwrap();
}

#[test]
fn yul_parser_tuple_test() {
    let expr = yul::ObjectParser::new().parse(
        r#"
        object "yul_parser_tuple_test_deployed" {
            code {
                function selector_call() {
                    let a, b := selector_ret()
                }

                function selector_ret() -> s, z {
                    s := div(calldataload(0), 0x100000000000000000000000000000000000000000000000000000000)
                    z := div(calldataload(0), 0x100000000000000000000000000000000000000000000000000000000)
                }

                selector_call()
            }
        }
        "#,
    ).unwrap();
    println!("{:?}", expr);
    let llvm_context = Context::create();
    let opts = Yul2IROptions::test("Token");
    let mut context = Yul2IRContext::new_with_object(&llvm_context, &opts, expr);
    let emited_bc = context.emit("yul_parser_tuple_test").unwrap();
    std::fs::write("test.out.wasm", emited_bc).unwrap();
}

#[test]
fn yul_parser_base_instruction_test() {
    let expr = yul::ObjectParser::new()
        .parse(
            r#"
            object "yul_parser_base_instruction_test" {
                code {
                    datacopy(0, dataoffset("runtime"), datasize("runtime"))
                    return(0, datasize("runtime"))
                }

                object "runtime" {
                    code {
                        // Define memory offset
                        let offset := 0x00

                        // Get chain ID
                        mstore(offset, chainid())             // Store Chain ID at 0x00
                        offset := add(offset, 0x20)           // Update offset

                        // Get base fee
                        mstore(offset, basefee())             // Store Basefee
                        offset := add(offset, 0x20)           // Update offset

                        // Get transaction origin
                        mstore(offset, origin())              // Store Origin
                        offset := add(offset, 0x20)           // Update offset

                        // Get current block miner address
                        mstore(offset, coinbase())            // Store Coinbase
                        offset := add(offset, 0x20)           // Update offset

                        // Get current block timestamp
                        mstore(offset, timestamp())           // Store Timestamp
                        offset := add(offset, 0x20)           // Update offset

                        // Get current block number
                        mstore(offset, number())              // Store Block Number
                        offset := add(offset, 0x20)           // Update offset

                        // Get current block difficulty (only for PoW, not recommended for PoS)
                        mstore(offset, difficulty())          // Store Difficulty
                        offset := add(offset, 0x20)           // Update offset

                        // Get current block prevrandao (random number used in PoS)
                        mstore(offset, prevrandao())          // Store Prevrandao
                        offset := add(offset, 0x20)           // Update offset

                        // Get recent block hash
                        let prevBlockHash := blockhash(number())
                        mstore(offset, prevBlockHash)         // Store Blockhash
                        offset := add(offset, 0x20)           // Update offset

                        // Return all stored data
                        let totalSize := sub(offset, 0x00)    // Calculate total data size
                        return(0x00, totalSize)
                    }
                }
            }
        "#,
        )
        .unwrap();
    println!("{:?}", expr);
    let llvm_context = Context::create();
    let opts = Yul2IROptions::test("yul_parser_base_instruction_test");
    let mut context = Yul2IRContext::new_with_object(&llvm_context, &opts, expr);
    let emited_bc = context.emit("yul_parser_base_instruction_test").unwrap();
    std::fs::write("test.out.wasm", emited_bc).unwrap();
}

#[test]
fn yul_parser_very_simple_empty() {
    let expr = yul::ObjectParser::new()
        .parse(
            r#"
        object "yul_parser_very_simple_empty" {
            code {
                function selector_ret() {
                }
            }
        }
        "#,
        )
        .unwrap();
    println!("{:?}", expr);
    let llvm_context = Context::create();
    let opts = Yul2IROptions::test("yul_parser_very_simple_empty");
    let mut context = Yul2IRContext::new_with_object(&llvm_context, &opts, expr);
    let emited_bc = context.emit("yul_parser_very_simple_empty").unwrap();
    std::fs::write("test.out.wasm", emited_bc).unwrap();
}

#[test]
fn yul_parser_datasize() {
    let expr = yul::ObjectParser::new()
        .parse(
            r#"
        object "yul_parser_datasize" {
            code {
                function selector_ret() -> r, s, v {
                    r := datasize("yul_parser_datasize")
                    s := datasize("userdata")
                    v := dataoffset("yul_parser_datasize_deployed")
                }
                selector_ret()
            }

            object "yul_parser_datasize_deployed" {
                code {
                }
            }

            data ".metadata" hex"aabb"
            data "userdata" hex"1122"
        }
        "#,
        )
        .unwrap();
    println!("{:?}", expr);
    let llvm_context = Context::create();
    let opts = Yul2IROptions::test("yul_parser_datasize");
    let mut context = Yul2IRContext::new_with_object(&llvm_context, &opts, expr);
    let emited_bc = context.emit("yul_parser_datasize").unwrap();
    std::fs::write("test.out.wasm", emited_bc).unwrap();
}

#[test]
fn yul_parser_very_simple_no_return() {
    let expr = yul::ObjectParser::new().parse(
        r#"
        object "yul_parser_very_simple_no_return" {
            code {
                selector_ret()

                function selector_ret() {
                    let s := div(calldataload(0), 0x100000000000000000000000000000000000000000000000000000000)
                    let z := div(calldataload(0), 0x100000000000000000000000000000000000000000000000000000000)
                }
            }

            object "yul_parser_very_simple_no_return_deployed" {
                code {
                    selector_ret()

                    function selector_ret() {
                        let s := div(calldataload(0), 0x100000000000000000000000000000000000000000000000000000000)
                        let z := div(calldataload(0), 0x100000000000000000000000000000000000000000000000000000000)
                    }
                }
            }
        }
        "#,
    ).unwrap();
    println!("{:?}", expr);
    let llvm_context = Context::create();
    let opts = Yul2IROptions::test("yul_parser_very_simple_no_return");
    let mut context = Yul2IRContext::new_with_object(&llvm_context, &opts, expr);
    let emited_bc = context.emit("yul_parser_very_simple_no_return").unwrap();
    std::fs::write("test.out.wasm", emited_bc).unwrap();
}

#[test]
fn yul_parser_create_test() {
    let expr = yul::ObjectParser::new()
        .parse(
            r#"
        object "MainContract" {
            code {
                // Create a new contract using create instruction
                let addr1 := create(0, 0, 0x20)

                // Check if create was successful
                if iszero(addr1) {
                    // If creation failed, revert
                    revert(0, 0)
                }

                // Define salt value
                let salt := 0x12345678

                // Create second contract using create2 instruction
                let addr2 := create2(0, 0, 0x20, salt)

                // Check if create2 was successful
                if iszero(addr2) {
                    // If creation failed, revert
                    revert(0, 0)
                }

                // Store both created addresses in memory
                mstore(0x00, addr1)  // Store address from create
                mstore(0x20, addr2)  // Store address from create2

                // Return both addresses
                return(0x00, 0x40)
            }

            object "SubContract" {
                code {
                    // Simple logic for the sub-contract: store 0x01 at memory location 0x00 and return
                    mstore(0x00, 0x01)
                    return(0x00, 0x20)
                }
            }
        }
        "#,
        )
        .unwrap();
    println!("{:?}", expr);
    let llvm_context = Context::create();
    let opts = Yul2IROptions::test("yul_parser_create_test");
    let mut context = Yul2IRContext::new_with_object(&llvm_context, &opts, expr);
    let emited_bc = context.emit("yul_parser_create_test").unwrap();
    std::fs::write("test.out.wasm", emited_bc).unwrap();
}

#[test]
fn yul_parser_call_instruction_test() {
    let expr = yul::ObjectParser::new()
        .parse(
            r#"
        object "ExampleContract" {
            code {
                datacopy(0, dataoffset("Runtime"), datasize("Runtime"))
                return(0, datasize("Runtime"))
            }

            object "Runtime" {
                code {
                    let target := 0x1234567890123456789012345678901234567890

                    let callData := mload(0x40)
                    mstore(callData, 0x12345678)
                    mstore(add(callData, 4), 42)

                    let output := add(callData, 36)
                    let outputSize := 32

                    {
                        let success := call(
                            gas(),
                            target,
                            100000000000000,
                            callData,
                            36,
                            output,
                            outputSize
                        )

                        if iszero(success) {
                            revert(0, 0)
                        }

                        // Read result from return data
                        let callResult := mload(output)
                        // Process return value (omitted here, directly print to log)
                    }

                    // **2. DELEGATECALL instruction: delegate call to library contract logic**
                    {
                        let success := delegatecall(
                            gas(),
                            target,
                            callData,
                            36,
                            output,
                            outputSize
                        )

                        if iszero(success) {
                            revert(0, 0)
                        }

                        // Read result from return data
                        let delegatecallResult := mload(output)
                        // Process return value (omitted here, directly print to log)
                    }

                    // **3. STATICCALL instruction: read-only call to external contract**
                    {
                        let success := staticcall(
                            gas(),
                            target,
                            callData,
                            36,
                            output,
                            outputSize
                        )

                        if iszero(success) {
                            revert(0, 0)
                        }

                        // Read result from return data
                        let staticcallResult := mload(output)
                        // Process return value (omitted here, directly print to log)
                    }

                    // Return result (for demonstration, return the result of the last call)
                    return(output, outputSize)
                }
            }
        }
        "#,
        )
        .unwrap();
    println!("{:?}", expr);
    let llvm_context = Context::create();
    let opts = Yul2IROptions::test("Token");
    let mut context = Yul2IRContext::new_with_object(&llvm_context, &opts, expr);
    let emited_bc = context.emit("yul_parser_call_instruction_test").unwrap();
    std::fs::write("test.out.wasm", emited_bc).unwrap();
}

#[test]
fn yul_parser_selfdestruct_invalid_instruction_test() {
    let expr = yul::ObjectParser::new()
        .parse(
            r#"
        // Define a contract structure
        object "yul_parser_selfdestruct_invalid_instruction_test" {
            code {
                // Load the first 32 bytes of calldata as input parameter
                let action := calldataload(0)

                // Execute different operations based on input parameter
                switch action
                case 0 {
                    // If action == 0, destroy contract and transfer all balance
                    let beneficiary := caller()
                    selfdestruct(beneficiary)
                }
                default {
                    // If action != 0, trigger invalid instruction to throw exception
                    invalid()
                }
            }
        }
        "#,
        )
        .unwrap();
    println!("{:?}", expr);
    let llvm_context = Context::create();
    let opts = Yul2IROptions::test("yul_parser_selfdestruct_invalid_instruction_test");
    let mut context = Yul2IRContext::new_with_object(&llvm_context, &opts, expr);
    let emited_bc = context
        .emit("yul_parser_selfdestruct_invalid_instruction_test")
        .unwrap();
    std::fs::write("test.out.wasm", emited_bc).unwrap();
}

#[test]
fn yul_parser_loadimmutable_instruction_test() {
    let expr = yul::ObjectParser::new()
        .parse(
            r#"
        object "LoadImmutableTest" {
            code {
                datacopy(0, dataoffset("Runtime"), datasize("Runtime"))
                setimmutable(0, "ADDRESS", 0xabcd)
                setimmutable(0x20, "FLAG", 0x1)
                return(0, datasize("Runtime"))
            }
            object "Runtime" {
                code {
                    let addr := loadimmutable("ADDRESS")
                    let flag := loadimmutable("FLAG")
                    mstore(0, or(shl(0x20, addr), flag))
                    return(0, 0x40)
                }
            }
        }
        "#,
        )
        .unwrap();
    println!("{:?}", expr);
    let llvm_context = Context::create();
    let opts = Yul2IROptions::test("yul_parser_loadimmutable_instruction_test");
    let mut context = Yul2IRContext::new_with_object(&llvm_context, &opts, expr);
    let emited_bc = context
        .emit("yul_parser_loadimmutable_instruction_test")
        .unwrap();
    std::fs::write("test.out.wasm", emited_bc).unwrap();
}

#[test]
fn yul_parser_simple_nested_object() {
    let expr = yul::ObjectParser::new()
        .parse(
            r#"
        object "OuterObject" {
            code {

            }
            object "OuterObject_deployed" {
                code {
                    function outer_function() -> result {
                        result := 42
                    }
                    let outer_result := outer_function()
                    mstore(0x00, outer_result)
                    return(0x00, 0x20)
                }
            }
            object "InnerObject" {
                code {
                    function inner_function() -> result {
                        result := 24
                    }
                    let inner_result := inner_function()
                    mstore(0x20, inner_result)
                    return(0x20, 0x20)
                }

                object "InnerObject_deployed" {
                    code {

                    }
                }
            }
        }
        "#,
        )
        .unwrap();
    let llvm_context = Context::create();
    let opts = Yul2IROptions::test("Token");
    let mut context = Yul2IRContext::new_with_object(&llvm_context, &opts, expr);
    let emited_bc = context.emit("yul_parser_simple_nested_object").unwrap();
    std::fs::write("test.out.wasm", emited_bc).unwrap();
}

#[test]
fn yul_parser_simple_break() {
    let expr = yul::ObjectParser::new()
        .parse(
            r#"
        object "Token" {
            code {
                // Initialize variables
                let sum := 0
                let i := 0

                // Yul for loop
                for { } lt(i, 10) { i := add(i, 1) } {
                    // Break loop if i equals 5
                    if eq(i, 5) {
                        break
                    }

                    // Add i to sum
                    sum := add(sum, i)
                }

                // Return sum value
                mstore(0, sum)
                return(0, 32)
            }
        }
        "#,
        )
        .unwrap();
    println!("{:?}", expr);
    let llvm_context = Context::create();
    let opts = Yul2IROptions::test("Token");
    let mut context = Yul2IRContext::new_with_object(&llvm_context, &opts, expr);
    let emited_bc = context.emit("yul_parser_simple_break").unwrap();
    std::fs::write("test.out.wasm", emited_bc).unwrap();
}

#[test]
fn yul_parser_simple_continue() {
    let expr = yul::ObjectParser::new()
        .parse(
            r#"
        object "Token" {
            code {
                // Initialize variables
                let sum := 0
                let i := 0

                // Yul for loop
                for { } lt(i, 10) { i := add(i, 1) } {
                    // Skip iteration if i equals 5
                    if eq(i, 5) {
                        continue
                    }

                    // Add i to sum
                    sum := add(sum, i)
                }

                // Return sum value
                mstore(0, sum)
                return(0, 32)
            }
        }
        "#,
        )
        .unwrap();
    println!("{:?}", expr);
    let llvm_context = Context::create();
    let opts = Yul2IROptions::test("Token");
    let mut context = Yul2IRContext::new_with_object(&llvm_context, &opts, expr);
    let emited_bc = context.emit("yul_parser_simple_continue").unwrap();
    std::fs::write("test.out.wasm", emited_bc).unwrap();
}

#[test]
fn yul_test_complex_if() {
    let expr = yul::ObjectParser::new()
        .parse(
            r#"
        object "SecuritiesToken_69926_deployed" {
            code {
                function panic_error_0x11() {
                    debug_print(1)
                }
                function checked_mul_t_uint256(x, y) -> product {

                    if iszero(
                        or(
                            iszero(x),
                            eq(y, div(product, x))
                        )
                    ) { panic_error_0x11() }

                }
            }
            data ".metadata" hex"a2"
        }
        "#,
        )
        .unwrap();
    println!("{:?}", expr);
    let llvm_context = Context::create();
    let opts = Yul2IROptions::test("PopTest");
    let mut context = Yul2IRContext::new_with_object(&llvm_context, &opts, expr);
    let emited_bc = context.emit("yul_test_complex_if").unwrap();
    std::fs::write("test.out.wasm", emited_bc).unwrap();
}
