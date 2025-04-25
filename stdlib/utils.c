// Copyright (c) the DTVM authors Core Contributors
// SPDX-License-Identifier: Apache-2.0

#include "utils.h"
#include "debug.h"
// Convert int32 to decimal string
// Returns the string length
int int32_to_str(int32_t value, char *str_buffer, int buffer_size) {
  int index = 0;
  int is_negative = 0;

  // Handle negative numbers
  if (value < 0) {
    is_negative = 1;
    value = -value; // Convert to positive for processing
  }

  // Special case for zero
  if (value == 0) {
    if (buffer_size > 0) {
      str_buffer[index++] = '0';
    }
  } else {
    // Convert from least significant digit to most significant
    while (value > 0 && index < buffer_size - 1) {
      str_buffer[index++] = '0' + (value % 10);
      value /= 10;
    }

    // Add negative sign if needed
    if (is_negative && index < buffer_size - 1) {
      str_buffer[index++] = '-';
    }

    // Reverse the string
    for (int j = 0; j < index / 2; j++) {
      char temp = str_buffer[j];
      str_buffer[j] = str_buffer[index - 1 - j];
      str_buffer[index - 1 - j] = temp;
    }
  }

  // Add string terminator
  if (index < buffer_size) {
    str_buffer[index] = '\0';
  }

  return index;
}

void debug_i32(int32_t value) {
#ifndef NDEBUG
  char str_buffer[12]; // 32-bit integer max 10 digits + possible minus sign +
                       // terminator
  int len = int32_to_str(value, str_buffer, sizeof(str_buffer));

  // Print bytes using debug_bytes
  debug_bytes((ADDRESS_UINT)&str_buffer, len);
#endif // NDEBUG
}

void debug_string(const char *str) {
#ifndef NDEBUG
  uint32_t str_len = 0;
  for (int i = 0;; i++) {
    if (str[i] == '\0') {
      break;
    }
    str_len++;
  }
  debug_bytes((ADDRESS_UINT)str, str_len);
#endif // NDEBUG
}
