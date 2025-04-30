// Copyright (C) 2024-2025 the DTVM authors. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use crate::yul2ir::ast::{FunctionDeclaration, FunctionDefinition, Object, TypeName};
use crate::yul2ir::config::Yul2IROptions;
use crate::yul2ir::errors::ASTLoweringError;
use crate::yul2ir::stdlib::load_stdlib;
use crate::yul2ir::var_scope::VarScope;
use ethereum_types::U256;
use indexmap::IndexMap;
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::passes::{PassManager, PassManagerBuilder};
use inkwell::targets::{CodeModel, FileType, RelocMode, TargetTriple};
use inkwell::types::{
    ArrayType, BasicType, BasicTypeEnum, IntType, PointerType, StringRadix, VoidType,
};
use inkwell::values::{
    BasicMetadataValueEnum, BasicValue, BasicValueEnum, FunctionValue, GlobalValue, IntValue,
    PointerValue,
};
use inkwell::{AddressSpace, IntPredicate};
use once_cell::sync::OnceCell;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::ops::Add;
use std::{cell::RefCell, rc::Rc};

use super::ast::{Expression, Literal, Statement};
use super::infer::ExpectedType;
use super::transform::has_sub_contract;
use super::yul_instruction::{
    parse_intrinsic_func_name, YulLowLevelFunctionType, YulLowLevelValue, YulLowLevelValueType,
};

/// The compiler function result
pub type CompileResult<'a> = Result<YulLowLevelValue<'a>, ASTLoweringError>;
pub type CompileFunctionResult<'a> = Result<FunctionValue<'a>, ASTLoweringError>;

/// Variable Usage Info
#[derive(Default, Clone, Debug)]
pub struct UsageInfo {
    pub reads: usize,
    pub writes: usize,
}

#[derive(Debug)]
pub struct Yul2IRContext<'ctx> {
    pub opts: &'ctx Yul2IROptions,
    pub yul_ast: Option<Object>,
    pub current_module_name: RefCell<String>,
    pub current_contract_name: RefCell<String>,

    pub main_module: RefCell<String>,

    pub llvm_context: &'ctx Context,
    pub llvm_module: RefCell<Module<'ctx>>,
    // pub memptr_global: RefCell<Option<GlobalValue<'ctx>>>,
    pub builder: RefCell<Builder<'ctx>>,
    pub functions: RefCell<Vec<Rc<FunctionValue<'ctx>>>>,
    pub current_function: RefCell<Option<Rc<FunctionValue<'ctx>>>>,
    pub current_function_definition: RefCell<Option<FunctionDefinition>>,
    // The exit label of the currently compiling function
    pub cur_func_exit_bb: RefCell<Option<BasicBlock<'ctx>>>,
    // The end basic blocks of control flow blocks, stack structure
    pub control_flow_blocks_end_bbs: RefCell<Vec<BasicBlock<'ctx>>>,
    // The continue operation basic blocks of control flow blocks, stack structure
    pub control_flow_blocks_continue_bbs: RefCell<Vec<BasicBlock<'ctx>>>,
    // Maintains the variable name to PointerValue mapping in the current scope when switching current_function
    pub vars_scopes: RefCell<Vec<VarScope<'ctx>>>,

    pub functions_mapping: RefCell<HashMap<String, Rc<FunctionValue<'ctx>>>>,
    pub current_func_decls: RefCell<IndexMap<String, FunctionDeclaration>>,
    pub revert_zero_functions: RefCell<HashSet<String>>,

    // yul function name => yul low level function type
    pub yul_func_infer_types: RefCell<HashMap<String, YulLowLevelFunctionType<'ctx>>>,

    // *** New: Variable Usage Tracking ***
    // Function Qualifier -> Variable Name -> UsageInfo
    pub variable_usage: RefCell<HashMap<String, HashMap<String, UsageInfo>>>,
    // *** End New ***
    pub iden_id_gen: RefCell<u32>,
    pub exported_func_names: RefCell<Vec<String>>,

    pub wasm_start_inited: RefCell<bool>,

    /// Length of each bytes (i8 array) global constant, qualified_name => length
    pub global_bytes_lengths: RefCell<HashMap<String, usize>>,

    /// LLVM GlobalValue of each bytes (i8 array) global constant, qualified_name => GlobalValue
    pub global_bytes_values: RefCell<HashMap<String, GlobalValue<'ctx>>>,

    pub default_ret_type: YulLowLevelValueType,
}

static LLVM_INIT: OnceCell<()> = OnceCell::new();

pub const FUNCTION_RETURN_VALUE_NOT_FOUND_MSG: &str = "Function return value is not found";

impl<'ctx> Yul2IRContext<'ctx> {
    pub fn new_with_object(
        context: &'ctx Context,
        opts: &'ctx Yul2IROptions,
        object: Object,
    ) -> Self {
        let llvm_module_name = "ir_generated";

        LLVM_INIT.get_or_init(|| {
            inkwell::targets::Target::initialize_webassembly(&Default::default());
        });
        // Create a empty LLVM module
        let module: RefCell<Module<'ctx>> = RefCell::new(context.create_module(llvm_module_name));
        // Link stdlib
        let extend_runtime: Vec<&[u8]> = vec![];
        let intr = load_stdlib(opts, context, extend_runtime);
        module.borrow_mut().link_in_module(intr).unwrap();
        Yul2IRContext {
            opts,
            yul_ast: Some(object),
            current_module_name: RefCell::new(llvm_module_name.to_string()),
            current_contract_name: RefCell::new("main".to_string()),
            main_module: RefCell::new("".to_string()),
            llvm_context: context,
            llvm_module: module,
            builder: RefCell::new(context.create_builder()),
            functions: RefCell::new(vec![]),
            current_function: RefCell::new(None),
            current_function_definition: RefCell::new(None),
            cur_func_exit_bb: RefCell::new(None),
            control_flow_blocks_end_bbs: RefCell::new(vec![]),
            control_flow_blocks_continue_bbs: RefCell::new(vec![]),
            vars_scopes: RefCell::new(vec![]),
            functions_mapping: RefCell::new(Default::default()),
            current_func_decls: RefCell::new(Default::default()),
            revert_zero_functions: RefCell::new(Default::default()),
            yul_func_infer_types: RefCell::new(Default::default()),
            iden_id_gen: RefCell::new(0),
            exported_func_names: RefCell::new(vec![]),
            wasm_start_inited: RefCell::new(false),
            global_bytes_lengths: RefCell::new(Default::default()),
            global_bytes_values: RefCell::new(Default::default()),
            variable_usage: RefCell::new(HashMap::new()),
            // memptr_global: RefCell::new(None),
            default_ret_type: YulLowLevelValueType::U256,
        }
    }

    pub fn emit(&mut self, output_basename: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        self.emit_code(output_basename)?;
        self.link_code()
    }

    fn run_llvm_passes(&self) {
        let opt_level = self.opts.opt_level.to_inkwell_optimization_level();
        let pass_manager_builder = PassManagerBuilder::create();
        pass_manager_builder.set_optimization_level(opt_level);
        let mpm = PassManager::create(());

        if self.opts.minify_wasm_size {
            // disable inliner
            pass_manager_builder.set_inliner_with_threshold(0);
        } else {
            // Inliner passes
            if !self.opts.no_inline {
                pass_manager_builder.set_inliner_with_threshold(512);
                mpm.add_always_inliner_pass();
            }
            if !self.opts.disable_all_optimizers {
                // Unroll and jam loops to improve performance
                mpm.add_loop_unroll_and_jam_pass();
                // Unroll loops to reduce loop overhead and enable other optimizations
                mpm.add_loop_unroll_pass();
            }
        }

        if !self.opts.disable_all_optimizers {
            // Loop optimization passes

            // Rotate loops to make them more amenable to optimizations
            mpm.add_loop_rotate_pass();
            // Transform loops into vector operations where possible
            mpm.add_loop_vectorize_pass();
            // Replace loop patterns with library calls or simpler forms
            mpm.add_loop_idiom_pass();
            // Remove loops with no side effects
            mpm.add_loop_deletion_pass();
            // Reroll loops to reduce code size when beneficial
            mpm.add_loop_reroll_pass();

            // if you want to see the original llvm ir, disable all optimizers

            // Additional aggressive optimizations for release mode
            mpm.add_aggressive_dce_pass();

            // Memory optimization passes
            mpm.add_promote_memory_to_register_pass();
            mpm.add_merged_load_store_motion_pass();
            mpm.add_demote_memory_to_register_pass();
            mpm.add_memcpy_optimize_pass();
            mpm.add_aggressive_dce_pass();
            mpm.add_licm_pass();
            mpm.add_gvn_pass();
            mpm.add_new_gvn_pass();
            mpm.add_sccp_pass();
            mpm.add_instruction_combining_pass();
            mpm.add_instruction_simplify_pass();
            mpm.add_jump_threading_pass();
            mpm.add_cfg_simplification_pass();
            mpm.add_tail_call_elimination_pass();
            mpm.add_reassociate_pass();
            mpm.add_early_cse_pass();
            mpm.add_early_cse_mem_ssa_pass();
            mpm.add_correlated_value_propagation_pass();
            mpm.add_dead_store_elimination_pass();
            mpm.add_ind_var_simplify_pass();
            mpm.add_lower_expect_intrinsic_pass();
            mpm.add_merge_functions_pass();
            mpm.add_strip_dead_prototypes_pass();

            mpm.add_scalarizer_pass();
        }

        // Populate the module pass manager with the passes configured in the pass manager builder
        pass_manager_builder.populate_module_pass_manager(&mpm);

        if !(mpm.run_on(&self.llvm_module.borrow_mut())) {
            if self.opts.verbose {
                let output_dir = &self.opts.output_dir;
                // Output the LLVM IR file in case of an error
                let ll_filepath = &format!("{output_dir}/error.out.ll");

                self.llvm_module
                    .borrow()
                    .print_to_file(ll_filepath)
                    .unwrap();
            }
            panic!("Failed to run llvm passes: ");
        }
    }

    fn emit_code(&mut self, output_basename: &str) -> Result<String, Box<dyn Error>> {
        if let Err(e) = self.transform() {
            return Err(format!("Transform error: {}", e).into());
        }

        self.transform().unwrap();
        // Run LLVM pass on the LLVM module.
        self.run_llvm_passes();
        if self.opts.verbose {
            let output_dir = &self.opts.output_dir;
            let ll_filepath = &format!(
                "{output_dir}/{}_{}.out.ll",
                self.opts.main_contract_name, output_basename
            );

            self.llvm_module
                .borrow()
                .print_to_file(ll_filepath)
                .unwrap();
            // this need llc installed for user
            if self.opts.use_llvm_toolchain {
                let ret = std::process::Command::new("llc")
                    .stdout(std::process::Stdio::inherit())
                    .stderr(std::process::Stdio::inherit())
                    .args([ll_filepath])
                    .output()
                    .expect("llc failed");
                if !ret.status.success() {
                    return Err(String::from_utf8(ret.stderr).unwrap().into());
                }
            }
        }
        self.llvm_module.borrow_mut().verify().unwrap();

        Ok(self.llvm_module.borrow().print_to_string().to_string())
    }

    fn llvm_target_name(&self) -> &'static str {
        "wasm32"
    }
    /// LLVM Target triple
    fn llvm_target_triple(&self) -> TargetTriple {
        TargetTriple::create("wasm32-unknown-unknown-wasm")
    }

    /// LLVM Target triple
    fn llvm_features(&self) -> &'static str {
        ""
    }

    pub fn link_code(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
        let target = inkwell::targets::Target::from_name(self.llvm_target_name()).unwrap();
        let level = inkwell::OptimizationLevel::Default;
        let target_machine = target
            .create_target_machine(
                &self.llvm_target_triple(),
                "",
                self.llvm_features(),
                level,
                RelocMode::Default,
                CodeModel::Default,
            )
            .unwrap();

        let has_sub_contract = has_sub_contract(&self.yul_ast.clone().unwrap());

        match target_machine.write_to_memory_buffer(&self.llvm_module.borrow(), FileType::Object) {
            Ok(out) => {
                let slice = out.as_slice();
                let export_names = self.exported_func_names.borrow().clone();
                let bs = crate::yul2ir::wasm::link(
                    slice,
                    "wasm_module",
                    &export_names,
                    self.opts,
                    has_sub_contract,
                );

                Ok(bs)
            }
            Err(s) => Err(s.to_string().into()),
        }
    }

    fn lookup_function(&self, name: &str) -> FunctionValue<'ctx> {
        self.llvm_module
            .borrow()
            .get_function(name)
            .unwrap_or_else(|| panic!("known function '{name}' is not found"))
    }

    pub fn build_void_call(
        &self,
        name: &str,
        args: &[BasicValueEnum<'ctx>],
    ) -> Result<(), ASTLoweringError> {
        let args: Vec<BasicMetadataValueEnum<'ctx>> = args.iter().map(|v| (*v).into()).collect();
        let res = self
            .builder
            .borrow_mut()
            .build_call(self.lookup_function(name), &args, "");
        match res {
            Ok(_) => Ok(()),
            Err(err) => Err(err.into()),
        }
    }

    pub fn build_call(
        &self,
        name: &str,
        args: &[BasicValueEnum<'ctx>],
    ) -> Result<BasicValueEnum<'ctx>, ASTLoweringError> {
        let args: Vec<BasicMetadataValueEnum> = args.iter().map(|v| (*v).into()).collect();
        let res = self
            .builder
            .borrow_mut()
            .build_call(self.lookup_function(name), &args, "");
        match res {
            Ok(res) => {
                if let Some(res) = res.try_as_basic_value().left() {
                    Ok(res)
                } else {
                    Err(ASTLoweringError::FunctionReturnValueNotFound(format!(
                        "{FUNCTION_RETURN_VALUE_NOT_FOUND_MSG}: {name}"
                    )))
                }
            }
            Err(err) => {
                eprintln!("{err}");
                panic!("function not found: {name}")
            }
        }
    }

    #[allow(unused)]
    pub fn keep_instruction(&self, value: BasicValueEnum<'ctx>) {
        // create a virtual use, ensure the instruction is used
        let dummy_value = self
            .builder
            .borrow_mut()
            .build_alloca(value.get_type(), "dummy_alloc_for_keep")
            .unwrap();
        self.builder
            .borrow_mut()
            .build_store(dummy_value, value)
            .unwrap();
    }

    pub fn parse_ty_name(&self, ty_name: &Option<TypeName>) -> BasicTypeEnum<'ctx> {
        self.parse_ty_name_or_default(ty_name, self.default_primitive_type())
    }

    pub fn parse_ty_name_or_default(
        &self,
        ty_name: &Option<TypeName>,
        default_type: BasicTypeEnum<'ctx>,
    ) -> BasicTypeEnum<'ctx> {
        match ty_name {
            Some(ty) => self.parse_ty_name_str(&ty.type_name.name).unwrap(),
            // https://docs.soliditylang.org/en/latest/yul.html#motivation-and-high-level-description
            // Currently, there is only one specified dialect of Yul. This dialect uses the EVM opcodes as builtin functions
            // (see below) and defines only the type u256, which is the native 256-bit type of the EVM.
            // Because of that, we will not provide types in the examples below.
            // But use bytes32(big endian u256) as default type will have better performance
            None => default_type,
        }
    }

    pub fn parse_ty_name_str(
        &self,
        ty_name: &str,
    ) -> Result<BasicTypeEnum<'ctx>, ASTLoweringError> {
        match ty_name {
            "bool" => Ok(self.bool_type().into()),
            "u8" => Ok(self.i8_type().into()),
            "u32" => Ok(self.i32_type().into()),
            "u64" => Ok(self.i64_type().into()),
            "u256" | "uint256" => Ok(self.u256_type().into()),
            "bytes32" => Ok(self.u256_type().into()), // bytes32 is stored as u256 in EVM
            "address" => Ok(self.u256_type().into()), // address is u160 but stored as u256 in EVM
            _ => Err(ASTLoweringError::UnsupportedType(ty_name.to_string())),
        }
    }

    pub fn next_iden_id(&self) -> u32 {
        self.iden_id_gen.borrow_mut().add(1)
    }

    pub fn bool_literal(&self, value: bool) -> IntValue<'ctx> {
        self.bool_type().const_int(if value { 1 } else { 0 }, true)
    }

    #[allow(unused)]
    pub fn hex_literal(&self, value: &str) -> IntValue<'ctx> {
        // TODO: Need to generate based on whether the type is u256 or some other type
        let value_without_prefix: String = value.chars().skip(2).collect();
        self.u256_type()
            .const_int_from_string(&value_without_prefix, StringRadix::Hexadecimal)
            .unwrap()
    }

    /// If the value is larger than i32 range, return bytes32 pointer
    #[allow(unused)]
    pub fn hex_literal_or_bytes32_literal(&self, value: &str) -> YulLowLevelValue<'ctx> {
        let u256_const = U256::from_str_radix(value, 16).unwrap();
        if u256_const > U256::from(i32::MAX) {
            // Must convert to 32 bytes, otherwise it is not convenient to use
            let mut value_bytes32 = [0u8; 32];
            u256_const.to_big_endian(&mut value_bytes32);
            let global_constant_name = format!("global_constant_{}", value);
            let global_value =
                self.add_global_constant_bytes32(&value_bytes32, &global_constant_name);
            let global_value_pointer = global_value.as_pointer_value();
            YulLowLevelValue {
                value_type: YulLowLevelValueType::Bytes32Pointer,
                value: global_value_pointer.into(),
            }
        } else {
            let value_int = self.hex_literal(value);
            YulLowLevelValue {
                value: value_int.as_basic_value_enum(),
                value_type: YulLowLevelValueType::from_int_type(value_int.get_type()),
            }
        }
    }

    pub fn dec_literal(&self, value: &str, expected_type: ExpectedType) -> IntValue<'ctx> {
        // Unless there's a clear expected type from context, or the variable is not used as u256 type,
        // always use u256 type. Otherwise, in cases like `let i := 0` followed by `i = add(i, 1)`,
        // overflow might occur within the i32 value range.

        // If the expected type is Untyped, choose i32, i64, or i256 based on the literal's size
        // If an expected type is provided, use that type instead
        match expected_type {
            // i32 is the most priority literal int type
            ExpectedType::I32 => {
                if value.len() <= 9 {
                    self.i32_type()
                        .const_int_from_string(value, StringRadix::Decimal)
                        .unwrap()
                } else if value.len() <= 18 {
                    self.i64_type()
                        .const_int_from_string(value, StringRadix::Decimal)
                        .unwrap()
                } else {
                    self.u256_type()
                        .const_int_from_string(value, StringRadix::Decimal)
                        .unwrap()
                }
            }
            ExpectedType::I64 => {
                if value.len() <= 18 {
                    self.i64_type()
                        .const_int_from_string(value, StringRadix::Decimal)
                        .unwrap()
                } else {
                    self.u256_type()
                        .const_int_from_string(value, StringRadix::Decimal)
                        .unwrap()
                }
            }
            _ => self
                .u256_type()
                .const_int_from_string(value, StringRadix::Decimal)
                .unwrap(),
        }
    }

    pub fn string_literal(&self, value: &str) -> IntValue<'ctx> {
        if value.len() > 31 {
            // panic!("String literal length exceeds 31 bytes: {}", value);
            // String literals used in linker symbol, data offset, and data size instructions
            // don't need to be processed by string_literal function
            // After modifying the logic in instruction.rs, we can restore the panic here
            return self.u256_type().const_zero();
        }
        // Convert string to bytes and pad to 32 bytes
        let mut bytes = value.as_bytes().to_vec();
        // right pad to 32 bytes
        bytes.resize(32, 0);
        // Decode as big endian to u256
        let mut be_bytes = [0u8; 32];
        be_bytes.copy_from_slice(&bytes);
        let be_bytes_hex = hex::encode(be_bytes);
        let int_value = self
            .u256_type()
            .const_int_from_string(&be_bytes_hex, StringRadix::Hexadecimal)
            .unwrap();
        int_value
    }

    #[allow(unused)]
    pub fn fetch_not_string_literal_constant(&self, expr: &Expression) -> Option<U256> {
        match expr {
            Expression::Literal(Literal::DecimalNumberLiteral(dec, _)) => {
                Some(U256::from_str_radix(&dec.dec, 10).unwrap())
            }
            Expression::Literal(Literal::HexNumberLiteral(hex, _)) => {
                Some(U256::from_str_radix(&hex.hex, 16).unwrap())
            }
            Expression::Literal(Literal::TrueLiteral(_)) => Some(U256::from(1)),
            Expression::Literal(Literal::FalseLiteral(_)) => Some(U256::from(0)),
            _ => None,
        }
    }

    #[allow(unused)]
    pub fn matches_constant_literal(&self, expr: &Expression, value: U256) -> bool {
        let constant_value = self.fetch_not_string_literal_constant(expr);
        if let Some(constant_value) = constant_value {
            constant_value == value
        } else {
            false
        }
    }

    #[allow(unused)]
    pub fn matches_yul_instruction(
        &self,
        expr: &Expression,
        opcode_name: &str,
        args_count: usize,
    ) -> Option<Vec<Expression>> {
        match expr {
            Expression::FunctionCall(func_call) => {
                if func_call.id.name == opcode_name && func_call.arguments.len() == args_count {
                    Some(func_call.arguments.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    #[allow(unused)]
    pub fn matches_single_declare_instruction(
        &self,
        stmt: &Statement,
    ) -> Option<(String, Expression)> {
        match stmt {
            Statement::VariableDeclaration(variable_declaration) => {
                if variable_declaration.identifiers.len() == 1
                    && variable_declaration.value.is_some()
                {
                    Some((
                        variable_declaration.identifiers[0].identifier.name.clone(),
                        variable_declaration.value.as_ref().unwrap().clone(),
                    ))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    #[allow(unused)]
    pub fn matches_mstore_statement(&self, stmt: &Statement) -> Option<Expression> {
        match stmt {
            Statement::FunctionCall(func_call) => {
                if func_call.id.name == "mstore" && func_call.arguments.len() == 2 {
                    Some(func_call.arguments[0].clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    #[allow(unused)]
    pub fn matches_sstore_statement(&self, stmt: &Statement) -> Option<Expression> {
        match stmt {
            Statement::FunctionCall(func_call) => {
                if func_call.id.name == "sstore" && func_call.arguments.len() == 2 {
                    Some(func_call.arguments[0].clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn matches_function_call(
        &self,
        stmt: &Statement,
        name: &str,
        args_count: usize,
    ) -> Option<Vec<Expression>> {
        match stmt {
            Statement::FunctionCall(func_call) => {
                if func_call.id.name == name && func_call.arguments.len() == args_count {
                    Some(func_call.arguments.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    #[allow(unused)]
    pub fn is_yul_function_call(&self, expr: &Expression) -> bool {
        match expr {
            Expression::FunctionCall(func_call) => {
                let func_name = func_call.id.name.clone();
                parse_intrinsic_func_name(&func_name).is_none()
            }
            _ => false,
        }
    }

    /// Helper function to match the address masking pattern: and(addr, sub(shl(160, 1), 1))
    /// return the addr expression
    pub fn matches_address_mask_pattern(&self, expr: &Expression) -> Option<Expression> {
        // Pattern: and(addr, sub(shl(160, 1), 1))
        if let Some(and_args) = self.matches_yul_instruction(expr, "and", 2) {
            if let Some(sub_args) = self.matches_yul_instruction(&and_args[1], "sub", 2) {
                if let Some(shl_args) = self.matches_yul_instruction(&sub_args[0], "shl", 2) {
                    // Check if shl(160, 1) and sub(..., 1)
                    if self.matches_constant_literal(&shl_args[0], U256::from(160))
                        && self.matches_constant_literal(&shl_args[1], U256::from(1))
                        && self.matches_constant_literal(&sub_args[1], U256::from(1))
                    {
                        return Some(and_args[0].clone());
                    }
                }
            }
        }
        None
    }

    pub fn add_global_constant_bytes32(
        &self,
        value: &[u8],
        global_constant_name: &str,
    ) -> GlobalValue<'ctx> {
        let data_array_type = self.llvm_context.i8_type().array_type(value.len() as u32);
        let data_array = self.llvm_context.const_string(value, false);
        let global_var =
            self.llvm_module
                .borrow_mut()
                .add_global(data_array_type, None, global_constant_name);
        global_var.set_initializer(&data_array);
        global_var
    }

    pub fn enter_scope(&self) {
        self.vars_scopes.borrow_mut().push(VarScope::new())
    }

    pub fn exit_scope(&self) {
        self.vars_scopes.borrow_mut().pop().unwrap();
    }

    pub fn set_var(
        &self,
        iden: &str,
        var_ty: BasicTypeEnum<'ctx>,
        var_low_level_value_type: YulLowLevelValueType,
        var_pointer: PointerValue<'ctx>,
        is_return_var: bool,
    ) -> Result<(), ASTLoweringError> {
        if let Some(last_var_scope) = self.vars_scopes.borrow_mut().last_mut() {
            if last_var_scope.vars.borrow().contains_key(iden) {
                return Err(ASTLoweringError::DuplicateVariableDefinition(format!(
                    "Variable '{}' is already defined in this scope",
                    iden
                )));
            }

            last_var_scope.vars.borrow_mut().insert(
                iden.to_string(),
                (var_ty, var_low_level_value_type, var_pointer, is_return_var),
            );
        }
        Ok(())
    }

    // return (var_type, var_low_level_value_type, var_pointer, is_return_var)
    pub fn get_var(
        &self,
        iden: &str,
    ) -> Option<(
        BasicTypeEnum<'ctx>,
        YulLowLevelValueType,
        PointerValue<'ctx>,
        bool,
    )> {
        for scope in self.vars_scopes.borrow().iter().rev() {
            if let Some(item) = scope.vars.borrow().get(iden) {
                return Some(*item);
            }
        }
        None
    }
}

/// some utils functions to wrapper inkwell APIs
impl<'ctx> Yul2IRContext<'ctx> {
    pub fn default_primitive_type(&self) -> BasicTypeEnum<'ctx> {
        // Must be consistent with default_primitive_type_low_level
        self.u256_type().into()
    }

    #[allow(unused)]
    pub fn default_primitive_type_low_level(&self) -> YulLowLevelValueType {
        // Must be consistent with default_primitive_type
        YulLowLevelValueType::U256
    }

    pub fn default_param_type(&self) -> BasicTypeEnum<'ctx> {
        self.default_primitive_type()
    }

    #[allow(unused)]
    pub fn default_param_expected_type(&self) -> ExpectedType {
        ExpectedType::U256
    }

    pub fn default_func_return_element_type(&self) -> BasicTypeEnum<'ctx> {
        match self.default_ret_type {
            YulLowLevelValueType::Bytes32Pointer => {
                self.bytes32_pointer_type().as_basic_type_enum()
            }
            _ => self.u256_type().as_basic_type_enum(),
        }
    }

    pub fn default_func_return_low_level_value_type(&self) -> YulLowLevelValueType {
        self.default_ret_type
    }

    pub fn u256_type(&self) -> IntType<'ctx> {
        self.llvm_context.custom_width_int_type(256)
    }

    pub fn i64_type(&self) -> IntType<'ctx> {
        self.llvm_context.i64_type()
    }

    pub fn i32_type(&self) -> IntType<'ctx> {
        self.llvm_context.i32_type()
    }

    pub fn i8_type(&self) -> IntType<'ctx> {
        self.llvm_context.i8_type()
    }

    pub fn bool_type(&self) -> IntType<'ctx> {
        self.llvm_context.bool_type()
    }

    pub fn void_type(&self) -> VoidType<'ctx> {
        self.llvm_context.void_type()
    }

    pub fn bytes32_type(&self) -> ArrayType<'ctx> {
        self.i8_type().array_type(32)
    }

    pub fn bytes32_pointer_type(&self) -> PointerType<'ctx> {
        // now llvm16+ only support directly ptr type
        self.llvm_context.ptr_type(AddressSpace::default())
    }

    pub fn int_to_bool(
        &self,
        int_value: IntValue<'ctx>,
    ) -> Result<IntValue<'ctx>, ASTLoweringError> {
        let zero: IntValue<'ctx> = int_value.get_type().const_zero();
        let res = self.builder.borrow_mut().build_int_compare(
            IntPredicate::NE,
            int_value,
            zero,
            "int_to_bool",
        )?;
        Ok(res)
    }

    pub fn int_as_u256(
        &self,
        int_value: IntValue<'ctx>,
    ) -> Result<IntValue<'ctx>, ASTLoweringError> {
        self.int_cast(int_value, self.u256_type())
    }

    pub fn try_into_int(
        &self,
        value: &BasicValueEnum<'ctx>,
    ) -> Result<IntValue<'ctx>, ASTLoweringError> {
        if value.get_type().is_int_type() {
            Ok(value.into_int_value())
        } else if self.is_bytes32_value(value) {
            self.bytes32_as_u256(value)
        } else if value.is_pointer_value() {
            // is bytes32 pointer
            self.bytes32_pointer_as_u256(value)
        } else {
            Err(ASTLoweringError::UnsupportedType(
                value.get_type().to_string(),
            ))
        }
    }

    pub fn try_into_i64(
        &self,
        value: &BasicValueEnum<'ctx>,
    ) -> Result<IntValue<'ctx>, ASTLoweringError> {
        if value.get_type().is_int_type() {
            self.int_cast(value.into_int_value(), self.i64_type())
        } else if self.is_bytes32_value(value) {
            let u256_value = self.bytes32_as_u256(value)?;
            self.int_cast(u256_value, self.i64_type())
        } else if value.is_pointer_value() {
            // is bytes32 pointer
            let bytes32_val = self.try_into_bytes32(value)?;
            self.try_into_i64(&bytes32_val)
        } else {
            Err(ASTLoweringError::UnsupportedType(
                value.get_type().to_string(),
            ))
        }
    }

    pub fn try_into_i32_value(
        &self,
        value: &BasicValueEnum<'ctx>,
        value_expr: &Expression,
    ) -> Result<IntValue<'ctx>, ASTLoweringError> {
        // If it's a small constant
        if let Some(constant_value) = self.fetch_not_string_literal_constant(value_expr) {
            if constant_value <= U256::from(i32::MAX) {
                Ok(self.i32_type().const_int(constant_value.as_u64(), false))
            } else {
                self.try_into_i32_across_int(value)
            }
        } else {
            self.try_into_i32_across_int(value)
        }
    }

    pub fn try_into_i32_across_int(
        &self,
        value: &BasicValueEnum<'ctx>,
    ) -> Result<IntValue<'ctx>, ASTLoweringError> {
        // This function is equivalent to try_into_int followed by int_cast to i32
        // For bytes32 type, we can directly take the last 4 bytes and convert to i32 using big endian
        // For bytes32 pointer type, we can first convert to u8*, then take the content starting from offset +28 and convert to i32 using big endian
        if self.is_bytes32_value(value) {
            let value_ptr = self.get_value_pointer(*value)?;
            let i32_val = self.build_call("i32_from_big_endian_bytes32", &[value_ptr.into()])?;
            Ok(i32_val.into_int_value())
        } else if value.is_pointer_value() {
            // is bytes32 pointer
            let i32_val = self.build_call("i32_from_big_endian_bytes32", &[*value])?;
            Ok(i32_val.into_int_value())
        } else {
            self.int_as_i32(self.try_into_int(value)?)
        }
    }

    pub fn try_into_i32(
        &self,
        value: &BasicValueEnum<'ctx>,
    ) -> Result<IntValue<'ctx>, ASTLoweringError> {
        if value.get_type().is_int_type() {
            self.int_as_i32(value.into_int_value())
        } else if self.is_bytes32_value(value) {
            let u256_value = self.bytes32_as_u256(value)?;
            self.int_cast(u256_value, self.i32_type())
        } else if value.is_pointer_value() {
            // is bytes32 pointer
            let bytes32_val = self.try_into_bytes32(value)?;
            self.try_into_i32(&bytes32_val)
        } else {
            Err(ASTLoweringError::UnsupportedType(
                value.get_type().to_string(),
            ))
        }
    }

    pub fn try_into_u256(
        &self,
        value: &BasicValueEnum<'ctx>,
    ) -> Result<IntValue<'ctx>, ASTLoweringError> {
        if value.get_type().is_int_type() {
            self.int_as_u256(value.into_int_value())
        } else if self.is_bytes32_value(value) {
            self.bytes32_as_u256(value)
        } else if value.is_pointer_value() {
            // is bytes32 pointer
            let bytes32_val = self.try_into_bytes32(value)?;
            self.bytes32_as_u256(&bytes32_val)
        } else {
            Err(ASTLoweringError::UnsupportedType(
                value.get_type().to_string(),
            ))
        }
    }

    pub fn is_bytes32_value(&self, value: &BasicValueEnum<'ctx>) -> bool {
        self.is_bytes32_type(&value.get_type())
    }

    pub fn is_bytes32_pointer_value(&self, value: &BasicValueEnum<'ctx>) -> bool {
        // Currently the only pointer type is bytes32 pointer
        value.is_pointer_value()
    }

    pub fn is_u256_type(&self, value: &BasicTypeEnum<'ctx>) -> bool {
        value.is_int_type() && value.into_int_type().get_bit_width() == 256
    }

    pub fn is_i32_type(&self, value: &BasicTypeEnum<'ctx>) -> bool {
        value.is_int_type() && value.into_int_type().get_bit_width() == 32
    }

    pub fn is_int32_value(&self, value: &BasicValueEnum<'ctx>) -> bool {
        self.is_i32_type(&value.get_type())
    }

    pub fn is_bytes32_type(&self, value: &BasicTypeEnum<'ctx>) -> bool {
        if value.is_array_type() {
            let array_type = value.into_array_type();
            array_type.get_element_type().is_int_type()
                && array_type
                    .get_element_type()
                    .into_int_type()
                    .get_bit_width()
                    == 8
                && array_type.is_sized()
                && array_type.len() == 32
        } else {
            false
        }
    }

    pub fn try_into_bytes32_pointer(
        &self,
        value: &BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, ASTLoweringError> {
        if self.is_bytes32_value(value) {
            Ok(self.get_value_pointer(*value)?.into())
        } else if value.is_pointer_value() {
            // is bytes32 pointer
            Ok(*value)
        } else if self.is_i32_type(&value.get_type()) {
            // call i32_to_bytes32_big_endian_bytes
            let ret_ty = self.bytes32_type();
            let ret_ptr = self.fast_alloca(ret_ty, "")?;
            self.build_void_call("i32_to_bytes32_big_endian_bytes", &[*value, ret_ptr.into()])?;
            Ok(ret_ptr.into())
        } else if self.is_u256_type(&value.get_type()) {
            // call u256_to_big_endian_bytes
            let u256_ptr = self.get_value_pointer(*value)?;
            let ret_ty = self.bytes32_type();
            let ret_ptr = self.fast_alloca(ret_ty, "")?;
            self.build_void_call(
                "u256_to_big_endian_bytes",
                &[u256_ptr.into(), ret_ptr.into()],
            )?;
            Ok(ret_ptr.into())
        } else if value.is_int_value() {
            let u256_value = self.int_as_u256(value.into_int_value())?;
            self.try_into_bytes32_pointer(&u256_value.into())
        } else {
            Ok(self
                .get_value_pointer(self.try_into_bytes32(value)?)?
                .into())
        }
    }

    pub fn try_into_bytes32(
        &self,
        value: &BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, ASTLoweringError> {
        if self.is_bytes32_value(value) {
            // already bytes32 type
            Ok(*value)
        } else if value.is_pointer_value() {
            // is bytes32 pointer
            let value = self.build_load(self.bytes32_type(), value.into_pointer_value(), "")?;
            Ok(value)
        } else {
            let u256_value = self.try_into_u256(value)?;
            Ok(self.u256_to_bytes32(u256_value)?)
        }
    }

    pub fn int_as_i32(
        &self,
        int_value: IntValue<'ctx>,
    ) -> Result<IntValue<'ctx>, ASTLoweringError> {
        self.int_cast(int_value, self.i32_type())
    }

    pub fn int_as_i64(
        &self,
        int_value: IntValue<'ctx>,
    ) -> Result<IntValue<'ctx>, ASTLoweringError> {
        self.int_cast(int_value, self.i64_type())
    }

    pub fn bytes32_as_u256(
        &self,
        value: &BasicValueEnum<'ctx>,
    ) -> Result<IntValue<'ctx>, ASTLoweringError> {
        if self.is_bytes32_value(value) {
            // call u256_from_big_endian_bytes(bytes32*, uint256_t*)
            let ret_ty = self.u256_type();
            let ret_ptr = self.fast_alloca(ret_ty, "")?;
            let value_ptr = self.get_value_pointer(*value)?;
            self.build_void_call(
                "u256_from_big_endian_bytes",
                &[value_ptr.into(), ret_ptr.into()],
            )?;
            let res = self.build_load(ret_ty, ret_ptr, "")?;
            Ok(res.into_int_value())
        } else {
            Err(ASTLoweringError::UnsupportedType(
                value.get_type().to_string(),
            ))
        }
    }

    pub fn bytes32_pointer_as_u256(
        &self,
        value: &BasicValueEnum<'ctx>,
    ) -> Result<IntValue<'ctx>, ASTLoweringError> {
        if self.is_bytes32_pointer_value(value) {
            // call u256_from_big_endian_bytes(bytes32*, uint256_t*)
            let ret_ty = self.u256_type();
            let ret_ptr = self.fast_alloca(ret_ty, "")?;
            self.build_void_call("u256_from_big_endian_bytes", &[*value, ret_ptr.into()])?;
            let res = self.build_load(ret_ty, ret_ptr, "")?;
            Ok(res.into_int_value())
        } else {
            Err(ASTLoweringError::UnsupportedType(
                value.get_type().to_string(),
            ))
        }
    }

    pub fn u256_to_bytes32(
        &self,
        value: IntValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, ASTLoweringError> {
        if self.is_u256_type(&value.get_type().as_basic_type_enum()) {
            let value_ptr = self.get_value_pointer(value)?;
            let ret_ty = self.bytes32_type();
            let ret_ptr = self.fast_alloca(ret_ty, "")?;
            self.build_void_call(
                "u256_to_big_endian_bytes",
                &[value_ptr.into(), ret_ptr.into()],
            )?;
            let res = self.build_load(ret_ty, ret_ptr, "")?;
            Ok(res)
        } else {
            Err(ASTLoweringError::UnsupportedType(
                value.get_type().to_string(),
            ))
        }
    }

    pub fn try_cast(
        &self,
        value: BasicValueEnum<'ctx>,
        target_ty: BasicTypeEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, ASTLoweringError> {
        if value.get_type() == target_ty {
            Ok(value)
        } else if self.is_bytes32_value(&value) && target_ty.is_int_type() {
            // bytes32 to u256, then cast to target_ty
            let u256_value = self.bytes32_as_u256(&value)?;
            let result = self.int_cast(u256_value, target_ty.into_int_type())?;
            Ok(result.into())
        } else if value.is_int_value() && self.is_bytes32_type(&target_ty) {
            // int to bytes32
            let int_value = value.into_int_value();
            let u256_value = self.int_as_u256(int_value)?;
            Ok(self.u256_to_bytes32(u256_value)?)
        } else if value.is_pointer_value() {
            // Check if value is a bytes32 pointer
            // If converting to u256: can directly convert from bytes32 pointer to u256
            if self.is_u256_type(&target_ty) {
                // call u256_from_big_endian_bytes(bytes32*, uint256_t*)
                let ret_ty = self.u256_type();
                let ret_ptr = self.fast_alloca(ret_ty, "")?;
                self.build_void_call("u256_from_big_endian_bytes", &[value, ret_ptr.into()])?;
                let res = self.build_load(ret_ty, ret_ptr, "")?;
                return Ok(res);
            }
            // If converting to i32, we can directly convert from bytes32 pointer to i32
            if self.is_i32_type(&target_ty) {
                // call i32_from_big_endian_bytes32(bytes32*)
                let result = self.build_call("i32_from_big_endian_bytes32", &[value])?;
                return Ok(result);
            }
            if self.is_bytes32_type(&target_ty) {
                let result =
                    self.build_load(self.bytes32_type(), value.into_pointer_value(), "")?;
                return Ok(result);
            }

            // For other cases, first convert to u256 then to target type
            let u256_type = self.u256_type();
            let u256_ptr = self.fast_alloca(u256_type, "")?;
            self.build_void_call("u256_from_big_endian_bytes", &[value, u256_ptr.into()])?;
            let u256_value = self.build_load(u256_type, u256_ptr, "")?;
            self.try_cast(u256_value, target_ty)
        } else if target_ty.is_pointer_type() {
            // to bytes32 pointer
            let bytes32_pointer = self.try_into_bytes32_pointer(&value)?;
            Ok(bytes32_pointer)
        } else {
            // int[M] to int[N]
            let int_value = self.int_cast(value.into_int_value(), target_ty.into_int_type())?;
            Ok(int_value.into())
        }
    }

    pub fn int_cast(
        &self,
        int_value: IntValue<'ctx>,
        target_int_ty: IntType<'ctx>,
    ) -> Result<IntValue<'ctx>, ASTLoweringError> {
        if int_value.get_type().get_bit_width() == target_int_ty.get_bit_width() {
            return Ok(int_value);
        }
        if int_value.get_type().get_bit_width() < target_int_ty.get_bit_width() {
            return self
                .builder
                .borrow_mut()
                .build_int_z_extend(int_value, target_int_ty, "")
                .map_err(|e| ASTLoweringError::BuilderError(e.to_string()));
        }
        self.builder
            .borrow_mut()
            .build_int_cast(int_value, target_int_ty, "")
            .map_err(|e| ASTLoweringError::BuilderError(e.to_string()))
    }

    #[allow(unused)]
    pub fn unify_ints3(
        &self,
        int_value1: IntValue<'ctx>,
        int_value2: IntValue<'ctx>,
        int_value3: IntValue<'ctx>,
    ) -> Result<(IntValue<'ctx>, IntValue<'ctx>, IntValue<'ctx>), ASTLoweringError> {
        let (int_value1, int_value2) = self.unify_ints(int_value1, int_value2)?;
        let (int_value1, int_value3) = self.unify_ints(int_value1, int_value3)?;
        Ok((int_value1, int_value2, int_value3))
    }

    pub fn unify_to_u256(
        &self,
        int_value1: IntValue<'ctx>,
        int_value2: IntValue<'ctx>,
    ) -> Result<(IntValue<'ctx>, IntValue<'ctx>), ASTLoweringError> {
        let int_value1 = self.try_into_u256(&int_value1.into())?;
        let int_value2 = self.try_into_u256(&int_value2.into())?;
        Ok((int_value1, int_value2))
    }

    pub fn unify_to_bytes32<T: BasicValue<'ctx>>(
        &self,
        int_value1: &T,
        int_value2: &T,
    ) -> Result<(BasicValueEnum<'ctx>, BasicValueEnum<'ctx>), ASTLoweringError> {
        let int_value1 = self.try_into_bytes32(&int_value1.as_basic_value_enum())?;
        let int_value2 = self.try_into_bytes32(&int_value2.as_basic_value_enum())?;
        Ok((int_value1, int_value2))
    }

    pub fn unify_to_bytes32_pointer(
        &self,
        int_value1: &BasicValueEnum<'ctx>,
        int_value2: &BasicValueEnum<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, BasicValueEnum<'ctx>), ASTLoweringError> {
        let int_value1 = self.try_into_bytes32_pointer(int_value1)?;
        let int_value2 = self.try_into_bytes32_pointer(int_value2)?;
        Ok((int_value1, int_value2))
    }

    pub fn unify_ints(
        &self,
        int_value1: IntValue<'ctx>,
        int_value2: IntValue<'ctx>,
    ) -> Result<(IntValue<'ctx>, IntValue<'ctx>), ASTLoweringError> {
        let t1 = int_value1.get_type();
        let t2 = int_value2.get_type();
        if t1.get_bit_width() == t2.get_bit_width() {
            return Ok((int_value1, int_value2));
        }
        if t1.get_bit_width() > t2.get_bit_width() {
            let new_value2 = self.int_cast(int_value2, t1)?;
            return Ok((int_value1, new_value2));
        }
        // t1.get_bit_width() < t2.get_bit_width()
        let new_value1 = self.int_cast(int_value1, t2)?;
        Ok((new_value1, int_value2))
    }

    #[allow(unused)]
    pub fn memory_alloca_first<T: BasicType<'ctx>>(
        &self,
        ty: T,
        name: &str,
    ) -> Result<PointerValue<'ctx>, ASTLoweringError> {
        if self.is_bytes32_type(&ty.as_basic_type_enum()) {
            let ptr = self.build_call("memory_alloca_bytes32", &[])?;
            Ok(ptr.into_pointer_value())
        } else if self.is_u256_type(&ty.as_basic_type_enum()) {
            let ptr = self.build_call("memory_alloca_u256", &[])?;
            Ok(ptr.into_pointer_value())
        } else {
            self.build_alloca(ty, name)
        }
    }

    #[allow(unused)]
    pub fn fast_alloca<T: BasicType<'ctx>>(
        &self,
        ty: T,
        name: &str,
    ) -> Result<PointerValue<'ctx>, ASTLoweringError> {
        // if alloca bytes32, alloca it in memory_alloca_bytes32
        // else use build_alloca

        let enable_fast_alloca: bool = false; // TODO; enable when func return allocaed pointer
        if enable_fast_alloca {
            self.memory_alloca_first(ty, name)
        } else {
            self.build_alloca(ty, name)
        }
    }

    #[allow(unused)]
    pub fn build_alloca<T: BasicType<'ctx>>(
        &self,
        ty: T,
        name: &str,
    ) -> Result<PointerValue<'ctx>, ASTLoweringError> {
        self.builder
            .borrow_mut()
            .build_alloca(ty, name)
            .map_err(|e| ASTLoweringError::BuilderError(e.to_string()))
    }

    #[allow(dead_code)]
    pub fn build_store<T: BasicValue<'ctx>>(
        &self,
        ptr: PointerValue<'ctx>,
        value: T,
    ) -> Result<(), ASTLoweringError> {
        let res = self.builder.borrow_mut().build_store(ptr, value);
        match res {
            Ok(_) => Ok(()),
            Err(err) => Err(err.into()),
        }
    }

    pub fn build_load<T: BasicType<'ctx>>(
        &self,
        pointee_ty: T,
        ptr: PointerValue<'ctx>,
        name: &str,
    ) -> Result<BasicValueEnum<'ctx>, ASTLoweringError> {
        self.builder
            .borrow_mut()
            .build_load(pointee_ty, ptr, name)
            .map_err(|e| ASTLoweringError::BuilderError(e.to_string()))
    }

    pub fn get_value_pointer<T: BasicValue<'ctx>>(
        &self,
        value: T,
    ) -> Result<PointerValue<'ctx>, ASTLoweringError> {
        let value_ptr: PointerValue<'ctx> =
            self.fast_alloca(value.as_basic_value_enum().get_type(), "")?;
        let value: BasicValueEnum<'ctx> = value.as_basic_value_enum();
        self.build_store(value_ptr, value)?;
        Ok(value_ptr)
    }

    #[allow(unused)]
    pub fn is_constant_int_value(&self, value: &BasicValueEnum<'ctx>) -> bool {
        match value.as_basic_value_enum() {
            BasicValueEnum::IntValue(int_value) => int_value.is_const(),
            _ => false,
        }
    }
}
