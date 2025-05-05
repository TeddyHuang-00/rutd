# Temporarily exclude rutd-tui since it is still WIP
EXCLUDE_OPT := "--exclude rutd-tui"
WORKSPACE_FLAGS := "--workspace --all-targets"

import "recipes/demo.just"

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
    cargo test {{WORKSPACE_FLAGS}} -- --test-threads=1

# Coverage report
coverage: check
    cargo tarpaulin {{WORKSPACE_FLAGS}} {{EXCLUDE_OPT}} --out Html --output-dir coverage

# Release
[no-cd]
release:
    #!/usr/bin/bash
    VERSION=$(git cliff --bump --bumped-version)
    cargo release "${VERSION#v}" --workspace {{EXCLUDE_OPT}}
    echo "Will bump version to $VERSION"
    read -p "Continue? [y/N] " yn;
        if [ "$yn" = "y" ]; then
            echo "Bumping version...";
        else
            echo "Aborting.";
            exit 1;
        fi
    git cliff --tag-pattern "^v[0-9]+.[0-9]+.[0-9]+$" --bump -o CHANGELOG.md
    git add CHANGELOG.md
    git commit -m "chore: update changelog"
    cargo release "${VERSION#v}" --workspace {{EXCLUDE_OPT}} -x
