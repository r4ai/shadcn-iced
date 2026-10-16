//! Displays a card with header, content, and footer.
//!
//! Port of <https://ui.shadcn.com/docs/components/card>.
//!
//! ```no_run
//! # use iced_ui::card::card;
//! # use iced_ui::button::button;
//! # #[derive(Clone)] enum Message { Login }
//! # fn view<'a>() -> iced::Element<'a, Message> {
//! card()
//!     .title("Login to your account")
//!     .description("Enter your email below to login to your account")
//!     .content(iced::widget::text("..."))
//!     .footer(button("Login").on_press(Message::Login))
//!     .width(360)
//!     .into()
//! # }
//! ```
use iced_ui_core::{Element, font, style, tokens};

use iced::widget::{Column, Row, container, text};
use iced::{Background, Length, Theme};

/// A card. See the [module documentation](self).
pub struct Card<'a, Message> {
    title: Option<Element<'a, Message>>,
    description: Option<Element<'a, Message>>,
    action: Option<Element<'a, Message>>,
    content: Vec<Element<'a, Message>>,
    footer: Option<Element<'a, Message>>,
    width: Length,
    header_border: bool,
    footer_border: bool,
}

/// Creates an empty [`Card`].
pub fn card<'a, Message: 'a>() -> Card<'a, Message> {
    Card {
        title: None,
        description: None,
        action: None,
        content: Vec::new(),
        footer: None,
        width: Length::Shrink,
        header_border: false,
        footer_border: false,
    }
}

impl<'a, Message: 'a> Card<'a, Message> {
    /// Sets the title (`leading-none font-semibold`).
    pub fn title(mut self, title: impl text::IntoFragment<'a>) -> Self {
        self.title = Some(
            text(title)
                .size(style::text::BASE)
                .font(font::SEMIBOLD)
                .line_height(iced::Pixels(16.0))
                .into(),
        );
        self
    }

    /// Sets a custom title element.
    pub fn title_element(mut self, title: impl Into<Element<'a, Message>>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the description (`text-sm text-muted-foreground`).
    pub fn description(mut self, description: impl text::IntoFragment<'a>) -> Self {
        self.description = Some(
            text(description)
                .size(style::text::SM)
                .style(|theme: &Theme| text::Style {
                    color: Some(tokens(theme).palette.muted_foreground),
                })
                .into(),
        );
        self
    }

    /// Sets an element displayed in the top-right corner of the header.
    pub fn action(mut self, action: impl Into<Element<'a, Message>>) -> Self {
        self.action = Some(action.into());
        self
    }

    /// Appends content to the body of the card.
    pub fn content(mut self, content: impl Into<Element<'a, Message>>) -> Self {
        self.content.push(content.into());
        self
    }

    /// Sets the footer.
    pub fn footer(mut self, footer: impl Into<Element<'a, Message>>) -> Self {
        self.footer = Some(footer.into());
        self
    }

    /// Separates the header from the content with a border.
    pub fn header_border(mut self, border: bool) -> Self {
        self.header_border = border;
        self
    }

    /// Separates the footer from the content with a border.
    pub fn footer_border(mut self, border: bool) -> Self {
        self.footer_border = border;
        self
    }

    /// Sets the width.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
}

impl<'a, Message: 'a> From<Card<'a, Message>> for Element<'a, Message> {
    fn from(card: Card<'a, Message>) -> Self {
        let mut sections: Vec<Element<'a, Message>> = Vec::new();

        if card.title.is_some() || card.description.is_some() || card.action.is_some() {
            let titles = Column::new()
                .push(card.title)
                .push(card.description)
                .spacing(8)
                .width(Length::Fill);

            let header = Row::new().push(titles).push(card.action).spacing(8);

            sections.push(container(header).padding([0, 24]).into());

            if card.header_border {
                sections.push(super::separator::separator().into());
            }
        }

        for content in card.content {
            sections.push(container(content).padding([0, 24]).width(Length::Fill).into());
        }

        if let Some(footer) = card.footer {
            if card.footer_border {
                sections.push(super::separator::separator().into());
            }

            sections.push(container(footer).padding([0, 24]).width(Length::Fill).into());
        }

        container(Column::with_children(sections).spacing(24))
            .padding([24, 0])
            .width(card.width)
            .style(style)
            .into()
    }
}

/// The style of a card (`rounded-xl border bg-card shadow-sm`).
pub fn style(theme: &Theme) -> container::Style {
    let t = tokens(theme);

    container::Style {
        background: Some(Background::Color(t.palette.card)),
        text_color: Some(t.palette.card_foreground),
        border: style::border(t.palette.border, t.radius_xl()),
        shadow: style::shadow_sm(),
        ..container::Style::default()
    }
}
