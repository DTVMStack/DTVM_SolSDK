// Copyright (c) the DTVM authors Core Contributors
// SPDX-License-Identifier: Apache-2.0

#ifndef __EVM_MEMORY_H
#define __EVM_MEMORY_H
#include <stdint.h>

#define WASM_PAGE_SIZE (65536)

// TODO: use revert with not malloc for abort
#define __abort(x) __builtin_unreachable()

void __init_evm_heap(int32_t try_new_wasm_page_as_evm_heap_bool);

// Ensures there is memory space of specified size available for EVM use
// Memory beyond the specified size requires additional checks to prevent access
// Returns the starting address of the EVM memory region
// The starting address of the EVM memory region remains unchanged after
// modifications
uint8_t *evm_make_sure_memory(uint32_t size);

// Returns the memory address at the given offset
// If offset < 0, copies data from non-EVM memory region
uint8_t *evm_get_memory_addr(int32_t offset);

// Checks if ptr is within the accessible memory range of EVM
// Returns 1(true) if accessible, 0(false) otherwise
int32_t is_available_evm_memory(uint8_t *ptr, uint32_t size);

#endif // __EVM_MEMORY_H
