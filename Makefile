# cqlsh-rs build targets for distribution
#
# Release binaries for Linux are statically linked via musl to avoid glibc
# version requirements. This ensures compatibility with RHEL 9+ (glibc 2.34),
# Ubuntu 22.04+, and container images like ubi9-minimal.
#
# Usage:
#   make release-linux-x86_64    # static x86_64 binary
#   make release-linux-aarch64   # static aarch64 binary (requires cross)
#   make release-linux           # both architectures

CARGO ?= cargo
CROSS ?= cross
BINARY_NAME := cqlsh-rs

# Default target
.PHONY: build
build:
	$(CARGO) build --release

# --- Static Linux builds (musl) ---

.PHONY: release-linux-x86_64
release-linux-x86_64:
	rustup target add x86_64-unknown-linux-musl
	$(CARGO) build --release --target x86_64-unknown-linux-musl
	@echo ""
	@echo "Binary: target/x86_64-unknown-linux-musl/release/$(BINARY_NAME)"
	@echo "  Statically linked — no glibc dependency, runs on any Linux x86_64"

.PHONY: release-linux-aarch64
release-linux-aarch64:
	$(CROSS) build --release --target aarch64-unknown-linux-musl
	@echo ""
	@echo "Binary: target/aarch64-unknown-linux-musl/release/$(BINARY_NAME)"
	@echo "  Statically linked — no glibc dependency, runs on any Linux aarch64"

.PHONY: release-linux
release-linux: release-linux-x86_64 release-linux-aarch64

# --- Development helpers ---

.PHONY: test
test:
	$(CARGO) test

.PHONY: clippy
clippy:
	$(CARGO) clippy --all-targets --all-features

.PHONY: fmt
fmt:
	$(CARGO) fmt --all

.PHONY: check
check: fmt clippy test

.PHONY: clean
clean:
	$(CARGO) clean
