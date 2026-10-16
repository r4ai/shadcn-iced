//! A two-state button that can be either on or off.
//!
//! Port of <https://ui.shadcn.com/docs/components/toggle>.
//!
//! ```no_run
//! # use iced_ui::toggle::toggle;
//! # use iced_ui_core::icon::lucide;
//! # #[derive(Clone)] enum Message { Bold(bool) }
//! # fn view<'a>(bold: bool) -> iced::Element<'a, Message> {
//! toggle(bold).icon(lucide::BOLD).on_toggle(Message::Bold).into()
//! # }
//! ```
use iced_ui_core::icon::Icon;
use iced_ui_core::primitive::Pressable;
use iced_ui_core::theme::alpha;
use iced_ui_core::{Element, font, style, tokens};

use iced::widget::button::{Status, Style};
use iced::widget::{Row, text};
use iced::{Alignment, Background, Border, Color, Length, Theme};

/// The visual style of a [`Toggle`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Variant {
    /// Transparent until hovered or on.
    #[default]
    Default,
    /// Bordered.
    Outline,
}

/// The size of a [`Toggle`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Size {
    /// `h-8 min-w-8 px-1.5`
    Sm,
    /// `h-9 min-w-9 px-2`
    #[default]
    Default,
    /// `h-10 min-w-10 px-2.5`
    Lg,
}

impl Size {
    /// The height of the toggle.
    pub fn height(self) -> f32 {
        match self {
            Size::Sm => 32.0,
            Size::Default => 36.0,
            Size::Lg => 40.0,
        }
    }

    fn padding(self) -> f32 {
        match self {
            Size::Sm => 6.0,
            Size::Default => 8.0,
            Size::Lg => 10.0,
        }
    }
}

pub use iced_ui_core::style::Position;

/// A toggle. See the [module documentation](self).
pub struct Toggle<'a, Message> {
    on: bool,
    icon: Option<Icon>,
    label: Option<String>,
    on_toggle: Option<Box<dyn Fn(bool) -> Message + 'a>>,
    variant: Variant,
    size: Size,
    position: Position,
    disabled: bool,
    width: Length,
}

/// Creates a [`Toggle`].
pub fn toggle<'a, Message: Clone + 'a>(on: bool) -> Toggle<'a, Message> {
    Toggle {
        on,
        icon: None,
        label: None,
        on_toggle: None,
        variant: Variant::Default,
        size: Size::Default,
        position: Position::Alone,
        disabled: false,
        width: Length::Shrink,
    }
}

impl<'a, Message: Clone + 'a> Toggle<'a, Message> {
    /// Sets the icon.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Sets the label.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the message produced when the toggle is pressed.
    pub fn on_toggle(mut self, f: impl Fn(bool) -> Message + 'a) -> Self {
        self.on_toggle = Some(Box::new(f));
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

    /// Sets the position in a joined group.
    pub fn position(mut self, position: Position) -> Self {
        self.position = position;
        self
    }

    /// Disables the toggle.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets the width.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
}

impl<'a, Message: Clone + 'a> From<Toggle<'a, Message>> for Element<'a, Message> {
    fn from(toggle: Toggle<'a, Message>) -> Self {
        let on = toggle.on;
        let size = toggle.size;
        let variant = toggle.variant;
        let position = toggle.position;
        let disabled = toggle.disabled || toggle.on_toggle.is_none();

        let icon_only = toggle.label.is_none();

        let content = Row::new()
            .push(toggle.icon.map(|icon| icon.sized(16)))
            .push(toggle.label.map(|label| {
                text(label)
                    .size(style::text::SM)
                    .font(font::MEDIUM)
                    .wrapping(text::Wrapping::None)
            }))
            .spacing(8)
            .align_y(Alignment::Center)
            .height(Length::Fill);

        // `min-w-*`: icon-only toggles are square.
        let width = match toggle.width {
            Length::Shrink if icon_only => Length::Fixed(size.height()),
            width => width,
        };

        let content: Element<'a, Message> = if width == Length::Shrink {
            content.into()
        } else {
            iced::widget::container(content)
                .center_x(Length::Fill)
                .height(Length::Fill)
                .into()
        };

        Pressable::new(content)
            .on_press_maybe(toggle.on_toggle.map(|f| f(!on)))
            .disabled(disabled)
            .width(width)
            .height(size.height())
            .padding([0.0, if icon_only { 0.0 } else { size.padding() }])
            .style(move |theme, status| self::style(theme, status, on, variant, position))
            .into()
    }
}

/// The style of a toggle.
pub fn style(
    theme: &Theme,
    status: Status,
    on: bool,
    variant: Variant,
    position: Position,
) -> Style {
    let t = tokens(theme);
    let p = &t.palette;
    let hovered = matches!(status, Status::Hovered | Status::Pressed);

    let (background, text_color) = if on {
        (Some(p.accent), p.accent_foreground)
    } else if hovered {
        match variant {
            Variant::Default => (Some(p.muted), p.muted_foreground),
            Variant::Outline => (Some(p.accent), p.accent_foreground),
        }
    } else {
        (None, p.foreground)
    };

    let mut style = Style {
        background: background.map(Background::Color),
        text_color,
        border: Border {
            color: match variant {
                Variant::Default => Color::TRANSPARENT,
                Variant::Outline => p.input,
            },
            width: match variant {
                Variant::Default => 0.0,
                Variant::Outline => 1.0,
            },
            radius: position.radius(t.radius_md()),
        },
        ..Style::default()
    };

    if status == Status::Disabled {
        style = super::button::disabled(style);
        style.text_color = alpha(p.foreground, style::DISABLED_OPACITY);
    }

    style
}
