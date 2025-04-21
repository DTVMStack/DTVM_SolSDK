// Copyright (c) The Ant Group Core Contributors
// SPDX-License-Identifier: Apache-2.0

#include "chain_math.h"
#include "chain.h"

BOOL wrapper_bytes32_lt(const bytes32 *a, const bytes32 *b) {
  // Compare bytes from left to right (big endian)
  uint8_t *a_bytes = (uint8_t *)a;
  uint8_t *b_bytes = (uint8_t *)b;
  for (int i = 0; i < 32; i++) {
    if (a_bytes[i] < b_bytes[i]) {
      return 1; // a < b
    } else if (a_bytes[i] > b_bytes[i]) {
      return 0; // a > b
    }
  }
  return 0; // a == b
}

BOOL wrapper_bytes32_gt(const bytes32 *a, const bytes32 *b) {
  // Compare bytes from left to right (big endian)
  uint8_t *a_bytes = (uint8_t *)a;
  uint8_t *b_bytes = (uint8_t *)b;
  for (int i = 0; i < 32; i++) {
    if (a_bytes[i] > b_bytes[i]) {
      return 1; // a > b
    } else if (a_bytes[i] < b_bytes[i]) {
      return 0; // a < b
    }
  }
  return 0; // a == b
}

BOOL wrapper_bytes32_eq(const bytes32 *a, const bytes32 *b) {
  // Compare bytes from left to right (big endian)
  uint64_t *a_as_u64_array = (uint64_t *)a;
  uint64_t *b_as_u64_array = (uint64_t *)b;
  for (int i = 0; i < 4; i++) {
    if (a_as_u64_array[i] != b_as_u64_array[i]) {
      return 0; // a != b
    }
  }
  return 1; // a == b
}

BOOL wrapper_bytes32_slt(const bytes32 *a, const bytes32 *b) {
  // Check sign bits (most significant bit of the first byte)
  const uint8_t *a_bytes = (const uint8_t *)a;
  const uint8_t *b_bytes = (const uint8_t *)b;
  int sign_a = (a_bytes[0] & 0x80) != 0;
  int sign_b = (b_bytes[0] & 0x80) != 0;

  if (sign_a != sign_b) {
    return sign_a; // If signs differ, the negative number (sign=1) is smaller
  }

  // If signs are the same, perform comparison based on magnitude
  // For signed comparison, the rest of the bytes are compared normally
  for (int i = 0; i < 32; i++) {
    if (a[i] < b[i]) {
      return 1; // a is smaller
    } else if (a[i] > b[i]) {
      return 0; // a is larger
    }
  }

  return 0; // a == b
}

BOOL wrapper_bytes32_sgt(const bytes32 *a, const bytes32 *b) {
  // Check sign bits
  const uint8_t *a_bytes = (const uint8_t *)a;
  const uint8_t *b_bytes = (const uint8_t *)b;
  int sign_a = (a_bytes[0] & 0x80) != 0;
  int sign_b = (b_bytes[0] & 0x80) != 0;

  if (sign_a != sign_b) {
    return sign_b; // If signs differ, the positive number (sign=0) is larger
  }

  // If signs are the same, perform comparison based on magnitude
  for (int i = 0; i < 32; i++) {
    if (a[i] > b[i]) {
      return 1; // a is larger
    } else if (a[i] < b[i]) {
      return 0; // a is smaller
    }
  }

  return 0; // a == b
}

BOOL wrapper_bytes32_iszero(const bytes32 *a) {
  // Check if all bytes in a are zero
  // Returns 1 if a is zero, 0 otherwise
  const int64_t *ptr = (const int64_t *)a;
  for (int i = 0; i < 4; i++) {
    if (ptr[i] != 0) {
      return 0; // a is not zero
    }
  }
  return 1; // a == 0
}

void wrapper_bytes32_not(const bytes32 *a, bytes32 *result) {
  // Cast bytes32 pointers to uint64_t pointers to process 8 bytes at a time
  const uint64_t *ptr_a = (const uint64_t *)a;
  uint64_t *ptr_result = (uint64_t *)result;
  // Process 4 uint64_t values (32 bytes total)
  for (int i = 0; i < 4; i++) {
    ptr_result[i] = ~ptr_a[i];
  }
}

void wrapper_bytes32_and(const bytes32 *a, const bytes32 *b, bytes32 *result) {
  // Cast bytes32 pointers to uint64_t pointers to process 8 bytes at a time
  const uint64_t *ptr_a = (const uint64_t *)a;
  const uint64_t *ptr_b = (const uint64_t *)b;
  uint64_t *ptr_result = (uint64_t *)result;
  // Process 4 uint64_t values (32 bytes total)
  for (int i = 0; i < 4; i++) {
    ptr_result[i] = ptr_a[i] & ptr_b[i];
  }
}

void wrapper_bytes32_or(const bytes32 *a, const bytes32 *b, bytes32 *result) {
  // Cast bytes32 pointers to uint64_t pointers to process 8 bytes at a time
  const uint64_t *ptr_a = (const uint64_t *)a;
  const uint64_t *ptr_b = (const uint64_t *)b;
  uint64_t *ptr_result = (uint64_t *)result;
  // Process 4 uint64_t values (32 bytes total)
  for (int i = 0; i < 4; i++) {
    ptr_result[i] = ptr_a[i] | ptr_b[i];
  }
}

void wrapper_bytes32_xor(const bytes32 *a, const bytes32 *b, bytes32 *result) {
  // Cast bytes32 pointers to uint64_t pointers to process 8 bytes at a time
  const uint64_t *ptr_a = (const uint64_t *)a;
  const uint64_t *ptr_b = (const uint64_t *)b;
  uint64_t *ptr_result = (uint64_t *)result;
  // Process 4 uint64_t values (32 bytes total)
  for (int i = 0; i < 4; i++) {
    ptr_result[i] = ptr_a[i] ^ ptr_b[i];
  }
}

void wrapper_bytes32_shl(const bytes32 *value, int32_t shift, bytes32 *result) {
  if (shift <= 0) {
    memcpy(result, value, sizeof(bytes32));
    return;
  }
  if (shift <= 64) {
    // when shift <= 32 and value top 3*64 + 32 bits are all 0
    uint64_t *value_as_u64_array = (uint64_t *)value;
    uint32_t *value_as_u32_array = (uint32_t *)value;
    if (shift <= 32 && value_as_u64_array[0] == 0 &&
        value_as_u64_array[1] == 0 && value_as_u64_array[2] == 0 &&
        value_as_u32_array[6] == 0) {
      uint64_t last_u64 = ((uint64_t)value_as_u32_array[7]) << shift;
      uint64_t *result_as_u64_array = (uint64_t *)result;
      result_as_u64_array[0] = 0;
      result_as_u64_array[1] = 0;
      result_as_u64_array[2] = 0;
      result_as_u64_array[3] = last_u64;
      return;
    } else {
      // TODO: optimize this
      uint256_t value_as_uint256;
      u256_from_big_endian_bytes((uint8_t *)value, &value_as_uint256);
      uint256_t result_as_uint256 = value_as_uint256 << shift;
      u256_to_big_endian_bytes(&result_as_uint256, (uint8_t *)result);
    }

    return;
  }
  if (shift >= 256) {
    __memset(result, 0, sizeof(bytes32));
    return;
  }
  // If shift count > 64, we can first perform several 64-bit shifts. The 64-bit
  // shifts can be done directly on a temporary int64_t[4] array.
  uint64_t *value_as_u64_array = (uint64_t *)value;

  int32_t shift_64_count = shift / 64;
  int32_t shift_remainer = shift % 64;

  uint64_t shift_partial[4];
  for (int i = 0; i < 4 - shift_64_count; i++) {
    shift_partial[i] = value_as_u64_array[i + shift_64_count];
  }
  for (int i = 4 - shift_64_count; i < 4; i++) {
    shift_partial[i] = 0;
  }
  return wrapper_bytes32_shl((const bytes32 *)shift_partial, shift_remainer,
                             result);
}

void wrapper_u256_shl(const uint256_t *value, int32_t shift,
                      uint256_t *result) {
  if (shift <= 0) {
    *result = *value;
    return;
  }
  if (shift >= 256) {
    *result = 0;
    return;
  }
  *result = *value << shift;
}

void wrapper_bytes32_shr(const bytes32 *value, int32_t shift, bytes32 *result) {
  if (shift <= 0) {
    memcpy(result, value, sizeof(bytes32));
    return;
  }
  if (shift >= 256) {
    __memset(result, 0, sizeof(bytes32));
    return;
  }

  const uint8_t *val_bytes = (const uint8_t *)value;
  uint8_t *res_bytes = (uint8_t *)result;
  int byte_shift = shift / 8;
  int bit_shift = shift % 8;

  __memset(res_bytes, 0, sizeof(bytes32));

  for (int i = 0; i < 32 - byte_shift; i++) {
    uint16_t current_byte = val_bytes[i];
    uint16_t shifted_chunk = current_byte >> bit_shift;

    res_bytes[i + byte_shift] |= (shifted_chunk & 0xFF);
    if (i > 0 && bit_shift > 0) { // Add carry from previous byte's shift
      uint16_t carry_byte = val_bytes[i - 1];
      uint16_t carry_shifted =
          (carry_byte << (8 - bit_shift)) & 0xFF; // Get bits shifted out
      res_bytes[i + byte_shift] |= carry_shifted;
    }
  }
}

void wrapper_u256_shr(const uint256_t *value, int32_t shift,
                      uint256_t *result) {
  if (shift <= 0) {
    *result = *value;
    return;
  }
  if (shift >= 256) {
    *result = 0;
    return;
  }
  *result = *value >> shift;
}

void wrapper_bytes32_add(const bytes32 *a, const bytes32 *b, bytes32 *result) {
  // big endian bytes32 add
  uint8_t *a_bytes = (uint8_t *)a;
  uint8_t *b_bytes = (uint8_t *)b;
  uint8_t *result_bytes = (uint8_t *)result;
  uint8_t carry = 0;
  for (int i = 31; i >= 0; i--) {
    uint32_t sum =
        (uint32_t)a_bytes[i] + (uint32_t)b_bytes[i] + (uint32_t)carry;
    result_bytes[i] = (uint8_t)(sum & 0xff);
    carry = sum >> 8; // Check for overflow, 1 or 0
  }
}
