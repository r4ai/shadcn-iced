//! Displays a button or a component that looks like a button.
//!
//! Port of <https://ui.shadcn.com/docs/components/button>.
//!
//! ```no_run
//! use shadcn_iced_core::icon::lucide;
//! # use shadcn_iced::button::{self, button};
//! # #[derive(Clone)] enum Message { Save }
//! # fn view<'a>() -> iced::Element<'a, Message> {
//! button("Save")
//!     .variant(button::Variant::Outline)
//!     .icon(lucide::SAVE)
//!     .on_press(Message::Save)
//!     .into()
//! # }
//! ```
use shadcn_iced_core::icon::Icon;
use shadcn_iced_core::primitive::{Pressable, underline};
use shadcn_iced_core::theme::{alpha, mix, tokens};
use shadcn_iced_core::{Element, font, style};

use iced::widget::button::{Status, Style};
use iced::widget::{Row, text};
use iced::{Alignment, Background, Length, Padding, Theme};

/// The visual style of a [`Button`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Variant {
    /// A solid button using the primary color.
    #[default]
    Default,
    /// A solid button for destructive actions.
    Destructive,
    /// A bordered button.
    Outline,
    /// A solid button using the secondary color.
    Secondary,
    /// A transparent button that is highlighted on hover.
    Ghost,
    /// A button that looks like a link.
    Link,
}

/// The size of a [`Button`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Size {
    /// `h-8 px-2.5`
    #[default]
    Default,
    /// `h-6 px-2 text-xs`
    Xs,
    /// `h-7 px-2.5 text-[0.8rem]`
    Sm,
    /// `h-9 px-2.5`
    Lg,
    /// A 32x32 square, for icon-only buttons.
    Icon,
    /// A 24x24 square, for icon-only buttons.
    IconXs,
    /// A 28x28 square, for icon-only buttons.
    IconSm,
    /// A 36x36 square, for icon-only buttons.
    IconLg,
}

impl Size {
    /// The height of the button.
    pub fn height(self) -> f32 {
        match self {
            Size::Xs | Size::IconXs => 24.0,
            Size::Sm | Size::IconSm => 28.0,
            Size::Default | Size::Icon => 32.0,
            Size::Lg | Size::IconLg => 36.0,
        }
    }

    /// Whether the size is a square meant for icon-only buttons.
    pub fn is_icon(self) -> bool {
        matches!(
            self,
            Size::Icon | Size::IconXs | Size::IconSm | Size::IconLg
        )
    }

    /// The horizontal padding, reduced on the side of an icon
    /// (`has-data-[icon=inline-start]:pl-2`).
    fn padding(self, leading: bool, trailing: bool) -> Padding {
        let (base, icon_side) = match self {
            Size::Xs => (8.0, 6.0),
            Size::Sm => (10.0, 6.0),
            Size::Default | Size::Lg => (10.0, 8.0),
            _ => (0.0, 0.0),
        };

        // The padding sits inside a 1px transparent border (`border
        // border-transparent`).
        Padding {
            left: 1.0 + if leading { icon_side } else { base },
            right: 1.0 + if trailing { icon_side } else { base },
            ..Padding::ZERO
        }
    }

    fn gap(self) -> f32 {
        match self {
            Size::Xs | Size::Sm => 4.0,
            _ => 6.0,
        }
    }

    /// The font size of the label.
    pub fn text_size(self) -> f32 {
        match self {
            Size::Xs | Size::IconXs => style::text::XS,
            Size::Sm | Size::IconSm => 12.8,
            _ => style::text::SM,
        }
    }

    /// The size of the icons.
    pub fn icon_size(self) -> f32 {
        match self {
            Size::Xs | Size::IconXs => 12.0,
            Size::Sm => 14.0,
            _ => 16.0,
        }
    }

    /// The corner radius: `rounded-lg`, smaller for the small sizes
    /// (`rounded-[min(var(--radius-md),10px)]`).
    pub fn radius(self, tokens: &shadcn_iced_core::Tokens) -> f32 {
        match self {
            Size::Xs | Size::IconXs => tokens.radius_md().min(10.0),
            Size::Sm | Size::IconSm => tokens.radius_md().min(12.0),
            _ => tokens.radius_lg(),
        }
    }
}

enum Content<'a, Message> {
    Label(String),
    Icon(Icon),
    Custom(Element<'a, Message>),
}

/// A button. See the [module documentation](self).
pub struct Button<'a, Message> {
    content: Content<'a, Message>,
    leading: Option<Icon>,
    trailing: Option<Icon>,
    on_press: Option<Message>,
    variant: Variant,
    size: Size,
    disabled: bool,
    width: Option<Length>,
    position: style::Position,
    vertical: bool,
    rounded: bool,
}

/// Creates a [`Button`] with a text label.
pub fn button<'a, Message: Clone + 'a>(label: impl Into<String>) -> Button<'a, Message> {
    Button::from_content(Content::Label(label.into()))
}

/// Creates an icon-only [`Button`] (`size="icon"`).
pub fn icon_button<'a, Message: Clone + 'a>(icon: Icon) -> Button<'a, Message> {
    Button::from_content(Content::Icon(icon)).size(Size::Icon)
}

impl<'a, Message: Clone + 'a> Button<'a, Message> {
    fn from_content(content: Content<'a, Message>) -> Self {
        Self {
            content,
            leading: None,
            trailing: None,
            on_press: None,
            variant: Variant::Default,
            size: Size::Default,
            disabled: false,
            width: None,
            position: style::Position::Alone,
            vertical: false,
            rounded: false,
        }
    }

    /// Creates a [`Button`] with arbitrary content.
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self::from_content(Content::Custom(content.into()))
    }

    /// Sets the message produced when the button is pressed.
    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    /// Sets the message produced when the button is pressed, if any.
    pub fn on_press_maybe(mut self, message: Option<Message>) -> Self {
        self.on_press = message;
        self
    }

    /// Sets the variant.
    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets the size.
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// Adds an icon before the label.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.leading = Some(icon);
        self
    }

    /// Adds an icon after the label.
    pub fn trailing_icon(mut self, icon: Icon) -> Self {
        self.trailing = Some(icon);
        self
    }

    /// Disables the button.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets the position of the button in a joined group, which flattens
    /// the inner corners. See [`button_group`](super::button_group).
    pub fn position(mut self, position: style::Position, vertical: bool) -> Self {
        self.position = position;
        self.vertical = vertical;
        self
    }

    /// Makes the button fully rounded (`rounded-full`).
    pub fn rounded(mut self, rounded: bool) -> Self {
        self.rounded = rounded;
        self
    }

    /// Sets the width. Buttons shrink to fit their content by default.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = Some(width.into());
        self
    }
}

impl<'a, Message: Clone + 'a> From<Button<'a, Message>> for Element<'a, Message> {
    fn from(button: Button<'a, Message>) -> Self {
        let size = button.size;
        let height = size.height();
        let icon_size = size.icon_size();
        let padding = size.padding(button.leading.is_some(), button.trailing.is_some());

        let label = match button.content {
            Content::Label(label) => text(label)
                .size(size.text_size())
                .font(font::medium())
                .wrapping(text::Wrapping::None)
                .into(),
            Content::Icon(icon) => icon.sized(icon_size).into(),
            Content::Custom(element) => element,
        };

        let content = Row::new()
            .push(button.leading.map(|icon| icon.sized(icon_size)))
            .push(label)
            .push(button.trailing.map(|icon| icon.sized(icon_size)))
            .spacing(size.gap())
            .align_y(Alignment::Center)
            .height(Length::Fill);

        // In a group, the buttons after the first one lose their leading
        // border (`border-l-0` / `border-t-0`) and overlap the previous one
        // by a pixel instead. Their fixed sizes (`h-8`, `size-8`) include the
        // missing border: they grow by a pixel to keep it.
        let joined = matches!(
            button.position,
            style::Position::Middle | style::Position::Last
        );
        let grow = |vertical: bool| {
            if joined && button.vertical == vertical {
                1.0
            } else {
                0.0
            }
        };

        let width = button.width.unwrap_or(if size.is_icon() {
            Length::Fixed(height + grow(false))
        } else {
            Length::Shrink
        });
        let height = height + grow(true);

        let content: Element<'a, Message> = if size.is_icon() || width != Length::Shrink {
            iced::widget::container(content)
                .center_x(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            content.into()
        };

        // `hover:underline underline-offset-4`
        let content = if button.variant == Variant::Link && !button.disabled {
            underline(content)
                .text_size(size.text_size())
                .hover_area(Padding {
                    left: padding.left,
                    right: padding.right,
                    ..Padding::ZERO
                })
                .into()
        } else {
            content
        };

        let variant = button.variant;
        let position = button.position;
        let vertical = button.vertical;
        let rounded = button.rounded;

        Pressable::new(content)
            .on_press_maybe(button.on_press)
            .disabled(button.disabled)
            .width(width)
            .height(height)
            .padding(padding)
            .press_offset(1.0)
            // `border border-transparent bg-clip-padding`
            .clip_background(true)
            .style(move |theme, status| {
                let mut style = self::style(theme, status, variant);
                let radius = if rounded {
                    f32::MAX
                } else if position != style::Position::Alone {
                    // `in-data-[slot=button-group]:rounded-lg`
                    tokens(theme).radius_lg()
                } else {
                    size.radius(&tokens(theme))
                };

                style.border.radius = if vertical {
                    position.vertical_radius(radius)
                } else {
                    position.radius(radius)
                };

                style
            })
            .into()
    }
}

/// The style of a [`Button`] of the given variant.
///
/// Usable with plain iced buttons too:
/// `iced::widget::button("Hi").style(|t, s| shadcn_iced::button::style(t, s, Variant::Outline))`.
pub fn style(theme: &Theme, status: Status, variant: Variant) -> Style {
    let t = tokens(theme);
    let p = &t.palette;
    let dark = t.is_dark();
    let hovered = matches!(status, Status::Hovered | Status::Pressed);
    let radius = t.radius_lg();

    let mut style = match variant {
        Variant::Default => Style {
            background: Some(Background::Color(if hovered {
                alpha(p.primary, 0.8)
            } else {
                p.primary
            })),
            text_color: p.primary_foreground,
            border: transparent_border(radius),
            ..Style::default()
        },
        Variant::Destructive => Style {
            background: Some(Background::Color(alpha(
                p.destructive,
                match (dark, hovered) {
                    (false, false) => 0.1,
                    (false, true) | (true, false) => 0.2,
                    (true, true) => 0.3,
                },
            ))),
            text_color: p.destructive,
            border: transparent_border(radius),
            ..Style::default()
        },
        Variant::Outline => Style {
            background: Some(Background::Color(match (dark, hovered) {
                (false, false) => p.background,
                (false, true) => p.muted,
                (true, false) => alpha(p.input, 0.3),
                (true, true) => alpha(p.input, 0.5),
            })),
            text_color: p.foreground,
            border: style::border(if dark { p.input } else { p.border }, radius),
            ..Style::default()
        },
        Variant::Secondary => Style {
            background: Some(Background::Color(if hovered {
                // `color-mix(in oklch, var(--secondary), var(--foreground) 5%)`
                mix(p.secondary, p.foreground, 0.05)
            } else {
                p.secondary
            })),
            text_color: p.secondary_foreground,
            border: transparent_border(radius),
            ..Style::default()
        },
        Variant::Ghost => Style {
            background: hovered
                .then(|| Background::Color(if dark { alpha(p.muted, 0.5) } else { p.muted })),
            text_color: p.foreground,
            border: transparent_border(radius),
            ..Style::default()
        },
        Variant::Link => Style {
            background: None,
            text_color: p.primary,
            border: transparent_border(radius),
            ..Style::default()
        },
    };

    if status == Status::Disabled {
        style = disabled_over(style, p.background);
    }

    style
}

/// `border border-transparent`: invisible, but it insets the background
/// (`bg-clip-padding`).
fn transparent_border(radius: f32) -> iced::Border {
    iced::Border {
        width: 1.0,
        ..style::border(iced::Color::TRANSPARENT, radius)
    }
}

/// Applies `opacity-50` to a style, for a button drawn over `background`.
///
/// CSS applies the opacity to the button as a whole, so its label blends
/// with the page rather than with its own translucent background: the label
/// stays opaque, mixed with `background` instead.
pub fn disabled_over(style: Style, background: iced::Color) -> Style {
    let opacity = style::DISABLED_OPACITY;
    let text = style.text_color;

    Style {
        text_color: if text.a >= 1.0 {
            mix(background, text, opacity)
        } else {
            alpha(text, opacity)
        },
        ..disabled(style)
    }
}

/// Applies `opacity-50` to a style.
pub fn disabled(style: Style) -> Style {
    let opacity = style::DISABLED_OPACITY;

    Style {
        background: style
            .background
            .map(|background| background.scale_alpha(opacity)),
        text_color: alpha(style.text_color, opacity),
        border: iced::Border {
            color: alpha(style.border.color, opacity),
            ..style.border
        },
        shadow: iced::Shadow {
            color: alpha(style.shadow.color, opacity),
            ..style.shadow
        },
        ..style
    }
}
