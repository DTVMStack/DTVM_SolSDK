// Copyright (c) the DTVM authors Core Contributors
// Copyright (c) The Smart Intermediate Representation Contributors
// SPDX-License-Identifier: Apache-2.0

// clang --target=wasm32 -c -emit-llvm -O3 -ffreestanding -fno-builtin -Wall
// stdlib.c

#include "stdlib.h"
#define MAX_ITOA_STR_SIZE 64
#define assert(x) (0)

#ifdef CC_LIB_TEST_MOCK
// revert hostapi mock
extern void revert(const char *error_msg, uint32_t error_msg_len) {
  // mock
}
#endif // CC_LIB_TEST_MOCK

#ifndef CC_LIB_TEST_MOCK
// sometimes llvm optimizers will include llvm.memset to set default value for
// struct which will import memset extern dependency so we add memset
// implementation for link
void *memset(void *dest, uint8_t val, size_t length) {
  __memset(dest, val, length);
  return dest;
}
#endif

// revert hostapi
extern void revert(const char *error_msg, uint32_t error_msg_len);

void __memset(void *_dest, uint8_t val, size_t length) {
  if (length == 0) {
    return;
  }
  uint8_t *dest = _dest;

  do {
    *dest++ = val;
  } while (--length);
}

#ifndef CC_LIB_TEST_MOCK

/*
 * Our memcpy can only deal with multiples of 8 bytes. This is
 * enough for simple allocator below.
 */
void *memcpy(void *_dest, const void *_src, uint32_t length) {
  uint8_t *dest = _dest;
  const uint8_t *src = _src;

  while (length--) {
    *dest++ = *src++;
  }
  return _dest;
}

#endif // CC_LIB_TEST_MOCK

bool __memcmp(uint8_t *left, uint32_t left_len, uint8_t *right,
              uint32_t right_len) {
  if (left_len != right_len)
    return false;

  while (left_len--) {
    if (*left++ != *right++)
      return false;
  }

  return true;
}
