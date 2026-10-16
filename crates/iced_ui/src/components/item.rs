//! A versatile component for displaying content with media, title,
//! description and actions.
//!
//! Port of <https://ui.shadcn.com/docs/components/item>.
use iced_ui_core::icon::Icon;
use iced_ui_core::primitive::Pressable;
use iced_ui_core::theme::alpha;
use iced_ui_core::{Element, font, style, tokens};

use iced::widget::button::{Status, Style};
use iced::widget::{Column, Row, container, text};
use iced::{Alignment, Background, Color, Length, Theme};

/// The visual style of an [`Item`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Variant {
    /// Transparent.
    #[default]
    Default,
    /// Bordered.
    Outline,
    /// Muted background.
    Muted,
}

/// The size of an [`Item`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Size {
    /// `gap-4 p-4`
    #[default]
    Default,
    /// `gap-2.5 px-4 py-3`
    Sm,
}

/// An item. See the [module documentation](self).
pub struct Item<'a, Message> {
    media: Option<Element<'a, Message>>,
    title: Option<Element<'a, Message>>,
    description: Option<String>,
    actions: Vec<Element<'a, Message>>,
    variant: Variant,
    size: Size,
    on_press: Option<Message>,
}

/// Creates an [`Item`] with a title.
pub fn item<'a, Message: Clone + 'a>(title: impl text::IntoFragment<'a>) -> Item<'a, Message> {
    Item {
        media: None,
        title: Some(
            text(title)
                .size(style::text::SM)
                .font(font::MEDIUM)
                .line_height(1.375)
                .into(),
        ),
        description: None,
        actions: Vec::new(),
        variant: Variant::Default,
        size: Size::Default,
        on_press: None,
    }
}

impl<'a, Message: Clone + 'a> Item<'a, Message> {
    /// Shows an icon in a bordered muted square (`variant="icon"`).
    pub fn icon(mut self, icon: Icon) -> Self {
        self.media = Some(
            container(icon.sized(16))
                .center(32)
                .style(|theme: &Theme| {
                    let t = tokens(theme);
                    container::Style {
                        background: Some(Background::Color(t.palette.muted)),
                        border: style::border(t.palette.border, t.radius_sm()),
                        ..container::Style::default()
                    }
                })
                .into(),
        );
        self
    }

    /// Shows custom media (avatar, image...).
    pub fn media(mut self, media: impl Into<Element<'a, Message>>) -> Self {
        self.media = Some(media.into());
        self
    }

    /// Sets the description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Appends an action, displayed on the right.
    pub fn action(mut self, action: impl Into<Element<'a, Message>>) -> Self {
        self.actions.push(action.into());
        self
    }

    /// Sets the variant.
    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets the size.
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// Makes the whole item clickable, like an `<a>` item.
    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }
}

impl<'a, Message: Clone + 'a> From<Item<'a, Message>> for Element<'a, Message> {
    fn from(item: Item<'a, Message>) -> Self {
        let content = Column::new()
            .push(item.title)
            .push(item.description.map(|description| {
                text(description)
                    .size(style::text::SM)
                    .line_height(1.5)
                    .style(|theme: &Theme| text::Style {
                        color: Some(tokens(theme).palette.muted_foreground),
                    })
            }))
            .spacing(4)
            .width(Length::Fill);

        let (gap, padding) = match item.size {
            Size::Default => (16.0, [16.0, 16.0]),
            Size::Sm => (10.0, [12.0, 16.0]),
        };

        let row = Row::new()
            .push(item.media)
            .push(content)
            .push((!item.actions.is_empty()).then(|| {
                Row::with_children(item.actions).spacing(8).align_y(Alignment::Center)
            }))
            .spacing(gap)
            .align_y(Alignment::Center);

        let variant = item.variant;
        let clickable = item.on_press.is_some();

        Pressable::new(row)
            .on_press_maybe(item.on_press)
            .padding(padding)
            .width(Length::Fill)
            .interaction(if clickable {
                iced::mouse::Interaction::Pointer
            } else {
                iced::mouse::Interaction::Idle
            })
            .style(move |theme, status| self::style(theme, status, variant, clickable))
            .into()
    }
}

/// The style of an item.
pub fn style(theme: &Theme, status: Status, variant: Variant, clickable: bool) -> Style {
    let t = tokens(theme);
    let p = &t.palette;
    let hovered = clickable && matches!(status, Status::Hovered | Status::Pressed);

    let background = match variant {
        Variant::Muted => Some(alpha(p.muted, 0.5)),
        _ => None,
    };

    Style {
        background: if hovered {
            Some(Background::Color(alpha(p.accent, 0.5)))
        } else {
            background.map(Background::Color)
        },
        text_color: p.foreground,
        border: style::border(
            match variant {
                Variant::Outline => p.border,
                _ => Color::TRANSPARENT,
            },
            t.radius_md(),
        ),
        ..Style::default()
    }
}
