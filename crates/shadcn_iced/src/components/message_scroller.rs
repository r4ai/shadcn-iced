//! A scrolling list of messages that sticks to the newest one.
//!
//! Port of <https://ui.shadcn.com/docs/components/message-scroller>. The
//! scroller stays anchored to the bottom: new messages appear without
//! having to scroll. A "scroll to end" button can be shown while the reader
//! is away from the newest message:
//!
//! ```no_run
//! # use shadcn_iced::message_scroller::{self, message_scroller};
//! # #[derive(Clone)] enum Message { Scrolled(bool), ScrollToEnd }
//! # fn view<'a>(messages: Vec<iced::Element<'a, Message>>, at_end: bool) -> iced::Element<'a, Message> {
//! message_scroller(messages)
//!     .id("chat")
//!     .on_scroll(|viewport| Message::Scrolled(message_scroller::is_at_end(&viewport)))
//!     .button((!at_end).then_some(Message::ScrollToEnd))
//!     .into()
//! # }
//! ```
//!
//! Answer `Message::ScrollToEnd` with [`scroll_to_end`].
use super::button::{Size, Variant, icon_button};

use shadcn_iced_core::Element;
use shadcn_iced_core::icon::lucide;

use iced::widget::scrollable::{self, Viewport};
use iced::widget::{Column, Id, Scrollable, Stack, container};
use iced::{Length, Padding, Task};

/// A message scroller. See the [module documentation](self).
pub struct MessageScroller<'a, Message> {
    items: Vec<Element<'a, Message>>,
    id: Option<Id>,
    padding: Padding,
    on_scroll: Option<Box<dyn Fn(Viewport) -> Message + 'a>>,
    button: Option<Message>,
}

/// Creates a scroller showing `items` from top to bottom, anchored to the
/// bottom (`MessageScrollerContent`, `gap-6`).
pub fn message_scroller<'a, Message: Clone + 'a>(
    items: impl IntoIterator<Item = Element<'a, Message>>,
) -> MessageScroller<'a, Message> {
    MessageScroller {
        items: items.into_iter().collect(),
        id: None,
        padding: Padding::new(16.0),
        on_scroll: None,
        button: None,
    }
}

impl<'a, Message: Clone + 'a> MessageScroller<'a, Message> {
    /// Sets the id of the scroller, used by [`scroll_to_end`].
    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the padding of the content (`p-(--card-spacing)` in the
    /// examples; 16 by default).
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Reports the viewport whenever it scrolls; see [`is_at_end`].
    pub fn on_scroll(mut self, f: impl Fn(Viewport) -> Message + 'a) -> Self {
        self.on_scroll = Some(Box::new(f));
        self
    }

    /// Shows the scroll to end button (`MessageScrollerButton`) producing
    /// the given message, if any.
    pub fn button(mut self, on_press: Option<Message>) -> Self {
        self.button = on_press;
        self
    }
}

impl<'a, Message: Clone + 'a> From<MessageScroller<'a, Message>> for Element<'a, Message> {
    fn from(scroller: MessageScroller<'a, Message>) -> Self {
        let mut scrollable = Scrollable::new(
            container(
                Column::with_children(scroller.items)
                    .spacing(24)
                    .width(Length::Fill),
            )
            .padding(scroller.padding),
        )
        // The anchor belongs to the scrollbar: set it after the direction.
        .direction(scrollable::Direction::Vertical(
            super::scroll_area::scrollbar(),
        ))
        .anchor_bottom()
        .style(super::scroll_area::style)
        .width(Length::Fill)
        .height(Length::Fill);

        if let Some(id) = scroller.id {
            scrollable = scrollable.id(id);
        }

        if let Some(on_scroll) = scroller.on_scroll {
            scrollable = scrollable.on_scroll(on_scroll);
        }

        let Some(on_press) = scroller.button else {
            return scrollable.into();
        };

        // `variant="secondary" size="icon-sm"`, restyled with `border-border
        // bg-background hover:bg-muted`, centered at `bottom-4`.
        let button = container(
            icon_button(lucide::ARROW_DOWN)
                .variant(Variant::Outline)
                .size(Size::IconSm)
                .on_press(on_press),
        )
        .center_x(Length::Fill)
        .align_bottom(Length::Fill)
        .padding(Padding::ZERO.bottom(16));

        Stack::new()
            .push(scrollable)
            .push(button)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

/// Whether the viewport shows the newest message (or everything fits).
pub fn is_at_end(viewport: &Viewport) -> bool {
    let bounds = viewport.bounds();
    let content = viewport.content_bounds();
    let offset = viewport.absolute_offset();

    content.height <= bounds.height || offset.y + bounds.height >= content.height - 1.0
}

/// Scrolls the scroller with the given id to the newest message.
pub fn scroll_to_end<Message: Send + 'static>(id: impl Into<Id>) -> Task<Message> {
    iced::widget::operation::snap_to_end(id)
}
