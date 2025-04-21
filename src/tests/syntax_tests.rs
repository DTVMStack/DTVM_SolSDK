// Copyright (C) 2024-2025 Ant Group Co., Ltd. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

#[allow(unused)]
use super::test_helper::solidity_selector;
#[allow(unused)]
use super::test_helper::TestRuntime;
#[allow(unused)]
use crate::yul;
#[allow(unused)]
use crate::yul2ir::config::Yul2IROptions;
#[allow(unused)]
use crate::Yul2IRContext;
#[allow(unused)]
use inkwell::context::Context;

#[cfg(test)]
mod tests {
    use crate::yul2ir::utils::remove_comments;

    use super::*;

    #[test]
    fn test_multiline_comment_in_expr_syntax() {
        let expr = yul::ObjectParser::new()
        .parse(
            &remove_comments(
            r#"
        object "test_multiline_comment_in_expr_syntax" {
           
            code {
                function test_comment() {
                    /** aaa */
                    mstore(/** @src */ 0, 123)
                    mstore(/** @src -1:-1:-1 */ 0, /** @src 46:163:376  "{..." */ shl(224, 0x4e487b71))
                }
                test_comment()
            }
            data ".metadata" hex"aa"
        }
        "#,
        ))
        .unwrap();
        println!("{:?}", expr);
        let llvm_context = Context::create();
        let opts = Yul2IROptions::test("test_multiline_comment_in_expr_syntax");
        let mut context = Yul2IRContext::new_with_object(&llvm_context, &opts, expr);
        let emited_bc = context
            .emit("test_multiline_comment_in_expr_syntax")
            .unwrap();
        std::fs::write("test.out.wasm", emited_bc).unwrap();
    }
}
