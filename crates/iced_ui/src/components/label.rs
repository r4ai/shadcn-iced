//! Renders an accessible label associated with controls.
//!
//! Port of <https://ui.shadcn.com/docs/components/label>.
use iced_ui_core::{font, style};

use iced::widget::{Text, text};

/// Creates a label (`text-sm font-medium leading-none`).
pub fn label<'a>(content: impl text::IntoFragment<'a>) -> Text<'a> {
    text(content)
        .size(style::text::SM)
        .font(font::MEDIUM)
        .line_height(iced::Pixels(14.0))
}
