//! A popup that displays information related to an element when the
//! element is hovered.
//!
//! Port of <https://ui.shadcn.com/docs/components/tooltip>.
//!
//! ```no_run
//! # use iced_ui::button::{button, Variant};
//! # use iced_ui::tooltip::tooltip;
//! # fn view<'a>() -> iced::Element<'a, ()> {
//! tooltip(button("Hover").variant(Variant::Outline), "Add to library").into()
//! # }
//! ```
use iced_ui_core::primitive::{Anchored, Side};
use iced_ui_core::{Element, style, tokens};

use iced::time::Duration;
use iced::widget::{container, text};
use iced::{Background, Theme, border};

/// Shows `content` above `trigger` while it is hovered.
pub fn tooltip<'a, Message: Clone + 'a>(
    trigger: impl Into<Element<'a, Message>>,
    content: impl text::IntoFragment<'a>,
) -> Anchored<'a, Message> {
    custom(trigger, text(content).size(style::text::XS).line_height(iced::Pixels(16.0)))
}

/// Shows any element as a tooltip.
pub fn custom<'a, Message: Clone + 'a>(
    trigger: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
) -> Anchored<'a, Message> {
    Anchored::new(trigger, container(content).padding([6, 12]).style(self::style))
        .hover(Duration::ZERO, Duration::ZERO)
        .side(Side::Top)
        .offset(6.0)
}

/// The style of a tooltip (`bg-foreground text-background`).
pub fn style(theme: &Theme) -> container::Style {
    let t = tokens(theme);

    container::Style {
        background: Some(Background::Color(t.palette.foreground)),
        text_color: Some(t.palette.background),
        border: border::rounded(t.radius_md()),
        ..container::Style::default()
    }
}
