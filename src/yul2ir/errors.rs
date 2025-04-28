// Copyright (C) 2024-2025 the DTVM authors. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum ASTLoweringError {
    BuilderError(String),
    DuplicateVariableDefinition(String),
    UnsupportedType(String),
    FunctionReturnValueNotFound(String),
}

impl From<inkwell::builder::BuilderError> for ASTLoweringError {
    fn from(err: inkwell::builder::BuilderError) -> ASTLoweringError {
        ASTLoweringError::BuilderError(err.to_string())
    }
}

impl Display for ASTLoweringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ASTLoweringError::BuilderError(msg) => {
                write!(f, "Builder error: {}", msg)
            }
            ASTLoweringError::DuplicateVariableDefinition(msg) => {
                write!(f, "Variable definition error: {}", msg)
            }
            ASTLoweringError::UnsupportedType(msg) => {
                write!(f, "Unsupported type: {}", msg)
            }
            ASTLoweringError::FunctionReturnValueNotFound(msg) => {
                write!(f, "Function return value not found: {}", msg)
            }
        }
    }
}
