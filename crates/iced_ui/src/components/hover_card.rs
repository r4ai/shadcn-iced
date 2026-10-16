//! For sighted users to preview content available behind a link.
//!
//! Port of <https://ui.shadcn.com/docs/components/hover-card>.
//!
//! ```no_run
//! # use iced_ui::button::{button, Variant};
//! # use iced_ui::hover_card::hover_card;
//! # fn view<'a>() -> iced::Element<'a, ()> {
//! hover_card(button("@nextjs").variant(Variant::Link), iced::widget::text("The React Framework")).into()
//! # }
//! ```
use iced_ui_core::Element;
use iced_ui_core::primitive::{Anchored, Side};

use iced::Length;
use iced::time::Duration;

/// Shows `content` (in a `w-64 p-2.5` panel) below `trigger` after hovering
/// it for 600ms; it closes 300ms after the cursor leaves (Base UI's
/// preview card delays).
pub fn hover_card<'a, Message: Clone + 'a>(
    trigger: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
) -> Anchored<'a, Message> {
    Anchored::new(
        trigger,
        super::popover::panel(content, Length::Fixed(256.0)),
    )
    .hover(Duration::from_millis(600), Duration::from_millis(300))
    .side(Side::Bottom)
    .offset(4.0)
}
