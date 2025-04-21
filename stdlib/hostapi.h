// Copyright (c) The Ant Group Core Contributors
// SPDX-License-Identifier: Apache-2.0

#ifndef __HOSTAPI_H_
#define __HOSTAPI_H_

#include <stdint.h>
// Distinguish between uint32_t and uint64_t for pointer integer types based on
// environment, making it easier to test in 64-bit environments

#if INTPTR_MAX == INT64_MAX
#define ADDRESS_UINT uint64_t
#else
#define ADDRESS_UINT uint32_t
#define IN_WASM_ENV
#endif

// hostapis

__attribute__((import_module("env"), import_name("getAddress"))) void
getAddress(ADDRESS_UINT result_offset);

__attribute__((import_module("env"), import_name("getCaller"))) void
getCaller(ADDRESS_UINT result_offset);

__attribute__((import_module("env"), import_name("getCallValue"))) void
getCallValue(ADDRESS_UINT result_offset);

__attribute__((import_module("env"), import_name("getCallDataSize"))) int32_t
getCallDataSize();

__attribute__((import_module("env"), import_name("callDataCopy"))) void
callDataCopy(ADDRESS_UINT target, int32_t offset, int32_t len);

__attribute__((import_module("env"), import_name("getBlockHash"))) int32_t
getBlockHash(int64_t number, ADDRESS_UINT result_offset);

__attribute__((import_module("env"), import_name("getBlockCoinbase"))) void
getBlockCoinbase(ADDRESS_UINT result_offset);

__attribute__((import_module("env"), import_name("getBlockPrevRandao"))) void
getBlockPrevRandao(ADDRESS_UINT result_offset);

__attribute__((import_module("env"), import_name("getBlockGasLimit"))) int64_t
getBlockGasLimit();

__attribute__((import_module("env"), import_name("getBlockTimestamp"))) int64_t
getBlockTimestamp();

__attribute__((import_module("env"), import_name("getGasLeft"))) int64_t
getGasLeft();

__attribute__((import_module("env"), import_name("getBlockNumber"))) int64_t
getBlockNumber();

__attribute__((import_module("env"), import_name("getTxGasPrice"))) void
getTxGasPrice(ADDRESS_UINT value_offset);

__attribute__((import_module("env"), import_name("getTxOrigin"))) void
getTxOrigin(ADDRESS_UINT result_offset);

__attribute__((import_module("env"), import_name("getBaseFee"))) void
getBaseFee(ADDRESS_UINT result_offset);

__attribute__((import_module("env"), import_name("getBlobBaseFee"))) void
getBlobBaseFee(ADDRESS_UINT result_offset);

__attribute__((import_module("env"), import_name("getChainId"))) void
getChainId(ADDRESS_UINT result_offset);

__attribute__((import_module("env"), import_name("getExternalBalance"))) void
getExternalBalance(ADDRESS_UINT address_offset, ADDRESS_UINT result_offset);

__attribute__((import_module("env"), import_name("getExternalCodeHash"))) void
getExternalCodeHash(ADDRESS_UINT address_offset, ADDRESS_UINT result_offset);

__attribute__((import_module("env"), import_name("storageLoad"))) void
storageLoad(ADDRESS_UINT key_offset, ADDRESS_UINT result_offset);

__attribute__((import_module("env"), import_name("storageStore"))) void
storageStore(ADDRESS_UINT key_offset, ADDRESS_UINT value_offset);

__attribute__((import_module("env"),
               import_name("storageLoadLittleEndian"))) void
storageLoadLittleEndian(ADDRESS_UINT key_offset, ADDRESS_UINT result_offset);

__attribute__((import_module("env"),
               import_name("storageStoreLittleEndian"))) void
storageStoreLittleEndian(ADDRESS_UINT key_offset, ADDRESS_UINT value_offset);

__attribute__((import_module("env"), import_name("transientStore"))) void
transientStore(ADDRESS_UINT key_offset, ADDRESS_UINT value_offset);

__attribute__((import_module("env"), import_name("transientLoad"))) void
transientLoad(ADDRESS_UINT key_offset, ADDRESS_UINT result_offset);

__attribute__((import_module("env"), import_name("codeCopy"))) void
codeCopy(ADDRESS_UINT result_offset, int32_t code_offset, int32_t length);

__attribute__((import_module("env"), import_name("getCodeSize"))) int32_t
getCodeSize();

__attribute__((import_module("env"), import_name("externalCodeCopy"))) void
externalCodeCopy(ADDRESS_UINT address_offset, ADDRESS_UINT result_offset,
                 int32_t code_offset, int32_t length);

__attribute__((import_module("env"), import_name("getExternalCodeSize")))
int32_t
getExternalCodeSize(ADDRESS_UINT address_offset);

__attribute__((import_module("env"), import_name("callContract"))) int32_t
callContract(int64_t gas, ADDRESS_UINT addressOffset, ADDRESS_UINT valueOffset,
             ADDRESS_UINT dataOffset, int32_t dataLength);

__attribute__((import_module("env"), import_name("callCode"))) int32_t
callCode(int64_t gas, ADDRESS_UINT addressOffset, ADDRESS_UINT valueOffset,
         ADDRESS_UINT dataOffset, int32_t dataLength);

__attribute__((import_module("env"), import_name("callDelegate"))) int32_t
callDelegate(int64_t gas, ADDRESS_UINT addressOffset, ADDRESS_UINT dataOffset,
             int32_t dataLength);

__attribute__((import_module("env"), import_name("callStatic"))) int32_t
callStatic(int64_t gas, ADDRESS_UINT addressOffset, ADDRESS_UINT dataOffset,
           int32_t dataLength);

__attribute__((import_module("env"), import_name("createContract"))) int32_t
createContract(ADDRESS_UINT valueOffset, ADDRESS_UINT codeOffset,
               int32_t codeLength, ADDRESS_UINT dataOffset, int32_t dataLength,
               ADDRESS_UINT saltOffset, int32_t is_create2,
               ADDRESS_UINT resultOffset);

__attribute__((import_module("env"), import_name("finish"))) void
finish(ADDRESS_UINT data_offset, int32_t length);

__attribute__((import_module("env"), import_name("revert"))) void
revert(ADDRESS_UINT data_offset, int32_t length);

__attribute__((import_module("env"), import_name("invalid"))) void invalid();

__attribute__((import_module("env"), import_name("emitLogEvent"))) void
emitLogEvent(ADDRESS_UINT data_offset, int32_t length, int32_t number_of_topics,
             ADDRESS_UINT topic1, ADDRESS_UINT topic2, ADDRESS_UINT topic3,
             ADDRESS_UINT topic4);

__attribute__((import_module("env"), import_name("getReturnDataSize"))) int32_t
getReturnDataSize();

__attribute__((import_module("env"), import_name("returnDataCopy"))) void
returnDataCopy(ADDRESS_UINT resultOffset, int32_t dataOffset, int32_t length);

__attribute__((import_module("env"), import_name("selfDestruct"))) void
selfDestruct(ADDRESS_UINT addressOffset);

__attribute__((import_module("env"), import_name("keccak256"))) void
keccak256(ADDRESS_UINT inputOffset, int32_t inputLength,
          ADDRESS_UINT resultOffset);

__attribute__((import_module("env"), import_name("sha256"))) void
sha256(ADDRESS_UINT inputOffset, int32_t inputLength,
       ADDRESS_UINT resultOffset);

__attribute__((import_module("env"), import_name("addmod"))) void
addmod(ADDRESS_UINT aOffset, ADDRESS_UINT bOffset, ADDRESS_UINT nOffset,
       ADDRESS_UINT resultOffset);

__attribute__((import_module("env"), import_name("mulmod"))) void
mulmod(ADDRESS_UINT aOffset, ADDRESS_UINT bOffset, ADDRESS_UINT nOffset,
       ADDRESS_UINT resultOffset);

#endif // __HOSTAPI_H_
