// Copyright (c) the DTVM authors Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::yul2ir::config::Yul2IROptions;
use inkwell::context::Context;
use inkwell::memory_buffer::MemoryBuffer;
use inkwell::module::Module;

/// Load standard libraries.
pub fn load_stdlib<'a>(
    opts: &'a Yul2IROptions,
    ctx_ref: &'a Context,
    a: Vec<&'a [u8]>,
) -> Module<'a> {
    let memory = MemoryBuffer::create_from_memory_range(WASM_IR[0], "wasm_bc");

    let module: Module<'a> = Module::parse_bitcode_from_buffer(&memory, ctx_ref).unwrap();

    let mut to_link_modules = WASM_IR.iter().skip(1).collect::<Vec<_>>();
    if !opts.debug_mode {
        for m in RELEASE_MODE_EXTRA_WASM_IR.iter() {
            to_link_modules.push(m);
        }
    }

    for bc in to_link_modules.iter() {
        let memory = MemoryBuffer::create_from_memory_range(bc, "wasm_bc");

        module
            .link_in_module(Module::parse_bitcode_from_buffer(&memory, ctx_ref).unwrap())
            .unwrap();
    }

    for bc in a.iter() {
        let memory = MemoryBuffer::create_from_memory_range(bc, "wasm_bc");

        module
            .link_in_module(Module::parse_bitcode_from_buffer(&memory, ctx_ref).unwrap())
            .unwrap();
    }

    module
}

#[cfg(debug_assertions)]
static WASM_IR: [&[u8]; 5] = [
    include_bytes!("../../stdlib/wasm/debug/stdlib.bc"),
    include_bytes!("../../stdlib/wasm/debug/chain.bc"),
    include_bytes!("../../stdlib/wasm/debug/utils.bc"),
    include_bytes!("../../stdlib/wasm/debug/evm_memory.bc"),
    include_bytes!("../../stdlib/wasm/debug/chain_math.bc"),
];

#[cfg(not(debug_assertions))]
static WASM_IR: [&[u8]; 5] = [
    include_bytes!("../../stdlib/wasm/release/stdlib.bc"),
    include_bytes!("../../stdlib/wasm/release/chain.bc"),
    include_bytes!("../../stdlib/wasm/release/utils.bc"),
    include_bytes!("../../stdlib/wasm/release/evm_memory.bc"),
    include_bytes!("../../stdlib/wasm/release/chain_math.bc"),
];

#[cfg(debug_assertions)]
static RELEASE_MODE_EXTRA_WASM_IR: [&[u8]; 0] = [];

#[cfg(not(debug_assertions))]
static RELEASE_MODE_EXTRA_WASM_IR: [&[u8]; 1] = [include_bytes!(
    "../../stdlib/wasm/release/debug_in_release.bc"
)];
