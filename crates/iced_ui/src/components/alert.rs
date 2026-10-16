//! Displays a callout for user attention.
//!
//! Port of <https://ui.shadcn.com/docs/components/alert>.
use iced_ui_core::icon::Icon;
use iced_ui_core::theme::alpha;
use iced_ui_core::{Element, font, style, tokens};

use iced::widget::{Column, Row, container, text};
use iced::{Background, Length, Theme};

/// The visual style of an [`Alert`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Variant {
    /// The default alert.
    #[default]
    Default,
    /// An alert for errors.
    Destructive,
}

/// An alert. See the [module documentation](self).
pub struct Alert<'a, Message> {
    icon: Option<Icon>,
    title: Option<Element<'a, Message>>,
    description: Option<Description<'a, Message>>,
    variant: Variant,
}

enum Description<'a, Message> {
    Text(text::Fragment<'a>),
    Element(Element<'a, Message>),
}

/// Creates an [`Alert`] with a title.
pub fn alert<'a, Message: 'a>(title: impl text::IntoFragment<'a>) -> Alert<'a, Message> {
    Alert {
        icon: None,
        title: Some(
            text(title)
                .size(style::text::SM)
                .font(font::MEDIUM)
                .line_height(iced::Pixels(20.0))
                .into(),
        ),
        description: None,
        variant: Variant::Default,
    }
}

impl<'a, Message: 'a> Alert<'a, Message> {
    /// Sets the icon.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Sets the description.
    pub fn description(mut self, description: impl text::IntoFragment<'a>) -> Self {
        self.description = Some(Description::Text(description.into_fragment()));
        self
    }

    /// Sets a custom description element.
    pub fn description_element(mut self, description: impl Into<Element<'a, Message>>) -> Self {
        self.description = Some(Description::Element(description.into()));
        self
    }

    /// Sets the variant.
    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }
}

fn description_style(theme: &Theme, variant: Variant) -> text::Style {
    let p = tokens(theme).palette;

    text::Style {
        color: Some(match variant {
            Variant::Default => p.muted_foreground,
            Variant::Destructive => alpha(p.destructive, 0.9),
        }),
    }
}

impl<'a, Message: 'a> From<Alert<'a, Message>> for Element<'a, Message> {
    fn from(alert: Alert<'a, Message>) -> Self {
        let variant = alert.variant;

        let description = alert.description.map(|description| match description {
            Description::Text(fragment) => text(fragment)
                .size(style::text::SM)
                .line_height(1.625)
                .style(move |theme: &Theme| description_style(theme, variant))
                .into(),
            Description::Element(element) => element,
        });

        let body = Column::new()
            .push(alert.title)
            .push(description)
            .spacing(2)
            .width(Length::Fill);

        let content = Row::new()
            .push(
                alert
                    .icon
                    .map(|icon| container(icon.sized(16)).padding([2, 0])),
            )
            .push(body)
            .spacing(12);

        container(content)
            .padding([12, 16])
            .width(Length::Fill)
            .style(move |theme| self::style(theme, variant))
            .into()
    }
}

/// The style of an alert.
pub fn style(theme: &Theme, variant: Variant) -> container::Style {
    let t = tokens(theme);
    let p = &t.palette;

    container::Style {
        background: Some(Background::Color(p.card)),
        text_color: Some(match variant {
            Variant::Default => p.card_foreground,
            Variant::Destructive => p.destructive,
        }),
        border: style::border(p.border, t.radius_lg()),
        ..container::Style::default()
    }
}
