//! Displays a form textarea.
//!
//! Port of <https://ui.shadcn.com/docs/components/textarea>.
//!
//! The text lives in a [`Content`] stored in your state:
//!
//! ```no_run
//! # use shadcn_iced::textarea::{textarea, Content, Action};
//! # #[derive(Clone)] enum Message { Edit(Action) }
//! # fn view(content: &Content) -> iced::Element<'_, Message> {
//! textarea(content).placeholder("Type your message here.").on_action(Message::Edit).into()
//! # }
//! ```
use shadcn_iced_core::{Element, font, style};

use iced::widget::text_editor::{self, Status, Style};
use iced::widget::{TextEditor, text_input};
use iced::{Length, Pixels, Theme};

pub use text_editor::{Action, Content};

/// A multi-line text input. See the [module documentation](self).
pub struct Textarea<'a, Message> {
    raw: TextEditor<'a, iced::advanced::text::highlighter::PlainText, Message>,
    invalid: bool,
    disabled: bool,
}

/// Creates a [`Textarea`] editing the given [`Content`].
pub fn textarea<'a, Message: Clone + 'a>(content: &'a Content) -> Textarea<'a, Message> {
    Textarea {
        raw: TextEditor::new(content)
            .size(style::text::SM)
            .line_height(Pixels(20.0))
            // `min-h-16 px-2.5 py-2 md:text-sm`, plus the 1px border.
            .padding([9, 11])
            .font(font::sans())
            .min_height(64),
        invalid: false,
        disabled: true,
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
        self.disabled = false;
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
        let disabled = textarea.disabled;

        shadcn_iced_core::primitive::FocusWithin::new(
            textarea.raw.style(move |theme, status| {
                // iced only reports the disabled status after a redraw.
                let status = if disabled { Status::Disabled } else { status };

                style(theme, status, invalid)
            }),
            move |theme, status| super::input::ring(theme, status, invalid),
        )
        .into()
    }
}

/// The style of a textarea: the same as an
/// [input's](super::input::style).
pub fn style(theme: &Theme, status: Status, invalid: bool) -> Style {
    let input = super::input::style(
        theme,
        match status {
            Status::Active => text_input::Status::Active,
            Status::Hovered => text_input::Status::Hovered,
            Status::Focused { is_hovered } => text_input::Status::Focused { is_hovered },
            Status::Disabled => text_input::Status::Disabled,
        },
        invalid,
    );

    Style {
        background: input.background,
        border: input.border,
        placeholder: input.placeholder,
        value: input.value,
        selection: input.selection,
    }
}
