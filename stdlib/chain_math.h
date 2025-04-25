// Copyright (c) the DTVM authors Core Contributors
// SPDX-License-Identifier: Apache-2.0

#ifndef __CHAIN_MATH_H_
#define __CHAIN_MATH_H_

#include "stdlib.h"

#ifdef __cplusplus
extern "C" {
#endif

BOOL wrapper_bytes32_lt(const bytes32 *a, const bytes32 *b);

BOOL wrapper_bytes32_gt(const bytes32 *a, const bytes32 *b);

BOOL wrapper_bytes32_slt(const bytes32 *a, const bytes32 *b);

BOOL wrapper_bytes32_sgt(const bytes32 *a, const bytes32 *b);

BOOL wrapper_bytes32_eq(const bytes32 *a, const bytes32 *b);

BOOL wrapper_bytes32_iszero(const bytes32 *a);

void wrapper_bytes32_not(const bytes32 *a, bytes32 *result);

void wrapper_bytes32_and(const bytes32 *a, const bytes32 *b, bytes32 *result);

void wrapper_bytes32_or(const bytes32 *a, const bytes32 *b, bytes32 *result);

void wrapper_bytes32_xor(const bytes32 *a, const bytes32 *b, bytes32 *result);

void wrapper_bytes32_shl(const bytes32 *value, int32_t shift, bytes32 *result);

void wrapper_u256_shl(const uint256_t *value, int32_t shift, uint256_t *result);

void wrapper_bytes32_shr(const bytes32 *value, int32_t shift, bytes32 *result);

void wrapper_u256_shr(const uint256_t *value, int32_t shift, uint256_t *result);

void wrapper_bytes32_add(const bytes32 *a, const bytes32 *b, bytes32 *result);

#ifdef __cplusplus
}
#endif

#endif // __CHAIN_MATH_H_
