//! An input where the user selects a value from within a given range.
//!
//! Port of <https://ui.shadcn.com/docs/components/slider>.
//!
//! ```no_run
//! # use shadcn_iced::slider::slider;
//! # #[derive(Clone)] enum Message { Volume(f32) }
//! # fn view<'a>(volume: f32) -> iced::Element<'a, Message> {
//! slider(0.0..=100.0, volume, Message::Volume).into()
//! # }
//! ```
//!
//! Several thumbs select a range ([`range_slider`]).
use shadcn_iced_core::primitive::Focus;
use shadcn_iced_core::primitive::pointer::{self, Pointer};
use shadcn_iced_core::theme::{alpha, mix};
use shadcn_iced_core::{Element, style, tokens};

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::keyboard::{self, key::Named};
use iced::{
    Background, Border, Color, Event, Length, Rectangle, Size, Theme, border, mouse, touch,
};

use std::ops::RangeInclusive;

/// `size-3`
const THUMB: f32 = 12.0;
/// `h-1`
const TRACK: f32 = 4.0;
/// `after:-inset-2`: the thumb reacts to the cursor around it.
const HIT_SLOP: f32 = 8.0;
/// A finger grabs the slider from farther: the 4px track and the 12px
/// thumbs are much smaller than a fingertip.
const TOUCH_HIT_SLOP: f32 = HIT_SLOP + pointer::TOUCH_SLOP;

/// The direction of a [`Slider`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Orientation {
    /// Left to right.
    #[default]
    Horizontal,
    /// Bottom to top (`data-vertical:min-h-40`).
    Vertical,
}

/// A slider. See the [module documentation](self).
pub struct Slider<'a, Message> {
    range: RangeInclusive<f32>,
    values: Vec<f32>,
    step: f32,
    on_change: Option<Box<dyn Fn(usize, f32) -> Message + 'a>>,
    on_release: Option<Message>,
    orientation: Orientation,
    disabled: bool,
    width: Length,
    height: Length,
}

/// Creates a horizontal slider with one thumb.
pub fn slider<'a, Message: Clone + 'a>(
    range: RangeInclusive<f32>,
    value: f32,
    on_change: impl Fn(f32) -> Message + 'a,
) -> Slider<'a, Message> {
    Slider::new(range, vec![value]).on_change(move |_, value| on_change(value))
}

/// Creates a vertical slider with one thumb.
pub fn vertical<'a, Message: Clone + 'a>(
    range: RangeInclusive<f32>,
    value: f32,
    on_change: impl Fn(f32) -> Message + 'a,
) -> Slider<'a, Message> {
    slider(range, value, on_change).orientation(Orientation::Vertical)
}

/// Creates a slider with one thumb per value. `on_change` receives the index
/// of the moved thumb and its new value.
pub fn range_slider<'a, Message: Clone + 'a>(
    range: RangeInclusive<f32>,
    values: &[f32],
    on_change: impl Fn(usize, f32) -> Message + 'a,
) -> Slider<'a, Message> {
    Slider::new(range, values.to_vec()).on_change(on_change)
}

impl<'a, Message: Clone + 'a> Slider<'a, Message> {
    /// Creates a read-only slider with one thumb per value; see
    /// [`Slider::on_change`].
    pub fn new(range: RangeInclusive<f32>, values: Vec<f32>) -> Self {
        Self {
            range,
            values,
            step: 1.0,
            on_change: None,
            on_release: None,
            orientation: Orientation::Horizontal,
            disabled: false,
            width: Length::Fill,
            height: Length::Shrink,
        }
    }

    /// Sets the message produced when a thumb moves, with its index and new
    /// value.
    pub fn on_change(mut self, f: impl Fn(usize, f32) -> Message + 'a) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    /// Sets the message produced when a thumb is released.
    pub fn on_release(mut self, message: Message) -> Self {
        self.on_release = Some(message);
        self
    }

    /// Sets the step. Defaults to `1.0`.
    pub fn step(mut self, step: f32) -> Self {
        self.step = step;
        self
    }

    /// Sets the orientation.
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Disables the slider (`data-disabled:opacity-50`).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets the width.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height.
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    fn vertical(&self) -> bool {
        self.orientation == Orientation::Vertical
    }

    fn ratio(&self, value: f32) -> f32 {
        let (start, end) = (*self.range.start(), *self.range.end());

        if end > start {
            ((value - start) / (end - start)).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// The bounds of the thumb at `index` (`thumbAlignment="edge"`).
    fn thumb(&self, bounds: Rectangle, index: usize) -> Rectangle {
        let ratio = self.ratio(self.values[index]);

        if self.vertical() {
            Rectangle {
                x: bounds.center_x() - THUMB / 2.0,
                y: bounds.y + (1.0 - ratio) * (bounds.height - THUMB),
                width: THUMB,
                height: THUMB,
            }
        } else {
            Rectangle {
                x: bounds.x + ratio * (bounds.width - THUMB),
                y: bounds.center_y() - THUMB / 2.0,
                width: THUMB,
                height: THUMB,
            }
        }
    }

    fn value_at(&self, bounds: Rectangle, position: iced::Point) -> f32 {
        let ratio = if self.vertical() {
            1.0 - (position.y - bounds.y - THUMB / 2.0) / (bounds.height - THUMB).max(1.0)
        } else {
            (position.x - bounds.x - THUMB / 2.0) / (bounds.width - THUMB).max(1.0)
        }
        .clamp(0.0, 1.0);

        let (start, end) = (*self.range.start(), *self.range.end());

        self.snap(start + ratio * (end - start))
    }

    /// Snaps `value` to the closest step of the range, rounded to the
    /// decimals of the step so that steps do not drift (`0.1 + 0.2`), and
    /// keeps it in the range.
    fn snap(&self, value: f32) -> f32 {
        let (start, end) = (*self.range.start(), *self.range.end());

        let value = if self.step > 0.0 {
            let (start, step) = (f64::from(start), f64::from(self.step));
            let snapped = start + ((f64::from(value) - start) / step).round() * step;
            let decimals = decimals(self.step).max(decimals(*self.range.start()));
            let factor = 10f64.powi(decimals.min(9));

            ((snapped * factor).round() / factor) as f32
        } else {
            value
        };

        value.clamp(start.min(end), end.max(start))
    }

    /// The values the thumb at `index` can take: thumbs cannot cross their
    /// neighbors.
    fn limits(&self, index: usize) -> (f32, f32) {
        let (start, end) = (*self.range.start(), *self.range.end());
        let low = index
            .checked_sub(1)
            .map_or(start, |previous| self.values[previous]);
        let high = self.values.get(index + 1).copied().unwrap_or(end);

        (low.min(high), high.max(low))
    }

    /// The thumb that moves when the thumb at `index` is moved to `value`:
    /// thumbs stacked on it (with the same value) part in the direction of
    /// the move, the one on that side going.
    fn moving(&self, mut index: usize, value: f32) -> usize {
        let current = self.values[index];

        if value < current {
            while index > 0 && self.values[index - 1] == current {
                index -= 1;
            }
        } else if value > current {
            while self.values.get(index + 1) == Some(&current) {
                index += 1;
            }
        }

        index
    }

    /// Moves the thumb at `index` towards `value`, snapped to the steps and
    /// kept between its neighbors. Returns the thumb that moved, and its new
    /// value if it changed.
    fn move_to(&self, index: usize, value: f32) -> (usize, Option<f32>) {
        let index = self.moving(index, value);
        let (low, high) = self.limits(index);
        let value = self.snap(value).clamp(low, high);

        (
            index,
            ((value - self.values[index]).abs() > f32::EPSILON).then_some(value),
        )
    }

    fn hovered_thumb(&self, bounds: Rectangle, cursor: mouse::Cursor) -> Option<usize> {
        self.thumb_at(bounds, cursor.position()?, HIT_SLOP)
    }

    /// The thumb within `slop` of `position`, the last one on top.
    fn thumb_at(&self, bounds: Rectangle, position: iced::Point, slop: f32) -> Option<usize> {
        (0..self.values.len())
            .rev()
            .find(|&index| self.thumb(bounds, index).expand(slop).contains(position))
    }
}

/// The number of decimals of `value`, as written.
fn decimals(value: f32) -> i32 {
    value
        .to_string()
        .split_once('.')
        .map_or(0, |(_, decimals)| decimals.len() as i32)
}

#[derive(Debug, Default)]
struct State {
    dragging: Option<usize>,
    /// The pointer dragging a thumb: other fingers are ignored, like the
    /// pointer capture of Radix.
    pointer: Option<Pointer>,
    /// The thumb under the cursor, which shows a ring.
    hovered: Option<usize>,
    /// The keyboard focus of each thumb.
    focus: Vec<Focus>,
}

impl State {
    fn focus(&mut self, count: usize) -> &mut [Focus] {
        self.focus.resize(count, Focus::default());
        &mut self.focus
    }
}

impl<Message: Clone> Widget<Message, Theme, iced::Renderer> for Slider<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        if self.vertical() {
            Size::new(self.width, self.height)
        } else {
            Size::new(self.width, Length::Shrink)
        }
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = if self.vertical() {
            // `data-vertical:min-h-40`
            let height = match self.height {
                Length::Shrink => Length::Fixed(160.0),
                height => height,
            };

            // A block as wide as its parent (`w-auto`), with the track and
            // thumbs centered in it (`items-center`).
            limits.resolve(self.width, height, Size::new(THUMB, 160.0))
        } else {
            // As tall as the track (`h-1`): the thumbs overflow it.
            limits.resolve(self.width, TRACK, Size::new(0.0, TRACK))
        };

        layout::Node::new(size)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        if self.disabled || self.on_change.is_none() {
            return;
        }

        // Each thumb takes the keyboard focus (`tabindex="0"`).
        let bounds = layout.bounds();
        let state = tree.state.downcast_mut::<State>();

        for (index, focus) in state.focus(self.values.len()).iter_mut().enumerate() {
            let thumb = self.thumb(bounds, index);

            operation.custom(None, thumb, focus);
            operation.focusable(None, thumb, focus);
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        pointer::observe(event);

        if self.disabled || self.values.is_empty() {
            return;
        }

        let Some(on_change) = &self.on_change else {
            return;
        };

        let state = tree.state.downcast_mut::<State>();
        let bounds = layout.bounds();
        let count = self.values.len();

        // Returns the thumb being dragged, which changes when it parts from
        // thumbs stacked on it.
        let change = |index: usize, position, shell: &mut Shell<'_, Message>| {
            let (index, value) = self.move_to(index, self.value_at(bounds, position));

            if let Some(value) = value {
                shell.publish(on_change(index, value));
            }

            index
        };

        // The keyboard moves the focused thumb: the arrow keys by a step
        // (`largeStep`, ten steps, with Shift or the page keys), Home and
        // End to the ends of the range.
        if let Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = event
            && !shell.is_event_captured()
            && let Some(index) = state.focus(count).iter().position(Focus::is_focused)
        {
            state.focus[index].key_pressed();
            shell.request_redraw();

            let (start, end) = (*self.range.start(), *self.range.end());
            let step = if self.step > 0.0 {
                self.step
            } else {
                (end - start) / 100.0
            };
            let large = step * 10.0;
            let value = self.values[index];

            let target = match key {
                keyboard::Key::Named(Named::ArrowRight | Named::ArrowUp) => {
                    Some(value + if modifiers.shift() { large } else { step })
                }
                keyboard::Key::Named(Named::ArrowLeft | Named::ArrowDown) => {
                    Some(value - if modifiers.shift() { large } else { step })
                }
                keyboard::Key::Named(Named::PageUp) => Some(value + large),
                keyboard::Key::Named(Named::PageDown) => Some(value - large),
                keyboard::Key::Named(Named::Home) => Some(start),
                keyboard::Key::Named(Named::End) => Some(end),
                _ => None,
            };

            if let Some(target) = target {
                let (moved, target) = self.move_to(index, target);

                if let Some(target) = target {
                    shell.publish(on_change(moved, target));
                }

                // The focus follows a thumb parting from the ones stacked
                // on it.
                if moved != index {
                    state.focus[index].blur();
                    state.focus[moved].focus_with_keyboard();
                }

                shell.capture_event();
            }

            return;
        }

        // A press elsewhere takes the focus away.
        if let Event::Mouse(mouse::Event::ButtonPressed(_))
        | Event::Touch(touch::Event::FingerPressed { .. }) = event
        {
            for focus in state.focus(count) {
                focus.blur();
            }
        }

        // Where the pointer dragging a thumb moved to, on an event about it.
        let dragged = match (state.pointer, event) {
            (Some(Pointer::Mouse), Event::Mouse(mouse::Event::CursorMoved { .. })) => {
                cursor.position()
            }
            (Some(Pointer::Finger(finger)), Event::Touch(touch::Event::FingerMoved { id, .. }))
                if *id == finger =>
            {
                cursor.position()
            }
            _ => None,
        };

        // Whether the pointer dragging a thumb was released, or taken by the
        // system.
        let released = match (state.pointer, event) {
            (
                Some(Pointer::Mouse),
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
            ) => true,
            (
                Some(Pointer::Finger(finger)),
                Event::Touch(
                    touch::Event::FingerLifted { id, .. } | touch::Event::FingerLost { id, .. },
                ),
            ) => *id == finger,
            _ => false,
        };

        let pressed = match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => cursor
                .position()
                .map(|position| (Pointer::Mouse, position, HIT_SLOP)),
            // The cursor follows the fingers, and is translated by the
            // scrollables around, unlike the positions of touch events.
            Event::Touch(touch::Event::FingerPressed { id, .. }) => cursor
                .position()
                .map(|position| (Pointer::Finger(*id), position, TOUCH_HIT_SLOP)),
            _ => None,
        };

        if let Some((by, position, slop)) = pressed {
            // A second finger does not take the thumb from the first one.
            if state.pointer.is_some() || !bounds.expand(slop).contains(position) {
                return;
            }

            // The hovered thumb, or the closest one.
            let index = self.thumb_at(bounds, position, slop).unwrap_or_else(|| {
                let value = self.value_at(bounds, position);

                (0..self.values.len())
                    .min_by(|&a, &b| {
                        (self.values[a] - value)
                            .abs()
                            .total_cmp(&(self.values[b] - value).abs())
                    })
                    .unwrap_or(0)
            });

            state.focus(count)[index].focus_with_pointer();
            state.dragging = Some(change(index, position, shell));
            state.pointer = Some(by);
            shell.capture_event();
            shell.request_redraw();
            return;
        }

        if let (Some(index), Some(position)) = (state.dragging, dragged) {
            state.dragging = Some(change(index, position, shell));
            shell.capture_event();
            shell.request_redraw();
        }

        if released {
            state.dragging = None;
            state.pointer = None;

            if let Some(message) = &self.on_release {
                shell.publish(message.clone());
            }

            shell.capture_event();
            shell.request_redraw();
        }

        // Only the ring of the hovered thumb follows the cursor; a finger
        // leaves no ring behind once it lifted (`hover:` is for mice).
        if let Event::Mouse(mouse::Event::CursorMoved { .. })
        | Event::Touch(
            touch::Event::FingerMoved { .. }
            | touch::Event::FingerLifted { .. }
            | touch::Event::FingerLost { .. },
        ) = event
        {
            let hovered = self.hovered_thumb(bounds, pointer::hover(cursor));

            if hovered != state.hovered {
                state.hovered = hovered;
                shell.request_redraw();
            }
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        use iced::advanced::Renderer as _;

        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        let t = tokens(theme);
        let p = &t.palette;
        let opacity = if self.disabled {
            style::DISABLED_OPACITY
        } else {
            1.0
        };

        // `data-disabled:opacity-50` fades the control as a whole: the
        // thumbs stay opaque over the track.
        let fade = |color: Color| mix(p.background, color, opacity);

        // `bg-muted rounded-full h-1 w-full`
        let track = if self.vertical() {
            Rectangle {
                x: bounds.center_x() - TRACK / 2.0,
                width: TRACK,
                ..bounds
            }
        } else {
            Rectangle {
                y: bounds.center_y() - TRACK / 2.0,
                height: TRACK,
                ..bounds
            }
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds: track,
                border: border::rounded(f32::MAX),
                ..renderer::Quad::default()
            },
            Background::Color(fade(p.muted)),
        );

        // `bg-primary`: from the start (or the first thumb) to the last thumb.
        let ratios: Vec<f32> = self.values.iter().map(|&v| self.ratio(v)).collect();
        let last = ratios.iter().copied().fold(0.0, f32::max);
        let first = if ratios.len() > 1 {
            ratios.iter().copied().fold(1.0, f32::min)
        } else {
            0.0
        };

        if !ratios.is_empty() {
            let indicator = if self.vertical() {
                Rectangle {
                    y: track.y + (1.0 - last) * track.height,
                    height: (last - first) * track.height,
                    ..track
                }
            } else {
                Rectangle {
                    x: track.x + first * track.width,
                    width: (last - first) * track.width,
                    ..track
                }
            };

            renderer.fill_quad(
                renderer::Quad {
                    bounds: indicator,
                    border: border::rounded(if ratios.len() > 1 { 0.0 } else { f32::MAX }),
                    ..renderer::Quad::default()
                },
                Background::Color(fade(p.primary)),
            );
        }

        let hovered = (!self.disabled)
            .then(|| self.hovered_thumb(bounds, pointer::hover(cursor)))
            .flatten();

        for index in 0..self.values.len() {
            let thumb = self.thumb(bounds, index);

            // `hover:ring-3 active:ring-3 ring-ring/50`. Its
            // `focus-visible:ring-3` never applies: the focus is on the
            // `<input>` inside the thumb.
            if state.dragging == Some(index) || (state.dragging.is_none() && hovered == Some(index))
            {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: thumb.expand(3.0),
                        border: border::rounded(f32::MAX),
                        ..renderer::Quad::default()
                    },
                    Background::Color(alpha(p.ring, 0.5)),
                );
            }

            // `size-3 rounded-full border border-ring bg-white`
            renderer.fill_quad(
                renderer::Quad {
                    bounds: thumb,
                    border: Border {
                        color: fade(p.ring),
                        width: 1.0,
                        radius: border::radius(f32::MAX),
                    },
                    ..renderer::Quad::default()
                },
                Background::Color(fade(Color::WHITE)),
            );
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();

        if self.disabled {
            if cursor.is_over(layout.bounds()) {
                mouse::Interaction::NotAllowed
            } else {
                mouse::Interaction::None
            }
        } else if state.dragging.is_some() {
            mouse::Interaction::Grabbing
        } else if cursor.is_over(layout.bounds().expand(HIT_SLOP)) && self.on_change.is_some() {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::None
        }
    }
}

impl<'a, Message: Clone + 'a> From<Slider<'a, Message>> for Element<'a, Message> {
    fn from(slider: Slider<'a, Message>) -> Self {
        Element::new(slider)
    }
}

#[cfg(test)]
mod tests {
    use super::Slider;

    #[test]
    fn steps_do_not_drift() {
        let slider = Slider::<()>::new(0.0..=1.0, vec![0.3]).step(0.1);

        assert_eq!(slider.move_to(0, 0.3 + 0.1), (0, Some(0.4)));
        assert_eq!(slider.snap(0.1 + 0.2), 0.3);
        assert_eq!(slider.snap(0.66), 0.7);
        assert_eq!(slider.snap(1.2), 1.0);
    }

    #[test]
    fn steps_snap_to_the_grid_of_the_range() {
        let slider = Slider::<()>::new(1.0..=11.0, vec![3.0]).step(2.0);

        // From a value off the grid, the next step lands on it.
        assert_eq!(slider.snap(4.0 + 2.0), 7.0);
        assert_eq!(slider.snap(5.9), 5.0);
    }

    #[test]
    fn thumbs_do_not_cross() {
        let slider = Slider::<()>::new(0.0..=100.0, vec![20.0, 50.0]);

        assert_eq!(slider.move_to(0, 70.0), (0, Some(50.0)));
        assert_eq!(slider.move_to(1, 10.0), (1, Some(20.0)));
        assert_eq!(slider.move_to(1, 100.0), (1, Some(100.0)));
    }

    #[test]
    fn stacked_thumbs_part_in_the_direction_of_the_move() {
        let slider = Slider::<()>::new(0.0..=100.0, vec![50.0, 50.0]);

        assert_eq!(slider.move_to(0, 70.0), (1, Some(70.0)));
        assert_eq!(slider.move_to(1, 30.0), (0, Some(30.0)));
        assert_eq!(slider.move_to(0, 50.0), (0, None));
    }
}
