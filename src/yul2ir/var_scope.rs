// Copyright (c) The Ant Group Core Contributors
// SPDX-License-Identifier: Apache-2.0
// A struct that represents the scope of a variable.
// It contains a HashMap that maps variable names to their corresponding PointerValue.

use crate::yul2ir::context::Yul2IRContext;
use inkwell::types::BasicTypeEnum;
use inkwell::values::PointerValue;
use std::cell::RefCell;
use std::collections::HashMap;

use super::yul_instruction::YulLowLevelValueType;

#[derive(Debug)]
pub struct VarScope<'ctx> {
    pub vars: RefCell<
        HashMap<
            String,
            (
                BasicTypeEnum<'ctx>,
                YulLowLevelValueType,
                PointerValue<'ctx>,
                bool, // is return var
            ),
        >,
    >,
}

impl<'ctx> VarScope<'ctx> {
    pub fn new() -> VarScope<'ctx> {
        VarScope {
            vars: RefCell::new(Default::default()),
        }
    }
}

pub struct ScopeGuard<'a, 'b> {
    context: &'b Yul2IRContext<'a>,
}

impl<'a, 'b> ScopeGuard<'a, 'b> {
    pub fn new(context: &'b Yul2IRContext<'a>) -> Self {
        context.enter_scope();
        ScopeGuard { context }
    }
}

impl<'a, 'b> Drop for ScopeGuard<'a, 'b> {
    fn drop(&mut self) {
        self.context.exit_scope();
    }
}
