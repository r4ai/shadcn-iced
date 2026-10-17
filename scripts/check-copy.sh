#!/usr/bin/env bash
# Copies every component into a fresh project with `cargo shadcn-iced add --all`
# and checks that it compiles with only `iced` and `shadcn_iced_core`.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
project="$(mktemp -d)"
trap 'rm -rf "$project"' EXIT

cargo build -q -p cargo-shadcn-iced --manifest-path "$root/Cargo.toml"

mkdir -p "$project/src"
cat > "$project/Cargo.toml" <<TOML
[package]
name = "copy-check"
version = "0.0.0"
edition = "2024"
publish = false

[dependencies]
iced = { version = "0.14", features = ["advanced", "canvas", "image", "svg"] }
shadcn_iced_core = { path = "$root/crates/shadcn_iced_core" }

[workspace]
TOML

cat > "$project/src/main.rs" <<'RUST'
mod ui;

fn main() {}
RUST

(cd "$project" && "$root/target/debug/cargo-shadcn-iced" add --all > /dev/null)
(cd "$project" && CARGO_TARGET_DIR="$root/target" RUSTFLAGS="-D warnings" cargo check -q)

echo "every component compiles on its own ✔"
