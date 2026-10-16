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
use iced::{Alignment, Background, Border, Color, Length, Padding, Theme};

/// The visual style of a [`Toggle`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Variant {
    /// Transparent until hovered or on (`bg-transparent`).
    #[default]
    Default,
    /// Bordered (`border-input border bg-transparent`).
    Outline,
}

/// The size of a [`Toggle`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Size {
    /// `h-7 min-w-7 px-2.5 text-[0.8rem]`
    Sm,
    /// `h-8 min-w-8 px-2.5`
    #[default]
    Default,
    /// `h-9 min-w-9 px-2.5`
    Lg,
}

impl Size {
    /// The height of the toggle.
    pub fn height(self) -> f32 {
        match self {
            Size::Sm => 28.0,
            Size::Default => 32.0,
            Size::Lg => 36.0,
        }
    }

    /// The horizontal padding: `px-2.5`.
    fn padding(self) -> f32 {
        10.0
    }

    /// The font size: `text-sm`, `text-[0.8rem]` when small.
    pub fn text_size(self) -> f32 {
        match self {
            Size::Sm => 12.8,
            _ => style::text::SM,
        }
    }

    /// The icon size: `size-4`, `size-3.5` when small.
    pub fn icon_size(self) -> f32 {
        match self {
            Size::Sm => 14.0,
            _ => 16.0,
        }
    }

    /// The corner radius: `rounded-lg`, or
    /// `rounded-[min(var(--radius-md),12px)]` when small.
    pub fn radius(self, tokens: &iced_ui_core::Tokens) -> f32 {
        match self {
            Size::Sm => tokens.radius_md().min(12.0),
            _ => tokens.radius_lg(),
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
    vertical: bool,
    padding: Option<f32>,
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
        vertical: false,
        padding: None,
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

    /// Sets the position in a joined horizontal group.
    pub fn position(mut self, position: Position) -> Self {
        self.position = position;
        self.vertical = false;
        self
    }

    /// Sets the position in a joined vertical group.
    pub fn vertical_position(mut self, position: Position) -> Self {
        self.position = position;
        self.vertical = true;
        self
    }

    /// Overrides the horizontal padding, e.g. `px-2` in joined groups.
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = Some(padding);
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
        let vertical = toggle.vertical;
        let disabled = toggle.disabled || toggle.on_toggle.is_none();

        // `gap-1`
        let content = Row::new()
            .push(toggle.icon.map(|icon| icon.sized(size.icon_size())))
            .push(toggle.label.map(|label| {
                text(label)
                    .size(size.text_size())
                    .font(font::MEDIUM)
                    .wrapping(text::Wrapping::None)
            }))
            .spacing(4)
            .align_y(Alignment::Center)
            .height(Length::Fill);

        let padding = Padding {
            left: toggle.padding.unwrap_or(size.padding()),
            right: toggle.padding.unwrap_or(size.padding()),
            ..Padding::ZERO
        };

        // Icon-only toggles are at least square (`min-w-*`).
        let content: Element<'a, Message> = if toggle.width == Length::Shrink {
            iced::widget::container(content)
                .center_x(Length::Shrink)
                .height(Length::Fill)
                .into()
        } else {
            iced::widget::container(content)
                .center_x(Length::Fill)
                .height(Length::Fill)
                .into()
        };

        Pressable::new(content)
            .on_press_maybe(toggle.on_toggle.map(|f| f(!on)))
            .disabled(disabled)
            .width(toggle.width)
            .height(size.height())
            .padding(padding)
            .style(move |theme, status| {
                let mut style = self::style(theme, status, on, variant, position);
                let t = tokens(theme);
                // Joined group items: `first:rounded-l-lg last:rounded-r-lg`
                let radius = if position == Position::Alone {
                    size.radius(&t)
                } else {
                    t.radius_lg()
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

    // `hover:bg-muted hover:text-foreground aria-pressed:bg-muted`
    let mut style = Style {
        background: (on || hovered).then_some(Background::Color(p.muted)),
        text_color: p.foreground,
        border: Border {
            color: match variant {
                Variant::Default => Color::TRANSPARENT,
                Variant::Outline => p.input,
            },
            width: match variant {
                Variant::Default => 0.0,
                Variant::Outline => 1.0,
            },
            radius: position.radius(t.radius_lg()),
        },
        ..Style::default()
    };

    // `disabled:opacity-50`
    if status == Status::Disabled {
        style.background = style
            .background
            .map(|_| Background::Color(alpha(p.muted, style::DISABLED_OPACITY)));
        style.text_color = alpha(p.foreground, style::DISABLED_OPACITY);
        style.border.color = alpha(style.border.color, style::DISABLED_OPACITY);
    }

    style
}
