//! A popup that displays information related to an element when the
//! element is hovered.
//!
//! Like Radix, it also opens when the element receives the keyboard focus
//! (`:focus-visible`), and closes on <kbd>Escape</kbd>. Touch screens have
//! no hover: a long press shows it, while the finger rests and a little
//! longer; a tap still presses the element.
//!
//! Port of <https://ui.shadcn.com/docs/components/tooltip>.
//!
//! ```no_run
//! # use shadcn_iced::button::{button, Variant};
//! # use shadcn_iced::tooltip::tooltip;
//! # fn view<'a>() -> iced::Element<'a, ()> {
//! tooltip(button("Hover").variant(Variant::Outline), "Add to library").into()
//! # }
//! ```
use shadcn_iced_core::primitive::anchored::AnchorTracker;
use shadcn_iced_core::primitive::{Align, Anchored, Side};
use shadcn_iced_core::{Element, style, tokens};

use iced::time::Duration;
use iced::widget::canvas::{self, Canvas, Frame, Path};
use iced::widget::{Column, Row, container, text};
use iced::{Alignment, Background, Length, Point, Rectangle, Renderer, Theme, border, mouse};

/// The width of the visible part of the arrow: a `size-2.5` square turned
/// by 45° whose center sits 2px inside the bubble.
const ARROW_WIDTH: f32 = 10.0;
/// How far the arrow pokes out of the bubble.
const ARROW_HEIGHT: f32 = 5.0;
/// How close the `size-2.5` square of the arrow gets to the edges of the
/// bubble (`arrowPadding` of Base UI).
const ARROW_PADDING: f32 = 5.0;

/// A tooltip. See the [module documentation](self).
pub struct Tooltip<'a, Message> {
    trigger: Element<'a, Message>,
    content: Element<'a, Message>,
    side: Side,
    align: Align,
    offset: f32,
    delay: Duration,
}

/// Shows `content` above `trigger` while it is hovered.
pub fn tooltip<'a, Message: Clone + 'a>(
    trigger: impl Into<Element<'a, Message>>,
    content: impl text::IntoFragment<'a>,
) -> Tooltip<'a, Message> {
    custom(
        trigger,
        text(content)
            .size(style::text::XS)
            .line_height(iced::Pixels(16.0)),
    )
}

/// Shows any element as a tooltip, e.g. a label and a
/// [`kbd`](super::kbd::kbd).
pub fn custom<'a, Message: Clone + 'a>(
    trigger: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
) -> Tooltip<'a, Message> {
    Tooltip {
        trigger: trigger.into(),
        content: content.into(),
        side: Side::Top,
        align: Align::Center,
        offset: 4.0,
        delay: Duration::ZERO,
    }
}

impl<'a, Message: Clone + 'a> Tooltip<'a, Message> {
    /// Sets the side of the trigger the tooltip is shown on. Defaults to
    /// [`Side::Top`].
    pub fn side(mut self, side: Side) -> Self {
        self.side = side;
        self
    }

    /// Sets the alignment along the side. Defaults to [`Align::Center`].
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    /// Sets the distance between the trigger and the bubble
    /// (`sideOffset`, 4 by default).
    pub fn offset(mut self, offset: f32) -> Self {
        self.offset = offset;
        self
    }

    /// Sets how long the trigger must be hovered before the tooltip opens.
    /// Defaults to `0`, like shadcn/ui's `TooltipProvider`.
    pub fn delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }
}

/// The style of a tooltip bubble (`rounded-md bg-foreground
/// text-background`).
pub fn style(theme: &Theme) -> container::Style {
    let t = tokens(theme);

    container::Style {
        background: Some(Background::Color(t.palette.foreground)),
        text_color: Some(t.palette.background),
        border: border::rounded(t.radius_md()),
        ..container::Style::default()
    }
}

/// The arrow of a tooltip, pointing at the trigger. It spans the side of
/// the bubble, to follow the trigger when the bubble is shifted to stay
/// inside the window.
struct Arrow {
    side: Side,
    tracker: AnchorTracker,
}

impl<Message> canvas::Program<Message> for Arrow {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        // The triangle pointing down, in (along, out) coordinates: the base
        // against the bubble and the tip rounded (`rounded-[2px]`).
        let half = ARROW_WIDTH / 2.0;
        let round = 1.2;
        let points = [
            (0.0, 0.0),
            (half - round, ARROW_HEIGHT - round),
            (half, ARROW_HEIGHT),
            (half + round, ARROW_HEIGHT - round),
            (ARROW_WIDTH, 0.0),
        ];

        let size = bounds.size();
        let vertical = matches!(self.side, Side::Top | Side::Bottom);
        let length = if vertical { size.width } else { size.height };

        // The center of the arrow faces the center of the trigger, as far
        // as the padding of the arrow allows.
        let center = self
            .tracker
            .anchor()
            .map(|anchor| {
                if vertical {
                    anchor.center_x() - bounds.x
                } else {
                    anchor.center_y() - bounds.y
                }
            })
            .unwrap_or(length / 2.0);
        let min = ARROW_PADDING + ARROW_WIDTH / 2.0;
        let center = if length >= min * 2.0 {
            center.clamp(min, length - min)
        } else {
            length / 2.0
        };
        let start = center - half;

        let map = |(along, out): (f32, f32)| match self.side {
            Side::Top => Point::new(start + along, out),
            Side::Bottom => Point::new(start + along, size.height - out),
            Side::Left => Point::new(out, start + along),
            Side::Right => Point::new(size.width - out, start + along),
        };

        let path = Path::new(|builder| {
            builder.move_to(map(points[0]));
            builder.line_to(map(points[1]));
            builder.quadratic_curve_to(map(points[2]), map(points[3]));
            builder.line_to(map(points[4]));
            builder.close();
        });

        frame.fill(&path, tokens(theme).palette.foreground);

        vec![frame.into_geometry()]
    }
}

impl<'a, Message: Clone + 'a> From<Tooltip<'a, Message>> for Element<'a, Message> {
    fn from(tooltip: Tooltip<'a, Message>) -> Self {
        // `px-3 py-1.5 text-xs`
        let bubble = container(tooltip.content)
            .padding([6, 12])
            .max_width(320)
            .style(self::style);

        let side = tooltip.side;
        let vertical = matches!(side, Side::Top | Side::Bottom);

        let tracker = AnchorTracker::new();

        let arrow = Canvas::new(Arrow {
            side,
            tracker: tracker.clone(),
        })
        .width(if vertical {
            Length::Fill
        } else {
            Length::Fixed(ARROW_HEIGHT)
        })
        .height(if vertical {
            Length::Fixed(ARROW_HEIGHT)
        } else {
            Length::Fill
        });

        let content: Element<'a, Message> = match side {
            Side::Top => Column::new()
                .push(bubble)
                .push(arrow)
                .align_x(Alignment::Center)
                // The arrow fills the side of the bubble only.
                .width(Length::Shrink)
                .into(),
            Side::Bottom => Column::new()
                .push(arrow)
                .push(bubble)
                .align_x(Alignment::Center)
                // The arrow fills the side of the bubble only.
                .width(Length::Shrink)
                .into(),
            Side::Left => Row::new()
                .push(bubble)
                .push(arrow)
                .align_y(Alignment::Center)
                // The arrow fills the side of the bubble only.
                .height(Length::Shrink)
                .into(),
            Side::Right => Row::new()
                .push(arrow)
                .push(bubble)
                .align_y(Alignment::Center)
                // The arrow fills the side of the bubble only.
                .height(Length::Shrink)
                .into(),
        };

        Anchored::new(tooltip.trigger, content)
            .hover(tooltip.delay, Duration::ZERO)
            .track_anchor(tracker)
            .side(side)
            .align(tooltip.align)
            // The arrow is part of the content.
            .offset(tooltip.offset - ARROW_HEIGHT)
            // Base UI keeps popups 5px away from the edges of the window.
            .collision_padding(5.0)
            .into()
    }
}
