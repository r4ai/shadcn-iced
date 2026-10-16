//! Used to display textual user input from keyboard.
//!
//! Port of <https://ui.shadcn.com/docs/components/kbd>.
use iced_ui_core::icon::Icon;
use iced_ui_core::{Element, font, style, tokens};

use iced::widget::{Column, Row, container, space, text};
use iced::{Alignment, Background, Theme};

/// Creates a key (`<kbd>`) with a text label (`font-sans text-xs
/// font-medium`).
pub fn kbd<'a, Message: 'a>(label: impl text::IntoFragment<'a>) -> Element<'a, Message> {
    frame(
        text(label)
            .size(style::text::XS)
            .font(font::MEDIUM)
            .line_height(iced::Pixels(16.0))
            .wrapping(text::Wrapping::None),
    )
}

/// Creates a key displaying an icon (`size-3`).
pub fn kbd_icon<'a, Message: 'a>(icon: Icon) -> Element<'a, Message> {
    frame(icon.sized(12))
}

/// Groups several keys (`inline-flex gap-1`).
pub fn kbd_group<'a, Message: 'a>(
    keys: impl IntoIterator<Item = Element<'a, Message>>,
) -> Element<'a, Message> {
    Row::with_children(keys)
        .spacing(4)
        .align_y(Alignment::Center)
        .into()
}

/// `h-5 w-fit min-w-5 gap-1 px-1 inline-flex items-center justify-center`
fn frame<'a, Message: 'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(
        // A zero-height spacer keeps the key at least 20px wide (`min-w-5`).
        Column::new()
            .push(space().width(12).height(0))
            .push(content)
            .align_x(Alignment::Center),
    )
    .padding([0, 4])
    .center_y(20)
    .style(style)
    .into()
}

/// The style of a key (`bg-muted text-muted-foreground rounded-sm`).
pub fn style(theme: &Theme) -> container::Style {
    let t = tokens(theme);

    container::Style {
        background: Some(Background::Color(t.palette.muted)),
        text_color: Some(t.palette.muted_foreground),
        border: style::rounded(t.radius_sm()),
        ..container::Style::default()
    }
}
