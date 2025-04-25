// Copyright (C) 2024-2025 the DTVM authors, Ltd. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// Definition of the YulInstructionName enum.

use inkwell::{
    types::{BasicTypeEnum, IntType},
    values::BasicValueEnum,
};
// The value type with semantic meaning during the yul->wasm transformation process, which needs to be returned during walk_expr
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum YulLowLevelValueType {
    None,
    I32,
    I64,
    U256,
    #[allow(unused)]
    U256Pointer,
    Bytes32, // TODO: don't use it anymore, use Bytes32Pointer instead
    Bytes32Pointer,
    Tuple, // only yul function return value maybe use this
}
impl YulLowLevelValueType {
    pub fn from_int_type(ty: IntType<'_>) -> Self {
        match ty.get_bit_width() {
            32 => YulLowLevelValueType::I32,
            64 => YulLowLevelValueType::I64,
            256 => YulLowLevelValueType::U256,
            _ => {
                unreachable!(
                    "unsupported int type in YulLowLevelValueType: Int{}",
                    ty.get_bit_width()
                );
            }
        }
    }

    pub fn from_basic_type_enum(ty: BasicTypeEnum<'_>) -> Self {
        if ty.is_int_type() {
            Self::from_int_type(ty.into_int_type())
        } else if ty.is_struct_type() {
            YulLowLevelValueType::Tuple
        } else if ty.is_pointer_type() {
            // is bytes32 pointer
            YulLowLevelValueType::Bytes32Pointer
        } else if ty.is_array_type() {
            // is bytes32
            YulLowLevelValueType::Bytes32
        } else {
            unreachable!("unsupported basic type in YulLowLevelValueType: {}", ty)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct YulLowLevelValue<'a> {
    pub value_type: YulLowLevelValueType,
    pub value: BasicValueEnum<'a>,
}

impl<'a> From<YulLowLevelValue<'a>> for BasicValueEnum<'a> {
    fn from(val: YulLowLevelValue<'a>) -> Self {
        val.value
    }
}

impl<'a> YulLowLevelValue<'a> {
    pub fn get_value(self) -> BasicValueEnum<'a> {
        self.value
    }
}

#[derive(Debug, Clone)]
pub struct YulLowLevelFunctionType<'a> {
    pub params: Vec<YulLowLevelValueType>,
    pub params_inkwell_type: Vec<BasicTypeEnum<'a>>,
    // multiple returns will be a struct type(tuple)
    pub returns: Vec<YulLowLevelValueType>,
    pub returns_inkwell_type: Vec<BasicTypeEnum<'a>>,
}

impl<'a> YulLowLevelFunctionType<'a> {
    pub fn new(params: Vec<YulLowLevelValueType>, returns: Vec<YulLowLevelValueType>) -> Self {
        Self {
            params,
            params_inkwell_type: vec![],
            returns,
            returns_inkwell_type: vec![],
        }
    }

    pub fn add_param(
        &mut self,
        param: YulLowLevelValueType,
        param_inkwell_type: BasicTypeEnum<'a>,
    ) {
        self.params.push(param);
        self.params_inkwell_type.push(param_inkwell_type);
    }

    pub fn add_return(&mut self, ret: YulLowLevelValueType, ret_inkwell_type: BasicTypeEnum<'a>) {
        self.returns.push(ret);
        self.returns_inkwell_type.push(ret_inkwell_type);
    }
}

#[derive(Debug, Clone)]
pub enum YulInstructionName {
    Stop,
    Add,
    Sub,
    Mul,
    Div,
    SDiv,
    Mod,
    SMod,
    Exp,
    Not,
    Lt,
    Gt,
    SLt,
    SGt,
    Eq,
    IsZero,
    And,
    Or,
    Xor,
    Byte,
    Shl,
    Shr,
    Sar,
    AddMod,
    MulMod,
    SignExtend,
    Keccak256,
    Pop,
    MLoad,
    MStore,
    MStore8,
    MCopy,
    SLoad,
    TLoad,
    LoadImmutable,
    SStore,
    TStore,
    SetImmutable,
    MSize,
    Gas,
    Address,
    Balance,
    SelfBalance,
    Caller,
    CallValue,
    CallDataLoad,
    CallDataSize,
    CallDataCopy,
    CodeSize,
    CodeCopy,
    ExtCodeSize,
    ExtCodeCopy,
    DataCopy,
    DataOffset,
    DataSize,
    ReturnDataSize,
    ReturnDataCopy,
    ExtCodeHash,
    Create,
    Create2,
    Call,
    CallCode,
    DelegateCall,
    StaticCall,
    Return,
    Revert,
    SelfDestruct,
    Invalid,
    Log0,
    Log1,
    Log2,
    Log3,
    Log4,
    ChainID,
    BaseFee,
    BlobBaseFee,
    Origin,
    GasPrice,
    BlockHash,
    BlobHash,
    CoinBase,
    TimeStamp,
    Number,
    Difficulty,
    Prevrandao,
    GasLimit,
    MemoryGuard,
    LinkerSymbol,

    // only supported in opts.debug mode
    /// debug instructions add by dev
    DebugPrint,
}

impl From<String> for YulInstructionName {
    fn from(s: String) -> YulInstructionName {
        match parse_intrinsic_func_name(&s) {
            Some(intrinsic) => intrinsic,
            None => unimplemented!("instruction {} unimplemented", s),
        }
    }
}

pub fn parse_intrinsic_func_name(s: &str) -> Option<YulInstructionName> {
    match s {
        "stop" => Some(YulInstructionName::Stop),
        "add" => Some(YulInstructionName::Add),
        "sub" => Some(YulInstructionName::Sub),
        "mul" => Some(YulInstructionName::Mul),
        "div" => Some(YulInstructionName::Div),
        "sdiv" => Some(YulInstructionName::SDiv),
        "mod" => Some(YulInstructionName::Mod),
        "smod" => Some(YulInstructionName::SMod),
        "exp" => Some(YulInstructionName::Exp),
        "not" => Some(YulInstructionName::Not),
        "lt" => Some(YulInstructionName::Lt),
        "gt" => Some(YulInstructionName::Gt),
        "slt" => Some(YulInstructionName::SLt),
        "sgt" => Some(YulInstructionName::SGt),
        "eq" => Some(YulInstructionName::Eq),
        "iszero" => Some(YulInstructionName::IsZero),
        "and" => Some(YulInstructionName::And),
        "or" => Some(YulInstructionName::Or),
        "xor" => Some(YulInstructionName::Xor),
        "byte" => Some(YulInstructionName::Byte),
        "shl" => Some(YulInstructionName::Shl),
        "shr" => Some(YulInstructionName::Shr),
        "sar" => Some(YulInstructionName::Sar),
        "addmod" => Some(YulInstructionName::AddMod),
        "mulmod" => Some(YulInstructionName::MulMod),
        "signextend" => Some(YulInstructionName::SignExtend),
        "keccak256" => Some(YulInstructionName::Keccak256),
        "pop" => Some(YulInstructionName::Pop),
        "mload" => Some(YulInstructionName::MLoad),
        "mstore" => Some(YulInstructionName::MStore),
        "mstore8" => Some(YulInstructionName::MStore8),
        "mcopy" => Some(YulInstructionName::MCopy),
        "sload" => Some(YulInstructionName::SLoad),
        "tload" => Some(YulInstructionName::TLoad),
        "loadimmutable" => Some(YulInstructionName::LoadImmutable),
        "sstore" => Some(YulInstructionName::SStore),
        "tstore" => Some(YulInstructionName::TStore),
        "setimmutable" => Some(YulInstructionName::SetImmutable),
        "msize" => Some(YulInstructionName::MSize),
        "gas" => Some(YulInstructionName::Gas),
        "address" => Some(YulInstructionName::Address),
        "balance" => Some(YulInstructionName::Balance),
        "selfbalance" => Some(YulInstructionName::SelfBalance),
        "caller" => Some(YulInstructionName::Caller),
        "callvalue" => Some(YulInstructionName::CallValue),
        "calldataload" => Some(YulInstructionName::CallDataLoad),
        "calldatasize" => Some(YulInstructionName::CallDataSize),
        "calldatacopy" => Some(YulInstructionName::CallDataCopy),
        "codesize" => Some(YulInstructionName::CodeSize),
        "codecopy" => Some(YulInstructionName::CodeCopy),
        "extcodesize" => Some(YulInstructionName::ExtCodeSize),
        "extcodecopy" => Some(YulInstructionName::ExtCodeCopy),
        "datacopy" => Some(YulInstructionName::DataCopy),
        "dataoffset" => Some(YulInstructionName::DataOffset),
        "datasize" => Some(YulInstructionName::DataSize),
        "returndatasize" => Some(YulInstructionName::ReturnDataSize),
        "returndatacopy" => Some(YulInstructionName::ReturnDataCopy),
        "extcodehash" => Some(YulInstructionName::ExtCodeHash),
        "create" => Some(YulInstructionName::Create),
        "create2" => Some(YulInstructionName::Create2),
        "call" => Some(YulInstructionName::Call),
        "callcode" => Some(YulInstructionName::CallCode),
        "delegatecall" => Some(YulInstructionName::DelegateCall),
        "staticcall" => Some(YulInstructionName::StaticCall),
        "return" => Some(YulInstructionName::Return),
        "revert" => Some(YulInstructionName::Revert),
        "selfdestruct" => Some(YulInstructionName::SelfDestruct),
        "invalid" => Some(YulInstructionName::Invalid),
        "log0" => Some(YulInstructionName::Log0),
        "log1" => Some(YulInstructionName::Log1),
        "log2" => Some(YulInstructionName::Log2),
        "log3" => Some(YulInstructionName::Log3),
        "log4" => Some(YulInstructionName::Log4),
        "chainid" => Some(YulInstructionName::ChainID),
        "basefee" => Some(YulInstructionName::BaseFee),
        "blobbasefee" => Some(YulInstructionName::BlobBaseFee),
        "origin" => Some(YulInstructionName::Origin),
        "gasprice" => Some(YulInstructionName::GasPrice),
        "blockhash" => Some(YulInstructionName::BlockHash),
        "blobhash" => Some(YulInstructionName::BlobHash),
        "coinbase" => Some(YulInstructionName::CoinBase),
        "timestamp" => Some(YulInstructionName::TimeStamp),
        "number" => Some(YulInstructionName::Number),
        "difficulty" => Some(YulInstructionName::Difficulty),
        "prevrandao" => Some(YulInstructionName::Prevrandao),
        "gaslimit" => Some(YulInstructionName::GasLimit),
        "memoryguard" => Some(YulInstructionName::MemoryGuard),
        "linkersymbol" => Some(YulInstructionName::LinkerSymbol),
        "debug_print" => Some(YulInstructionName::DebugPrint),
        _ => None,
    }
}
