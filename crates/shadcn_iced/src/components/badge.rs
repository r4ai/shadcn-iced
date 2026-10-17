//! Displays a badge or a component that looks like a badge.
//!
//! Port of <https://ui.shadcn.com/docs/components/badge>.
//!
//! ```no_run
//! # use shadcn_iced::badge::{badge, Variant};
//! # use shadcn_iced::prelude::lucide;
//! # fn view<'a>() -> iced::Element<'a, ()> {
//! badge("Verified")
//!     .icon(lucide::BADGE_CHECK)
//!     .variant(Variant::Secondary)
//!     .into()
//! # }
//! ```
use shadcn_iced_core::icon::Icon;
use shadcn_iced_core::primitive::Pressable;
use shadcn_iced_core::theme::{alpha, tokens};
use shadcn_iced_core::{Element, font, style};

use iced::widget::button;
use iced::widget::{Row, container, text};
use iced::{Alignment, Background, Border, Color, Padding, Theme, border, mouse};

/// The visual style of a [`Badge`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Variant {
    /// `bg-primary text-primary-foreground`
    #[default]
    Default,
    /// `bg-secondary text-secondary-foreground`
    Secondary,
    /// `bg-destructive/10 text-destructive` (`/20` in dark mode)
    Destructive,
    /// `border-border text-foreground`
    Outline,
    /// Transparent, `hover:bg-muted hover:text-muted-foreground`.
    Ghost,
    /// `text-primary underline-offset-4`
    Link,
}

/// The height of a badge (`h-5`).
pub const HEIGHT: f32 = 20.0;

/// A badge. See the [module documentation](self).
pub struct Badge<'a, Message> {
    content: Element<'a, Message>,
    leading: Option<Element<'a, Message>>,
    trailing: Option<Element<'a, Message>>,
    variant: Variant,
    on_press: Option<Message>,
}

/// Creates a [`Badge`] with a text label (`text-xs font-medium`).
pub fn badge<'a, Message: 'a>(label: impl text::IntoFragment<'a>) -> Badge<'a, Message> {
    Badge::new(
        text(label)
            .size(style::text::XS)
            .font(font::medium())
            .line_height(iced::Pixels(16.0))
            .wrapping(text::Wrapping::None),
    )
}

impl<'a, Message: 'a> Badge<'a, Message> {
    /// Creates a [`Badge`] with arbitrary content.
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
            leading: None,
            trailing: None,
            variant: Variant::Default,
            on_press: None,
        }
    }

    /// Sets the variant.
    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }

    /// Adds a leading icon (`data-icon="inline-start"`, `size-3`).
    pub fn icon(self, icon: Icon) -> Self {
        self.leading(icon.sized(12))
    }

    /// Adds a trailing icon (`data-icon="inline-end"`, `size-3`).
    pub fn trailing_icon(self, icon: Icon) -> Self {
        self.trailing(icon.sized(12))
    }

    /// Adds a leading element, e.g. a 12px [`spinner`](super::spinner).
    pub fn leading(mut self, element: impl Into<Element<'a, Message>>) -> Self {
        self.leading = Some(element.into());
        self
    }

    /// Adds a trailing element, e.g. a 12px [`spinner`](super::spinner).
    pub fn trailing(mut self, element: impl Into<Element<'a, Message>>) -> Self {
        self.trailing = Some(element.into());
        self
    }

    /// Makes the badge a link (`render={<a />}`), with a hover style.
    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }
}

impl<'a, Message: Clone + 'a> From<Badge<'a, Message>> for Element<'a, Message> {
    fn from(badge: Badge<'a, Message>) -> Self {
        let variant = badge.variant;

        // `px-2`, `pl-1.5` / `pr-1.5` next to an icon, plus the 1px border.
        let padding = Padding {
            left: if badge.leading.is_some() { 7.0 } else { 9.0 },
            right: if badge.trailing.is_some() { 7.0 } else { 9.0 },
            ..Padding::ZERO
        };

        let content = Row::new()
            .push(badge.leading)
            .push(badge.content)
            .push(badge.trailing)
            .spacing(4)
            .align_y(Alignment::Center);

        // Links (`[a]:hover:`) and ghost badges (`hover:bg-muted`) react to
        // the pointer.
        let link = badge.on_press.is_some();

        match badge.on_press {
            _ if link || variant == Variant::Ghost => {
                Pressable::new(container(content).center_y(iced::Length::Fill))
                    .on_press_maybe(badge.on_press)
                    .interaction(if link {
                        mouse::Interaction::Pointer
                    } else {
                        mouse::Interaction::Idle
                    })
                    .height(HEIGHT)
                    .padding(padding)
                    .style(move |theme, status| {
                        let hovered =
                            matches!(status, button::Status::Hovered | button::Status::Pressed);
                        let style = self::style_with(theme, variant, hovered);

                        button::Style {
                            background: style.background,
                            text_color: style.text_color.unwrap_or(Color::BLACK),
                            border: style.border,
                            ..button::Style::default()
                        }
                    })
                    .into()
            }
            _ => container(content)
                .height(HEIGHT)
                .center_y(HEIGHT)
                .padding(padding)
                .style(move |theme| self::style(theme, variant))
                .into(),
        }
    }
}

/// The style of a [`Badge`] of the given variant.
pub fn style(theme: &Theme, variant: Variant) -> container::Style {
    style_with(theme, variant, false)
}

/// The style of a [`Badge`] of the given variant, hovered or not. Only
/// links (`[a]:hover:`) and ghost badges (`hover:`) have a hover style.
pub fn style_with(theme: &Theme, variant: Variant, hovered: bool) -> container::Style {
    let t = tokens(theme);
    let p = &t.palette;
    let dark = t.is_dark();

    let (background, text_color, border_color) = match variant {
        Variant::Default => (
            Some(if hovered {
                alpha(p.primary, 0.8)
            } else {
                p.primary
            }),
            p.primary_foreground,
            Color::TRANSPARENT,
        ),
        Variant::Secondary => (
            Some(if hovered {
                alpha(p.secondary, 0.8)
            } else {
                p.secondary
            }),
            p.secondary_foreground,
            Color::TRANSPARENT,
        ),
        Variant::Destructive => (
            Some(alpha(
                p.destructive,
                match (dark, hovered) {
                    (false, false) => 0.1,
                    (false, true) | (true, false) => 0.2,
                    (true, true) => 0.3,
                },
            )),
            p.destructive,
            Color::TRANSPARENT,
        ),
        Variant::Outline => (
            hovered.then_some(p.muted),
            if hovered {
                p.muted_foreground
            } else {
                p.foreground
            },
            p.border,
        ),
        Variant::Ghost => (
            hovered.then(|| if dark { alpha(p.muted, 0.5) } else { p.muted }),
            if hovered {
                p.muted_foreground
            } else {
                p.foreground
            },
            Color::TRANSPARENT,
        ),
        Variant::Link => (None, p.primary, Color::TRANSPARENT),
    };

    container::Style {
        background: background.map(Background::Color),
        text_color: Some(text_color),
        // `rounded-4xl`, a pill at this height.
        border: Border {
            color: border_color,
            width: 1.0,
            radius: border::radius(t.radius_4xl()),
        },
        ..container::Style::default()
    }
}
