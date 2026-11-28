use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_cargo-shadcn-iced"))
}

fn temp_dir(label: &str) -> PathBuf {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "cargo-shadcn-iced-cli-{}-{id}-{label}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn cli_subcommand_prefix_list_and_errors() {
    // Invoked as `cargo shadcn-iced list` passes `shadcn-iced` as argv[1].
    let out = bin().args(["shadcn-iced", "list"]).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("button"));
    assert!(stdout.contains("pagination"));

    let fail = bin().args(["unknown-cmd"]).output().unwrap();
    assert!(!fail.status.success());
}

#[test]
fn cli_add_dependency_hint_and_diff_output() {
    let dir = temp_dir("diff");
    let ui = dir.join("src/ui");

    // Without Cargo.toml mentioning shadcn_iced_core, add prints the dependency hint.
    let add_out = bin()
        .current_dir(&dir)
        .args(["add", "--dir", ui.to_str().unwrap(), "button"])
        .output()
        .unwrap();
    assert!(add_out.status.success());
    assert!(
        String::from_utf8_lossy(&add_out.stdout).contains("cargo add shadcn_iced_core"),
        "expected dependency hint"
    );

    // With Cargo.toml mentioning shadcn_iced_core, the hint is omitted.
    fs::write(
        dir.join("Cargo.toml"),
        "[dependencies]\nshadcn_iced_core = \"0.3\"\n",
    )
    .unwrap();
    let add_again = bin()
        .current_dir(&dir)
        .args(["add", "--dir", ui.to_str().unwrap(), "button"])
        .output()
        .unwrap();
    assert!(add_again.status.success());
    assert!(
        !String::from_utf8_lossy(&add_again.stdout).contains("cargo add shadcn_iced_core"),
        "unexpected dependency hint"
    );

    // diff reports `same` when unmodified.
    let diff_same = bin()
        .current_dir(&dir)
        .args(["diff", "--dir", ui.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(diff_same.status.success());
    assert!(String::from_utf8_lossy(&diff_same.stdout).contains("same     button"));

    // diff reports `modified` when edited, and `missing` for an absent component.
    fs::write(ui.join("button.rs"), "// modified\n").unwrap();
    let diff_mod = bin()
        .current_dir(&dir)
        .args(["diff", "--dir", ui.to_str().unwrap(), "button", "card"])
        .output()
        .unwrap();
    assert!(diff_mod.status.success());
    let stdout = String::from_utf8_lossy(&diff_mod.stdout);
    assert!(stdout.contains("modified button"));
    assert!(stdout.contains("missing  card"));

    let _ = fs::remove_dir_all(&dir);
}

#[cfg(unix)]
#[test]
fn cli_reports_non_utf8_arguments_instead_of_panicking() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let bad = OsString::from_vec(vec![b'b', 0xff, 0xfe]);
    let out = bin().arg("add").arg(bad).output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("not valid UTF-8"), "{stderr}");
    assert!(!stderr.contains("panicked"), "{stderr}");
}

#[test]
fn cli_prints_help_for_subcommand_flags_and_rejects_bad_arguments() {
    for args in [
        ["add", "--help"],
        ["diff", "-h"],
        ["list", "-h"],
        ["view", "--help"],
    ] {
        let out = bin().args(args).output().unwrap();
        assert!(out.status.success(), "{args:?}");
        assert!(
            String::from_utf8_lossy(&out.stdout).contains("USAGE:"),
            "{args:?}"
        );
    }

    for args in [
        &["list", "--bogus"][..],
        &["list", "extra"],
        &["add", "--all", "button"],
        &["add", "--dir", "--overwrite", "button"],
    ] {
        let out = bin().args(args).output().unwrap();
        assert_eq!(out.status.code(), Some(1), "{args:?}");
        assert!(
            String::from_utf8_lossy(&out.stderr).starts_with("error:"),
            "{args:?}"
        );
    }
}

#[test]
fn cli_add_handles_dir_equals_double_dash_and_case() {
    let dir = temp_dir("forms");
    let ui = dir.join("ui");

    let out = bin()
        .current_dir(&dir)
        .arg("add")
        .arg(format!("--dir={}", ui.display()))
        .args(["--", "Button"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(ui.join("button.rs").is_file());
    assert!(
        fs::read_to_string(ui.join("mod.rs"))
            .unwrap()
            .contains("pub mod button;")
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn cli_add_does_not_duplicate_a_commented_declaration() {
    let dir = temp_dir("mod-forms");
    let ui = dir.join("ui");
    fs::create_dir_all(&ui).unwrap();
    fs::write(ui.join("mod.rs"), "pub(crate) mod button; // mine\n").unwrap();

    let out = bin()
        .current_dir(&dir)
        .args(["add", "--dir", ui.to_str().unwrap(), "button"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(
        fs::read_to_string(ui.join("mod.rs")).unwrap(),
        "pub(crate) mod button; // mine\n"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn cli_dependency_hint_ignores_mentions_that_are_not_dependencies() {
    let dir = temp_dir("hint");
    fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"shadcn_iced_core_demo\"\n\n[dependencies]\n# shadcn_iced_core = \"1\"\n",
    )
    .unwrap();

    let out = bin()
        .current_dir(&dir)
        .args(["add", "--dir", dir.join("ui").to_str().unwrap(), "button"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("cargo add shadcn_iced_core"));

    let _ = fs::remove_dir_all(&dir);
}
