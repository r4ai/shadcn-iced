//! For sighted users to preview content available behind a link.
//!
//! Port of <https://ui.shadcn.com/docs/components/hover-card>.
//!
//! ```no_run
//! # use shadcn_iced::button::{button, Variant};
//! # use shadcn_iced::hover_card::hover_card;
//! # fn view<'a>() -> iced::Element<'a, ()> {
//! hover_card(button("@nextjs").variant(Variant::Link), iced::widget::text("The React Framework")).into()
//! # }
//! ```
use shadcn_iced_core::Element;
use shadcn_iced_core::primitive::anchored::TouchOpen;
use shadcn_iced_core::primitive::{Anchored, Side};

use iced::Length;
use iced::time::Duration;

/// Shows `content` (in a `w-64 p-2.5` panel) below `trigger` after hovering
/// it for 600ms; it closes 300ms after the cursor leaves (Base UI's
/// preview card delays). It also opens when the trigger receives the
/// keyboard focus, and with a long press on a touch screen, until a tap
/// outside (a tap on the trigger still presses it).
pub fn hover_card<'a, Message: Clone + 'a>(
    trigger: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
) -> Anchored<'a, Message> {
    Anchored::new(
        trigger,
        super::popover::panel(content, Length::Fixed(256.0)),
    )
    .hover(Duration::from_millis(600), Duration::from_millis(300))
    .touch_open(TouchOpen::LongPress)
    .side(Side::Bottom)
    .offset(4.0)
    // `ring-1` around the panel.
    .outset(1.0)
    // Base UI keeps popups 5px away from the edges of the window.
    .collision_padding(5.0)
}
