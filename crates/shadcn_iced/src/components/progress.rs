//! Displays an indicator showing the completion progress of a task.
//!
//! Port of <https://ui.shadcn.com/docs/components/progress>.
use shadcn_iced_core::{Element, font, style, tokens};

use iced::widget::{Column, ProgressBar, Row, progress_bar, space, text};
use iced::{Background, Length, Theme, border};

/// Creates a progress bar (`h-1 rounded-full bg-muted`) for a value in
/// `0.0..=100.0`.
pub fn progress<'a>(value: f32) -> ProgressBar<'a> {
    progress_bar(0.0..=100.0, value)
        .length(Length::Fill)
        .girth(4)
        .style(style)
}

/// Creates a progress bar with a label on the left and the value, as a
/// percentage, on the right (`ProgressLabel` and `ProgressValue`, in a
/// `flex flex-wrap gap-3` root).
pub fn progress_with_label<'a, Message: 'a>(
    label: impl text::IntoFragment<'a>,
    value: f32,
) -> Element<'a, Message> {
    let value = value.clamp(0.0, 100.0);

    Column::new()
        .push(
            Row::new()
                // `text-sm font-medium`
                .push(text(label).size(style::text::SM).font(font::medium()))
                .push(space::horizontal())
                // `text-muted-foreground ml-auto text-sm tabular-nums`
                .push(
                    text(format!("{}%", value.round()))
                        .size(style::text::SM)
                        .style(|theme: &Theme| text::Style {
                            color: Some(tokens(theme).palette.muted_foreground),
                        }),
                ),
        )
        .push(progress(value))
        .spacing(12)
        .width(Length::Fill)
        .into()
}

/// The style of a progress bar (`bg-muted`, `bg-primary` indicator).
pub fn style(theme: &Theme) -> progress_bar::Style {
    let p = tokens(theme).palette;

    progress_bar::Style {
        background: Background::Color(p.muted),
        bar: Background::Color(p.primary),
        border: border::rounded(f32::MAX),
    }
}
