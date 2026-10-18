//! Use the Empty component to display an empty state.
//!
//! Port of <https://ui.shadcn.com/docs/components/empty>.
//!
//! ```no_run
//! # use shadcn_iced::empty::empty;
//! # use shadcn_iced::prelude::lucide;
//! # fn view<'a>() -> iced::Element<'a, ()> {
//! empty("No Projects Yet")
//!     .icon(lucide::FOLDER_CODE)
//!     .description("You haven't created any projects yet.")
//!     .into()
//! # }
//! ```
use super::typography::balance;

use shadcn_iced_core::icon::Icon;
use shadcn_iced_core::primitive::dashed_border;
use shadcn_iced_core::theme::alpha;
use shadcn_iced_core::{Element, font, style, tokens};

use iced::widget::{Column, container, text};
use iced::{Alignment, Background, Length, Padding, Theme, border};

/// An empty state. See the [module documentation](self).
pub struct Empty<'a, Message> {
    icon: Option<Icon>,
    media: Option<Element<'a, Message>>,
    title: Option<String>,
    description: Option<String>,
    content: Option<Element<'a, Message>>,
    extra: Vec<Element<'a, Message>>,
    bordered: bool,
    muted: bool,
}

/// Creates an [`Empty`] state with a title.
pub fn empty<'a, Message: 'a>(title: impl Into<String>) -> Empty<'a, Message> {
    Empty {
        icon: None,
        media: None,
        title: Some(title.into()),
        description: None,
        content: None,
        extra: Vec::new(),
        bordered: false,
        muted: false,
    }
}

impl<'a, Message: 'a> Empty<'a, Message> {
    /// Shows an icon in a muted square (`EmptyMedia variant="icon"`).
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Shows custom media (e.g. an avatar) above the title
    /// (`EmptyMedia variant="default"`).
    pub fn media(mut self, media: impl Into<Element<'a, Message>>) -> Self {
        self.media = Some(media.into());
        self
    }

    /// Sets the description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the content displayed below the header, usually actions
    /// (`EmptyContent`).
    pub fn content(mut self, content: impl Into<Element<'a, Message>>) -> Self {
        self.content = Some(content.into());
        self
    }

    /// Appends an element below the content, e.g. a "Learn More" link.
    pub fn extra(mut self, element: impl Into<Element<'a, Message>>) -> Self {
        self.extra.push(element.into());
        self
    }

    /// Draws a dashed border around the empty state
    /// (`border border-dashed`).
    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }

    /// Fills the empty state with a muted background (`bg-muted/30`).
    pub fn muted(mut self, muted: bool) -> Self {
        self.muted = muted;
        self
    }
}

impl<'a, Message: 'a> From<Empty<'a, Message>> for Element<'a, Message> {
    fn from(empty: Empty<'a, Message>) -> Self {
        // `bg-muted text-foreground size-8 rounded-lg [&_svg]:size-4`
        let icon = empty.icon.map(|icon| {
            container(icon.sized(16)).center(32).style(|theme: &Theme| {
                let t = tokens(theme);
                container::Style {
                    background: Some(Background::Color(t.palette.muted)),
                    text_color: Some(t.palette.foreground),
                    border: border::rounded(t.radius_lg()),
                    ..container::Style::default()
                }
            })
        });

        // `mb-2`
        let media = icon
            .map(Element::from)
            .or(empty.media)
            .map(|media| container(media).padding(Padding::ZERO.bottom(8)));

        // `flex max-w-sm flex-col items-center gap-2`
        let header = Column::new()
            .push(media)
            .push(empty.title.map(|title| {
                // `text-sm font-medium tracking-tight text-balance`
                balance(
                    text(title)
                        .size(style::text::SM)
                        // `cn-font-heading`
                        .font(font::heading(iced::font::Weight::Medium))
                        .line_height(style::text::leading(style::text::SM))
                        .align_x(iced::alignment::Horizontal::Center),
                )
            }))
            .push(empty.description.map(|description| {
                // `text-sm/relaxed text-muted-foreground text-balance`
                balance(
                    text(description)
                        .size(style::text::SM)
                        .line_height(1.625)
                        .align_x(iced::alignment::Horizontal::Center)
                        .style(|theme: &Theme| text::Style {
                            color: Some(tokens(theme).palette.muted_foreground),
                        }),
                )
            }))
            .spacing(8)
            .max_width(384)
            .align_x(Alignment::Center);

        let bordered = empty.bordered;
        let muted = empty.muted;

        // `gap-4 p-6`, plus the 1px border.
        let content = container(
            Column::new()
                .push(header)
                .push(
                    empty
                        .content
                        .map(|content| container(content).max_width(384).center_x(Length::Shrink)),
                )
                .extend(empty.extra)
                .spacing(16)
                .align_x(Alignment::Center),
        )
        .padding(if bordered { 25 } else { 24 })
        .center_x(Length::Fill)
        .style(move |theme: &Theme| {
            let t = tokens(theme);
            container::Style {
                background: muted.then(|| Background::Color(alpha(t.palette.muted, 0.3))),
                border: style::rounded(t.radius_xl()),
                ..container::Style::default()
            }
        });

        if bordered {
            // `border border-dashed rounded-xl`
            dashed_border(content, |theme| {
                let t = tokens(theme);
                (t.palette.border, t.radius_xl())
            })
            .into()
        } else {
            content.into()
        }
    }
}
