// Copyright (C) 2024-2025 the DTVM authors, Ltd. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone, Default)]
pub struct ASTLoweringError {
    #[allow(unused)]
    pub message: String,
}

impl From<inkwell::builder::BuilderError> for ASTLoweringError {
    fn from(err: inkwell::builder::BuilderError) -> ASTLoweringError {
        ASTLoweringError {
            message: err.to_string(), // Convert the error message to a string
        }
    }
}
