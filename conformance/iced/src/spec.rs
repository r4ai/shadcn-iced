//! The scenario files shared with the shadcn/ui runner (`conformance/scenarios`).
//!
//! The format is documented in `conformance/README.md` and typed for the
//! Playwright runner in `conformance/tests/spec.ts`; both must stay in sync.
use serde::Deserialize;

use std::path::Path;

/// A scenario file: every scenario of a component.
#[derive(Debug, Clone, Deserialize)]
pub struct File {
    pub component: String,
    pub scenarios: Vec<Scenario>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Scenario {
    pub id: String,
    /// The logical size of the page, `[width, height]`.
    #[serde(default = "default_viewport")]
    pub viewport: [f32; 2],
    #[serde(default = "default_themes")]
    pub themes: Vec<Theme>,
    #[serde(default)]
    pub steps: Vec<Step>,
}

impl Scenario {
    /// The steps to perform; a scenario without steps takes a single snapshot.
    pub fn steps(&self) -> Vec<Step> {
        if self.steps.is_empty() {
            vec![Step::Snapshot {
                snapshot: "default".to_owned(),
            }]
        } else {
            self.steps.clone()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    pub fn name(self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Step {
    Snapshot {
        snapshot: String,
    },
    Hover {
        hover: Locator,
        /// The number of intermediate pointer moves (1 by default: a jump).
        #[serde(default = "one")]
        steps: u32,
    },
    Click {
        click: Locator,
    },
    RightClick {
        #[serde(rename = "rightClick")]
        right_click: Locator,
    },
    Drag {
        drag: Drag,
    },
    Scroll {
        scroll: Scroll,
    },
    Press {
        press: String,
    },
    Type {
        r#type: String,
    },
    Wait {
        wait: u64,
    },
    Expect {
        expect: Expectation,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct Drag {
    pub from: Locator,
    pub to: Locator,
}

/// A wheel scroll: `{ "at": [x, y], "delta": [dx, dy] }`, or any other
/// object locator instead of `at` (`{ "text": "...", "delta": ... }`).
#[derive(Debug, Clone, Deserialize)]
pub struct Scroll {
    #[serde(flatten)]
    pub target: Locator,
    /// The logical pixels to scroll by; a positive `dy` scrolls down.
    pub delta: [f32; 2],
}

/// Where a pointer action happens.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Locator {
    /// The center of the `nth` (0-based) visible text with this content,
    /// or containing it when not `exact`.
    Text {
        text: String,
        #[serde(default)]
        nth: usize,
        #[serde(default = "yes")]
        exact: bool,
    },
    /// A logical position in the page.
    At { at: [f32; 2] },
    /// The center of the control with this accessible name: its
    /// `aria-label` on the web, its widget id in iced (for controls without
    /// text, such as icon buttons).
    Label { label: String },
    /// Shorthand for `{ "text": "..." }`.
    Plain(String),
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Expectation {
    /// Texts that must be visible.
    #[serde(default)]
    pub visible: Vec<String>,
    /// Texts that must not be visible.
    #[serde(default)]
    pub hidden: Vec<String>,
}

fn one() -> u32 {
    1
}

fn yes() -> bool {
    true
}

fn default_viewport() -> [f32; 2] {
    [320.0, 120.0]
}

fn default_themes() -> Vec<Theme> {
    vec![Theme::Light, Theme::Dark]
}

/// Reads every scenario file of a directory, sorted by component.
pub fn read_all(directory: &Path) -> Result<Vec<File>, String> {
    let mut files = Vec::new();

    let entries = std::fs::read_dir(directory).map_err(|error| error.to_string())?;

    for entry in entries {
        let path = entry.map_err(|error| error.to_string())?.path();

        if path
            .extension()
            .is_some_and(|extension| extension == "json")
            && path.file_name().is_some_and(|name| name != "schema.json")
        {
            let content = std::fs::read_to_string(&path).map_err(|error| error.to_string())?;
            let file: File = serde_json::from_str(&content)
                .map_err(|error| format!("{}: {error}", path.display()))?;
            files.push(file);
        }
    }

    files.sort_by(|a, b| a.component.cmp(&b.component));
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_scroll_steps() {
        let step: Step =
            serde_json::from_str(r#"{ "scroll": { "at": [10, 20], "delta": [0, 120] } }"#).unwrap();
        let Step::Scroll { scroll } = step else {
            panic!("not a scroll: {step:?}");
        };
        assert!(matches!(scroll.target, Locator::At { at: [10.0, 20.0] }));
        assert_eq!(scroll.delta, [0.0, 120.0]);

        let step: Step =
            serde_json::from_str(r#"{ "scroll": { "text": "Item", "delta": [0, -40] } }"#).unwrap();
        assert!(matches!(
            step,
            Step::Scroll {
                scroll: Scroll {
                    target: Locator::Text { .. },
                    ..
                }
            }
        ));
    }

    #[test]
    fn parses_label_locators() {
        let step: Step = serde_json::from_str(r#"{ "click": { "label": "Close" } }"#).unwrap();
        assert!(matches!(
            step,
            Step::Click { click: Locator::Label { label } } if label == "Close"
        ));
    }
}
