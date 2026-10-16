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
use iced::{Alignment, Length, Theme};

/// A chat message. See the [module documentation](self).
pub struct Message<'a, M> {
    avatar: Option<Element<'a, M>>,
    header: Option<String>,
    content: Vec<Element<'a, M>>,
    footer: Option<String>,
    align: Align,
}

/// Creates an empty [`Message`].
pub fn message<'a, M: 'a>() -> Message<'a, M> {
    Message {
        avatar: None,
        header: None,
        content: Vec::new(),
        footer: None,
        align: Align::Start,
    }
}

impl<'a, M: 'a> Message<'a, M> {
    /// Sets the avatar, shown next to the last bubble.
    pub fn avatar(mut self, avatar: impl Into<Element<'a, M>>) -> Self {
        self.avatar = Some(avatar.into());
        self
    }

    /// Sets the header, usually the name of the author.
    pub fn header(mut self, header: impl Into<String>) -> Self {
        self.header = Some(header.into());
        self
    }

    /// Appends a bubble, aligned like the message.
    pub fn bubble(mut self, bubble: Bubble<'a, M>) -> Self {
        self.content.push(bubble.align(self.align).into());
        self
    }

    /// Appends arbitrary content (attachments, markers...).
    pub fn push(mut self, content: impl Into<Element<'a, M>>) -> Self {
        self.content.push(content.into());
        self
    }

    /// Sets the footer (time, status...).
    pub fn footer(mut self, footer: impl Into<String>) -> Self {
        self.footer = Some(footer.into());
        self
    }

    /// Aligns the message: [`Align::End`] for outgoing messages. Call it
    /// before adding bubbles.
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }
}

fn caption<'a, M: 'a>(content: String, align: Align) -> Element<'a, M> {
    let caption = container(text(content).size(style::text::XS).font(font::MEDIUM).style(|theme: &Theme| {
        text::Style {
            color: Some(tokens(theme).palette.muted_foreground),
        }
    }))
    .padding([0, 12])
    .width(Length::Fill);

    match align {
        Align::Start => caption.align_left(Length::Fill).into(),
        Align::End => caption.align_right(Length::Fill).into(),
    }
}

impl<'a, M: 'a> From<Message<'a, M>> for Element<'a, M> {
    fn from(message: Message<'a, M>) -> Self {
        let align = message.align;

        let content = Column::new()
            .push(message.header.map(|header| caption(header, align)))
            .extend(message.content)
            .push(message.footer.map(|footer| caption(footer, align)))
            .spacing(10)
            .width(Length::Fill)
            .align_x(match align {
                Align::Start => Alignment::Start,
                Align::End => Alignment::End,
            });

        let avatar = message.avatar.map(|avatar| container(avatar).align_bottom(Length::Shrink));

        let row = match align {
            Align::Start => Row::new().push(avatar).push(content),
            Align::End => Row::new().push(content).push(avatar),
        };

        row.spacing(8).align_y(Alignment::End).width(Length::Fill).into()
    }
}

/// Stacks messages (`MessageGroup`).
pub fn message_group<'a, M: 'a>(messages: impl IntoIterator<Item = Element<'a, M>>) -> Element<'a, M> {
    Column::with_children(messages).spacing(8).width(Length::Fill).into()
}
