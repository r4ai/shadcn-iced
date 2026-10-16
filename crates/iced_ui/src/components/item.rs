//! A versatile component for displaying content with media, title,
//! description and actions.
//!
//! Port of <https://ui.shadcn.com/docs/components/item>.
//!
//! ```no_run
//! # use iced_ui::item::{item, Variant};
//! # use iced_ui::prelude::lucide;
//! # fn view<'a>() -> iced::Element<'a, ()> {
//! item("Security Alert")
//!     .icon(lucide::SHIELD_ALERT)
//!     .description("New login detected from unknown device.")
//!     .variant(Variant::Outline)
//!     .into()
//! # }
//! ```
use iced_ui_core::icon::Icon;
use iced_ui_core::primitive::Pressable;
use iced_ui_core::theme::alpha;
use iced_ui_core::{Element, font, style, tokens};

use iced::widget::button::{Status, Style};
use iced::widget::{Column, Row, container, text};
use iced::{Alignment, Background, Color, Length, Padding, Theme};

/// The visual style of an [`Item`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Variant {
    /// `border-transparent`
    #[default]
    Default,
    /// `border-border`
    Outline,
    /// `bg-muted/50 border-transparent`
    Muted,
}

/// The size of an [`Item`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Size {
    /// `gap-2.5 px-3 py-2.5`
    #[default]
    Default,
    /// `gap-2.5 px-3 py-2.5`, with smaller images.
    Sm,
    /// `gap-2 px-2.5 py-2`, with an `text-xs` description.
    Xs,
}

impl Size {
    fn gap(self) -> f32 {
        match self {
            Size::Default | Size::Sm => 10.0,
            Size::Xs => 8.0,
        }
    }

    /// The padding, plus the 1px border.
    fn padding(self) -> Padding {
        match self {
            Size::Default | Size::Sm => Padding::from([11.0, 13.0]),
            Size::Xs => Padding::from([9.0, 11.0]),
        }
    }

    /// The size of an image media (`size-10`, `size-8`, `size-6`).
    pub fn image_size(self) -> f32 {
        match self {
            Size::Default => 40.0,
            Size::Sm => 32.0,
            Size::Xs => 24.0,
        }
    }
}

enum Media<'a, Message> {
    Default(Element<'a, Message>),
    Icon(Icon),
    Image(Element<'a, Message>),
}

/// An item. See the [module documentation](self).
pub struct Item<'a, Message> {
    media: Option<Media<'a, Message>>,
    title: Option<Element<'a, Message>>,
    description: Option<String>,
    actions: Vec<Element<'a, Message>>,
    header: Option<Element<'a, Message>>,
    footer: Option<Element<'a, Message>>,
    variant: Variant,
    size: Size,
    on_press: Option<Message>,
}

/// Creates an [`Item`] with a title (`text-sm leading-snug font-medium`).
pub fn item<'a, Message: Clone + 'a>(title: impl text::IntoFragment<'a>) -> Item<'a, Message> {
    Item {
        media: None,
        title: Some(
            text(title)
                .size(style::text::SM)
                .font(font::medium())
                .line_height(1.375)
                .into(),
        ),
        description: None,
        actions: Vec::new(),
        header: None,
        footer: None,
        variant: Variant::Default,
        size: Size::Default,
        on_press: None,
    }
}

impl<'a, Message: Clone + 'a> Item<'a, Message> {
    /// Shows an icon (`ItemMedia variant="icon"`, `size-4`).
    pub fn icon(mut self, icon: Icon) -> Self {
        self.media = Some(Media::Icon(icon));
        self
    }

    /// Shows custom media, e.g. an avatar (`ItemMedia`).
    pub fn media(mut self, media: impl Into<Element<'a, Message>>) -> Self {
        self.media = Some(Media::Default(media.into()));
        self
    }

    /// Shows an image, cropped to a rounded square
    /// (`ItemMedia variant="image"`).
    pub fn image(mut self, image: impl Into<Element<'a, Message>>) -> Self {
        self.media = Some(Media::Image(image.into()));
        self
    }

    /// Sets the description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Appends an action, displayed on the right (`ItemActions`).
    pub fn action(mut self, action: impl Into<Element<'a, Message>>) -> Self {
        self.actions.push(action.into());
        self
    }

    /// Sets a full-width header above the item's content (`ItemHeader`).
    pub fn header(mut self, header: impl Into<Element<'a, Message>>) -> Self {
        self.header = Some(header.into());
        self
    }

    /// Sets a full-width footer below the item's content (`ItemFooter`).
    pub fn footer(mut self, footer: impl Into<Element<'a, Message>>) -> Self {
        self.footer = Some(footer.into());
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
        let size = item.size;
        let has_description = item.description.is_some();

        // `gap-1 group-data-[size=xs]/item:gap-0`
        let content = Column::new()
            .push(item.title)
            .push(item.description.map(|description| {
                // `text-muted-foreground text-sm leading-normal`
                text(description)
                    .size(if size == Size::Xs {
                        style::text::XS
                    } else {
                        style::text::SM
                    })
                    .line_height(1.5)
                    .style(|theme: &Theme| text::Style {
                        color: Some(tokens(theme).palette.muted_foreground),
                    })
            }))
            .spacing(if size == Size::Xs { 0 } else { 4 })
            .width(Length::Fill);

        let media = item.media.map(|media| -> Element<'a, Message> {
            let media: Element<'a, Message> = match media {
                Media::Default(element) => element,
                Media::Icon(icon) => icon.sized(16).into(),
                // `size-10 overflow-hidden rounded-sm`
                Media::Image(image) => container(image)
                    .width(size.image_size())
                    .height(size.image_size())
                    .clip(true)
                    .style(|theme: &Theme| container::Style {
                        border: style::rounded(tokens(theme).radius_sm()),
                        ..container::Style::default()
                    })
                    .into(),
            };

            // `group-has-data-[slot=item-description]/item:translate-y-0.5
            // self-start`
            if has_description {
                container(media).padding(Padding::ZERO.top(2)).into()
            } else {
                media
            }
        });

        // The content and actions stay centered (`items-center`), while the
        // media sticks to the top when there is a description.
        let main = Row::new()
            .push(content)
            .push((!item.actions.is_empty()).then(|| {
                Row::with_children(item.actions)
                    .spacing(8)
                    .align_y(Alignment::Center)
            }))
            .spacing(size.gap())
            .align_y(Alignment::Center);

        let row = Row::new()
            .push(media)
            .push(main)
            .spacing(size.gap())
            .align_y(if has_description {
                Alignment::Start
            } else {
                Alignment::Center
            });

        let body: Element<'a, Message> = if item.header.is_some() || item.footer.is_some() {
            Column::new()
                .push(item.header)
                .push(row)
                .push(item.footer)
                .spacing(size.gap())
                .into()
        } else {
            row.into()
        };

        let variant = item.variant;
        let clickable = item.on_press.is_some();

        Pressable::new(body)
            .on_press_maybe(item.on_press)
            .padding(size.padding())
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

/// Stacks items (`ItemGroup`, `flex flex-col gap-4`), closer together
/// when they are small (`gap-2.5` for `sm`, `gap-2` for `xs`).
pub fn item_group<'a, Message: Clone + 'a>(
    items: impl IntoIterator<Item = Item<'a, Message>>,
) -> Element<'a, Message> {
    let items: Vec<Item<'a, Message>> = items.into_iter().collect();

    let spacing = if items.iter().any(|item| item.size == Size::Xs) {
        8.0
    } else if items.iter().any(|item| item.size == Size::Sm) {
        10.0
    } else {
        16.0
    };

    Column::with_children(items.into_iter().map(Element::from))
        .spacing(spacing)
        .width(Length::Fill)
        .into()
}

/// The style of an item (`rounded-lg border`, `[a]:hover:bg-muted`).
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
            Some(Background::Color(p.muted))
        } else {
            background.map(Background::Color)
        },
        text_color: p.foreground,
        border: style::border(
            match variant {
                Variant::Outline => p.border,
                _ => Color::TRANSPARENT,
            },
            t.radius_lg(),
        ),
        ..Style::default()
    }
}
