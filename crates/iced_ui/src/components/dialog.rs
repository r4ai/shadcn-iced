//! A window overlaid on either the primary window or another dialog window,
//! rendering the content underneath inert.
//!
//! Port of <https://ui.shadcn.com/docs/components/dialog>. Whether the
//! dialog is open lives in your state:
//!
//! ```no_run
//! # use iced_ui::button::{button, Variant};
//! # use iced_ui::dialog::dialog;
//! # #[derive(Clone)] enum Message { Open, Close }
//! # fn view<'a>(open: bool) -> iced::Element<'a, Message> {
//! dialog(button("Edit Profile").variant(Variant::Outline).on_press(Message::Open), open)
//!     .title("Edit profile")
//!     .description("Make changes to your profile here.")
//!     .footer(button("Save changes").on_press(Message::Close))
//!     .on_close(Message::Close)
//!     .into()
//! # }
//! ```
use iced_ui_core::icon::lucide;
use iced_ui_core::primitive::{Layer, Pressable};
use iced_ui_core::theme::alpha;
use iced_ui_core::{Element, font, style, tokens};

use iced::widget::button::{Status, Style};
use iced::widget::{Column, Row, Stack, container, space, text};
use iced::{Alignment, Background, Length, Padding, Theme};

/// A dialog. See the [module documentation](self).
pub struct Dialog<'a, Message> {
    base: Element<'a, Message>,
    open: bool,
    title: Option<String>,
    description: Option<String>,
    content: Vec<Element<'a, Message>>,
    footer: Option<Element<'a, Message>>,
    on_close: Option<Message>,
    close_button: bool,
    width: f32,
    padding: f32,
}

/// Creates a [`Dialog`] shown over the whole window when `open`.
///
/// `base` is displayed as usual; it is typically the button that opens the
/// dialog, or the root of your view.
pub fn dialog<'a, Message: Clone + 'a>(base: impl Into<Element<'a, Message>>, open: bool) -> Dialog<'a, Message> {
    Dialog {
        base: base.into(),
        open,
        title: None,
        description: None,
        content: Vec::new(),
        footer: None,
        on_close: None,
        close_button: true,
        width: 512.0,
        padding: 24.0,
    }
}

impl<'a, Message: Clone + 'a> Dialog<'a, Message> {
    /// Sets the title (`text-lg font-semibold`).
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Appends content below the header.
    pub fn content(mut self, content: impl Into<Element<'a, Message>>) -> Self {
        self.content.push(content.into());
        self
    }

    /// Sets the footer, aligned to the right.
    pub fn footer(mut self, footer: impl Into<Element<'a, Message>>) -> Self {
        self.footer = Some(footer.into());
        self
    }

    /// Sets the message produced when the user closes the dialog (close
    /// button, <kbd>Escape</kbd> or a click outside).
    pub fn on_close(mut self, message: Message) -> Self {
        self.on_close = Some(message);
        self
    }

    /// Shows or hides the close button in the top-right corner.
    pub fn close_button(mut self, show: bool) -> Self {
        self.close_button = show;
        self
    }

    /// Sets the maximum width. Defaults to `sm:max-w-lg` (512).
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Sets the padding. Defaults to `p-6`; command dialogs use `0.0`.
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }
}

/// The header of a dialog: a title and a description.
pub fn header<'a, Message: 'a>(title: Option<String>, description: Option<String>, centered: bool) -> Option<Element<'a, Message>> {
    if title.is_none() && description.is_none() {
        return None;
    }

    let align = if centered { Alignment::Center } else { Alignment::Start };

    Some(
        Column::new()
            .push(title.map(|title| {
                text(title)
                    .size(style::text::LG)
                    .font(font::SEMIBOLD)
                    .line_height(iced::Pixels(18.0))
            }))
            .push(description.map(|description| {
                text(description)
                    .size(style::text::SM)
                    .line_height(1.43)
                    .align_x(if centered {
                        iced::alignment::Horizontal::Center
                    } else {
                        iced::alignment::Horizontal::Left
                    })
                    .style(|theme: &Theme| text::Style {
                        color: Some(tokens(theme).palette.muted_foreground),
                    })
            }))
            .spacing(8)
            .align_x(align)
            .width(Length::Fill)
            .into(),
    )
}

/// The close button of dialogs and sheets (`opacity-70 hover:opacity-100`).
pub fn close_button<'a, Message: Clone + 'a>(on_close: Option<Message>) -> Element<'a, Message> {
    Pressable::new(lucide::X.sized(16))
        .on_press_maybe(on_close)
        .style(|theme: &Theme, status| {
            let t = tokens(theme);
            Style {
                background: None,
                text_color: alpha(
                    t.palette.foreground,
                    if matches!(status, Status::Hovered | Status::Pressed) { 1.0 } else { 0.7 },
                ),
                border: style::rounded(2.0),
                ..Style::default()
            }
        })
        .into()
}

/// The style of a dialog panel (`rounded-lg border bg-background
/// shadow-lg`).
pub fn panel(theme: &Theme) -> container::Style {
    let t = tokens(theme);

    container::Style {
        background: Some(Background::Color(t.palette.background)),
        text_color: Some(t.palette.foreground),
        border: style::border(t.palette.border, t.radius_lg()),
        shadow: style::shadow_lg(),
        ..container::Style::default()
    }
}

impl<'a, Message: Clone + 'a> From<Dialog<'a, Message>> for Element<'a, Message> {
    fn from(dialog: Dialog<'a, Message>) -> Self {
        let footer = dialog.footer.map(|footer| {
            Row::new()
                .push(space::horizontal())
                .push(footer)
                .align_y(Alignment::Center)
                .width(Length::Fill)
        });

        let body = Column::new()
            .push(header(dialog.title, dialog.description, false))
            .extend(dialog.content)
            .push(footer)
            .spacing(16)
            .width(Length::Fill);

        let mut layers = Stack::new().push(container(body).padding(dialog.padding).width(Length::Fill));

        if dialog.close_button {
            layers = layers.push(
                container(close_button(dialog.on_close.clone()))
                    .padding(16)
                    .align_right(Length::Fill),
            );
        }

        let panel = container(layers).max_width(dialog.width).width(Length::Fill).style(panel);

        Layer::new(dialog.base, panel, dialog.open)
            .padding(Padding::new(16.0))
            .on_dismiss_maybe(dialog.on_close)
            .into()
    }
}
