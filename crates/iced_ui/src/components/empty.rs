//! Use the Empty component to display an empty state.
//!
//! Port of <https://ui.shadcn.com/docs/components/empty>.
use iced_ui_core::icon::Icon;
use iced_ui_core::{Element, font, style, tokens};

use iced::widget::{Column, container, text};
use iced::{Alignment, Background, Length, Theme, border};

/// An empty state. See the [module documentation](self).
pub struct Empty<'a, Message> {
    icon: Option<Icon>,
    media: Option<Element<'a, Message>>,
    title: Option<String>,
    description: Option<String>,
    content: Option<Element<'a, Message>>,
    bordered: bool,
}

/// Creates an [`Empty`] state with a title.
pub fn empty<'a, Message: 'a>(title: impl Into<String>) -> Empty<'a, Message> {
    Empty {
        icon: None,
        media: None,
        title: Some(title.into()),
        description: None,
        content: None,
        bordered: false,
    }
}

impl<'a, Message: 'a> Empty<'a, Message> {
    /// Shows an icon in a muted square (`variant="icon"`).
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Shows custom media (e.g. an avatar) above the title.
    pub fn media(mut self, media: impl Into<Element<'a, Message>>) -> Self {
        self.media = Some(media.into());
        self
    }

    /// Sets the description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the content displayed below the header, usually actions.
    pub fn content(mut self, content: impl Into<Element<'a, Message>>) -> Self {
        self.content = Some(content.into());
        self
    }

    /// Draws a dashed-looking border around the empty state.
    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }
}

impl<'a, Message: 'a> From<Empty<'a, Message>> for Element<'a, Message> {
    fn from(empty: Empty<'a, Message>) -> Self {
        let icon = empty.icon.map(|icon| {
            container(icon.sized(24)).center(40).style(|theme: &Theme| {
                let t = tokens(theme);
                container::Style {
                    background: Some(Background::Color(t.palette.muted)),
                    text_color: Some(t.palette.foreground),
                    border: border::rounded(t.radius_lg()),
                    ..container::Style::default()
                }
            })
        });

        let header = Column::new()
            .push(icon)
            .push(empty.media)
            .push(empty.title.map(|title| {
                text(title).size(style::text::LG).font(font::MEDIUM)
            }))
            .push(empty.description.map(|description| {
                text(description)
                    .size(style::text::SM)
                    .line_height(1.625)
                    .align_x(iced::alignment::Horizontal::Center)
                    .style(|theme: &Theme| text::Style {
                        color: Some(tokens(theme).palette.muted_foreground),
                    })
            }))
            .spacing(8)
            .max_width(384)
            .align_x(Alignment::Center);

        let bordered = empty.bordered;

        container(
            Column::new()
                .push(header)
                .push(empty.content)
                .spacing(24)
                .align_x(Alignment::Center),
        )
        .padding(48)
        .center_x(Length::Fill)
        .style(move |theme: &Theme| {
            let t = tokens(theme);
            container::Style {
                border: if bordered {
                    style::border(t.palette.border, t.radius_lg())
                } else {
                    style::rounded(t.radius_lg())
                },
                ..container::Style::default()
            }
        })
        .into()
    }
}
