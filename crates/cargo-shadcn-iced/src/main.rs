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
use std::ffi::OsString;
use std::io::ErrorKind;
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
    --                    Treat everything after it as a component name

Component names are case-insensitive, and `--all` cannot be combined with names.

Copied components depend on `iced` and `shadcn_iced_core`:

    cargo add iced --features advanced,canvas,image,svg
    cargo add shadcn_iced_core

Then declare the module (`mod ui;`) and load the fonts with
`shadcn_iced_core::ApplicationExt::shadcn_iced`.";

fn main() -> ExitCode {
    let result = utf8_args(env::args_os().skip(1)).and_then(|mut args| {
        // When invoked as `cargo shadcn-iced`, cargo passes `shadcn-iced` first.
        if args.first().is_some_and(|arg| arg == "shadcn-iced") {
            args.remove(0);
        }
        run(args)
    });

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

/// The arguments as strings, or an error naming the first one that is not
/// valid UTF-8 (`env::args` would panic instead).
fn utf8_args(args: impl Iterator<Item = OsString>) -> Result<Vec<String>, String> {
    args.map(|arg| {
        arg.into_string()
            .map_err(|arg| format!("argument {arg:?} is not valid UTF-8"))
    })
    .collect()
}

#[derive(Debug)]
struct Options {
    dir: PathBuf,
    overwrite: bool,
    all: bool,
    help: bool,
    names: Vec<String>,
}

/// A `--dir` value: it must be present and must not look like another option.
fn dir_value(value: Option<&str>) -> Result<PathBuf, String> {
    match value {
        Some(path) if !path.is_empty() && !path.starts_with('-') => Ok(PathBuf::from(path)),
        _ => Err("--dir needs a path".to_owned()),
    }
}

/// The name of a component as the registry spells it: `Input-OTP` is `input_otp`.
fn normalize(name: &str) -> String {
    name.replace('-', "_").to_lowercase()
}

fn parse(args: &[String]) -> Result<Options, String> {
    let mut options = Options {
        dir: PathBuf::from(DEFAULT_DIR),
        overwrite: false,
        all: false,
        help: false,
        names: Vec::new(),
    };

    let mut args = args.iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--" => {
                options
                    .names
                    .extend(args.by_ref().map(|name| normalize(name)));
            }
            "--dir" => options.dir = dir_value(args.next().map(String::as_str))?,
            "--overwrite" => options.overwrite = true,
            "--all" => options.all = true,
            "-h" | "--help" => options.help = true,
            flag if flag.starts_with("--dir=") => {
                options.dir = dir_value(flag.strip_prefix("--dir="))?;
            }
            flag if flag.starts_with('-') => return Err(format!("unknown option `{flag}`")),
            name => options.names.push(normalize(name)),
        }
    }

    if options.all && !options.names.is_empty() {
        return Err("`--all` cannot be combined with component names".to_owned());
    }

    Ok(options)
}

/// Runs a command that takes the common options, or prints the help.
fn with_options(
    args: &[String],
    command: impl FnOnce(Options) -> Result<(), String>,
) -> Result<(), String> {
    let options = parse(args)?;

    if options.help {
        println!("{HELP}");
        return Ok(());
    }

    command(options)
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
        "list" => match rest {
            [] => {
                list();
                Ok(())
            }
            [flag] if flag == "-h" || flag == "--help" => {
                println!("{HELP}");
                Ok(())
            }
            [other, ..] => Err(format!("`list` takes no arguments, got `{other}`")),
        },
        "add" => with_options(rest, add),
        "diff" => with_options(rest, diff),
        "view" => with_options(rest, |options| {
            let [name] = options.names.as_slice() else {
                return Err("`view` takes exactly one component".to_owned());
            };
            println!("{}", source(name)?);
            Ok(())
        }),
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

/// Whether `c` can be part of an identifier.
fn is_ident(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// The identifier at the start of `text`, after any whitespace.
fn leading_ident(text: &str) -> &str {
    let text = text.trim_start();
    let end = text.find(|c| !is_ident(c)).unwrap_or(text.len());
    &text[..end]
}

/// The paths directly under `super::` in a use: `super::button::Button` and
/// `super::{button, card::Card}` both give `button` (and `card`).
fn super_references(code: &str) -> Vec<&str> {
    const PREFIX: &str = "super::";
    let mut found = Vec::new();

    for (index, _) in code.match_indices(PREFIX) {
        if code[..index].chars().next_back().is_some_and(is_ident) {
            continue;
        }

        let rest = code[index + PREFIX.len()..].trim_start();

        match rest.strip_prefix('{') {
            Some(group) => found.extend(group_items(group)),
            None => found.push(leading_ident(rest)),
        }
    }

    found
}

/// The first identifier of each top-level item of a use group, given the text
/// after its opening brace: `button, card::Card}` gives `button` and `card`.
fn group_items(group: &str) -> Vec<&str> {
    let mut items = Vec::new();
    let mut depth = 1_usize;
    let mut start = 0;

    for (index, c) in group.char_indices() {
        match c {
            '{' => depth += 1,
            '}' | ',' if depth == 1 => {
                items.extend(
                    Some(leading_ident(&group[start..index])).filter(|item| !item.is_empty()),
                );
                start = index + 1;
                if c == '}' {
                    break;
                }
            }
            '}' => depth -= 1,
            _ => {}
        }
    }

    items
}

/// The sibling components a component uses, through `super::`.
fn dependencies(source: &str) -> BTreeSet<&'static str> {
    let code: String = source
        .lines()
        .map(|line| line.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n");
    let references = super_references(&code);

    registry::COMPONENTS
        .iter()
        .map(|(name, _)| *name)
        .filter(|name| references.contains(name))
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
/// This covers module (`//!`) and item (`///`) documentation, at any indentation.
fn localize(source: &str) -> String {
    source.split_inclusive('\n').map(localize_line).collect()
}

/// `localize` for one line, with its line ending.
fn localize_line(line: &str) -> String {
    let content = line.trim_end();
    let body = content.trim_start();

    let Some(after_marker) = body
        .strip_prefix("//!")
        .or_else(|| body.strip_prefix("///"))
    else {
        return line.to_owned();
    };
    let Some(info) = after_marker.trim_start().strip_prefix("```") else {
        return line.to_owned();
    };

    let fence = &content[..content.len() - info.len()];
    let info = info
        .split(',')
        .map(|token| {
            if token.trim() == "no_run" {
                "ignore"
            } else {
                token
            }
        })
        .collect::<Vec<_>>()
        .join(",");

    format!("{fence}{info}{}", &line[content.len()..])
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
    let mut failure = None;

    for name in &components {
        match copy_component(&options, name) {
            Ok(true) => written.push(*name),
            Ok(false) => skipped.push(*name),
            Err(error) => {
                failure = Some(error);
                break;
            }
        }
    }

    // Declare what is on disk, even after a failure, so no file is left out of `mod.rs`.
    let present: BTreeSet<&str> = written.iter().chain(&skipped).copied().collect();
    let declared = if present.is_empty() {
        Ok(())
    } else {
        update_mod(&options.dir, &present)
    };

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

    match (failure, declared) {
        (Some(error), Err(mod_error)) => return Err(format!("{error}; {mod_error}")),
        (Some(error), Ok(())) | (None, Err(error)) => return Err(error),
        (None, Ok(())) => {}
    }

    if !has_dependency("shadcn_iced_core") {
        println!("\nThe components need `shadcn_iced_core`:\n    cargo add shadcn_iced_core");
    }

    Ok(())
}

/// Copies one component into the directory. Gives `false` when the file exists
/// and `--overwrite` was not given.
fn copy_component(options: &Options, name: &str) -> Result<bool, String> {
    let path = options.dir.join(format!("{name}.rs"));

    if path.exists() && !options.overwrite {
        return Ok(false);
    }

    atomic_write(&path, &localize(source(name)?))?;
    Ok(true)
}

/// Writes a file through a temporary file in the same directory and a rename, so
/// that a failure never leaves a truncated file behind.
fn atomic_write(path: &Path, content: &str) -> Result<(), String> {
    let describe = |error: std::io::Error| format!("{}: {error}", path.display());

    let mut temporary_name = OsString::from(".");
    temporary_name.push(path.file_name().unwrap_or_default());
    temporary_name.push(format!(".{}.tmp", std::process::id()));
    let temporary = path.with_file_name(temporary_name);

    let result = fs::write(&temporary, content)
        .and_then(|()| match fs::metadata(path) {
            Ok(existing) => fs::set_permissions(&temporary, existing.permissions()),
            Err(_) => Ok(()),
        })
        .and_then(|()| fs::rename(&temporary, path));

    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }

    result.map_err(describe)
}

const NEW_MOD: &str = "//! UI components copied from shadcn_iced with `cargo shadcn-iced add`.\n//!\n//! You own this code: edit it as you wish.\n#![allow(dead_code, unused_imports)]\n\n";

/// The module declared at the start of `text` (`pub mod x;`, `pub(crate) mod x;`,
/// `#[cfg(unix)] mod x { .. }`...), with the text after its name.
fn module_declaration(text: &str) -> Option<(&str, &str)> {
    let mut rest = text.trim_start();

    while rest.starts_with("#[") {
        rest = rest[rest.find(']')? + 1..].trim_start();
    }

    if let Some(after) = rest.strip_prefix("pub") {
        rest = after.trim_start();
        if let Some(restricted) = rest.strip_prefix('(') {
            rest = restricted[restricted.find(')')? + 1..].trim_start();
        }
    }

    let rest = rest.strip_prefix("mod")?;
    if !rest.starts_with(char::is_whitespace) {
        return None;
    }

    let name = leading_ident(rest);
    let after = rest.trim_start()[name.len()..].trim_start();
    (!name.is_empty() && (after.starts_with(';') || after.starts_with('{')))
        .then_some((name, after))
}

/// The modules a `mod.rs` already declares, in any form.
fn declared_modules(content: &str) -> BTreeSet<&str> {
    let mut declared = BTreeSet::new();

    for line in content.lines() {
        let mut rest = line;
        while let Some((name, after)) = module_declaration(rest) {
            declared.insert(name);
            rest = after.strip_prefix(';').unwrap_or("");
        }
    }

    declared
}

/// Adds `pub mod name;` to the `mod.rs` of the directory, creating it if
/// needed. Modules it already declares are left alone.
fn update_mod(dir: &Path, components: &BTreeSet<&str>) -> Result<(), String> {
    let path = dir.join("mod.rs");
    let (existing, exists) = match fs::read_to_string(&path) {
        Ok(content) => (content, true),
        Err(error) if error.kind() == ErrorKind::NotFound => (NEW_MOD.to_owned(), false),
        Err(error) => return Err(format!("{}: {error}", path.display())),
    };

    let declared = declared_modules(&existing);
    let missing: Vec<_> = components
        .iter()
        .filter(|name| !declared.contains(**name))
        .collect();

    if missing.is_empty() && exists {
        return Ok(());
    }

    let mut content = existing.clone();

    if !content.ends_with('\n') {
        content.push('\n');
    }

    for name in missing {
        content.push_str(&format!("pub mod {name};\n"));
    }

    atomic_write(&path, &content)
}

/// Whether `manifest` (the text of a `Cargo.toml`) lists the crate as a
/// dependency: a key of a dependencies table, or a `[dependencies.name]` table.
fn manifest_has_dependency(manifest: &str, name: &str) -> bool {
    let name = name.replace('-', "_");
    let same = |key: &str| key.trim().trim_matches(['"', '\'']).replace('-', "_") == name;
    let mut in_dependencies = false;

    for line in manifest.lines().map(str::trim) {
        if let Some(header) = line.strip_prefix('[') {
            if header.starts_with('[') {
                in_dependencies = false;
                continue;
            }
            let table = header
                .rsplit_once(']')
                .map_or(header, |(table, _)| table)
                .trim();
            if let Some((parent, last)) = table.rsplit_once('.')
                && parent.trim_end().ends_with("dependencies")
                && same(last)
            {
                return true;
            }
            in_dependencies = table.ends_with("dependencies");
        } else if in_dependencies
            && let Some((key, _)) = line.split_once('=')
            && same(key.split('.').next().unwrap_or(key))
        {
            return true;
        }
    }

    false
}

fn has_dependency(name: &str) -> bool {
    fs::read_to_string("Cargo.toml").is_ok_and(|manifest| manifest_has_dependency(&manifest, name))
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
    let names: Vec<String> = if options.names.is_empty() {
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
            "Input-OTP",
            "date-picker",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect();
        let opts = parse(&args).unwrap();
        assert_eq!(opts.dir, PathBuf::from("custom/ui"));
        assert!(opts.overwrite);
        assert!(!opts.all);
        assert_eq!(opts.names, vec!["input_otp", "date_picker"]);

        assert!(parse(&["--dir".to_owned()]).is_err());
        assert!(parse(&["--bogus".to_owned()]).is_err());
        assert!(parse(&["-z".to_owned()]).is_err());
    }

    fn strings(args: &[&str]) -> Vec<String> {
        args.iter().map(|arg| (*arg).to_owned()).collect()
    }

    #[test]
    fn parse_dir_value_never_swallows_an_option() {
        assert!(parse(&strings(&["--dir", "--overwrite", "button"])).is_err());
        assert!(parse(&strings(&["--dir", "-x"])).is_err());
        assert!(parse(&strings(&["--dir", ""])).is_err());
        assert!(parse(&strings(&["--dir="])).is_err());
        assert!(parse(&strings(&["--dir=-x"])).is_err());

        let options = parse(&strings(&["--dir=custom/ui", "button"])).unwrap();
        assert_eq!(options.dir, PathBuf::from("custom/ui"));
        assert_eq!(options.names, vec!["button"]);
        assert!(!options.overwrite);

        let options = parse(&strings(&["--dir", "a", "--dir=b"])).unwrap();
        assert_eq!(options.dir, PathBuf::from("b"));
    }

    #[test]
    fn parse_supports_help_double_dash_case_and_all_conflict() {
        assert!(parse(&strings(&["-h"])).unwrap().help);
        assert!(parse(&strings(&["--help"])).unwrap().help);
        assert!(!parse(&strings(&[])).unwrap().help);

        let options = parse(&strings(&["--", "--overwrite", "Card"])).unwrap();
        assert!(!options.overwrite);
        assert_eq!(options.names, vec!["__overwrite", "card"]);
        assert!(!parse(&strings(&["--", "--all"])).unwrap().all);

        assert_eq!(parse(&strings(&["Button"])).unwrap().names, vec!["button"]);
        assert_eq!(
            parse(&strings(&["Date-Picker"])).unwrap().names,
            vec!["date_picker"]
        );

        let error = parse(&strings(&["--all", "button"])).unwrap_err();
        assert!(error.contains("--all"), "{error}");
        assert!(parse(&strings(&["button", "--all"])).is_err());
        assert!(parse(&strings(&["--all"])).unwrap().all);
    }

    #[test]
    fn run_prints_help_for_subcommands_and_rejects_extra_list_arguments() {
        for command in ["add", "diff", "view", "list"] {
            assert!(run(strings(&[command, "-h"])).is_ok(), "{command} -h");
            assert!(
                run(strings(&[command, "--help"])).is_ok(),
                "{command} --help"
            );
        }

        assert!(run(strings(&["list", "--bogus"])).is_err());
        assert!(run(strings(&["list", "extra"])).is_err());
        assert!(run(strings(&["list", "--help", "extra"])).is_err());
        assert!(run(strings(&["view", "button", "card"])).is_err());
        assert!(run(strings(&["add", "--all", "button"])).is_err());
        assert!(run(strings(&["diff", "--all", "button"])).is_err());
        assert!(run(strings(&["view", "Button"])).is_ok());
    }

    #[test]
    fn utf8_args_reports_invalid_arguments_instead_of_panicking() {
        let ok = utf8_args(["add", "button"].into_iter().map(OsString::from)).unwrap();
        assert_eq!(ok, vec!["add", "button"]);

        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStringExt;
            let bad = OsString::from_vec(vec![b'a', 0xff]);
            let error = utf8_args([OsString::from("add"), bad].into_iter()).unwrap_err();
            assert!(error.contains("UTF-8"), "{error}");
        }
    }

    #[test]
    fn localize_handles_item_docs_indentation_and_attributes() {
        assert_eq!(
            localize("/// ```no_run\n/// code()\n/// ```\nfn f() {}\n"),
            "/// ```ignore\n/// code()\n/// ```\nfn f() {}\n"
        );
        assert_eq!(
            localize("    /// ```no_run\r\n    //! ```no_run\n"),
            "    /// ```ignore\r\n    //! ```ignore\n"
        );
        assert_eq!(localize("///```no_run"), "///```ignore");
        assert_eq!(localize("/// ```rust,no_run"), "/// ```rust,ignore");
        assert_eq!(
            localize("/// ```no_run,edition2021"),
            "/// ```ignore,edition2021"
        );

        // Everything else stays as it is.
        for untouched in [
            "/// ```text",
            "/// ```",
            "//! ```",
            "// ```no_run",
            "//// ```no_run",
            "let s = \"/// ```no_run\";",
            "/// not a fence ```no_run",
            "/// ```no_run_not",
            "",
        ] {
            assert_eq!(localize(untouched), untouched);
        }
    }

    #[test]
    fn registry_doctests_are_all_localized() {
        for (name, source) in registry::COMPONENTS {
            for line in localize(source).lines() {
                let body = line.trim_start();
                assert!(
                    !(body.starts_with("//! ```no_run") || body.starts_with("/// ```no_run")),
                    "{name}: {line}"
                );
            }
        }
        let typography = localize(source("typography").unwrap());
        assert!(!typography.contains("/// ```no_run"));
    }

    #[test]
    fn super_references_finds_single_and_grouped_imports() {
        assert_eq!(super_references("use super::button::Button;"), ["button"]);
        assert_eq!(
            super_references("use super::{button, card::Card, input as i};"),
            ["button", "card", "input"]
        );
        assert_eq!(
            super_references("use super::{\n    button,\n    menu::{Menu, Item},\n    self,\n};"),
            ["button", "menu", "self"]
        );
        assert_eq!(super_references("super:: button::B"), ["button"]);
        assert_eq!(super_references("supersuper::button"), Vec::<&str>::new());
        assert_eq!(super_references("my_super::button"), Vec::<&str>::new());
        assert_eq!(super_references("super::{}"), Vec::<&str>::new());
    }

    #[test]
    fn dependencies_see_grouped_imports_and_ignore_comments() {
        let deps = dependencies("use super::{button, card};\n");
        assert!(deps.contains("button") && deps.contains("card"), "{deps:?}");

        let deps = dependencies("use super::{\n    button, // not input\n    menu::Menu,\n};\n");
        assert!(deps.contains("button") && deps.contains("menu"), "{deps:?}");
        assert!(!deps.contains("input"), "{deps:?}");

        let deps = dependencies("let x = 1; // use super::card;\n    // super::{button}\n");
        assert!(deps.is_empty(), "{deps:?}");

        let deps = dependencies("use super::{button_group, card2};\n");
        assert!(deps.contains("button_group"), "{deps:?}");
        assert!(
            !deps.contains("button") && !deps.contains("card"),
            "{deps:?}"
        );
    }

    #[test]
    fn update_mod_recognizes_every_declaration_form() {
        let dir = temp_dir("mod-forms");
        let set: BTreeSet<&str> = ["button"].into_iter().collect();

        for existing in [
            "pub mod button;\n",
            "pub mod button; // note\n",
            "pub(crate) mod button;\n",
            "pub(super) mod button;\n",
            "pub(in crate::ui) mod button;\n",
            "mod button;\n",
            "  mod   button ;\n",
            "#[cfg(unix)] pub mod button;\n",
            "#[cfg(unix)]\npub mod button;\n",
            "pub mod button {}\n",
            "pub mod card; pub mod button;\n",
        ] {
            fs::write(dir.join("mod.rs"), existing).unwrap();
            update_mod(&dir, &set).unwrap();
            assert_eq!(fs::read_to_string(dir.join("mod.rs")).unwrap(), existing);
        }

        // Look-alikes do not count as declarations.
        for existing in [
            "// pub mod button;\n",
            "pub mod button_group;\n",
            "pub mod buttons;\n",
            "pub use button::Button;\n",
            "mod button_ext {}\n",
            "pub modbutton;\n",
        ] {
            fs::write(dir.join("mod.rs"), existing).unwrap();
            update_mod(&dir, &set).unwrap();
            assert_eq!(
                fs::read_to_string(dir.join("mod.rs")).unwrap(),
                format!("{existing}pub mod button;\n")
            );
        }

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn module_declaration_needs_a_name_and_a_terminator() {
        assert_eq!(module_declaration("mod a;"), Some(("a", ";")));
        assert_eq!(module_declaration("pub mod a { }"), Some(("a", "{ }")));
        assert_eq!(
            module_declaration("#[x] pub(crate) mod a; mod b;"),
            Some(("a", "; mod b;"))
        );
        assert_eq!(module_declaration("mod ;"), None);
        assert_eq!(module_declaration("mod {"), None);
        assert_eq!(module_declaration("mod a"), None);
        assert_eq!(module_declaration("mod a b;"), None);
        assert_eq!(module_declaration("pub mod"), None);
        assert_eq!(module_declaration("#[unclosed pub mod a;"), None);
        assert_eq!(module_declaration("pub(crate mod a;"), None);
        assert_eq!(
            declared_modules("mod a; pub mod c; mod b {}"),
            ["a", "b", "c"].into()
        );
    }

    #[test]
    fn update_mod_does_not_replace_a_mod_rs_it_cannot_read() {
        let dir = temp_dir("mod-unreadable");
        let path = dir.join("mod.rs");
        let set: BTreeSet<&str> = ["button"].into_iter().collect();

        // Invalid UTF-8 is an error, and the file is left alone.
        fs::write(&path, [0xff, 0xfe, b'\n']).unwrap();
        assert!(update_mod(&dir, &set).is_err());
        assert_eq!(fs::read(&path).unwrap(), [0xff, 0xfe, b'\n']);

        // A directory in place of the file is an error too.
        fs::remove_file(&path).unwrap();
        fs::create_dir(&path).unwrap();
        assert!(update_mod(&dir, &set).is_err());
        assert!(path.is_dir());

        // No file at all is fine: it is created with the header.
        fs::remove_dir(&path).unwrap();
        update_mod(&dir, &set).unwrap();
        let created = fs::read_to_string(&path).unwrap();
        assert!(created.starts_with("//! UI components"), "{created}");
        assert!(created.ends_with("pub mod button;\n"), "{created}");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn atomic_write_replaces_without_leftovers_and_keeps_permissions() {
        let dir = temp_dir("atomic");
        let path = dir.join("file.rs");

        atomic_write(&path, "one").unwrap();
        atomic_write(&path, "two").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "two");
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);

        // A failure (the target is a non-empty directory) leaves the target and no temp file.
        let blocked = dir.join("blocked.rs");
        fs::create_dir_all(blocked.join("inner")).unwrap();
        assert!(atomic_write(&blocked, "x").is_err());
        assert!(blocked.join("inner").is_dir());
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 2);

        assert!(atomic_write(&dir.join("missing/dir/file.rs"), "x").is_err());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn add_declares_written_files_even_when_a_later_one_fails() {
        let dir = temp_dir("partial");

        // Make `card.rs` unwritable: a non-empty directory with that name, so
        // `--overwrite` cannot replace it. Components come in alphabetical order.
        fs::create_dir_all(dir.join("card.rs/inner")).unwrap();
        let options = Options {
            dir: dir.clone(),
            overwrite: true,
            all: true,
            help: false,
            names: Vec::new(),
        };
        let error = add(options).unwrap_err();
        assert!(error.contains("card.rs"), "{error}");

        let declared = fs::read_to_string(dir.join("mod.rs")).unwrap();
        let declared = declared_modules(&declared);
        assert!(declared.contains("accordion"), "{declared:?}");
        assert!(!declared.contains("card"), "{declared:?}");
        for name in &declared {
            assert!(dir.join(format!("{name}.rs")).is_file(), "{name}");
        }
        for entry in fs::read_dir(&dir).unwrap() {
            let name = entry.unwrap().file_name();
            assert!(!name.to_string_lossy().ends_with(".tmp"), "{name:?}");
        }

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn add_accepts_names_case_insensitively() {
        let dir = temp_dir("case");
        run(strings(&["add", "--dir", dir.to_str().unwrap(), "Button"])).unwrap();
        assert!(dir.join("button.rs").is_file());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn manifest_dependency_matches_keys_not_substrings() {
        let name = "shadcn_iced_core";

        for manifest in [
            "[dependencies]\nshadcn_iced_core = \"0.3\"\n",
            "[dependencies]\nshadcn_iced_core.workspace = true\n",
            "[dependencies]\n  shadcn_iced_core={ version = \"1\" }\n",
            "[dependencies]\nshadcn-iced-core = \"1\"\n",
            "[dependencies]\n\"shadcn_iced_core\" = \"1\"\n",
            "[dev-dependencies]\nshadcn_iced_core = \"1\"\n",
            "[build-dependencies]\nshadcn_iced_core = \"1\"\n",
            "[workspace.dependencies]\nshadcn_iced_core = \"1\"\n",
            "[target.'cfg(unix)'.dependencies]\nshadcn_iced_core = \"1\"\n",
            "[dependencies.shadcn_iced_core]\nversion = \"1\"\n",
            "[dependencies.shadcn_iced_core] # comment\nversion = \"1\"\n",
            "[package]\nname = \"x\"\n\n[dependencies]\niced = \"0.13\"\nshadcn_iced_core = \"1\"\n",
        ] {
            assert!(manifest_has_dependency(manifest, name), "{manifest}");
        }

        for manifest in [
            "",
            "[package]\nname = \"shadcn_iced_core\"\n",
            "[package]\nname = \"x\"\ndescription = \"uses shadcn_iced_core\"\n",
            "[dependencies]\nshadcn_iced_core_extra = \"1\"\n",
            "[dependencies]\nmy_shadcn_iced_core = \"1\"\n",
            "[dependencies]\niced = { package = \"shadcn_iced_core\" }\n",
            "[dependencies]\n# shadcn_iced_core = \"1\"\n",
            "[features]\nshadcn_iced_core = []\n",
            "[dependencies]\n[features]\nshadcn_iced_core = []\n",
            "[[bin]]\nshadcn_iced_core = 1\n",
            "[dependencies.other]\nshadcn_iced_core = \"1\"\n",
            "[dependencies.shadcn_iced_core_extra]\nversion = \"1\"\n",
        ] {
            assert!(!manifest_has_dependency(manifest, name), "{manifest}");
        }
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

    /// One line of an existing `mod.rs`: a declaration of `name` in some form, or noise.
    fn mod_line(name: &'static str) -> impl Strategy<Value = String> {
        prop_oneof![
            Just(format!("pub mod {name};")),
            Just(format!("pub mod {name}; // note")),
            Just(format!("pub(crate) mod {name};")),
            Just(format!("mod {name};")),
            Just(format!("#[cfg(unix)] pub mod {name};")),
            Just(format!("  pub  mod  {name}  ;")),
            Just(format!("pub mod {name} {{}}")),
        ]
    }

    fn noise_line() -> impl Strategy<Value = String> {
        prop_oneof![
            Just(String::new()),
            Just("// pub mod button;".to_owned()),
            Just("pub use button::Button;".to_owned()),
            Just("pub mod button_group;".to_owned()),
            Just("#![allow(dead_code)]".to_owned()),
            Just("//! docs".to_owned()),
            "[ -~]{0,20}",
        ]
    }

    /// Counts the lines that declare `name` the plain way or with a visibility.
    fn declarations_of(content: &str, name: &str) -> usize {
        declared_modules(content)
            .into_iter()
            .filter(|declared| *declared == name)
            .count()
    }

    proptest! {
        #[test]
        fn update_mod_is_idempotent_and_never_declares_twice(
            button in proptest::option::of(mod_line("button")),
            card in proptest::option::of(mod_line("card")),
            noise in proptest::collection::vec(noise_line(), 0..6),
            trailing_newline in any::<bool>(),
            position in 0usize..8,
        ) {
            let dir = temp_dir("prop-mod");
            let mut lines = noise;
            let at = position.min(lines.len());
            lines.splice(at..at, button.into_iter().chain(card));
            let mut existing = lines.join("\n");
            if trailing_newline {
                existing.push('\n');
            }
            fs::write(dir.join("mod.rs"), &existing).unwrap();

            let set: BTreeSet<&str> = ["button", "card"].into_iter().collect();
            update_mod(&dir, &set).unwrap();
            let once = fs::read_to_string(dir.join("mod.rs")).unwrap();
            update_mod(&dir, &set).unwrap();
            let twice = fs::read_to_string(dir.join("mod.rs")).unwrap();
            let _ = fs::remove_dir_all(&dir);

            prop_assert_eq!(&once, &twice);
            prop_assert!(once.starts_with(&existing));
            for name in ["button", "card"] {
                prop_assert_eq!(declarations_of(&once, name), 1, "{}", once);
            }
        }

        #[test]
        fn parse_never_panics_and_keeps_names_normalized(
            args in proptest::collection::vec(
                prop_oneof![
                    Just("--dir".to_owned()),
                    Just("--all".to_owned()),
                    Just("--overwrite".to_owned()),
                    Just("--".to_owned()),
                    Just("-h".to_owned()),
                    "[ -~]{0,8}",
                    "\\PC{0,6}",
                ],
                0..8,
            ),
        ) {
            if let Ok(options) = parse(&args) {
                prop_assert!(!(options.all && !options.names.is_empty()));
                for name in &options.names {
                    prop_assert!(!name.contains('-') || args.iter().any(|a| a == "--"));
                    prop_assert_eq!(name.to_lowercase(), name.clone());
                }
            }
        }

        #[test]
        fn localize_and_dependencies_never_panic_and_keep_line_count(
            source in "[ -~\n\t]{0,120}|(//[/!] ?```(no_run)?\n){0,4}",
        ) {
            let localized = localize(&source);
            prop_assert_eq!(localized.lines().count(), source.lines().count());
            prop_assert_eq!(localize(&localized), localized.clone());
            let _ = dependencies(&source);
            let _ = declared_modules(&source);
            let _ = manifest_has_dependency(&source, "shadcn_iced_core");
        }

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
