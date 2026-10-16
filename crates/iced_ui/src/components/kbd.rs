//! Used to display textual user input from keyboard.
//!
//! Port of <https://ui.shadcn.com/docs/components/kbd>.
use iced_ui_core::icon::Icon;
use iced_ui_core::{Element, font, style, tokens};

use iced::widget::{Row, container, text};
use iced::{Alignment, Background, Theme};

/// Creates a key (`<kbd>`) with a text label.
pub fn kbd<'a, Message: 'a>(label: impl text::IntoFragment<'a>) -> Element<'a, Message> {
    frame(text(label).size(style::text::XS).font(font::MEDIUM))
}

/// Creates a key displaying an icon.
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

fn frame<'a, Message: 'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content)
        .padding([0, 4])
        .height(20)
        .center_y(20)
        .style(style)
        .into()
}

/// The style of a key.
pub fn style(theme: &Theme) -> container::Style {
    let t = tokens(theme);

    container::Style {
        background: Some(Background::Color(t.palette.muted)),
        text_color: Some(t.palette.muted_foreground),
        border: style::rounded(t.radius_sm()),
        ..container::Style::default()
    }
}
