// Copyright (c) the DTVM authors Core Contributors
// SPDX-License-Identifier: Apache-2.0

#include "evm_memory.h"
#include "debug.h"

static int32_t inited_before = 0; // 1 is true, 0 is false

// EVM memory start address,
// Once set, it remains unchanged. This is possible because after the WASM
// instance starts (before optimization), the newly grown memory area is treated
// as EVM memory, preventing conflicts with the LLVM constant pool.
static uint8_t *evm_memory_begin = 0;
static uint8_t *evm_memory_end = 0;
static uint32_t evm_memory_size = 0; // accessible memory range
static uint32_t evm_memory_full_size = 0;

// In wasm-ld mode, it is possible to avoid allocating separate pages and
// instead use the remaining space after the constant data segment on the stack.
// Another approach is to use static allocation(current choice)
// Allocate a data segment and point to it, allowing space without needing to
// grow; in most cases, 16384 (16k) is sufficient.
// TODO: Implement memory relocation and growth when exceeding limits
#define EVM_MEMORY_MAX_SIZE 16384 // 16KB
static uint8_t evm_memory_init_buffer[EVM_MEMORY_MAX_SIZE] = {0};

void __init_evm_heap(int32_t try_new_wasm_page_as_evm_heap_bool) {
  if (inited_before) {
    // disable re-call abi using the same vm(memory not free)
    return;
  }

  // if current is large module(init memory size > 1 page), we use the
  // evm_memory_begin in new page else we use the evm_memory_init_buffer in
  // stack
  uint32_t init_memory_pages = __builtin_wasm_memory_size(0);
  if (init_memory_pages > 1 || try_new_wasm_page_as_evm_heap_bool) {
    evm_memory_begin = (uint8_t *)(init_memory_pages * WASM_PAGE_SIZE);
    // Preallocate some memory, as it is generally needed.
    __builtin_wasm_memory_grow(0, 1);
    uint32_t after_pages = __builtin_wasm_memory_size(0);
    if (after_pages == init_memory_pages) {
      __abort("wasm memory grow failed");
    }
    evm_memory_end = (uint8_t *)(after_pages * WASM_PAGE_SIZE);
    evm_memory_size = 0;
    evm_memory_full_size =
        (uint32_t)evm_memory_end - (uint32_t)evm_memory_begin;
  } else {
    // small contract
    evm_memory_begin = evm_memory_init_buffer;
    evm_memory_end = evm_memory_begin + EVM_MEMORY_MAX_SIZE;
    evm_memory_size = 0;
    evm_memory_full_size = EVM_MEMORY_MAX_SIZE;
  }

  inited_before = 1;
}

uint8_t *evm_make_sure_memory(uint32_t size) {
  if (evm_memory_full_size < size) {
    uint32_t target_evm_pages = (size + WASM_PAGE_SIZE - 1) / WASM_PAGE_SIZE;
    uint32_t grow_pages =
        target_evm_pages - evm_memory_full_size / WASM_PAGE_SIZE;
    if (__builtin_wasm_memory_grow(0, grow_pages) == -1) {
      __abort("__malloc: failed");
    };
    evm_memory_end += (grow_pages * WASM_PAGE_SIZE);
    evm_memory_size = size;
    evm_memory_full_size = target_evm_pages * WASM_PAGE_SIZE;
    return evm_memory_begin;
  }
  if (evm_memory_size < size) {
    // EVM memory is allocated but not accessible for the requested size
    evm_memory_size = size;
  }

  return evm_memory_begin;
}

uint8_t *evm_get_memory_addr(int32_t offset) {
  return evm_memory_begin + offset;
}

int32_t is_available_evm_memory(uint8_t *ptr, uint32_t size) {
  return ptr >= evm_memory_begin && size <= evm_memory_size;
}
