// Copyright (c) The Ant Group Core Contributors
// SPDX-License-Identifier: Apache-2.0

#ifndef __DEBUG_H_
#define __DEBUG_H_
#include "hostapi.h"

__attribute__((import_module("env"), import_name("debug_bytes"))) void
debug_bytes(ADDRESS_UINT data_offset, int32_t data_length);

#endif // __DEBUG_H_
