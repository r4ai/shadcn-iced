//! Scrollbars drawn over a scrollable, like the ones of Base UI's scroll
//! area.
//!
//! iced draws the scrollbars of a [`Scrollable`] in the whole height (or
//! width) of its bounds and maps the drag of the thumb onto that length.
//! Base UI insets the track by its padding (`p-px`), gives the thumb a
//! minimum size of 16px, and scrolls by the distance the pointer moved over
//! the free length of the track. [`Scrollbars`] does the same: hide the
//! scrollbars of the scrollable ([`Scrollbar::hidden`]) and wrap it, and
//! [`Scrollbars`] draws the thumbs, drags them, and jumps to where the track
//! is pressed.
//!
//! ```no_run
//! # use shadcn_iced_core::primitive::scrollbars;
//! # use iced::widget::{scrollable, text};
//! # use iced::widget::scrollable::{Direction, Scrollbar};
//! # fn view<'a, Message: 'a>() -> iced::Element<'a, Message> {
//! let list = scrollable(text("A long text..."))
//!     .direction(Direction::Vertical(Scrollbar::hidden()));
//!
//! scrollbars(list)
//!     .vertical(true)
//!     .thumb(|_theme| iced::Color::BLACK.into())
//!     .into()
//! # }
//! ```
//!
//! [`Scrollable`]: iced::widget::Scrollable
//! [`Scrollbar::hidden`]: iced::widget::scrollable::Scrollbar::hidden
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Id, Operation, Tree, Widget, operation, tree};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::widget::scrollable::AbsoluteOffset;
use iced::{Background, Element, Event, Length, Point, Rectangle, Size, Vector, border, mouse};

/// The smallest length of a thumb (`MIN_THUMB_SIZE` of Base UI).
pub const MIN_THUMB: f32 = 16.0;

/// Scrollbars over a scrollable. See the [module documentation](self).
pub struct Scrollbars<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    vertical: bool,
    horizontal: bool,
    size: f32,
    padding: f32,
    border: f32,
    thumb: Box<dyn Fn(&Theme) -> Background + 'a>,
}

/// Creates [`Scrollbars`] over `content`, which is the scrollable.
pub fn scrollbars<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Scrollbars<'a, Message, Theme, Renderer> {
    Scrollbars {
        content: content.into(),
        vertical: false,
        horizontal: false,
        size: 10.0,
        padding: 1.0,
        border: 1.0,
        thumb: Box::new(|_| Background::Color(iced::Color::BLACK)),
    }
}

impl<'a, Message, Theme, Renderer> Scrollbars<'a, Message, Theme, Renderer> {
    /// Shows a vertical scrollbar when the content is taller than the
    /// scrollable.
    pub fn vertical(mut self, vertical: bool) -> Self {
        self.vertical = vertical;
        self
    }

    /// Shows a horizontal scrollbar when the content is wider than the
    /// scrollable.
    pub fn horizontal(mut self, horizontal: bool) -> Self {
        self.horizontal = horizontal;
        self
    }

    /// Sets the thickness of a track. Defaults to `10.0` (`w-2.5`).
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Sets the padding of a track. Defaults to `1.0` (`p-px`).
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    /// Sets the transparent border on the inner side of a track. Defaults
    /// to `1.0` (`border-l border-l-transparent`).
    pub fn border(mut self, border: f32) -> Self {
        self.border = border;
        self
    }

    /// Sets the background of the thumbs.
    pub fn thumb(mut self, thumb: impl Fn(&Theme) -> Background + 'a) -> Self {
        self.thumb = Box::new(thumb);
        self
    }
}

/// An axis of scrolling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Axis {
    Vertical,
    Horizontal,
}

impl Axis {
    fn of(self, point: Point) -> f32 {
        match self {
            Axis::Vertical => point.y,
            Axis::Horizontal => point.x,
        }
    }

    fn offset(self, scroll: f32) -> AbsoluteOffset<Option<f32>> {
        match self {
            Axis::Vertical => AbsoluteOffset {
                x: None,
                y: Some(scroll),
            },
            Axis::Horizontal => AbsoluteOffset {
                x: Some(scroll),
                y: None,
            },
        }
    }
}

/// What [`Scrollbars`] knows of its scrollable.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct Metrics {
    bounds: Size,
    content: Size,
    translation: Vector,
}

/// A drag of a thumb.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Drag {
    axis: Axis,
    start: f32,
    scroll: f32,
}

#[derive(Debug, Clone, Copy, Default)]
struct State {
    metrics: Option<Metrics>,
    drag: Option<Drag>,
}

/// A visible scrollbar.
#[derive(Debug, Clone, Copy)]
struct Bar {
    axis: Axis,
    track: Rectangle,
    thumb: Rectangle,
    /// How far the thumb moves, for the pointer: Base UI measures the
    /// thumb with `offsetHeight`, which rounds it to a whole pixel.
    travel: f32,
    /// How far the content scrolls.
    range: f32,
    scroll: f32,
}

impl Bar {
    /// The scroll for a thumb moved by `delta` from `scroll`.
    fn scroll(&self, scroll: f32, delta: f32) -> f32 {
        if self.travel <= 0.0 {
            return scroll;
        }

        (scroll + delta / self.travel * self.range).clamp(0.0, self.range)
    }
}

impl<'a, Message, Theme, Renderer> Scrollbars<'a, Message, Theme, Renderer> {
    fn bars(&self, bounds: Rectangle, metrics: Metrics) -> [Option<Bar>; 2] {
        let Metrics {
            bounds: viewport,
            content,
            translation,
        } = metrics;

        let vertical = self.vertical && content.height > viewport.height;
        let horizontal = self.horizontal && content.width > viewport.width;

        // Where both are shown, each track stops at the corner.
        let corner = |other: bool| if other { self.size } else { 0.0 };
        let padding = self.padding;

        let bar = |length: f32, viewport: f32, content: f32, scroll: f32| {
            let inner = (length - padding * 2.0).max(0.0);
            let thumb = (inner * viewport / content).max(MIN_THUMB);
            let range = content - viewport;
            let offset = if range > 0.0 {
                scroll.clamp(0.0, range) / range * (length - thumb - padding * 2.0).max(0.0)
            } else {
                0.0
            };
            let travel = length.round() - thumb.round() - padding * 2.0;

            (thumb, travel, range, padding + offset)
        };

        let y = vertical.then(|| {
            let length = bounds.height - corner(horizontal);
            let (thumb, travel, range, offset) =
                bar(length, viewport.height, content.height, translation.y);
            let track = Rectangle {
                x: bounds.x + bounds.width - self.size,
                y: bounds.y,
                width: self.size,
                height: length,
            };

            Bar {
                axis: Axis::Vertical,
                track,
                thumb: Rectangle {
                    x: track.x + self.border + padding,
                    y: track.y + offset,
                    width: self.size - self.border - padding * 2.0,
                    height: thumb,
                },
                travel,
                range,
                scroll: translation.y,
            }
        });

        let x = horizontal.then(|| {
            let length = bounds.width - corner(vertical);
            let (thumb, travel, range, offset) =
                bar(length, viewport.width, content.width, translation.x);
            let track = Rectangle {
                x: bounds.x,
                y: bounds.y + bounds.height - self.size,
                width: length,
                height: self.size,
            };

            Bar {
                axis: Axis::Horizontal,
                track,
                thumb: Rectangle {
                    x: track.x + offset,
                    y: track.y + self.border + padding,
                    width: thumb,
                    height: self.size - self.border - padding * 2.0,
                },
                travel,
                range,
                scroll: translation.x,
            }
        });

        [y, x]
    }

    /// Reads the bounds, the content bounds and the translation of the
    /// scrollable.
    fn refresh(&mut self, tree: &mut Tree, layout: Layout<'_>, renderer: &Renderer)
    where
        Renderer: iced::advanced::Renderer,
    {
        let mut query = Query(None);

        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, &mut query);

        tree.state.downcast_mut::<State>().metrics = query.0;
    }

    fn scroll_to(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        offset: AbsoluteOffset<Option<f32>>,
    ) where
        Renderer: iced::advanced::Renderer,
    {
        self.content.as_widget_mut().operate(
            &mut tree.children[0],
            layout,
            renderer,
            &mut ScrollTo(Some(offset)),
        );

        self.refresh(tree, layout, renderer);
    }
}

/// Reads the metrics of the first scrollable.
struct Query(Option<Metrics>);

impl Operation for Query {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        if self.0.is_none() {
            operate(self);
        }
    }

    fn scrollable(
        &mut self,
        _id: Option<&Id>,
        bounds: Rectangle,
        content_bounds: Rectangle,
        translation: Vector,
        _state: &mut dyn operation::Scrollable,
    ) {
        if self.0.is_none() {
            self.0 = Some(Metrics {
                bounds: bounds.size(),
                content: content_bounds.size(),
                translation,
            });
        }
    }
}

/// Scrolls the first scrollable.
struct ScrollTo(Option<AbsoluteOffset<Option<f32>>>);

impl Operation for ScrollTo {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        if self.0.is_some() {
            operate(self);
        }
    }

    fn scrollable(
        &mut self,
        _id: Option<&Id>,
        _bounds: Rectangle,
        _content_bounds: Rectangle,
        _translation: Vector,
        state: &mut dyn operation::Scrollable,
    ) {
        if let Some(offset) = self.0.take() {
            state.scroll_to(offset);
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Scrollbars<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let child = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);

        self.refresh(tree, Layout::new(&child), renderer);

        layout::Node::with_children(child.size(), vec![child])
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content.as_widget_mut().operate(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            operation,
        );
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let content = layout.children().next().unwrap();
        let state = *tree.state.downcast_ref::<State>();
        let bars = state
            .metrics
            .map(|metrics| self.bars(layout.bounds(), metrics))
            .unwrap_or_default();
        let bar = |axis: Axis| bars.into_iter().flatten().find(|bar| bar.axis == axis);

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let pressed = cursor.position().and_then(|position| {
                    bars.into_iter()
                        .flatten()
                        .find(|bar| bar.track.contains(position))
                        .map(|bar| (bar, position))
                });

                if let Some((bar, position)) = pressed {
                    let axis = bar.axis;
                    let at = axis.of(position);

                    // A press on the track (not on the thumb) first moves
                    // the center of the thumb under the pointer.
                    let scroll = if bar.thumb.contains(position) {
                        bar.scroll
                    } else {
                        let (start, length) = match axis {
                            Axis::Vertical => (bar.track.y, bar.thumb.height),
                            Axis::Horizontal => (bar.track.x, bar.thumb.width),
                        };
                        let click = at - start - length / 2.0 - self.padding * 2.0;
                        let scroll = bar.scroll(0.0, click);

                        self.scroll_to(tree, content, renderer, axis.offset(scroll));

                        scroll
                    };

                    tree.state.downcast_mut::<State>().drag = Some(Drag {
                        axis,
                        start: at,
                        scroll,
                    });

                    shell.capture_event();
                    shell.request_redraw();

                    return;
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                if let Some(drag) = state.drag {
                    if let Some(bar) = bar(drag.axis) {
                        let scroll = bar.scroll(drag.scroll, drag.axis.of(*position) - drag.start);

                        if scroll != bar.scroll {
                            self.scroll_to(tree, content, renderer, drag.axis.offset(scroll));
                            shell.request_redraw();
                        }
                    }

                    shell.capture_event();

                    return;
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if state.drag.is_some() =>
            {
                tree.state.downcast_mut::<State>().drag = None;
            }
            _ => {}
        }

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            content,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        let before = tree.state.downcast_ref::<State>().metrics;
        self.refresh(tree, content, renderer);

        if tree.state.downcast_ref::<State>().metrics != before {
            shell.request_redraw();
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout.children().next().unwrap(),
            cursor,
            viewport,
        );

        let Some(metrics) = tree.state.downcast_ref::<State>().metrics else {
            return;
        };

        let background = (self.thumb)(theme);

        for bar in self.bars(layout.bounds(), metrics).into_iter().flatten() {
            let thickness = bar.thumb.width.min(bar.thumb.height);

            renderer.fill_quad(
                renderer::Quad {
                    bounds: bar.thumb,
                    border: border::rounded(thickness / 2.0),
                    ..renderer::Quad::default()
                },
                background,
            );
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();
        let over_track = state.metrics.is_some_and(|metrics| {
            cursor.position().is_some_and(|position| {
                self.bars(layout.bounds(), metrics)
                    .into_iter()
                    .flatten()
                    .any(|bar| bar.track.contains(position))
            })
        });

        if state.drag.is_some() || over_track {
            return mouse::Interaction::Idle;
        }

        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<Scrollbars<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(scrollbars: Scrollbars<'a, Message, Theme, Renderer>) -> Self {
        Element::new(scrollbars)
    }
}
