#!/usr/bin/env bash
# Prepares the crates for `cargo publish` (`release-plz`, see
# `.github/workflows/release.yml`): the published `cargo-shadcn-iced` cannot
# reach the components of its sibling `shadcn_iced`, so they are copied into
# it (`crates/cargo-shadcn-iced/build.rs`). The copy is not committed, so
# `cargo publish -p cargo-shadcn-iced` needs `--allow-dirty`.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
dest="$root/crates/cargo-shadcn-iced/components"

rm -rf "$dest"
mkdir -p "$dest"
cp "$root"/crates/shadcn_iced/src/components/*.rs "$dest/"
