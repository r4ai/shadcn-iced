//! Renders the conformance scenarios with shadcn_iced.
//!
//! ```sh
//! cargo run -p conformance --release -- [FILTER...] [--out DIR]
//! ```
//!
//! Every scenario of `conformance/scenarios/*.json` whose key
//! (`component/scenario`) contains one of the filters is run in each of its
//! themes. Snapshots are written to `DIR/<component>/<scenario>/<snapshot>-<theme>.png`
//! (`conformance/out/iced` by default) and the outcome of every run, including
//! its expectations, to `DIR/runs/<component>--<scenario>--<theme>.json`. The Playwright runner compares them
//! with shadcn/ui.
mod fixtures;
mod fonts;
mod harness;
mod page;
mod spec;
mod store;

use harness::Harness;
use spec::{Locator, Step};

use serde::Serialize;
use shadcn_iced::Mode;
use shadcn_iced::theme::{BaseColor, Config};

use std::path::{Path, PathBuf};

#[derive(Debug, Default, Serialize)]
struct Outcome {
    /// `ok`, `missing` (no fixture) or `error` (a step could not be performed).
    status: &'static str,
    errors: Vec<String>,
    checks: Vec<Check>,
    snapshots: Vec<String>,
}

#[derive(Debug, Serialize)]
struct Check {
    step: usize,
    text: String,
    expected: &'static str,
    ok: bool,
}

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut out = root.join("out/iced");
    let mut filters = Vec::new();

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--out" => out = PathBuf::from(args.next().expect("--out DIR")),
            _ => filters.push(arg),
        }
    }

    // Deterministic snapshots: the shadcn/ui runner disables animations too.
    shadcn_iced::animation::set_enabled(false);

    fonts::load();

    let files = spec::read_all(&root.join("scenarios")).unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(1);
    });

    let mut ran = 0;
    let mut failed = 0;

    for file in &files {
        for scenario in &file.scenarios {
            let key = format!("{}/{}", file.component, scenario.id);

            if !filters.is_empty() && !filters.iter().any(|filter| key.contains(filter.as_str())) {
                continue;
            }

            for theme in &scenario.themes {
                let outcome = run(&out, &file.component, scenario, *theme);
                ran += 1;

                if outcome.status != "ok" || outcome.checks.iter().any(|check| !check.ok) {
                    failed += 1;
                    eprintln!("{key} [{}]: {}", theme.name(), describe(&outcome));
                }

                let path = out.join("runs").join(format!(
                    "{}--{}--{}.json",
                    file.component,
                    scenario.id,
                    theme.name()
                ));
                std::fs::create_dir_all(path.parent().expect("parent")).expect("create directory");
                std::fs::write(
                    &path,
                    serde_json::to_string_pretty(&outcome).expect("serialize"),
                )
                .expect("write outcome");
            }
        }
    }

    println!("{ran} runs, {failed} with problems -> {}", out.display());
}

fn describe(outcome: &Outcome) -> String {
    let mut problems: Vec<String> = outcome.errors.clone();

    for check in outcome.checks.iter().filter(|check| !check.ok) {
        problems.push(format!(
            "step {}: {:?} should be {}",
            check.step, check.text, check.expected
        ));
    }

    if problems.is_empty() {
        outcome.status.to_owned()
    } else {
        format!("{}: {}", outcome.status, problems.join("; "))
    }
}

fn run(out: &Path, component: &str, scenario: &spec::Scenario, theme: spec::Theme) -> Outcome {
    let Some(fixture) = fixtures::get(component, &scenario.id) else {
        return Outcome {
            status: "missing",
            ..Outcome::default()
        };
    };

    let mode = match theme {
        spec::Theme::Light => Mode::Light,
        spec::Theme::Dark => Mode::Dark,
    };

    let [width, height] = scenario.viewport;
    let mut harness = Harness::new(
        fixture,
        Config::new(BaseColor::Neutral, mode).build(),
        iced::Size::new(width, height),
    );

    let mut outcome = Outcome {
        status: "ok",
        ..Outcome::default()
    };

    for (index, step) in scenario.steps().iter().enumerate() {
        if let Err(error) = perform(
            &mut harness,
            out,
            component,
            scenario,
            theme,
            index,
            step,
            &mut outcome,
        ) {
            outcome.status = "error";
            outcome.errors.push(format!("step {index}: {error}"));
            break;
        }
    }

    outcome
}

#[allow(clippy::too_many_arguments)]
fn perform(
    harness: &mut Harness,
    out: &Path,
    component: &str,
    scenario: &spec::Scenario,
    theme: spec::Theme,
    index: usize,
    step: &Step,
    outcome: &mut Outcome,
) -> Result<(), String> {
    match step {
        Step::Snapshot { snapshot } => {
            let (rgba, width, height) = harness.snapshot();
            let path = out
                .join(component)
                .join(&scenario.id)
                .join(format!("{snapshot}-{}.png", theme.name()));
            write_png(&path, &rgba, width, height)?;
            outcome.snapshots.push(snapshot.clone());
        }
        Step::Hover { hover, steps } => {
            let position = locate(harness, hover)?;
            harness.move_in_steps(position, *steps);
        }
        Step::Click { click } => {
            let position = locate(harness, click)?;
            harness.click(position);
        }
        Step::RightClick { right_click } => {
            let position = locate(harness, right_click)?;
            harness.right_click(position);
        }
        Step::Drag { drag } => {
            let from = locate(harness, &drag.from)?;
            let to = locate(harness, &drag.to)?;
            harness.drag(from, to);
        }
        Step::Scroll { scroll } => {
            let position = locate(harness, &scroll.target)?;
            let [dx, dy] = scroll.delta;
            harness.scroll(position, iced::Vector::new(dx, dy));
        }
        Step::Press { press } => harness.press_key(press)?,
        Step::Type { r#type } => harness.type_text(r#type),
        Step::Wait { wait } => harness.wait(*wait),
        Step::Expect { expect } => {
            for text in &expect.visible {
                let ok = !harness.visible_texts(text, true).is_empty();
                outcome.checks.push(Check {
                    step: index,
                    text: text.clone(),
                    expected: "visible",
                    ok,
                });
            }

            for text in &expect.hidden {
                let ok = harness.visible_texts(text, true).is_empty();
                outcome.checks.push(Check {
                    step: index,
                    text: text.clone(),
                    expected: "hidden",
                    ok,
                });
            }
        }
    }

    Ok(())
}

fn locate(harness: &mut Harness, locator: &Locator) -> Result<iced::Point, String> {
    match locator {
        Locator::At { at: [x, y] } => Ok(iced::Point::new(*x, *y)),
        Locator::Text { text, nth, exact } => harness
            .locate_text(text, *nth, *exact)
            .ok_or_else(|| format!("text {text:?} (#{nth}) not found")),
        Locator::Label { label } => harness
            .locate_id(label)
            .ok_or_else(|| format!("control labeled {label:?} not found")),
        Locator::Plain(text) => harness
            .locate_text(text, 0, true)
            .ok_or_else(|| format!("text {text:?} not found")),
    }
}

fn write_png(path: &Path, rgba: &[u8], width: u32, height: u32) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let file = std::fs::File::create(path).map_err(|error| error.to_string())?;
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header().map_err(|error| error.to_string())?;
    writer
        .write_image_data(rgba)
        .map_err(|error| error.to_string())?;

    Ok(())
}
