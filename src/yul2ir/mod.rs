// Copyright (C) 2024-2025 Ant Group Co., Ltd. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(pub yul); // synthesized by LALRPOP

pub mod ast;
pub mod config;
pub mod context;
pub mod errors;
pub mod infer;
pub mod instruction;
pub mod stdlib;
pub mod transform;
pub mod usage;
pub mod utils;
pub mod var_scope;
pub mod wasm;
pub mod yul_instruction;
