//! Displays rich content in a portal, triggered by a button.
//!
//! Port of <https://ui.shadcn.com/docs/components/popover>. Opens on click
//! and closes on its own (click outside or <kbd>Escape</kbd>); call
//! [`Anchored::open`] to control it from your application instead.
//!
//! ```no_run
//! # use shadcn_iced::button::{button, Variant};
//! # use shadcn_iced::popover::popover;
//! # fn view<'a>() -> iced::Element<'a, ()> {
//! popover(button("Open popover").variant(Variant::Outline), iced::widget::text("Content")).into()
//! # }
//! ```
use shadcn_iced_core::primitive::{Align, Anchored, Side, box_shadow};
use shadcn_iced_core::{Element, font, style, tokens};

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
        // `ring-1` around the panel.
        .outset(1.0)
        // `focus-visible:border-ring focus-visible:ring-3` of a button,
        // once dismissed with the keyboard.
        .focus_ring(|theme| {
            let t = tokens(theme);
            style::focus_visible(&t, t.radius_lg())
        })
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
        // `ring-1` around the panel.
        .outset(1.0)
        // `focus-visible:border-ring focus-visible:ring-3` of a button,
        // once dismissed with the keyboard.
        .focus_ring(|theme| {
            let t = tokens(theme);
            style::focus_visible(&t, t.radius_lg())
        })
}

/// Wraps content in a popover panel of the given width (`p-2.5 text-sm`),
/// with its `ring-1` around it: the panel is 2px larger, to be placed with
/// [`Anchored::outset`].
pub fn panel<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    width: Length,
) -> Element<'a, Message> {
    shadow(
        container(content)
            .padding(1.0 + 10.0)
            .width(match width {
                Length::Fixed(width) => Length::Fixed(width + 2.0),
                width => width,
            })
            .style(style),
    )
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
        ..container::Style::default()
    }
}

/// Draws the shadow of a popover panel (`shadow-md`) behind it, which a
/// container style cannot (see [`box_shadow`]).
pub fn shadow<'a, Message: 'a>(panel: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    box_shadow(panel, style::SHADOW_MD, |theme| tokens(theme).radius_lg())
        // The `ring-1` is outside of the box that casts the shadow.
        .inset(1.0)
        .into()
}
