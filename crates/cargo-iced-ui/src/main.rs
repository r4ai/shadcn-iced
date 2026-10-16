//! `cargo iced-ui`: copy iced_ui components into your project and own them,
//! like `npx shadcn add`.
//!
//! ```text
//! cargo iced-ui list
//! cargo iced-ui add button card dialog
//! cargo iced-ui add --all
//! cargo iced-ui diff button
//! ```
mod registry {
    include!(concat!(env!("OUT_DIR"), "/registry.rs"));
}

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::{env, fs};

const DEFAULT_DIR: &str = "src/ui";

const HELP: &str = "\
cargo-iced-ui: copy iced_ui components into your project

USAGE:
    cargo iced-ui <COMMAND> [OPTIONS]

COMMANDS:
    list                  List the available components
    add <NAME>...         Copy components (and the components they use)
    add --all             Copy every component
    diff <NAME>...        Show which local copies differ from the registry
    view <NAME>           Print the source of a component

OPTIONS:
    --dir <PATH>          Where components go [default: src/ui]
    --overwrite           Replace files that already exist
    -h, --help            Print help

Copied components depend on `iced` and `iced_ui_core`:

    cargo add iced --features advanced,canvas,image,svg
    cargo add iced_ui_core

Then declare the module (`mod ui;`) and load the fonts with
`iced_ui_core::ApplicationExt::iced_ui`.";

fn main() -> ExitCode {
    // When invoked as `cargo iced-ui`, cargo passes `iced-ui` first.
    let mut args: Vec<String> = env::args().skip(1).collect();
    if args.first().is_some_and(|arg| arg == "iced-ui") {
        args.remove(0);
    }

    match run(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

struct Options {
    dir: PathBuf,
    overwrite: bool,
    all: bool,
    names: Vec<String>,
}

fn parse(args: &[String]) -> Result<Options, String> {
    let mut options = Options {
        dir: PathBuf::from(DEFAULT_DIR),
        overwrite: false,
        all: false,
        names: Vec::new(),
    };

    let mut args = args.iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--dir" => options.dir = PathBuf::from(args.next().ok_or("--dir needs a path")?),
            "--overwrite" => options.overwrite = true,
            "--all" => options.all = true,
            flag if flag.starts_with('-') => return Err(format!("unknown option `{flag}`")),
            name => options.names.push(name.replace('-', "_")),
        }
    }

    Ok(options)
}

fn run(args: Vec<String>) -> Result<(), String> {
    let Some((command, rest)) = args.split_first() else {
        println!("{HELP}");
        return Ok(());
    };

    match command.as_str() {
        "-h" | "--help" | "help" => {
            println!("{HELP}");
            Ok(())
        }
        "list" => {
            list();
            Ok(())
        }
        "add" => add(parse(rest)?),
        "diff" => diff(parse(rest)?),
        "view" => {
            let options = parse(rest)?;
            let name = options.names.first().ok_or("which component?")?;
            println!("{}", source(name)?);
            Ok(())
        }
        other => Err(format!(
            "unknown command `{other}`; see `cargo iced-ui --help`"
        )),
    }
}

fn source(name: &str) -> Result<&'static str, String> {
    registry::COMPONENTS
        .iter()
        .find(|(candidate, _)| *candidate == name)
        .map(|(_, source)| *source)
        .ok_or_else(|| format!("unknown component `{name}`; see `cargo iced-ui list`"))
}

/// The first paragraph of the module documentation of a component.
fn summary(source: &str) -> String {
    source
        .lines()
        .map_while(|line| line.strip_prefix("//! "))
        .collect::<Vec<_>>()
        .join(" ")
}

/// The sibling components a component uses, through `super::`.
fn dependencies(source: &str) -> BTreeSet<&'static str> {
    let code: String = source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    let source = code.as_str();

    registry::COMPONENTS
        .iter()
        .map(|(name, _)| *name)
        .filter(|name| {
            let pattern = format!("super::{name}");
            source.match_indices(&pattern).any(|(index, _)| {
                !source[index + pattern.len()..]
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_alphanumeric() || c == '_')
            })
        })
        .collect()
}

/// Resolves a set of components and everything they depend on.
fn resolve(names: &[String]) -> Result<BTreeSet<&'static str>, String> {
    let mut resolved = BTreeSet::new();
    let mut queue: Vec<&'static str> = names
        .iter()
        .map(|name| {
            registry::COMPONENTS
                .iter()
                .find(|(candidate, _)| candidate == name)
                .map(|(name, _)| *name)
                .ok_or_else(|| format!("unknown component `{name}`; see `cargo iced-ui list`"))
        })
        .collect::<Result<_, _>>()?;

    while let Some(name) = queue.pop() {
        if resolved.insert(name) {
            queue.extend(dependencies(source(name)?));
        }
    }

    Ok(resolved)
}

fn list() {
    let width = registry::COMPONENTS
        .iter()
        .map(|(name, _)| name.len())
        .max()
        .unwrap_or(0);

    for (name, source) in registry::COMPONENTS {
        println!("{:width$}  {}", name.replace('_', "-"), summary(source));
    }
}

/// Adapts a component for a project that does not depend on `iced_ui`:
/// documentation examples refer to `iced_ui`, so they must not be compiled.
fn localize(source: &str) -> String {
    source.replace("//! ```no_run", "//! ```ignore")
}

fn add(options: Options) -> Result<(), String> {
    let names: Vec<String> = if options.all {
        registry::COMPONENTS
            .iter()
            .map(|(name, _)| (*name).to_owned())
            .collect()
    } else {
        options.names.clone()
    };

    if names.is_empty() {
        return Err("which components? e.g. `cargo iced-ui add button`".to_owned());
    }

    let components = resolve(&names)?;
    fs::create_dir_all(&options.dir)
        .map_err(|error| format!("{}: {error}", options.dir.display()))?;

    let mut written = Vec::new();
    let mut skipped = Vec::new();

    for name in &components {
        let path = options.dir.join(format!("{name}.rs"));

        if path.exists() && !options.overwrite {
            skipped.push(*name);
            continue;
        }

        fs::write(&path, localize(source(name)?))
            .map_err(|error| format!("{}: {error}", path.display()))?;
        written.push(*name);
    }

    update_mod(&options.dir, &components)?;

    for name in &written {
        println!(
            "  added  {}",
            options.dir.join(format!("{name}.rs")).display()
        );
    }

    for name in &skipped {
        println!(
            "  exists {} (use --overwrite to replace it)",
            options.dir.join(format!("{name}.rs")).display()
        );
    }

    if !has_dependency("iced_ui_core") {
        println!("\nThe components need `iced_ui_core`:\n    cargo add iced_ui_core");
    }

    Ok(())
}

/// Adds `pub mod name;` to the `mod.rs` of the directory, creating it if
/// needed.
fn update_mod(dir: &Path, components: &BTreeSet<&str>) -> Result<(), String> {
    let path = dir.join("mod.rs");
    let existing = fs::read_to_string(&path).unwrap_or_else(|_| {
        "//! UI components copied from iced_ui with `cargo iced-ui add`.\n//!\n//! You own this code: edit it as you wish.\n#![allow(dead_code, unused_imports)]\n\n".to_owned()
    });

    let declared: BTreeSet<String> = existing
        .lines()
        .filter_map(|line| line.trim().strip_prefix("pub mod "))
        .map(|rest| rest.trim_end_matches(';').trim().to_owned())
        .collect();

    let mut content = existing.clone();
    let missing: Vec<_> = components
        .iter()
        .filter(|name| !declared.contains(**name))
        .collect();

    if missing.is_empty() && path.exists() {
        return Ok(());
    }

    if !content.ends_with('\n') {
        content.push('\n');
    }

    for name in missing {
        content.push_str(&format!("pub mod {name};\n"));
    }

    fs::write(&path, content).map_err(|error| format!("{}: {error}", path.display()))
}

fn has_dependency(name: &str) -> bool {
    fs::read_to_string("Cargo.toml").is_ok_and(|manifest| manifest.contains(name))
}

fn diff(options: Options) -> Result<(), String> {
    let names: Vec<String> = if options.names.is_empty() || options.all {
        registry::COMPONENTS
            .iter()
            .map(|(name, _)| (*name).to_owned())
            .filter(|name| options.dir.join(format!("{name}.rs")).exists())
            .collect()
    } else {
        options.names.clone()
    };

    for name in names {
        let path = options.dir.join(format!("{name}.rs"));
        let upstream = localize(source(&name)?);

        match fs::read_to_string(&path) {
            Ok(local) if local == upstream => println!("  same     {name}"),
            Ok(local) => {
                let changed = local
                    .lines()
                    .zip(upstream.lines())
                    .filter(|(a, b)| a != b)
                    .count()
                    + local.lines().count().abs_diff(upstream.lines().count());
                println!("  modified {name} (~{changed} lines differ)");
            }
            Err(_) => println!("  missing  {name}"),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_is_complete() {
        assert!(registry::COMPONENTS.len() >= 60);
        assert!(source("button").is_ok());
    }

    #[test]
    fn resolves_dependencies() {
        let resolved = resolve(&["button".to_owned()]).unwrap();
        assert_eq!(resolved.len(), 1, "{resolved:?}");

        let resolved = resolve(&["pagination".to_owned()]).unwrap();
        assert!(resolved.contains("button"));
        assert!(resolved.contains("pagination"));

        let resolved = resolve(&["combobox".to_owned()]).unwrap();
        for dependency in [
            "command",
            "button",
            "menu",
            "input",
            "popover",
            "separator",
            "scroll_area",
        ] {
            assert!(resolved.contains(dependency), "{dependency}");
        }
    }

    #[test]
    fn every_component_only_uses_known_siblings() {
        for (name, source) in registry::COMPONENTS {
            for line in source
                .lines()
                .filter(|line| !line.trim_start().starts_with("//"))
            {
                assert!(!line.contains("crate::"), "{name} uses `crate::`: {line}");
                assert!(!line.contains("iced_ui::"), "{name} uses `iced_ui`: {line}");
            }
        }
    }
}
