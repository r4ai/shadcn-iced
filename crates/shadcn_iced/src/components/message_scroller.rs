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
//!
//! Like a scrollable `div` of a browser, a click inside the scroller makes
//! it the target of the scrolling keys (<kbd>Home</kbd>, <kbd>End</kbd>,
//! <kbd>PageUp</kbd>, <kbd>PageDown</kbd>, <kbd>Space</kbd>, <kbd>↑</kbd>
//! and <kbd>↓</kbd>).
use super::button::{self, icon_button};

use shadcn_iced_core::icon::lucide;
use shadcn_iced_core::primitive::scroll_keys;
use shadcn_iced_core::theme::alpha;
use shadcn_iced_core::{Element, tokens};

use iced::widget::scrollable::{self, Viewport};
use iced::widget::{Column, Id, Scrollable, Stack, container};
use iced::{Background, Border, Color, Length, Padding, Task, Theme};

/// The width of the stable scrollbar gutter (`scrollbar-thin
/// scrollbar-gutter-stable`), reserved at the end of the viewport.
const GUTTER: f32 = 10.0;

/// The width of the scrollbar thumb, centered in the gutter.
const THUMB: f32 = 6.0;

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
        // `scrollbar-gutter-stable`: the gutter is always reserved, and the
        // thin scrollbar is drawn inside it.
        let padding = Padding {
            right: scroller.padding.right + GUTTER,
            ..scroller.padding
        };

        let mut scrollable = Scrollable::new(
            container(
                Column::with_children(scroller.items)
                    .spacing(24)
                    .width(Length::Fill),
            )
            .padding(padding),
        )
        // The anchor belongs to the scrollbar: set it after the direction.
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::new()
                .width(THUMB)
                .scroller_width(THUMB)
                .margin((GUTTER - THUMB) / 2.0),
        ))
        .anchor_bottom()
        .style(style)
        .width(Length::Fill)
        .height(Length::Fill);

        if let Some(id) = scroller.id {
            scrollable = scrollable.id(id);
        }

        if let Some(on_scroll) = scroller.on_scroll {
            scrollable = scrollable.on_scroll(on_scroll);
        }

        // The scrollable stays the first child of the stack whether the
        // button shows or not, so it keeps its offset.
        let mut content = Stack::new()
            .push(scrollable)
            .width(Length::Fill)
            .height(Length::Fill);

        if let Some(on_press) = scroller.button {
            // `variant="secondary" size="icon-sm"`, restyled with
            // `border-border bg-background hover:bg-muted`, centered at
            // `bottom-4`.
            content = content.push(
                container(
                    icon_button(lucide::ARROW_DOWN)
                        .variant(button::Variant::Outline)
                        .size(button::Size::IconSm)
                        .on_press(on_press),
                )
                .center_x(Length::Fill)
                .align_bottom(Length::Fill)
                .padding(Padding::ZERO.bottom(16)),
            );
        }

        // Focused by a click inside, it then scrolls with the keyboard and
        // shows its outline (`outline-ring/50`).
        scroll_keys(content)
            .anchor_end(true)
            .outline(|theme, status| Border {
                color: alpha(tokens(theme).palette.ring, 0.5),
                width: if status.focus_visible { 1.0 } else { 0.0 },
                ..Border::default()
            })
            .into()
    }
}

/// The style of the viewport: a thin `bg-border` thumb, shown while the
/// pointer is over the conversation, like overlay scrollbars.
fn style(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let style = super::scroll_area::style(theme, status);

    match status {
        scrollable::Status::Hovered { .. } | scrollable::Status::Dragged { .. } => style,
        scrollable::Status::Active { .. } => {
            let mut rail = style.vertical_rail;
            rail.scroller.background = Background::Color(Color::TRANSPARENT);

            scrollable::Style {
                vertical_rail: rail,
                ..style
            }
        }
    }
}

/// Whether the viewport of a [`MessageScroller`] shows the newest message
/// (or everything fits).
///
/// The scroller is anchored at the end, so its offset is measured from the
/// newest message.
pub fn is_at_end(viewport: &Viewport) -> bool {
    viewport.absolute_offset().y <= 1.0
}

/// Scrolls the scroller with the given id to the newest message.
pub fn scroll_to_end<Message: Send + 'static>(id: impl Into<Id>) -> Task<Message> {
    iced::widget::operation::snap_to_end(id)
}
