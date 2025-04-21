// Copyright (c) The Ant Group Core Contributors
// Copyright (c) The Smart Intermediate Representation Contributors
// SPDX-License-Identifier: Apache-2.0

use std::rc::Rc;

use crate::yul2ir::context::CompileFunctionResult;
use crate::yul2ir::utils::merge_sub_wasm_with_length_prefix;
use crate::yul2ir::var_scope::ScopeGuard;
use crate::{
    yul2ir::ast::{self, Block, Expression, FunctionDefinition, InnerSegment, Object, Statement},
    yul2ir::context::{CompileResult, Yul2IRContext},
    yul2ir::errors::ASTLoweringError,
    yul2ir::infer::ExpectedType,
    yul2ir::yul_instruction::parse_intrinsic_func_name,
};
use ast::FunctionDeclaration;
use indexmap::IndexMap;
use inkwell::basic_block::BasicBlock;
use inkwell::context::Context;
use inkwell::module::Linkage;
use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, FunctionType};
use inkwell::values::{BasicValue, BasicValueEnum, FunctionValue, IntValue, PointerValue};

use super::yul_instruction::{YulLowLevelFunctionType, YulLowLevelValue, YulLowLevelValueType};

/// Checks if an object contains any sub-contracts.
/// A sub-contract is defined as any nested object that is not a deployed contract
/// (i.e., its name doesn't end with "_deployed").
/// This method is used to determine compilation paths and contract structure.
pub fn has_sub_contract(object: &Object) -> bool {
    // Check if there are any nested objects that are not deployed objects
    for inner_segment in object.inner_segments.iter() {
        if let InnerSegment::Object(inner_object) = inner_segment {
            // If the object name doesn't end with "_deployed", it's considered a sub-contract
            if !inner_object.name.ends_with("_deployed") {
                return true;
            }
            // Recursively check nested objects
            if has_sub_contract(inner_object) {
                return true;
            }
        }
    }
    false
}

#[allow(unused)]
impl<'a> Yul2IRContext<'a> {
    pub(crate) fn walk_block(&self, yul_func_name: &str, block: &Block) -> CompileResult<'a> {
        for stmt in &block.statements {
            self.walk_stmt(yul_func_name, stmt);
        }
        self.ok_result()
    }

    pub(crate) fn walk_stmt(&self, yul_func_name: &str, stmt: &Statement) -> CompileResult<'a> {
        match stmt {
            Statement::Assignment(assign) => self.walk_assignment(yul_func_name, assign),
            Statement::VariableDeclaration(var_decl) => {
                self.walk_variable_declaration(yul_func_name, var_decl)
            }
            Statement::If(r#if) => self.walk_if(yul_func_name, r#if),
            Statement::For(r#for) => self.walk_for(yul_func_name, r#for),
            Statement::Switch(switch) => self.walk_switch(yul_func_name, switch),
            Statement::Leave => self.walk_leave(),
            Statement::Break => self.walk_break(),
            Statement::Continue => self.walk_continue(),
            Statement::Block(block) => self.walk_block(yul_func_name, block),
            Statement::FunctionDefinition(func_def) => self.walk_function_definition(func_def),
            Statement::FunctionCall(func_call) => {
                let val = self.walk_function_call(yul_func_name, func_call)?;
                self.ok_result()
            }
            Statement::Comment => self.ok_result(),
        }
    }

    pub(crate) fn walk_function_definition(
        &self,
        func_def: &FunctionDefinition,
    ) -> CompileResult<'a> {
        // The LLVM function signature is added during the scan phase,
        // and each function is compiled at the end of walk_object,
        // so nothing needs to be done here.
        self.ok_result()
    }

    fn walk_switch(&self, yul_func_name: &str, switch: &ast::Switch) -> CompileResult<'a> {
        let cur_func = self.current_function.borrow().clone().unwrap();
        let cur_func_value = *cur_func.clone();

        // For switch condition, we ideally want the type to match case values (typically u256)
        let cond_value =
            self.walk_expr_with_type(yul_func_name, &switch.condition, ExpectedType::I32)?;
        let cond_value = cond_value.get_value();
        let cond_value = if cond_value.is_int_value() {
            cond_value.into_int_value()
        } else {
            self.try_into_u256(&cond_value)?
        };
        let cond_value_ty = cond_value.get_type();

        // Determine the expected type for case values based on the condition value type
        let expected_type = self.type_to_expected(cond_value_ty.as_basic_type_enum());

        let end_block = self
            .llvm_context
            .append_basic_block(cur_func_value, "switch_end");
        match &switch.opt {
            ast::SwitchOptions::Cases(cases, default) => {
                let mut jump_tbl: IndexMap<IntValue<'a>, (BasicBlock, Block)> = IndexMap::default();
                let mut jump_blocks: Vec<(IntValue<'a>, BasicBlock<'a>)> = vec![];
                for case in cases {
                    let case_int = match &case.case {
                        ast::Literal::HexNumberLiteral(hex, _) => self.hex_literal(&hex.hex),
                        ast::Literal::DecimalNumberLiteral(dec, _) => {
                            self.dec_literal(&dec.dec, expected_type)
                        }
                        ast::Literal::StringLiteral(string, _) => self.string_literal(&string.str),
                        ast::Literal::TrueLiteral(_) => self.bool_literal(true),
                        ast::Literal::FalseLiteral(_) => self.bool_literal(false),
                    };
                    let case_int =
                        self.try_cast(case_int.into(), cond_value_ty.as_basic_type_enum())?;
                    let case_int = if case_int.is_int_value() {
                        case_int.into_int_value()
                    } else {
                        self.try_into_u256(&case_int)?
                    };

                    let case_block = self
                        .llvm_context
                        .append_basic_block(cur_func_value, "switch_else");
                    jump_tbl.insert(case_int, (case_block, case.body.clone()));
                    jump_blocks.push((case_int, case_block));
                }

                let otherwise_block = self
                    .llvm_context
                    .append_basic_block(cur_func_value, "switch_otherwise");
                self.builder
                    .borrow_mut()
                    .build_switch(cond_value, otherwise_block, &jump_blocks);

                for (_, bb) in jump_tbl {
                    self.builder.borrow_mut().position_at_end(bb.0);
                    self.walk_block(yul_func_name, &bb.1);
                    self.builder
                        .borrow_mut()
                        .build_unconditional_branch(end_block);
                }

                self.builder.borrow_mut().position_at_end(otherwise_block);
                if let Some(default) = default {
                    self.walk_block(yul_func_name, &default.body);
                }
                self.builder
                    .borrow_mut()
                    .build_unconditional_branch(end_block);
            }
            ast::SwitchOptions::Default(default) => {
                let mut jump_tbl: IndexMap<u32, u32> = IndexMap::default();
                let otherwise_block = self
                    .llvm_context
                    .append_basic_block(cur_func_value, "switch_otherwise");
                self.builder.borrow_mut().position_at_end(otherwise_block);
                self.walk_block(yul_func_name, &default.body);
                self.builder
                    .borrow_mut()
                    .build_unconditional_branch(end_block);
            }
        }
        self.builder.borrow_mut().position_at_end(end_block);
        self.ok_result()
    }

    fn push_control_flow_end_bb(&self, bb: BasicBlock<'a>) {
        self.control_flow_blocks_end_bbs.borrow_mut().push(bb);
    }

    fn push_control_flow_continue_bb(&self, bb: BasicBlock<'a>) {
        self.control_flow_blocks_continue_bbs.borrow_mut().push(bb);
    }

    fn pop_control_flow_end_bb(&self) -> BasicBlock<'a> {
        self.control_flow_blocks_end_bbs.borrow_mut().pop().unwrap()
    }

    fn pop_control_flow_continue_bb(&self) -> BasicBlock<'a> {
        self.control_flow_blocks_continue_bbs
            .borrow_mut()
            .pop()
            .unwrap()
    }

    fn walk_for(&self, yul_func_name: &str, r#for: &ast::For) -> CompileResult<'a> {
        self.walk_block(yul_func_name, &r#for.init_block);
        let cur_func = self.current_function.borrow().clone().unwrap();
        let cur_func_value = *cur_func.clone();
        let cond_block = self
            .llvm_context
            .append_basic_block(cur_func_value, "for_cond");
        self.push_control_flow_continue_bb(cond_block);
        let body_block = self
            .llvm_context
            .append_basic_block(cur_func_value, "for_body");
        let end_block = self
            .llvm_context
            .append_basic_block(cur_func_value, "for_end");

        // Push the control flow end block onto the stack to facilitate break/continue in control flow
        self.push_control_flow_end_bb(end_block);

        self.builder
            .borrow_mut()
            .build_unconditional_branch(cond_block);

        self.builder.borrow_mut().position_at_end(cond_block);
        // For conditions, we prefer boolean (i32) results
        let cond_value =
            self.walk_expr_with_type(yul_func_name, &r#for.condition, ExpectedType::I32)?;
        let cond_value = cond_value.get_value();
        let cond_value = if cond_value.is_int_value() {
            cond_value.into_int_value()
        } else {
            self.try_into_u256(&cond_value)?
        };
        let bool_cond_value = self.int_to_bool(cond_value)?;
        self.builder
            .borrow_mut()
            .build_conditional_branch(bool_cond_value, body_block, end_block);

        self.builder.borrow_mut().position_at_end(body_block);
        self.walk_block(yul_func_name, &r#for.execution_block);
        self.walk_block(yul_func_name, &r#for.post_block);
        self.builder
            .borrow_mut()
            .build_unconditional_branch(cond_block);

        self.builder.borrow_mut().position_at_end(end_block);
        self.pop_control_flow_end_bb();
        self.pop_control_flow_continue_bb();
        self.ok_result()
    }

    fn walk_if(&self, yul_func_name: &str, r#if: &ast::If) -> CompileResult<'a> {
        // For conditions, we prefer boolean (i32) results
        let cond_value = self.walk_expr_with_type(yul_func_name, &r#if.cond, ExpectedType::Bool)?;
        let cond_value = cond_value.get_value();
        let cond_value = if cond_value.is_int_value() {
            cond_value.into_int_value()
        } else {
            self.try_into_u256(&cond_value)?
        };
        let bool_cond_value = self.int_to_bool(cond_value)?;
        let cur_func = self.current_function.borrow().clone().unwrap();
        let cur_func_value = *cur_func.clone();
        let then_block = self
            .llvm_context
            .append_basic_block(cur_func_value, "if_body");
        let end_block = self
            .llvm_context
            .append_basic_block(cur_func_value, "if_exit");

        let mut unreachableflag = false;
        self.builder
            .borrow_mut()
            .build_conditional_branch(bool_cond_value, then_block, end_block);
        self.builder.borrow_mut().position_at_end(then_block);
        // To avoid an empty then_block, add a no-operation instruction
        self.builder.borrow_mut().build_int_add(
            self.i32_type().const_zero(),
            self.i32_type().const_zero(),
            "nop",
        )?;
        for stmt in &r#if.body.statements {
            self.walk_stmt(yul_func_name, stmt)?;
            unreachableflag = self.is_unreachable_node(stmt);
        }
        // The internal then block should branch to the end block
        self.builder
            .borrow_mut()
            .build_unconditional_branch(end_block);

        if !unreachableflag {
            self.builder.borrow_mut().position_at_end(end_block);
        }

        self.builder.borrow_mut().position_at_end(end_block);
        self.ok_result()
    }

    fn walk_tuple_assignment(
        &self,
        yul_func_name: &str,
        assign: &ast::Assignment,
    ) -> CompileResult<'a> {
        let mut ret_tys = vec![];
        let value = &assign.value;
        ret_tys = match value {
            Expression::FunctionCall(func) => {
                let func_name = self.current_func_decls.borrow();
                let func_decl = func_name.get(&func.id.name);
                func_decl
                    .unwrap()
                    .returns
                    .clone()
                    .into_iter()
                    .map(|x| {
                        self.parse_ty_name_or_default(
                            &x.type_name,
                            self.default_func_return_element_type(),
                        )
                    })
                    .collect::<Vec<BasicTypeEnum<'a>>>()
            }
            _ => {
                return Err(ASTLoweringError {
                    message: "tuple variable assign value now only support function call"
                        .to_string(),
                });
            }
        };

        let ty = self.llvm_context.struct_type(&ret_tys, false);

        let tmp_value = self.fast_alloca(ty, "")?;

        let val = &assign.value;
        // For tuple returns, we still want to evaluate the function call with appropriate expected type
        // The function call will handle the return type properly
        let to_set_value = self.walk_expr_with_type(yul_func_name, val, ExpectedType::Untyped)?;
        self.builder
            .borrow_mut()
            .build_store(tmp_value, to_set_value.get_value());

        for (index, iden) in assign.identifiers.iter().enumerate() {
            let name = iden.name.clone();
            let _ty = ret_tys[index];

            let (cur_var_ty, _cur_var_low_level_value_type, cur_var, is_return_var) =
                self.get_var(&name).unwrap();

            let item_ty = ty.get_field_type_at_index(index as u32).unwrap();

            let item_value = self
                .builder
                .borrow_mut()
                .build_struct_gep(ty, tmp_value, index as u32, &format!("tmp_{name}"))
                .unwrap();

            let item = if self.default_func_return_low_level_value_type()
                == YulLowLevelValueType::Bytes32Pointer
            {
                let item_value = self
                    .build_load(self.bytes32_pointer_type(), item_value, "")?
                    .into_pointer_value();

                let item = self
                    .builder
                    .borrow_mut()
                    .build_load(self.bytes32_type(), item_value, "")
                    .unwrap();
                item
            } else {
                self.build_load(item_ty, item_value, "")?
            };

            if is_return_var
                && self.default_func_return_low_level_value_type()
                    == YulLowLevelValueType::Bytes32Pointer
            {
                // If the left-hand side variable is a local variable (bytes32*), the right-hand side Yul function return value after loading is also bytes32
                self.builder.borrow_mut().build_store(cur_var, item);
                continue;
            }

            // When using build_store, the pointer type and value type must match.
            let item = self.try_cast(item, cur_var_ty)?;

            self.builder.borrow_mut().build_store(cur_var, item);
        }
        self.ok_result()
    }

    fn walk_assignment(&self, yul_func_name: &str, assign: &ast::Assignment) -> CompileResult<'a> {
        let id = match assign.identifiers.len() {
            1 => &assign.identifiers[0],
            _ => {
                return self.walk_tuple_assignment(yul_func_name, assign);
            }
        };

        // Get the variable type to determine the expected type
        let (var_ty, var_low_level_value_type, var_pointer, is_return_var) =
            self.get_var(&id.name).unwrap();
        let expected_type = if var_low_level_value_type == YulLowLevelValueType::Bytes32Pointer {
            ExpectedType::Bytes32Pointer
        } else {
            self.type_to_expected(var_ty)
        };

        // Use the walk_expr_with_type to pass the expected type
        let val = self.walk_expr_with_type(yul_func_name, &assign.value, expected_type)?;

        // If the left-hand side is a return variable, the right-hand side is the function's return value.
        let var_low_level_value_real_type = if is_return_var {
            if self.default_func_return_low_level_value_type()
                == YulLowLevelValueType::Bytes32Pointer
            {
                if self.is_yul_function_call(&assign.value) {
                    // If the right value is a bytes32 pointer, load the content of the return variable.
                    let val_content =
                        self.build_load(self.bytes32_type(), val.value.into_pointer_value(), "")?;
                    self.builder
                        .borrow_mut()
                        .build_store(var_pointer, val_content);

                    return self.ok_result();
                }
                YulLowLevelValueType::Bytes32
            } else {
                var_low_level_value_type
            }
        } else {
            var_low_level_value_type
        };

        let val = self.try_cast(
            val.get_value(),
            if is_return_var
                && self.default_func_return_low_level_value_type()
                    == YulLowLevelValueType::Bytes32Pointer
            {
                self.bytes32_type().as_basic_type_enum() // ret var need read value of bytes32
            } else {
                var_ty
            },
        )?;

        self.builder.borrow_mut().build_store(var_pointer, val);

        self.ok_result()
    }

    fn walk_variable_declaration(
        &self,
        yul_func_name: &str,
        var_decl: &ast::VariableDeclaration,
    ) -> CompileResult<'a> {
        let (name, ty) = match var_decl.identifiers.len() {
            1 => (
                var_decl.identifiers[0].identifier.name.clone(),
                self.parse_ty_name(&var_decl.identifiers[0].type_name),
            ),
            _ => {
                return self.walk_tuple_variable_declaration(yul_func_name, var_decl);
            }
        };

        let id = self.next_iden_id();

        // if has right expression, and it matches linkersymbol expr,
        // then check if the variable is used in the function
        if let Some(val) = &var_decl.value {
            if self
                .matches_yul_instruction(val, "linkersymbol", 1)
                .is_some()
            {
                let usage_info = self.get_variable_usage(yul_func_name, &name);
                if usage_info.is_none() || usage_info.unwrap().reads == 0 {
                    // the variable not used, so we can optimize this variable declaration
                    return self.ok_result();
                }
            }
        }

        // Get the expected type based on the variable's type
        // If no type is explicitly declared, use Untyped
        let expected_type = if var_decl.identifiers[0].type_name.is_some() {
            self.type_to_expected(ty)
        } else {
            ExpectedType::Untyped
        };

        // Initialize with the expected type if there's an initializer
        let init_val_low_level = match &var_decl.value {
            Some(val) => Some(self.walk_expr_with_type(yul_func_name, val, expected_type)?),
            None => None,
        };

        // If the right-hand value is bytes32*, declare a bytes32* variable and copy the value
        if init_val_low_level.is_some()
            && init_val_low_level.unwrap().value_type == YulLowLevelValueType::Bytes32Pointer
        {
            let var_low_level_value_type = YulLowLevelValueType::Bytes32Pointer;
            let ty = self.bytes32_pointer_type().as_basic_type_enum();
            let llvm_var = self.fast_alloca(ty, &format!("var_{name}"))?;
            self.set_var(&name, ty, var_low_level_value_type, llvm_var, false);
            self.build_store(llvm_var, init_val_low_level.unwrap().value)?;
            return self.ok_result();
        }

        // TODO: Simplify the following code

        // if init_val is bytes32 pointer, convert to bytes32 to store in variable pointer
        let init_val = if init_val_low_level.is_some()
            && init_val_low_level.unwrap().value_type == YulLowLevelValueType::Bytes32Pointer
        {
            // is bytes32 pointer

            // now load bytes32 pointer target is to copy data to variable pointer
            // TODO: (alloca bytes32*) in stack to store bytes32 pointer
            let init_val_content = self.build_load(
                self.bytes32_type(),
                init_val_low_level.unwrap().get_value().into_pointer_value(),
                "",
            )?;
            Some(init_val_content)
        } else if init_val_low_level.is_some() {
            Some(init_val_low_level.unwrap().get_value())
        } else {
            None
        };

        let ty = if init_val_low_level.is_some()
            && init_val_low_level.unwrap().value_type == YulLowLevelValueType::Bytes32Pointer
        {
            self.bytes32_type().as_basic_type_enum()
        } else if init_val_low_level.is_some() {
            init_val_low_level.unwrap().get_value().get_type()
        } else {
            self.default_primitive_type()
        };

        // if not specified type, infer type from init value
        // let ty: BasicTypeEnum<'a> = if var_decl.identifiers[0].type_name.is_some() {
        //     ty
        // } else if init_val.is_some() {
        //     let res = {
        //         let this = &self;
        //         let value: &BasicValueEnum<'a> = &init_val.unwrap();
        //         self.is_bytes32_value(value)
        //     };
        //     if res {
        //         self.bytes32_type().into()
        //     } else {
        //         init_val.unwrap().into_int_value().get_type().into()
        //     }
        // } else {
        //     self.default_primitive_type()
        // };

        let var_low_level_value_type = if init_val_low_level.is_some()
            && init_val_low_level.unwrap().value_type == YulLowLevelValueType::Bytes32Pointer
        {
            YulLowLevelValueType::Bytes32
        } else if init_val_low_level.is_some() {
            init_val_low_level.unwrap().value_type
        } else {
            self.default_primitive_type_low_level()
        };

        let llvm_var = self.fast_alloca(ty, &format!("var_{name}"))?;
        self.set_var(&name, ty, var_low_level_value_type, llvm_var, false);

        if init_val_low_level.is_some()
            && init_val_low_level.unwrap().value_type == YulLowLevelValueType::Bytes32Pointer
        {
            // bytes32 pointer allocated in memory now, load content of pointer to variable
            // init_val bytes32 loaded yet
            self.builder
                .borrow_mut()
                .build_store(llvm_var, init_val.unwrap())
                .unwrap();
        } else if let Some(init_val) = init_val {
            let init_val = self.try_cast(init_val, ty)?;
            self.builder
                .borrow_mut()
                .build_store(llvm_var, init_val)
                .unwrap();
        }
        self.ok_result()
    }

    fn walk_tuple_variable_declaration(
        &self,
        yul_func_name: &str,
        var_decl: &ast::VariableDeclaration,
    ) -> CompileResult<'a> {
        let mut ret_tys = None;
        if let Some(value) = &var_decl.value {
            // TODO: func call return types can get from self.yul_func_infer_types
            ret_tys = Some(match value {
                Expression::FunctionCall(func) => {
                    let func_name = self.current_func_decls.borrow();
                    let func_decl = func_name.get(&func.id.name);
                    func_decl
                        .unwrap()
                        .returns
                        .clone()
                        .into_iter()
                        .map(|x| {
                            self.parse_ty_name_or_default(
                                &x.type_name,
                                self.default_func_return_element_type(),
                            )
                        })
                        .collect::<Vec<BasicTypeEnum<'a>>>()
                }
                _ => {
                    return Err(ASTLoweringError {
                        message: "tuple variable declaration value now only support function call"
                            .to_string(),
                    })
                }
            });
        }

        let ty = ret_tys
            .as_ref()
            .map(|ret_tys| self.llvm_context.struct_type(ret_tys, false));

        // Currently tuple assignment is mainly used for function return tuples
        // TODO: Need to handle cases like let a, b := 1, 2 assignment
        let tmp_value_ty = ty.map_or_else(
            || self.default_func_return_element_type(),
            |t| t.as_basic_type_enum(),
        );
        let tmp_value = self.fast_alloca(tmp_value_ty, "")?;

        if let Some(val) = &var_decl.value {
            // For tuple returns, use an appropriate expected type
            let init_val = self.walk_expr_with_type(yul_func_name, val, ExpectedType::Untyped)?;
            self.builder
                .borrow_mut()
                .build_store(tmp_value, init_val.get_value());
        }

        for (index, i) in var_decl.identifiers.iter().enumerate() {
            let new_var_name = i.identifier.name.clone();
            // let new_var_ty = self.parse_ty_name(&i.type_name);
            // TODO: Currently tuple assignment is mainly used for function return tuples
            // TODO: Need to handle cases like let a, b := 1, 2 assignment
            // let new_var_ty = self.default_func_return_element_type();
            // let new_var_low_level_value_type = self.default_func_return_low_level_value_type();

            let (new_var_ty, new_var_low_level_value_type) = if self
                .default_func_return_low_level_value_type()
                == YulLowLevelValueType::Bytes32Pointer
            {
                let new_var_ty = self.bytes32_type().as_basic_type_enum();
                let new_var_low_level_value_type = YulLowLevelValueType::Bytes32;
                (new_var_ty, new_var_low_level_value_type)
            } else {
                (
                    self.default_func_return_element_type(),
                    self.default_func_return_low_level_value_type(),
                )
            };

            let new_var = self.fast_alloca(new_var_ty, &new_var_name)?;
            self.set_var(
                &new_var_name,
                new_var_ty,
                new_var_low_level_value_type,
                new_var,
                false,
            );

            let item = match ty {
                Some(ty) => {
                    let item_ty = ty.get_field_type_at_index(index as u32).unwrap();
                    let item_value_ptr = self
                        .builder
                        .borrow_mut()
                        .build_struct_gep(
                            ty,
                            tmp_value,
                            index as u32,
                            &format!("tmp_{new_var_name}"),
                        )
                        .unwrap();

                    let item_value = self.build_load(item_ty, item_value_ptr, "")?;

                    // each field is bytes32 pointer
                    // need to load content of pointer to variable
                    // let item_value = if self.default_func_return_low_level_value_type() == YulLowLevelValueType::Bytes32Pointer {
                    //     self.build_load(self.bytes32_pointer_type(), item_value, "load_bytes32_pp")?.into_pointer_value()
                    // } else {
                    //     item_value
                    // };

                    let item_value = if self.default_func_return_low_level_value_type()
                        == YulLowLevelValueType::Bytes32Pointer
                    {
                        // func return each item is bytes32*, load content of pointer to variable
                        self.builder
                            .borrow_mut()
                            .build_load(self.bytes32_type(), item_value.into_pointer_value(), "")
                            .unwrap()
                    } else {
                        item_value
                    };
                    item_value
                }
                None => self.default_primitive_type().const_zero(),
            };
            // let item = self.try_cast(item, new_var_ty)?;
            self.builder.borrow_mut().build_store(new_var, item);
        }

        self.ok_result()
    }

    fn walk_typed_identifier(&self, typed_id: &ast::TypedIdentifier) -> CompileResult<'a> {
        self.walk_identifier(&typed_id.identifier);
        if let Some(ty) = &typed_id.type_name {
            self.walk_identifier(&ty.type_name);
        }
        self.ok_result()
    }

    pub(crate) fn walk_expr_with_type(
        &self,
        yul_func_name: &str,
        expr: &ast::Expression,
        expected_type: ExpectedType,
    ) -> CompileResult<'a> {
        match expr {
            ast::Expression::Identifier(id) => self.walk_identifier(id),
            ast::Expression::FunctionCall(func_call) => {
                // Use expected type for function calls
                let func_name = func_call.id.name.clone();
                if let Some(instr) = parse_intrinsic_func_name(&func_name) {
                    self.walk_yul_instruction(
                        yul_func_name,
                        instr,
                        func_call.arguments.as_slice(),
                        expected_type,
                    )
                } else {
                    self.walk_function_call(yul_func_name, func_call)
                }
            }
            ast::Expression::Literal(literal) => self.walk_literal(literal, expected_type),
        }
    }

    fn walk_function_call(
        &self,
        yul_func_name: &str,
        func_call: &ast::FunctionCall,
    ) -> CompileResult<'a> {
        let func_name = func_call.id.name.clone();
        if let Some(instr) = parse_intrinsic_func_name(&func_name) {
            let yul_generated_expr = self.walk_yul_instruction(
                yul_func_name,
                instr,
                func_call.arguments.as_slice(),
                ExpectedType::Untyped,
            );
            match yul_generated_expr {
                Ok(expr) => return Ok(expr),
                Err(e) => {
                    return Err(e);
                }
            }
        }

        if self.opts.enable_all_optimizers && self.opts.enable_storage_load_store_little_endian {
            // The Solidity ERC20 transfer function contains redundant keccak256 calls that need optimization.
            // Currently using C implementation for optimization.

            // the function logic check is in the transform_func
            if (func_call.id.name == "fun_transfer" || func_call.id.name == "fun__transfer")
                && func_call.arguments.len() == 3
            {
                let from_arg = self.walk_expr_with_type(
                    yul_func_name,
                    &func_call.arguments[0],
                    ExpectedType::Bytes32Pointer,
                )?;
                let from_arg = self.try_into_bytes32_pointer(&from_arg.get_value())?;
                let to_arg = self.walk_expr_with_type(
                    yul_func_name,
                    &func_call.arguments[1],
                    ExpectedType::Bytes32Pointer,
                )?;
                let to_arg = self.try_into_bytes32_pointer(&to_arg.get_value())?;
                let value_arg = self.walk_expr_with_type(
                    yul_func_name,
                    &func_call.arguments[2],
                    ExpectedType::Bytes32Pointer,
                )?;
                let value_arg = self.try_into_bytes32_pointer(&value_arg.get_value())?;
                self.build_void_call(
                    "wrapper_optimized_erc20_fun_transfer",
                    &[from_arg, to_arg, value_arg],
                )?;
                return Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::I32,
                    value: self.i32_type().const_zero().into(),
                });
            }
        }

        let module_name = self.current_module_name.borrow().clone();
        let contract_name = self.current_contract_name.borrow().clone();
        let mut ret_ty: BasicTypeEnum<'a> = self.default_func_return_element_type();
        let mut has_ret: bool = false;
        let mut is_tuple_ret_ty: bool = false;
        if let Some(func_ret_ty) = self.current_func_decls.borrow().get(&func_name) {
            if func_ret_ty.returns.len() > 1 {
                let item_returns = func_ret_ty
                    .returns
                    .clone()
                    .into_iter()
                    .map(|_x| self.default_func_return_element_type())
                    .collect::<Vec<BasicTypeEnum<'a>>>();
                let return_tuple_ty = self.llvm_context.struct_type(&item_returns, false);
                ret_ty = return_tuple_ty.into();
                has_ret = true;
                is_tuple_ret_ty = true;
            } else if func_ret_ty.returns.len() == 1 {
                ret_ty = self.default_func_return_element_type();
                has_ret = true;
            } else if func_ret_ty.returns.is_empty() {
                has_ret = false;
            }
        }

        let qualifier_func_name = self.get_func_decl_qualifier_name_by_str(&func_name);
        let infered_yul_func_ty = self
            .yul_func_infer_types
            .borrow()
            .get(&qualifier_func_name)
            .unwrap()
            .clone();

        let mut call_args = vec![];
        for (i, arg) in func_call.arguments.iter().enumerate() {
            // Try to get the expected parameter type from function declaration
            let expected_param_type =
                if let Some(func_decl) = self.current_func_decls.borrow().get(&func_name) {
                    if i < func_decl.params.len() {
                        let param_type = infered_yul_func_ty.params_inkwell_type[i];
                        self.type_to_expected(param_type)
                    } else {
                        unreachable!(
                            "param index out of range for function {}.{}",
                            module_name, func_name
                        );
                    }
                } else {
                    unreachable!("function {} not found in module {}", func_name, module_name);
                };

            match self.walk_expr_with_type(yul_func_name, arg, expected_param_type) {
                Ok(expr) => {
                    // Try to cast to the expected parameter type
                    let param_type = infered_yul_func_ty.params_inkwell_type[i];
                    let expr = self.try_cast(expr.get_value(), param_type)?;
                    let expr: BasicValueEnum = expr;
                    call_args.push(expr)
                }
                e => return e,
            }
        }

        let func_def = self.current_func_decls.borrow().get(&func_name).unwrap();

        if has_ret {
            let result = self.build_call(&qualifier_func_name, &call_args)?;
            // TODO: Set the return value type based on the function's inferred return type from self.yul_func_infer_types
            let func_result_low_level_value_ty = if has_ret && is_tuple_ret_ty {
                YulLowLevelValueType::Tuple
            } else if has_ret {
                self.default_func_return_low_level_value_type()
            } else {
                // When a Yul function has no return value
                YulLowLevelValueType::None
            };
            Ok(YulLowLevelValue {
                value_type: func_result_low_level_value_ty,
                value: result,
            })
        } else {
            self.build_void_call(&qualifier_func_name, &call_args);
            Ok(YulLowLevelValue {
                value_type: YulLowLevelValueType::I32,
                value: self.i32_type().const_zero().into(),
            })
        }
    }

    // The leave statement can be used to exit the current function.
    // It works like the return statement in other languages just that it does not take a value to return,
    // it just exits the functions and the function will return whatever values are currently assigned to the return variable(s).
    //
    // The leave statement can only be used inside a function.
    fn walk_leave(&self) -> CompileResult<'a> {
        // Similar to the return statement in regular functions, but the return instruction returns the result of the current contract call,
        // while leave returns the result of the current function.
        // Jump to the exit basic block.
        self.builder
            .borrow_mut()
            .build_unconditional_branch(self.cur_func_exit_bb.borrow().unwrap())
            .unwrap();
        // Here we need to create a temporary basic block to avoid having return as the last basic block,
        // which would generate LLVM IR with consecutive br $exit_bb, instead of a valid block.
        let tmp_bb = self
            .llvm_context
            .append_basic_block(*self.current_function.borrow().clone().unwrap(), "");
        self.builder.borrow_mut().position_at_end(tmp_bb);
        self.ok_result()
    }

    //  A continue or break statement can only be used inside the body of a for-loop, as follows.
    fn walk_break(&self) -> CompileResult<'a> {
        let binding = self.control_flow_blocks_end_bbs.borrow();
        let target_bb = binding.last().unwrap();
        self.builder
            .borrow_mut()
            .build_unconditional_branch(*target_bb);
        // Since the previous branch instruction will end the current basic block, we need to create a new basic block to continue.
        let cur_func = self.current_function.borrow().clone().unwrap();
        let cur_func_value = *cur_func.clone();
        let new_bb = self
            .llvm_context
            .append_basic_block(cur_func_value, "bb_after_break");
        self.builder.borrow_mut().position_at_end(new_bb);
        self.ok_result()
    }

    fn walk_continue(&self) -> CompileResult<'a> {
        let binding = self.control_flow_blocks_continue_bbs.borrow();
        let target_bb = binding.last().unwrap();
        self.builder
            .borrow_mut()
            .build_unconditional_branch(*target_bb);
        // Since the previous branch instruction will end the current basic block, we need to create a new basic block to continue.
        let cur_func = self.current_function.borrow().clone().unwrap();
        let cur_func_value = *cur_func.clone();
        let new_bb = self
            .llvm_context
            .append_basic_block(cur_func_value, "bb_after_continue");
        self.builder.borrow_mut().position_at_end(new_bb);
        self.ok_result()
    }

    pub fn get_bytes32_identifier_pointer(&self, id: &ast::Identifier) -> Option<PointerValue<'a>> {
        if let Some((id_ty, id_low_level_value_type, identifier_pointer, _is_return_var)) =
            self.get_var(&id.name)
        {
            if self.is_bytes32_type(&id_ty) {
                Some(identifier_pointer)
            } else {
                None
            }
        } else {
            unreachable!("Identifier {} not found", id.name)
        }
    }

    pub fn walk_identifier(&self, id: &ast::Identifier) -> CompileResult<'a> {
        // Find the PointerValue of the identifier in the current scope corresponding to its alloca
        if let Some((id_ty, id_low_level_value_type, identifier_pointer, is_return_var)) =
            self.get_var(&id.name)
        {
            if is_return_var
                && self.default_func_return_low_level_value_type()
                    == YulLowLevelValueType::Bytes32Pointer
            {
                return Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::Bytes32Pointer,
                    value: identifier_pointer.into(),
                });
            }

            if id_low_level_value_type == YulLowLevelValueType::Bytes32Pointer {
                let result =
                    self.build_load(self.bytes32_pointer_type(), identifier_pointer, &id.name)?;
                return Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::Bytes32Pointer,
                    value: result,
                });
            }

            if self.is_bytes32_type(&id_ty) {
                // Attempt to return the local variable pointer directly, using it or loading it when necessary
                return Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::Bytes32Pointer,
                    value: identifier_pointer.into(),
                });
            }
            let result = self
                .builder
                .borrow_mut()
                .build_load(id_ty, identifier_pointer, &id.name)
                .unwrap();
            Ok(YulLowLevelValue {
                value_type: id_low_level_value_type,
                value: result,
            })
        } else {
            unreachable!("Identifier {} not found", id.name)
        }
    }

    fn walk_literal(&self, lit: &ast::Literal, expected_type: ExpectedType) -> CompileResult<'a> {
        match lit {
            ast::Literal::TrueLiteral(ty_name) => Ok(YulLowLevelValue {
                value_type: YulLowLevelValueType::I32,
                value: self.bool_literal(true).into(),
            }),
            ast::Literal::FalseLiteral(ty_name) => Ok(YulLowLevelValue {
                value_type: YulLowLevelValueType::I32,
                value: self.bool_literal(false).into(),
            }),
            ast::Literal::HexNumberLiteral(hex, ty_name) => {
                Ok(self.hex_literal_or_bytes32_literal(&hex.hex))
            }
            ast::Literal::DecimalNumberLiteral(dec, ty_name) => {
                let value = self.dec_literal(&dec.dec, expected_type);
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::from_int_type(value.get_type()),
                    value: value.into(),
                })
            }
            ast::Literal::StringLiteral(s, ty_name) => {
                let value = self.string_literal(&s.str);
                Ok(YulLowLevelValue {
                    value_type: YulLowLevelValueType::from_int_type(value.get_type()),
                    value: value.into(),
                })
            }
        }
    }

    fn ok_result(&self) -> CompileResult<'a> {
        Ok(YulLowLevelValue {
            value_type: YulLowLevelValueType::I32,
            value: self.i32_type().const_zero().into(),
        })
    }
}

impl<'a> Yul2IRContext<'a> {
    pub fn transform(&mut self) -> CompileResult<'a> {
        self.scan_func_decls(&self.yul_ast.clone().unwrap())?;
        self.transform_object(&self.yul_ast.clone().unwrap(), true)?;
        self.ok_result()
    }

    fn get_func_decl_qualifier_name_by_str(&self, func_name: &str) -> String {
        // TODO: Use nested object names
        let module_name = self.current_module_name.borrow().clone();
        let contract_name = self.current_contract_name.borrow().clone();
        format!("{}.{}.{}", module_name, contract_name, func_name)
    }

    pub fn get_func_decl_qualifier_name(&self, func_def: &FunctionDefinition) -> String {
        self.get_func_decl_qualifier_name_by_str(&func_def.name.name)
    }

    fn matches_fun_transfer_pattern(&self, function: &FunctionDefinition) -> bool {
        // this pattern is foundry compiled with optimizer, by solc 0.8.25
        // optimized implementation of fun_transfer for standard ERC20:
        // yul function:
        // function fun_transfer(var_from, var_to, var_value)
        // {
        //     let _1 := and(var_from, sub(shl(160, 1), 1))
        //     let _2 := iszero(_1)
        //     if _2
        //     {
        //         mstore(0x00,shl(225, 0x4b637e8f))
        //         mstore(4,0x00)
        //         revert(0x00, 36)
        //     }
        //     let _3 := and(var_to, sub(shl(160, 1), 1))
        //     let _4 := iszero(_3)
        //     if _4
        //     {
        //         mstore(0x00, shl(224, 0xec442f05))
        //         mstore(4, 0x00)
        //         revert(0x00, 36)
        //     }
        //     _2 := 0x00
        //     mstore(0x00, _1)
        //     mstore(0x20, 0x00)
        //     let _5 := sload(keccak256(0x00, 0x40))
        //     if lt(_5, var_value)
        //     {
        //         mstore(0x00, shl(226, 0x391434e3))
        //         mstore(4, _1)
        //         mstore(36, _5)
        //         mstore(68, var_value)
        //         revert(0x00, 100)
        //     }
        //     mstore(0x00, _1)
        //     mstore(0x20, 0x00)
        //     sstore(keccak256(0x00, 0x40), sub(_5, var_value))
        //     _4 := 0x00
        //     mstore(0x00,  _3)
        //     mstore(0x20, 0x00)
        //     let dataSlot := keccak256(0x00, 0x40)
        //     sstore(dataSlot, add(sload(dataSlot), var_value))
        //     let _6 := mload(64)
        //     mstore(_6, var_value)
        //     log3(_6, 32,
        //     0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef, _1,
        //     _3)
        // }
        if !function.name.name.contains("fun_transfer")
            && !function.name.name.contains("fun__transfer")
        {
            return false;
        }
        if !self.current_module_name.borrow().contains("Token")
            && !self.current_module_name.borrow().contains("ERC20")
        {
            return false;
        }
        if function.params.len() != 3 {
            return false;
        }
        let _param0_iden = function.params[0].identifier.name.clone();
        let _param1_iden = function.params[1].identifier.name.clone();
        let _param2_iden = function.params[2].identifier.name.clone();
        let stmts = &function.body.statements;
        if stmts.len() < 18 || stmts.len() > 22 {
            return false;
        }
        // let _1 := sub(shl(160, 1), 1)
        let var_1_instr = self.matches_single_declare_instruction(&stmts[0]);
        if var_1_instr.is_none() {
            return false;
        }
        let (var_1_iden, var_1_expr) = var_1_instr.unwrap();
        if var_1_iden != "_1" {
            return false;
        }
        if self
            .matches_yul_instruction(&var_1_expr, "sub", 2)
            .is_none()
        {
            return false;
        }
        // let _2 := and(var_from, _1)
        let var_2_instr = self.matches_single_declare_instruction(&stmts[1]);
        if var_2_instr.is_none() {
            return false;
        }
        let (var_2_iden, var_2_expr) = var_2_instr.unwrap();
        if var_2_iden != "_2" {
            return false;
        }
        if self
            .matches_yul_instruction(&var_2_expr, "and", 2)
            .is_none()
        {
            return false;
        }
        // if iszero(_2)
        if !matches!(stmts[2], Statement::If(_)) {
            return false;
        }
        // let _4 := and(var_to, _1)
        let var_4_instr = self.matches_single_declare_instruction(&stmts[3]);
        if var_4_instr.is_none() {
            return false;
        }
        let (var_4_iden, _) = var_4_instr.unwrap();
        if var_4_iden != "_4" {
            return false;
        }
        // if iszero(_4)
        if !matches!(stmts[4], Statement::If(_)) {
            return false;
        }
        // mstore(0, _2)
        if self.matches_mstore_statement(&stmts[5]).is_none() {
            return false;
        }
        // mstore(0x20, 0)
        if self.matches_mstore_statement(&stmts[6]).is_none() {
            return false;
        }
        // let _6 := sload(keccak256(0, 0x40))
        let var_6_instr = self.matches_single_declare_instruction(&stmts[7]);
        if var_6_instr.is_none() {
            return false;
        }
        let (var_6_iden, _) = var_6_instr.unwrap();
        if var_6_iden != "_6" {
            return false;
        }
        // if lt(_6, var_value)
        if !matches!(stmts[8], Statement::If(_)) {
            return false;
        }
        // mstore(0, _2)
        if self.matches_mstore_statement(&stmts[9]).is_none() {
            return false;
        }
        // mstore(0x20, 0)
        if self.matches_mstore_statement(&stmts[10]).is_none() {
            return false;
        }
        // sstore(keccak256(0, 0x40), sub(_6, var_value))
        if self.matches_sstore_statement(&stmts[11]).is_none() {
            return false;
        }
        // mstore(0, _4)
        if self.matches_mstore_statement(&stmts[12]).is_none() {
            return false;
        }
        // let dataSlot := keccak256(0, 0x40)
        if self
            .matches_single_declare_instruction(&stmts[13])
            .is_none()
        {
            return false;
        }
        // sstore(dataSlot, add(sload(dataSlot), var_value))
        if self.matches_sstore_statement(&stmts[14]).is_none() {
            return false;
        }
        // let _8 := mload(0x40)
        if self
            .matches_single_declare_instruction(&stmts[15])
            .is_none()
        {
            return false;
        }
        // mstore(_8, var_value)
        if self.matches_mstore_statement(&stmts[16]).is_none() {
            return false;
        }
        //  log3
        if self.matches_log3_statement(&stmts[17]).is_none() {
            return false;
        }
        true
    }

    fn transform_func_llvm_ty(
        &self,
        function: &FunctionDefinition,
    ) -> (FunctionType<'a>, YulLowLevelFunctionType<'a>) {
        let (mut params, mut vars) = (vec![], IndexMap::new());

        let mut func_low_level_type = YulLowLevelFunctionType::new(vec![], vec![]);

        for (i, param) in function.params.iter().enumerate() {
            let _param_name = param.identifier.name.clone();
            let mut ir_ty: BasicTypeEnum<'a> =
                self.parse_ty_name_or_default(&param.type_name, self.default_param_type());

            let mut param_low_level_type = YulLowLevelValueType::from_basic_type_enum(ir_ty);

            if (function.name.name.contains("fun_transfer")
                || function.name.name.contains("fun__transfer"))
                && function.params.len() == 3
                && ((i == 0 || i == 1) || (self.opts.enable_all_optimizers && i == 2))
            {
                // The first two parameters of fun_transfer are typically addresses, and the third parameter is usually uint256
                // Therefore, we prioritize using appropriate parameter types
                // When dealing with ERC20 and using the enable_all_optimizers option, we match the fun_transfer pattern and use the C implementation

                ir_ty = self.bytes32_pointer_type().into();
                param_low_level_type = YulLowLevelValueType::Bytes32Pointer;
            }

            let id = self.next_iden_id();
            params.push(ir_ty);
            vars.insert(id, ir_ty);

            func_low_level_type.add_param(param_low_level_type, ir_ty);
        }

        let mut params_meta_types: Vec<BasicMetadataTypeEnum<'a>> = vec![];
        for param in &params {
            params_meta_types.push(BasicMetadataTypeEnum::from(*param));
        }

        let func_ty: FunctionType<'a> = match function.returns.clone().len() {
            0 => self
                .llvm_context
                .void_type()
                .fn_type(&params_meta_types, false),
            1 => {
                let ret_ty = self.default_func_return_element_type();
                let ret_low_level_value_type = self.default_func_return_low_level_value_type();

                let id = self.next_iden_id();
                let _ret_name = &function.returns[0].identifier.name;
                vars.insert(id, ret_ty);

                func_low_level_type.add_return(ret_low_level_value_type, ret_ty);

                ret_ty.fn_type(&params_meta_types, false)
            }
            _ => {
                let mut tuple_ty = vec![];
                for ret_ty in function.returns.clone() {
                    let parsed_ret_ty = self.default_func_return_element_type();
                    let parsed_ret_low_level_value_type =
                        self.default_func_return_low_level_value_type();

                    let id = self.next_iden_id();
                    let _ret_name = &ret_ty.identifier.name;
                    vars.insert(id, parsed_ret_ty);
                    tuple_ty.push(parsed_ret_ty);

                    func_low_level_type.add_return(parsed_ret_low_level_value_type, parsed_ret_ty);
                }

                let ret_struct = self.llvm_context.struct_type(&tuple_ty, false);
                ret_struct.fn_type(&params_meta_types, false)
            }
        };

        (func_ty, func_low_level_type)
    }

    fn scan_func_decls(&self, object: &Object) -> CompileResult<'a> {
        // Recursively scan the function definitions within each object, generate the LLVM functions,
        // and store the mapping of function names to func_value in functions_mapping.
        *self.current_module_name.borrow_mut() = object.name.clone();
        *self.current_contract_name.borrow_mut() = object.name.clone();

        // Scan function declarations in current object
        for func in object
            .code
            .statements
            .iter()
            .filter(|stmt| matches!(stmt, ast::Statement::FunctionDefinition(_)))
        {
            if let ast::Statement::FunctionDefinition(func_def) = func {
                let qualifier_func_name = self.get_func_decl_qualifier_name(func_def);
                let (func_ty, func_low_level_type) = self.transform_func_llvm_ty(func_def);

                self.yul_func_infer_types
                    .borrow_mut()
                    .insert(qualifier_func_name.clone(), func_low_level_type);

                // Add function to module
                let function = self.llvm_module.borrow_mut().add_function(
                    &qualifier_func_name,
                    func_ty,
                    Some(Linkage::External),
                );

                // walk function body for variable usage
                self.analyze_function_usage(func_def);

                // Store function mapping
                self.functions.borrow_mut().push(Rc::new(function));
                self.functions_mapping
                    .borrow_mut()
                    .insert(qualifier_func_name, Rc::new(function));
            }
        }

        // Recursively scan nested objects in inner_segments
        for inner_segment in object.inner_segments.iter() {
            if let InnerSegment::Object(inner_object) = inner_segment {
                self.scan_func_decls(inner_object)?;
            }
        }

        self.ok_result()
    }

    pub fn transform_object(&self, object: &Object, is_main: bool) -> CompileResult<'a> {
        // First, compile the data segments in the inner segments of the object, as they may be accessed by internal functions.

        // Then, compile the nested objects
        for inner_segment in object.inner_segments.iter() {
            match inner_segment {
                InnerSegment::Object(inner_object) => {
                    // If the current object and the child object are contracts with the same name (the child object has a "_deployed" suffix),
                    // then since the wasm contracts are compiled together, recompiling the deployed child object will only bloat the bytecode.
                    // Therefore, in this case, the data_bytes of the child object will be treated as empty bytes.
                    let is_same_object = format!("{}_deployed", object.name) == inner_object.name;

                    let emited_sub_wasm = if is_same_object {
                        vec![]
                    } else {
                        // Create a new context to transform compile the child object into wasm,
                        // then add it as a global constant with name {parent_object}.{child_object_name}
                        let sub_llvm_ctx = Context::create();
                        let sub_opts = self.opts.clone();
                        let mut sub_ctx = Yul2IRContext::new_with_object(
                            &sub_llvm_ctx,
                            &sub_opts,
                            *inner_object.clone(),
                        );
                        let output_path = &self.opts.output_dir;
                        let sub_contract_base_path = &format!(
                            "{output_path}/{}_{}",
                            self.opts.main_contract_name, &inner_object.name
                        );
                        let emited_sub_wasm = sub_ctx.emit(&inner_object.name).unwrap();
                        // write sub contract to wasm path
                        std::fs::write(
                            format!("{}.wasm", sub_contract_base_path),
                            &emited_sub_wasm,
                        )
                        .unwrap();
                        emited_sub_wasm
                    };

                    // Store the LLVM global variable with 4 bytes (big endian int) + emitted sub wasm, which together form the contract bytecode ABI
                    let sub_wasm_with_length_prefix_bytes =
                        merge_sub_wasm_with_length_prefix(&emited_sub_wasm);

                    // Write the LLVM global constant bytes for the emitted sub wasm and record it in global_bytes_lengths
                    let data_segment_qualified_name =
                        format!("{}.{}", object.name, inner_object.name);
                    let global_var = self.add_global_constant_bytes32(
                        &sub_wasm_with_length_prefix_bytes,
                        &data_segment_qualified_name,
                    );

                    self.global_bytes_lengths.borrow_mut().insert(
                        data_segment_qualified_name.to_string(),
                        sub_wasm_with_length_prefix_bytes.len(),
                    );
                    self.global_bytes_values
                        .borrow_mut()
                        .insert(data_segment_qualified_name.to_string(), global_var);

                    // If it's a deployed child contract, we need to compile it in this context as well to get the call function
                    if object.name == inner_object.name
                        || inner_object.name == format!("{}_deployed", &object.name)
                    {
                        self.transform_object(inner_object, false)?;
                    }
                }
                InnerSegment::Data(data_segment_name, data_literals) => {
                    // Compile data segments from object inner_segments into LLVM global constants
                    for data_literal in data_literals {
                        match data_literal {
                            ast::DataLiteral::HexLiteral(hex_literal) => {
                                let mut bytes = hex_literal.clone();
                                if hex_literal.len() % 2 != 0 {
                                    bytes.push('0');
                                }
                                let data_bytes = hex::decode(bytes).unwrap();
                                let data_bytes_with_length_prefix =
                                    merge_sub_wasm_with_length_prefix(&data_bytes);

                                let data_segment_qualified_name =
                                    &format!("{}.{}", object.name, data_segment_name);
                                let global_var = self.add_global_constant_bytes32(
                                    &data_bytes_with_length_prefix,
                                    data_segment_qualified_name,
                                );

                                self.global_bytes_lengths.borrow_mut().insert(
                                    data_segment_qualified_name.to_string(),
                                    data_bytes_with_length_prefix.len(),
                                );
                                self.global_bytes_values
                                    .borrow_mut()
                                    .insert(data_segment_qualified_name.to_string(), global_var);
                            }
                            ast::DataLiteral::StringLiteral(string_literal) => {
                                let data_bytes = string_literal.as_bytes();
                                let data_bytes_with_length_prefix =
                                    merge_sub_wasm_with_length_prefix(data_bytes);

                                let data_segment_qualified_name =
                                    &format!("{}.{}", object.name, data_segment_name);
                                let global_var = self.add_global_constant_bytes32(
                                    &data_bytes_with_length_prefix,
                                    data_segment_qualified_name,
                                );

                                self.global_bytes_lengths.borrow_mut().insert(
                                    data_segment_qualified_name.to_string(),
                                    data_bytes_with_length_prefix.len(),
                                );
                                self.global_bytes_values
                                    .borrow_mut()
                                    .insert(data_segment_qualified_name.to_string(), global_var);
                            }
                        }
                    }
                }
            }
        }

        // Start scanning and compiling function implementations in this object
        // Set current module name to help with compiling internal function instructions
        *self.current_module_name.borrow_mut() = object.name.clone();
        *self.current_contract_name.borrow_mut() = object.name.clone();

        let module_name = self.current_module_name.borrow().clone();
        let contract_name = self.current_contract_name.borrow().clone();
        if is_main {
            *self.main_module.borrow_mut() = module_name.clone();
        }

        for func in object
            .code
            .statements
            .iter()
            .filter(|stmt| matches!(stmt, ast::Statement::FunctionDefinition(_)))
        {
            let mut func_decls = self.current_func_decls.borrow_mut();
            if let ast::Statement::FunctionDefinition(func_def) = func {
                func_decls.insert(
                    func_def.name.name.clone(),
                    FunctionDeclaration {
                        name: func_def.name.clone(),
                        params: func_def.params.clone(),
                        returns: func_def.returns.clone(),
                    },
                );
            }
        }

        for func in object
            .code
            .statements
            .iter()
            .filter(|stmt| matches!(stmt, ast::Statement::FunctionDefinition(_)))
        {
            if let ast::Statement::FunctionDefinition(func_def) = func {
                let qualifier_func_name = format!(
                    "{}.{}.{}",
                    module_name,
                    contract_name,
                    func_def.name.name.clone()
                );
                let func_def: Rc<FunctionValue> =
                    Rc::new(self.transform_func(func_def, qualifier_func_name.clone())?);

                self.functions_mapping
                    .borrow_mut()
                    .insert(qualifier_func_name, func_def);
            }
        }

        let (init_def, init_def_qualified_name) = self.transform_init_func(object);
        let init_def = Rc::new(init_def?);
        self.functions_mapping
            .borrow_mut()
            .insert("init".to_string(), init_def);

        // If this object is the main contract object, generate a deploy function and call function that internally calls the init function of "XXX" and "XXX_object"
        let is_deployed_object = object.name.ends_with("_deployed");
        // Both parent and child contracts need to export deploy and call functions
        let export_func_name = if is_deployed_object { "call" } else { "deploy" };
        let export_func_ty: FunctionType<'a> = self.llvm_context.void_type().fn_type(&[], false);
        let export_func = Rc::new(self.llvm_module.borrow_mut().add_function(
            export_func_name,
            export_func_ty,
            None,
        ));
        let entry_bb = self.llvm_context.append_basic_block(*export_func, "entry");
        self.functions_mapping
            .borrow_mut()
            .insert(export_func_name.to_string(), export_func.clone());
        self.functions.borrow_mut().push(export_func.clone());
        self.builder.borrow_mut().position_at_end(entry_bb);
        *self.current_function.borrow_mut() = Some(export_func);
        *self.current_function_definition.borrow_mut() = None;
        // call set_is_deploying_tx()
        if export_func_name == "deploy" {
            self.build_void_call("set_is_deploying_tx", &[])?;
        }
        // call init code func
        self.build_void_call(&init_def_qualified_name, &[])?;
        self.builder.borrow_mut().build_return(None)?;
        self.exported_func_names
            .borrow_mut()
            .push(export_func_name.to_string());

        if !*(self.wasm_start_inited.borrow()) {
            // if the wasm start function hasn't been added yet,
            // add one to initialize the EVM heap(__init_evm_heap) and other functions
            let wasm_start_func = self.llvm_module.borrow_mut().add_function(
                "_start",
                self.void_type().fn_type(&[], false),
                None,
            );
            let wasm_start_entry = self
                .llvm_context
                .append_basic_block(wasm_start_func, "entry");
            self.builder.borrow_mut().position_at_end(wasm_start_entry);

            let try_new_wasm_page_as_evm_heap =
                !self.opts.enable_all_optimizers || has_sub_contract(object);

            self.build_void_call(
                "__init_evm_heap",
                &[self
                    .i32_type()
                    .const_int(try_new_wasm_page_as_evm_heap.into(), false)
                    .into()],
            )?;
            self.builder.borrow_mut().build_return(None)?;
            *self.wasm_start_inited.borrow_mut() = true;
        }

        self.ok_result()
    }

    fn transform_init_func(&self, object: &Object) -> (CompileFunctionResult<'a>, String) {
        let module_name = self.current_module_name.borrow().clone();
        let contract_name = self.current_contract_name.borrow().clone();
        let qualifier_func_name = format!("{}.{}.init", module_name, contract_name,);

        let params_meta_types: Vec<BasicMetadataTypeEnum<'a>> = vec![];
        let func_ty: FunctionType<'a> = self
            .llvm_context
            .void_type()
            .fn_type(&params_meta_types, false);
        let func_value: FunctionValue<'a> = self.llvm_module.borrow_mut().add_function(
            &qualifier_func_name,
            func_ty,
            None, /* Linkage */
        );
        self.functions.borrow_mut().push(Rc::new(func_value));
        self.functions_mapping
            .borrow_mut()
            .insert(qualifier_func_name.clone(), Rc::new(func_value));
        let entry_bb = self.llvm_context.append_basic_block(func_value, "entry");
        let exit_bb = self.llvm_context.append_basic_block(func_value, "exit");
        *self.cur_func_exit_bb.borrow_mut() = Some(exit_bb);

        self.builder.borrow_mut().position_at_end(entry_bb);
        *self.current_function.borrow_mut() = Some(Rc::new(func_value));
        *self.current_function_definition.borrow_mut() = None;

        let _scope_guard = ScopeGuard::new(self);

        for stmt in &object.code.statements {
            if let Err(err) = self.walk_stmt(&qualifier_func_name, stmt) {
                return (Err(err), qualifier_func_name);
            }
        }
        // Jump to exit block since each basic block must end with an explicit branch or return
        self.builder
            .borrow_mut()
            .build_unconditional_branch(exit_bb)
            .unwrap();
        // Exit block logic

        self.builder.borrow_mut().position_at_end(exit_bb);

        self.builder.borrow_mut().build_return(None).unwrap();
        (Ok(func_value), qualifier_func_name)
    }

    fn transform_func(
        &self,
        function: &FunctionDefinition,
        qualifier_func_name: String,
    ) -> CompileFunctionResult<'a> {
        let (func_ty, func_low_level_type) = self.transform_func_llvm_ty(function);
        let mut func_value_new = false;
        let func_value = self
            .functions_mapping
            .borrow()
            .get(&qualifier_func_name)
            .map(|x| *x.clone())
            .unwrap_or_else(|| {
                let func_value = self.llvm_module.borrow_mut().add_function(
                    &qualifier_func_name,
                    func_ty,
                    None, /* Linkage */
                );
                func_value_new = true;
                func_value
            });
        if func_value_new {
            self.functions.borrow_mut().push(Rc::new(func_value));
            self.functions_mapping
                .borrow_mut()
                .insert(qualifier_func_name.clone(), Rc::new(func_value));
        }

        let entry_bb = self.llvm_context.append_basic_block(func_value, "entry");
        let exit_bb = self.llvm_context.append_basic_block(func_value, "exit");
        *self.cur_func_exit_bb.borrow_mut() = Some(exit_bb);

        self.builder.borrow_mut().position_at_end(entry_bb);
        *self.current_function.borrow_mut() = Some(Rc::new(func_value));
        *self.current_function_definition.borrow_mut() = Some(function.clone());

        let _scope_guard = ScopeGuard::new(self);

        let infered_yul_func_ty = self
            .yul_func_infer_types
            .borrow()
            .get(&qualifier_func_name)
            .unwrap()
            .clone();

        // Declare parameter variables
        for (i, param) in function.params.iter().enumerate() {
            let param_name = &param.identifier.name.clone();
            let param_ty = infered_yul_func_ty.params_inkwell_type[i];
            let param_low_level_value_type = infered_yul_func_ty.params[i];
            let param_var_pointer = self.fast_alloca(param_ty, param_name)?;
            // Store the parameter value in this variable
            let param_value = func_value.get_nth_param(i as u32).unwrap();
            let param_value = self.try_cast(param_value, param_ty)?;
            self.build_store(param_var_pointer, param_value)?;
            self.set_var(
                param_name,
                param_ty,
                param_low_level_value_type,
                param_var_pointer,
                false,
            );
        }
        // Declare return variables
        for (i, ret_info) in function.returns.iter().enumerate() {
            let ret_ty = func_low_level_type.returns_inkwell_type[i];
            let ret_var_low_level_value_type = func_low_level_type.returns[i];

            let ret_name = &ret_info.identifier.name;
            let (ret_var_pointer, ret_var_low_level_value_type) =
                if ret_var_low_level_value_type == YulLowLevelValueType::Bytes32Pointer {
                    // alloca bytes32 in memory
                    // var used like bytes32 pointer, but not load data to return
                    (
                        self.memory_alloca_first(self.bytes32_type(), ret_name)?,
                        YulLowLevelValueType::Bytes32,
                    )
                } else {
                    (
                        self.fast_alloca(ret_ty, ret_name)?,
                        ret_var_low_level_value_type,
                    )
                };

            // TODO: Set initial value for the return variable
            self.set_var(
                ret_name,
                ret_ty,
                ret_var_low_level_value_type,
                ret_var_pointer,
                true,
            );
        }

        if self.opts.enable_all_optimizers {
            // For standard ERC20 implementations, we can use the optimized C implementation for fun_transfer
            // Since walk_function_call will handle the call replacement, we only need to verify if the function logic can be replaced
            if (function.name.name.contains("fun_transfer")
                || function.name.name.contains("fun__transfer"))
                && function.params.len() == 3
                && !self.matches_fun_transfer_pattern(function)
            {
                panic!("fun_transfer is not a standard ERC20 implementation, please disable the --enable-all-optimizers option for this contract");
            }
        }

        for stmt in &function.body.statements {
            self.walk_stmt(&qualifier_func_name, stmt)?;
        }
        // Entering the exit function label
        // The previous logic needs to jump to the exit basic block,
        // as each basic block must have an explicit jump or return at the end
        self.builder
            .borrow_mut()
            .build_unconditional_branch(exit_bb)?;
        // Logic for exiting the block

        self.builder.borrow_mut().position_at_end(exit_bb);

        // Adding return values for the current function
        match function.returns.len() {
            0 => {
                self.builder.borrow_mut().build_return(None).unwrap();
            }
            1 => {
                let ret_name = &function.returns[0].identifier.name;
                let (ret_var_ty, _ret_var_low_level_value_type, ret_var_pointer, _) =
                    self.get_var(ret_name).unwrap();
                let ret_value = if self.default_func_return_low_level_value_type()
                    == YulLowLevelValueType::Bytes32Pointer
                {
                    // ret value is bytes32 pointer, it is allocated in memory now
                    // so return it directly
                    ret_var_pointer.as_basic_value_enum()
                } else {
                    self.build_load(ret_var_ty, ret_var_pointer, ret_name)?
                };
                self.builder
                    .borrow_mut()
                    .build_return(Some(&ret_value))
                    .unwrap();
            }
            _ => {
                // Read return variable values and fill them into the return struct
                let full_return_tuple_ty = func_ty.get_return_type().unwrap().into_struct_type();
                // Create an empty struct first, then set field values one by one
                let return_tuple = self.fast_alloca(
                    full_return_tuple_ty,
                    &format!("return.{}", qualifier_func_name),
                )?;

                for (ret_idx, ret_info) in function.returns.iter().enumerate() {
                    let ret_name = &ret_info.identifier.name;
                    let (ret_var_ty, _ret_var_low_level_value_type, ret_var_pointer, _) =
                        self.get_var(ret_name).unwrap();

                    let ret_value = if self.default_func_return_low_level_value_type()
                        == YulLowLevelValueType::Bytes32Pointer
                    {
                        // ret value is bytes32 pointer, it is allocated in memory now
                        // so return it directly
                        ret_var_pointer.as_basic_value_enum()
                    } else {
                        self.build_load(ret_var_ty, ret_var_pointer, ret_name)?
                    };

                    let field_pointer = self
                        .builder
                        .borrow_mut()
                        .build_struct_gep(
                            full_return_tuple_ty,
                            return_tuple,
                            ret_idx as u32,
                            ret_name,
                        )
                        .unwrap();
                    self.builder
                        .borrow_mut()
                        .build_store(field_pointer, ret_value)
                        .unwrap();
                }
                let return_tuple_value = self
                    .builder
                    .borrow_mut()
                    .build_load(full_return_tuple_ty, return_tuple, "")
                    .unwrap();
                self.builder
                    .borrow_mut()
                    .build_return(Some(&return_tuple_value))
                    .unwrap();
            }
        }

        Ok(func_value)
    }

    pub(crate) fn is_unreachable_node(&self, stmt: &Statement) -> bool {
        matches!(
            stmt,
            Statement::Break | Statement::Continue | Statement::Leave
        )
    }
}
