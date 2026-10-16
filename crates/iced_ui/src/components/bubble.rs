//! A chat bubble.
//!
//! Port of <https://ui.shadcn.com/docs/components/bubble>.
//!
//! ```no_run
//! # use iced_ui::bubble::{bubble, Align, Variant};
//! # fn view<'a>() -> iced::Element<'a, ()> {
//! bubble("Hey! How is it going?").variant(Variant::Muted).align(Align::End).into()
//! # }
//! ```
use iced_ui_core::theme::{alpha, mix};
use iced_ui_core::{Element, style, tokens};

use iced::widget::{Column, Row, Stack, container, text};
use iced::{Alignment, Background, Border, Color, Length, Padding, Theme, border};

/// The visual style of a [`Bubble`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Variant {
    /// Primary color.
    #[default]
    Default,
    /// Secondary color.
    Secondary,
    /// Muted background.
    Muted,
    /// A light tint of the primary color.
    Tinted,
    /// Bordered.
    Outline,
    /// No background nor padding.
    Ghost,
    /// Destructive tint.
    Destructive,
}

/// The side a [`Bubble`] is aligned to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Align {
    /// Left, for incoming messages.
    #[default]
    Start,
    /// Right, for outgoing messages.
    End,
}

/// A chat bubble. See the [module documentation](self).
pub struct Bubble<'a, Message> {
    content: Element<'a, Message>,
    variant: Variant,
    align: Align,
    reactions: Option<Element<'a, Message>>,
    max_width: f32,
}

/// Creates a [`Bubble`] with some text.
pub fn bubble<'a, Message: 'a>(content: impl text::IntoFragment<'a>) -> Bubble<'a, Message> {
    Bubble::new(text(content).size(style::text::SM).line_height(1.625))
}

impl<'a, Message: 'a> Bubble<'a, Message> {
    /// Creates a [`Bubble`] with arbitrary content.
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
            variant: Variant::Default,
            align: Align::Start,
            reactions: None,
            max_width: 420.0,
        }
    }

    /// Sets the variant.
    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets the alignment.
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    /// Shows reactions (e.g. emoji) overlapping the bottom of the bubble
    /// (`BubbleReactions`).
    pub fn reactions(mut self, reactions: impl text::IntoFragment<'a>) -> Self {
        self.reactions = Some(
            container(text(reactions).size(style::text::SM).line_height(iced::Pixels(20.0)))
                .padding([2, 8])
                .style(|theme: &Theme| {
                    let p = tokens(theme).palette;
                    container::Style {
                        background: Some(Background::Color(p.muted)),
                        text_color: Some(p.foreground),
                        border: Border {
                            color: p.background,
                            width: 2.0,
                            radius: border::radius(f32::MAX),
                        },
                        ..container::Style::default()
                    }
                })
                .into(),
        );
        self
    }

    /// Sets the maximum width of the bubble.
    pub fn max_width(mut self, width: f32) -> Self {
        self.max_width = width;
        self
    }
}

impl<'a, Message: 'a> From<Bubble<'a, Message>> for Element<'a, Message> {
    fn from(bubble: Bubble<'a, Message>) -> Self {
        let variant = bubble.variant;
        let padding = if variant == Variant::Ghost { Padding::ZERO } else { Padding::from([8, 12]) };

        let content = container(bubble.content)
            .padding(padding)
            .max_width(bubble.max_width)
            .style(move |theme| self::style(theme, variant));

        let align = match bubble.align {
            Align::Start => Alignment::Start,
            Align::End => Alignment::End,
        };

        let body: Element<'a, Message> = match bubble.reactions {
            Some(reactions) => Stack::new()
                .push(Column::new().push(content).push(iced::widget::space().height(14)).align_x(align))
                .push(
                    container(reactions)
                        .align_x(align)
                        .align_bottom(Length::Fill)
                        .width(Length::Fill)
                        .padding(Padding::ZERO.left(8).right(8)),
                )
                .into(),
            None => content.into(),
        };

        Row::new()
            .push((bubble.align == Align::End).then(iced::widget::space::horizontal))
            .push(body)
            .width(Length::Fill)
            .into()
    }
}

/// The style of the content of a bubble.
pub fn style(theme: &Theme, variant: Variant) -> container::Style {
    let t = tokens(theme);
    let p = &t.palette;
    let dark = t.is_dark();

    let (background, text_color, border_color) = match variant {
        Variant::Default => (Some(p.primary), p.primary_foreground, Color::TRANSPARENT),
        Variant::Secondary => (Some(p.secondary), p.secondary_foreground, Color::TRANSPARENT),
        Variant::Muted => (Some(p.muted), p.foreground, Color::TRANSPARENT),
        Variant::Tinted => (
            Some(if dark { mix(p.primary, Color::BLACK, 0.7) } else { mix(p.primary, Color::WHITE, 0.85) }),
            p.foreground,
            Color::TRANSPARENT,
        ),
        Variant::Outline => (Some(p.background), p.foreground, p.border),
        Variant::Ghost => (None, p.foreground, Color::TRANSPARENT),
        Variant::Destructive => (
            Some(alpha(p.destructive, if dark { 0.2 } else { 0.1 })),
            p.destructive,
            Color::TRANSPARENT,
        ),
    };

    container::Style {
        background: background.map(Background::Color),
        text_color: Some(text_color),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: border::radius(if variant == Variant::Ghost { 0.0 } else { t.radius_xl() }),
        },
        ..container::Style::default()
    }
}
