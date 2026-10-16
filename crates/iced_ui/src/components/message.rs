//! A chat message: an avatar, a header, bubbles and a footer.
//!
//! Port of <https://ui.shadcn.com/docs/components/message>.
//!
//! ```no_run
//! # use iced_ui::message::message;
//! # use iced_ui::bubble::{bubble, Align, Variant};
//! # use iced_ui::avatar::avatar;
//! # fn view<'a>() -> iced::Element<'a, ()> {
//! message()
//!     .avatar(avatar("CN"))
//!     .header("shadcn")
//!     .bubble(bubble("Hello there!").variant(Variant::Muted))
//!     .footer("Read 2m ago")
//!     .into()
//! # }
//! ```
use super::bubble::{Align, Bubble};

use iced_ui_core::{Element, font, style, tokens};

use iced::widget::{Column, Row, container, text};
use iced::{Alignment, Length, Padding, Theme};

/// A chat message. See the [module documentation](self).
pub struct Message<'a, M> {
    avatar: Option<Element<'a, M>>,
    header: Option<Element<'a, M>>,
    content: Vec<Element<'a, M>>,
    footer: Option<Element<'a, M>>,
    align: Align,
    ghost: bool,
}

/// Creates an empty [`Message`].
pub fn message<'a, M: 'a>() -> Message<'a, M> {
    Message {
        avatar: None,
        header: None,
        content: Vec::new(),
        footer: None,
        align: Align::Start,
        ghost: false,
    }
}

/// `text-xs font-medium text-muted-foreground`
fn caption<'a, M: 'a>(content: String) -> Element<'a, M> {
    text(content)
        .size(style::text::XS)
        .font(font::medium())
        .style(muted)
        .into()
}

fn muted(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(tokens(theme).palette.muted_foreground),
    }
}

impl<'a, M: Clone + 'a> Message<'a, M> {
    /// Sets the avatar (`MessageAvatar`), aligned with the last bubble.
    pub fn avatar(mut self, avatar: impl Into<Element<'a, M>>) -> Self {
        self.avatar = Some(avatar.into());
        self
    }

    /// Sets the header, usually the name of the author.
    pub fn header(self, header: impl Into<String>) -> Self {
        self.header_content(caption(header.into()))
    }

    /// Sets an arbitrary header (`MessageHeader`).
    pub fn header_content(mut self, header: impl Into<Element<'a, M>>) -> Self {
        self.header = Some(header.into());
        self
    }

    /// Appends a bubble, aligned like the message.
    pub fn bubble(mut self, bubble: Bubble<'a, M>) -> Self {
        self.ghost |= bubble.is_ghost();
        self.content.push(bubble.align(self.align).into());
        self
    }

    /// Appends arbitrary content (a bubble group, attachments, markers...).
    pub fn push(mut self, content: impl Into<Element<'a, M>>) -> Self {
        self.content.push(content.into());
        self
    }

    /// Sets the footer (time, status...).
    pub fn footer(self, footer: impl Into<String>) -> Self {
        self.footer_content(caption(footer.into()))
    }

    /// Sets an arbitrary footer (`MessageFooter`), e.g. action buttons.
    pub fn footer_content(mut self, footer: impl Into<Element<'a, M>>) -> Self {
        self.footer = Some(footer.into());
        self
    }

    /// Aligns the message: [`Align::End`] for outgoing messages
    /// (`flex-row-reverse`). Call it before adding bubbles.
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    /// Removes the padding of the header and footer, for messages made of
    /// ghost bubbles added with [`push`](Self::push).
    pub fn ghost(mut self, ghost: bool) -> Self {
        self.ghost = ghost;
        self
    }
}

impl<'a, M: 'a> From<Message<'a, M>> for Element<'a, M> {
    fn from(message: Message<'a, M>) -> Self {
        let align = message.align;
        let has_footer = message.footer.is_some();
        // `px-3 group-has-data-[variant=ghost]/message:px-0`
        let inset = if message.ghost { 0 } else { 12 };

        let slot = |content: Element<'a, M>| {
            let slot = container(content).padding([0, inset]).width(Length::Fill);

            match align {
                Align::Start => slot.align_left(Length::Fill),
                Align::End => slot.align_right(Length::Fill),
            }
        };

        // `gap-2.5`
        let content = Column::new()
            .push(message.header.map(slot))
            .extend(message.content)
            .push(message.footer.map(slot))
            .spacing(10)
            .width(Length::Fill)
            .align_x(match align {
                Align::Start => Alignment::Start,
                Align::End => Alignment::End,
            });

        // `min-w-8 self-end`, raised by `-translate-y-8` above the footer.
        let avatar = message.avatar.map(|avatar| {
            container(avatar).padding(Padding::ZERO.bottom(if has_footer { 32 } else { 0 }))
        });

        let row = match align {
            Align::Start => Row::new().push(avatar).push(content),
            Align::End => Row::new().push(content).push(avatar),
        };

        // `gap-2`
        row.spacing(8)
            .align_y(Alignment::End)
            .width(Length::Fill)
            .into()
    }
}

/// Stacks messages (`MessageGroup`, `gap-2`).
pub fn message_group<'a, M: 'a>(
    messages: impl IntoIterator<Item = Element<'a, M>>,
) -> Element<'a, M> {
    Column::with_children(messages)
        .spacing(8)
        .width(Length::Fill)
        .into()
}
