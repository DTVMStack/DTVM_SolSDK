// Copyright (c) The Ant Group Core Contributors
// SPDX-License-Identifier: Apache-2.0

#ifndef __CHAIN_H_
#define __CHAIN_H_

#include "hostapi.h"
#include "stdlib.h"
#ifdef __cplusplus
extern "C" {
#endif

void u256_to_big_endian_bytes(const uint256_t *value, uint8_t *memory_ptr);

void i32_to_bytes32_big_endian_bytes(int32_t value, uint8_t *memory_ptr);

void u256_from_big_endian_bytes(const uint8_t *memory_ptr, uint256_t *result);

int32_t i32_from_big_endian_bytes32(const bytes32 *value);

// Functions exported for inkwell usage cannot have complex custom struct value
// types, otherwise parameter passing will fail after wasm generation
void wrapper_calldataload_u256(uint32_t calldata_offset, uint256_t *result);

void wrapper_calldataload_bytes32(uint32_t calldata_offset, bytes32 *result);

// Optimized function to load just the function selector (first 4 bytes) from
// calldata This is an optimization for the common pattern: shr(224,
// calldataload(0))
uint32_t __attribute__((pure)) wrapper_calldata_load_selector();

int32_t __attribute__((pure)) wrapper_calldata_size();

void wrapper_caller(bytes32 *result);

void wrapper_current_contract(bytes32 *result);

void wrapper_callvalue(uint256_t *result);

BOOL wrapper_callvalue_not_zero();

void wrapper_query_balance(bytes32 *addr_ptr, uint256_t *result);

void wrapper_self_balance(uint256_t *result);

void wrapper_revert(int32_t error_msg_evm_mem, uint32_t size);

void wrapper_stop();

void wrapper_codecopy(int32_t target_evm_mem_offset, int32_t evm_memory_offset,
                      uint32_t size);

// Takes a memory size to protect and returns the starting address of memory
// that won't be overwritten by other operations
uint32_t wrapper_memory_guard(uint32_t size);

void wrapper_mstore_bytes32(int32_t evm_mem, bytes32 *value_ptr);

void wrapper_mstore_u256(int32_t evm_mem, uint256_t *value_ptr);

void wrapper_mstore_u32(int32_t evm_mem, uint32_t value);

void wrapper_mstore_u64(int32_t evm_mem, uint64_t value);

void wrapper_mstore_u8(int32_t evm_mem, uint8_t value);

void wrapper_mload_u256(int32_t evm_mem, uint256_t *result);

uint32_t wrapper_mload_u32(int32_t evm_mem);

uint64_t wrapper_mload_u64(int32_t evm_mem);

void wrapper_mload_bytes32(int32_t evm_mem, bytes32 *result);

void wrapper_mcopy(int32_t evm_dst, int32_t evm_src, uint32_t size);

void wrapper_sload_u256(bytes32 *slot_ptr, uint256_t *result);

void wrapper_sload_bytes32(bytes32 *slot_ptr, bytes32 *result);

void wrapper_tload_u256(bytes32 *slot_ptr, uint256_t *result);

void wrapper_tload_bytes32(bytes32 *slot_ptr, bytes32 *result);

void wrapper_sstore_u256(uint256_t *slot_ptr, uint256_t *value_ptr);

void wrapper_sstore_u256_using_little_endian_hostapi(bytes32 *slot_ptr,
                                                     uint256_t *value_ptr);

void wrapper_sstore_bytes32(bytes32 *slot_ptr, bytes32 *value_ptr);

void wrapper_tstore_u256(uint256_t *slot_ptr, uint256_t *value_ptr);

void wrapper_tstore_bytes32(bytes32 *slot_ptr, bytes32 *value_ptr);

void wrapper_setimmutable(int32_t offset, uint256_t *slot_ptr,
                          uint256_t *value_ptr);

void wrapper_loadimmutable(uint256_t *slot_ptr, uint256_t *result);

void wrapper_keccak256(int32_t evm_mem, uint32_t size, bytes32 *result);

void wrapper_return(int32_t src_evm_mem, uint32_t size);

void wrapper_decode_big_endian_i256_from_wasm_mem(uint8_t *wasm_mem,
                                                  uint256_t *result);

void wrapper_log0(int32_t data_evm_mem, uint32_t data_size);
void wrapper_log1(int32_t data_evm_mem, uint32_t data_size,
                  bytes32 *topic0_ptr);
void wrapper_log2(int32_t data_evm_mem, uint32_t data_size, bytes32 *topic0_ptr,
                  bytes32 *topic1_ptr);
void wrapper_log3(int32_t data_evm_mem, uint32_t data_size, bytes32 *topic0_ptr,
                  bytes32 *topic1_ptr, bytes32 *topic2_ptr);
void wrapper_log4(int32_t data_evm_mem, uint32_t data_size, bytes32 *topic0_ptr,
                  bytes32 *topic1_ptr, bytes32 *topic2_ptr,
                  bytes32 *topic3_ptr);

uint64_t wrapper_time_stamp();

uint64_t wrapper_block_number();

void wrapper_current_chainid(uint256_t *result);

void wrapper_current_base_fee(uint256_t *result);

void wrapper_current_blob_base_fee(uint256_t *result);

void wrapper_origin(bytes32 *result);

void wrapper_block_coin_base(bytes32 *result);

void wrapper_block_hash(uint64_t block_number, bytes32 *result);

void wrapper_block_prevRandao(bytes32 *result);

void wrapper_exp(uint256_t *base_ptr, uint256_t *exp_ptr, uint256_t *result);

void wrapper_create(uint256_t *value, int32_t code_evm_mem, int32_t code_length,
                    bytes32 *result);

void wrapper_create2(uint256_t *value, int32_t code_evm_mem,
                     int32_t code_length, uint256_t *salt, bytes32 *result);

BOOL write_call_data(int32_t call_status, int32_t out_evm_offset,
                     int32_t out_length);

int wrapper_call_contract(uint64_t gas, bytes32 *callee_addr_ptr,
                          uint256_t *value, int32_t in_evm_offset,
                          int32_t in_length, int32_t out_evm_offset,
                          int32_t out_length);

int wrapper_delegatecall(uint64_t gas, bytes32 *callee_addr_ptr,
                         int32_t in_evm_offset, int32_t in_length,
                         int32_t out_evm_offset, int32_t out_length);

int wrapper_staticcall(uint64_t gas, bytes32 *callee_addr_ptr,
                       int32_t in_evm_offset, int32_t in_length,
                       int32_t out_evm_offset, int32_t out_length);

void wrapper_selfdestruct(bytes32 *addr_bytes);

void wrapper_invalid();

uint64_t wrapper_memory_size();

uint64_t wrapper_gas();
uint64_t wrapper_gas_limit();
void wrapper_gas_price(uint256_t *result);

uint32_t wrapper_returndata_size();

void wrapper_calldata_copy(int32_t dst_evm, uint32_t calldata_offset,
                           uint32_t len);

void wrapper_returndata_copy(int32_t dst_evm, uint32_t return_data_offset,
                             uint32_t len);

void set_is_deploying_tx();

// 1(true), 0(false)
int32_t is_deploying_tx();

// Returns the total size of the current contract's bytecode including calldata
// if deploying
uint32_t wrapper_current_contract_code_size();

// Returns only the pure contract bytecode size (prefix + wasm bytecode)
// excluding calldata
uint32_t wrapper_current_contract_pure_contract_size();

uint32_t wrapper_extcode_size(bytes32 *addr_ptr);

void wrapper_extcode_copy(bytes32 *addr_ptr, int32_t dst_evm, uint32_t offset,
                          uint32_t len);

void wrapper_extcode_hash(bytes32 *addr_ptr, bytes32 *result);

void wrapper_data_copy(int32_t dst_evm, int32_t src_evm, uint32_t len);

void wrapper_sign_extend(uint32_t bits, uint256_t *value_ptr,
                         uint256_t *result);

int32_t wrapper_calldata_size_minus_4();

// Allocate memory of size bytes32 and return a pointer to the memory.
// This is useful in scenarios such as function returns, to avoid needing
// additional stack memory copies when returning a bytes32 type.
bytes32 *memory_alloca_bytes32();

// Since u256 and bytes32 have the same memory size, we can reuse the same
// memory buffer for both.
uint256_t *memory_alloca_u256();
// Optimized implementation for setting and getting the global memory pointer in
// EVM
void wrapper_set_memptr_global(int32_t evm_mem);
int32_t wrapper_get_memptr_global();

// Optimized implementation of allocate_memory generated by Solidity; this
// function generally follows a fixed logic. return evm memory offset
int32_t wrapper_allocate_memory(uint32_t size);

// A zero bytes32 is commonly used, such as for the first slot, so we optimize
// it directly.
bytes32 *wrapper_zero_bytes32();

void wrapper_addmod(bytes32 *a_ptr, bytes32 *b_ptr, bytes32 *mod_ptr,
                    bytes32 *result_ptr);

void wrapper_mulmod(bytes32 *a_ptr, bytes32 *b_ptr, bytes32 *mod_ptr,
                    bytes32 *result_ptr);

// Optimized implementation of fun_transfer for standard ERC20
// only available when enable_all_optimizers is true
void wrapper_optimized_erc20_fun_transfer(bytes32 *from, bytes32 *to,
                                          bytes32 *var_value);

void wrapper_debug_i256(uint256_t *value_ptr);

void wrapper_debug_bytes32(bytes32 *value_ptr);

#ifdef __cplusplus
} // extern "C"
#endif

#endif // __CHAIN_H_
