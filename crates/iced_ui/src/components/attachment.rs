//! A file attached to a message: an icon or a thumbnail, a name, a size and
//! actions.
//!
//! Port of <https://ui.shadcn.com/docs/components/attachment>.
//!
//! ```no_run
//! # use iced_ui::attachment::{attachment, State};
//! # use iced_ui_core::icon::lucide;
//! # fn view<'a>() -> iced::Element<'a, ()> {
//! attachment("report.pdf").icon(lucide::FILE_TEXT).description("2.4 MB").into()
//! # }
//! ```
use iced_ui_core::icon::{Icon, lucide};
use iced_ui_core::theme::alpha;
use iced_ui_core::{Element, font, style, tokens};

use iced::widget::{Column, Row, Stack, container, image, text};
use iced::{Alignment, Background, Border, Length, Padding, Theme, border};

/// The state of an [`Attachment`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum State {
    /// Not uploaded yet: dashed-looking border.
    Idle,
    /// Uploading: dimmed media.
    Uploading,
    /// Processing: dimmed media.
    Processing,
    /// Ready.
    #[default]
    Done,
    /// Failed: destructive border and description.
    Error,
}

/// The size of an [`Attachment`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Size {
    /// 40x40 media, `text-sm`.
    #[default]
    Default,
    /// 32x32 media, `text-xs`.
    Sm,
    /// 28x28 media, `text-xs`.
    Xs,
}

enum Media {
    Icon(Icon),
    Image(image::Handle),
}

/// An attachment. See the [module documentation](self).
pub struct Attachment<'a, Message> {
    title: String,
    description: Option<String>,
    media: Option<Media>,
    state: State,
    size: Size,
    vertical: bool,
    actions: Vec<Element<'a, Message>>,
}

/// Creates an [`Attachment`] with a title (usually the file name).
pub fn attachment<'a, Message: 'a>(title: impl Into<String>) -> Attachment<'a, Message> {
    Attachment {
        title: title.into(),
        description: None,
        media: None,
        state: State::Done,
        size: Size::Default,
        vertical: false,
        actions: Vec::new(),
    }
}

impl<'a, Message: 'a> Attachment<'a, Message> {
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

    /// Shows an image as media.
    pub fn image(mut self, handle: impl Into<image::Handle>) -> Self {
        self.media = Some(Media::Image(handle.into()));
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

    /// Lays the attachment out vertically, as a tile.
    pub fn vertical(mut self, vertical: bool) -> Self {
        self.vertical = vertical;
        self
    }

    /// Appends an action, e.g. a remove button.
    pub fn action(mut self, action: impl Into<Element<'a, Message>>) -> Self {
        self.actions.push(action.into());
        self
    }
}

impl<'a, Message: 'a> From<Attachment<'a, Message>> for Element<'a, Message> {
    fn from(attachment: Attachment<'a, Message>) -> Self {
        let state = attachment.state;
        let size = attachment.size;
        let dimmed = matches!(state, State::Uploading | State::Processing);

        let (media_size, text_size, gap, padding) = match size {
            Size::Default => (40.0, style::text::SM, 8.0, 8.0),
            Size::Sm => (32.0, style::text::XS, 10.0, 6.0),
            Size::Xs => (28.0, style::text::XS, 6.0, 4.0),
        };

        let media = attachment.media.map(|media| -> Element<'a, Message> {
            let inner: Element<'a, Message> = match media {
                Media::Icon(icon) => icon.sized(media_size * 0.45).into(),
                Media::Image(handle) => image(handle)
                    .width(media_size)
                    .height(media_size)
                    .content_fit(iced::ContentFit::Cover)
                    .opacity(if dimmed { 0.6_f32 } else { 1.0_f32 })
                    .into(),
            };

            let tile = container(inner)
                .center(if attachment.vertical {
                    Length::Fill
                } else {
                    Length::Fixed(media_size)
                })
                .height(if attachment.vertical {
                    Length::Fixed(80.0)
                } else {
                    Length::Fixed(media_size)
                })
                .style(move |theme: &Theme| {
                    let t = tokens(theme);
                    container::Style {
                        background: Some(Background::Color(t.palette.muted)),
                        text_color: Some(alpha(
                            t.palette.foreground,
                            if dimmed { 0.6 } else { 1.0 },
                        )),
                        border: border::rounded(if size == Size::Xs {
                            t.radius_md()
                        } else {
                            t.radius_lg()
                        }),
                        ..container::Style::default()
                    }
                });

            if dimmed {
                Stack::new()
                    .push(tile)
                    .push(container(super::spinner::spinner().size(16.0)).center(Length::Fill))
                    .into()
            } else {
                tile.into()
            }
        });

        let content = Column::new()
            .push(
                text(attachment.title)
                    .size(text_size)
                    .font(font::MEDIUM)
                    .wrapping(text::Wrapping::None)
                    .line_height(1.25),
            )
            .push(attachment.description.map(|description| {
                text(description)
                    .size(style::text::XS)
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
            .spacing(2);

        let actions = (!attachment.actions.is_empty()).then(|| {
            Row::with_children(attachment.actions)
                .spacing(4)
                .align_y(Alignment::Center)
        });

        let body: Element<'a, Message> = if attachment.vertical {
            Column::new()
                .push(media)
                .push(container(content).padding([0, 4]))
                .push(actions)
                .spacing(gap)
                .width(120)
                .into()
        } else {
            Row::new()
                .push(media)
                .push(container(content).width(Length::Shrink))
                .push(actions)
                .spacing(gap)
                .align_y(Alignment::Center)
                .into()
        };

        container(body)
            .padding(Padding::new(padding).right(padding + 2.0))
            .style(move |theme: &Theme| {
                let t = tokens(theme);
                let p = &t.palette;

                container::Style {
                    background: Some(Background::Color(p.card)),
                    text_color: Some(p.card_foreground),
                    border: Border {
                        color: match state {
                            State::Error => alpha(p.destructive, 0.3),
                            State::Idle => alpha(p.border, 1.0),
                            _ => p.border,
                        },
                        width: 1.0,
                        radius: border::radius(if size == Size::Xs {
                            t.radius_lg()
                        } else {
                            t.radius_xl()
                        }),
                    },
                    ..container::Style::default()
                }
            })
            .into()
    }
}

/// A horizontally scrolling row of attachments (`AttachmentGroup`).
pub fn attachment_group<'a, Message: 'a>(
    attachments: impl IntoIterator<Item = Element<'a, Message>>,
) -> Element<'a, Message> {
    iced::widget::scrollable(Row::with_children(attachments).spacing(12).padding([4, 0]))
        .direction(iced::widget::scrollable::Direction::Horizontal(
            iced::widget::scrollable::Scrollbar::hidden(),
        ))
        .into()
}

/// A small remove button for attachments (`AttachmentAction`).
pub fn remove_action<'a, Message: Clone + 'a>(on_press: Message) -> Element<'a, Message> {
    super::button::icon_button(lucide::X)
        .variant(super::button::Variant::Ghost)
        .size(super::button::Size::IconXs)
        .on_press(on_press)
        .into()
}
