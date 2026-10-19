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
use super::button::{Variant, button};

use shadcn_iced_core::icon::Icon;
use shadcn_iced_core::primitive::layer::auto_focus;
use shadcn_iced_core::primitive::{Breakpoint, Layer};
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
    action: Option<(String, Message, Variant)>,
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
        self.action = Some((label.into(), message, Variant::Default));
        self
    }

    /// Sets a destructive action button.
    pub fn destructive_action(mut self, label: impl Into<String>, message: Message) -> Self {
        self.action = Some((label.into(), message, Variant::Destructive));
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
        let sm = Breakpoint::sm();

        // Below `sm`, the default size looks like the small one, but for its
        // footer.
        let content = |centered: bool, footer: Footer| {
            Column::new()
                .push(
                    container(header(
                        alert.title.clone(),
                        alert.description.clone(),
                        alert.media,
                        centered,
                    ))
                    .padding(16)
                    .width(Length::Fill),
                )
                .push(super::dialog::footer_band(buttons(
                    alert.cancel.clone(),
                    alert.action.clone(),
                    footer,
                )))
                .width(Length::Fill)
        };

        let (content, wide, narrow): (Element<'a, Message>, _, _) = if small {
            // `max-w-xs`
            (content(true, Footer::Grid).into(), 320.0, 320.0)
        } else {
            // `max-w-xs sm:max-w-sm`
            (
                sm.either(content(false, Footer::Row), content(true, Footer::Stack))
                    .into(),
                384.0,
                320.0,
            )
        };

        let panel = super::dialog::responsive_surface(content, &sm, wide, narrow);

        Layer::new(alert.base, panel, alert.open)
            .padding(Padding::new(super::dialog::LAYER_PADDING))
            .backdrop_with(super::dialog::backdrop)
            .on_dismiss_maybe(on_cancel)
            .dismiss_on_backdrop(false)
            .into()
    }
}

/// The header of an alert dialog: the media, the title and the description,
/// `gap-1.5`, centered (`place-items-center text-center`) or not.
fn header<'a, Message: 'a>(
    title: Option<String>,
    description: Option<String>,
    media: Option<Icon>,
    centered: bool,
) -> Element<'a, Message> {
    // `bg-muted size-10 rounded-md *:[svg]:size-6`
    let media = media.map(|icon| {
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

    let text = Column::new()
        .push(title.map(super::dialog::title_text))
        // `text-balance` (`md:text-pretty` on wide screens)
        .push(
            description
                .map(|description| super::dialog::balanced_description(description, centered)),
        )
        .spacing(6)
        .align_x(if centered {
            Alignment::Center
        } else {
            Alignment::Start
        })
        .width(Length::Fill);

    match media {
        // `mb-2` under the media, then `gap-1.5`.
        Some(media) if centered => Column::new()
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
    }
}

/// The layout of the buttons of an alert dialog.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Footer {
    /// `sm:flex-row sm:justify-end`
    Row,
    /// `flex-col-reverse`: the action above the cancel button, both
    /// filling the footer.
    Stack,
    /// `grid grid-cols-2`, in the small size.
    Grid,
}

/// The buttons of an alert dialog, `gap-2` apart.
fn buttons<'a, Message: Clone + 'a>(
    cancel: Option<(String, Message)>,
    action: Option<(String, Message, Variant)>,
    footer: Footer,
) -> Element<'a, Message> {
    let fill = footer != Footer::Row;

    // Like Radix, the alert dialog focuses its cancel button when it opens,
    // so that a key press does not confirm the action by accident.
    let cancel = cancel.map(|(label, message)| -> Element<'a, Message> {
        let button = button(label).variant(Variant::Outline).on_press(message);

        auto_focus(if fill {
            button.width(Length::Fill)
        } else {
            button
        })
        .into()
    });

    let action = action.map(|(label, message, variant)| {
        let button = button(label).variant(variant).on_press(message);

        if fill {
            button.width(Length::Fill)
        } else {
            button
        }
    });

    match footer {
        Footer::Row => Row::new()
            .push(space::horizontal())
            .push(cancel)
            .push(action)
            .spacing(8)
            .width(Length::Fill)
            .into(),
        Footer::Grid => Row::new()
            .push(cancel)
            .push(action)
            .spacing(8)
            .width(Length::Fill)
            .into(),
        Footer::Stack => Column::new()
            .push(action)
            .push(cancel)
            .spacing(8)
            .width(Length::Fill)
            .into(),
    }
}
