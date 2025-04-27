// Copyright (c) the DTVM authors Core Contributors
// SPDX-License-Identifier: Apache-2.0

#ifndef __STDLIB_H_
#define __STDLIB_H_

#ifdef CC_LIB_TEST_MOCK
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#endif // CC_LIB_TEST_MOCK

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#define uint128_t __uint128_t
#define int128_t __int128_t

// int256 extension needs at least LLVM12
typedef unsigned _BitInt(256) uint256_t;
typedef _BitInt(256) int256_t;

typedef uint8_t bytes32[32];
// 0 is false, 1 or others is true
typedef int32_t BOOL;

#define INT128_MAX                                                             \
  (__int128)(((unsigned __int128)1                                             \
              << ((__SIZEOF_INT128__ * __CHAR_BIT__) - 1)) -                   \
             1)
#define INT128_MIN                                                             \
  ((__int128_t)0 - ((__int128_t)1 << 126) - ((__int128_t)1 << 126))
#define UINT128_MAX (((__uint128_t)INT128_MAX << 1) + 1)

#define INT256_MAX                                                             \
  (                                                                            \
      int256_t)(((uint256_t)1                                                  \
                 << (uint256_t)((2 * __SIZEOF_INT128__ * __CHAR_BIT__) - 1)) - \
                (uint256_t)1)
#define INT256_MIN (-(INT256_MAX) - (int256_t)1)
#define UINT256_MAX (((uint256_t)INT256_MAX << (uint256_t)1) + (uint256_t)1)

#ifdef __cplusplus
extern "C" {
#endif

#ifdef CC_LIB_TEST_MOCK
extern void *memset(void *dest, uint8_t val, size_t length);
#endif

extern void __memset(void *dest, uint8_t val, size_t length);
extern bool __memcmp(uint8_t *left, uint32_t left_len, uint8_t *right,
                     uint32_t right_len);

#ifndef CC_LIB_TEST_MOCK
extern void *memcpy(void *_dest, const void *_src, uint32_t length);
#endif // CC_LIB_TEST_MOCK

#ifndef CC_LIB_TEST_MOCK

#ifdef __cplusplus
}
#endif

#endif // CC_LIB_TEST_MOCK

extern void init_global_bytes_length(uint8_t *bytes, uint32_t length);
extern uint32_t get_global_bytes_length(uint8_t *bytes);

#ifdef __cplusplus
} // end "C"
#endif

#endif // __STDLIB_H_
