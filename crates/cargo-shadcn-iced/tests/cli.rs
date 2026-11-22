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
