//! A drawer component for iced, sliding from the bottom of the window.
//!
//! Port of <https://ui.shadcn.com/docs/components/drawer> (vaul).
//!
//! ```no_run
//! # use iced_ui::button::{button, Variant};
//! # use iced_ui::drawer::drawer;
//! # #[derive(Clone)] enum Message { Open, Close }
//! # fn view<'a>(open: bool) -> iced::Element<'a, Message> {
//! drawer(button("Open Drawer").variant(Variant::Outline).on_press(Message::Open), open)
//!     .title("Move Goal")
//!     .description("Set your daily activity goal.")
//!     .on_close(Message::Close)
//!     .into()
//! # }
//! ```
use iced_ui_core::primitive::Layer;
use iced_ui_core::{Element, font, style, tokens};

use iced::widget::{Column, container, space, text};
use iced::{Alignment, Background, Length, Theme, border};

/// A drawer. See the [module documentation](self).
pub struct Drawer<'a, Message> {
    base: Element<'a, Message>,
    open: bool,
    title: Option<String>,
    description: Option<String>,
    content: Vec<Element<'a, Message>>,
    footer: Option<Element<'a, Message>>,
    on_close: Option<Message>,
    max_width: f32,
}

/// Creates a [`Drawer`] shown at the bottom of the window when `open`.
pub fn drawer<'a, Message: Clone + 'a>(base: impl Into<Element<'a, Message>>, open: bool) -> Drawer<'a, Message> {
    Drawer {
        base: base.into(),
        open,
        title: None,
        description: None,
        content: Vec::new(),
        footer: None,
        on_close: None,
        max_width: 384.0,
    }
}

impl<'a, Message: Clone + 'a> Drawer<'a, Message> {
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

    /// Sets the footer.
    pub fn footer(mut self, footer: impl Into<Element<'a, Message>>) -> Self {
        self.footer = Some(footer.into());
        self
    }

    /// Sets the message produced when the user closes the drawer.
    pub fn on_close(mut self, message: Message) -> Self {
        self.on_close = Some(message);
        self
    }

    /// Sets the maximum width of the inner content (`max-w-sm` in the
    /// shadcn/ui example).
    pub fn max_width(mut self, width: f32) -> Self {
        self.max_width = width;
        self
    }
}

impl<'a, Message: Clone + 'a> From<Drawer<'a, Message>> for Element<'a, Message> {
    fn from(drawer: Drawer<'a, Message>) -> Self {
        let handle = container(space().width(100).height(8)).style(|theme: &Theme| container::Style {
            background: Some(Background::Color(tokens(theme).palette.muted)),
            border: border::rounded(f32::MAX),
            ..container::Style::default()
        });

        let header = (drawer.title.is_some() || drawer.description.is_some()).then(|| {
            Column::new()
                .push(drawer.title.map(|title| text(title).size(style::text::BASE).font(font::SEMIBOLD)))
                .push(drawer.description.map(|description| {
                    text(description).size(style::text::SM).style(|theme: &Theme| text::Style {
                        color: Some(tokens(theme).palette.muted_foreground),
                    })
                }))
                .spacing(6)
                .padding(16)
                .align_x(Alignment::Center)
                .width(Length::Fill)
        });

        let body = Column::new()
            .push(header)
            .extend(drawer.content.into_iter().map(|element| container(element).padding([0, 16]).width(Length::Fill).into()))
            .push(drawer.footer.map(|footer| container(footer).padding(16).width(Length::Fill)))
            .max_width(drawer.max_width)
            .width(Length::Fill);

        let panel = container(
            Column::new()
                .push(container(handle).padding(iced::Padding::ZERO.top(16)))
                .push(body)
                .align_x(Alignment::Center)
                .width(Length::Fill),
        )
        .width(Length::Fill)
        .style(|theme: &Theme| {
            let t = tokens(theme);

            container::Style {
                background: Some(Background::Color(t.palette.background)),
                text_color: Some(t.palette.foreground),
                border: iced::Border {
                    color: t.palette.border,
                    width: 1.0,
                    radius: border::radius(0).top_left(t.radius_lg()).top_right(t.radius_lg()),
                },
                ..container::Style::default()
            }
        });

        Layer::new(drawer.base, panel, drawer.open)
            .align_y(iced::alignment::Vertical::Bottom)
            .padding(iced::Padding::ZERO.top(96))
            .on_dismiss_maybe(drawer.on_close)
            .into()
    }
}
