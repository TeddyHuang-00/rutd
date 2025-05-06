# Pre-release to github only
[working-directory: '.']
pre-release LEVEL: test
    cargo release {{LEVEL}} --workspace {{EXCLUDE_OPT}} -x --no-publish

# Release
[working-directory: '.']
release: test
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