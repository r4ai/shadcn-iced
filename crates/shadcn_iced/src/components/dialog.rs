//! A window overlaid on either the primary window or another dialog window,
//! rendering the content underneath inert.
//!
//! Port of <https://ui.shadcn.com/docs/components/dialog>. Whether the
//! dialog is open lives in your state:
//!
//! ```no_run
//! # use shadcn_iced::button::{button, Variant};
//! # use shadcn_iced::dialog::dialog;
//! # #[derive(Clone)] enum Message { Open, Close }
//! # fn view<'a>(open: bool) -> iced::Element<'a, Message> {
//! dialog(button("Open Dialog").variant(Variant::Outline).on_press(Message::Open), open)
//!     .title("Edit profile")
//!     .description("Make changes to your profile here.")
//!     .footer(button("Save changes").on_press(Message::Close))
//!     .on_close(Message::Close)
//!     .into()
//! # }
//! ```
use super::button::{Size, Variant, icon_button};

use shadcn_iced_core::icon::lucide;
use shadcn_iced_core::primitive::Layer;
use shadcn_iced_core::theme::alpha;
use shadcn_iced_core::{Element, font, style, tokens};

use iced::widget::{Column, Row, Stack, container, space, text};
use iced::{Alignment, Background, Border, Color, Length, Padding, Theme, border};

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
pub fn dialog<'a, Message: Clone + 'a>(
    base: impl Into<Element<'a, Message>>,
    open: bool,
) -> Dialog<'a, Message> {
    Dialog {
        base: base.into(),
        open,
        title: None,
        description: None,
        content: Vec::new(),
        footer: None,
        on_close: None,
        close_button: true,
        width: 384.0,
        padding: 16.0,
    }
}

impl<'a, Message: Clone + 'a> Dialog<'a, Message> {
    /// Sets the title (`text-base leading-none font-medium`).
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the description (`text-sm text-muted-foreground`).
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Appends content below the header, `gap-4` apart.
    pub fn content(mut self, content: impl Into<Element<'a, Message>>) -> Self {
        self.content.push(content.into());
        self
    }

    /// Sets the footer: a muted band at the bottom of the dialog
    /// (`bg-muted/50 border-t p-4`), its content aligned to the right.
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

    /// Sets the maximum width. Defaults to `sm:max-w-sm` (384).
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Sets the padding. Defaults to `p-4`; command dialogs use `0.0`.
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }
}

/// The header of a dialog: a title (`text-base leading-none font-medium`)
/// and a description, `gap-2` apart.
pub fn header<'a, Message: 'a>(
    title: Option<String>,
    description: Option<String>,
    centered: bool,
) -> Option<Element<'a, Message>> {
    if title.is_none() && description.is_none() {
        return None;
    }

    Some(
        Column::new()
            .push(title.map(|title| title_text(title).line_height(iced::Pixels(16.0))))
            .push(description.map(|description| description_text(description, centered)))
            .spacing(8)
            .align_x(if centered {
                Alignment::Center
            } else {
                Alignment::Start
            })
            .width(Length::Fill)
            .into(),
    )
}

/// The title of a dialog, sheet or drawer (`text-base font-medium`).
pub fn title_text<'a>(title: String) -> text::Text<'a> {
    text(title)
        .size(style::text::BASE)
        // `cn-font-heading`
        .font(font::heading(iced::font::Weight::Medium))
        .line_height(iced::Pixels(24.0))
        .style(|theme: &Theme| text::Style {
            color: Some(tokens(theme).palette.foreground),
        })
}

/// The description of a dialog, sheet or drawer (`text-sm
/// text-muted-foreground`).
pub fn description_text<'a>(description: String, centered: bool) -> text::Text<'a> {
    text(description)
        .size(style::text::SM)
        .line_height(iced::Pixels(20.0))
        .align_x(if centered {
            iced::alignment::Horizontal::Center
        } else {
            iced::alignment::Horizontal::Left
        })
        .style(|theme: &Theme| text::Style {
            color: Some(tokens(theme).palette.muted_foreground),
        })
}

/// The close button of dialogs and sheets: a ghost `icon-sm` button with
/// an `X`.
pub fn close_button<'a, Message: Clone + 'a>(on_close: Option<Message>) -> Element<'a, Message> {
    icon_button(lucide::X)
        .variant(Variant::Ghost)
        .size(Size::IconSm)
        .on_press_maybe(on_close)
        .into()
}

/// The backdrop of modal overlays: `bg-black/10 backdrop-blur-xs`.
///
/// iced cannot blur what is behind an overlay, so the blur is approximated
/// by washing the window out a little with the background color (a blur
/// lowers the contrast of what it covers) before dimming it.
pub fn backdrop(theme: &Theme) -> Color {
    // A `background/20` wash under `black/10`, composed as a single color.
    const WASH: f32 = 0.2;
    const DIM: f32 = 0.1;

    let background = tokens(theme).palette.background;
    let a = 1.0 - (1.0 - DIM) * (1.0 - WASH);
    let scale = (1.0 - DIM) * WASH / a;

    Color {
        r: background.r * scale,
        g: background.g * scale,
        b: background.b * scale,
        a,
    }
}

/// The style of a dialog panel (`rounded-xl bg-popover
/// text-popover-foreground ring-1 ring-foreground/10`).
pub fn panel(theme: &Theme) -> container::Style {
    let t = tokens(theme);

    container::Style {
        background: Some(Background::Color(t.palette.popover)),
        text_color: Some(t.palette.popover_foreground),
        border: style::ring_border(&t, t.radius_xl()),
        ..container::Style::default()
    }
}

/// Wraps the content of a dialog in a [`panel`] of the given maximum width,
/// drawing its ring on top so a [`footer_band`] can reach the edges.
pub fn surface<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    max_width: f32,
) -> Element<'a, Message> {
    let ring =
        container(space().width(Length::Fill).height(Length::Fill)).style(|theme: &Theme| {
            let t = tokens(theme);

            container::Style {
                border: style::ring_border(&t, t.radius_xl()),
                ..container::Style::default()
            }
        });

    container(Stack::new().push(content).push(ring))
        .max_width(max_width)
        .width(Length::Fill)
        .style(|theme: &Theme| container::Style {
            border: style::rounded(tokens(theme).radius_xl()),
            ..panel(theme)
        })
        .into()
}

/// The footer band of dialogs and alert dialogs: `bg-muted/50 border-t
/// p-4 rounded-b-xl`.
pub fn footer_band<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    let rule =
        container(space().width(Length::Fill).height(1)).style(|theme: &Theme| container::Style {
            background: Some(Background::Color(tokens(theme).palette.border)),
            ..container::Style::default()
        });

    container(
        Column::new()
            .push(rule)
            .push(container(content).padding(16).width(Length::Fill)),
    )
    .width(Length::Fill)
    .style(|theme: &Theme| {
        let t = tokens(theme);
        let radius = t.radius_xl();

        container::Style {
            background: Some(Background::Color(alpha(t.palette.muted, 0.5))),
            border: Border {
                radius: border::radius(0).bottom_left(radius).bottom_right(radius),
                ..Border::default()
            },
            ..container::Style::default()
        }
    })
    .into()
}

impl<'a, Message: Clone + 'a> From<Dialog<'a, Message>> for Element<'a, Message> {
    fn from(dialog: Dialog<'a, Message>) -> Self {
        let footer = dialog.footer.map(|footer| {
            footer_band(
                Row::new()
                    .push(space::horizontal())
                    .push(footer)
                    .align_y(Alignment::Center)
                    .width(Length::Fill),
            )
        });

        let body = Column::new()
            .push(header(dialog.title, dialog.description, false))
            .extend(dialog.content)
            .spacing(16)
            .width(Length::Fill);

        let inner = Column::new()
            .push(container(body).padding(dialog.padding).width(Length::Fill))
            .push(footer)
            .width(Length::Fill);

        let mut layers = Stack::new().push(inner);

        if dialog.close_button {
            // `absolute top-2 right-2`
            layers = layers.push(
                container(close_button(dialog.on_close.clone()))
                    .padding(8)
                    .align_right(Length::Fill),
            );
        }

        Layer::new(dialog.base, surface(layers, dialog.width), dialog.open)
            .padding(Padding::new(16.0))
            .backdrop_with(backdrop)
            .on_dismiss_maybe(dialog.on_close)
            .into()
    }
}
