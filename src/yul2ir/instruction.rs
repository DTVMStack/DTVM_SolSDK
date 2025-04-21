// Copyright (C) 2024-2025 Ant Group Co., Ltd. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use crate::yul2ir::ast::{FunctionCall, Identifier, Literal};
use crate::yul2ir::infer::ExpectedType;
use crate::yul2ir::yul_instruction::YulInstructionName;
use crate::{
    yul2ir::ast::Expression,
    yul2ir::context::{CompileResult, Yul2IRContext},
    yul2ir::errors::ASTLoweringError,
};
use ethereum_types::U256;
use inkwell::types::StringRadix;
use inkwell::values::{BasicValue, BasicValueEnum, PointerValue};
use inkwell::IntPredicate;

use super::yul_instruction::{YulLowLevelValue, YulLowLevelValueType};

fn check_args_count(
    instr: &YulInstructionName,
    args: &[BasicValueEnum<'_>],
    expected: usize,
) -> Result<(), ASTLoweringError> {
    if args.len() != expected {
        return Err(ASTLoweringError {
            message: format!(
                "Instruction {:?} Expected {} arguments, but got {}",
                instr,
                expected,
                args.len()
            ),
        });
    }
    Ok(())
}

impl<'a> Yul2IRContext<'a> {
    fn read_string_literal(&self, expr: &Expression) -> Option<String> {
        let mut result: Option<String> = None;
        if let Expression::Literal(Literal::StringLiteral(s, _)) = expr {
            result = Some(s.str.clone());
        }
        result
    }

    /// Determine expected types for each argument of a YUL instruction
    fn get_instruction_arg_expected_types(
        &self,
        _yul_func_name: &str,
        instr: &YulInstructionName,
        _expected_type: ExpectedType,
        arg_count: usize,
    ) -> Vec<ExpectedType> {
        let mut arg_expected_types: Vec<ExpectedType> = match instr {
            // For MLoad, we don't need an ExpectedType::U256 for the memory pointer
            // since we'll be converting it to i32 anyway - using I32 may be more efficient
            // offset maybe accept i32/bytes32, to avoid unnecessary type conversions and encoding/decoding
            YulInstructionName::MLoad => vec![ExpectedType::Untyped],

            // For MStore, first argument should be a memory pointer (i32), second depends on what we're storing
            // offset maybe accept i32/bytes32, to avoid unnecessary type conversions and encoding/decoding
            YulInstructionName::MStore => vec![ExpectedType::Untyped, ExpectedType::Bytes32],

            // For SLoad/TLoad/SStore/TStore, the slot should be a u256
            YulInstructionName::SLoad | YulInstructionName::TLoad => {
                vec![ExpectedType::Bytes32]
            }
            YulInstructionName::SStore | YulInstructionName::TStore => {
                if self.opts.enable_storage_load_store_little_endian {
                    vec![ExpectedType::Bytes32, ExpectedType::U256]
                } else {
                    vec![ExpectedType::Bytes32, ExpectedType::Bytes32]
                }
            }

            // For arithmetic operations like Add/Sub/Mul, use u256 as expected args type
            // This is because the result maybe different from the input types(maybe overflow for small int types)
            YulInstructionName::Add
            | YulInstructionName::Sub
            | YulInstructionName::Mul
            | YulInstructionName::Div
            | YulInstructionName::SDiv
            | YulInstructionName::Mod
            | YulInstructionName::SMod
            | YulInstructionName::And
            | YulInstructionName::Or
            | YulInstructionName::Xor => vec![ExpectedType::Bytes32, ExpectedType::Bytes32], // TODO: maybe expect bytes32 is faster?

            // For comparison operations, we generally want i32 results but operate on the input types
            YulInstructionName::Lt
            | YulInstructionName::Gt
            | YulInstructionName::SLt
            | YulInstructionName::SGt
            | YulInstructionName::Eq => vec![ExpectedType::Untyped, ExpectedType::Untyped], // TODO: u256/bytes32 which faster here?

            // For IsZero, the input type should match the context's expected type
            YulInstructionName::IsZero => vec![ExpectedType::Untyped],

            // For shift operations, first arg is shift amount (i32), second is value (expected_type)
            YulInstructionName::Shl | YulInstructionName::Shr | YulInstructionName::Sar => {
                vec![ExpectedType::Untyped, ExpectedType::Untyped]
            }

            // For memory operations, typically use I32 for offsets and sizes
            YulInstructionName::MCopy => {
                vec![ExpectedType::I32, ExpectedType::I32, ExpectedType::I32]
            }
            YulInstructionName::MStore8 => vec![ExpectedType::I32, ExpectedType::I32],
            YulInstructionName::CallDataLoad => vec![ExpectedType::I32],
            YulInstructionName::CallDataCopy => {
                vec![ExpectedType::I32, ExpectedType::I32, ExpectedType::I32]
            }
            YulInstructionName::CodeCopy => {
                vec![ExpectedType::I32, ExpectedType::I32, ExpectedType::I32]
            }
            YulInstructionName::ReturnDataCopy => {
                vec![ExpectedType::I32, ExpectedType::I32, ExpectedType::I32]
            }
            YulInstructionName::Return => vec![ExpectedType::I32, ExpectedType::I32],
            YulInstructionName::Revert => vec![ExpectedType::I32, ExpectedType::I32],

            // Keccak256 uses memory pointer and size
            YulInstructionName::Keccak256 => vec![ExpectedType::I32, ExpectedType::I32],

            // For contract creation, use appropriate types for value, memory offset, and size
            YulInstructionName::Create => {
                vec![ExpectedType::U256, ExpectedType::I32, ExpectedType::I32]
            }
            YulInstructionName::Create2 => vec![
                ExpectedType::U256,
                ExpectedType::I32,
                ExpectedType::I32,
                ExpectedType::U256,
            ],

            // For contract calls, use appropriate types for gas, address, value, and memory params
            YulInstructionName::Call => vec![
                ExpectedType::I64,     // gas
                ExpectedType::Bytes32, // address
                ExpectedType::U256,    // value
                ExpectedType::I32,     // in_offset
                ExpectedType::I32,     // in_size
                ExpectedType::I32,     // out_offset
                ExpectedType::I32,     // out_size
            ],
            YulInstructionName::DelegateCall | YulInstructionName::StaticCall => vec![
                ExpectedType::I64,     // gas
                ExpectedType::Bytes32, // address
                ExpectedType::I32,     // in_offset
                ExpectedType::I32,     // in_size
                ExpectedType::I32,     // out_offset
                ExpectedType::I32,     // out_size
            ],

            // Default to Untyped for all arguments when no specific requirement
            _ => vec![ExpectedType::Untyped; arg_count],
        };

        // Ensure we have expected types for all arguments (use Untyped if not specified)
        while arg_expected_types.len() < arg_count {
            arg_expected_types.push(ExpectedType::Untyped);
        }

        arg_expected_types
    }

    /// Detect and optimize common YUL instruction patterns
    fn try_optimize_instruction_pattern(
        &self,
        yul_func_name: &str,
        instr: &YulInstructionName,
        args_exprs: &[Expression],
        _args_values: &[ExpectedType],
    ) -> (
        Option<YulLowLevelValue<'a>>,
        Option<Vec<YulLowLevelValue<'a>>>,
    ) {
        // Special case optimization for function selector extraction: shr(224, calldataload(0))
        if matches!(instr, YulInstructionName::Shr) && args_exprs.len() == 2 {
            // shr 224 is special, it means read the last 4 bytes of second arg
            if let Expression::Literal(Literal::DecimalNumberLiteral(dec, _)) = &args_exprs[0] {
                if dec.dec == "224" {
                    // Check if second arg is calldataload(0)
                    if let Expression::FunctionCall(func_call) = &args_exprs[1] {
                        if func_call.id.name == "calldataload" && func_call.arguments.len() == 1 {
                            if let Expression::Literal(Literal::DecimalNumberLiteral(pos_dec, _)) =
                                &func_call.arguments[0]
                            {
                                if pos_dec.dec == "0" {
                                    // This is shr(224, calldataload(0)), which extracts the function selector
                                    // Optimize by directly calling a specialized wrapper that only loads 4 bytes
                                    if let Ok(result) =
                                        self.build_call("wrapper_calldata_load_selector", &[])
                                    {
                                        return (
                                            Some(YulLowLevelValue {
                                                value_type: YulLowLevelValueType::I32,
                                                value: result,
                                            }),
                                            None,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Optimize for shl with constant arguments
        if matches!(instr, YulInstructionName::Shl) && args_exprs.len() == 2 {
            let shift = args_exprs[0].clone();
            let value = args_exprs[1].clone();
            let shift_constant =
                if let Some(shift_constant) = self.fetch_not_string_literal_constant(&shift) {
                    if shift_constant < U256::from(i32::MAX) {
                        Some(shift_constant.as_u64() as i32)
                    } else {
                        None
                    }
                } else {
                    None
                };
            let value_constant =
                if let Some(value_constant) = self.fetch_not_string_literal_constant(&value) {
                    if value_constant < U256::from(i32::MAX) {
                        Some(value_constant.as_u64())
                    } else {
                        None
                    }
                } else {
                    None
                };

            if let (Some(shift), Some(value)) = (shift_constant, value_constant) {
                // Shift operation on constants
                let result = if shift <= 0 {
                    Some(U256::from(value))
                } else if shift >= 256 {
                    Some(U256::from(0))
                } else {
                    Some(U256::from(value) << shift)
                };
                if let Some(result) = result {
                    if result <= U256::from(i32::MAX) {
                        return (
                            Some(YulLowLevelValue {
                                value_type: YulLowLevelValueType::I32,
                                value: self.i32_type().const_int(result.as_u64(), false).into(),
                            }),
                            None,
                        );
                    }
                    if result <= U256::from(i64::MAX) {
                        return (
                            Some(YulLowLevelValue {
                                value_type: YulLowLevelValueType::I64,
                                value: self.i64_type().const_int(result.as_u64(), false).into(),
                            }),
                            None,
                        );
                    }
                    return (
                        Some(YulLowLevelValue {
                            value_type: YulLowLevelValueType::U256,
                            value: self
                                .u256_type()
                                .const_int_from_string(&result.to_string(), StringRadix::Decimal)
                                .unwrap()
                                .into(),
                        }),
                        None,
                    );
                }
            }
        }

        // Optimize for common operation: add(calldatasize(), not(3)) which is frequently used
        // for checking calldata size in function selector validation
        if matches!(instr, YulInstructionName::Add) && args_exprs.len() == 2 {
            // Check if first arg is calldatasize()
            if self
                .matches_yul_instruction(&args_exprs[0], "calldatasize", 0)
                .is_some()
            {
                // Check if second arg is not(3)
                if let Some(not_call_args) = self.matches_yul_instruction(&args_exprs[1], "not", 1)
                {
                    if self.matches_constant_literal(&not_call_args[0], U256::from(3)) {
                        // This is add(calldatasize(), not(3)), which is used for checking calldata size
                        // calldatasize not too large, and not(3) equals -4, so if calldatasize() > 4, return minus result
                        // else return negative result.
                        // Optimize by directly calling a specialized wrapper
                        if let Ok(result) = self.build_call("wrapper_calldata_size_minus_4", &[]) {
                            return (
                                Some(YulLowLevelValue {
                                    value_type: YulLowLevelValueType::I32,
                                    value: result,
                                }),
                                None,
                            );
                        }
                    }
                }
            }
        }

        // optimize for mstore(offset, constant_value)
        if matches!(instr, YulInstructionName::MStore) && args_exprs.len() == 2 {
            let offset = &args_exprs[0].clone();
            let value = &args_exprs[1].clone();

            // Helper function to handle memory offset
            let get_memory_offset = || -> BasicValueEnum<'a> {
                let offset_val = self
                    .walk_expr_with_type(yul_func_name, offset, ExpectedType::I32)
                    .unwrap();
                self.try_into_i32_across_int(&offset_val.get_value())
                    .unwrap()
                    .into()
            };

            // Helper function to handle constant value storage
            let store_constant_value = |evm_mem: BasicValueEnum<'a>, value: U256| {
                if value < U256::from(u32::MAX) {
                    let value_i32 = self.i32_type().const_int(value.as_u64(), false);
                    self.build_void_call("wrapper_mstore_u32", &[evm_mem, value_i32.into()])
                        .unwrap();
                } else if value < U256::from(u64::MAX) {
                    let value_i64 = self.i64_type().const_int(value.as_u64(), false);
                    self.build_void_call("wrapper_mstore_u64", &[evm_mem, value_i64.into()])
                        .unwrap();
                } else {
                    let mut constant_bytes32 = [0u8; 32];
                    value.to_big_endian(&mut constant_bytes32);
                    let bytes32_global_var = self.add_global_constant_bytes32(
                        &constant_bytes32,
                        &format!("global_constant_bytes32_{}", value),
                    );
                    let bytes32_global_ptr = bytes32_global_var.as_pointer_value();
                    self.build_void_call(
                        "wrapper_mstore_bytes32",
                        &[evm_mem, bytes32_global_ptr.into()],
                    )
                    .unwrap();
                }
            };

            if let Some(value) = self.fetch_not_string_literal_constant(value) {
                let evm_mem = get_memory_offset();
                store_constant_value(evm_mem, value);
                return (
                    Some(YulLowLevelValue {
                        value_type: YulLowLevelValueType::I32,
                        value: self.i32_type().const_zero().into(),
                    }),
                    None,
                );
            }
        }

        //  Optimize iszero(eq(value, and(value, sub(shl(160, 1), 1)))) by checking if the first 12 bytes of u256/bytes32 are zero

        if yul_func_name.contains("abi_decode_address")
            || yul_func_name.contains("abi_decode_t_address")
        {
            // TODO: Since abi_decode_address only uses the last 20 bytes of the address, we can simply return false for the iszero check
            if let YulInstructionName::IsZero = instr {
                return (
                    Some(YulLowLevelValue {
                        value_type: YulLowLevelValueType::I32,
                        value: self.i32_type().const_zero().into(),
                    }),
                    None,
                );
            }
        }

        // TODO: and(addr, sub(shl(160, 1), 1)) can be optimized to clear the first 12 bytes of the address
        // TODO: However, since only the last 20 bytes of the address are used, we can implement a special optimization
        if self.opts.enable_all_optimizers {
            if let YulInstructionName::And = instr {
                let lhs = args_exprs.first().unwrap();
                let rhs = args_exprs.get(1).unwrap();

                // Use simplified address mask pattern matching
                if let Some(Expression::Identifier(addr_iden)) = self.matches_address_mask_pattern(
                    &Expression::FunctionCall(Box::new(FunctionCall {
                        id: Identifier {
                            name: "and".to_string(),
                        },
                        arguments: vec![lhs.clone(), rhs.clone()],
                    })),
                ) {
                    // Optimize by directly returning the address since only the last 20 bytes will be used
                    let result = self.walk_identifier(&addr_iden).unwrap();
                    return (Some(result), None);
                }
            }
        }

        if yul_func_name.contains("abi_decode_address")
            || yul_func_name.contains("abi_decode_t_address")
        {
            // code like:
            // function abi_decode_address() -> value {
            //     value := calldataload(4)
            //     if iszero(eq(value, and(value, sub(shl(160, 1), 1)))) { revert(0, 0) }
            // }
            // In abi_decode_address, the 'and' operation is best suited for bytes32 type parameters
            if let YulInstructionName::And = instr {
                let lhs = args_exprs.first().unwrap();
                let rhs = args_exprs.get(1).unwrap();
                if let Expression::Identifier(id) = lhs {
                    let lhs_bytes32_ptr = self.get_bytes32_identifier_pointer(id);
                    if let Some(lhs_bytes32_ptr) = lhs_bytes32_ptr {
                        // TODO: expect bytes32 pointer constant
                        let rhs = self
                            .walk_expr_with_type(yul_func_name, rhs, ExpectedType::Bytes32)
                            .unwrap();
                        let rhs = self.try_into_bytes32_pointer(&rhs.value).unwrap();
                        // call wrapper_bytes32_and
                        let result = self
                            .fast_alloca(self.bytes32_type(), "bytes32_and_result")
                            .unwrap();
                        self.build_void_call(
                            "wrapper_bytes32_and",
                            &[lhs_bytes32_ptr.into(), rhs, result.into()],
                        )
                        .unwrap();
                        return (
                            Some(YulLowLevelValue {
                                value_type: YulLowLevelValueType::Bytes32Pointer,
                                value: result.into(),
                            }),
                            None,
                        );
                    }
                }
            }
        }
        // In the fun_transfer function, the is_zero instruction is often used to check if an address is the zero address. Therefore, we prioritize parsing the argument as an address (bytes32) type to avoid unnecessary encoding and decoding.
        if yul_func_name.contains("fun_transfer") && matches!(instr, YulInstructionName::IsZero) {
            // Process the argument as expected bytes32 type and continue execution
            let args = self.walk_args(yul_func_name, args_exprs, &[ExpectedType::Bytes32]);
            return (None, Some(args));
        }

        // Optimize mstore(64, value) and mload(64) operations to use wasm global variables
        if self.opts.enable_all_optimizers {
            if matches!(instr, YulInstructionName::MStore) && args_exprs.len() == 2 {
                if let Some(offset_constant) =
                    self.fetch_not_string_literal_constant(&args_exprs[0])
                {
                    if offset_constant == U256::from(64) {
                        // mstore(64, value)
                        // call wrapper_set_memptr_global(value)
                        let value_expr = args_exprs[1].clone();
                        let value = self
                            .walk_expr_with_type(yul_func_name, &value_expr, ExpectedType::I32)
                            .unwrap();
                        let value = self.try_into_i32_value(&value.value, &value_expr).unwrap();
                        self.build_void_call("wrapper_set_memptr_global", &[value.into()])
                            .unwrap();

                        // let memptr_global = self.memptr_global.borrow().unwrap();
                        // self.build_store(
                        //     memptr_global.as_pointer_value(),
                        //     value.as_basic_value_enum(),
                        // )
                        // .unwrap();

                        return (
                            Some(YulLowLevelValue {
                                value_type: YulLowLevelValueType::I32,
                                value: self.i32_type().const_zero().into(),
                            }),
                            None,
                        );
                    }
                }
            }
            if matches!(instr, YulInstructionName::MLoad) && args_exprs.len() == 1 {
                if let Some(offset_constant) =
                    self.fetch_not_string_literal_constant(&args_exprs[0])
                {
                    if offset_constant == U256::from(64) {
                        // mload(64)
                        // call wrapper_get_memptr_global()
                        let result = self.build_call("wrapper_get_memptr_global", &[]).unwrap();

                        // let memptr_global = self.memptr_global.borrow().unwrap();
                        // let result = self
                        //     .build_load(self.i32_type(), memptr_global.as_pointer_value(), "")
                        //     .unwrap();
                        return (
                            Some(YulLowLevelValue {
                                value_type: YulLowLevelValueType::I32,
                                value: result,
                            }),
                            None,
                        );
                    }
                }
            }
        }

        (None, None)
    }

    fn walk_args(
        &self,
        yul_func_name: &str,
        args_exprs: &[Expression],
        arg_expected_types: &[ExpectedType],
    ) -> Vec<YulLowLevelValue<'a>> {
        // Evaluate each argument with its expected type
        let args: Vec<YulLowLevelValue<'a>> = args_exprs
            .iter()
            .enumerate()
            .map(|(i, arg)| {
                let arg_expected_type = if i < arg_expected_types.len() {
                    arg_expected_types[i]
                } else {
                    ExpectedType::Untyped
                };
                self.walk_expr_with_type(yul_func_name, arg, arg_expected_type)
                    .unwrap()
            })
            .collect();
        args
    }

    pub(crate) fn walk_yul_instruction(
        &self,
        yul_func_name: &str,
        instr: YulInstructionName,
        args: &[Expression],
        expected_type: ExpectedType,
    ) -> CompileResult<'a> {
        let args_exprs = args;

        // Get expected types for each argument
        let arg_expected_types = self.get_instruction_arg_expected_types(
            yul_func_name,
            &instr,
            expected_type,
            args.len(),
        );

        // This is done to avoid redundant walk_args calls (hostapi calls will still be preserved)
        let (optimized_result, optimized_args) = self.try_optimize_instruction_pattern(
            yul_func_name,
            &instr,
            args_exprs,
            &arg_expected_types,
        );

        // First try to match and optimize common instruction patterns
        if let Some(optimized_result) = optimized_result {
            return Ok(optimized_result);
        }

        let args_low_level = if let Some(optimized_args) = optimized_args {
            optimized_args
        } else {
            self.walk_args(yul_func_name, args_exprs, &arg_expected_types)
        };
        let args = args_low_level
            .iter()
            .map(|arg| arg.get_value())
            .collect::<Vec<_>>();

        match instr {
            YulInstructionName::Stop => {
                self.build_void_call("wrapper_stop", &args)?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I32,
                    value: self.i32_type().const_zero().into(),
                })
            }
            YulInstructionName::Add => {
                check_args_count(&instr, &args, 2)?;
                let lhs = args.first().unwrap();
                let rhs = args.get(1).unwrap();

                if self.is_int32_value(lhs) && self.is_int32_value(rhs) {
                    // because i32 add maybe cause overflow, so we use i64 add
                    let lhs = self.int_as_i64(lhs.into_int_value())?;
                    let rhs = self.int_as_i64(rhs.into_int_value())?;
                    // Both operands are i64
                    let result = self
                        .builder
                        .borrow_mut()
                        .build_int_add(lhs, rhs, "add_result")?;
                    return Ok(YulLowLevelValue {
                        value_type: YulLowLevelValueType::from_int_type(result.get_type()),
                        value: result.into(),
                    });
                } else if (self.is_bytes32_value(lhs) || self.is_bytes32_pointer_value(lhs))
                    && (self.is_bytes32_value(rhs) || self.is_bytes32_pointer_value(rhs))
                {
                    // unify to bytes32 pointer
                    // use bytes32 add
                    let (lhs, rhs) = self.unify_to_bytes32_pointer(lhs, rhs)?;
                    let ret_ty = self.bytes32_type();
                    let result_ptr = self.fast_alloca(ret_ty, "add_result")?;
                    self.build_void_call("wrapper_bytes32_add", &[lhs, rhs, result_ptr.into()])?;
                    return Ok(YulLowLevelValue {
                        value_type: YulLowLevelValueType::Bytes32Pointer,
                        value: result_ptr.into(),
                    });
                } else {
                    // At least one operand is u256 or different bit widths
                    let lhs = self.try_into_int(lhs)?;
                    let rhs = self.try_into_int(rhs)?;
                    let (lhs_u256, rhs_u256) = self.unify_to_u256(lhs, rhs)?;
                    let result = self.builder.borrow_mut().build_int_add(
                        lhs_u256,
                        rhs_u256,
                        "add_result",
                    )?;
                    return Ok(YulLowLevelValue {
                        value_type: YulLowLevelValueType::from_int_type(result.get_type()),
                        value: result.into(),
                    });
                };
            }
            YulInstructionName::Sub => {
                check_args_count(&instr, &args, 2)?;
                let (arg0, arg1) = self.unify_to_u256(
                    self.try_into_int(args.first().unwrap())?,
                    self.try_into_int(args.get(1).unwrap())?,
                )?;
                let result = self
                    .builder
                    .borrow_mut()
                    .build_int_sub(arg0, arg1, "sub_result")?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::from_int_type(result.get_type()),
                    value: result.into(),
                })
            }
            YulInstructionName::Mul => {
                check_args_count(&instr, &args, 2)?;
                let (arg0, arg1) = self.unify_to_u256(
                    self.try_into_int(args.first().unwrap())?,
                    self.try_into_int(args.get(1).unwrap())?,
                )?;
                let result = self
                    .builder
                    .borrow_mut()
                    .build_int_mul(arg0, arg1, "mul_result")?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::from_int_type(result.get_type()),
                    value: result.into(),
                })
            }
            YulInstructionName::AddMod => {
                // addmod(a, b, m) = (a + b) % m
                check_args_count(&instr, &args, 3)?;
                let a = args.first().unwrap();
                let b = args.get(1).unwrap();
                let m = args.get(2).unwrap();
                let (a, b) = self.unify_to_bytes32_pointer(a, b)?;
                let m = self.try_into_bytes32_pointer(m)?;
                let result_ptr = self.fast_alloca(self.bytes32_type(), "addmod_result")?;
                self.build_void_call("wrapper_addmod", &[a, b, m, result_ptr.into()])?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::Bytes32Pointer,
                    value: result_ptr.into(),
                })
            }
            YulInstructionName::MulMod => {
                // mulmod(a, b, m) = (a * b) % m
                check_args_count(&instr, &args, 3)?;
                let a = args.first().unwrap();
                let b = args.get(1).unwrap();
                let m = args.get(2).unwrap();
                let (a, b) = self.unify_to_bytes32_pointer(a, b)?;
                let m = self.try_into_bytes32_pointer(m)?;
                let result_ptr = self.fast_alloca(self.bytes32_type(), "mulmod_result")?;
                self.build_void_call("wrapper_mulmod", &[a, b, m, result_ptr.into()])?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::Bytes32Pointer,
                    value: result_ptr.into(),
                })
            }
            YulInstructionName::SignExtend => {
                // signextend(b, x) extends a smaller signed integer x to 256 bits while preserving its sign bit.
                // The bits before the specified byte count (b) are extended according to the sign of the most significant bit.
                check_args_count(&instr, &args, 2)?;
                let b = self.try_into_int(args.first().unwrap())?;
                let b = self.int_as_i32(b)?;
                let x = self.try_into_u256(args.get(1).unwrap())?;
                let x_ptr = self.get_value_pointer(x)?;
                let tmp_result = self.fast_alloca(self.u256_type(), "")?;
                self.build_void_call(
                    "wrapper_sign_extend",
                    &[b.into(), x_ptr.into(), tmp_result.into()],
                )?;
                let result = self.build_load(self.u256_type(), tmp_result, "")?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::U256,
                    value: result,
                })
            }
            YulInstructionName::Div => {
                check_args_count(&instr, &args, 2)?;
                let (arg0, arg1) = self.unify_to_u256(
                    self.try_into_int(args.first().unwrap())?,
                    self.try_into_int(args.get(1).unwrap())?,
                )?;
                let ret_value_ty = YulLowLevelValueType::from_int_type(arg0.get_type());
                Ok(YulLowLevelValue {
                    value_type: ret_value_ty,
                    value: self
                        .builder
                        .borrow_mut()
                        .build_int_unsigned_div(arg0, arg1, "div_result")?
                        .into(),
                })
            }
            YulInstructionName::SDiv => {
                check_args_count(&instr, &args, 2)?;
                let (arg0, arg1) = self.unify_to_u256(
                    self.try_into_int(args.first().unwrap())?,
                    self.try_into_int(args.get(1).unwrap())?,
                )?;
                let ret_value_ty = YulLowLevelValueType::from_int_type(arg0.get_type());
                Ok(YulLowLevelValue {
                    value_type: ret_value_ty,
                    value: self
                        .builder
                        .borrow_mut()
                        .build_int_signed_div(arg0, arg1, "sdiv_result")?
                        .into(),
                })
            }
            YulInstructionName::Mod => {
                check_args_count(&instr, &args, 2)?;
                let (arg0, arg1) = self.unify_to_u256(
                    self.try_into_int(args.first().unwrap())?,
                    self.try_into_int(args.get(1).unwrap())?,
                )?;
                let ret_value_ty = YulLowLevelValueType::from_int_type(arg0.get_type());
                Ok(YulLowLevelValue {
                    value_type: ret_value_ty,
                    value: self
                        .builder
                        .borrow_mut()
                        .build_int_unsigned_rem(arg0, arg1, "mod_result")?
                        .into(),
                })
            }
            YulInstructionName::SMod => {
                check_args_count(&instr, &args, 2)?;
                let (arg0, arg1) = self.unify_to_u256(
                    self.try_into_int(args.first().unwrap())?,
                    self.try_into_int(args.get(1).unwrap())?,
                )?;
                let ret_value_ty = YulLowLevelValueType::from_int_type(arg0.get_type());
                Ok(YulLowLevelValue {
                    value_type: ret_value_ty,
                    value: self
                        .builder
                        .borrow_mut()
                        .build_int_signed_rem(arg0, arg1, "smod_result")?
                        .into(),
                })
            }
            YulInstructionName::Exp => {
                check_args_count(&instr, &args, 2)?;
                let base = self.int_as_u256(self.try_into_int(args.first().unwrap())?)?;
                let base_ptr = self.get_value_pointer(base)?;
                let exp = self.int_as_u256(self.try_into_int(args.get(1).unwrap())?)?;
                let exp_ptr = self.get_value_pointer(exp)?;
                let result_ptr: PointerValue<'a> =
                    self.fast_alloca(self.u256_type(), "exp_result")?;
                self.build_void_call(
                    "wrapper_exp",
                    &[base_ptr.into(), exp_ptr.into(), result_ptr.into()],
                )?;
                // u256 pointer can't return directly, so we need to load the result
                let result = self.build_load(self.u256_type(), result_ptr, "")?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::U256,
                    value: result,
                })
            }
            YulInstructionName::Not => {
                check_args_count(&instr, &args, 1)?;
                let value = args.first().unwrap();

                if self.is_bytes32_value(value) {
                    let value_ptr = self.get_value_pointer(*value)?;
                    let result_ptr = self.fast_alloca(self.bytes32_type(), "not_result")?;
                    self.build_void_call(
                        "wrapper_bytes32_not",
                        &[value_ptr.into(), result_ptr.into()],
                    )?;
                    // bytes32 pointer can return directly
                    return Ok(YulLowLevelValue {
                        value_type: YulLowLevelValueType::Bytes32Pointer,
                        value: result_ptr.into(),
                    });
                }
                if self.is_bytes32_pointer_value(value) {
                    let result_ptr = self.fast_alloca(self.bytes32_type(), "not_result")?;
                    self.build_void_call("wrapper_bytes32_not", &[*value, result_ptr.into()])?;
                    // bytes32 pointer can return directly
                    return Ok(YulLowLevelValue {
                        value_type: YulLowLevelValueType::Bytes32Pointer,
                        value: result_ptr.into(),
                    });
                }

                let arg0 = self.int_as_u256(self.try_into_int(value)?)?;
                // bitwise "not" of x (every bit of x is negated)
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::U256,
                    value: self
                        .builder
                        .borrow_mut()
                        .build_not(arg0, "not_result")?
                        .into(),
                })
            }
            YulInstructionName::Lt => {
                check_args_count(&instr, &args, 2)?;
                let arg0 = args.first().unwrap();
                let arg1 = args.get(1).unwrap();

                // TODO: Test optimization effect better after recovery
                // if one of arg0 or arg1 is bytes32, then use bytes32 compare func maybe faster
                // if self.is_bytes32_value(arg0) || self.is_bytes32_pointer_value(arg0) || self.is_bytes32_value(arg1) || self.is_bytes32_pointer_value(arg1) {
                //     let (arg0, arg1) = self.unify_to_bytes32_pointer(arg0, arg1)?;
                //     // call bytes32_lt to compare
                //     let result_bool_i32 =
                //         self.build_call("wrapper_bytes32_lt", &[arg0, arg1])?;
                //     return Ok(YulLowLevelValue {
                //         value_type: YulLowLevelValueType::I32,
                //         value: result_bool_i32,
                //     });
                // }

                let arg0 = self.try_into_int(arg0)?;
                let arg1 = self.try_into_int(arg1)?;
                let (arg0, arg1) = self.unify_ints(arg0, arg1)?;
                // For u256 == u256 comparison, we need to convert the i1 result to i32 type with zero extension
                // to avoid issues with the higher bits
                let cmp_result_i1 = self.builder.borrow_mut().build_int_compare(
                    IntPredicate::ULT,
                    arg0,
                    arg1,
                    "",
                )?;
                let result = self.int_cast(cmp_result_i1, self.i32_type())?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I32,
                    value: result.into(),
                })
            }
            YulInstructionName::Gt => {
                check_args_count(&instr, &args, 2)?;
                let arg0 = args.first().unwrap();
                let arg1 = args.get(1).unwrap();

                // TODO: Test optimization effect better after recovery
                // if one of arg0 or arg1 is bytes32, then use bytes32 compare func maybe faster
                // if self.is_bytes32_value(arg0) || self.is_bytes32_value(arg1) {
                //     let (arg0, arg1) = self.unify_to_bytes32_pointer(arg0, arg1)?;

                //     // call bytes32_gt to compare
                //     let result_bool_i32 =
                //         self.build_call("wrapper_bytes32_gt", &[arg0, arg1])?;
                //     return Ok(YulLowLevelValue {
                //         value_type: YulLowLevelValueType::I32,
                //         value: result_bool_i32,
                //     });
                // }

                let arg0 = self.try_into_int(arg0)?;
                let arg1 = self.try_into_int(arg1)?;
                let (arg0, arg1) = self.unify_ints(arg0, arg1)?;
                // For u256 == u256 comparison, we need to convert the i1 result to i32 type with zero extension
                // to avoid issues with the higher bits
                let cmp_result_i1 = self.builder.borrow_mut().build_int_compare(
                    IntPredicate::UGT,
                    arg0,
                    arg1,
                    "",
                )?;
                let result = self.int_cast(cmp_result_i1, self.i32_type())?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I32,
                    value: result.into(),
                })
            }
            YulInstructionName::SLt => {
                check_args_count(&instr, &args, 2)?;
                let arg0 = args.first().unwrap();
                let arg1 = args.get(1).unwrap();

                // TODO: Test optimization effect better after recovery
                // if one of arg0 or arg1 is bytes32, then use bytes32 compare func maybe faster
                // if self.is_bytes32_value(arg0) || self.is_bytes32_pointer_value(arg0) || self.is_bytes32_value(arg1) || self.is_bytes32_pointer_value(arg1) {
                //     let (arg0, arg1) = self.unify_to_bytes32_pointer(arg0, arg1)?;
                //     // call bytes32_slt to compare
                //     let result_bool_i32 = self
                //         .build_call("wrapper_bytes32_slt", &[arg0, arg1])?;
                //     return Ok(YulLowLevelValue {
                //         value_type: YulLowLevelValueType::I32,
                //         value: result_bool_i32,
                //     });
                // }

                let arg0 = self.try_into_int(arg0)?;
                let arg1 = self.try_into_int(arg1)?;
                let (arg0, arg1) = self.unify_ints(arg0, arg1)?;
                // For u256 == u256 comparison, we need to convert the result from i1 to i32 type with zero extension,
                // otherwise the higher bits will be incorrect
                let cmp_result_i1 = self.builder.borrow_mut().build_int_compare(
                    IntPredicate::SLT,
                    arg0,
                    arg1,
                    "",
                )?;
                let result = self.int_cast(cmp_result_i1, self.i32_type())?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I32,
                    value: result.into(),
                })
            }
            YulInstructionName::SGt => {
                check_args_count(&instr, &args, 2)?;
                let arg0 = args.first().unwrap();
                let arg1 = args.get(1).unwrap();

                // TODO: Test optimization effect better after recovery
                // if one of arg0 or arg1 is bytes32, then use bytes32 compare func maybe faster
                // if self.is_bytes32_value(arg0) || self.is_bytes32_pointer_value(arg0) || self.is_bytes32_value(arg1) || self.is_bytes32_pointer_value(arg1) {
                //     let (arg0, arg1) = self.unify_to_bytes32_pointer(arg0, arg1)?;
                //     // call bytes32_sgt to compare
                //     let result_bool_i32 = self
                //         .build_call("wrapper_bytes32_sgt", &[arg0, arg1])?;
                //     return Ok(YulLowLevelValue {
                //         value_type: YulLowLevelValueType::I32,
                //         value: result_bool_i32,
                //     });
                // }

                let arg0 = self.try_into_int(arg0)?;
                let arg1 = self.try_into_int(arg1)?;
                let (arg0, arg1) = self.unify_ints(arg0, arg1)?;
                // For u256 == u256 comparison, we need to convert the result from i1 to i32 type with zero extension, otherwise the higher bits will be incorrect
                let cmp_result_i1 = self.builder.borrow_mut().build_int_compare(
                    IntPredicate::SGT,
                    arg0,
                    arg1,
                    "",
                )?;
                let result = self.int_cast(cmp_result_i1, self.i32_type())?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I32,
                    value: result.into(),
                })
            }
            YulInstructionName::Eq => {
                check_args_count(&instr, &args, 2)?;
                let arg0 = args.first().unwrap();
                let arg1 = args.get(1).unwrap();

                // if one of arg0 or arg1 is bytes32, then use bytes32 compare func maybe faster
                if self.is_bytes32_value(arg0)
                    || self.is_bytes32_pointer_value(arg0)
                    || self.is_bytes32_value(arg1)
                    || self.is_bytes32_pointer_value(arg1)
                {
                    let (arg0, arg1) = self.unify_to_bytes32_pointer(arg0, arg1)?;
                    // call bytes32_eq to compare
                    let result_bool_i32 = self.build_call("wrapper_bytes32_eq", &[arg0, arg1])?;
                    return Ok(YulLowLevelValue {
                        value_type: YulLowLevelValueType::I32,
                        value: result_bool_i32,
                    });
                }

                let arg0 = self.try_into_int(arg0)?;
                let arg1 = self.try_into_int(arg1)?;
                let (arg0, arg1) = self.unify_ints(arg0, arg1)?;
                // For u256 == u256 comparison, we need to convert the result from i1 to i32 type with zero extension, otherwise the higher bits will be incorrect
                let cmp_result_i1 = self.builder.borrow_mut().build_int_compare(
                    IntPredicate::EQ,
                    arg0,
                    arg1,
                    "",
                )?;
                let result = self.int_cast(cmp_result_i1, self.i32_type())?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I32,
                    value: result.into(),
                })
            }
            YulInstructionName::IsZero => {
                check_args_count(&instr, &args, 1)?;
                let value = args.first().unwrap();
                if self.is_bytes32_value(value) || self.is_bytes32_pointer_value(value) {
                    let value_ptr = self.try_into_bytes32_pointer(value)?;
                    let result_bool_i32 =
                        self.build_call("wrapper_bytes32_iszero", &[value_ptr])?;
                    return Ok(YulLowLevelValue {
                        value_type: YulLowLevelValueType::I32,
                        value: result_bool_i32,
                    });
                }
                let value = self.try_into_int(value)?;

                // Compare directly with zero constant of matching type
                let zero = value.get_type().const_zero();
                // cmp result is i1 type
                let cmp_result = self.builder.borrow_mut().build_int_compare(
                    IntPredicate::EQ,
                    value,
                    zero,
                    "is_zero",
                )?;
                // i1 is too small in wasm, the value will maybe wrong in prefix bits
                let result = self.int_cast(cmp_result, self.i32_type())?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I32,
                    value: result.into(),
                })
            }
            YulInstructionName::And => {
                check_args_count(&instr, &args, 2)?;
                let arg0 = args.first().unwrap();
                let arg1 = args.get(1).unwrap();

                if self.is_bytes32_value(arg0)
                    || self.is_bytes32_pointer_value(arg0)
                    || self.is_bytes32_value(arg1)
                    || self.is_bytes32_pointer_value(arg1)
                {
                    let (arg0, arg1) = self.unify_to_bytes32_pointer(arg0, arg1)?;
                    let result_ptr = self.fast_alloca(self.bytes32_type(), "and_result")?;
                    self.build_void_call("wrapper_bytes32_and", &[arg0, arg1, result_ptr.into()])?;
                    return Ok(YulLowLevelValue {
                        value_type: YulLowLevelValueType::Bytes32Pointer,
                        value: result_ptr.into(),
                    });
                }

                let (arg0, arg1) =
                    self.unify_to_u256(self.try_into_int(arg0)?, self.try_into_int(arg1)?)?;
                let result = self
                    .builder
                    .borrow_mut()
                    .build_and(arg0, arg1, "and_result")?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::U256,
                    value: result.into(),
                })
            }
            YulInstructionName::Or => {
                check_args_count(&instr, &args, 2)?;
                let arg0 = args.first().unwrap();
                let arg1 = args.get(1).unwrap();

                if self.is_bytes32_value(arg0) || self.is_bytes32_value(arg1) {
                    let (arg0, arg1) = self.unify_to_bytes32(arg0, arg1)?;
                    let arg0_ptr = self.get_value_pointer(arg0)?;
                    let arg1_ptr = self.get_value_pointer(arg1)?;
                    let result_ptr = self.fast_alloca(self.bytes32_type(), "or_result")?;
                    self.build_void_call(
                        "wrapper_bytes32_or",
                        &[arg0_ptr.into(), arg1_ptr.into(), result_ptr.into()],
                    )?;
                    // bytes32 pointer can return directly
                    return Ok(YulLowLevelValue {
                        value_type: YulLowLevelValueType::Bytes32Pointer,
                        value: result_ptr.into(),
                    });
                }
                let (arg0, arg1) =
                    self.unify_to_u256(self.try_into_int(arg0)?, self.try_into_int(arg1)?)?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::U256,
                    value: self
                        .builder
                        .borrow_mut()
                        .build_or(arg0, arg1, "or_result")?
                        .into(),
                })
            }
            YulInstructionName::Xor => {
                check_args_count(&instr, &args, 2)?;
                let arg0 = args.first().unwrap();
                let arg1 = args.get(1).unwrap();

                if self.is_bytes32_value(arg0) || self.is_bytes32_value(arg1) {
                    let (arg0, arg1) = self.unify_to_bytes32(arg0, arg1)?;
                    let arg0_ptr = self.get_value_pointer(arg0)?;
                    let arg1_ptr = self.get_value_pointer(arg1)?;
                    let result_ptr = self.fast_alloca(self.bytes32_type(), "xor_result")?;
                    self.build_void_call(
                        "wrapper_bytes32_xor",
                        &[arg0_ptr.into(), arg1_ptr.into(), result_ptr.into()],
                    )?;
                    // bytes32 pointer can return directly
                    return Ok(YulLowLevelValue {
                        value_type: YulLowLevelValueType::Bytes32Pointer,
                        value: result_ptr.into(),
                    });
                }
                let (arg0, arg1) =
                    self.unify_to_u256(self.try_into_int(arg0)?, self.try_into_int(arg1)?)?;
                let result = self
                    .builder
                    .borrow_mut()
                    .build_xor(arg0, arg1, "xor_result")?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::from_int_type(result.get_type()),
                    value: result.into(),
                })
            }
            YulInstructionName::Byte => {
                check_args_count(&instr, &args, 2)?;
                let nth_byte_i32 =
                    self.try_into_i32_value(args.first().unwrap(), args_exprs.first().unwrap())?;
                let value = self.try_into_int(args.get(1).unwrap())?;
                let value = self.int_as_u256(value)?;
                let shift_amount_tmp = self.builder.borrow_mut().build_int_sub(
                    self.i32_type().const_int(31, false),
                    nth_byte_i32,
                    "shift_amount",
                )?;
                let shift_count = self.builder.borrow_mut().build_int_mul(
                    self.i32_type().const_int(8, false),
                    shift_amount_tmp,
                    "shift_amount_mul",
                )?;
                let (value, shift_count) = self.unify_ints(value, shift_count)?;
                let shifted_value = self.builder.borrow_mut().build_right_shift(
                    value,
                    shift_count,
                    false, /* unsigned */
                    "shifted_value",
                )?;
                // get last byte
                let result = self
                    .builder
                    .borrow_mut()
                    .build_and(
                        shifted_value,
                        self.u256_type().const_int(0xff, false),
                        "byte_result",
                    )?
                    .into();
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::U256,
                    value: result,
                })
            }
            YulInstructionName::Shl => {
                check_args_count(&instr, &args, 2)?;
                let shift = args.first().unwrap();
                let value = args.get(1).unwrap();

                // llvm shl not work well for large shift
                // so we use bytes32 version
                if self.is_bytes32_value(value) {
                    // Ensure shift is i32
                    let shift_i32 = self.try_into_i32(shift)?;
                    let value = self.try_into_bytes32_pointer(value)?;
                    let result_ptr = self.fast_alloca(self.bytes32_type(), "shl_result")?;
                    self.build_void_call(
                        "wrapper_bytes32_shl",
                        &[value, shift_i32.into(), result_ptr.into()],
                    )?;
                    // bytes32 pointer can return directly
                    return Ok(YulLowLevelValue {
                        value_type: YulLowLevelValueType::Bytes32Pointer,
                        value: result_ptr.into(),
                    });
                }
                // because llvm shl not work well for shift>=256, so we use c wrapper
                let value = self.try_into_u256(value)?;
                let value_ptr = self.get_value_pointer(value)?;
                let shift = self.try_into_i32(shift)?;
                let ret_ty = self.u256_type();
                let result_ptr = self.fast_alloca(ret_ty, "shl_result")?;
                self.build_void_call(
                    "wrapper_u256_shl",
                    &[value_ptr.into(), shift.into(), result_ptr.into()],
                )?;
                // u256 pointer can't return directly, so we need to load the result
                let result = self.build_load(ret_ty, result_ptr, "")?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::U256,
                    value: result,
                })
            }
            YulInstructionName::Sar => {
                // Signed Arithmetic Right Shift
                check_args_count(&instr, &args, 2)?;
                let (arg0, arg1) = self.unify_to_u256(
                    self.try_into_int(args.first().unwrap())?,
                    self.try_into_int(args.get(1).unwrap())?,
                )?;
                let result = self
                    .builder
                    .borrow_mut()
                    .build_right_shift(
                        arg1,
                        arg0, /* yul sar is arg1 >> arg0 */
                        true, // signed shift
                        "sar_result",
                    )?
                    .into();
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::U256,
                    value: result,
                })
            }
            YulInstructionName::Shr => {
                check_args_count(&instr, &args, 2)?;

                let shift = args.first().unwrap();
                let value = args.get(1).unwrap();

                if self.is_bytes32_value(value) {
                    let value = self.try_into_bytes32_pointer(value)?;
                    // Ensure shift is i32
                    let shift_i32 = self.try_into_i32(shift)?;
                    let result_ptr = self.fast_alloca(self.bytes32_type(), "shr_result")?;
                    self.build_void_call(
                        "wrapper_bytes32_shr",
                        &[value, shift_i32.into(), result_ptr.into()],
                    )?;
                    return Ok(YulLowLevelValue {
                        value_type: YulLowLevelValueType::Bytes32Pointer,
                        value: result_ptr.into(),
                    });
                }
                let value = self.try_into_u256(value)?;
                let value_ptr = self.get_value_pointer(value)?;
                let shift = self.try_into_i32(shift)?;
                let ret_ty = self.u256_type();
                let result_ptr = self.fast_alloca(ret_ty, "shr_result")?;
                self.build_void_call(
                    "wrapper_u256_shr",
                    &[value_ptr.into(), shift.into(), result_ptr.into()],
                )?;
                // u256 pointer can't return directly, so we need to load the result
                let result = self.build_load(ret_ty, result_ptr, "")?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::U256,
                    value: result,
                })
            }
            YulInstructionName::Keccak256 => {
                check_args_count(&instr, &args, 2)?;
                let evm_mem =
                    self.try_into_i32_value(args.first().unwrap(), args_exprs.first().unwrap())?;

                let size =
                    self.try_into_i32_value(args.get(1).unwrap(), args_exprs.get(1).unwrap())?;

                let ret_ty = self.bytes32_type();
                let result_ptr: PointerValue<'a> = self.fast_alloca(ret_ty, "")?;
                self.build_void_call(
                    "wrapper_keccak256",
                    &[evm_mem.into(), size.into(), result_ptr.into()],
                )?;
                // bytes32 pointer can return directly
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::Bytes32Pointer,
                    value: result_ptr.into(),
                })
            }
            YulInstructionName::MCopy => {
                check_args_count(&instr, &args, 3)?;
                let evm_dst =
                    self.try_into_i32_value(args.first().unwrap(), args_exprs.first().unwrap())?;

                let evm_src =
                    self.try_into_i32_value(args.get(1).unwrap(), args_exprs.get(1).unwrap())?;

                let size =
                    self.try_into_i32_value(args.get(2).unwrap(), args_exprs.get(2).unwrap())?;

                self.build_void_call(
                    "wrapper_mcopy",
                    &[evm_dst.into(), evm_src.into(), size.into()],
                )?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I32,
                    value: self.i32_type().const_zero().into(),
                })
            }
            YulInstructionName::CodeCopy => {
                check_args_count(&instr, &args, 3)?;
                // arg0: target memory location (starting address to write to)
                // arg1: offset in the contract code to copy from (in YUL), compiled to direct linear memory address in WASM (including constant area)
                // arg2: number of bytes to copy
                let target_evm_mem_offset =
                    self.try_into_i32_value(args.first().unwrap(), args_exprs.first().unwrap())?;

                let code_ptr =
                    self.try_into_i32_value(args.get(1).unwrap(), args_exprs.get(1).unwrap())?;

                let size =
                    self.try_into_i32_value(args.get(2).unwrap(), args_exprs.get(2).unwrap())?;

                self.build_void_call(
                    "wrapper_codecopy",
                    &[target_evm_mem_offset.into(), code_ptr.into(), size.into()],
                )?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I32,
                    value: self.i32_type().const_zero().into(),
                })
            }

            YulInstructionName::MLoad => {
                check_args_count(&instr, &args, 1)?;

                if let Some(constant_offset) =
                    self.fetch_not_string_literal_constant(args_exprs.first().unwrap())
                {
                    if self.opts.enable_all_optimizers {
                        // If all optimizations are enabled, for mload(64) we can treat it as returning an integer (this value represents the evm memptr in solidity)
                        if constant_offset == U256::from(64) {
                            let result = self.build_call(
                                "wrapper_mload_u32",
                                &[self.i32_type().const_int(64, false).into()],
                            )?;
                            return Ok(YulLowLevelValue {
                                value_type: YulLowLevelValueType::I32,
                                value: result,
                            });
                        }
                    }
                }

                let evm_mem =
                    self.try_into_i32_value(args.first().unwrap(), args_exprs.first().unwrap())?;

                match expected_type {
                    ExpectedType::I32 => {
                        // If i32 is expected, use direct wrapper_mload_u32
                        let result = self.build_call("wrapper_mload_u32", &[evm_mem.into()])?;
                        Ok(YulLowLevelValue {
                            value_type: YulLowLevelValueType::I32,
                            value: result,
                        })
                    }
                    ExpectedType::I64 => {
                        // If i64 is expected, use direct wrapper_mload_u64
                        let result = self.build_call("wrapper_mload_u64", &[evm_mem.into()])?;
                        Ok(YulLowLevelValue {
                            value_type: YulLowLevelValueType::I64,
                            value: result,
                        })
                    }
                    ExpectedType::U256 => {
                        let result_ptr: PointerValue<'a> =
                            self.fast_alloca(self.u256_type(), "")?;
                        self.build_void_call(
                            "wrapper_mload_u256",
                            &[evm_mem.into(), result_ptr.into()],
                        )?;
                        let result = self.build_load(self.u256_type(), result_ptr, "")?;
                        Ok(YulLowLevelValue {
                            value_type: YulLowLevelValueType::U256,
                            value: result,
                        })
                    }
                    _ => {
                        // Default case: load as bytes32
                        // For bytes32, load directly as bytes32
                        let result_ptr: PointerValue<'a> =
                            self.fast_alloca(self.bytes32_type(), "")?;
                        self.build_void_call(
                            "wrapper_mload_bytes32",
                            &[evm_mem.into(), result_ptr.into()],
                        )?;
                        // bytes32 pointer can return directly
                        Ok(YulLowLevelValue {
                            value_type: YulLowLevelValueType::Bytes32Pointer,
                            value: result_ptr.into(),
                        })
                    }
                }
            }
            YulInstructionName::MStore => {
                check_args_count(&instr, &args, 2)?;

                let evm_mem =
                    self.try_into_i32_value(args.first().unwrap(), args_exprs.first().unwrap())?;

                let value = args.get(1).unwrap();

                // Determine the best way to store based on the value type
                if self.is_bytes32_type(&value.get_type()) {
                    // If value is bytes32, use bytes32-specific store
                    let value_ptr = self.get_value_pointer(*value)?;
                    self.build_void_call(
                        "wrapper_mstore_bytes32",
                        &[evm_mem.into(), value_ptr.into()],
                    )?;
                    Ok(YulLowLevelValue {
                        value_type: YulLowLevelValueType::I32,
                        value: self.i32_type().const_zero().into(),
                    })
                } else if value.is_pointer_value() {
                    // is bytes32 pointer
                    self.build_void_call("wrapper_mstore_bytes32", &[evm_mem.into(), *value])?;
                    Ok(YulLowLevelValue {
                        value_type: YulLowLevelValueType::I32,
                        value: self.i32_type().const_zero().into(),
                    })
                } else if value.get_type().into_int_type().get_bit_width() == 32 {
                    // If value is u32, use u32-specific store
                    self.build_void_call("wrapper_mstore_u32", &[evm_mem.into(), *value])?;
                    Ok(YulLowLevelValue {
                        value_type: YulLowLevelValueType::I32,
                        value: self.i32_type().const_zero().into(),
                    })
                } else if value.get_type().into_int_type().get_bit_width() == 64 {
                    // If value is u64, use u64-specific store
                    self.build_void_call("wrapper_mstore_u64", &[evm_mem.into(), *value])?;
                    Ok(YulLowLevelValue {
                        value_type: YulLowLevelValueType::I32,
                        value: self.i32_type().const_zero().into(),
                    })
                } else {
                    // Default: convert to u256 and store
                    let value = self.try_into_u256(value)?;
                    let value_ptr = self.get_value_pointer(value)?;

                    self.build_void_call(
                        "wrapper_mstore_u256",
                        &[evm_mem.into(), value_ptr.into()],
                    )?;
                    Ok(YulLowLevelValue {
                        value_type: YulLowLevelValueType::I32,
                        value: self.i32_type().const_zero().into(),
                    })
                }
            }
            YulInstructionName::MStore8 => {
                check_args_count(&instr, &args, 2)?;
                let evm_mem = self.try_into_i32_across_int(args.first().unwrap())?;

                let value = self.try_into_int(args.get(1).unwrap())?;
                let u8_value = self.int_cast(value, self.i8_type())?;
                self.build_void_call("wrapper_mstore_u8", &[evm_mem.into(), u8_value.into()])?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I32,
                    value: self.i32_type().const_zero().into(),
                })
            }
            YulInstructionName::SLoad => {
                check_args_count(&instr, &args, 1)?;

                let slot =
                    if self.matches_constant_literal(args_exprs.first().unwrap(), U256::zero()) {
                        // For slot 0, we can handle it directly
                        self.build_call("wrapper_zero_bytes32", &[])?
                    } else {
                        let slot: &BasicValueEnum<'a> = args.first().unwrap();
                        self.try_into_bytes32_pointer(slot)?.as_basic_value_enum()
                    };

                if self.opts.enable_storage_load_store_little_endian
                    && (expected_type != ExpectedType::Bytes32
                        && expected_type != ExpectedType::Bytes32Pointer)
                {
                    let ret_ty = self.u256_type();
                    let ret_value = self.fast_alloca(ret_ty, "")?;
                    self.build_void_call(
                        "wrapper_sload_u256_using_little_endian_hostapi",
                        &[slot, ret_value.into()],
                    )?;
                    let result = self.build_load(ret_ty, ret_value, "")?;
                    return Ok(YulLowLevelValue {
                        value_type: YulLowLevelValueType::U256,
                        value: result,
                    });
                }
                let ret_ty = self.bytes32_type();
                let result_ptr: PointerValue<'a> = self.fast_alloca(ret_ty, "")?;
                self.build_void_call("wrapper_sload_bytes32", &[slot, result_ptr.into()])?;
                // bytes32 pointer can return directly
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::Bytes32Pointer,
                    value: result_ptr.into(),
                })
            }
            YulInstructionName::TLoad => {
                check_args_count(&instr, &args, 1)?;
                let slot: &BasicValueEnum<'a> = args.first().unwrap();
                let slot = self.try_into_bytes32_pointer(slot)?;

                let ret_ty = self.bytes32_type();
                let result_ptr: PointerValue<'a> = self.fast_alloca(ret_ty, "")?;
                self.build_void_call("wrapper_tload_bytes32", &[slot, result_ptr.into()])?;
                // bytes32 pointer can return directly
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::Bytes32Pointer,
                    value: result_ptr.into(),
                })
            }
            YulInstructionName::LoadImmutable => {
                assert_eq!(args_exprs.len(), 1);
                let name = self
                    .read_string_literal(&args_exprs[0])
                    .expect("immutable name is empty");
                let key_name = format!("{}_{}", "immutable", name);
                let mut data = key_name.as_bytes().to_vec();
                data.resize(32, 0u8);
                keccak_hash::keccak256(&mut data);
                let hash = &data[0..32];
                let hash_hex = hex::encode(hash);
                let hash_llvm_u256 = self
                    .u256_type()
                    .const_int_from_string(&hash_hex, inkwell::types::StringRadix::Hexadecimal)
                    .unwrap();
                let hash_u256_ptr = self.get_value_pointer(hash_llvm_u256)?;

                let result_ptr: PointerValue<'a> = self.fast_alloca(self.u256_type(), "")?;
                self.build_void_call(
                    "wrapper_loadimmutable",
                    &[hash_u256_ptr.into(), result_ptr.into()],
                )?;
                let result = self.build_load(self.u256_type(), result_ptr, "")?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::U256,
                    value: result,
                })
            }
            YulInstructionName::SStore => {
                check_args_count(&instr, &args, 2)?;
                let slot =
                    if self.matches_constant_literal(args_exprs.first().unwrap(), U256::zero()) {
                        // For slot 0, we can handle it directly
                        self.build_call("wrapper_zero_bytes32", &[])?
                    } else {
                        let slot: &BasicValueEnum<'a> = args.first().unwrap();
                        self.try_into_bytes32_pointer(slot)?.as_basic_value_enum()
                    };

                if self.opts.enable_storage_load_store_little_endian
                    && (expected_type != ExpectedType::Bytes32
                        && expected_type != ExpectedType::Bytes32Pointer)
                {
                    let value: &BasicValueEnum<'a> = args.get(1).unwrap();
                    let value = self.try_into_u256(value)?;
                    let value_ptr = self.get_value_pointer(value)?;
                    self.build_void_call(
                        "wrapper_sstore_u256_using_little_endian_hostapi",
                        &[slot, value_ptr.into()],
                    )?;
                    return Ok(YulLowLevelValue {
                        value_type: YulLowLevelValueType::I32,
                        value: self.i32_type().const_zero().into(),
                    });
                }

                let value: &BasicValueEnum<'a> = args.get(1).unwrap();
                let value = self.try_into_bytes32_pointer(value)?;

                self.build_void_call("wrapper_sstore_bytes32", &[slot, value])?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I32,
                    value: self.i32_type().const_zero().into(),
                })
            }
            YulInstructionName::TStore => {
                check_args_count(&instr, &args, 2)?;
                let slot: &BasicValueEnum<'a> = args.first().unwrap();
                let slot = self.try_into_bytes32_pointer(slot)?;

                let value: &BasicValueEnum<'a> = args.get(1).unwrap();
                let value = self.try_into_bytes32_pointer(value)?;

                self.build_void_call("wrapper_tstore_bytes32", &[slot, value])?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I32,
                    value: self.i32_type().const_zero().into(),
                })
            }
            YulInstructionName::SetImmutable => {
                check_args_count(&instr, &args, 3)?;
                let evm_dst: &BasicValueEnum<'a> = args.first().unwrap();
                let evm_dst = self.try_into_i32(evm_dst)?;

                assert_eq!(args_exprs.len(), 3);
                let name = self
                    .read_string_literal(&args_exprs[1])
                    .expect("immutable name is empty");
                let key_name = format!("{}_{}", "immutable", name);
                let mut data = key_name.as_bytes().to_vec();
                data.resize(32, 0u8);
                keccak_hash::keccak256(&mut data);
                let hash = &data[0..32];
                let hash_hex = hex::encode(hash);
                let hash_llvm_u256 = self
                    .u256_type()
                    .const_int_from_string(&hash_hex, inkwell::types::StringRadix::Hexadecimal)
                    .unwrap();
                let hash_u256_ptr = self.get_value_pointer(hash_llvm_u256)?;

                let value: &BasicValueEnum<'a> = args.get(2).unwrap();
                let value = self.try_into_u256(value)?;
                let value_ptr = self.get_value_pointer(value)?;

                self.build_void_call(
                    "wrapper_setimmutable",
                    &[evm_dst.into(), hash_u256_ptr.into(), value_ptr.into()],
                )?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I32,
                    value: self.i32_type().const_zero().into(),
                })
            }
            YulInstructionName::MSize => {
                let msize_i64 = self
                    .build_call("wrapper_memory_size", &[])?
                    .into_int_value();
                // Return i64 value directly, let the caller handle type conversion as needed
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I64,
                    value: msize_i64.into(),
                })
            }
            YulInstructionName::Gas => {
                let gas_i64 = self.build_call("wrapper_gas", &[])?.into_int_value();
                // Return i64 value directly, let the caller handle type conversion as needed
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I64,
                    value: gas_i64.into(),
                })
            }
            YulInstructionName::GasPrice => {
                let tmp_result = self.fast_alloca(self.u256_type(), "")?;
                self.build_void_call("wrapper_gas_price", &[tmp_result.into()])?;
                let result = self.build_load(self.u256_type(), tmp_result, "")?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::U256,
                    value: result,
                })
            }
            YulInstructionName::GasLimit => {
                let gas_limit_i64 = self.build_call("wrapper_gas_limit", &[])?.into_int_value();
                // Return i64 value directly, let the caller handle type conversion as needed
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I64,
                    value: gas_limit_i64.into(),
                })
            }
            YulInstructionName::Address => {
                let ret_ty = self.bytes32_type();
                let result_ptr: PointerValue<'a> = self.fast_alloca(ret_ty, "")?;
                self.build_void_call("wrapper_current_contract", &[result_ptr.into()])?;
                // bytes32 pointer can return directly
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::Bytes32Pointer,
                    value: result_ptr.into(),
                })
            }
            YulInstructionName::Balance => {
                check_args_count(&instr, &args, 1)?;
                let addr: &BasicValueEnum<'a> = args.first().unwrap();
                let addr = self.try_into_bytes32_pointer(addr)?;

                let result_ptr: PointerValue<'a> = self.fast_alloca(self.u256_type(), "")?;
                self.build_void_call("wrapper_query_balance", &[addr, result_ptr.into()])?;
                let result = self.build_load(self.u256_type(), result_ptr, "")?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::U256,
                    value: result,
                })
            }
            YulInstructionName::SelfBalance => {
                let result_ptr: PointerValue<'a> = self.fast_alloca(self.u256_type(), "")?;
                self.build_void_call("wrapper_self_balance", &[result_ptr.into()])?;
                let result = self.build_load(self.u256_type(), result_ptr, "")?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::U256,
                    value: result,
                })
            }
            YulInstructionName::Caller => {
                let ret_ty = self.bytes32_type();
                let tmp_result = self.fast_alloca(ret_ty, "")?;
                self.build_void_call("wrapper_caller", &[tmp_result.into()])?;
                // bytes32 pointer can return directly
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::Bytes32Pointer,
                    value: tmp_result.into(),
                })
            }
            YulInstructionName::CallValue => {
                // Check if we're expecting a boolean result (for callvalue != 0 check)
                // This is an optimization for the common pattern in require(msg.value == 0)
                if matches!(expected_type, ExpectedType::Bool) {
                    let result = self.build_call("wrapper_callvalue_not_zero", &[])?;
                    return Ok(YulLowLevelValue {
                        value_type: YulLowLevelValueType::I32,
                        value: result,
                    });
                }
                let tmp_result = self.fast_alloca(self.u256_type(), "")?;
                self.build_void_call("wrapper_callvalue", &[tmp_result.into()])?;
                let result = self.build_load(self.u256_type(), tmp_result, "")?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::U256,
                    value: result,
                })
            }
            YulInstructionName::CallDataLoad => {
                check_args_count(&instr, &args, 1)?;
                let offset =
                    self.try_into_i32_value(args.first().unwrap(), args_exprs.first().unwrap())?;

                let ret_ty = self.bytes32_type();
                let tmp_result = self.fast_alloca(ret_ty, "")?;
                self.build_void_call(
                    "wrapper_calldataload_bytes32",
                    &[offset.into(), tmp_result.into()],
                )?;
                // bytes32 pointer can return directly
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::Bytes32Pointer,
                    value: tmp_result.into(),
                })
            }
            YulInstructionName::MemoryGuard => {
                check_args_count(&instr, &args, 1)?;
                let guard_size =
                    self.try_into_i32_value(args.first().unwrap(), args_exprs.first().unwrap())?;

                let result = self.build_call("wrapper_memory_guard", &[guard_size.into()])?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I32,
                    value: result,
                })
            }
            YulInstructionName::CallDataSize => {
                let result = self.build_call("wrapper_calldata_size", &[])?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I32,
                    value: result,
                })
            }
            YulInstructionName::CallDataCopy => {
                check_args_count(&instr, &args, 3)?;
                let dst_evm =
                    self.try_into_i32_value(args.first().unwrap(), args_exprs.first().unwrap())?;

                let calldata_offset =
                    self.try_into_i32_value(args.get(1).unwrap(), args_exprs.get(1).unwrap())?;

                let len =
                    self.try_into_i32_value(args.get(2).unwrap(), args_exprs.get(2).unwrap())?;

                self.build_void_call(
                    "wrapper_calldata_copy",
                    &[dst_evm.into(), calldata_offset.into(), len.into()],
                )?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I32,
                    value: self.i32_type().const_zero().into(),
                })
            }
            YulInstructionName::DataOffset => {
                check_args_count(&instr, &args, 1)?;
                let current_module_name = &*self.current_module_name.borrow();
                // The argument is a constant string that can be obtained at compile time
                assert_eq!(args_exprs.len(), 1);
                // Could be either a yul object name or data segment name
                let data_segment_name = self.read_string_literal(&args_exprs[0]);
                assert!(data_segment_name.is_some());
                let data_segment_name = data_segment_name.unwrap();
                if current_module_name == &data_segment_name {
                    // Get the bytecode offset of the current contract. Since the current bytecode cannot appear in the current contract's data segment, this case is impossible
                    unreachable!(
                        "dataoffset param can't be the same object name as the current object"
                    );
                }
                // Otherwise get the starting address of the wasm bytecode constants generated by the child object/data segment

                let global_var_name = format!(
                    "{}.{data_segment_name}",
                    *self.current_contract_name.borrow()
                );
                let global_var = self
                    .llvm_module
                    .borrow_mut()
                    .get_global(&global_var_name)
                    .unwrap();
                // Convert global_var_addr from PointerValue to IntValue
                let global_var_addr = self.builder.borrow_mut().build_ptr_to_int(
                    global_var.as_pointer_value(),
                    self.i32_type(),
                    "global_var_addr",
                )?;
                // The first 4 bytes contain the length in big endian format, but we don't skip them
                let data_offset = global_var_addr;

                let memory_begin = self.build_call(
                    "evm_get_memory_addr",
                    &[self.i32_type().const_int(0, false).into()],
                )?;
                let memory_begin = self.builder.borrow_mut().build_ptr_to_int(
                    memory_begin.into_pointer_value(),
                    self.i32_type(),
                    "",
                )?;
                let result = self.builder.borrow_mut().build_int_sub(
                    data_offset,
                    memory_begin,
                    "data_offset_result",
                )?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I32,
                    value: result.into(),
                })
            }
            YulInstructionName::DataSize => {
                check_args_count(&instr, &args, 1)?;
                // If datasize(name) target name is the current object, get the pure contract size directly (excluding calldata)
                let current_module_name = &*self.current_module_name.borrow();
                // The argument is a constant string that can be obtained at compile time
                assert_eq!(args_exprs.len(), 1);
                // Could be either a yul object name or a data segment name
                let data_segment_name = self.read_string_literal(&args_exprs[0]);
                assert!(data_segment_name.is_some());
                let data_segment_name = data_segment_name.unwrap();
                if current_module_name == &data_segment_name {
                    // Get the actual bytecode length of the current contract (excluding calldata), different from codesize, hence calling a different C function
                    let cur_contract_bytecode_len_i32 = self
                        .build_call("wrapper_current_contract_pure_contract_size", &[])?
                        .into_int_value();
                    Ok(YulLowLevelValue {
                        value_type: YulLowLevelValueType::I32,
                        value: cur_contract_bytecode_len_i32.into(),
                    })
                } else {
                    // Otherwise get the length of the wasm bytecode generated by the data segment or child object
                    let global_qualified_name =
                        &format!("{}.{}", current_module_name, data_segment_name);
                    // let data_global_value = self.llvm_module.borrow_mut().get_global(global_qualified_name).unwrap();
                    let data_len = *self
                        .global_bytes_lengths
                        .borrow()
                        .get(global_qualified_name)
                        .unwrap();
                    Ok(YulLowLevelValue {
                        value_type: YulLowLevelValueType::I32,
                        value: self.i32_type().const_int(data_len as u64, true).into(),
                    })
                }
            }
            YulInstructionName::ReturnDataSize => {
                let size_i32 = self
                    .build_call("wrapper_returndata_size", &[])?
                    .into_int_value();
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I32,
                    value: size_i32.into(),
                })
            }
            YulInstructionName::ReturnDataCopy => {
                check_args_count(&instr, &args, 3)?;
                let dst_evm =
                    self.try_into_i32_value(args.first().unwrap(), args_exprs.first().unwrap())?;

                let return_data_offset =
                    self.try_into_i32_value(args.get(1).unwrap(), args_exprs.get(1).unwrap())?;

                let len =
                    self.try_into_i32_value(args.get(2).unwrap(), args_exprs.get(2).unwrap())?;

                self.build_void_call(
                    "wrapper_returndata_copy",
                    &[dst_evm.into(), return_data_offset.into(), len.into()],
                )?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I32,
                    value: self.i32_type().const_zero().into(),
                })
            }
            YulInstructionName::Return => {
                check_args_count(&instr, &args, 2)?;
                let src_evm_mem =
                    self.try_into_i32_value(args.first().unwrap(), args_exprs.first().unwrap())?;

                let size =
                    self.try_into_i32_value(args.get(1).unwrap(), args_exprs.get(1).unwrap())?;

                self.build_void_call("wrapper_return", &[src_evm_mem.into(), size.into()])?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I32,
                    value: self.i32_type().const_zero().into(),
                })
            }
            YulInstructionName::CodeSize => {
                let size_i32 = self
                    .build_call("wrapper_current_contract_code_size", &[])?
                    .into_int_value();
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I32,
                    value: size_i32.into(),
                })
            }
            YulInstructionName::ExtCodeSize => {
                // extcodesize(uint256 address)
                check_args_count(&instr, &args, 1)?;
                let addr: &BasicValueEnum<'a> = args.first().unwrap();
                let addr = self.try_into_bytes32_pointer(addr)?;

                let size_i32 = self
                    .build_call("wrapper_extcode_size", &[addr])?
                    .into_int_value();
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I32,
                    value: size_i32.into(),
                })
            }
            YulInstructionName::ExtCodeCopy => {
                // arg0 address: contract address to copy code from
                // arg1 dst: target memory offset in bytes, indicating where to copy to
                // arg2 offset: source code offset in bytes, indicating where to start copying from
                // arg3 size: number of bytes to copy
                check_args_count(&instr, &args, 4)?;
                let addr: &BasicValueEnum<'a> = args.first().unwrap();
                let addr = self.try_into_bytes32_pointer(addr)?;

                let dst_evm =
                    self.try_into_i32_value(args.get(1).unwrap(), args_exprs.get(1).unwrap())?;

                let src_offset =
                    self.try_into_i32_value(args.get(2).unwrap(), args_exprs.get(2).unwrap())?;

                let len =
                    self.try_into_i32_value(args.get(3).unwrap(), args_exprs.get(3).unwrap())?;

                self.build_void_call(
                    "wrapper_extcode_copy",
                    &[addr, dst_evm.into(), src_offset.into(), len.into()],
                )?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I32,
                    value: self.i32_type().const_zero().into(),
                })
            }
            YulInstructionName::DataCopy => {
                check_args_count(&instr, &args, 3)?;
                let dst_evm =
                    self.try_into_i32_value(args.first().unwrap(), args_exprs.first().unwrap())?;

                let src_evm =
                    self.try_into_i32_value(args.get(1).unwrap(), args_exprs.get(1).unwrap())?;

                let len =
                    self.try_into_i32_value(args.get(2).unwrap(), args_exprs.get(2).unwrap())?;

                self.build_void_call(
                    "wrapper_data_copy",
                    &[dst_evm.into(), src_evm.into(), len.into()],
                )?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I32,
                    value: self.i32_type().const_zero().into(),
                })
            }
            YulInstructionName::ExtCodeHash => {
                check_args_count(&instr, &args, 1)?;
                let addr: &BasicValueEnum<'a> = args.first().unwrap();
                let addr = self.try_into_bytes32_pointer(addr)?;

                let ret_ty = self.bytes32_type();
                let result_ptr: PointerValue<'a> = self.fast_alloca(ret_ty, "")?;
                self.build_void_call("wrapper_extcode_hash", &[addr, result_ptr.into()])?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::Bytes32Pointer,
                    value: result_ptr.into(),
                })
            }
            YulInstructionName::DebugPrint => {
                check_args_count(&instr, &args, 1)?;
                let value: &BasicValueEnum<'a> = args.first().unwrap();

                if self.is_bytes32_pointer_value(value) {
                    // when input is bytes32*
                    self.build_void_call("wrapper_debug_bytes32", &[*value])?;
                    return Ok(YulLowLevelValue {
                        value_type: YulLowLevelValueType::I32,
                        value: self.i32_type().const_zero().into(),
                    });
                }

                let value = self.try_into_u256(value)?;
                let value_ptr = self.get_value_pointer(value)?;

                self.build_void_call("wrapper_debug_i256", &[value_ptr.into()])?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I32,
                    value: self.i32_type().const_zero().into(),
                })
            }
            YulInstructionName::Revert => {
                // arg0: Starting position of the error message in memory (pointer)
                // arg1: Length of the error message (in bytes)
                check_args_count(&instr, &args, 2)?;
                let error_msg_evm_mem =
                    self.try_into_i32_value(args.first().unwrap(), args_exprs.first().unwrap())?;

                let size =
                    self.try_into_i32_value(args.get(1).unwrap(), args_exprs.get(1).unwrap())?;

                self.build_void_call("wrapper_revert", &[error_msg_evm_mem.into(), size.into()])?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I32,
                    value: self.i32_type().const_zero().into(),
                })
            }
            YulInstructionName::Log0 => {
                check_args_count(&instr, &args, 2)?;
                let data_evm_mem =
                    self.try_into_i32_value(args.first().unwrap(), args_exprs.first().unwrap())?;

                let data_size =
                    self.try_into_i32_value(args.get(1).unwrap(), args_exprs.get(1).unwrap())?;

                self.build_void_call("wrapper_log0", &[data_evm_mem.into(), data_size.into()])?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I32,
                    value: self.i32_type().const_zero().into(),
                })
            }
            YulInstructionName::Log1 => {
                check_args_count(&instr, &args, 3)?;
                let data_evm_mem =
                    self.try_into_i32_value(args.first().unwrap(), args_exprs.first().unwrap())?;

                let data_size =
                    self.try_into_i32_value(args.get(1).unwrap(), args_exprs.get(1).unwrap())?;

                let topic0: &BasicValueEnum<'a> = args.get(2).unwrap();
                let topic0_ptr = self.try_into_bytes32_pointer(topic0)?;

                self.build_void_call(
                    "wrapper_log1",
                    &[data_evm_mem.into(), data_size.into(), topic0_ptr],
                )?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I32,
                    value: self.i32_type().const_zero().into(),
                })
            }
            YulInstructionName::Log2 => {
                check_args_count(&instr, &args, 4)?;
                let data_evm_mem =
                    self.try_into_i32_value(args.first().unwrap(), args_exprs.first().unwrap())?;

                let data_size =
                    self.try_into_i32_value(args.get(1).unwrap(), args_exprs.get(1).unwrap())?;

                let topic0: &BasicValueEnum<'a> = args.get(2).unwrap();
                let topic0_ptr = self.try_into_bytes32_pointer(topic0)?;

                let topic1: &BasicValueEnum<'a> = args.get(3).unwrap();
                let topic1_ptr = self.try_into_bytes32_pointer(topic1)?;

                self.build_void_call(
                    "wrapper_log2",
                    &[
                        data_evm_mem.into(),
                        data_size.into(),
                        topic0_ptr,
                        topic1_ptr,
                    ],
                )?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I32,
                    value: self.i32_type().const_zero().into(),
                })
            }
            YulInstructionName::Log3 => {
                check_args_count(&instr, &args, 5)?;
                let data_evm_mem =
                    self.try_into_i32_value(args.first().unwrap(), args_exprs.first().unwrap())?;

                let data_size =
                    self.try_into_i32_value(args.get(1).unwrap(), args_exprs.get(1).unwrap())?;

                let topic0: &BasicValueEnum<'a> = args.get(2).unwrap();
                let topic0_ptr = self.try_into_bytes32_pointer(topic0)?;

                let topic1: &BasicValueEnum<'a> = args.get(3).unwrap();
                let topic1_ptr = self.try_into_bytes32_pointer(topic1)?;

                let topic2: &BasicValueEnum<'a> = args.get(4).unwrap();
                let topic2_ptr = self.try_into_bytes32_pointer(topic2)?;

                self.build_void_call(
                    "wrapper_log3",
                    &[
                        data_evm_mem.into(),
                        data_size.into(),
                        topic0_ptr,
                        topic1_ptr,
                        topic2_ptr,
                    ],
                )?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I32,
                    value: self.i32_type().const_zero().into(),
                })
            }
            YulInstructionName::Log4 => {
                check_args_count(&instr, &args, 6)?;
                let data_evm_mem =
                    self.try_into_i32_value(args.first().unwrap(), args_exprs.first().unwrap())?;

                let data_size =
                    self.try_into_i32_value(args.get(1).unwrap(), args_exprs.get(1).unwrap())?;

                let topic0: &BasicValueEnum<'a> = args.get(2).unwrap();
                let topic0_ptr = self.try_into_bytes32_pointer(topic0)?;

                let topic1: &BasicValueEnum<'a> = args.get(3).unwrap();
                let topic1_ptr = self.try_into_bytes32_pointer(topic1)?;

                let topic2: &BasicValueEnum<'a> = args.get(4).unwrap();
                let topic2_ptr = self.try_into_bytes32_pointer(topic2)?;

                let topic3: &BasicValueEnum<'a> = args.get(5).unwrap();
                let topic3_ptr = self.try_into_bytes32_pointer(topic3)?;

                self.build_void_call(
                    "wrapper_log4",
                    &[
                        data_evm_mem.into(),
                        data_size.into(),
                        topic0_ptr,
                        topic1_ptr,
                        topic2_ptr,
                        topic3_ptr,
                    ],
                )?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I32,
                    value: self.i32_type().const_zero().into(),
                })
            }

            YulInstructionName::Pop => {
                // pop removes a value from the EVM stack, but when compiling to LLVM IR, no operation is needed
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I32,
                    value: self.i32_type().const_zero().into(),
                })
            }
            YulInstructionName::Create => {
                check_args_count(&instr, &args, 3)?;
                let value: &BasicValueEnum<'a> = args.first().unwrap();
                let value = self.try_into_u256(value)?;
                let value_ptr = self.get_value_pointer(value)?;

                let code_evm_mem =
                    self.try_into_i32_value(args.get(1).unwrap(), args_exprs.get(1).unwrap())?;

                let code_length =
                    self.try_into_i32_value(args.get(2).unwrap(), args_exprs.get(2).unwrap())?;

                let ret_ty = self.bytes32_type();
                let result_ptr: PointerValue<'a> = self.fast_alloca(ret_ty, "")?;
                self.build_void_call(
                    "wrapper_create",
                    &[
                        value_ptr.into(),
                        code_evm_mem.into(),
                        code_length.into(),
                        result_ptr.into(),
                    ],
                )?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::Bytes32Pointer,
                    value: result_ptr.into(),
                })
            }
            YulInstructionName::Create2 => {
                check_args_count(&instr, &args, 4)?;
                let value: &BasicValueEnum<'a> = args.first().unwrap();
                let value = self.try_into_u256(value)?;
                let value_ptr = self.get_value_pointer(value)?;

                let code_evm_mem =
                    self.try_into_i32_value(args.get(1).unwrap(), args_exprs.get(1).unwrap())?;

                let code_length =
                    self.try_into_i32_value(args.get(2).unwrap(), args_exprs.get(2).unwrap())?;

                let salt: &BasicValueEnum<'a> = args.get(3).unwrap();
                let salt = self.try_into_u256(salt)?;
                let salt_ptr = self.get_value_pointer(salt)?;

                let ret_ty = self.bytes32_type();
                let result_ptr: PointerValue<'a> = self.fast_alloca(ret_ty, "")?;
                self.build_void_call(
                    "wrapper_create2",
                    &[
                        value_ptr.into(),
                        code_evm_mem.into(),
                        code_length.into(),
                        salt_ptr.into(),
                        result_ptr.into(),
                    ],
                )?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::Bytes32Pointer,
                    value: result_ptr.into(),
                })
            }
            YulInstructionName::Call => {
                check_args_count(&instr, &args, 7)?;
                let gas: &BasicValueEnum<'a> = args.first().unwrap();
                let gas = self.try_into_i64(gas)?;

                let callee_addr: &BasicValueEnum<'a> = args.get(1).unwrap();
                let callee_addr = self.try_into_bytes32_pointer(callee_addr)?;

                let value: &BasicValueEnum<'a> = args.get(2).unwrap();
                let value = self.try_into_u256(value)?;
                let value_ptr = self.get_value_pointer(value)?;

                let in_offset =
                    self.try_into_i32_value(args.get(3).unwrap(), args_exprs.get(3).unwrap())?;
                let in_length =
                    self.try_into_i32_value(args.get(4).unwrap(), args_exprs.get(4).unwrap())?;

                let out_evm_offset =
                    self.try_into_i32_value(args.get(5).unwrap(), args_exprs.get(5).unwrap())?;
                let out_length =
                    self.try_into_i32_value(args.get(6).unwrap(), args_exprs.get(6).unwrap())?;

                let result = self.build_call(
                    "wrapper_call_contract",
                    &[
                        gas.into(),
                        callee_addr,
                        value_ptr.into(),
                        in_offset.into(),
                        in_length.into(),
                        out_evm_offset.into(),
                        out_length.into(),
                    ],
                )?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I32,
                    value: result,
                })
            }
            YulInstructionName::CallCode => {
                unreachable!("callcode not supported, use delegatecall instead")
            }
            YulInstructionName::DelegateCall => {
                check_args_count(&instr, &args, 6)?;
                let gas: &BasicValueEnum<'a> = args.first().unwrap();
                let gas = self.try_into_i64(gas)?;

                let callee_addr: &BasicValueEnum<'a> = args.get(1).unwrap();
                let callee_addr = self.try_into_bytes32_pointer(callee_addr)?;

                let in_offset =
                    self.try_into_i32_value(args.get(2).unwrap(), args_exprs.get(2).unwrap())?;
                let in_length =
                    self.try_into_i32_value(args.get(3).unwrap(), args_exprs.get(3).unwrap())?;

                let out_evm_offset =
                    self.try_into_i32_value(args.get(4).unwrap(), args_exprs.get(4).unwrap())?;
                let out_length =
                    self.try_into_i32_value(args.get(5).unwrap(), args_exprs.get(5).unwrap())?;

                let result = self.build_call(
                    "wrapper_delegatecall",
                    &[
                        gas.into(),
                        callee_addr,
                        in_offset.into(),
                        in_length.into(),
                        out_evm_offset.into(),
                        out_length.into(),
                    ],
                )?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I32,
                    value: result,
                })
            }
            YulInstructionName::StaticCall => {
                check_args_count(&instr, &args, 6)?;
                let gas: &BasicValueEnum<'a> = args.first().unwrap();
                let gas = self.try_into_i64(gas)?;

                let callee_addr: &BasicValueEnum<'a> = args.get(1).unwrap();
                let callee_addr = self.try_into_bytes32_pointer(callee_addr)?;

                let in_offset =
                    self.try_into_i32_value(args.get(2).unwrap(), args_exprs.get(2).unwrap())?;
                let in_length =
                    self.try_into_i32_value(args.get(3).unwrap(), args_exprs.get(3).unwrap())?;

                let out_evm_offset =
                    self.try_into_i32_value(args.get(4).unwrap(), args_exprs.get(4).unwrap())?;
                let out_length =
                    self.try_into_i32_value(args.get(5).unwrap(), args_exprs.get(5).unwrap())?;

                let result = self.build_call(
                    "wrapper_staticcall",
                    &[
                        gas.into(),
                        callee_addr,
                        in_offset.into(),
                        in_length.into(),
                        out_evm_offset.into(),
                        out_length.into(),
                    ],
                )?;

                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I32,
                    value: result,
                })
            }
            YulInstructionName::SelfDestruct => {
                check_args_count(&instr, &args, 1)?;
                let addr: &BasicValueEnum<'a> = args.first().unwrap();
                let addr = self.try_into_bytes32_pointer(addr)?;
                self.build_void_call("wrapper_selfdestruct", &[addr])?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I32,
                    value: self.i32_type().const_zero().into(),
                })
            }
            YulInstructionName::Invalid => {
                self.build_void_call("wrapper_invalid", &args)?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I32,
                    value: self.i32_type().const_zero().into(),
                })
            }
            YulInstructionName::ChainID => {
                let chain_id_ptr: PointerValue<'a> = self.fast_alloca(self.u256_type(), "")?;
                self.build_void_call("wrapper_current_chainid", &[chain_id_ptr.into()])?;
                let chain_id: BasicValueEnum<'a> =
                    self.build_load(self.u256_type(), chain_id_ptr, "")?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::U256,
                    value: chain_id,
                })
            }
            YulInstructionName::BaseFee => {
                let base_fee_ptr: PointerValue<'a> = self.fast_alloca(self.u256_type(), "")?;
                self.build_void_call("wrapper_current_base_fee", &[base_fee_ptr.into()])?;
                let base_fee: BasicValueEnum<'a> =
                    self.build_load(self.u256_type(), base_fee_ptr, "")?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::U256,
                    value: base_fee,
                })
            }
            YulInstructionName::BlobBaseFee => {
                let base_fee_ptr: PointerValue<'a> = self.fast_alloca(self.u256_type(), "")?;
                self.build_void_call("wrapper_current_blob_base_fee", &[base_fee_ptr.into()])?;
                let base_fee: BasicValueEnum<'a> =
                    self.build_load(self.u256_type(), base_fee_ptr, "")?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::U256,
                    value: base_fee,
                })
            }
            YulInstructionName::Origin => {
                let ret_ty = self.bytes32_type();
                let result_ptr: PointerValue<'a> = self.fast_alloca(ret_ty, "")?;
                self.build_void_call("wrapper_origin", &[result_ptr.into()])?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::Bytes32Pointer,
                    value: result_ptr.into(),
                })
            }
            YulInstructionName::BlockHash => {
                check_args_count(&instr, &args, 1)?;
                let block_number: &BasicValueEnum<'a> = args.first().unwrap();
                let block_number = self.try_into_i64(block_number)?;

                let ret_ty = self.bytes32_type();
                let block_hash_ptr: PointerValue<'a> = self.fast_alloca(ret_ty, "")?;
                self.build_void_call(
                    "wrapper_block_hash",
                    &[block_number.into(), block_hash_ptr.into()],
                )?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::Bytes32Pointer,
                    value: block_hash_ptr.into(),
                })
            }
            YulInstructionName::BlobHash => {
                unreachable!("blobhash not supported in this chain")
            }
            YulInstructionName::CoinBase => {
                let ret_ty = self.bytes32_type();
                let result_ptr: PointerValue<'a> = self.fast_alloca(ret_ty, "")?;
                self.build_void_call("wrapper_block_coin_base", &[result_ptr.into()])?;
                // bytes32 pointer can return directly
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::Bytes32Pointer,
                    value: result_ptr.into(),
                })
            }
            YulInstructionName::TimeStamp => {
                let time_stamp_i64 = self.build_call("wrapper_time_stamp", &[])?.into_int_value();
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I64,
                    value: time_stamp_i64.into(),
                })
            }
            YulInstructionName::Number => {
                let block_number_i64 = self
                    .build_call("wrapper_block_number", &[])?
                    .into_int_value();
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I64,
                    value: block_number_i64.into(),
                })
            }
            YulInstructionName::Difficulty => {
                let ret_ty = self.bytes32_type();
                let result_ptr: PointerValue<'a> = self.fast_alloca(ret_ty, "")?;
                self.build_void_call("wrapper_block_prevRandao", &[result_ptr.into()])?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::Bytes32Pointer,
                    value: result_ptr.into(),
                })
            }
            YulInstructionName::Prevrandao => {
                let ret_ty = self.bytes32_type();
                let result_ptr: PointerValue<'a> = self.fast_alloca(ret_ty, "")?;
                self.build_void_call("wrapper_block_prevRandao", &[result_ptr.into()])?;
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::Bytes32Pointer,
                    value: result_ptr.into(),
                })
            }
            YulInstructionName::LinkerSymbol => {
                check_args_count(&instr, &args, 1)?;
                assert_eq!(args_exprs.len(), 1);
                let linkersymbol_name = self.read_string_literal(&args_exprs[0]);
                assert!(linkersymbol_name.is_some());
                let linkersymbol_name = linkersymbol_name.unwrap();
                let mut symbol_addr = self.opts.symbol2addr.get(&linkersymbol_name).cloned();
                if self.opts.ignore_unknown_linker_library && symbol_addr.is_none() {
                    println!("Warning: linker symbol {} not found, use address 0xffffffffffffffffffffffffffffffffffffffff", linkersymbol_name);
                    symbol_addr = Some("0xffffffffffffffffffffffffffffffffffffffff".to_string());
                }
                assert!(symbol_addr.is_some());
                let symbol_addr = symbol_addr.unwrap();
                let symbol_addr = self.hex_literal(&symbol_addr);
                let ret_value_ty = YulLowLevelValueType::from_int_type(symbol_addr.get_type());
                Ok(YulLowLevelValue {
                    value_type: ret_value_ty,
                    value: symbol_addr.into(),
                })
            }
        }
    }
}
