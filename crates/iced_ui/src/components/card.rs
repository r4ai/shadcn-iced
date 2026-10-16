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
use iced_ui_core::theme::alpha;
use iced_ui_core::{Element, font, style, tokens};

use iced::widget::{Column, Row, container, text};
use iced::{Background, Border, Length, Padding, Theme, border};

/// The size of a [`Card`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Size {
    /// `--card-spacing: --spacing(4)`
    #[default]
    Default,
    /// `--card-spacing: --spacing(3)`, with a `text-sm` title.
    Sm,
}

impl Size {
    /// The padding of the card and the gap between its sections
    /// (`--card-spacing`).
    pub fn spacing(self) -> f32 {
        match self {
            Size::Default => 16.0,
            Size::Sm => 12.0,
        }
    }
}

/// A card. See the [module documentation](self).
pub struct Card<'a, Message> {
    media: Option<Element<'a, Message>>,
    title: Option<Title<'a, Message>>,
    description: Option<Element<'a, Message>>,
    action: Option<Element<'a, Message>>,
    content: Vec<Element<'a, Message>>,
    footer: Option<Element<'a, Message>>,
    width: Length,
    size: Size,
    header_border: bool,
}

enum Title<'a, Message> {
    Text(text::Fragment<'a>),
    Element(Element<'a, Message>),
}

/// Creates an empty [`Card`].
pub fn card<'a, Message: 'a>() -> Card<'a, Message> {
    Card {
        media: None,
        title: None,
        description: None,
        action: None,
        content: Vec::new(),
        footer: None,
        width: Length::Fill,
        size: Size::Default,
        header_border: false,
    }
}

impl<'a, Message: 'a> Card<'a, Message> {
    /// Sets the title (`text-base leading-snug font-medium`, `text-sm` in
    /// small cards).
    pub fn title(mut self, title: impl text::IntoFragment<'a>) -> Self {
        self.title = Some(Title::Text(title.into_fragment()));
        self
    }

    /// Sets a custom title element.
    pub fn title_element(mut self, title: impl Into<Element<'a, Message>>) -> Self {
        self.title = Some(Title::Element(title.into()));
        self
    }

    /// Sets the description (`text-sm text-muted-foreground`).
    pub fn description(mut self, description: impl text::IntoFragment<'a>) -> Self {
        self.description = Some(
            text(description)
                .size(style::text::SM)
                .line_height(iced::Pixels(20.0))
                .style(|theme: &Theme| text::Style {
                    color: Some(tokens(theme).palette.muted_foreground),
                })
                .into(),
        );
        self
    }

    /// Sets a custom description element.
    pub fn description_element(mut self, description: impl Into<Element<'a, Message>>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets an element displayed in the top-right corner of the header
    /// (`CardAction`).
    pub fn action(mut self, action: impl Into<Element<'a, Message>>) -> Self {
        self.action = Some(action.into());
        self
    }

    /// Shows an edge-to-edge element, usually an image, at the top of the
    /// card (`has-[>img:first-child]:pt-0`).
    pub fn media(mut self, media: impl Into<Element<'a, Message>>) -> Self {
        self.media = Some(media.into());
        self
    }

    /// Appends content to the body of the card.
    pub fn content(mut self, content: impl Into<Element<'a, Message>>) -> Self {
        self.content.push(content.into());
        self
    }

    /// Sets the footer (`border-t bg-muted/50`).
    pub fn footer(mut self, footer: impl Into<Element<'a, Message>>) -> Self {
        self.footer = Some(footer.into());
        self
    }

    /// Separates the header from the content with a border
    /// (`CardHeader className="border-b"`).
    pub fn header_border(mut self, border: bool) -> Self {
        self.header_border = border;
        self
    }

    /// Sets the size.
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// Sets the width. Cards fill the available width by default, like a
    /// block element.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
}

impl<'a, Message: 'a> From<Card<'a, Message>> for Element<'a, Message> {
    fn from(card: Card<'a, Message>) -> Self {
        let size = card.size;
        let spacing = size.spacing();
        let mut sections: Vec<Element<'a, Message>> = Vec::new();

        // Sections stretch to the card, unless the card shrinks to fit them.
        let inner = if card.width == Length::Shrink {
            Length::Shrink
        } else {
            Length::Fill
        };

        let has_media = card.media.is_some();

        if let Some(media) = card.media {
            sections.push(
                container(media)
                    .width(inner)
                    .clip(true)
                    .style(move |theme: &Theme| container::Style {
                        border: Border {
                            radius: border::radius(0)
                                .top_left(radius(theme))
                                .top_right(radius(theme)),
                            ..Border::default()
                        },
                        ..container::Style::default()
                    })
                    .into(),
            );
        }

        if card.title.is_some() || card.description.is_some() || card.action.is_some() {
            let title = card.title.map(|title| -> Element<'a, Message> {
                match title {
                    Title::Text(fragment) => {
                        let text_size = match size {
                            Size::Default => style::text::BASE,
                            Size::Sm => style::text::SM,
                        };

                        text(fragment)
                            .size(text_size)
                            // `cn-font-heading`
                            .font(font::heading(iced::font::Weight::Medium))
                            .line_height(1.375)
                            .into()
                    }
                    Title::Element(element) => element,
                }
            });

            let titles = Column::new()
                .push(title)
                .push(card.description)
                .spacing(4)
                .width(inner);

            let header = Row::new().push(titles).push(card.action).spacing(spacing);

            sections.push(
                container(header)
                    .padding(Padding {
                        bottom: if card.header_border { spacing } else { 0.0 },
                        ..Padding::from([0.0, spacing])
                    })
                    .width(inner)
                    .into(),
            );

            if card.header_border {
                sections.push(super::separator::separator().into());
            }
        }

        for content in card.content {
            sections.push(
                container(content)
                    .padding([0.0, spacing])
                    .width(inner)
                    .into(),
            );
        }

        let has_footer = card.footer.is_some();

        if let Some(footer) = card.footer {
            sections.push(
                Column::new()
                    .push(super::separator::separator())
                    .push(
                        container(footer)
                            .padding(spacing)
                            .width(inner)
                            .style(footer_style),
                    )
                    .width(inner)
                    .into(),
            );
        }

        // The 1px ring is drawn outside of the box in CSS: inset the
        // content so that the footer background doesn't cover it.
        container(
            Column::with_children(sections)
                .spacing(spacing)
                .width(inner)
                .padding(Padding {
                    top: if has_media { 0.0 } else { spacing },
                    bottom: if has_footer { 0.0 } else { spacing },
                    ..Padding::ZERO
                }),
        )
        .padding(1)
        .width(card.width)
        .clip(true)
        .style(style)
        .into()
    }
}

/// The inner radius of a card, inside its 1px ring.
fn radius(theme: &Theme) -> f32 {
    (tokens(theme).radius_xl() - 1.0).max(0.0)
}

/// The style of a card footer (`bg-muted/50 rounded-b-xl border-t`).
fn footer_style(theme: &Theme) -> container::Style {
    let t = tokens(theme);

    container::Style {
        background: Some(Background::Color(alpha(t.palette.muted, 0.5))),
        border: Border {
            radius: border::radius(0)
                .bottom_left(radius(theme))
                .bottom_right(radius(theme)),
            ..Border::default()
        },
        ..container::Style::default()
    }
}

/// The style of a card (`bg-card text-card-foreground rounded-xl ring-1
/// ring-foreground/10`).
pub fn style(theme: &Theme) -> container::Style {
    let t = tokens(theme);

    container::Style {
        background: Some(Background::Color(t.palette.card)),
        text_color: Some(t.palette.card_foreground),
        border: style::ring_border(&t, t.radius_xl()),
        ..container::Style::default()
    }
}
