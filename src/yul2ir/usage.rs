// Copyright (c) The Ant Group Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use super::{
    ast::{Block, Expression, FunctionDefinition, Statement, SwitchOptions},
    context::{UsageInfo, Yul2IRContext},
};

impl<'a> Yul2IRContext<'a> {
    /// Variable Usage Analysis Functions (Free Functions)
    #[allow(unused)]
    fn analyze_expression_usage(expr: &Expression, usage_info: &mut HashMap<String, UsageInfo>) {
        match expr {
            Expression::Identifier(id) => {
                // This is a read
                usage_info.entry(id.name.clone()).or_default().reads += 1;
            }
            Expression::FunctionCall(func_call) => {
                // Analyze arguments for reads
                for arg in &func_call.arguments {
                    Self::analyze_expression_usage(arg, usage_info);
                }
                // We don't analyze function names as variable reads here
            }
            // Literals don't involve variable reads
            Expression::Literal(_) => {}
        }
    }

    fn analyze_statement_usage(
        &self,
        stmt: &Statement,
        usage_info: &mut HashMap<String, UsageInfo>,
    ) {
        match stmt {
            Statement::Assignment(assign) => {
                // Increment write count for assigned variables
                for iden in &assign.identifiers {
                    usage_info.entry(iden.name.clone()).or_default().writes += 1;
                }
                // Analyze expression for reads
                Self::analyze_expression_usage(&assign.value, usage_info);
            }
            Statement::VariableDeclaration(decl) => {
                // Increment write count for declared variables
                for iden in &decl.identifiers {
                    usage_info
                        .entry(iden.identifier.name.clone())
                        .or_default()
                        .writes += 1;
                }
                // Analyze initializer expression for reads
                if let Some(value) = &decl.value {
                    Self::analyze_expression_usage(value, usage_info);
                }
            }
            Statement::If(if_stmt) => {
                Self::analyze_expression_usage(&if_stmt.cond, usage_info); // Fixed: Direct call
                self.analyze_block_usage(&if_stmt.body, usage_info); // Fixed: Direct call
            }
            Statement::For(for_stmt) => {
                self.analyze_block_usage(&for_stmt.init_block, usage_info); // Fixed: Direct call
                Self::analyze_expression_usage(&for_stmt.condition, usage_info); // Fixed: Direct call
                self.analyze_block_usage(&for_stmt.execution_block, usage_info); // Fixed: Direct call
                self.analyze_block_usage(&for_stmt.post_block, usage_info); // Fixed: Direct call
            }
            Statement::Switch(switch_stmt) => {
                Self::analyze_expression_usage(&switch_stmt.condition, usage_info); // Fixed: Direct call
                match &switch_stmt.opt {
                    SwitchOptions::Cases(cases, default_opt) => {
                        for case in cases {
                            // case.case is Literal, no reads in the case value itself
                            self.analyze_block_usage(&case.body, usage_info); // Fixed: Direct call
                        }
                        if let Some(default_case) = default_opt {
                            self.analyze_block_usage(&default_case.body, usage_info);
                            // Fixed: Direct call
                        }
                    }
                    SwitchOptions::Default(default_case) => {
                        self.analyze_block_usage(&default_case.body, usage_info);
                        // Fixed: Direct call
                    }
                }
            }
            Statement::Block(block) => {
                self.analyze_block_usage(block, usage_info);
            }
            Statement::FunctionCall(func_call) => {
                // Analyze the function call arguments as expressions
                // Fixed: Analyze args directly, not the Boxed FunctionCall expression itself
                for arg in &func_call.arguments {
                    Self::analyze_expression_usage(arg, usage_info);
                }
            }
            // Leave, Break, Continue, FunctionDefinition, Comment don't directly read/write variables
            // in the current scope in a way that affects this simple usage analysis.
            _ => {}
        }
    }

    #[allow(unused)]
    fn analyze_block_usage(&self, block: &Block, usage_info: &mut HashMap<String, UsageInfo>) {
        for stmt in &block.statements {
            self.analyze_statement_usage(stmt, usage_info);
        }
    }

    #[allow(unused)]
    pub fn analyze_function_usage(&self, func_def: &FunctionDefinition) {
        let func_qualifier = self.get_func_decl_qualifier_name(func_def);
        // Avoid re-analyzing if already done (e.g., if called multiple times)
        if self.variable_usage.borrow().contains_key(&func_qualifier) {
            return;
        }
        let mut usage_map = HashMap::new();
        self.analyze_block_usage(&func_def.body, &mut usage_map);
        self.variable_usage
            .borrow_mut()
            .insert(func_qualifier, usage_map.clone());
    }

    #[allow(unused)]
    pub fn get_variable_usage(
        &self,
        func_qualifier: &str,
        variable_name: &str,
    ) -> Option<UsageInfo> {
        self.variable_usage
            .borrow()
            .get(func_qualifier)
            .and_then(|map| map.get(variable_name))
            .cloned()
    }
}
