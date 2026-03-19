default:
    just --list

# Usage: just tag_and_release
tag: tag_and_release

tag_and_release:
    sh tag_and_release.sh

sync_readme:
    cp README.md npm/README.md
