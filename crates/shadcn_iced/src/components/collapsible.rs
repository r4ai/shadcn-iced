//! An interactive component which expands/collapses a panel.
//!
//! Port of <https://ui.shadcn.com/docs/components/collapsible>.
//!
//! ```no_run
//! # use shadcn_iced::collapsible::collapsible;
//! # fn view<'a>() -> iced::Element<'a, ()> {
//! collapsible("@peduarte starred 3 repositories", iced::widget::text("@radix-ui/primitives")).into()
//! # }
//! ```
use super::button::{self, Size, Variant};

use shadcn_iced_core::icon::lucide;
use shadcn_iced_core::primitive::{Disclosure, Pressable};
use shadcn_iced_core::{Element, font, style, tokens};

use iced::widget::{Row, container, text};
use iced::{Alignment, Background, Length, border};

/// Creates a collapsible with a text header (`text-sm font-semibold`) and
/// a ghost chevron button (`size-8`) in a `px-4 gap-4` row, like the
/// shadcn/ui example. The content is shown `gap-2` below. For custom triggers, use [`custom`].
pub fn collapsible<'a, Message: Clone + 'a>(
    title: impl Into<String>,
    content: impl Into<Element<'a, Message>>,
) -> Disclosure<'a, Message> {
    let title = title.into();

    let header = |title: String, expanded: bool| -> Element<'a, Message> {
        Row::new()
            .push(
                text(title)
                    .size(style::text::SM)
                    .line_height(style::text::leading(style::text::SM))
                    .font(font::semibold())
                    .width(Length::Fill),
            )
            .push(trigger::<Message>(expanded))
            .spacing(16)
            .padding([0, 16])
            .align_y(Alignment::Center)
            .into()
    };

    Disclosure::new(header(title.clone(), false), header(title, true), content).spacing(8.0)
}

/// The ghost chevron button (`Button variant="ghost" size="icon"`) of the
/// collapsible: it takes the keyboard focus, and the disclosure toggles on
/// Enter and Space. It stays muted while the content is shown
/// (`aria-expanded:bg-muted`).
fn trigger<'a, Message: Clone + 'a>(expanded: bool) -> Element<'a, Message> {
    let size = Size::Icon;

    Pressable::new(container(lucide::CHEVRONS_UP_DOWN.sized(size.icon_size())).center(Length::Fill))
        .focusable(true)
        .width(size.height())
        .height(size.height())
        .press_offset(1.0)
        // `border border-transparent bg-clip-padding`
        .clip_background(true)
        .focus_style(move |theme, status, focused| {
            let t = tokens(theme);
            let mut style = button::style(theme, status, Variant::Ghost);

            style.border.radius = border::radius(size.radius(&t));

            if expanded {
                style.background = Some(Background::Color(t.palette.muted));
                style.text_color = t.palette.foreground;
            }

            // `focus-visible:border-ring`
            if focused {
                style.border.color = t.palette.ring;
            }

            style
        })
        // `focus-visible:ring-3 focus-visible:ring-ring/50`
        .ring(move |theme, _status, focused| {
            focused.then(|| {
                let t = tokens(theme);

                style::ring(&t, size.radius(&t))
            })
        })
        .into()
}

/// Creates a collapsible with custom triggers for the closed and open states.
pub fn custom<'a, Message: 'a>(
    closed: impl Into<Element<'a, Message>>,
    opened: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
) -> Disclosure<'a, Message> {
    Disclosure::new(closed, opened, content)
}
