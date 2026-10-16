//! Displays a form textarea.
//!
//! Port of <https://ui.shadcn.com/docs/components/textarea>.
//!
//! The text lives in a [`Content`] stored in your state:
//!
//! ```no_run
//! # use iced_ui::textarea::{textarea, Content, Action};
//! # #[derive(Clone)] enum Message { Edit(Action) }
//! # fn view(content: &Content) -> iced::Element<'_, Message> {
//! textarea(content).placeholder("Type your message here.").on_action(Message::Edit).into()
//! # }
//! ```
use iced_ui_core::theme::{alpha, tokens};
use iced_ui_core::{Element, font, style};

use iced::widget::TextEditor;
use iced::widget::text_editor::{self, Status, Style};
use iced::{Background, Border, Color, Length, Pixels, Theme, border};

pub use text_editor::{Action, Content};

/// A multi-line text input. See the [module documentation](self).
pub struct Textarea<'a, Message> {
    raw: TextEditor<'a, iced::advanced::text::highlighter::PlainText, Message>,
    invalid: bool,
}

/// Creates a [`Textarea`] editing the given [`Content`].
pub fn textarea<'a, Message: Clone + 'a>(content: &'a Content) -> Textarea<'a, Message> {
    Textarea {
        raw: TextEditor::new(content)
            .size(style::text::SM)
            .line_height(Pixels(20.0))
            .padding([8, 12])
            .font(font::SANS)
            .min_height(64),
        invalid: false,
    }
}

impl<'a, Message: Clone + 'a> Textarea<'a, Message> {
    /// Sets the placeholder.
    pub fn placeholder(mut self, placeholder: impl iced::widget::text::IntoFragment<'a>) -> Self {
        self.raw = self.raw.placeholder(placeholder);
        self
    }

    /// Sets the message produced on every edit. Without it, the textarea is
    /// disabled.
    pub fn on_action(mut self, f: impl Fn(Action) -> Message + 'a) -> Self {
        self.raw = self.raw.on_action(f);
        self
    }

    /// Marks the textarea as invalid.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Sets the height.
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.raw = self.raw.height(height);
        self
    }

    /// Sets the width.
    pub fn width(mut self, width: impl Into<Pixels>) -> Self {
        self.raw = self.raw.width(width);
        self
    }
}

impl<'a, Message: Clone + 'a> From<Textarea<'a, Message>> for Element<'a, Message> {
    fn from(textarea: Textarea<'a, Message>) -> Self {
        let invalid = textarea.invalid;

        iced_ui_core::primitive::FocusWithin::new(
            textarea
                .raw
                .style(move |theme, status| style(theme, status, invalid)),
            move |theme, status| super::input::ring(theme, status, invalid),
        )
        .into()
    }
}

/// The style of a textarea.
pub fn style(theme: &Theme, status: Status, invalid: bool) -> Style {
    let t = tokens(theme);
    let p = &t.palette;
    let dark = t.is_dark();

    let focused = matches!(status, Status::Focused { .. });
    let opacity = if status == Status::Disabled {
        style::DISABLED_OPACITY
    } else {
        1.0
    };

    let border_color = if invalid {
        p.destructive
    } else if focused {
        p.ring
    } else {
        p.input
    };

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
        placeholder: alpha(p.muted_foreground, opacity),
        value: alpha(p.foreground, opacity),
        selection: alpha(p.primary, if dark { 0.4 } else { 0.2 }),
    }
}
