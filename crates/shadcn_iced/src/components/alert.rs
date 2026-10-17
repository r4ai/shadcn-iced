//! Displays a callout for user attention.
//!
//! Port of <https://ui.shadcn.com/docs/components/alert>.
use super::typography::balance;

use shadcn_iced_core::icon::Icon;
use shadcn_iced_core::theme::alpha;
use shadcn_iced_core::{Element, font, style, tokens};

use iced::widget::{Column, Row, Stack, container, text};
use iced::{Background, Length, Padding, Theme};

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
    action: Option<Element<'a, Message>>,
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
        // `text-balance` (`md:text-pretty` on wide screens)
        title: Some(
            balance(
                text(title)
                    .size(style::text::SM)
                    .font(font::medium())
                    .line_height(style::text::leading(style::text::SM)),
            )
            .into(),
        ),
        description: None,
        action: None,
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

    /// Sets an action displayed in the top-right corner (`AlertAction`,
    /// `absolute top-2 right-2`), e.g. an extra small button.
    pub fn action(mut self, action: impl Into<Element<'a, Message>>) -> Self {
        self.action = Some(action.into());
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
            // `text-balance` (`md:text-pretty` on wide screens)
            Description::Text(fragment) => balance(
                text(fragment)
                    .size(style::text::SM)
                    .line_height(style::text::leading(style::text::SM))
                    .style(move |theme: &Theme| description_style(theme, variant)),
            )
            .into(),
            Description::Element(element) => element,
        });

        // `gap-0.5`
        let body = Column::new()
            .push(alert.title)
            .push(description)
            .spacing(2)
            .width(Length::Fill);

        // `has-[>svg]:gap-x-2 *:[svg]:translate-y-0.5 *:[svg]:size-4`
        let content = Row::new()
            .push(
                alert
                    .icon
                    .map(|icon| container(icon.sized(16)).padding([2, 0])),
            )
            .push(body)
            .spacing(8);

        // `px-2.5 py-2`, `has-data-[slot=alert-action]:pr-18`, plus the
        // 1px border.
        let padding = Padding {
            right: if alert.action.is_some() { 73.0 } else { 11.0 },
            ..Padding::from([9.0, 11.0])
        };

        let alert_box = container(content)
            .padding(padding)
            .width(Length::Fill)
            .style(move |theme| self::style(theme, variant));

        match alert.action {
            Some(action) => Stack::new()
                .push(alert_box)
                .push(
                    container(action)
                        .align_right(Length::Fill)
                        .padding(Padding::ZERO.top(9).right(9)),
                )
                .width(Length::Fill)
                .into(),
            None => alert_box.into(),
        }
    }
}

/// The style of an alert (`rounded-lg border bg-card`).
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
