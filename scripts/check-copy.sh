#!/usr/bin/env bash
# Copies every component into a fresh project with `cargo iced-ui add --all`
# and checks that it compiles with only `iced` and `iced_ui_core`.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
project="$(mktemp -d)"
trap 'rm -rf "$project"' EXIT

cargo build -q -p cargo-iced-ui --manifest-path "$root/Cargo.toml"

mkdir -p "$project/src"
cat > "$project/Cargo.toml" <<TOML
[package]
name = "copy-check"
version = "0.0.0"
edition = "2024"
publish = false

[dependencies]
iced = { version = "0.14", features = ["advanced", "canvas", "image", "svg"] }
iced_ui_core = { path = "$root/crates/iced_ui_core" }

[workspace]
TOML

cat > "$project/src/main.rs" <<'RUST'
mod ui;

fn main() {}
RUST

(cd "$project" && "$root/target/debug/cargo-iced-ui" add --all > /dev/null)
(cd "$project" && CARGO_TARGET_DIR="$root/target" RUSTFLAGS="-D warnings" cargo check -q)

echo "every component compiles on its own ✔"
