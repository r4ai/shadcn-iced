//! A file attached to a message: an icon or a thumbnail, a name, a size and
//! actions.
//!
//! Port of <https://ui.shadcn.com/docs/components/attachment>.
//!
//! ```no_run
//! # use shadcn_iced::attachment::{attachment, State};
//! # use shadcn_iced_core::icon::lucide;
//! # fn view<'a>() -> iced::Element<'a, ()> {
//! attachment("report.pdf").icon(lucide::FILE_TEXT).description("2.4 MB").into()
//! # }
//! ```
use shadcn_iced_core::icon::{Icon, lucide};
use shadcn_iced_core::primitive::Pressable;
use shadcn_iced_core::theme::alpha;
use shadcn_iced_core::{Element, font, style, tokens};

use iced::widget::button::Status;
use iced::widget::{Column, Row, Stack, container, image, text};
use iced::{Alignment, Background, Border, Length, Padding, Theme, border};

/// The state of an [`Attachment`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum State {
    /// Selected but not uploaded yet (`border-dashed`).
    Idle,
    /// Uploading: dimmed image.
    Uploading,
    /// Processing: dimmed image.
    Processing,
    /// Ready.
    #[default]
    Done,
    /// Failed: destructive border, media and description.
    Error,
}

/// The size of an [`Attachment`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Size {
    /// `w-10` media, `gap-2 px-2.5 py-2 text-sm`.
    #[default]
    Default,
    /// `w-8` media, `gap-2.5 px-2 py-1.5 text-xs`.
    Sm,
    /// `w-7` media, `gap-1.5 px-1.5 py-1 text-xs rounded-lg`.
    Xs,
}

impl Size {
    fn media(self) -> f32 {
        match self {
            Size::Default => 40.0,
            Size::Sm => 32.0,
            Size::Xs => 28.0,
        }
    }

    fn gap(self) -> f32 {
        match self {
            Size::Default => 8.0,
            Size::Sm => 10.0,
            Size::Xs => 6.0,
        }
    }

    /// The padding with content, and with media only.
    fn padding(self, content: bool) -> Padding {
        match (self, content) {
            (Size::Default, true) => Padding::from([8, 10]),
            (Size::Sm, true) => Padding::from([6, 8]),
            (Size::Xs, true) => Padding::from([4, 6]),
            (Size::Default, false) => Padding::new(8.0),
            (Size::Sm, false) => Padding::new(6.0),
            (Size::Xs, false) => Padding::new(4.0),
        }
    }

    fn text_size(self) -> f32 {
        match self {
            Size::Default => style::text::SM,
            _ => style::text::XS,
        }
    }
}

enum Media {
    Icon(Icon),
    Image(image::Handle),
    Spinner,
}

/// An attachment. See the [module documentation](self).
pub struct Attachment<'a, Message> {
    title: Option<String>,
    description: Option<String>,
    media: Option<Media>,
    state: State,
    size: Size,
    vertical: bool,
    width: Option<Length>,
    actions: Vec<Element<'a, Message>>,
    on_press: Option<Message>,
}

/// Creates an [`Attachment`] with a title (usually the file name).
pub fn attachment<'a, Message: 'a>(title: impl Into<String>) -> Attachment<'a, Message> {
    let title: String = title.into();

    Attachment {
        title: (!title.is_empty()).then_some(title),
        description: None,
        media: None,
        state: State::Done,
        size: Size::Default,
        vertical: false,
        width: None,
        actions: Vec::new(),
        on_press: None,
    }
}

impl<'a, Message: Clone + 'a> Attachment<'a, Message> {
    /// Sets the description (size, type, error...).
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Shows an icon as media.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.media = Some(Media::Icon(icon));
        self
    }

    /// Shows an image as media (`AttachmentMedia variant="image"`).
    pub fn image(mut self, handle: impl Into<image::Handle>) -> Self {
        self.media = Some(Media::Image(handle.into()));
        self
    }

    /// Shows a spinner as media, e.g. while uploading.
    pub fn spinner(mut self) -> Self {
        self.media = Some(Media::Spinner);
        self
    }

    /// Sets the state.
    pub fn state(mut self, state: State) -> Self {
        self.state = state;
        self
    }

    /// Sets the size.
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// Lays the attachment out vertically, as a tile
    /// (`orientation="vertical"`).
    pub fn vertical(mut self, vertical: bool) -> Self {
        self.vertical = vertical;
        self
    }

    /// Sets the width, e.g. `Length::Fill` for `w-full`.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = Some(width.into());
        self
    }

    /// Appends an action, e.g. [`action`].
    pub fn action(mut self, action: impl Into<Element<'a, Message>>) -> Self {
        self.actions.push(action.into());
        self
    }

    /// Makes the whole attachment pressable (`AttachmentTrigger`),
    /// highlighted on hover (`hover:bg-muted/50`).
    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }
}

impl<'a, Message: Clone + 'a> From<Attachment<'a, Message>> for Element<'a, Message> {
    fn from(attachment: Attachment<'a, Message>) -> Self {
        let state = attachment.state;
        let size = attachment.size;
        let vertical = attachment.vertical;
        let has_content = attachment.title.is_some() || attachment.description.is_some();
        let padding = size.padding(has_content);

        // `w-24 has-data-[slot=attachment-content]:w-30`, border included.
        let tile_width = if has_content { 120.0 } else { 96.0 };
        let media_size = if vertical {
            tile_width - 2.0 - padding.left - padding.right
        } else {
            size.media()
        };
        let icon_size = match (vertical, size) {
            (true, _) => 24.0,
            (false, Size::Xs) => 14.0,
            (false, _) => 16.0,
        };

        let media = attachment.media.map(|media| {
            let dimmed =
                matches!(media, Media::Image(_)) && !matches!(state, State::Idle | State::Done);

            let inner: Element<'a, Message> = match media {
                Media::Icon(icon) => icon.sized(icon_size).into(),
                Media::Spinner => super::spinner::spinner().size(icon_size).into(),
                Media::Image(handle) => image(handle)
                    .width(media_size)
                    .height(media_size)
                    .content_fit(iced::ContentFit::Cover)
                    // `opacity-60`
                    .opacity(if dimmed { 0.6_f32 } else { 1.0_f32 })
                    .into(),
            };

            container(inner)
                .center(media_size)
                .clip(true)
                .style(move |theme: &Theme| {
                    let t = tokens(theme);
                    let p = &t.palette;
                    let error = state == State::Error;

                    container::Style {
                        background: Some(Background::Color(if error {
                            alpha(p.destructive, 0.1)
                        } else {
                            p.muted
                        })),
                        text_color: Some(if error { p.destructive } else { p.foreground }),
                        // `rounded-lg`, `rounded-md` when `xs`
                        border: border::rounded(if size == Size::Xs {
                            t.radius_md()
                        } else {
                            t.radius_lg()
                        }),
                        ..container::Style::default()
                    }
                })
        });

        // `leading-tight`
        let content = has_content.then(|| {
            Column::new()
                .push(attachment.title.map(|title| {
                    text(title)
                        .size(size.text_size())
                        .font(font::medium())
                        .wrapping(text::Wrapping::None)
                        .line_height(1.25)
                }))
                .push(attachment.description.map(|description| {
                    // `mt-0.5 text-xs`
                    text(description)
                        .size(style::text::XS)
                        .line_height(1.25)
                        .wrapping(text::Wrapping::None)
                        .style(move |theme: &Theme| {
                            let p = tokens(theme).palette;
                            text::Style {
                                color: Some(if state == State::Error {
                                    alpha(p.destructive, 0.8)
                                } else {
                                    p.muted_foreground
                                }),
                            }
                        })
                }))
                .spacing(2)
                .clip(true)
        });

        let actions = (!attachment.actions.is_empty()).then(|| {
            Row::with_children(attachment.actions)
                .spacing(if vertical { 4 } else { 0 })
                .align_y(Alignment::Center)
        });

        let (body, actions): (Element<'a, Message>, _) = if vertical {
            (
                Column::new()
                    .push(media)
                    // `px-1`
                    .push(content.map(|content| container(content).padding([0, 4])))
                    .spacing(size.gap())
                    .width(Length::Fill)
                    .into(),
                actions,
            )
        } else {
            (
                Row::new()
                    .push(media)
                    .push(content.map(|content| content.width(Length::Fill)))
                    .push(actions)
                    .spacing(size.gap())
                    .align_y(Alignment::Center)
                    .into(),
                None,
            )
        };

        let width = attachment.width.unwrap_or(if vertical {
            Length::Fixed(tile_width)
        } else {
            Length::Shrink
        });

        let body = container(body).padding(padding).width(width);

        let card: Element<'a, Message> = match attachment.on_press {
            Some(message) => Pressable::new(body)
                .on_press(message)
                .style(move |theme, status| {
                    let style = self::style(theme, state, size, status);
                    iced::widget::button::Style {
                        background: style.background,
                        text_color: style.text_color.unwrap_or_default(),
                        border: style.border,
                        ..iced::widget::button::Style::default()
                    }
                })
                .into(),
            None => body
                .style(move |theme| self::style(theme, state, size, Status::Active))
                .into(),
        };

        match actions {
            // `absolute top-3 right-3`, inside the border.
            Some(actions) => Stack::new()
                .push(card)
                .push(
                    container(actions)
                        .align_right(Length::Fill)
                        .padding(Padding::new(13.0)),
                )
                .into(),
            None => card,
        }
    }
}

/// The style of an attachment.
pub fn style(theme: &Theme, state: State, size: Size, status: Status) -> container::Style {
    let t = tokens(theme);
    let p = &t.palette;
    let hovered = matches!(status, Status::Hovered | Status::Pressed);

    container::Style {
        // `has-[>a,>button]:hover:bg-muted/50`
        background: Some(Background::Color(if hovered {
            shadcn_iced_core::theme::over(alpha(p.muted, 0.5), p.card)
        } else {
            p.card
        })),
        text_color: Some(p.card_foreground),
        border: Border {
            color: match state {
                State::Error => alpha(p.destructive, 0.3),
                _ => p.border,
            },
            width: 1.0,
            // `rounded-xl`, `rounded-lg` when `xs`
            radius: border::radius(if size == Size::Xs {
                t.radius_lg()
            } else {
                t.radius_xl()
            }),
        },
        ..container::Style::default()
    }
}

/// A horizontally scrolling row of attachments (`AttachmentGroup`,
/// `gap-3 py-1`).
pub fn attachment_group<'a, Message: 'a>(
    attachments: impl IntoIterator<Item = Element<'a, Message>>,
) -> Element<'a, Message> {
    iced::widget::scrollable(Row::with_children(attachments).spacing(12).padding([4, 0]))
        .direction(iced::widget::scrollable::Direction::Horizontal(
            iced::widget::scrollable::Scrollbar::hidden(),
        ))
        .width(Length::Fill)
        .into()
}

/// A small ghost icon button for attachments (`AttachmentAction`,
/// `size="icon-xs"`).
pub fn action<'a, Message: Clone + 'a>(icon: Icon, on_press: Message) -> Element<'a, Message> {
    super::button::icon_button(icon)
        .variant(super::button::Variant::Ghost)
        .size(super::button::Size::IconXs)
        .on_press(on_press)
        .into()
}

/// A remove [`action`] with an `X` icon.
pub fn remove_action<'a, Message: Clone + 'a>(on_press: Message) -> Element<'a, Message> {
    action(lucide::X, on_press)
}
