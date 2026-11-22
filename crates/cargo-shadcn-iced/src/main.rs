//! `cargo shadcn-iced`: copy shadcn_iced components into your project and own them,
//! like `npx shadcn add`.
//!
//! ```text
//! cargo shadcn-iced list
//! cargo shadcn-iced add button card dialog
//! cargo shadcn-iced add --all
//! cargo shadcn-iced diff button
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
cargo-shadcn-iced: copy shadcn_iced components into your project

USAGE:
    cargo shadcn-iced <COMMAND> [OPTIONS]

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

Copied components depend on `iced` and `shadcn_iced_core`:

    cargo add iced --features advanced,canvas,image,svg
    cargo add shadcn_iced_core

Then declare the module (`mod ui;`) and load the fonts with
`shadcn_iced_core::ApplicationExt::shadcn_iced`.";

fn main() -> ExitCode {
    // When invoked as `cargo shadcn-iced`, cargo passes `shadcn-iced` first.
    let mut args: Vec<String> = env::args().skip(1).collect();
    if args.first().is_some_and(|arg| arg == "shadcn-iced") {
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
            "unknown command `{other}`; see `cargo shadcn-iced --help`"
        )),
    }
}

fn source(name: &str) -> Result<&'static str, String> {
    registry::COMPONENTS
        .iter()
        .find(|(candidate, _)| *candidate == name)
        .map(|(_, source)| *source)
        .ok_or_else(|| format!("unknown component `{name}`; see `cargo shadcn-iced list`"))
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
                .ok_or_else(|| format!("unknown component `{name}`; see `cargo shadcn-iced list`"))
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

/// Adapts a component for a project that does not depend on `shadcn_iced`:
/// documentation examples refer to `shadcn_iced`, so they must not be compiled.
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
        return Err("which components? e.g. `cargo shadcn-iced add button`".to_owned());
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

    if !has_dependency("shadcn_iced_core") {
        println!("\nThe components need `shadcn_iced_core`:\n    cargo add shadcn_iced_core");
    }

    Ok(())
}

/// Adds `pub mod name;` to the `mod.rs` of the directory, creating it if
/// needed.
fn update_mod(dir: &Path, components: &BTreeSet<&str>) -> Result<(), String> {
    let path = dir.join("mod.rs");
    let existing = fs::read_to_string(&path).unwrap_or_else(|_| {
        "//! UI components copied from shadcn_iced with `cargo shadcn-iced add`.\n//!\n//! You own this code: edit it as you wish.\n#![allow(dead_code, unused_imports)]\n\n".to_owned()
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

fn diff_lines(local: &str, upstream: &str) -> usize {
    local
        .lines()
        .zip(upstream.lines())
        .filter(|(a, b)| a != b)
        .count()
        + local.lines().count().abs_diff(upstream.lines().count())
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
                let changed = diff_lines(&local, &upstream);
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
    use proptest::prelude::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn temp_dir(label: &str) -> PathBuf {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = env::temp_dir().join(format!(
            "cargo-shadcn-iced-test-{}-{id}-{label}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn registry_is_complete() {
        assert!(registry::COMPONENTS.len() >= 66);
        assert!(source("button").is_ok());
        assert!(source("does_not_exist").is_err());
    }

    #[test]
    fn resolves_dependencies() {
        let resolved = resolve(&["button".to_owned()]).unwrap();
        assert_eq!(resolved.len(), 1, "{resolved:?}");

        let resolved = resolve(&["pagination".to_owned()]).unwrap();
        assert!(resolved.contains("button"));
        assert!(resolved.contains("pagination"));

        let resolved = resolve(&["combobox".to_owned()]).unwrap();
        for dependency in ["combobox", "button", "menu", "input"] {
            assert!(resolved.contains(dependency), "{dependency}");
        }

        let resolved = resolve(&["code_block".to_owned()]).unwrap();
        for dependency in ["code_block", "button", "scroll_area"] {
            assert!(resolved.contains(dependency), "{dependency}");
        }

        let resolved = resolve(&["questionnaire".to_owned()]).unwrap();
        for dependency in ["questionnaire", "button", "input"] {
            assert!(resolved.contains(dependency), "{dependency}");
        }

        for name in ["toast", "trace"] {
            assert!(resolve(&[name.to_owned()]).unwrap().contains(name));
        }

        assert!(resolve(&["unknown_widget".to_owned()]).is_err());
    }

    #[test]
    fn every_component_only_uses_known_siblings() {
        for (name, source) in registry::COMPONENTS {
            for line in source
                .lines()
                .filter(|line| !line.trim_start().starts_with("//"))
            {
                assert!(!line.contains("crate::"), "{name} uses `crate::`: {line}");
                assert!(
                    !line.contains("shadcn_iced::"),
                    "{name} uses `shadcn_iced`: {line}"
                );
            }
        }
    }

    #[test]
    fn parse_handles_options_flags_and_errors() {
        let args: Vec<String> = [
            "--dir",
            "custom/ui",
            "--overwrite",
            "--all",
            "input-otp",
            "date-picker",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect();
        let opts = parse(&args).unwrap();
        assert_eq!(opts.dir, PathBuf::from("custom/ui"));
        assert!(opts.overwrite);
        assert!(opts.all);
        assert_eq!(opts.names, vec!["input_otp", "date_picker"]);

        assert!(parse(&["--dir".to_owned()]).is_err());
        assert!(parse(&["--bogus".to_owned()]).is_err());
        assert!(parse(&["-z".to_owned()]).is_err());
    }

    #[test]
    fn run_dispatches_all_commands() {
        assert!(run(vec![]).is_ok());
        assert!(run(vec!["-h".to_owned()]).is_ok());
        assert!(run(vec!["--help".to_owned()]).is_ok());
        assert!(run(vec!["help".to_owned()]).is_ok());
        assert!(run(vec!["list".to_owned()]).is_ok());
        assert!(run(vec!["view".to_owned(), "button".to_owned()]).is_ok());
        assert!(run(vec!["view".to_owned()]).is_err());
        assert!(run(vec!["view".to_owned(), "nope".to_owned()]).is_err());
        assert!(run(vec!["unknown".to_owned()]).is_err());
        assert!(run(vec!["add".to_owned()]).is_err());
    }

    #[test]
    fn summary_and_localize_and_dependencies_word_boundaries() {
        assert_eq!(
            summary("//! First line\n//! second line\n\n//! ignored"),
            "First line second line"
        );
        assert_eq!(summary("fn no_doc() {}"), "");

        let raw = "//! ```no_run\n//! code()\n//! ```";
        assert_eq!(localize(raw), "//! ```ignore\n//! code()\n//! ```");

        // `super::button_group` must not match `button` (word boundary check).
        let deps = dependencies("use super::button_group::ButtonGroup;\n// super::card::Card\n");
        assert!(deps.contains("button_group"));
        assert!(!deps.contains("button"));
        assert!(!deps.contains("card"));

        let deps_alnum = dependencies("use super::button2::Foo;\nuse super::button::Button;\n");
        assert!(deps_alnum.contains("button"));
    }

    #[test]
    fn add_update_mod_and_diff_lifecycle() {
        let dir = temp_dir("lifecycle");

        // Adding pagination pulls in button and creates mod.rs.
        run(vec![
            "add".to_owned(),
            "--dir".to_owned(),
            dir.to_string_lossy().into_owned(),
            "pagination".to_owned(),
        ])
        .unwrap();

        let button_path = dir.join("button.rs");
        let pagination_path = dir.join("pagination.rs");
        let mod_path = dir.join("mod.rs");

        assert!(button_path.exists());
        assert!(pagination_path.exists());
        let mod_content = fs::read_to_string(&mod_path).unwrap();
        assert!(mod_content.contains("pub mod button;\n"));
        assert!(mod_content.contains("pub mod pagination;\n"));

        // Modifying button.rs without --overwrite preserves local edit.
        fs::write(&button_path, "// local edit\n").unwrap();
        run(vec![
            "add".to_owned(),
            "--dir".to_owned(),
            dir.to_string_lossy().into_owned(),
            "button".to_owned(),
        ])
        .unwrap();
        assert_eq!(fs::read_to_string(&button_path).unwrap(), "// local edit\n");

        // diff works for modified, same, and missing components.
        assert!(
            run(vec![
                "diff".to_owned(),
                "--dir".to_owned(),
                dir.to_string_lossy().into_owned(),
            ])
            .is_ok()
        );
        assert!(
            run(vec![
                "diff".to_owned(),
                "--dir".to_owned(),
                dir.to_string_lossy().into_owned(),
                "card".to_owned(),
            ])
            .is_ok()
        );

        // --overwrite replaces the local edit.
        run(vec![
            "add".to_owned(),
            "--dir".to_owned(),
            dir.to_string_lossy().into_owned(),
            "--overwrite".to_owned(),
            "button".to_owned(),
        ])
        .unwrap();
        assert_eq!(
            fs::read_to_string(&button_path).unwrap(),
            localize(source("button").unwrap())
        );

        // update_mod handles files without trailing newline and avoids duplicates.
        fs::write(&mod_path, "pub mod button;").unwrap();
        let set: BTreeSet<&str> = ["button", "card"].into_iter().collect();
        update_mod(&dir, &set).unwrap();
        assert_eq!(
            fs::read_to_string(&mod_path).unwrap(),
            "pub mod button;\npub mod card;\n"
        );
        // Calling again is a no-op.
        update_mod(&dir, &set).unwrap();
        assert_eq!(
            fs::read_to_string(&mod_path).unwrap(),
            "pub mod button;\npub mod card;\n"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn diff_lines_counts_substitutions_and_length_differences() {
        assert_eq!(diff_lines("a\nb\n", "a\nb\n"), 0);
        assert_eq!(diff_lines("a\nb\n", "a\nc\n"), 1);
        assert_eq!(diff_lines("a\n", "a\nb\nc\n"), 2);
        assert_eq!(diff_lines("x\nb\nc\n", "a\n"), 3);
    }

    proptest! {
        #[test]
        fn resolve_is_idempotent_and_transitively_closed(
            idx in 0..registry::COMPONENTS.len(),
        ) {
            let name = registry::COMPONENTS[idx].0.to_owned();
            let first = resolve(std::slice::from_ref(&name)).unwrap();
            prop_assert!(first.contains(name.as_str()));

            let names: Vec<String> = first.iter().map(|s| (*s).to_owned()).collect();
            let second = resolve(&names).unwrap();
            prop_assert_eq!(first, second);
        }

        #[test]
        fn diff_lines_is_symmetric_and_zero_iff_equal(
            a in "[a-c\n]{0,30}",
            b in "[a-c\n]{0,30}",
        ) {
            prop_assert_eq!(diff_lines(&a, &b), diff_lines(&b, &a));
            prop_assert_eq!(diff_lines(&a, &a), 0);
        }
    }
}
