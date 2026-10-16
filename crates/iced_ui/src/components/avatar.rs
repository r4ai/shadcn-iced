//! An image element with a fallback for representing the user.
//!
//! Port of <https://ui.shadcn.com/docs/components/avatar>.
use iced_ui_core::{Element, font, style, tokens};

use iced::widget::{Stack, container, image, text};
use iced::{Background, Length, Theme, border};

/// The size of an [`Avatar`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Size {
    /// 24x24
    Sm,
    /// 32x32
    #[default]
    Default,
    /// 40x40
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
}

/// An avatar. See the [module documentation](self).
pub struct Avatar {
    fallback: String,
    image: Option<image::Handle>,
    size: Size,
    ring: bool,
}

/// Creates an [`Avatar`] showing the given fallback text (usually initials)
/// until an image is set and loaded.
pub fn avatar(fallback: impl Into<String>) -> Avatar {
    Avatar {
        fallback: fallback.into(),
        image: None,
        size: Size::Default,
        ring: false,
    }
}

impl Avatar {
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

    /// Draws a ring of the background color around the avatar
    /// (`ring-2 ring-background`), as used in avatar groups.
    pub fn ring(mut self, ring: bool) -> Self {
        self.ring = ring;
        self
    }
}

impl<'a, Message: 'a> From<Avatar> for Element<'a, Message> {
    fn from(avatar: Avatar) -> Self {
        let diameter = avatar.size.diameter();

        let fallback = container(
            text(avatar.fallback)
                .size(if avatar.size == Size::Sm { style::text::XS } else { style::text::SM })
                .font(font::SANS),
        )
        .center(diameter)
        .style(fallback_style);

        let mut stack = Stack::new().push(fallback);

        if let Some(handle) = avatar.image {
            stack = stack.push(
                image(handle)
                    .width(diameter)
                    .height(diameter)
                    .content_fit(iced::ContentFit::Cover)
                    .border_radius(diameter / 2.0),
            );
        }

        if avatar.ring {
            container(stack)
                .padding(2)
                .style(|theme: &Theme| container::Style {
                    background: Some(Background::Color(tokens(theme).palette.background)),
                    border: border::rounded(f32::MAX),
                    ..container::Style::default()
                })
                .into()
        } else {
            stack.width(Length::Fixed(diameter)).height(Length::Fixed(diameter)).into()
        }
    }
}

fn fallback_style(theme: &Theme) -> container::Style {
    let p = tokens(theme).palette;

    container::Style {
        background: Some(Background::Color(p.muted)),
        text_color: Some(p.muted_foreground),
        border: border::rounded(f32::MAX),
        ..container::Style::default()
    }
}

/// Displays overlapping avatars (`-space-x-2`).
pub fn avatar_group<'a, Message: 'a>(avatars: impl IntoIterator<Item = Avatar>) -> Element<'a, Message> {
    let mut stack = Stack::new();
    let mut offset = 0.0;
    let mut width: f32 = 0.0;
    let mut height: f32 = 0.0;

    for avatar in avatars {
        // Each avatar is wrapped in a 2px ring and overlaps the previous one
        // by 8px (`-space-x-2`).
        let outer = avatar.size.diameter() + 4.0;
        let element: Element<'a, Message> = avatar.ring(true).into();

        stack = stack.push(container(element).padding(iced::Padding::ZERO.left(offset)));
        width = width.max(offset + outer);
        height = height.max(outer);
        offset += outer - 8.0 - 4.0;
    }

    stack.width(width).height(height).into()
}
