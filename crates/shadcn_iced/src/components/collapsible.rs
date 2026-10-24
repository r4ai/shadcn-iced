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
use super::button::{self, Variant};

use shadcn_iced_core::icon::lucide;
use shadcn_iced_core::primitive::Disclosure;
use shadcn_iced_core::{Element, font, style};

use iced::widget::{Row, text};
use iced::{Alignment, Length};

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
    button::icon_button(lucide::CHEVRONS_UP_DOWN)
        .variant(Variant::Ghost)
        .expanded(expanded)
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
