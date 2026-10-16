//! A scrolling list of messages that sticks to the newest one.
//!
//! Port of <https://ui.shadcn.com/docs/components/message-scroller>. The
//! scroller stays anchored to the bottom: new messages appear without
//! having to scroll.
//!
//! ```no_run
//! # use iced_ui::message_scroller::message_scroller;
//! # fn view<'a>(messages: Vec<iced::Element<'a, ()>>) -> iced::Element<'a, ()> {
//! message_scroller(messages).into()
//! # }
//! ```
use iced_ui_core::Element;

use iced::widget::{Column, Scrollable, container, scrollable};
use iced::{Length, Padding};

/// Creates a scroller showing `items` from top to bottom, anchored to the
/// bottom (`gap-8` between items).
pub fn message_scroller<'a, Message: 'a>(
    items: impl IntoIterator<Item = Element<'a, Message>>,
) -> Scrollable<'a, Message> {
    Scrollable::new(
        container(Column::with_children(items).spacing(32).width(Length::Fill))
            .padding(Padding::new(16.0).right(20.0)),
    )
    .anchor_bottom()
    .direction(scrollable::Direction::Vertical(
        super::scroll_area::scrollbar(),
    ))
    .style(super::scroll_area::style)
    .width(Length::Fill)
    .height(Length::Fill)
}
