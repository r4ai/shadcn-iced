//! A control that allows the user to toggle between checked and not checked.
//!
//! Port of <https://ui.shadcn.com/docs/components/switch>.
//!
//! ```no_run
//! # use iced_ui::switch::switch;
//! # #[derive(Clone)] enum Message { Airplane(bool) }
//! # fn view<'a>(on: bool) -> iced::Element<'a, Message> {
//! switch(on).label("Airplane Mode").on_toggle(Message::Airplane).into()
//! # }
//! ```
use iced_ui_core::primitive::Pressable;
use iced_ui_core::theme::alpha;
use iced_ui_core::{Element, font, style, tokens};

use iced::widget::{Row, container, space, text};
use iced::{Alignment, Background, Color, Length, Theme, border, mouse};

/// The size of a [`Switch`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Size {
    /// 24x14
    Sm,
    /// 32x18
    #[default]
    Default,
}

/// A switch. See the [module documentation](self).
pub struct Switch<'a, Message> {
    checked: bool,
    label: Option<String>,
    on_toggle: Option<Box<dyn Fn(bool) -> Message + 'a>>,
    disabled: bool,
    size: Size,
}

/// Creates a [`Switch`].
pub fn switch<'a, Message: Clone + 'a>(checked: bool) -> Switch<'a, Message> {
    Switch {
        checked,
        label: None,
        on_toggle: None,
        disabled: false,
        size: Size::Default,
    }
}

impl<'a, Message: Clone + 'a> Switch<'a, Message> {
    /// Sets the label, displayed on the right.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the message produced when the switch is toggled.
    pub fn on_toggle(mut self, f: impl Fn(bool) -> Message + 'a) -> Self {
        self.on_toggle = Some(Box::new(f));
        self
    }

    /// Disables the switch.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets the size.
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }
}

impl<'a, Message: Clone + 'a> From<Switch<'a, Message>> for Element<'a, Message> {
    fn from(switch: Switch<'a, Message>) -> Self {
        let checked = switch.checked;
        let disabled = switch.disabled || switch.on_toggle.is_none();

        let (width, height, thumb) = match switch.size {
            Size::Default => (32.0, 18.0, 16.0),
            Size::Sm => (24.0, 14.0, 12.0),
        };

        let thumb = container(space().width(thumb).height(thumb))
            .style(move |theme| self::thumb(theme, checked, disabled));

        let track = container(thumb)
            .width(width)
            .height(height)
            .padding(1)
            .align_x(if checked {
                Alignment::End
            } else {
                Alignment::Start
            })
            .align_y(Alignment::Center)
            .style(move |theme| self::track(theme, checked, disabled));

        let label = switch.label.map(|label| {
            text(label)
                .size(style::text::SM)
                .font(font::MEDIUM)
                .line_height(iced::Pixels(14.0))
                .style(move |theme: &Theme| text::Style {
                    color: Some(alpha(
                        tokens(theme).palette.foreground,
                        if disabled {
                            style::DISABLED_OPACITY
                        } else {
                            1.0
                        },
                    )),
                })
        });

        let on_press = switch.on_toggle.map(|f| f(!checked));

        Pressable::new(
            Row::new()
                .push(track)
                .push(label)
                .spacing(8)
                .align_y(Alignment::Center),
        )
        .on_press_maybe(if disabled { None } else { on_press })
        .disabled(disabled)
        .width(Length::Shrink)
        .interaction(if disabled {
            mouse::Interaction::NotAllowed
        } else {
            mouse::Interaction::Pointer
        })
        .style(|_, _| iced::widget::button::Style::default())
        .into()
    }
}

/// The style of the track of a switch.
pub fn track(theme: &Theme, checked: bool, disabled: bool) -> container::Style {
    let t = tokens(theme);
    let p = &t.palette;
    let opacity = if disabled {
        style::DISABLED_OPACITY
    } else {
        1.0
    };

    let background = match (checked, t.is_dark()) {
        (true, _) => p.primary,
        (false, false) => p.input,
        (false, true) => alpha(p.input, 0.8),
    };

    container::Style {
        background: Some(Background::Color(alpha(background, opacity))),
        border: iced::Border {
            color: Color::TRANSPARENT,
            width: 1.0,
            radius: border::radius(f32::MAX),
        },
        shadow: style::shadow_xs(),
        ..container::Style::default()
    }
}

/// The style of the thumb of a switch.
pub fn thumb(theme: &Theme, checked: bool, disabled: bool) -> container::Style {
    let t = tokens(theme);
    let p = &t.palette;
    let opacity = if disabled {
        style::DISABLED_OPACITY
    } else {
        1.0
    };

    let color = match (t.is_dark(), checked) {
        (false, _) => p.background,
        (true, true) => p.primary_foreground,
        (true, false) => p.foreground,
    };

    container::Style {
        background: Some(Background::Color(iced_ui_core::theme::over(
            alpha(color, opacity),
            alpha(p.background, 1.0 - opacity),
        ))),
        border: border::rounded(f32::MAX),
        ..container::Style::default()
    }
}
