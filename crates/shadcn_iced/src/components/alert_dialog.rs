//! A modal dialog that interrupts the user with important content and
//! expects a response.
//!
//! Port of <https://ui.shadcn.com/docs/components/alert-dialog>. Unlike a
//! [`dialog`](super::dialog), it cannot be dismissed by clicking outside.
//!
//! ```no_run
//! # use shadcn_iced::button::{button, Variant};
//! # use shadcn_iced::alert_dialog::alert_dialog;
//! # #[derive(Clone)] enum Message { Open, Cancel, Delete }
//! # fn view<'a>(open: bool) -> iced::Element<'a, Message> {
//! alert_dialog(button("Show Dialog").variant(Variant::Outline).on_press(Message::Open), open)
//!     .title("Are you absolutely sure?")
//!     .description("This action cannot be undone.")
//!     .cancel("Cancel", Message::Cancel)
//!     .action("Continue", Message::Delete)
//!     .into()
//! # }
//! ```
use super::button::{Button, Variant, button};

use shadcn_iced_core::icon::Icon;
use shadcn_iced_core::primitive::Layer;
use shadcn_iced_core::{Element, tokens};

use iced::widget::{Column, Row, container, space};
use iced::{Alignment, Background, Length, Padding, Theme, border};

/// The size of an [`AlertDialog`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Size {
    /// `sm:max-w-sm`, left-aligned content and buttons on the right.
    #[default]
    Default,
    /// `max-w-xs`, centered content and two full-width buttons
    /// (`grid-cols-2`).
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
    /// Sets the title (`text-base font-medium`).
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the description (`text-sm text-muted-foreground`).
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Shows an icon in a muted square (`AlertDialogMedia`): next to the
    /// title in the default size, above it in the small size.
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

        // `bg-muted size-10 rounded-md *:[svg]:size-6`
        let media = alert.media.map(|icon| {
            container(icon.sized(24)).center(40).style(|theme: &Theme| {
                let t = tokens(theme);
                container::Style {
                    background: Some(Background::Color(t.palette.muted)),
                    text_color: Some(t.palette.foreground),
                    border: border::rounded(t.radius_md()),
                    ..container::Style::default()
                }
            })
        });

        // `gap-1.5`, centered in the small size.
        let text = Column::new()
            .push(alert.title.map(super::dialog::title_text))
            .push(
                alert
                    .description
                    .map(|description| super::dialog::description_text(description, small)),
            )
            .spacing(6)
            .align_x(if small {
                Alignment::Center
            } else {
                Alignment::Start
            })
            .width(Length::Fill);

        let header: Element<'a, Message> = match media {
            // `mb-2` under the media, then `gap-1.5`.
            Some(media) if small => Column::new()
                .push(media)
                .push(text)
                .spacing(14)
                .align_x(Alignment::Center)
                .width(Length::Fill)
                .into(),
            // `row-span-2 gap-x-4`
            Some(media) => Row::new()
                .push(media)
                .push(text)
                .spacing(16)
                .width(Length::Fill)
                .into(),
            None => text.into(),
        };

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

        let body = container(header).padding(16).width(Length::Fill);

        let panel = super::dialog::surface(
            Column::new()
                .push(body)
                .push(super::dialog::footer_band(buttons))
                .width(Length::Fill),
            if small { 320.0 } else { 384.0 },
        );

        Layer::new(alert.base, panel, alert.open)
            .padding(Padding::new(16.0))
            .backdrop_with(super::dialog::backdrop)
            .on_dismiss_maybe(on_cancel)
            .dismiss_on_backdrop(false)
            .into()
    }
}
