# Temporarily exclude rutd-tui from release builds since it is still WIP
RELEASE_OPT := "--exclude rutd-tui"

# Format code
format:
    cargo +nightly fmt --all

# Check unused dependencies
deps:
    cargo +nightly udeps --workspace --all-targets

# Check for errors
check: format
    cargo clippy --workspace --all-targets --fix --allow-staged
    @just format

# Unit tests
test: check
    cargo test --workspace --all-targets

# Test release
[no-cd]
release-test TARGET:
    cargo release {{TARGET}} --workspace {{RELEASE_OPT}}

# Release
[no-cd]
release TARGET:
    #!/usr/bin/bash
    just release-test {{TARGET}}
    echo "Do you want to continue publishing the release? (y/n)"
    cargo release {{TARGET}} --workspace {{RELEASE_OPT}} -x
