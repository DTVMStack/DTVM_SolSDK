CC=clang
DEBUG_FLAGS=-g -O0
RELEASE_FLAGS=-O2 -DNDEBUG
COMMON_FLAGS=$(TARGET_FLAGS) --target=wasm32 -emit-llvm -Xclang -fexperimental-max-bitint-width=256 -ffreestanding -fno-builtin -Wall -Wno-unused-function -ftrapv -v

wasm/debug/%.bc: %.c Makefile | wasm/debug
	$(CC) -c $(CFLAGS) $< -o $@

wasm/release/%.bc: %.c Makefile | wasm/release
	$(CC) -c $(CFLAGS) $< -o $@

wasm/debug:
	mkdir -p wasm/debug

wasm/release:
	mkdir -p wasm/release

WASM_DEBUG=$(addprefix wasm/debug/,stdlib.bc)              \
           $(addprefix wasm/debug/,chain.bc)               \
           $(addprefix wasm/debug/,utils.bc)               \
		   $(addprefix wasm/debug/,chain_math.bc)          \
           $(addprefix wasm/debug/,evm_memory.bc)

WASM_RELEASE=$(addprefix wasm/release/,stdlib.bc)          \
             $(addprefix wasm/release/,chain.bc)           \
             $(addprefix wasm/release/,utils.bc)           \
             $(addprefix wasm/release/,chain_math.bc)      \
             $(addprefix wasm/release/,evm_memory.bc)      \
             $(addprefix wasm/release/,debug_in_release.bc)

all: Makefile echo

debug: CFLAGS=$(COMMON_FLAGS) $(DEBUG_FLAGS)
debug: $(WASM_DEBUG) Makefile echo

release: CFLAGS=$(COMMON_FLAGS) $(RELEASE_FLAGS)
release: $(WASM_RELEASE) Makefile echo

echo:
	echo "stdlib make done"

clean:
	rm -rf wasm/debug/*.bc wasm/release/*.bc wasm/debug wasm/release

fmt:
	clang-format -i *.h *.c

fmt_check:
	clang-format --dry-run --Werror *.h *.c

.DEFAULT_GOAL := error

error:
	@echo "Error: You must specify either 'make debug/release/clean/fmt/clang-format'"
	@exit 1
