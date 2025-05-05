# Temporarily exclude rutd-tui since it is still WIP
EXCLUDE_OPT := "--exclude rutd-tui"
WORKSPACE_FLAGS := "--workspace --all-targets"

# Format code
format:
    cargo +nightly fmt --all

# Check unused dependencies
deps:
    cargo +nightly udeps {{WORKSPACE_FLAGS}}

# Check for errors
check: format
    cargo clippy {{WORKSPACE_FLAGS}} --fix --allow-staged
    @just format

# Unit tests
test: check
    cargo test {{WORKSPACE_FLAGS}}

# Coverage report
coverage: check
    cargo tarpaulin {{WORKSPACE_FLAGS}} {{EXCLUDE_OPT}} --out Html --output-dir coverage

# Test release
[no-cd]
release-test TARGET:
    cargo release {{TARGET}} --workspace {{EXCLUDE_OPT}}

# Release
[no-cd]
release TARGET:
    just release-test {{TARGET}}
    echo "Do you want to continue publishing the following releases?"
    cargo release {{TARGET}} --workspace {{EXCLUDE_OPT}} -x
