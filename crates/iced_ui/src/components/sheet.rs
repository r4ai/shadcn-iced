//! Extends the Dialog component to display content that complements the
//! main content of the screen.
//!
//! Port of <https://ui.shadcn.com/docs/components/sheet>.
//!
//! ```no_run
//! # use iced_ui::button::{button, Variant};
//! # use iced_ui::sheet::sheet;
//! # #[derive(Clone)] enum Message { Open, Close }
//! # fn view<'a>(open: bool) -> iced::Element<'a, Message> {
//! sheet(button("Open").variant(Variant::Outline).on_press(Message::Open), open)
//!     .title("Edit profile")
//!     .on_close(Message::Close)
//!     .into()
//! # }
//! ```
use iced_ui_core::primitive::Layer;
use iced_ui_core::primitive::layer::{Animation, Edge};
use iced_ui_core::{Element, font, style, tokens};

use iced::widget::{Column, Stack, container, space, text};
use iced::{Background, Length, Theme};

/// The edge of the window a [`Sheet`] slides from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Side {
    /// From the top.
    Top,
    /// From the right.
    #[default]
    Right,
    /// From the bottom.
    Bottom,
    /// From the left.
    Left,
}

/// A sheet. See the [module documentation](self).
pub struct Sheet<'a, Message> {
    base: Element<'a, Message>,
    open: bool,
    side: Side,
    title: Option<String>,
    description: Option<String>,
    content: Vec<Element<'a, Message>>,
    footer: Option<Element<'a, Message>>,
    on_close: Option<Message>,
    size: f32,
}

/// Creates a [`Sheet`] shown over the whole window when `open`.
pub fn sheet<'a, Message: Clone + 'a>(
    base: impl Into<Element<'a, Message>>,
    open: bool,
) -> Sheet<'a, Message> {
    Sheet {
        base: base.into(),
        open,
        side: Side::Right,
        title: None,
        description: None,
        content: Vec::new(),
        footer: None,
        on_close: None,
        size: 384.0,
    }
}

impl<'a, Message: Clone + 'a> Sheet<'a, Message> {
    /// Sets the side.
    pub fn side(mut self, side: Side) -> Self {
        self.side = side;
        self
    }

    /// Sets the title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Appends content.
    pub fn content(mut self, content: impl Into<Element<'a, Message>>) -> Self {
        self.content.push(content.into());
        self
    }

    /// Sets the footer, pushed to the bottom.
    pub fn footer(mut self, footer: impl Into<Element<'a, Message>>) -> Self {
        self.footer = Some(footer.into());
        self
    }

    /// Sets the message produced when the user closes the sheet.
    pub fn on_close(mut self, message: Message) -> Self {
        self.on_close = Some(message);
        self
    }

    /// Sets the width of a left/right sheet. Defaults to `sm:max-w-sm`.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
}

impl<'a, Message: Clone + 'a> From<Sheet<'a, Message>> for Element<'a, Message> {
    fn from(sheet: Sheet<'a, Message>) -> Self {
        let vertical = matches!(sheet.side, Side::Left | Side::Right);

        let header = (sheet.title.is_some() || sheet.description.is_some()).then(|| {
            Column::new()
                .push(
                    sheet
                        .title
                        .map(|title| text(title).size(style::text::BASE).font(font::SEMIBOLD)),
                )
                .push(sheet.description.map(|description| {
                    text(description)
                        .size(style::text::SM)
                        .style(|theme: &Theme| text::Style {
                            color: Some(tokens(theme).palette.muted_foreground),
                        })
                }))
                .spacing(6)
                .padding(16)
        });

        let content = Column::with_children(sheet.content.into_iter().map(|element| {
            container(element)
                .padding([0, 16])
                .width(Length::Fill)
                .into()
        }))
        .spacing(16);

        let footer = sheet
            .footer
            .map(|footer| container(footer).padding(16).width(Length::Fill));

        let body = Column::new()
            .push(header)
            .push(content)
            .push(vertical.then(space::vertical))
            .push(footer)
            .spacing(16)
            .width(Length::Fill)
            .height(if vertical {
                Length::Fill
            } else {
                Length::Shrink
            });

        let layers = Stack::new().push(body).push(
            container(super::dialog::close_button(sheet.on_close.clone()))
                .padding(16)
                .align_right(Length::Fill),
        );

        let side = sheet.side;

        let panel = container(layers)
            .width(if vertical {
                Length::Fixed(sheet.size)
            } else {
                Length::Fill
            })
            .height(if vertical {
                Length::Fill
            } else {
                Length::Shrink
            })
            .style(move |theme: &Theme| {
                let t = tokens(theme);

                container::Style {
                    background: Some(Background::Color(t.palette.background)),
                    text_color: Some(t.palette.foreground),
                    border: style::border(t.palette.border, 0.0),
                    shadow: style::shadow_lg(),
                    ..container::Style::default()
                }
            });

        let (align_x, align_y) = match side {
            Side::Top => (
                iced::alignment::Horizontal::Center,
                iced::alignment::Vertical::Top,
            ),
            Side::Right => (
                iced::alignment::Horizontal::Right,
                iced::alignment::Vertical::Center,
            ),
            Side::Bottom => (
                iced::alignment::Horizontal::Center,
                iced::alignment::Vertical::Bottom,
            ),
            Side::Left => (
                iced::alignment::Horizontal::Left,
                iced::alignment::Vertical::Center,
            ),
        };

        let edge = match side {
            Side::Top => Edge::Top,
            Side::Right => Edge::Right,
            Side::Bottom => Edge::Bottom,
            Side::Left => Edge::Left,
        };

        Layer::new(sheet.base, panel, sheet.open)
            .animation(Animation::Slide(edge))
            .align_x(align_x)
            .align_y(align_y)
            .on_dismiss_maybe(sheet.on_close)
            .into()
    }
}
