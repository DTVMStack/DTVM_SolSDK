// Copyright (c) the DTVM authors Core Contributors
// SPDX-License-Identifier: Apache-2.0
#include "chain.h"
#include "chain_math.h"
#include "debug.h"
#include "evm_memory.h"
#include "hostapi.h"
#include "utils.h"

static void u256_to_big_endian(const uint256_t value, uint8_t *memory_ptr) {
  // write to memory by big endian
  for (int i = 0; i < 32; i++) {
    // write from the largest bytes position
    memory_ptr[i] = (value >> (248 - i * 8)) & 0xFF;
  }
}

// extern func, so param and return types must be basic types
void u256_from_big_endian_bytes(const uint8_t *memory_ptr, uint256_t *result) {
  // read and constructor uint256_t by big endain

  // Since uint256_t is little endian, we can simply reverse the order.
  uint8_t *result_bytes_ptr = (uint8_t *)result;
  for (int i = 0; i < 32; i++) {
    result_bytes_ptr[i] = memory_ptr[31 - i];
  }
}

// extern func, so param and return types must be basic types
void u256_to_big_endian_bytes(const uint256_t *value, uint8_t *memory_ptr) {
  u256_to_big_endian(*value, memory_ptr);
}
void i32_to_bytes32_big_endian_bytes(int32_t value, uint8_t *memory_ptr) {
  // Clear first 28 bytes (3 i64 clears and 1 i32 clear)
  uint64_t *memory_ptr_u64 = (uint64_t *)memory_ptr;
  uint32_t *memory_ptr_u32 = (uint32_t *)memory_ptr;
  for (int i = 0; i < 3; i++) {
    memory_ptr_u64[i] = 0;
  }
  memory_ptr_u32[6] = 0;
  // Write int32 value in big endian format to last 4 bytes
  for (int i = 0; i < 4; i++) {
    memory_ptr[28 + i] = (value >> (24 - i * 8)) & 0xFF;
  }
}

int32_t i32_from_big_endian_bytes32(const bytes32 *value) {
  uint8_t *value_bytes = (uint8_t *)value;
  uint32_t result = 0;
  for (int i = 0; i < 4; i++) {
    result = (result << 8) + (0xff & value_bytes[28 + i]);
  }
  return result;
}

static void u32_to_big_endian(const uint32_t value, uint8_t *memory_ptr) {
  // write to memory by big endian
  for (int i = 0; i < 4; i++) {
    memory_ptr[i] = (value >> (24 - i * 8)) & 0xFF;
  }
}

static void u64_to_big_endian(const uint64_t value, uint8_t *memory_ptr) {
  // write to memory by big endian
  for (int i = 0; i < 8; i++) {
    memory_ptr[i] = (value >> (56 - i * 8)) & 0xFF;
  }
}

static int32_t cachedCallDataSize() {
  static int32_t cached = -1;
  if (cached < 0) {
    cached = getCallDataSize();
  }
  return cached;
}
static void cachedCallDataCopy(int32_t /* uint8_t* */ memory_ptr,
                               int32_t offset, uint32_t size) {
  // Avoid repeated calldatacopy operations
  int32_t calldata_size = cachedCallDataSize();
#define CACHED_CALLDATA_COPY_THRESHOLD 100
  if (calldata_size > CACHED_CALLDATA_COPY_THRESHOLD) {
    // For larger calldata, skip caching to avoid excessive memory usage
    callDataCopy(memory_ptr, offset, size);
    return;
  }
  static uint8_t cached_calldata[CACHED_CALLDATA_COPY_THRESHOLD] = {0};
  static BOOL cached = 0;
  if (!cached) {
    callDataCopy((ADDRESS_UINT)cached_calldata, 0, calldata_size);
    cached = 1;
  }
  memcpy((void *)memory_ptr, cached_calldata + offset, size);
}
void wrapper_calldataload_u256(uint32_t calldata_offset, uint256_t *result) {
  uint32_t calldata_size = cachedCallDataSize();
  uint8_t data[32] = {0};
  if ((calldata_offset + 32 > calldata_size)) {
    // If calldata is insufficient, read remaining part to the front of data
    // array The rest is already padded with zeros
    uint32_t read_count = calldata_size - calldata_offset;
    cachedCallDataCopy((ADDRESS_UINT)&data, calldata_offset, read_count);
  } else {
    cachedCallDataCopy((ADDRESS_UINT)&data, calldata_offset, 32);
  }
  u256_from_big_endian_bytes(data, result);
}
void wrapper_calldataload_bytes32(uint32_t calldata_offset, bytes32 *result) {
  uint32_t calldata_size = cachedCallDataSize();
  if ((calldata_offset + 32 > calldata_size)) {
    // If calldata is insufficient, read remaining part to the front of data
    // array The rest is already padded with zeros
    uint32_t read_count = calldata_size - calldata_offset;
    cachedCallDataCopy((ADDRESS_UINT)result, calldata_offset, read_count);
    // Pad remaining bytes with zeros
    uint8_t *result_bytes_ptr = (uint8_t *)result;
    for (int i = read_count; i < 32; i++) {
      result_bytes_ptr[i] = 0;
    }
  } else {
    cachedCallDataCopy((ADDRESS_UINT)result, calldata_offset, 32);
  }
}

// Optimized function to load just the function selector (first 4 bytes) from
// calldata Returns the selector as a uint32_t in big-endian format, which is
// the common way to represent function selectors in EVM
uint32_t wrapper_calldata_load_selector() {
  uint32_t calldata_size = cachedCallDataSize();

  if (calldata_size == 0) {
    return 0;
  }

  if (calldata_size < 4) {
    // if calldata not greater than 4 bytes, remaing bytes treat as zeros
    uint8_t selector_bytes[4] = {0};
    cachedCallDataCopy((ADDRESS_UINT)&selector_bytes, 0, calldata_size);
    // Convert to uint32_t (big-endian)
    uint32_t selector = 0;
    for (int i = 0; i < 4; i++) {
      selector = (selector << 8) | selector_bytes[i];
    }

    return selector;
  }

  // Only read the first 4 bytes
  uint8_t selector_bytes[4] = {0};
  cachedCallDataCopy((ADDRESS_UINT)&selector_bytes, 0, 4);

  // Convert to uint32_t (big-endian)
  uint32_t selector = 0;
  for (int i = 0; i < 4; i++) {
    selector = (selector << 8) | selector_bytes[i];
  }

  return selector;
}

int32_t wrapper_calldata_size() {
  uint32_t size = cachedCallDataSize();
  return (int32_t)size;
}

void wrapper_caller(bytes32 *result) {
  getCaller(12 + (ADDRESS_UINT)result); // address is 20 bytes
  // clear first 12 bytes with zeros

  uint32_t *tmp_ptr = (uint32_t *)result;
  tmp_ptr[0] = 0;
  tmp_ptr[1] = 0;
  tmp_ptr[2] = 0;
}

void wrapper_current_contract(bytes32 *result) {
  getAddress(12 + (ADDRESS_UINT)result); // address is 20 bytes
}

void wrapper_origin(bytes32 *result) {
  getTxOrigin(12 + (ADDRESS_UINT)result); // address is 20 bytes
}

void wrapper_block_coin_base(bytes32 *result) {
  getBlockCoinbase(12 + (ADDRESS_UINT)result); // address is 20 bytes
}

void wrapper_block_prevRandao(bytes32 *result) {
  getBlockPrevRandao(12 + (ADDRESS_UINT)result); // 32 bytes
}

void wrapper_callvalue(uint256_t *result) {
  uint8_t data[32];
  getCallValue((ADDRESS_UINT)&data);
  u256_from_big_endian_bytes(data, result);
}

BOOL wrapper_callvalue_not_zero() {
  uint8_t data[32];
  getCallValue((ADDRESS_UINT)&data);
  int64_t *data_int64 = (int64_t *)data;
  for (int i = 0; i < 4; i++) {
    if (data_int64[i] != 0) {
      return 1;
    }
  }
  return 0;
}

void wrapper_query_balance(bytes32 *addr_ptr, uint256_t *result) {
  uint8_t result_bytes[32];
  getExternalBalance(12 + (ADDRESS_UINT)addr_ptr,
                     (ADDRESS_UINT)&result_bytes); // address is 20 bytes
  u256_from_big_endian_bytes(result_bytes, result);
}

void wrapper_self_balance(uint256_t *result) {
  uint8_t addr_bytes[20] = {0};
  getAddress((ADDRESS_UINT)&addr_bytes);
  uint8_t result_bytes[32];
  getExternalBalance((ADDRESS_UINT)&addr_bytes, (ADDRESS_UINT)&result_bytes);
  u256_from_big_endian_bytes(result_bytes, result);
}

void wrapper_revert(int32_t error_msg_evm_mem, uint32_t size) {
  uint8_t *error_msg_ptr = evm_get_memory_addr(error_msg_evm_mem);
  revert((ADDRESS_UINT)error_msg_ptr, size);
}

void wrapper_stop() { finish(0, 0); }
void wrapper_codecopy(int32_t target_evm_mem_offset, int32_t evm_memory_offset,
                      uint32_t size) {
  evm_make_sure_memory(target_evm_mem_offset + size);
  uint8_t *target_mem_ptr = evm_get_memory_addr(target_evm_mem_offset);
  // For Yul, codecopy is sometimes used to copy actual calldata during contract
  // deployment (since in EVM, when deploying a contract, the contract bytecode
  // is init_code + calldata) However, this is different from WASM design. In
  // yul->wasm, wrapper_codecopy's second parameter arg1 could be either a
  // linear memory pointer (dataoffset constant area) or a contract bytecode
  // offset (e.g. for getting actual calldata) All yul instructions return
  // memory addresses relative to evm_memory_begin offset, so dataoffset returns
  // a 32-bit negative number, indicating it's part of the contract bytecode

  if ((int32_t)evm_memory_offset < 0) {
    // Copy from EVM contract bytecode, actually copying from WASM data segment
    // (linear memory) e.g. for codecopy(xxx, dataoffset(xxx), xxx)
    uint8_t *src_ptr = evm_get_memory_addr(evm_memory_offset);
    memcpy(target_mem_ptr, src_ptr, size);
    return;
  }

  int32_t cur_code_size = getCodeSize();

  if ((evm_memory_offset + size) < cur_code_size) {
    // Copy current WASM contract bytecode
    codeCopy((ADDRESS_UINT)target_mem_ptr, evm_memory_offset, size);
    return;
  }
  // Copy actual calldata, input parameter is wasm code size + calldata_offset
  int32_t calldata_offset = evm_memory_offset - cur_code_size;

  if (calldata_offset < 0) {
    __builtin_unreachable();
  }
  uint32_t calldata_size = cachedCallDataSize();
  if ((calldata_offset + size) > calldata_size) {
    // Only 4 bytes larger might be the input size
    __builtin_unreachable();
  }
  cachedCallDataCopy((ADDRESS_UINT)target_mem_ptr, calldata_offset, size);
}
uint32_t wrapper_memory_guard(uint32_t size) {
  evm_make_sure_memory(size);
  // Returns EVM memory address fixed at 96 (corresponding to linear memory
  // address evm_memory_begin + 96) The value 96 is used because in Solidity
  // EVM:
  // - Address range [0,64) is reserved for internal use
  // - Range 64-96 is used for the free memory pointer
  // - Actual usable memory starts from EVM memory address 96
  return 96;
}

void wrapper_mstore_bytes32(int32_t evm_mem, bytes32 *value_ptr) {
  evm_make_sure_memory(evm_mem + 32);
  uint8_t *memory_ptr = evm_get_memory_addr(evm_mem);
  memcpy(memory_ptr, value_ptr, 32);
}
void wrapper_mstore_u256(int32_t evm_mem, uint256_t *value_ptr) {
  uint256_t value = *value_ptr;
  // Automatically expand memory if it exceeds EVM accessible memory
  evm_make_sure_memory(evm_mem + 32);
  uint8_t *memory_ptr = evm_get_memory_addr(evm_mem);
  u256_to_big_endian(value, memory_ptr);
}
void wrapper_mstore_u32(int32_t evm_mem, uint32_t value) {
  // This is an optimized version of wrapper_mstore_u256 for u32 type
  // Although it writes 32 bytes, the first 28 bytes can be directly set to 0

  // Automatically expand memory if it exceeds EVM accessible memory
  evm_make_sure_memory(evm_mem + 32);
  uint8_t *memory_ptr = evm_get_memory_addr(evm_mem);

  // mstore(64, value) in Solidity reads/writes memptr value, not using the
  // front part

  // Write 0 to first 28 bytes. 28 bytes = 3 i64 + 1 i32
  uint64_t *memory_ptr_u64 = (uint64_t *)memory_ptr;
  memory_ptr_u64[0] = 0;
  memory_ptr_u64[1] = 0;
  memory_ptr_u64[2] = 0;
  if (value == 0) {
    memory_ptr_u64[3] = 0;
    return;
  }
  uint32_t *memory_ptr_u32 = (uint32_t *)(memory_ptr + 24);
  *memory_ptr_u32 = 0;

  // Write big endian encoded value to last 4 bytes
  u32_to_big_endian(value, memory_ptr + 28);
}

void wrapper_mstore_u64(int32_t evm_mem, uint64_t value) {
  // This is an optimized version of wrapper_mstore_u256 for u64 type
  // Although it writes 32 bytes, the first 24 bytes can be directly set to 0

  // Automatically expand memory if it exceeds EVM accessible memory
  evm_make_sure_memory(evm_mem + 32);
  uint8_t *memory_ptr = evm_get_memory_addr(evm_mem);

  // Write 0 to first 24 bytes (3 i64)
  uint64_t *memory_ptr_u64 = (uint64_t *)memory_ptr;
  memory_ptr_u64[0] = 0;
  memory_ptr_u64[1] = 0;
  memory_ptr_u64[2] = 0;
  if (value == 0) {
    memory_ptr_u64[3] = 0;
    return;
  }

  // Write big endian encoded value to last 8 bytes
  u64_to_big_endian(value, memory_ptr + 24);
}

void wrapper_mstore_u8(int32_t evm_mem, uint8_t value) {
  // Automatically expand memory if it exceeds EVM accessible memory
  evm_make_sure_memory(evm_mem + 32);
  uint8_t *memory_ptr = evm_get_memory_addr(evm_mem);
  *memory_ptr = value & 0xff;
}

void wrapper_mload_u256(int32_t evm_mem, uint256_t *result) {
  uint8_t *memory_ptr = evm_get_memory_addr(evm_mem);
  u256_from_big_endian_bytes(memory_ptr, result);
}

uint32_t wrapper_mload_u32(int32_t evm_mem) {
  uint8_t *memory_ptr = evm_get_memory_addr(evm_mem);
  // Read 4 bytes in big-endian order from the last 4 bytes of the 32-byte slot
  uint32_t result = 0;
  for (int i = 0; i < 4; i++) {
    result = (result << 8) | memory_ptr[28 + i];
  }
  return result;
}

uint64_t wrapper_mload_u64(int32_t evm_mem) {
  uint8_t *memory_ptr = evm_get_memory_addr(evm_mem);
  // Read 8 bytes in big-endian order from the last 8 bytes of the 32-byte slot
  uint64_t result = 0;
  for (int i = 0; i < 8; i++) {
    result = (result << 8) | memory_ptr[24 + i];
  }
  return result;
}

void wrapper_mload_bytes32(int32_t evm_mem, bytes32 *result) {
  uint8_t *memory_ptr = evm_get_memory_addr(evm_mem);
  // Copy all 32 bytes directly
  memcpy(result, memory_ptr, 32);
}

void wrapper_mcopy(int32_t evm_dst, int32_t evm_src, uint32_t size) {
  evm_make_sure_memory(evm_dst + size);
  uint8_t *dst_memory = evm_get_memory_addr(evm_dst);
  uint8_t *src_memory = evm_get_memory_addr(evm_src);
  memcpy(dst_memory, src_memory, size);
}

void wrapper_sstore_u256(uint256_t *slot_ptr, uint256_t *value_ptr) {
  uint256_t slot = *slot_ptr;
  uint8_t slot_bytes[32];
  u256_to_big_endian(slot, slot_bytes);
  uint8_t value_bytes[32];
  uint256_t value = *value_ptr;
  u256_to_big_endian(value, value_bytes);
  storageStore((ADDRESS_UINT)&slot_bytes, (ADDRESS_UINT)&value_bytes);
}

void wrapper_sstore_u256_using_little_endian_hostapi(bytes32 *slot_ptr,
                                                     uint256_t *value_ptr) {
  storageStoreLittleEndian((ADDRESS_UINT)slot_ptr, (ADDRESS_UINT)value_ptr);
}

void wrapper_sstore_bytes32(bytes32 *slot_ptr, bytes32 *value_ptr) {
  storageStore((ADDRESS_UINT)slot_ptr, (ADDRESS_UINT)value_ptr);
}

void wrapper_tstore_u256(uint256_t *slot_ptr, uint256_t *value_ptr) {
  uint256_t slot = *slot_ptr;
  uint8_t slot_bytes[32];
  u256_to_big_endian(slot, slot_bytes);
  uint8_t value_bytes[32];
  uint256_t value = *value_ptr;
  u256_to_big_endian(value, value_bytes);
  transientStore((ADDRESS_UINT)&slot_bytes, (ADDRESS_UINT)&value_bytes);
}

void wrapper_tstore_bytes32(bytes32 *slot_ptr, bytes32 *value_ptr) {
  transientStore((ADDRESS_UINT)slot_ptr, (ADDRESS_UINT)value_ptr);
}

void wrapper_sload_u256(bytes32 *slot_ptr, uint256_t *result) {
  uint8_t slot_value_bytes[32];
  storageLoad((ADDRESS_UINT)slot_ptr, (ADDRESS_UINT)&slot_value_bytes);
  u256_from_big_endian_bytes(slot_value_bytes, result);
}

void wrapper_sload_u256_using_little_endian_hostapi(bytes32 *slot_ptr,
                                                    uint256_t *result) {
  storageLoadLittleEndian((ADDRESS_UINT)slot_ptr, (ADDRESS_UINT)result);
}

void wrapper_sload_bytes32(bytes32 *slot_ptr, bytes32 *result) {
  storageLoad((ADDRESS_UINT)slot_ptr, (ADDRESS_UINT)result);
}

void wrapper_tload_u256(bytes32 *slot_ptr, uint256_t *result) {
  uint8_t slot_value_bytes[32];
  transientLoad((ADDRESS_UINT)slot_ptr, (ADDRESS_UINT)&slot_value_bytes);
  u256_from_big_endian_bytes(slot_value_bytes, result);
}

void wrapper_tload_bytes32(bytes32 *slot_ptr, bytes32 *result) {
  transientLoad((ADDRESS_UINT)slot_ptr, (ADDRESS_UINT)result);
}

void wrapper_setimmutable(int32_t offset, uint256_t *slot_ptr,
                          uint256_t *value_ptr) {
  uint256_t slot = *slot_ptr;

  uint8_t slot_bytes[32];
  u256_to_big_endian(slot, slot_bytes);
  uint8_t value_bytes[32];
  uint256_t value = *value_ptr;
  u256_to_big_endian(value, value_bytes);

  uint256_t flag_slot = slot; // not full

  uint8_t flag_slot_bytes[32];
  u256_to_big_endian(flag_slot, flag_slot_bytes);
  const char prefix[] = "setimmutable_";
  for (int i = 0; i < sizeof(prefix) - 1; i++) {
    flag_slot_bytes[i] += prefix[i];
  }
  uint8_t flag_value_bytes[32];
  storageLoad((ADDRESS_UINT)&flag_slot_bytes, (ADDRESS_UINT)&flag_value_bytes);
  uint256_t flag_value;
  u256_from_big_endian_bytes(flag_value_bytes, &flag_value);
  if (flag_value != 0) {
    revert((ADDRESS_UINT) "immutable slot already set", 26);
    return;
  }

  storageStore((ADDRESS_UINT)&slot_bytes, (ADDRESS_UINT)&value_bytes);

  flag_value_bytes[31] = 0x01;
  storageStore((ADDRESS_UINT)&flag_slot_bytes, (ADDRESS_UINT)&flag_value_bytes);
}

void wrapper_loadimmutable(uint256_t *slot_ptr, uint256_t *result) {
  uint256_t slot = *slot_ptr;
  uint8_t slot_bytes[32];
  u256_to_big_endian(slot, slot_bytes);
  uint8_t slot_value_bytes[32];
  storageLoad((ADDRESS_UINT)&slot_bytes, (ADDRESS_UINT)&slot_value_bytes);
  u256_from_big_endian_bytes(slot_value_bytes, result);
}

void wrapper_keccak256(int32_t evm_mem, uint32_t size, bytes32 *result) {
  uint8_t *memory_ptr = evm_get_memory_addr(evm_mem);
  keccak256((ADDRESS_UINT)memory_ptr, size, (ADDRESS_UINT)result);
}

void wrapper_return(int32_t src_evm_mem, uint32_t size) {
  uint8_t *memory_ptr = evm_get_memory_addr(src_evm_mem);
  finish((ADDRESS_UINT)memory_ptr, size);
}

void wrapper_decode_big_endian_i256_from_wasm_mem(uint8_t *wasm_mem,
                                                  uint256_t *result) {
  u256_from_big_endian_bytes(wasm_mem, result);
}

void wrapper_log0(int32_t data_evm_mem, uint32_t data_size) {
  uint8_t *memory_ptr = evm_get_memory_addr(data_evm_mem);
  uint32_t num_topics = 0;
  emitLogEvent((ADDRESS_UINT)memory_ptr, data_size, num_topics, 0, 0, 0, 0);
}

void wrapper_log1(int32_t data_evm_mem, uint32_t data_size,
                  bytes32 *topic0_ptr) {
  uint8_t *memory_ptr = evm_get_memory_addr(data_evm_mem);

  uint32_t num_topics = 1;
  uint8_t *topic0 = (uint8_t *)topic0_ptr;

  emitLogEvent((ADDRESS_UINT)memory_ptr, data_size, num_topics,
               (ADDRESS_UINT)topic0, 0, 0, 0);
}

void wrapper_log2(int32_t data_evm_mem, uint32_t data_size, bytes32 *topic0_ptr,
                  bytes32 *topic1_ptr) {
  uint8_t *memory_ptr = evm_get_memory_addr(data_evm_mem);

  uint8_t *topic0 = (uint8_t *)topic0_ptr;
  uint8_t *topic1 = (uint8_t *)topic1_ptr;

  uint32_t num_topics = 2;

  emitLogEvent((ADDRESS_UINT)memory_ptr, data_size, num_topics,
               (ADDRESS_UINT)topic0, (ADDRESS_UINT)topic1, 0, 0);
}

void wrapper_log3(int32_t data_evm_mem, uint32_t data_size, bytes32 *topic0_ptr,
                  bytes32 *topic1_ptr, bytes32 *topic2_ptr) {
  uint8_t *memory_ptr = evm_get_memory_addr(data_evm_mem);

  uint8_t *topic0 = (uint8_t *)topic0_ptr;
  uint8_t *topic1 = (uint8_t *)topic1_ptr;
  uint8_t *topic2 = (uint8_t *)topic2_ptr;

  uint32_t num_topics = 3;

  emitLogEvent((ADDRESS_UINT)memory_ptr, data_size, num_topics,
               (ADDRESS_UINT)topic0, (ADDRESS_UINT)topic1, (ADDRESS_UINT)topic2,
               0);
}

void wrapper_log4(int32_t data_evm_mem, uint32_t data_size, bytes32 *topic0_ptr,
                  bytes32 *topic1_ptr, bytes32 *topic2_ptr,
                  bytes32 *topic3_ptr) {
  uint8_t *memory_ptr = evm_get_memory_addr(data_evm_mem);

  uint8_t *topic0 = (uint8_t *)topic0_ptr;
  uint8_t *topic1 = (uint8_t *)topic1_ptr;
  uint8_t *topic2 = (uint8_t *)topic2_ptr;
  uint8_t *topic3 = (uint8_t *)topic3_ptr;

  uint32_t num_topics = 4;

  emitLogEvent((ADDRESS_UINT)memory_ptr, data_size, num_topics,
               (ADDRESS_UINT)topic0, (ADDRESS_UINT)topic1, (ADDRESS_UINT)topic2,
               (ADDRESS_UINT)topic3);
}
void wrapper_create(uint256_t *value, int32_t code_evm_mem, int32_t code_length,
                    bytes32 *result) {
  uint256_t value_u256 = *value;
  uint8_t value_bytes[32];
  u256_to_big_endian(value_u256, value_bytes);

  uint8_t *code_mem = evm_get_memory_addr(code_evm_mem);

  // The first 4 bytes of code_mem is a big-endian uint32_t representing the
  // wasm bytecode length The remaining bytes are calldata
  uint32_t wasm_code_length = 0;
  for (int i = 0; i < 4; i++) {
    wasm_code_length = (wasm_code_length << 8) | ((uint32_t)code_mem[i] & 0xFF);
  }

  uint32_t calldata_length = code_length - 4 - wasm_code_length;
  uint8_t *calldata_mem = code_mem + 4 + wasm_code_length;

  // Use ABI format (4 bytes wasm length + wasm code)
  int32_t ret_code =
      createContract((ADDRESS_UINT)&value_bytes, (ADDRESS_UINT)code_mem,
                     wasm_code_length + 4, (ADDRESS_UINT)calldata_mem,
                     calldata_length, 0, 0, 12 + (ADDRESS_UINT)result);
  if (ret_code == 0) {
    return;
  }
  revert((ADDRESS_UINT) "create contract failed", 22);
}
void wrapper_create2(uint256_t *value, int32_t code_evm_mem,
                     int32_t code_length, uint256_t *salt, bytes32 *result) {
  uint256_t value_u256 = *value;
  uint8_t value_bytes[32];
  u256_to_big_endian(value_u256, value_bytes);

  uint8_t *code_mem = evm_get_memory_addr(code_evm_mem);

  // The first 4 bytes of code_mem is a big-endian uint32_t representing the
  // wasm bytecode length The remaining bytes are calldata
  uint32_t wasm_code_length = 0;
  for (int i = 0; i < 4; i++) {
    wasm_code_length = (wasm_code_length << 8) | ((uint32_t)code_mem[i] & 0xFF);
  }
  uint32_t calldata_length = code_length - 4 - wasm_code_length;
  uint8_t *calldata_mem = code_mem + 4 + wasm_code_length;

  uint256_t salt_u256 = *salt;
  uint8_t salt_bytes[32];
  u256_to_big_endian(salt_u256, salt_bytes);

  // Use ABI format (4 bytes wasm length + wasm code)
  int32_t ret_code = createContract(
      (ADDRESS_UINT)&value_bytes, (ADDRESS_UINT)code_mem, wasm_code_length + 4,
      (ADDRESS_UINT)calldata_mem, calldata_length, (ADDRESS_UINT)&salt_bytes, 1,
      12 + (ADDRESS_UINT)result);
  if (ret_code == 0) {
    return;
  }
  revert((ADDRESS_UINT) "create2 contract failed", 23);
}

BOOL write_call_data(int32_t call_status, int32_t out_evm_offset,
                     int32_t out_length) {
  if (call_status == 2) {
    revert((ADDRESS_UINT) "call contract failed", 20);
    return 0;
  } else if (out_length == 0) {
    return call_status == 0 ? 1 : 0;
  } else {
    int32_t ret_data_size = getReturnDataSize();
    if (ret_data_size < out_length) {
      if (call_status == 0) {
        // success call
        wrapper_returndata_copy(out_evm_offset, 0, ret_data_size);
        return 1; // true
      }
#define MAX_REVERT_LENGTH 1024
      static uint8_t tmp_revert_data[MAX_REVERT_LENGTH] = {0};
      if (ret_data_size == 0) {
        revert((ADDRESS_UINT) "call failed with no revert data", 31);
      } else if (ret_data_size <= MAX_REVERT_LENGTH) {
        returnDataCopy((ADDRESS_UINT)&tmp_revert_data, 0, ret_data_size);
        revert((ADDRESS_UINT)&tmp_revert_data, ret_data_size);
      } else {
        revert((ADDRESS_UINT) "out length is not enough", 24);
      }
    } else {
      wrapper_returndata_copy(out_evm_offset, 0, out_length);
    }
    return call_status == 0 ? 1 : 0; // true or false
  }
}

static BOOL is_precompiled_contract(bytes32 *callee_addr_ptr) {
  uint64_t *addr_u64_ptr = (uint64_t *)callee_addr_ptr;
  if (addr_u64_ptr[0] != 0 || addr_u64_ptr[1] != 0 || addr_u64_ptr[2] != 0) {
    return 0; // false
  }
  uint8_t *last_8bytes = (uint8_t *)&addr_u64_ptr[3];
  if (last_8bytes[0] != 0 || last_8bytes[1] != 0 || last_8bytes[2] != 0 ||
      last_8bytes[3] != 0 || last_8bytes[4] != 0 || last_8bytes[5] != 0 ||
      last_8bytes[6] != 0) {
    return 0; // false
  }
  uint8_t addr_last_byte = last_8bytes[7] & 0xff;
  if (addr_last_byte >= 0x01 && addr_last_byte <= 0x0a) {
    // is precompiled contract
    return 1; // true
  }
  return 0; // false
}

int wrapper_call_contract(uint64_t gas, bytes32 *callee_addr_ptr,
                          uint256_t *value, int32_t in_evm_offset,
                          int32_t in_length, int32_t out_evm_offset,
                          int32_t out_length) {
  uint256_t value_u256 = *value;
  uint8_t value_bytes[32];
  u256_to_big_endian(value_u256, value_bytes);

  uint8_t *in_offset = evm_get_memory_addr(in_evm_offset);

  int32_t retCode = callContract(gas, 12 + (ADDRESS_UINT)callee_addr_ptr,
                                 (ADDRESS_UINT)&value_bytes,
                                 (ADDRESS_UINT)in_offset, in_length);

  if (retCode != 0 && is_precompiled_contract(callee_addr_ptr)) {
    // if the callee is a precompiled contract address, no return data if failed
    return 0;
  }

  return write_call_data(retCode, out_evm_offset, out_length);
}

int wrapper_delegatecall(uint64_t gas, bytes32 *callee_addr_ptr,
                         int32_t in_evm_offset, int32_t in_length,
                         int32_t out_evm_offset, int32_t out_length) {
  uint8_t *in_offset = evm_get_memory_addr(in_evm_offset);
  int32_t retCode = callDelegate(gas, 12 + (ADDRESS_UINT)callee_addr_ptr,
                                 (ADDRESS_UINT)in_offset, in_length);

  if (retCode != 0 && is_precompiled_contract(callee_addr_ptr)) {
    // if the callee is a precompiled contract address, no return data if failed
    return 0;
  }

  return write_call_data(retCode, out_evm_offset, out_length);
}

int wrapper_staticcall(uint64_t gas, bytes32 *callee_addr_ptr,
                       int32_t in_evm_offset, int32_t in_length,
                       int32_t out_evm_offset, int32_t out_length) {
  uint8_t *in_offset = evm_get_memory_addr(in_evm_offset);
  int32_t retCode = callStatic(gas, 12 + (ADDRESS_UINT)callee_addr_ptr,
                               (ADDRESS_UINT)in_offset, in_length);
  if (retCode != 0 && is_precompiled_contract(callee_addr_ptr)) {
    // if the callee is a precompiled contract address, no return data if failed
    return 0;
  }

  return write_call_data(retCode, out_evm_offset, out_length);
}

void wrapper_selfdestruct(bytes32 *addr_bytes) {
  uint8_t *addr_bytes_ptr = (uint8_t *)addr_bytes;
  selfDestruct(12 + (ADDRESS_UINT)addr_bytes_ptr);
}

void wrapper_invalid() { invalid(); }

void wrapper_current_chainid(uint256_t *result) {
  uint8_t blockchain_id_bytes[32] = {0};
  getChainId((ADDRESS_UINT)&blockchain_id_bytes); // block chain id is 32 bytes
  u256_from_big_endian_bytes(blockchain_id_bytes, result);
}

void wrapper_current_base_fee(uint256_t *result) {
  uint8_t base_fee_bytes[32] = {0};
  getBaseFee((ADDRESS_UINT)&base_fee_bytes); // base fee is 32 bytes
  u256_from_big_endian_bytes(base_fee_bytes, result);
}

void wrapper_current_blob_base_fee(uint256_t *result) {
  uint8_t base_fee_bytes[32] = {0};
  getBlobBaseFee((ADDRESS_UINT)&base_fee_bytes); // base fee is 32 bytes
  u256_from_big_endian_bytes(base_fee_bytes, result);
}

void wrapper_block_hash(uint64_t block_number, bytes32 *result) {
  getBlockHash(block_number, (ADDRESS_UINT)result); // block hash is 32 bytes
}

void wrapper_exp(uint256_t *base_ptr, uint256_t *exp_ptr, uint256_t *result) {
  uint256_t base = *base_ptr;
  uint256_t exp = *exp_ptr;
  uint256_t result_u256 = 1;
  uint256_t val = 1;
  for (;;) {
    if (exp & val) {
      result_u256 *= base;
    }
    exp >>= 1;
    if (!exp) {
      break;
    }
    base *= base;
  }
  *result = result_u256;
}

uint64_t wrapper_memory_size() {
  return (uint64_t)__builtin_wasm_memory_size(0);
}

uint64_t wrapper_gas() { return (uint64_t)getGasLeft(); }

uint64_t wrapper_time_stamp() { return (uint64_t)getBlockTimestamp(); }

uint64_t wrapper_block_number() { return (uint64_t)getBlockNumber(); }

uint64_t wrapper_gas_limit() { return (uint64_t)getBlockGasLimit(); }

void wrapper_gas_price(uint256_t *result) {
  uint8_t data[32] = {0};
  getTxGasPrice((ADDRESS_UINT)&data);
  u256_from_big_endian_bytes(data, result);
}

uint32_t wrapper_returndata_size() { return (uint32_t)getReturnDataSize(); }

void wrapper_calldata_copy(int32_t dst_evm, uint32_t calldata_offset,
                           uint32_t len) {
  evm_make_sure_memory(dst_evm + len);
  uint8_t *dst_memory_ptr = evm_get_memory_addr(dst_evm);

  // For out of bound bytes, 0s will be copied.
  uint32_t calldata_size = cachedCallDataSize();
  if (calldata_offset >= calldata_size) {
    __memset(dst_memory_ptr, 0, len);
  } else if (calldata_offset + len > calldata_size) {
    uint32_t data_len = calldata_size - calldata_offset;
    cachedCallDataCopy((ADDRESS_UINT)dst_memory_ptr, calldata_offset, data_len);
    __memset(dst_memory_ptr + data_len, 0, len - data_len);
  } else {
    cachedCallDataCopy((ADDRESS_UINT)dst_memory_ptr, calldata_offset, len);
  }
}

void wrapper_returndata_copy(int32_t dst_evm, uint32_t return_data_offset,
                             uint32_t len) {
  evm_make_sure_memory(dst_evm + len);
  uint8_t *dst_memory_ptr = evm_get_memory_addr(dst_evm);

  returnDataCopy((ADDRESS_UINT)dst_memory_ptr, (uint32_t)return_data_offset,
                 len);
}

// This flag is set when the contract deployment entry point is called,
// allowing us to determine if the current contract is being deployed.
static int32_t is_deploying_tx_flag = 0; // 1 (true), 0 (false)

void set_is_deploying_tx() { is_deploying_tx_flag = 1; }

// 1(true), 0(false)
int32_t is_deploying_tx() { return is_deploying_tx_flag; }

uint32_t wrapper_current_contract_code_size() {
  // During contract deployment, the semantics of yul codesize() actually
  // represent the creation code(length+wasm code) plus the actual calldata.
  if (is_deploying_tx_flag) {
    return (uint32_t)(getCodeSize() + cachedCallDataSize());
  }
  return (uint32_t)getCodeSize();
}

uint32_t wrapper_current_contract_pure_contract_size() {
  return (uint32_t)getCodeSize();
}

uint32_t wrapper_extcode_size(bytes32 *addr_ptr) {
  return (uint32_t)getExternalCodeSize(12 + (ADDRESS_UINT)addr_ptr);
}

void wrapper_extcode_copy(bytes32 *addr_ptr, int32_t dst_evm, uint32_t offset,
                          uint32_t len) {
  evm_make_sure_memory(dst_evm + len);
  uint8_t *dst = evm_get_memory_addr(dst_evm);
  externalCodeCopy(12 + (ADDRESS_UINT)addr_ptr, (ADDRESS_UINT)dst, offset, len);
}

void wrapper_addmod(bytes32 *a_ptr, bytes32 *b_ptr, bytes32 *mod_ptr,
                    bytes32 *result_ptr) {
  addmod((ADDRESS_UINT)a_ptr, (ADDRESS_UINT)b_ptr, (ADDRESS_UINT)mod_ptr,
         (ADDRESS_UINT)result_ptr);
}

void wrapper_mulmod(bytes32 *a_ptr, bytes32 *b_ptr, bytes32 *mod_ptr,
                    bytes32 *result_ptr) {
  mulmod((ADDRESS_UINT)a_ptr, (ADDRESS_UINT)b_ptr, (ADDRESS_UINT)mod_ptr,
         (ADDRESS_UINT)result_ptr);
}

void wrapper_extcode_hash(bytes32 *addr_ptr, bytes32 *result) {
  // Returns the keccak256 hash of the contract bytecode at the specified
  // address. If there is no contract deployed at this address, it returns a
  // zero hash.
  getExternalCodeHash(12 + (ADDRESS_UINT)addr_ptr, (ADDRESS_UINT)result);
}

void wrapper_data_copy(int32_t dst_evm, int32_t src_evm, uint32_t len) {
  evm_make_sure_memory(dst_evm + len);
  uint8_t *dst = evm_get_memory_addr(dst_evm);
  uint8_t *src = evm_get_memory_addr(src_evm);
  memcpy(dst, src, len);
}

void wrapper_sign_extend(uint32_t bits, uint256_t *value_ptr,
                         uint256_t *result) {
  // signextend(b, x) extends a smaller signed integer x to 256 bits, preserving
  // the original value's sign bit, and extends the bits preceding the specified
  // byte count (b) according to the most significant bit's sign.
  uint256_t value = *value_ptr;
  if (bits > 31) {
    *result = value;
    return; // If b is greater than 31, return the original value
  }
  // Calculate the number of bits to extend
  unsigned int bits_to_extend = (bits + 1) * 8;

  // Create the mask for the sign bit
  uint256_t sign_bit_mask = 1ULL << (bits_to_extend - 1);
  uint256_t extension_mask = ~((((uint256_t)1) << bits_to_extend) - 1);

  // Check the sign bit
  if (value & sign_bit_mask) {
    // If negative, extend with 1s
    *result = value | extension_mask;
  } else {
    // If positive, extend with 0s
    *result = value & ~extension_mask;
  }
}

int32_t wrapper_calldata_size_minus_4() {
  uint32_t calldata_size = cachedCallDataSize();
  return calldata_size - 4;
}

#define MEMORY_ALLOCA_BYTES32_BUFFER_COUNT 300

static bytes32
    memory_alloca_bytes32_buffer[MEMORY_ALLOCA_BYTES32_BUFFER_COUNT] = {0};
static uint32_t memory_alloca_bytes32_buffer_used_count = 0;
static uint8_t *extra_memory_alloca_bytes32_memory_begin = NULL;
static uint8_t *extra_memory_alloca_bytes32_memory_end = NULL;
static uint32_t extra_memory_alloca_bytes32_memory_used_size = 0;

bytes32 *memory_alloca_bytes32() {
  // Preallocate a small amount of bytes32-sized memory to avoid wasm memory
  // growth and to prevent the data segment from easily exceeding one page.
  if (memory_alloca_bytes32_buffer_used_count <
      MEMORY_ALLOCA_BYTES32_BUFFER_COUNT) {
    return &memory_alloca_bytes32_buffer
        [memory_alloca_bytes32_buffer_used_count++];
  }
  // If the predefined memory is insufficient, use additional allocated memory
  if (extra_memory_alloca_bytes32_memory_begin == NULL) {
    // grow new wasm memory page as extra alloca memory buffer
    uint32_t before_pages = __builtin_wasm_memory_size(0);
    extra_memory_alloca_bytes32_memory_begin =
        (uint8_t *)(before_pages * WASM_PAGE_SIZE);
    __builtin_wasm_memory_grow(0, 1);
    uint32_t after_pages = __builtin_wasm_memory_size(0);
    if (after_pages == before_pages) {
      __abort("wasm memory grow failed");
    }
    extra_memory_alloca_bytes32_memory_end =
        (uint8_t *)(after_pages * WASM_PAGE_SIZE);
    // allocate one bytes32 memory for return
    extra_memory_alloca_bytes32_memory_used_size = 0;
    bytes32 *result = (bytes32 *)(extra_memory_alloca_bytes32_memory_begin +
                                  extra_memory_alloca_bytes32_memory_used_size);
    extra_memory_alloca_bytes32_memory_used_size += sizeof(bytes32);
    return result;
  }
  if ((extra_memory_alloca_bytes32_memory_begin +
       extra_memory_alloca_bytes32_memory_used_size + sizeof(bytes32)) >=
      extra_memory_alloca_bytes32_memory_end) {
    // if extra memory buffer is not enough, allocate new memory buffer
    uint32_t before_pages = __builtin_wasm_memory_size(0);
    __builtin_wasm_memory_grow(0, 1);
    uint32_t after_pages = __builtin_wasm_memory_size(0);
    if (after_pages == before_pages) {
      __abort("wasm memory grow failed");
    }
    extra_memory_alloca_bytes32_memory_end =
        (uint8_t *)(after_pages * WASM_PAGE_SIZE);
    bytes32 *result = (bytes32 *)(extra_memory_alloca_bytes32_memory_begin +
                                  extra_memory_alloca_bytes32_memory_used_size);
    extra_memory_alloca_bytes32_memory_used_size += sizeof(bytes32);
    return result;
  }
  // have enough extra alloca memory buffer
  bytes32 *result = (bytes32 *)(extra_memory_alloca_bytes32_memory_begin +
                                extra_memory_alloca_bytes32_memory_used_size);
  extra_memory_alloca_bytes32_memory_used_size += sizeof(bytes32);
  return result;
}

uint256_t *memory_alloca_u256() { return (uint256_t *)memory_alloca_bytes32(); }

// When enable_all_optimizers is on, mstore(64, value) and mload(64)
// instructions will operate on this value
static int32_t evm_memptr_global = 0;

void wrapper_set_memptr_global(int32_t evm_mem) { evm_memptr_global = evm_mem; }
int32_t wrapper_get_memptr_global() { return evm_memptr_global; }

int32_t wrapper_allocate_memory(uint32_t size) {
  // Implementation of memory allocation logic equivalent to:
  // function allocate_memory(size) -> memPtr
  // {
  //     memPtr := mload(64)
  //     let newFreePtr := add(memPtr, and(add(size, 31), not(31)))
  //     if or(gt(newFreePtr, sub(shl(64, 1), 1)), lt(newFreePtr, memPtr))
  //     {
  //         mstore(0, shl(224, 0x4e487b71))
  //         mstore(4, 0x41)
  //         revert(0, 0x24)
  //     }
  //     mstore(64, newFreePtr)
  // }
  int32_t mem_ptr = wrapper_get_memptr_global();
  int32_t new_free_ptr = mem_ptr + (size + 31) & ~31;
  if (new_free_ptr <= mem_ptr) {
    __abort("allocate memory failed");
  }
  wrapper_set_memptr_global(new_free_ptr);
  return mem_ptr;
}

bytes32 *wrapper_zero_bytes32() {
  static bytes32 zero_bytes32 = {0};
  return &zero_bytes32;
}

static BOOL is_zero_address(uint8_t *bytes20) {
  // bytes20 = 2 * i64 + i32
  uint64_t *bytes20_u64 = (uint64_t *)bytes20;
  uint32_t *bytes20_u32 = (uint32_t *)bytes20;
  return bytes20_u64[0] == 0 && bytes20_u64[1] == 0 && bytes20_u32[4] == 0;
}
void wrapper_optimized_erc20_fun_transfer(bytes32 *from, bytes32 *to,
                                          bytes32 *var_value) {
  // optimized implementation of fun_transfer for standard ERC20:
  // yul function:
  // function fun_transfer(var_from, var_to, var_value)
  // {
  //     let _1 := and(var_from, sub(shl(160, 1), 1))
  //     let _2 := iszero(_1)
  //     if _2
  //     {
  //         mstore(0x00,shl(225, 0x4b637e8f))
  //         mstore(4,0x00)
  //         revert(0x00, 36)
  //     }
  //     let _3 := and(var_to, sub(shl(160, 1), 1))
  //     let _4 := iszero(_3)
  //     if _4
  //     {
  //         mstore(0x00, shl(224, 0xec442f05))
  //         mstore(4, 0x00)
  //         revert(0x00, 36)
  //     }
  //     _2 := 0x00
  //     mstore(0x00, _1)
  //     mstore(0x20, 0x00)
  //     let _5 := sload(keccak256(0x00, 0x40))
  //     if lt(_5, var_value)
  //     {
  //         mstore(0x00, shl(226, 0x391434e3))
  //         mstore(4, _1)
  //         mstore(36, _5)
  //         mstore(68, var_value)
  //         revert(0x00, 100)
  //     }
  //     mstore(0x00, _1)
  //     mstore(0x20, 0x00)
  //     sstore(keccak256(0x00, 0x40), sub(_5, var_value))
  //     _4 := 0x00
  //     mstore(0x00,  _3)
  //     mstore(0x20, 0x00)
  //     let dataSlot := keccak256(0x00, 0x40)
  //     sstore(dataSlot, add(sload(dataSlot), var_value))
  //     let _6 := mload(64)
  //     mstore(_6, var_value)
  //     log3(_6, 32,
  //     0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef, _1,
  //     _3)
  // }
  if (is_zero_address(((uint8_t *)from) + 12)) {
    // shl(225, 0x4b637e8f) ==
    // 0x96c6fd1e00000000000000000000000000000000000000000000000000000000
    static uint8_t revert_msg[36] = {
        0x96, 0xc6, 0xfd, 0x1e, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00};
    revert((ADDRESS_UINT)revert_msg, 36);
    return;
  }
  if (is_zero_address(((uint8_t *)to) + 12)) {
    // shl(224, 0xec442f05) ==
    // 0xec442f0500000000000000000000000000000000000000000000000000000000
    static uint8_t revert_msg[36] = {
        0xec, 0x44, 0x2f, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00};
    revert((ADDRESS_UINT)revert_msg, 36);
    return;
  }
  uint8_t tmp_keccak256_result[32];
  // calculate the slot of the balance of the from address
  uint8_t tmp_to_keccak256[64] = {0};
  memcpy(tmp_to_keccak256, from, 32);
  keccak256((ADDRESS_UINT)tmp_to_keccak256, 0x40,
            (ADDRESS_UINT)tmp_keccak256_result);
  uint256_t tmp_balance_u256;
  storageLoadLittleEndian((ADDRESS_UINT)tmp_keccak256_result,
                          (ADDRESS_UINT)&tmp_balance_u256);
  // now tmp_balance is the balance of the from address
  uint256_t var_value_u256;
  u256_from_big_endian_bytes((uint8_t *)var_value, &var_value_u256);

  if (tmp_balance_u256 < var_value_u256) { // from balance not enough
    // from balance is less than the value
    // shl(226, 0x391434e3) =
    // 0xe450d38c00000000000000000000000000000000000000000000000000000000
    uint8_t revert_msg[100] = {0xe4, 0x50, 0xd3, 0x8c};
    memcpy(revert_msg + 4, from, 32);
    uint8_t bigendian_balance[32];
    u256_to_big_endian_bytes(&tmp_balance_u256, (uint8_t *)bigendian_balance);
    memcpy(revert_msg + 36, bigendian_balance, 32);
    memcpy(revert_msg + 68, var_value, 32);
    revert((ADDRESS_UINT)revert_msg, 100);
    return;
  }
  // The original Yul code had redundant hash calculations, but this step is
  // unnecessary

  // write new balance to the from
  uint256_t new_from_balance_u256 = tmp_balance_u256 - var_value_u256;
  storageStoreLittleEndian((ADDRESS_UINT)tmp_keccak256_result,
                           (ADDRESS_UINT)&new_from_balance_u256);
  // calculate the slot of the balance of the to address
  memcpy(tmp_to_keccak256, to, 32);
  keccak256((ADDRESS_UINT)tmp_to_keccak256, 0x40,
            (ADDRESS_UINT)tmp_keccak256_result);
  storageLoadLittleEndian((ADDRESS_UINT)tmp_keccak256_result,
                          (ADDRESS_UINT)&tmp_balance_u256);

  uint256_t new_to_balance_u256 = tmp_balance_u256 + var_value_u256;
  storageStoreLittleEndian((ADDRESS_UINT)tmp_keccak256_result,
                           (ADDRESS_UINT)&new_to_balance_u256);
  // emitLogEvent(ADDRESS_UINT data_offset, int32_t length, int32_t
  // number_of_topics,
  //            ADDRESS_UINT topic1, ADDRESS_UINT topic2, ADDRESS_UINT topic3,
  //            ADDRESS_UINT topic4);
  // log3(_6, 32,
  // 0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef, _1, _3)
  uint8_t log_topic1[32] = {0xdd, 0xf2, 0x52, 0xad, 0x1b, 0xe2, 0xc8, 0x9b,
                            0x69, 0xc2, 0xb0, 0x68, 0xfc, 0x37, 0x8d, 0xaa,
                            0x95, 0x2b, 0xa7, 0xf1, 0x63, 0xc4, 0xa1, 0x16,
                            0x28, 0xf5, 0x5a, 0x4d, 0xf5, 0x23, 0xb3, 0xef};
  emitLogEvent((ADDRESS_UINT)var_value, 32, 3, (ADDRESS_UINT)log_topic1,
               (ADDRESS_UINT)from, (ADDRESS_UINT)to, 0);
}

void wrapper_debug_i256(uint256_t *value_ptr) {
#ifndef NDEBUG
  uint256_t value = *value_ptr;
  debug_string("debugging_i256\nas u32:");
  debug_i32((uint32_t)value);
  debug_string("as hex:");
  uint8_t data[32];
  u256_to_big_endian(value, data);
  debug_bytes((ADDRESS_UINT)&data, 32);
#endif // NDEBUG
}

void wrapper_debug_bytes32(bytes32 *value_ptr) {
#ifndef NDEBUG
  debug_bytes((ADDRESS_UINT)value_ptr, 32);
#endif // NDEBUG
}
