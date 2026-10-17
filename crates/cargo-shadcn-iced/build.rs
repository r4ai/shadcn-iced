//! Embeds the component sources of `shadcn_iced` into the CLI.
//!
//! In the workspace, they come from `shadcn_iced` itself. The published crate
//! cannot reach its sibling, so `scripts/prepare-publish.sh` copies them into
//! `components/` before `cargo publish`.
use std::path::PathBuf;
use std::{env, fs};

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace = manifest.join("../shadcn_iced/src/components");
    let components = if workspace.is_dir() {
        workspace
    } else {
        manifest.join("components")
    };

    println!("cargo:rerun-if-changed={}", components.display());

    let mut entries: Vec<_> = fs::read_dir(&components)
        .expect("shadcn_iced components")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
        .filter(|path| path.file_stem().is_some_and(|stem| stem != "mod"))
        .collect();

    entries.sort();

    let mut registry = String::from(
        "/// Every component as `(module name, source)`.\npub const COMPONENTS: &[(&str, &str)] = &[\n",
    );

    for path in entries {
        let name = path.file_stem().unwrap().to_string_lossy();
        println!("cargo:rerun-if-changed={}", path.display());
        registry.push_str(&format!(
            "    ({name:?}, include_str!({:?})),\n",
            path.canonicalize().unwrap()
        ));
    }

    registry.push_str("];\n");

    fs::write(
        PathBuf::from(env::var("OUT_DIR").unwrap()).join("registry.rs"),
        registry,
    )
    .unwrap();
}
