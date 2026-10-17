//! An image element with a fallback for representing the user.
//!
//! Port of <https://ui.shadcn.com/docs/components/avatar>.
//!
//! ```no_run
//! # use shadcn_iced::avatar::{avatar, avatar_group, avatar_group_count};
//! # fn view<'a>() -> iced::Element<'a, ()> {
//! avatar_group([
//!     avatar("CN"),
//!     avatar("LR"),
//!     avatar("ER"),
//!     avatar_group_count("+3"),
//! ])
//! # }
//! ```
use shadcn_iced_core::icon::Icon;
use shadcn_iced_core::primitive::focus::{self, focus_within};
use shadcn_iced_core::{Element, font, style, tokens};

use iced::widget::{Stack, container, image, space, text};
use iced::{Background, Color, Length, Padding, Theme, border};

/// The size of an [`Avatar`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Size {
    /// `size-6`
    Sm,
    /// `size-8`
    #[default]
    Default,
    /// `size-10`
    Lg,
}

impl Size {
    /// The diameter of the avatar.
    pub fn diameter(self) -> f32 {
        match self {
            Size::Sm => 24.0,
            Size::Default => 32.0,
            Size::Lg => 40.0,
        }
    }

    /// The diameter of an [`Avatar::badge`] (`size-2`, `size-2.5`,
    /// `size-3`).
    fn badge(self) -> f32 {
        match self {
            Size::Sm => 8.0,
            Size::Default => 10.0,
            Size::Lg => 12.0,
        }
    }
}

enum Fallback {
    Text(String),
    Icon(Icon),
}

/// An avatar. See the [module documentation](self).
pub struct Avatar {
    fallback: Fallback,
    image: Option<image::Handle>,
    size: Size,
    diameter: Option<f32>,
    ring: bool,
    count: bool,
    badge: Option<Option<Color>>,
    badge_icon: Option<Icon>,
}

/// Creates an [`Avatar`] showing the given fallback text (usually initials)
/// until an image is set and loaded.
pub fn avatar(fallback: impl Into<String>) -> Avatar {
    Avatar::new(Fallback::Text(fallback.into()))
}

/// Creates the counter displayed at the end of an [`avatar_group`]
/// (`AvatarGroupCount`), e.g. `+3`.
pub fn avatar_group_count(label: impl Into<String>) -> Avatar {
    Avatar {
        count: true,
        ..Avatar::new(Fallback::Text(label.into()))
    }
}

/// Creates the counter of an [`avatar_group`] showing an icon
/// (`AvatarGroupCount` with an icon).
pub fn avatar_group_count_icon(icon: Icon) -> Avatar {
    Avatar {
        count: true,
        ..Avatar::new(Fallback::Icon(icon))
    }
}

impl Avatar {
    fn new(fallback: Fallback) -> Self {
        Self {
            fallback,
            image: None,
            size: Size::Default,
            diameter: None,
            ring: false,
            count: false,
            badge: None,
            badge_icon: None,
        }
    }

    /// Sets the image.
    pub fn image(mut self, handle: impl Into<image::Handle>) -> Self {
        self.image = Some(handle.into());
        self
    }

    /// Sets the size.
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// Overrides the diameter, like `className="size-12"`.
    pub fn diameter(mut self, diameter: f32) -> Self {
        self.diameter = Some(diameter);
        self
    }

    /// Draws a ring of the background color around the avatar
    /// (`ring-2 ring-background`), as used in avatar groups. Like CSS rings,
    /// it is drawn outside of the avatar and takes no space.
    pub fn ring(mut self, ring: bool) -> Self {
        self.ring = ring;
        self
    }

    /// Shows a status dot in the bottom-right corner (`AvatarBadge`), of
    /// the given color or `bg-primary`.
    pub fn badge(mut self, color: Option<Color>) -> Self {
        self.badge = Some(color);
        self
    }

    /// Shows an [`Avatar::badge`] with an icon (`size-2`), hidden in small
    /// avatars.
    pub fn badge_icon(mut self, icon: Icon) -> Self {
        self.badge = Some(self.badge.flatten());
        self.badge_icon = Some(icon);
        self
    }

    fn outer_diameter(&self) -> f32 {
        self.diameter.unwrap_or(self.size.diameter())
    }
}

impl<'a, Message: 'a> From<Avatar> for Element<'a, Message> {
    fn from(avatar: Avatar) -> Self {
        let size = avatar.size;
        let diameter = avatar.diameter.unwrap_or(size.diameter());

        // `text-sm group-data-[size=sm]/avatar:text-xs`
        let fallback: Element<'a, Message> = match avatar.fallback {
            Fallback::Text(label) => text(label)
                .size(if size == Size::Sm && !avatar.count {
                    style::text::XS
                } else {
                    style::text::SM
                })
                .font(font::sans())
                .wrapping(text::Wrapping::None)
                .into(),
            // `[&>svg]:size-4`, `size-5` / `size-3` in large / small groups
            Fallback::Icon(icon) => icon
                .sized(match size {
                    Size::Sm => 12,
                    Size::Default => 16,
                    Size::Lg => 20,
                })
                .into(),
        };

        let mut stack =
            Stack::new().push(container(fallback).center(diameter).style(fallback_style));

        if let Some(handle) = avatar.image {
            stack = stack.push(
                image(handle)
                    .width(diameter)
                    .height(diameter)
                    .content_fit(iced::ContentFit::Cover)
                    .border_radius(diameter / 2.0),
            );
        }

        // `after:border after:border-border after:rounded-full`
        if !avatar.count {
            stack = stack.push(container(space()).width(diameter).height(diameter).style(
                |theme: &Theme| container::Style {
                    border: style::border(tokens(theme).palette.border, f32::MAX),
                    ..container::Style::default()
                },
            ));
        }

        if let Some(color) = avatar.badge {
            let badge = size.badge();
            let icon = avatar
                .badge_icon
                .filter(|_| size != Size::Sm)
                .map(|icon| icon.sized(8));

            // `absolute right-0 bottom-0 rounded-full ring-2 ring-background`
            stack = stack.push(
                container(ring(
                    container(icon).center(badge).style(move |theme: &Theme| {
                        let p = tokens(theme).palette;

                        container::Style {
                            background: Some(Background::Color(color.unwrap_or(p.primary))),
                            text_color: Some(p.primary_foreground),
                            border: border::rounded(f32::MAX),
                            ..container::Style::default()
                        }
                    }),
                    badge,
                ))
                .align_right(diameter)
                .align_bottom(diameter),
            );
        }

        let stack = stack
            .width(Length::Fixed(diameter))
            .height(Length::Fixed(diameter));

        if avatar.ring {
            ring(stack, diameter)
        } else {
            stack.into()
        }
    }
}

/// Draws a `ring-2 ring-background` outside of a circle of the given
/// diameter.
fn ring<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    diameter: f32,
) -> Element<'a, Message> {
    focus_within(content, move |theme: &Theme, _| focus::Style {
        ring: Some(focus::Ring {
            color: tokens(theme).palette.background,
            width: 2.0,
            radius: diameter / 2.0,
        }),
        ..focus::Style::default()
    })
    .into()
}

/// `bg-muted text-muted-foreground rounded-full`
fn fallback_style(theme: &Theme) -> container::Style {
    let p = tokens(theme).palette;

    container::Style {
        background: Some(Background::Color(p.muted)),
        text_color: Some(p.muted_foreground),
        border: border::rounded(f32::MAX),
        ..container::Style::default()
    }
}

/// Displays overlapping avatars (`flex -space-x-2`), each with a
/// `ring-2 ring-background`. End it with an [`avatar_group_count`].
pub fn avatar_group<'a, Message: 'a>(
    avatars: impl IntoIterator<Item = Avatar>,
) -> Element<'a, Message> {
    let mut stack = Stack::new();
    let mut offset = 0.0;
    let mut width: f32 = 0.0;
    let mut height: f32 = 0.0;

    for avatar in avatars {
        // Each avatar has a 2px ring and overlaps the previous one by 8px
        // (`-space-x-2`).
        let avatar = avatar.ring(true);
        let outer = avatar.outer_diameter();
        let element: Element<'a, Message> = avatar.into();

        stack = stack.push(container(element).padding(Padding::ZERO.left(offset)));
        width = width.max(offset + outer);
        height = height.max(outer);
        offset += outer - 8.0;
    }

    stack.width(width).height(height).into()
}
