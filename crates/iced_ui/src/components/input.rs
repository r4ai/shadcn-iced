//! Displays a form input field.
//!
//! Port of <https://ui.shadcn.com/docs/components/input>.
//!
//! ```no_run
//! # use iced_ui::input::input;
//! # #[derive(Clone)] enum Message { Email(String) }
//! # fn view(email: &str) -> iced::Element<'_, Message> {
//! input("Email", email).on_input(Message::Email).into()
//! # }
//! ```
use iced_ui_core::icon::Icon;
use iced_ui_core::theme::{alpha, tokens};
use iced_ui_core::{Element, font, style};

use iced::widget::text_input::{self, Status, Style};
use iced::widget::{Id, TextInput};
use iced::{Background, Border, Color, Length, Pixels, Theme, border};

/// A text input. See the [module documentation](self).
pub struct Input<'a, Message: Clone> {
    raw: TextInput<'a, Message>,
    invalid: bool,
    icon: Option<Icon>,
}

/// Creates an [`Input`] with a placeholder and a value.
///
/// The input is disabled until [`Input::on_input`] is set.
pub fn input<'a, Message: Clone + 'a>(placeholder: &str, value: &str) -> Input<'a, Message> {
    Input {
        raw: TextInput::new(placeholder, value)
            .size(style::text::SM)
            .line_height(Pixels(20.0))
            .padding([8, 12])
            .font(font::SANS),
        invalid: false,
        icon: None,
    }
}

impl<'a, Message: Clone + 'a> Input<'a, Message> {
    /// Sets the message produced when the value changes.
    pub fn on_input(mut self, f: impl Fn(String) -> Message + 'a) -> Self {
        self.raw = self.raw.on_input(f);
        self
    }

    /// Sets the message produced when the value changes, if any.
    pub fn on_input_maybe(mut self, f: Option<impl Fn(String) -> Message + 'a>) -> Self {
        self.raw = self.raw.on_input_maybe(f);
        self
    }

    /// Sets the message produced when <kbd>Enter</kbd> is pressed.
    pub fn on_submit(mut self, message: Message) -> Self {
        self.raw = self.raw.on_submit(message);
        self
    }

    /// Sets the message produced when text is pasted.
    pub fn on_paste(mut self, f: impl Fn(String) -> Message + 'a) -> Self {
        self.raw = self.raw.on_paste(f);
        self
    }

    /// Hides the value, for passwords.
    pub fn secure(mut self, secure: bool) -> Self {
        self.raw = self.raw.secure(secure);
        self
    }

    /// Marks the input as invalid (`aria-invalid`).
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Adds a leading icon.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Sets the [`Id`] of the input, to focus it with an operation.
    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.raw = self.raw.id(id);
        self
    }

    /// Sets the width. Inputs fill the available width by default.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.raw = self.raw.width(width);
        self
    }

    /// Returns the underlying iced [`TextInput`], styled.
    pub fn into_text_input(self) -> TextInput<'a, Message> {
        let invalid = self.invalid;
        let mut raw = self.raw;

        if let Some(icon) = self.icon {
            raw = raw
                .icon(text_input::Icon {
                    font: font::ICONS,
                    code_point: icon.codepoint(),
                    size: Some(Pixels(16.0)),
                    spacing: 8.0,
                    side: text_input::Side::Left,
                })
                .padding([8, 12]);
        }

        raw.style(move |theme, status| style(theme, status, invalid))
    }
}

impl<'a, Message: Clone + 'a> From<Input<'a, Message>> for Element<'a, Message> {
    fn from(input: Input<'a, Message>) -> Self {
        input.into_text_input().into()
    }
}

/// The style of an input.
pub fn style(theme: &Theme, status: Status, invalid: bool) -> Style {
    let t = tokens(theme);
    let p = &t.palette;
    let dark = t.is_dark();

    let focused = matches!(status, Status::Focused { .. });
    let disabled = status == Status::Disabled;

    let border_color = if invalid {
        p.destructive
    } else if focused {
        p.ring
    } else {
        p.input
    };

    let opacity = if disabled { style::DISABLED_OPACITY } else { 1.0 };

    Style {
        background: Background::Color(if dark {
            alpha(p.input, 0.3 * opacity)
        } else {
            Color::TRANSPARENT
        }),
        border: Border {
            color: alpha(border_color, opacity),
            width: 1.0,
            radius: border::radius(t.radius_md()),
        },
        icon: alpha(p.muted_foreground, opacity),
        placeholder: alpha(p.muted_foreground, opacity),
        value: alpha(p.foreground, opacity),
        selection: alpha(p.primary, if dark { 0.4 } else { 0.2 }),
    }
}
