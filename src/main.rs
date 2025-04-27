// Copyright (C) 2024-2025 the DTVM authors. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

mod test;
mod tests;
mod yul2ir;

use crate::yul2ir::config::Yul2IROptions;
use crate::yul2ir::context::Yul2IRContext;
use crate::yul2ir::yul;
use crate::yul2ir::yul_instruction::YulLowLevelValueType;
use clap::Parser;
use inkwell::context::Context;
use std::fs;
use std::path::Path;
use std::process;

#[derive(Parser, Debug)]
#[command(name = "yul2wasm")]
#[command(about = "Compile Yul source to wasm")]
#[command(version = env!("GIT_HASH"))]
struct Args {
    #[arg(long = "input", help = "Input file path")]
    input: String,
    #[arg(long = "output", help = "Output wasm path")]
    output: String,
    #[arg(long = "verbose", help = "Verbose output")]
    verbose: bool,
    #[arg(long = "debug", help = "Debug output", default_value = "false")]
    debug: bool,
    // config::OptimizationLevel
    #[arg(
        long = "opt-level",
        help = "Optimization level",
        default_value = "default"
    )]
    opt_level: String,
    // if not set main contract name, default is the top object name
    #[arg(long = "main-contract", help = "Main contract name")]
    main_contract: Option<String>,
    #[arg(long = "symbol", help = "Symbol path=address", value_name = "PATH=ADDRESS", action = clap::ArgAction::Append)]
    symbol: Vec<String>,
    #[arg(
        long = "ignore-unknown-linker-library",
        help = "Ignore unknown linker library",
        default_value = "false"
    )]
    ignore_unknown_linker_library: bool,
    #[arg(
        long = "no-binaryen-optimize",
        help = "No binaryen optimize",
        default_value = "true"
    )]
    no_binaryen_optimize: bool,
    #[arg(
        long = "minify-wasm-size",
        help = "Minify wasm size",
        default_value = "false"
    )]
    minify_wasm_size: bool,
    #[arg(
        long = "disable-all-optimizers",
        help = "Disable all optimizers",
        default_value = "false"
    )]
    disable_all_optimizers: bool,
    // Enabling this option will activate all possible optimizations
    #[arg(
        long = "enable-all-optimizers",
        help = "Enable all optimizers",
        default_value = "false"
    )]
    enable_all_optimizers: bool,
    // Whether the platform provides storageLoadLittleEndian and storageStoreLittleEndian hostapi
    #[arg(
        long = "enable-little-endian-storage-load-store",
        help = "Enable little endian storage load/store",
        default_value = "false"
    )]
    enable_little_endian_storage_load_store: bool,
    #[arg(
        long = "default_ret_type",
        help = "Default return type",
        default_value = "u256"
    )]
    default_ret_type: Option<String>,
}

fn main() {
    let args = Args::parse();
    // Read the --input parameter from command line arguments and load file content into yul_src
    let yul_src = match fs::read_to_string(&args.input) {
        Ok(content) => content,
        Err(err) => {
            println!("Error reading file {}: {}", args.input, err);
            process::exit(1);
        }
    };
    let yul_src = yul2ir::utils::remove_comments(&yul_src);
    let block = match yul::ObjectParser::new().parse(&yul_src) {
        Ok(obj) => obj,
        Err(err) => {
            println!("Error parsing yul file {}: {}", args.input, err);
            process::exit(1);
        }
    };

    let llvm_context = Context::create();
    let mut opts = Yul2IROptions {
        verbose: args.verbose,
        opt_level: args.opt_level.parse().unwrap(),
        ignore_unknown_linker_library: args.ignore_unknown_linker_library,
        disable_all_optimizers: args.disable_all_optimizers,
        minify_wasm_size: args.minify_wasm_size,
        no_binaryen_optimize: args.no_binaryen_optimize,
        enable_all_optimizers: args.enable_all_optimizers,
        enable_storage_load_store_little_endian: args.enable_little_endian_storage_load_store,
        ..Default::default()
    };

    for sym in &args.symbol {
        let parts: Vec<&str> = sym.split('=').collect();
        if parts.len() == 2 {
            let path = parts[0];
            let address = parts[1];
            opts.symbol2addr
                .insert(path.to_string(), address.to_string());
        } else {
            eprintln!(
                "Invalid symbol format: {}. Expected format: path=address",
                sym
            );
        }
    }

    if let Some(main_contract_name) = args.main_contract {
        opts.main_contract_name = main_contract_name;
    } else {
        opts.main_contract_name = block.name.clone();
    }
    // Get the absolute path directory of args.output, create it if it doesn't exist
    let output_dir = Path::new(&args.output).parent().unwrap();
    // If output_dir is empty, it represents the current directory
    let output_dir = if output_dir == Path::new("") {
        Path::new(".")
    } else {
        output_dir
    };
    let output_dir = output_dir.canonicalize().unwrap();
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir).unwrap();
    }

    opts.output_dir = output_dir.to_str().unwrap_or(".").to_string();

    if args.debug {
        opts.no_inline = true;
        opts.use_llvm_toolchain = true;
        opts.opt_level = yul2ir::config::OptimizationLevel::None;
        opts.debug_mode = true;
    }
    let mut context = Yul2IRContext::new_with_object(&llvm_context, &opts, block);
    context.default_ret_type = match args.default_ret_type {
        None => YulLowLevelValueType::U256,
        Some(string) => match string.as_str() {
            "u256" => YulLowLevelValueType::U256,
            "bytes32" => YulLowLevelValueType::Bytes32Pointer,
            str => {
                eprintln!(
                    "Invalid default return type: {}. Expected: u256, bytes32",
                    str
                );
                process::exit(1);
            }
        },
    };
    let wasm_basename = Path::new(&args.output)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();
    let emited_bc = match context.emit(wasm_basename) {
        Ok(bc) => bc,
        Err(err) => {
            println!("Error emitting WebAssembly: {}", err);
            process::exit(1);
        }
    };
    std::fs::write(&args.output, &emited_bc).unwrap();
    // cbin is contract bin code(prefix + wasm)
    let contract_cbin_path = Path::new(&args.output).with_extension("cbin");
    let wasmcode_length_big_endian_4bytes = (emited_bc.len() as u32).to_be_bytes();
    let contract_cbin_code = [wasmcode_length_big_endian_4bytes.to_vec(), emited_bc].concat();
    std::fs::write(&contract_cbin_path, &contract_cbin_code).unwrap();
    let contract_cbin_code_hex = hex::encode(&contract_cbin_code);
    let contract_cbin_hex_path = Path::new(&args.output).with_extension("cbin.hex");
    std::fs::write(&contract_cbin_hex_path, contract_cbin_code_hex).unwrap();
    println!("wasm writen to {}", &args.output);
    println!(
        "cbin(contract to deploy) writen to {} and {}",
        &contract_cbin_path.display(),
        &contract_cbin_hex_path.display()
    );
}
