//! Displays an indicator showing the completion progress of a task.
//!
//! Port of <https://ui.shadcn.com/docs/components/progress>.
use iced_ui_core::theme::{alpha, tokens};

use iced::widget::{ProgressBar, progress_bar};
use iced::{Background, Length, Theme, border};

/// Creates a progress bar (`h-2 rounded-full bg-primary/20`) for a value in
/// `0.0..=100.0`.
pub fn progress<'a>(value: f32) -> ProgressBar<'a> {
    progress_bar(0.0..=100.0, value)
        .length(Length::Fill)
        .girth(8)
        .style(style)
}

/// The style of a progress bar.
pub fn style(theme: &Theme) -> progress_bar::Style {
    let p = tokens(theme).palette;

    progress_bar::Style {
        background: Background::Color(alpha(p.primary, 0.2)),
        bar: Background::Color(p.primary),
        border: border::rounded(f32::MAX),
    }
}
