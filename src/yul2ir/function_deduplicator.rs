// Copyright (C) 2024-2025 the DTVM authors. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use crate::yul2ir::ast::FunctionDefinition;
use crate::yul2ir::transform::UNIFIED_REVERT_ERROR_ZERO;
use crate::yul2ir::{context::CompileResult, context::Yul2IRContext};

use ethereum_types::U256;
use inkwell::module::Linkage;
use std::rc::Rc;

impl<'a> Yul2IRContext<'a> {
    pub fn is_revert_zero_function(&self, function: &FunctionDefinition) -> bool {
        if function.body.statements.len() != 1 {
            return false;
        }

        let stmt = &function.body.statements[0];
        let revert_params = self.matches_function_call(stmt, "revert", 2);
        if let Some(params) = revert_params {
            return self.matches_constant_literal(&params[0], U256::from(0))
                && self.matches_constant_literal(&params[1], U256::from(0));
        }

        false
    }

    pub fn generate_unified_revert_zero(&self) -> CompileResult<'a> {
        if self
            .functions_mapping
            .borrow()
            .contains_key(UNIFIED_REVERT_ERROR_ZERO)
        {
            return self.ok_result();
        }

        let func_ty = self.llvm_context.void_type().fn_type(&[], false);
        let function = self.llvm_module.borrow_mut().add_function(
            UNIFIED_REVERT_ERROR_ZERO,
            func_ty,
            Some(Linkage::External),
        );
        let entry_bb = self.llvm_context.append_basic_block(function, "entry");
        self.builder.borrow_mut().position_at_end(entry_bb);
        self.build_void_call(
            "wrapper_revert",
            &[
                self.i32_type().const_zero().into(),
                self.i32_type().const_zero().into(),
            ],
        )?;
        self.builder.borrow_mut().build_return(None)?;

        self.functions.borrow_mut().push(Rc::new(function));
        self.functions_mapping
            .borrow_mut()
            .insert(UNIFIED_REVERT_ERROR_ZERO.to_string(), Rc::new(function));

        self.ok_result()
    }
}
