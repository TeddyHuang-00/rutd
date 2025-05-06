# Temporarily exclude rutd-tui since it is still WIP
EXCLUDE_OPT := "--exclude rutd-tui"
WORKSPACE_FLAGS := "--workspace --all-targets --locked"

import "recipes/demo.just"
import "recipes/release.just"

# Format code
format:
    cargo +nightly fmt --all
    cargo sort --workspace
    cargo sort-derives

# Check unused dependencies
deps:
    cargo +nightly udeps {{WORKSPACE_FLAGS}}

# Check for errors
check: format
    cargo clippy {{WORKSPACE_FLAGS}} --fix --allow-staged
    @just format

# Unit tests
test: check
    cargo test {{WORKSPACE_FLAGS}} -- --test-threads=1

# Coverage report
coverage: check
    cargo tarpaulin {{WORKSPACE_FLAGS}} {{EXCLUDE_OPT}} --out Html --output-dir coverage

