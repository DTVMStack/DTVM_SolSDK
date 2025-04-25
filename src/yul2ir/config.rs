// Copyright (C) 2024-2025 the DTVM authors, Ltd. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::fmt::Display;

// An enum type that can be mapped to inkwell OptimizationLevel and supports string conversion
#[derive(Debug, Clone, Default)]
pub enum OptimizationLevel {
    None,
    Less,
    #[default]
    Default,
    Aggressive,
}

impl OptimizationLevel {
    pub fn to_inkwell_optimization_level(&self) -> inkwell::OptimizationLevel {
        match self {
            OptimizationLevel::None => inkwell::OptimizationLevel::None,
            OptimizationLevel::Less => inkwell::OptimizationLevel::Less,
            OptimizationLevel::Default => inkwell::OptimizationLevel::Default,
            OptimizationLevel::Aggressive => inkwell::OptimizationLevel::Aggressive,
        }
    }
}

impl std::str::FromStr for OptimizationLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(OptimizationLevel::None),
            "less" => Ok(OptimizationLevel::Less),
            "default" => Ok(OptimizationLevel::Default),
            "aggressive" => Ok(OptimizationLevel::Aggressive),
            _ => Err(format!("Unknown optimization level: {}", s)),
        }
    }
}

impl Display for OptimizationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptimizationLevel::None => write!(f, "none"),
            OptimizationLevel::Less => write!(f, "less"),
            OptimizationLevel::Default => write!(f, "default"),
            OptimizationLevel::Aggressive => write!(f, "aggressive"),
        }
    }
}

/// Compile options.
#[derive(Debug, Clone)]
pub struct Yul2IROptions {
    pub output_dir: String,
    pub main_contract_name: String,
    pub verbose: bool,
    pub debug_mode: bool,
    pub opt_level: OptimizationLevel,
    pub no_inline: bool,
    pub use_llvm_toolchain: bool,
    pub no_binaryen_optimize: bool,
    #[allow(unused)]
    pub generate_llvm_ir: bool,
    pub symbol2addr: HashMap<String, String>,
    pub ignore_unknown_linker_library: bool,
    pub minify_wasm_size: bool,
    pub disable_all_optimizers: bool,
    // Enabling this option will activate all possible optimizations
    // recommended for erc20 contracts
    pub enable_all_optimizers: bool,
    /// Enable storage load/store little endian
    pub enable_storage_load_store_little_endian: bool,
}

impl Default for Yul2IROptions {
    fn default() -> Self {
        Self {
            output_dir: ".".to_string(),
            main_contract_name: "main".to_string(),
            verbose: false,
            debug_mode: false,
            opt_level: OptimizationLevel::Default,
            no_inline: false,
            use_llvm_toolchain: false,
            no_binaryen_optimize: false,
            generate_llvm_ir: false,
            symbol2addr: Default::default(),
            ignore_unknown_linker_library: false,
            minify_wasm_size: false,
            disable_all_optimizers: false,
            enable_all_optimizers: false,
            enable_storage_load_store_little_endian: false,
        }
    }
}

impl Yul2IROptions {
    #[allow(unused)]
    pub fn test(main_contract_name: &str) -> Self {
        let main_contract_name = Self::get_contract_name_without_deployed_ext(main_contract_name);
        let output_dir = &format!("./target/{main_contract_name}");
        if !std::fs::exists(output_dir).unwrap() {
            std::fs::create_dir_all(output_dir).unwrap();
        }
        Yul2IROptions {
            verbose: true,
            no_inline: true,
            use_llvm_toolchain: true,
            opt_level: OptimizationLevel::None,
            main_contract_name: main_contract_name.to_string(),
            output_dir: output_dir.to_string(),
            symbol2addr: Default::default(),
            ignore_unknown_linker_library: false,
            debug_mode: true,
            no_binaryen_optimize: false,
            generate_llvm_ir: false,
            minify_wasm_size: false,
            disable_all_optimizers: false,
            enable_all_optimizers: false,
            enable_storage_load_store_little_endian: true,
        }
    }
    #[allow(unused)]
    pub fn perf_test(main_contract_name: &str) -> Self {
        let main_contract_name = Self::get_contract_name_without_deployed_ext(main_contract_name);
        Yul2IROptions {
            opt_level: OptimizationLevel::Aggressive,
            main_contract_name: main_contract_name.to_string(),
            output_dir: ".".to_string(),
            ..Default::default()
        }
    }
    #[allow(unused)]
    pub fn debug(main_contract_name: &str) -> Self {
        let main_contract_name = Self::get_contract_name_without_deployed_ext(main_contract_name);
        Yul2IROptions {
            verbose: true,
            no_inline: true,
            use_llvm_toolchain: true,
            opt_level: OptimizationLevel::None,
            main_contract_name: main_contract_name.to_string(),
            output_dir: ".".to_string(),
            symbol2addr: Default::default(),
            ignore_unknown_linker_library: false,
            debug_mode: true,
            no_binaryen_optimize: false,
            generate_llvm_ir: false,
            minify_wasm_size: false,
            disable_all_optimizers: false,
            enable_all_optimizers: false,
            enable_storage_load_store_little_endian: true,
        }
    }

    fn get_contract_name_without_deployed_ext(contract_name: &str) -> String {
        let contract_name = contract_name.to_string();
        if contract_name.ends_with("_deployed") {
            contract_name
                .get(0..(contract_name.len() - "_deployed".len()))
                .unwrap()
                .to_string()
        } else {
            contract_name
        }
    }
}
