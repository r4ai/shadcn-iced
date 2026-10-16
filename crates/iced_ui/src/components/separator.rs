//! Visually or semantically separates content.
//!
//! Port of <https://ui.shadcn.com/docs/components/separator>.
use iced_ui_core::tokens;

use iced::widget::{Rule, rule};
use iced::Theme;

/// A horizontal separator (`h-px w-full bg-border`).
pub fn separator<'a>() -> Rule<'a> {
    rule::horizontal(1).style(style)
}

/// A vertical separator (`w-px h-full bg-border`).
pub fn vertical_separator<'a>() -> Rule<'a> {
    rule::vertical(1).style(style)
}

/// The style of a separator.
pub fn style(theme: &Theme) -> rule::Style {
    rule::Style {
        color: tokens(theme).palette.border,
        radius: 0.0.into(),
        fill_mode: rule::FillMode::Full,
        snap: true,
    }
}
