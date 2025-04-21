// Copyright (c) The Ant Group Core Contributors
// Copyright (c) The Smart Intermediate Representation Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::yul2ir::utils;
use inkwell::OptimizationLevel;
use parity_wasm::builder;
use parity_wasm::elements::{InitExpr, Instruction, Module};
use rand::distr::Alphanumeric;
use rand::Rng;
use std::ffi::CString;
use std::fs::{self, File};
use std::io::Read;
use std::io::Write;
use std::process::Command;
use tempfile::tempdir;
use wizer::Wizer;

use super::config::Yul2IROptions;

fn generate_string(len: usize) -> String {
    rand::rngs::ThreadRng::default()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

fn run_wasm_optimizer(
    input: &[u8],
    optimizer_path: &str,
    optimizer_args: &[&str],
    check_command: Option<&str>,
) -> Option<Vec<u8>> {
    // Check if optimizer is available
    if let Some(cmd) = check_command {
        if Command::new(cmd).arg("--version").output().is_err() {
            println!(
                "{} is not installed or not in PATH. Skipping optimization.",
                cmd
            );
            return None;
        }
    }

    // Create temp directory
    let tmp_dir = tempdir().ok()?;
    let rand_input_filename = format!("wasm_input_{}.wasm", generate_string(10));
    let input_path = tmp_dir.path().join(rand_input_filename);
    fs::write(&input_path, input).ok()?;

    let output_filename = format!("wasm_output_{}.wasm", generate_string(10));
    let output_path = tmp_dir.path().join(output_filename);

    // Run optimizer
    println!("running {} to optimize file", optimizer_path);
    let mut command = Command::new(optimizer_path);
    command.arg("-o").arg(output_path.to_str().unwrap());
    command.args(optimizer_args);
    command.arg(input_path.to_str().unwrap());

    let optimize_result = command.spawn().and_then(|mut c| c.wait());

    // Cleanup input file
    let _ = fs::remove_file(input_path);

    match optimize_result {
        Ok(child) if child.success() => {
            let result = fs::read(&output_path).ok();
            let _ = fs::remove_file(output_path);
            result
        }
        _ => None,
    }
}

#[allow(unused)]
fn do_binaryen_optimize(input: &[u8]) -> Option<Vec<u8>> {
    run_wasm_optimizer(input, "wasm-opt", &["-O2", "-g"], Some("wasm-opt"))
}

#[allow(unused)]
fn do_start_optimize(input: &[u8]) -> Option<Vec<u8>> {
    // use wizer to optimize the wasm
    let initialized_wasm_bytes = Wizer::new().init_func("_start").run(input);
    if let Err(e) = initialized_wasm_bytes {
        println!(
            "wizer error: {}, this is optional optimizer, we will continue",
            e
        );
        return None;
    }
    Some(initialized_wasm_bytes.unwrap())
}

#[cfg(feature = "release")]
extern "C" {
    fn LLDWasmLink(args: *const *const libc::c_char, size: libc::size_t) -> libc::c_int;
}

#[cfg(feature = "release")]
pub fn wasm_linker(args: &[CString]) -> bool {
    let mut command_line: Vec<*const libc::c_char> = Vec::with_capacity(args.len() + 1);

    let executable_name = CString::new("wasm-ld").unwrap();

    command_line.push(executable_name.as_ptr());

    for arg in args {
        command_line.push(arg.as_ptr());
    }

    unsafe { LLDWasmLink(command_line.as_ptr(), command_line.len()) == 0 }
}

#[cfg(not(feature = "release"))]
pub fn wasm_linker(args: &[CString]) -> bool {
    use std::process::Command;

    let mut command_line: Vec<String> = Vec::with_capacity(args.len() + 1);

    for arg in args {
        command_line.push(arg.to_str().unwrap().to_string());
    }

    let result = Command::new("wasm-ld")
        .args(&command_line)
        .output()
        .expect("wasm-ld run error");
    if !result.stderr.is_empty() {
        println!("{}", String::from_utf8(result.stderr).unwrap());
    }
    false
}

fn clang_opt_level_string(level: &inkwell::OptimizationLevel) -> String {
    match level {
        OptimizationLevel::None => "O0".to_string(),
        OptimizationLevel::Less => "O1".to_string(),
        OptimizationLevel::Default => "O2".to_string(),
        OptimizationLevel::Aggressive => "O3".to_string(),
    }
}

pub fn link(
    input: &[u8],
    name: &str,
    export_names: &[String],
    opts: &Yul2IROptions,
    has_sub_contract: bool,
) -> Vec<u8> {
    let dir = tempdir().expect("failed to create temp directory for linking");

    let object_filename = dir.path().join(format!("{name}.o"));
    let res_filename = dir.path().join(format!("{name}.wasm"));

    let mut objectfile =
        File::create(object_filename.clone()).expect("failed to create object file");

    objectfile
        .write_all(input)
        .expect("failed to write object file to temp file");

    let clang_rt_lib_dir = std::env::var("CHAIN_IR_CLANG_RT_LIB_DIR")
        .unwrap_or_else(|_| utils::get_clang_rt_lib_dir());

    let mut command_line = vec![
        CString::new(format!(
            "-{}",
            clang_opt_level_string(&opts.opt_level.to_inkwell_optimization_level())
        ))
        .unwrap(),
        CString::new("--allow-undefined").unwrap(),
        CString::new("--gc-sections").unwrap(),
        CString::new("--global-base=0").unwrap(),
        CString::new("--stack-first").unwrap(),
        // Link compiler-rt for wasm32 target.
        CString::new("-lclang_rt.builtins-wasm32").unwrap(),
        CString::new(format!("-L{clang_rt_lib_dir}")).unwrap(),
    ];
    command_line.push(CString::new("--no-entry").unwrap());

    let mut stack_size = 0x10000; // 64K stack size default

    if opts.enable_all_optimizers && !has_sub_contract {
        command_line.push(CString::new("-z").unwrap());
        stack_size = 32768; // half page if it's small contract
                            // use half page stack, so we can have some space for const segments and init evm memory
        command_line.push(CString::new(format!("stack-size={stack_size}")).unwrap());
    }

    if export_names.is_empty() {
        command_line.push(CString::new("--export-all").unwrap());
    } else {
        command_line.push(CString::new("--export").unwrap());
        command_line.push(CString::new("__wasm_call_ctors").unwrap());
        command_line.push(CString::new("--export").unwrap());
        command_line.push(CString::new("_start").unwrap());
        for name in export_names {
            command_line.push(CString::new("--export").unwrap());
            command_line.push(CString::new(name.as_str()).unwrap());
        }
    }

    if !opts.debug_mode {
        command_line.push(CString::new("-O").unwrap());
        command_line.push(CString::new("2").unwrap());
    }

    command_line.push(
        CString::new(
            object_filename
                .to_str()
                .expect("temp path should be unicode"),
        )
        .unwrap(),
    );

    if opts.minify_wasm_size {
        // when needed smallest wasm size, strip all debug info
        command_line.push(CString::new("--strip-all").unwrap());
    }

    command_line.push(CString::new("-o").unwrap());
    command_line
        .push(CString::new(res_filename.to_str().expect("temp path should be unicode")).unwrap());

    assert!(!wasm_linker(&command_line), "linker failed");

    let mut output = Vec::new();
    // read the whole file
    let mut outputfile = File::open(res_filename).expect("output file should exist");

    outputfile
        .read_to_end(&mut output)
        .expect("failed to read output file");

    let mut module: Module =
        parity_wasm::deserialize_buffer(&output).expect("cannot deserialize llvm wasm");

    // set stack pointer (there is only one global)
    if let Some(global) = module
        .global_section_mut()
        .unwrap()
        .entries_mut()
        .iter_mut()
        .next()
    {
        let init_expr = global.init_expr_mut();
        *init_expr = InitExpr::new(vec![Instruction::I32Const(stack_size), Instruction::End]);
    }

    let linked = builder::module().with_module(module);

    let linked_wasm_bytes =
        parity_wasm::serialize(linked.build()).expect("cannot serialize linked wasm");

    // use optimizers to optimize the wasm
    let linked_wasm_bytes = if !opts.no_binaryen_optimize {
        do_binaryen_optimize(&linked_wasm_bytes).unwrap_or(linked_wasm_bytes)
    } else {
        linked_wasm_bytes
    };

    do_start_optimize(&linked_wasm_bytes).unwrap_or(linked_wasm_bytes)
}
