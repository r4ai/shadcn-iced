//! Displays rich content in a portal, triggered by a button.
//!
//! Port of <https://ui.shadcn.com/docs/components/popover>. Opens on click
//! and closes on its own (click outside or <kbd>Escape</kbd>); call
//! [`Anchored::open`] to control it from your application instead.
//!
//! ```no_run
//! # use iced_ui::button::{button, Variant};
//! # use iced_ui::popover::popover;
//! # fn view<'a>() -> iced::Element<'a, ()> {
//! popover(button("Open popover").variant(Variant::Outline), iced::widget::text("Content")).into()
//! # }
//! ```
use iced_ui_core::primitive::{Align, Anchored, Side};
use iced_ui_core::{Element, font, style, tokens};

use iced::widget::{Column, container, text};
use iced::{Background, Length, Theme};

/// Creates a popover showing `content` (in a `w-72 p-2.5` panel) when
/// `trigger` is clicked, below it and centered (`sideOffset={4}`).
pub fn popover<'a, Message: Clone + 'a>(
    trigger: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
) -> Anchored<'a, Message> {
    Anchored::new(trigger, panel(content, Length::Fixed(288.0)))
        .side(Side::Bottom)
        .align(Align::Center)
        .offset(4.0)
}

/// Creates a popover whose panel has the given width (e.g. `w-80`).
pub fn popover_with_width<'a, Message: Clone + 'a>(
    trigger: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
    width: f32,
) -> Anchored<'a, Message> {
    Anchored::new(trigger, panel(content, Length::Fixed(width)))
        .side(Side::Bottom)
        .align(Align::Center)
        .offset(4.0)
}

/// Wraps content in a popover panel of the given width (`p-2.5 text-sm`).
pub fn panel<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    width: Length,
) -> Element<'a, Message> {
    container(content)
        .padding(10)
        .width(width)
        .style(style)
        .into()
}

/// A title and a description (`PopoverHeader`: `flex flex-col gap-0.5
/// text-sm`).
pub fn header<'a, Message: 'a>(
    title: impl Into<String>,
    description: impl Into<String>,
) -> Element<'a, Message> {
    Column::new()
        .push(
            text(title.into())
                .size(style::text::SM)
                .line_height(iced::Pixels(20.0))
                .font(font::medium()),
        )
        .push(
            text(description.into())
                .size(style::text::SM)
                .line_height(iced::Pixels(20.0))
                .style(|theme: &Theme| text::Style {
                    color: Some(tokens(theme).palette.muted_foreground),
                }),
        )
        .spacing(2)
        .into()
}

/// The style of a popover panel (`rounded-lg bg-popover shadow-md ring-1
/// ring-foreground/10`).
pub fn style(theme: &Theme) -> container::Style {
    let t = tokens(theme);

    container::Style {
        background: Some(Background::Color(t.palette.popover)),
        text_color: Some(t.palette.popover_foreground),
        border: style::ring_border(&t, t.radius_lg()),
        shadow: style::shadow_md(),
        ..container::Style::default()
    }
}
