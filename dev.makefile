# if use makefile, the wizer lib will not work well
# Default target
all: fmt_check test

# Format code using format.sh script
fmt:
	./tools/format.sh format

# Check code format using format.sh script
fmt_check:
	./tools/format.sh check

# Run unit tests
test:
	cargo test -- --nocapture

# Build release version
release:
	cd stdlib && make release
	cargo build --release --features release

# Build debug version
debug:
	cd stdlib && make debug
	cargo build

# Clean build artifacts
clean:
	cargo clean
	cd stdlib && make clean

.PHONY: all fmt fmt_check test release debug clean
