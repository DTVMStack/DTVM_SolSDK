// Copyright (C) 2024-2025 the DTVM authors. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

#[allow(unused)]
use crate::yul2ir::config::Yul2IROptions;
#[allow(unused)]
use crate::yul2ir::context::Yul2IRContext;
#[allow(unused)]
use crate::yul2ir::yul;
use ethabi::{encode, ParamType, Token};
use ethereum_types::H160;
#[allow(unused)]
use inkwell::context::Context;
use rand::distr::Alphanumeric;
use rand::Rng;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

pub struct TestRuntime {
    /// Test case name, also used in test.db file name
    #[allow(unused)]
    case_name: String,
    #[allow(unused)]
    wasm_bytecode: Vec<u8>,
    #[allow(unused)]
    last_exit_code: i32,
    #[allow(unused)]
    last_output: String,
    #[allow(unused)]
    last_error_output: String,
    #[allow(unused)]
    sender: Option<String>,
    #[allow(unused)]
    output_dir: String,
    #[allow(unused)]
    enable_gas_meter: bool,
    #[allow(unused)]
    enable_all_optimizers: bool,
}

// Helper function to calculate solidity selector from function signature
#[allow(unused)]
pub fn solidity_selector(function_signature: &str) -> Vec<u8> {
    let mut function_signature = function_signature.to_string();
    let mut data = function_signature.as_bytes().to_vec();
    keccak_hash::keccak256(&mut data);
    let selector = &data[0..4];
    selector.to_vec()
}

// Helper function to encode various types of parameters into calldata using ethabi library
#[allow(unused)]
pub fn encode_abi_parameters(parameters: &[(String, ParamType)]) -> Vec<u8> {
    let mut params: Vec<Token> = vec![];
    for (param_value_str, param_ty) in parameters.iter() {
        match param_ty {
            ParamType::Address => {
                let param_value_str =
                    if param_value_str.starts_with("0x") || param_value_str.starts_with("0X") {
                        &param_value_str[2..]
                    } else {
                        param_value_str
                    };
                params.push(Token::Address(H160::from_slice(
                    &hex::decode(param_value_str).unwrap(),
                )));
            }
            ParamType::Bool => params.push(Token::Bool(param_value_str.parse().unwrap())),
            ParamType::Uint(_) => params.push(Token::Uint(
                ethabi::Uint::from_str_radix(param_value_str, 10).unwrap(),
            )),
            ParamType::Int(_) => params.push(Token::Int(
                ethabi::Int::from_str_radix(param_value_str, 10).unwrap(),
            )),
            ParamType::Bytes => params.push(Token::Bytes(hex::decode(param_value_str).unwrap())),
            ParamType::String => params.push(Token::String(param_value_str.to_string())),
            _ => unreachable!("not supported param type {param_ty:?}"),
        }
    }
    encode(&params)
}

impl TestRuntime {
    #[allow(unused)]
    pub fn new(case_name: &str, output_basepath: &str) -> TestRuntime {
        if !std::fs::exists(output_basepath).unwrap() {
            std::fs::create_dir_all(output_basepath).unwrap();
        }
        TestRuntime {
            case_name: case_name.to_string(),
            wasm_bytecode: Vec::new(),
            last_exit_code: 0,
            last_output: String::new(),
            last_error_output: String::new(),
            sender: None,
            output_dir: output_basepath.to_string(),
            enable_gas_meter: true,
            enable_all_optimizers: false,
        }
    }

    #[allow(unused)]
    pub fn clear_testdata(&self) {
        let test_db_path = &self.get_test_db_path();
        if std::fs::exists(test_db_path).unwrap() {
            std::fs::remove_file(test_db_path).unwrap();
        }
    }

    fn get_test_db_path(&self) -> String {
        let test_db_path =
            std::path::Path::new(&self.output_dir).join(format!("{}.test.db", self.case_name));
        test_db_path.to_str().unwrap().to_string()
    }

    #[allow(unused)]
    pub fn compile_test_yul(&mut self, yul_code: &str) -> Result<Vec<u8>, String> {
        let expr = yul::ObjectParser::new().parse(yul_code).unwrap();
        let llvm_context = Context::create();
        let mut opts = Yul2IROptions::test(&self.case_name);
        if self.enable_all_optimizers {
            opts.enable_all_optimizers = true;
        }
        let mut context = Yul2IRContext::new_with_object(&llvm_context, &opts, expr);
        let emited_bc = context.emit("output");
        match emited_bc {
            Ok(emited_bc) => {
                self.wasm_bytecode = emited_bc.clone();
                Ok(emited_bc)
            }
            Err(e) => Err(e.to_string()),
        }
    }

    #[allow(unused)]
    pub fn compile_solidity_to_yul(
        &mut self,
        solidity_code: &str,
        contract_name: &str,
    ) -> Result<String, String> {
        // avoid using the invalid contract name(not strict check)
        // this is because solc will generate yul files with basename of contract name
        // but we don't known the name
        assert!(solidity_code.contains(&format!("contract {contract_name}")));

        // 1. create tmp file to store the solidity code
        let tmp_dir = Path::new(&self.output_dir);
        let sol_path = tmp_dir.join("input.sol");
        let yul_file_basename = &format!("{contract_name}.yul");
        let yul_path = tmp_dir.join(yul_file_basename);
        println!(
            "tmp_dir: {:?}, yul_path: {:?}",
            tmp_dir.as_os_str(),
            yul_path.as_os_str()
        );

        if !sol_path.exists() {
            let mut sol_file = File::create(&sol_path).map_err(|e| e.to_string())?;
            sol_file
                .write_all(solidity_code.as_bytes())
                .map_err(|e| e.to_string())?;
        }
        // There are multiple output files, and the output file names are related to the Solidity contract name.
        // 2. using `solc` to compile,
        // eg. solc --ir --optimize-yul -o . --overwrite input.sol
        let output = Command::new("solc")
            .current_dir(tmp_dir)
            .arg("--ir")
            .arg("--optimize-yul")
            .arg("--overwrite")
            .arg("-o")
            .arg(".")
            .arg("input.sol")
            .output()
            .map_err(|e| e.to_string())?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }
        // 3. read the generated yul file content
        let yul_content = std::fs::read_to_string(&yul_path)
            .map_err(|e| format!("Failed to read yul file: {}", e))?;
        Ok(yul_content)
    }

    #[allow(unused)]
    pub fn set_sender(&mut self, sender: Option<String>) {
        self.sender = sender;
    }

    #[allow(unused)]
    pub fn set_enable_gas_meter(&mut self, enable_gas_meter: bool) {
        self.enable_gas_meter = enable_gas_meter;
    }

    #[allow(unused)]
    pub fn set_enable_all_optimizers(&mut self, enable_all_optimizers: bool) {
        self.enable_all_optimizers = enable_all_optimizers;
    }

    // Deploy contract and return exit code
    #[allow(unused)]
    pub fn deploy(&mut self, calldata: &[u8]) -> Result<(), String> {
        // Call /opt/chain_mockcli subprocess and record exit code and output logs
        let mut cmd = Command::new("/opt/chain_mockcli");
        // Create temporary .wasm file to store wasm_bytecode
        let tmp_dir = tempdir().unwrap();
        let rand_str: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
        let tmp_file_path = tmp_dir.path().join(format!("{}.wasm", rand_str));
        let mut tmp_file = File::create(&tmp_file_path).unwrap();
        tmp_file.write_all(&self.wasm_bytecode).unwrap();

        cmd.arg("-f")
            .arg(tmp_file_path.to_str().unwrap())
            .arg("--action")
            .arg("deploy")
            .arg("--db-file")
            .arg(self.get_test_db_path())
            .arg("-i")
            .arg(hex::encode(calldata));
        if let Some(ref sender) = self.sender {
            cmd.arg("-s").arg(sender);
        }
        let output = cmd.output().unwrap();
        self.last_exit_code = output.status.code().unwrap();
        self.last_output = String::from_utf8_lossy(&output.stdout).to_string();
        self.last_error_output = String::from_utf8_lossy(&output.stderr).to_string();
        println!("last_output: {}", self.last_output);
        println!("last_error_output: {}", self.last_error_output);
        if self.last_exit_code != 0 {
            return Err(self.last_error_output.clone());
        }
        Ok(())
    }

    // Call contract function and return exit code
    #[allow(unused)]
    pub fn call(&mut self, selector: &[u8], encoded_params: &[u8]) -> Result<i32, String> {
        let calldata = &[selector, encoded_params].concat();
        // Call /opt/chain_mockcli subprocess and record exit code and output logs
        let mut cmd = Command::new("/opt/chain_mockcli");
        // Create temporary .wasm file to store wasm_bytecode
        let tmp_dir = tempdir().unwrap();
        let rand_str: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
        let tmp_file_path = tmp_dir.path().join(format!("{}.wasm", rand_str));
        let mut tmp_file = File::create(&tmp_file_path).unwrap();
        tmp_file.write_all(&self.wasm_bytecode).unwrap();

        cmd.arg("-f")
            .arg(tmp_file_path.to_str().unwrap())
            .arg("--action")
            .arg("call")
            .arg("--db-file")
            .arg(self.get_test_db_path())
            .arg("-i")
            .arg(hex::encode(calldata));

        if self.enable_gas_meter {
            cmd.arg("--enable-gas-meter");
        }

        if let Some(ref sender) = self.sender {
            cmd.arg("-s").arg(sender);
        }
        let output = cmd.output().unwrap();
        self.last_exit_code = output.status.code().unwrap_or(999);
        self.last_output = String::from_utf8_lossy(&output.stdout).to_string();
        self.last_error_output = String::from_utf8_lossy(&output.stderr).to_string();
        println!("last_output: {}", self.last_output);
        println!("last_error_output: {}", self.last_error_output);
        if self.last_exit_code != 0 {
            return Err(self.last_error_output.clone());
        }
        Ok(self.last_exit_code)
    }

    #[allow(unused)]
    pub fn assert_success(&self) {
        assert!(self.last_exit_code == 0);
    }

    #[allow(unused)]
    pub fn assert_failure(&self) {
        assert!(self.last_exit_code != 0);
    }

    // Helper function to find the last status line and assert its content.
    fn assert_last_status_line(
        &self,
        expected_prefix: &str,
        expected_hex: &str,
        status_type: &str,
    ) {
        let finish_prefix = "evm finish with result hex: ";
        let revert_prefix = "evm revert with result hex: ";
        let last_line = self
            .last_output
            .lines()
            .filter(|line| line.starts_with(finish_prefix) || line.starts_with(revert_prefix))
            .last();

        let expected_line = format!("{}{}", expected_prefix, expected_hex);

        match last_line {
            Some(line) => {
                assert_eq!(
                    line, expected_line,
                    "Expected last result line to be a {} '{}', but got '{}'. Full output:\n{}",
                    status_type, expected_line, line, self.last_output
                );
            }
            None => {
                panic!(
                    "Expected a {} line '{}', but no finish or revert lines found in output:\n{}",
                    status_type, expected_line, self.last_output
                );
            }
        }
    }

    #[allow(unused)]
    pub fn assert_result(&self, result_hex: &str) {
        let finish_prefix = "evm finish with result hex: ";
        self.assert_last_status_line(finish_prefix, result_hex, "finish");
    }

    #[allow(unused)]
    pub fn assert_revert(&self, revert_hex: &str) {
        let revert_prefix = "evm revert with result hex: ";
        self.assert_last_status_line(revert_prefix, revert_hex, "revert");
    }

    #[allow(unused)]
    pub fn wasm2wat(&self, wasm_file_path: &str, wat_file_path: &str) {
        let _ = std::process::Command::new("wasm2wat")
            .arg("-o")
            .arg(wat_file_path)
            .arg(wasm_file_path)
            .output()
            .unwrap();
    }
}
