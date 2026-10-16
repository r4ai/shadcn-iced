//! Displays a badge or a component that looks like a badge.
//!
//! Port of <https://ui.shadcn.com/docs/components/badge>.
use iced_ui_core::icon::Icon;
use iced_ui_core::theme::{alpha, tokens};
use iced_ui_core::{Element, font, style};

use iced::widget::{Row, container, text};
use iced::{Alignment, Background, Border, Color, Theme, border};

/// The visual style of a [`Badge`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Variant {
    /// Primary color.
    #[default]
    Default,
    /// Secondary color.
    Secondary,
    /// Destructive color.
    Destructive,
    /// Bordered.
    Outline,
    /// Transparent.
    Ghost,
    /// Looks like a link.
    Link,
}

/// A badge. See the [module documentation](self).
pub struct Badge<'a, Message> {
    content: Element<'a, Message>,
    icon: Option<Icon>,
    variant: Variant,
}

/// Creates a [`Badge`] with a text label.
pub fn badge<'a, Message: 'a>(label: impl text::IntoFragment<'a>) -> Badge<'a, Message> {
    Badge::new(text(label).size(style::text::XS).font(font::MEDIUM).line_height(iced::Pixels(16.0)))
}

impl<'a, Message: 'a> Badge<'a, Message> {
    /// Creates a [`Badge`] with arbitrary content.
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
            icon: None,
            variant: Variant::Default,
        }
    }

    /// Sets the variant.
    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }

    /// Adds a leading icon.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }
}

impl<'a, Message: 'a> From<Badge<'a, Message>> for Element<'a, Message> {
    fn from(badge: Badge<'a, Message>) -> Self {
        let variant = badge.variant;

        container(
            Row::new()
                .push(badge.icon.map(|icon| icon.sized(12)))
                .push(badge.content)
                .spacing(4)
                .align_y(Alignment::Center),
        )
        .padding([3, 9])
        .style(move |theme| self::style(theme, variant))
        .into()
    }
}

/// The style of a [`Badge`] of the given variant.
pub fn style(theme: &Theme, variant: Variant) -> container::Style {
    let t = tokens(theme);
    let p = &t.palette;

    let (background, text_color, border_color) = match variant {
        Variant::Default => (Some(p.primary), p.primary_foreground, Color::TRANSPARENT),
        Variant::Secondary => (Some(p.secondary), p.secondary_foreground, Color::TRANSPARENT),
        Variant::Destructive => (
            Some(if t.is_dark() { alpha(p.destructive, 0.6) } else { p.destructive }),
            Color::WHITE,
            Color::TRANSPARENT,
        ),
        Variant::Outline => (None, p.foreground, p.border),
        Variant::Ghost => (None, p.foreground, Color::TRANSPARENT),
        Variant::Link => (None, p.primary, Color::TRANSPARENT),
    };

    container::Style {
        background: background.map(Background::Color),
        text_color: Some(text_color),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: border::radius(f32::MAX),
        },
        ..container::Style::default()
    }
}
