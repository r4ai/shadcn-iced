//! The sliding track of a carousel.
//!
//! [`Track`] lays all of its items out in a row (or a column), shows a
//! window of them and slides to the item at its `index` with an ease-out
//! when it changes, like [Embla Carousel](https://www.embla-carousel.com)
//! behind the carousel of shadcn/ui. The items can also be dragged (swiped)
//! with the pointer: the track follows the pointer and, once released,
//! asks to go to the item it was dragged to ([`Track::on_change`]).
//!
//! The index lives in your state, like the value of any other control.
use crate::animation::{self, Transition};

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::time::{Duration, Instant};
use iced::{Element, Event, Length, Point, Rectangle, Size, Vector, mouse, touch, window};

/// How long the track takes to slide to another item.
const SLIDE: Duration = Duration::from_millis(450);

/// How far the pointer must move before a press becomes a drag.
const DRAG_THRESHOLD: f32 = 6.0;

/// The part of an item a drag must cover to go to the next one.
const SWIPE: f32 = 0.2;

/// The sliding track of a carousel. See the [module documentation](self).
pub struct Track<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    items: Vec<Element<'a, Message, Theme, Renderer>>,
    index: usize,
    per_view: usize,
    spacing: f32,
    vertical: bool,
    height: Option<f32>,
    on_change: Option<Box<dyn Fn(usize) -> Message + 'a>>,
}

/// Creates a [`Track`] of `items`, showing the item at `index` first.
pub fn track<'a, Message, Theme, Renderer>(
    items: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
    index: usize,
) -> Track<'a, Message, Theme, Renderer> {
    Track {
        items: items.into_iter().collect(),
        index,
        per_view: 1,
        spacing: 0.0,
        vertical: false,
        height: None,
        on_change: None,
    }
}

impl<'a, Message, Theme, Renderer> Track<'a, Message, Theme, Renderer> {
    /// Sets how many items are visible at once.
    pub fn per_view(mut self, count: usize) -> Self {
        self.per_view = count.max(1);
        self
    }

    /// Sets the space between items.
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Stacks the items vertically.
    pub fn vertical(mut self, vertical: bool) -> Self {
        self.vertical = vertical;
        self
    }

    /// Gives a vertical track a fixed height: the items keep their own
    /// height and the ones that do not fit are clipped.
    pub fn height(mut self, height: Option<f32>) -> Self {
        self.height = height;
        self
    }

    /// Lets the items be dragged; `f` receives the index of the item the
    /// track was dragged to.
    pub fn on_change(mut self, f: impl Fn(usize) -> Message + 'a) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    /// The last index that still fills the view.
    fn last(&self) -> usize {
        self.items.len().saturating_sub(self.per_view)
    }
}

#[derive(Debug, Default)]
struct State {
    /// The start of every item along the track, from the layout.
    starts: Vec<f32>,
    /// The offset of the track.
    offset: Transition,
    /// The index the offset goes to, once known.
    index: Option<usize>,
    drag: Option<Drag>,
}

#[derive(Debug, Clone, Copy)]
struct Drag {
    origin: Point,
    offset: f32,
    dragging: bool,
}

impl State {
    fn start(&self, index: usize) -> f32 {
        self.starts.get(index).copied().unwrap_or_default()
    }

    /// The farthest the track can slide.
    fn max(&self, last: usize) -> f32 {
        self.start(last)
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Track<'_, Message, Theme, Renderer>
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
        self.items.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.items);
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let width = limits.max().width;
        let per_view = self.per_view as f32;

        let mut starts = Vec::with_capacity(self.items.len());
        let mut nodes = Vec::with_capacity(self.items.len());

        let size = if self.vertical {
            let item_limits =
                layout::Limits::new(Size::ZERO, Size::new(width, f32::INFINITY)).width(width);

            let mut y = 0.0;
            let mut largest: f32 = 0.0;

            for (item, tree) in self.items.iter_mut().zip(&mut tree.children) {
                let node = item.as_widget_mut().layout(tree, renderer, &item_limits);
                let height = node.size().height;

                starts.push(y);
                nodes.push(node.move_to(Point::new(0.0, y)));
                largest = largest.max(height);
                y += height + self.spacing;
            }

            let height = self
                .height
                .unwrap_or(largest * per_view + self.spacing * (per_view - 1.0));

            Size::new(width, height)
        } else {
            let item = ((width - self.spacing * (per_view - 1.0)) / per_view).max(0.0);
            let item_limits =
                layout::Limits::new(Size::ZERO, Size::new(item, limits.max().height)).width(item);

            let mut height: f32 = 0.0;

            for (index, (child, tree)) in self.items.iter_mut().zip(&mut tree.children).enumerate()
            {
                let x = index as f32 * (item + self.spacing);
                let node = child.as_widget_mut().layout(tree, renderer, &item_limits);

                height = height.max(node.size().height);
                starts.push(x);
                nodes.push(node.move_to(Point::new(x, 0.0)));
            }

            Size::new(width, height)
        };

        tree.state.downcast_mut::<State>().starts = starts;

        layout::Node::with_children(size, nodes)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            for ((item, tree), layout) in self
                .items
                .iter_mut()
                .zip(&mut tree.children)
                .zip(layout.children())
            {
                item.as_widget_mut()
                    .operate(tree, layout, renderer, operation);
            }
        });
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
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let last = self.last();
        let index = self.index.min(last);
        let now = Instant::now();
        let vertical = self.vertical;
        let along = |vector: Vector| if vertical { vector.y } else { vector.x };

        {
            let state = tree.state.downcast_mut::<State>();
            let target = state.start(index);

            match state.index {
                // Slides to a new index (unless the drag goes on).
                Some(current) if current != index && state.drag.is_none_or(|d| !d.dragging) => {
                    state.offset.go(target, SLIDE, animation::ease_out, now);
                    state.index = Some(index);
                    shell.request_redraw();
                }
                Some(_) => {
                    // The layout changed (e.g. the window was resized).
                    if state.drag.is_none()
                        && !state.offset.is_animating(now)
                        && (state.offset.value(now) - target).abs() > 0.5
                    {
                        state.offset.set(target);
                    }
                }
                None => {
                    state.offset.set(target);
                    state.index = Some(index);
                }
            }

            if let Event::Window(window::Event::RedrawRequested(now)) = event
                && state.offset.is_animating(*now)
            {
                shell.request_redraw();
            }
        }

        let offset = tree.state.downcast_ref::<State>().offset.value(now);
        let translation = if vertical {
            Vector::new(0.0, offset)
        } else {
            Vector::new(offset, 0.0)
        };

        // Dragging.
        if self.on_change.is_some() && self.items.len() > self.per_view {
            let state = tree.state.downcast_mut::<State>();
            let max = state.max(last);

            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerPressed { .. }) => {
                    if let Some(position) = cursor.position_over(bounds) {
                        state.drag = Some(Drag {
                            origin: position,
                            offset,
                            dragging: false,
                        });
                    }
                }
                Event::Mouse(mouse::Event::CursorMoved { .. })
                | Event::Touch(touch::Event::FingerMoved { .. }) => {
                    if let Some(drag) = &mut state.drag
                        && let Some(position) = cursor.position()
                    {
                        let moved = along(position - drag.origin);

                        if !drag.dragging && moved.abs() >= DRAG_THRESHOLD {
                            drag.dragging = true;
                        }

                        if drag.dragging {
                            let wanted = drag.offset - moved;

                            // Rubber band past both ends.
                            let offset = if wanted < 0.0 {
                                wanted * 0.3
                            } else if wanted > max {
                                max + (wanted - max) * 0.3
                            } else {
                                wanted
                            };

                            state.offset.set(offset);
                            shell.request_redraw();
                            shell.capture_event();
                            return;
                        }
                    }
                }
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                | Event::Touch(
                    touch::Event::FingerLifted { .. } | touch::Event::FingerLost { .. },
                ) => {
                    if let Some(drag) = state.drag.take()
                        && drag.dragging
                    {
                        let moved = offset - drag.offset;
                        let item = (state.start(1) - state.start(0)).max(1.0);
                        let from = state.index.unwrap_or(index);

                        // At least one item when the drag covers a part of it.
                        let steps = (moved / item).round() as isize;
                        let steps = if steps == 0 && moved.abs() >= item * SWIPE {
                            moved.signum() as isize
                        } else {
                            steps
                        };

                        let next = (from as isize + steps).clamp(0, last as isize) as usize;

                        // Slides from where the drag left it; back to the
                        // current item if the index does not change.
                        state.offset.set(offset);
                        state
                            .offset
                            .go(state.start(next), SLIDE, animation::ease_out, now);
                        state.index = Some(next);
                        shell.request_redraw();

                        if next != from
                            && let Some(on_change) = &self.on_change
                        {
                            shell.publish(on_change(next));
                        }

                        shell.capture_event();
                        return;
                    }
                }
                _ => {}
            }
        }

        let cursor = match cursor.position_over(bounds) {
            Some(position) => mouse::Cursor::Available(position + translation),
            None => cursor.levitate() + translation,
        };

        for ((item, tree), layout) in self
            .items
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            item.as_widget_mut().update(
                tree, event, layout, cursor, renderer, clipboard, shell, &bounds,
            );
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
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        let offset = state.offset.value(Instant::now());
        let translation = if self.vertical {
            Vector::new(0.0, offset)
        } else {
            Vector::new(offset, 0.0)
        };

        let Some(clip) = bounds.intersection(viewport) else {
            return;
        };

        let cursor = match cursor.position_over(bounds) {
            Some(position) => mouse::Cursor::Available(position + translation),
            None => cursor.levitate() + translation,
        };

        renderer.with_layer(clip, |renderer| {
            renderer.with_translation(-translation, |renderer| {
                let visible = bounds + translation;

                for ((item, tree), layout) in
                    self.items.iter().zip(&tree.children).zip(layout.children())
                {
                    if layout.bounds().intersects(&visible.expand(1.0)) {
                        item.as_widget()
                            .draw(tree, renderer, theme, style, layout, cursor, &visible);
                    }
                }
            });
        });
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

        if state.drag.is_some_and(|drag| drag.dragging) {
            return mouse::Interaction::Grabbing;
        }

        let bounds = layout.bounds();
        let offset = state.offset.value(Instant::now());
        let translation = if self.vertical {
            Vector::new(0.0, offset)
        } else {
            Vector::new(offset, 0.0)
        };

        let Some(position) = cursor.position_over(bounds) else {
            return mouse::Interaction::None;
        };

        let cursor = mouse::Cursor::Available(position + translation);

        self.items
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((item, tree), layout)| {
                item.as_widget()
                    .mouse_interaction(tree, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let offset = tree
            .state
            .downcast_ref::<State>()
            .offset
            .value(Instant::now());
        let slide = if self.vertical {
            Vector::new(0.0, -offset)
        } else {
            Vector::new(-offset, 0.0)
        };

        overlay::from_children(
            &mut self.items,
            tree,
            layout,
            renderer,
            viewport,
            translation + slide,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<Track<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(track: Track<'a, Message, Theme, Renderer>) -> Self {
        Element::new(track)
    }
}
