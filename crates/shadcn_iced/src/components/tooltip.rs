//! A popup that displays information related to an element when the
//! element is hovered.
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
use shadcn_iced_core::primitive::{Align, Anchored, Side};
use shadcn_iced_core::{Element, style, tokens};

use iced::time::Duration;
use iced::widget::canvas::{self, Canvas, Frame, Path};
use iced::widget::{Column, Row, container, text};
use iced::{Alignment, Background, Point, Rectangle, Renderer, Theme, border, mouse};

/// The width of the visible part of the arrow: a `size-2.5` square turned
/// by 45° whose center sits 2px inside the bubble.
const ARROW_WIDTH: f32 = 10.0;
/// How far the arrow pokes out of the bubble.
const ARROW_HEIGHT: f32 = 5.0;

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

/// The arrow of a tooltip, pointing at the trigger.
struct Arrow {
    side: Side,
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
        let map = |(along, out): (f32, f32)| match self.side {
            Side::Top => Point::new(along, out),
            Side::Bottom => Point::new(along, size.height - out),
            Side::Left => Point::new(out, along),
            Side::Right => Point::new(size.width - out, along),
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

        let arrow = Canvas::new(Arrow { side })
            .width(if vertical { ARROW_WIDTH } else { ARROW_HEIGHT })
            .height(if vertical { ARROW_HEIGHT } else { ARROW_WIDTH });

        let content: Element<'a, Message> = match side {
            Side::Top => Column::new()
                .push(bubble)
                .push(arrow)
                .align_x(Alignment::Center)
                .into(),
            Side::Bottom => Column::new()
                .push(arrow)
                .push(bubble)
                .align_x(Alignment::Center)
                .into(),
            Side::Left => Row::new()
                .push(bubble)
                .push(arrow)
                .align_y(Alignment::Center)
                .into(),
            Side::Right => Row::new()
                .push(arrow)
                .push(bubble)
                .align_y(Alignment::Center)
                .into(),
        };

        Anchored::new(tooltip.trigger, content)
            .hover(tooltip.delay, Duration::ZERO)
            .side(side)
            .align(tooltip.align)
            // The arrow is part of the content.
            .offset(tooltip.offset - ARROW_HEIGHT)
            .into()
    }
}
