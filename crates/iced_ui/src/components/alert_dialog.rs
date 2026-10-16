//! A modal dialog that interrupts the user with important content and
//! expects a response.
//!
//! Port of <https://ui.shadcn.com/docs/components/alert-dialog>. Unlike a
//! [`dialog`](super::dialog), it cannot be dismissed by clicking outside.
//!
//! ```no_run
//! # use iced_ui::button::{button, Variant};
//! # use iced_ui::alert_dialog::alert_dialog;
//! # #[derive(Clone)] enum Message { Open, Cancel, Delete }
//! # fn view<'a>(open: bool) -> iced::Element<'a, Message> {
//! alert_dialog(button("Delete").variant(Variant::Outline).on_press(Message::Open), open)
//!     .title("Are you absolutely sure?")
//!     .description("This action cannot be undone.")
//!     .cancel("Cancel", Message::Cancel)
//!     .action("Continue", Message::Delete)
//!     .into()
//! # }
//! ```
use super::button::{Button, Variant, button};

use iced_ui_core::icon::Icon;
use iced_ui_core::primitive::Layer;
use iced_ui_core::{Element, tokens};

use iced::widget::{Column, Row, container, space};
use iced::{Alignment, Background, Length, Padding, Theme, border};

/// The size of an [`AlertDialog`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Size {
    /// `max-w-lg`, left-aligned content.
    #[default]
    Default,
    /// `max-w-xs`, centered content and full-width buttons.
    Sm,
}

/// An alert dialog. See the [module documentation](self).
pub struct AlertDialog<'a, Message> {
    base: Element<'a, Message>,
    open: bool,
    title: Option<String>,
    description: Option<String>,
    media: Option<Icon>,
    cancel: Option<(String, Message)>,
    action: Option<Button<'a, Message>>,
    size: Size,
}

/// Creates an [`AlertDialog`] shown over the whole window when `open`.
pub fn alert_dialog<'a, Message: Clone + 'a>(
    base: impl Into<Element<'a, Message>>,
    open: bool,
) -> AlertDialog<'a, Message> {
    AlertDialog {
        base: base.into(),
        open,
        title: None,
        description: None,
        media: None,
        cancel: None,
        action: None,
        size: Size::Default,
    }
}

impl<'a, Message: Clone + 'a> AlertDialog<'a, Message> {
    /// Sets the title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Shows an icon in a muted square above the title.
    pub fn media(mut self, icon: Icon) -> Self {
        self.media = Some(icon);
        self
    }

    /// Sets the cancel button, which also handles <kbd>Escape</kbd>.
    pub fn cancel(mut self, label: impl Into<String>, message: Message) -> Self {
        self.cancel = Some((label.into(), message));
        self
    }

    /// Sets the action button.
    pub fn action(mut self, label: impl Into<String>, message: Message) -> Self {
        self.action = Some(button(label).on_press(message));
        self
    }

    /// Sets a destructive action button.
    pub fn destructive_action(mut self, label: impl Into<String>, message: Message) -> Self {
        self.action = Some(
            button(label)
                .variant(Variant::Destructive)
                .on_press(message),
        );
        self
    }

    /// Sets the size.
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }
}

impl<'a, Message: Clone + 'a> From<AlertDialog<'a, Message>> for Element<'a, Message> {
    fn from(alert: AlertDialog<'a, Message>) -> Self {
        let small = alert.size == Size::Sm;
        let on_cancel = alert.cancel.as_ref().map(|(_, message)| message.clone());

        let media = alert.media.map(|icon| {
            container(icon.sized(32)).center(64).style(|theme: &Theme| {
                let t = tokens(theme);
                container::Style {
                    background: Some(Background::Color(t.palette.muted)),
                    border: border::rounded(t.radius_md()),
                    ..container::Style::default()
                }
            })
        });

        let fill = |button: Button<'a, Message>| {
            if small {
                button.width(Length::Fill)
            } else {
                button
            }
        };

        let cancel = alert.cancel.map(|(label, message)| {
            fill(button(label).variant(Variant::Outline).on_press(message))
        });

        let buttons = Row::new()
            .push((!small).then(space::horizontal))
            .push(cancel)
            .push(alert.action.map(fill))
            .spacing(8)
            .width(Length::Fill);

        let body = Column::new()
            .push(media)
            .push(super::dialog::header(alert.title, alert.description, small))
            .push(buttons)
            .spacing(16)
            .align_x(if small {
                Alignment::Center
            } else {
                Alignment::Start
            })
            .width(Length::Fill);

        let panel = container(body)
            .padding(24)
            .max_width(if small { 320.0 } else { 512.0 })
            .width(Length::Fill)
            .style(super::dialog::panel);

        Layer::new(alert.base, panel, alert.open)
            .padding(Padding::new(16.0))
            .on_dismiss_maybe(on_cancel)
            .dismiss_on_backdrop(false)
            .into()
    }
}
