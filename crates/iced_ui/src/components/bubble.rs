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
use iced_ui_core::primitive::Pressable;
use iced_ui_core::theme::{alpha, mix};
use iced_ui_core::{Element, style, tokens};

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Renderer as _};
use iced::advanced::widget::{Operation, Tree, Widget};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::widget::button::Status;
use iced::widget::{Column, container, text};
use iced::{
    Background, Border, Color, Event, Length, Padding, Point, Rectangle, Size, Theme, Vector,
    border, mouse,
};

/// The visual style of a [`Bubble`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Variant {
    /// `bg-primary text-primary-foreground`
    #[default]
    Default,
    /// `bg-secondary text-secondary-foreground`
    Secondary,
    /// `bg-muted`
    Muted,
    /// A light tint of the primary color
    /// (`oklch(from var(--primary) 0.93 calc(c*0.4) h)`).
    Tinted,
    /// `bg-background border-border`
    Outline,
    /// No background nor padding, full width.
    Ghost,
    /// `bg-destructive/10 text-destructive`
    Destructive,
}

/// The side a [`Bubble`] (or its reactions) is aligned to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Align {
    /// Left, for incoming messages.
    #[default]
    Start,
    /// Right, for outgoing messages.
    End,
}

/// The edge of the bubble the reactions overlap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Side {
    /// `top-0 -translate-y-3/4`
    Top,
    /// `bottom-0 translate-y-3/4`
    #[default]
    Bottom,
}

/// A chat bubble. See the [module documentation](self).
pub struct Bubble<'a, Message> {
    content: Element<'a, Message>,
    variant: Variant,
    align: Align,
    reactions: Option<Element<'a, Message>>,
    reactions_side: Side,
    reactions_align: Align,
    max_width: Option<f32>,
    on_press: Option<Message>,
}

/// Creates a [`Bubble`] with some text (`text-sm leading-relaxed`).
pub fn bubble<'a, Message: Clone + 'a>(
    content: impl text::IntoFragment<'a>,
) -> Bubble<'a, Message> {
    Bubble::new(text(content).size(style::text::SM).line_height(1.625))
}

impl<'a, Message: Clone + 'a> Bubble<'a, Message> {
    /// Creates a [`Bubble`] with arbitrary content (`BubbleContent`).
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
            variant: Variant::Default,
            align: Align::Start,
            reactions: None,
            reactions_side: Side::Bottom,
            reactions_align: Align::End,
            max_width: None,
            on_press: None,
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
    /// (`BubbleReactions`). Separate several reactions with spaces.
    pub fn reactions(self, reactions: impl text::IntoFragment<'a>) -> Self {
        let reactions = text(reactions)
            .size(style::text::SM)
            .line_height(iced::Pixels(20.0));

        // `px-1.5 py-0.5`
        self.reactions_content(container(reactions).padding([2, 6]))
    }

    /// Shows arbitrary reactions, e.g. a small button
    /// (`has-[button]:p-0`).
    pub fn reactions_content(mut self, reactions: impl Into<Element<'a, Message>>) -> Self {
        self.reactions = Some(
            container(reactions)
                .style(|theme: &Theme| {
                    let p = tokens(theme).palette;
                    container::Style {
                        background: Some(Background::Color(p.muted)),
                        text_color: Some(p.foreground),
                        border: border::rounded(f32::MAX),
                        ..container::Style::default()
                    }
                })
                .into(),
        );
        self
    }

    /// Sets the edge the reactions overlap.
    pub fn reactions_side(mut self, side: Side) -> Self {
        self.reactions_side = side;
        self
    }

    /// Sets the side the reactions are aligned to (`end` by default).
    pub fn reactions_align(mut self, align: Align) -> Self {
        self.reactions_align = align;
        self
    }

    /// Caps the width of the bubble in pixels, on top of `max-w-[80%]`.
    pub fn max_width(mut self, width: f32) -> Self {
        self.max_width = Some(width);
        self
    }

    /// Whether the bubble is a [`Variant::Ghost`] bubble, which removes the
    /// padding of the header and footer of a [`message`](super::message).
    pub fn is_ghost(&self) -> bool {
        self.variant == Variant::Ghost
    }

    /// Makes the bubble interactive (`BubbleContent render={<button />}`),
    /// highlighted on hover.
    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }
}

impl<'a, Message: Clone + 'a> From<Bubble<'a, Message>> for Element<'a, Message> {
    fn from(bubble: Bubble<'a, Message>) -> Self {
        let variant = bubble.variant;
        // `px-3 py-2` inside a 1px border; ghost bubbles have neither.
        let padding = if variant == Variant::Ghost {
            Padding::ZERO
        } else {
            Padding::from([9, 13])
        };

        let mut content = container(bubble.content).padding(padding);

        if let Some(max_width) = bubble.max_width {
            content = content.max_width(max_width);
        }

        let content: Element<'a, Message> = match bubble.on_press {
            Some(message) => Pressable::new(content)
                .on_press(message)
                .style(move |theme, status| {
                    let style = self::style(theme, variant, status);
                    iced::widget::button::Style {
                        background: style.background,
                        text_color: style.text_color.unwrap_or(Color::BLACK),
                        border: style.border,
                        ..iced::widget::button::Style::default()
                    }
                })
                .into(),
            None => content
                .style(move |theme| self::style(theme, variant, Status::Active))
                .into(),
        };

        Element::new(Frame {
            children: std::iter::once(content).chain(bubble.reactions).collect(),
            align: bubble.align,
            // `max-w-[80%] data-[variant=ghost]:max-w-full`
            ratio: if variant == Variant::Ghost { 1.0 } else { 0.8 },
            reactions_side: bubble.reactions_side,
            reactions_align: bubble.reactions_align,
        })
    }
}

/// Stacks bubbles of the same author (`BubbleGroup`, `gap-2`).
pub fn bubble_group<'a, Message: 'a>(
    bubbles: impl IntoIterator<Item = Element<'a, Message>>,
) -> Column<'a, Message> {
    Column::with_children(bubbles)
        .spacing(8)
        .width(Length::Fill)
}

/// The style of the content of a bubble.
pub fn style(theme: &Theme, variant: Variant, status: Status) -> container::Style {
    let t = tokens(theme);
    let p = &t.palette;
    let dark = t.is_dark();
    let hovered = matches!(status, Status::Hovered | Status::Pressed);

    let (background, text_color, border_color) = match variant {
        Variant::Default => (
            Some(if hovered {
                alpha(p.primary, 0.8)
            } else {
                p.primary
            }),
            p.primary_foreground,
            Color::TRANSPARENT,
        ),
        Variant::Secondary => (
            Some(if hovered {
                mix(p.secondary, p.foreground, 0.05)
            } else {
                p.secondary
            }),
            p.secondary_foreground,
            Color::TRANSPARENT,
        ),
        Variant::Muted => (
            Some(if hovered {
                mix(p.muted, p.foreground, 0.05)
            } else {
                p.muted
            }),
            p.foreground,
            Color::TRANSPARENT,
        ),
        Variant::Tinted => (
            Some(match (dark, hovered) {
                (false, false) => relative_oklch(p.primary, 0.93, 0.4),
                (false, true) => relative_oklch(p.primary, 0.88, 0.5),
                (true, false) => relative_oklch(p.primary, 0.3, 0.4),
                (true, true) => relative_oklch(p.primary, 0.35, 0.5),
            }),
            p.foreground,
            Color::TRANSPARENT,
        ),
        Variant::Outline => (
            Some(match (dark, hovered) {
                (_, false) => p.background,
                (false, true) => p.muted,
                (true, true) => alpha(p.input, 0.3),
            }),
            p.foreground,
            p.border,
        ),
        Variant::Ghost => (
            hovered.then(|| if dark { alpha(p.muted, 0.5) } else { p.muted }),
            p.foreground,
            Color::TRANSPARENT,
        ),
        Variant::Destructive => (
            Some(alpha(
                p.destructive,
                match (dark, hovered) {
                    (false, false) => 0.1,
                    (false, true) | (true, false) => 0.2,
                    (true, true) => 0.3,
                },
            )),
            p.destructive,
            Color::TRANSPARENT,
        ),
    };

    container::Style {
        background: background.map(Background::Color),
        text_color: Some(text_color),
        border: Border {
            color: border_color,
            width: if variant == Variant::Ghost { 0.0 } else { 1.0 },
            radius: border::radius(if variant == Variant::Ghost {
                0.0
            } else {
                t.radius_xl()
            }),
        },
        ..container::Style::default()
    }
}

/// `oklch(from color l calc(c * chroma) h)`.
fn relative_oklch(color: Color, lightness: f32, chroma: f32) -> Color {
    let linear = |c: f32| {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    let gamma = |c: f32| {
        let c = c.clamp(0.0, 1.0);
        if c <= 0.003_130_8 {
            c * 12.92
        } else {
            1.055 * c.powf(1.0 / 2.4) - 0.055
        }
    };

    let (r, g, b) = (linear(color.r), linear(color.g), linear(color.b));

    let l = (0.412_221_47 * r + 0.536_332_55 * g + 0.051_445_995 * b).cbrt();
    let m = (0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b).cbrt();
    let s = (0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b).cbrt();

    let a = 1.977_998_5 * l - 2.428_592_2 * m + 0.450_593_7 * s;
    let bb = 0.025_904_037 * l + 0.782_771_77 * m - 0.808_675_77 * s;

    let (a, bb) = (a * chroma, bb * chroma);
    let ll = lightness;

    let l = (ll + 0.396_337_78 * a + 0.215_803_76 * bb).powi(3);
    let m = (ll - 0.105_561_346 * a - 0.063_854_17 * bb).powi(3);
    let s = (ll - 0.089_484_18 * a - 1.291_485_5 * bb).powi(3);

    Color {
        r: gamma(4.076_741_7 * l - 3.307_711_6 * m + 0.230_969_94 * s),
        g: gamma(-1.268_438 * l + 2.609_757_4 * m - 0.341_319_38 * s),
        b: gamma(-0.004_196_086_3 * l - 0.703_418_6 * m + 1.707_614_7 * s),
        a: color.a,
    }
}

/// Lays the content out at most `ratio` of the available width, on the
/// side of `align`, and positions the reactions over its edge without
/// taking space (`absolute`), like CSS.
struct Frame<'a, Message> {
    children: Vec<Element<'a, Message>>,
    align: Align,
    ratio: f32,
    reactions_side: Side,
    reactions_align: Align,
}

impl<Message> Widget<Message, Theme, iced::Renderer> for Frame<'_, Message> {
    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.children);
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let max = limits.max();
        let available = if max.width.is_finite() {
            max.width * self.ratio
        } else {
            f32::INFINITY
        };

        let mut children = self.children.iter_mut().zip(&mut tree.children);

        let (content, state) = children.next().expect("bubble content");
        let content = content.as_widget_mut().layout(
            state,
            renderer,
            &layout::Limits::new(Size::ZERO, Size::new(available, max.height)),
        );
        let size = content.size();
        let width = if max.width.is_finite() {
            max.width
        } else {
            size.width
        };
        let x = match self.align {
            Align::Start => 0.0,
            Align::End => width - size.width,
        };

        let mut nodes = vec![content.move_to(Point::new(x, 0.0))];

        if let Some((reactions, state)) = children.next() {
            let node = reactions.as_widget_mut().layout(
                state,
                renderer,
                &layout::Limits::new(Size::ZERO, Size::new(size.width, f32::INFINITY)),
            );
            let reaction = node.size();

            // `left-3` / `right-3`
            let rx = match self.reactions_align {
                Align::Start => x + 12.0,
                Align::End => x + size.width - 12.0 - reaction.width,
            };
            // `translate-y-3/4` past the edge
            let ry = match self.reactions_side {
                Side::Top => -reaction.height * 0.75,
                Side::Bottom => size.height - reaction.height * 0.25,
            };

            nodes.push(node.move_to(Point::new(rx, ry)));
        }

        layout::Node::with_children(Size::new(width, size.height), nodes)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        for ((child, state), layout) in self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            child
                .as_widget_mut()
                .operate(state, layout, renderer, operation);
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        // The reactions sit on top, so they get events first.
        for ((child, state), layout) in self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .rev()
        {
            child.as_widget_mut().update(
                state, event, layout, cursor, renderer, clipboard, shell, viewport,
            );
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        for (index, ((child, state), layout)) in self
            .children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .enumerate()
        {
            if index == 1 {
                // `ring-3 ring-card`
                let bounds = layout.bounds().expand(3.0);
                renderer.fill_quad(
                    renderer::Quad {
                        bounds,
                        border: border::rounded(f32::MAX),
                        ..renderer::Quad::default()
                    },
                    Background::Color(tokens(theme).palette.card),
                );
            }

            child
                .as_widget()
                .draw(state, renderer, theme, style, layout, cursor, viewport);
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child
                    .as_widget()
                    .mouse_interaction(state, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, iced::Renderer>> {
        let overlays: Vec<_> = self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .filter_map(|((child, state), layout)| {
                child
                    .as_widget_mut()
                    .overlay(state, layout, renderer, viewport, translation)
            })
            .collect();

        (!overlays.is_empty()).then(|| overlay::Group::with_children(overlays).overlay())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_oklch_keeps_gray_gray() {
        let color = relative_oklch(Color::from_rgb(0.2, 0.2, 0.2), 0.93, 0.4);
        assert!((color.r - color.g).abs() < 0.01 && (color.g - color.b).abs() < 0.01);
        assert!(color.r > 0.85 && color.r < 0.95, "{color:?}");
    }
}
