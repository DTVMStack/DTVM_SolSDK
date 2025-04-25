// Copyright (C) 2024-2025 the DTVM authors, Ltd. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use inkwell::types::BasicTypeEnum;

use crate::yul2ir::context::Yul2IRContext;

/// Represents the expected type for a YUL instruction result
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ExpectedType {
    /// No specific type expected, use the default for the instruction
    #[default]
    Untyped,
    /// Expecting a bool result
    Bool,
    /// Expecting an i32 result
    I32,
    /// Expecting an i64 result
    I64,
    /// Expecting a u256 result
    U256,
    /// Expecting bytes32 result
    Bytes32,
    /// Expecting bytes32* result
    Bytes32Pointer,
}

impl<'a> Yul2IRContext<'a> {
    /// Convert a LLVM BasicTypeEnum to our ExpectedType enum
    pub(crate) fn type_to_expected(&self, ty: BasicTypeEnum<'a>) -> ExpectedType {
        if ty.is_int_type() {
            let int_ty = ty.into_int_type();
            match int_ty.get_bit_width() {
                1 => ExpectedType::Bool,
                32 => ExpectedType::I32,
                64 => ExpectedType::I64,
                256 => ExpectedType::U256,
                _ => ExpectedType::Untyped,
            }
        } else if self.is_bytes32_type(&ty) {
            ExpectedType::Bytes32
        } else {
            ExpectedType::Untyped
        }
    }
}
