//! Renders an accessible label associated with controls.
//!
//! Port of <https://ui.shadcn.com/docs/components/label>.
//!
//! In shadcn/ui, `htmlFor` associates a label with its control. In iced,
//! wrap the label and its control in [`labelled`]: clicking the label then
//! focuses the control and, for checkboxes, radios and switches, toggles it.
//!
//! ```no_run
//! # use shadcn_iced::checkbox::checkbox;
//! # use shadcn_iced::label::{label, labelled};
//! # #[derive(Clone)] enum Message { Terms(bool) }
//! # fn view<'a>(accepted: bool) -> iced::Element<'a, Message> {
//! labelled(
//!     iced::widget::row![
//!         checkbox(accepted).on_toggle(Message::Terms),
//!         label("Accept terms and conditions"),
//!     ]
//!     .spacing(12),
//! )
//! .into()
//! # }
//! ```
use shadcn_iced_core::primitive::Labelled;
use shadcn_iced_core::{Element, font, style};

use iced::widget::{Text, text};

/// Creates a label (`text-sm font-medium leading-none`).
pub fn label<'a>(content: impl text::IntoFragment<'a>) -> Text<'a> {
    text(content)
        .size(style::text::SM)
        .font(font::medium())
        .line_height(iced::Pixels(14.0))
}

/// Associates the labels in `content` with its first control, like
/// `htmlFor`: a click that no control handles focuses the control (and
/// toggles checkboxes, radios and switches).
pub fn labelled<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
) -> Labelled<'a, Message> {
    Labelled::new(content)
}
