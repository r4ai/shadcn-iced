//! Wheel scrolling like in browsers: horizontal with <kbd>Shift</kbd>, and
//! smooth if opted in.
//!
//! iced's scrollables jump by the whole delta of a mouse wheel notch and
//! only turn <kbd>Shift</kbd> + wheel into a horizontal scroll for line
//! deltas (not for the pixel deltas of the web). [`Wheel`] wraps any content
//! (usually the whole view of an application) and, before its scrollables
//! see them, rewrites the wheel events:
//!
//! - <kbd>Shift</kbd> swaps the axes (except on macOS, which does it),
//! - a notch of a mouse wheel (a line delta, or a large pixel delta) scrolls
//!   100px, like in Chrome,
//! - with smooth scrolling, a notch is animated like the smooth scrolling of
//!   Chrome: an `ease-in-out` over 100–200ms (shorter for longer
//!   distances), and a notch during a scroll retargets it, keeping its
//!   velocity, so spinning the wheel scrolls faster without jerks. Small
//!   pixel deltas (touchpads) apply right away.
//!
//! Smooth scrolling is off by default. Turn it on for every [`Wheel`] with
//! [`set_smooth`], or for one of them with [`Wheel::smooth`]. Wheels can be
//! nested: a [`smooth_scroll`] around a single scroll area makes only that
//! area smooth, under a [`wheel`] wrapping the whole application (each
//! event is rewritten once).
//!
//! The scrolled scrollable stays the one under the pointer when the scroll
//! started, and a scroll that reaches the end of every scrollable is not
//! captured, so the scroll still reaches whatever is outside of the
//! content (e.g. the page around a web preview).
//!
//! iced's scrollables follow a finger, but stop dead when it lifts. A
//! [`Wheel`] also gives touch scrolling momentum, like mobile browsers and
//! iOS: a flick keeps the content scrolling at the speed the finger had,
//! slowing down exponentially (a time constant of 325ms, close to the
//! deceleration of iOS) until it stops, reaches the end, or the content is
//! touched, clicked or scrolled by a wheel again. There is no momentum while
//! animations are disabled, and nested wheels give it once (the outermost
//! one).
//!
//! ```no_run
//! # fn view<'a>(content: iced::Element<'a, ()>, list: iced::Element<'a, ()>) -> iced::Element<'a, ()> {
//! use shadcn_iced_core::primitive::wheel;
//!
//! // Every wheel smooth, from the start of the application:
//! wheel::set_smooth(true);
//!
//! // Or one part of the view only:
//! let list = wheel::smooth_scroll(list);
//! # let _ = list;
//!
//! wheel::wheel(content).into()
//! # }
//! ```
use crate::animation;
use crate::primitive::pointer;

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Id, Operation, Tree, Widget, operation, tree};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::time::{Duration, Instant};
use iced::{
    Element, Event, Length, Point, Rectangle, Size, Vector, keyboard, mouse, touch, window,
};

use std::cell::Cell;
use std::sync::atomic::{self, AtomicBool};

/// The pixels of a line delta: a notch scrolls 100px in Chrome.
const LINE: f32 = 100.0;

/// Pixel deltas from this size are wheel notches (browsers report about
/// 100px per notch), smaller ones come from touchpads.
const NOTCH: f32 = 50.0;

/// The duration of a frame, for the first step of a scroll.
const FRAME: Duration = Duration::from_millis(16);

/// The control points of Chrome's `ease-in-out` (`cubic-bezier(0.42, 0,
/// 0.58, 1)`).
const X1: f32 = 0.42;
const X2: f32 = 0.58;

/// How far back the velocity of a finger is measured when it lifts.
const VELOCITY_WINDOW: Duration = Duration::from_millis(100);

/// The time constant of the deceleration of a fling, in seconds: the
/// velocity falls to about a third every 325ms.
const DECELERATION: f32 = 0.325;

/// The slowest flick that flings, in pixels per second (0.3px/ms).
const MIN_FLING: f32 = 300.0;

/// A fling stops under this velocity, in pixels per second (0.02px/ms).
const STOP_FLING: f32 = 20.0;

/// The fastest fling, in pixels per second.
const MAX_FLING: f32 = 8000.0;

static SMOOTH: AtomicBool = AtomicBool::new(false);

/// Turns smooth scrolling on or off for every [`Wheel`] that does not set
/// it with [`Wheel::smooth`] (off by default).
pub fn set_smooth(smooth: bool) {
    SMOOTH.store(smooth, atomic::Ordering::Relaxed);
}

/// Returns whether wheels scroll smoothly unless they say otherwise.
pub fn smooth() -> bool {
    SMOOTH.load(atomic::Ordering::Relaxed)
}

/// Browser-like wheel scrolling for some content. See the
/// [module documentation](self).
pub struct Wheel<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    smooth: Option<bool>,
}

/// Wraps `content` in a [`Wheel`], smooth if [`set_smooth`] says so.
pub fn wheel<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Wheel<'a, Message, Theme, Renderer> {
    Wheel {
        content: content.into(),
        smooth: None,
    }
}

/// Wraps `content` in a smooth [`Wheel`], whatever [`set_smooth`] says,
/// e.g. to make a single scroll area smooth.
pub fn smooth_scroll<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Wheel<'a, Message, Theme, Renderer> {
    wheel(content).smooth(true)
}

impl<Message, Theme, Renderer> Wheel<'_, Message, Theme, Renderer> {
    /// Sets whether wheel notches scroll smoothly, overriding
    /// [`set_smooth`]. Scrolling is never smooth while animations are
    /// disabled.
    pub fn smooth(mut self, smooth: bool) -> Self {
        self.smooth = Some(smooth);
        self
    }
}

/// What the wheels around the one handling an event did to it.
#[derive(Debug, Clone, Copy, Default)]
struct Handled {
    /// The event comes from an outer wheel, which swapped the axes for
    /// `Shift` and turned lines into pixels.
    rewritten: bool,
    /// Whether the original event was a notch.
    notch: bool,
    /// The event is a step of a smooth scroll.
    smoothed: bool,
}

thread_local! {
    static HANDLED: Cell<Handled> = Cell::new(Handled::default());
}

/// Runs `f` with the wheel events it dispatches marked as `handled`.
fn handled<T>(handled: Handled, f: impl FnOnce() -> T) -> T {
    let outer = HANDLED.replace(handled);
    let result = f();
    HANDLED.set(outer);
    result
}

/// A smooth scroll in progress: a segment from where the content was when
/// it started (or was retargeted) to `delta` further.
#[derive(Debug, Clone, Copy)]
struct Scroll {
    /// The distance of the segment, in pixels (wheel direction).
    delta: Vector,
    /// The distance scrolled so far.
    scrolled: Vector,
    start: Instant,
    duration: f32,
    /// The initial velocity, relative to the average one (0 for an
    /// `ease-in-out`).
    slope: f32,
    /// Where the pointer was when the scroll started.
    position: Point,
}

impl Scroll {
    fn new(delta: Vector, slope: f32, start: Instant, position: Point) -> Self {
        let duration = duration(delta);

        Self {
            delta,
            scrolled: Vector::ZERO,
            start,
            duration,
            slope: slope.clamp(0.0, 1.0 / X1),
            position,
        }
    }

    /// The linear progress of the segment at `now`, from 0 to 1.
    fn time(&self, now: Instant) -> f32 {
        let elapsed = now.saturating_duration_since(self.start).as_secs_f32();

        (elapsed / self.duration).clamp(0.0, 1.0)
    }

    /// The eased progress at the linear progress `t`: Chrome's
    /// `ease-in-out`, or an ease-out starting at the velocity of a
    /// retargeted scroll.
    fn progress(&self, t: f32) -> f32 {
        animation::cubic_bezier(X1, X1 * self.slope, X2, 1.0, t)
    }

    /// The distance the content should have scrolled at `now`.
    fn target(&self, now: Instant) -> Vector {
        self.delta * self.progress(self.time(now))
    }

    /// The velocity at `now`, in pixels per second.
    fn velocity(&self, now: Instant) -> Vector {
        let t = self.time(now);
        let h = 0.01;
        let (a, b) = ((t - h).max(0.0), (t + h).min(1.0));

        if b <= a {
            return Vector::ZERO;
        }

        self.delta * ((self.progress(b) - self.progress(a)) / (b - a) / self.duration)
    }

    /// Whether the segment is over at `now`.
    fn is_done(&self, now: Instant) -> bool {
        self.time(now) >= 1.0
    }

    /// Takes the distance to scroll in the frame at `now`, if any.
    ///
    /// Steps under half a pixel may not move the content, so they are held
    /// back until they add up to more, and what is left is taken when the
    /// segment is over, so that the whole `delta` is scrolled.
    fn step(&mut self, now: Instant) -> Option<Vector> {
        let target = if self.is_done(now) {
            self.delta
        } else {
            self.target(now)
        };
        let step = target - self.scrolled;
        let size = step.x.abs().max(step.y.abs());

        if size == 0.0 || (size < 0.5 && !self.is_done(now)) {
            return None;
        }

        self.scrolled = target;

        Some(step)
    }

    /// Adds `pixels` to the scroll at `now`, keeping its velocity, like
    /// Chrome does for a notch during a smooth scroll.
    fn retarget(&mut self, pixels: Vector, now: Instant) {
        let remaining = self.delta - self.scrolled + pixels;
        let velocity = self.velocity(now);
        let duration = duration(remaining);

        // The velocity relative to the average one of the new segment,
        // along its main axis (0 when going the other way).
        let slope = if remaining.x.abs() > remaining.y.abs() {
            velocity.x * duration / remaining.x
        } else if remaining.y != 0.0 {
            velocity.y * duration / remaining.y
        } else {
            0.0
        };

        *self = Self::new(remaining, slope, now, self.position);
    }
}

/// The duration of a smooth scroll of `delta`, in seconds: Chrome's
/// "inverse delta" durations, from 200ms up to 120px down to 100ms from
/// 480px.
fn duration(delta: Vector) -> f32 {
    const MIN: f32 = 0.1;
    const MAX: f32 = 0.2;
    const RAMP_START: f32 = 120.0;
    const RAMP_END: f32 = 480.0;

    let distance = delta.x.abs().max(delta.y.abs());
    let ramp = ((distance - RAMP_START) / (RAMP_END - RAMP_START)).clamp(0.0, 1.0);

    MAX - (MAX - MIN) * ramp
}

/// A finger followed for the momentum of its scroll.
#[derive(Debug, Clone)]
struct Touch {
    finger: touch::Finger,
    /// Where the content was touched: the fling scrolls what is there.
    origin: Point,
    /// Where the finger was lately, and when (at most
    /// [`VELOCITY_WINDOW`] back).
    samples: Vec<(Instant, Point)>,
    /// The translations of the scrollables when it pressed, to tell whether
    /// it scrolled any (and was not, say, dragging a slider).
    translations: Vec<Vector>,
}

impl Touch {
    /// Records the finger at `position`, forgetting what is too old to tell
    /// its velocity.
    fn record(&mut self, position: Point, now: Instant) {
        self.forget(now);
        self.samples.push((now, position));
    }

    fn forget(&mut self, now: Instant) {
        self.samples
            .retain(|(at, _)| now.saturating_duration_since(*at) <= VELOCITY_WINDOW);
    }

    /// The velocity of the finger over the last [`VELOCITY_WINDOW`] up to
    /// `now`, in pixels per second. A finger that rested before lifting is
    /// slow, since the time it rested counts.
    fn velocity(&mut self, now: Instant) -> Vector {
        self.forget(now);

        let (Some((start, from)), Some((_, to))) = (self.samples.first(), self.samples.last())
        else {
            return Vector::ZERO;
        };

        // Touch events come a frame apart at least: closer ones are not
        // faster.
        let elapsed = now.saturating_duration_since(*start).max(FRAME);
        let velocity = (*to - *from) * (1.0 / elapsed.as_secs_f32());
        let speed = velocity.x.hypot(velocity.y);

        if speed > MAX_FLING {
            velocity * (MAX_FLING / speed)
        } else {
            velocity
        }
    }
}

/// The momentum of a touch scroll, after its finger lifted.
#[derive(Debug, Clone, Copy)]
struct Fling {
    /// In pixels per second, in the direction of the wheel (the one the
    /// finger moved in).
    velocity: Vector,
    /// The last frame; none before the first.
    last: Option<Instant>,
    position: Point,
    /// What was too small to scroll yet.
    pending: Vector,
}

impl Fling {
    /// Takes the distance to scroll in the frame at `now`, and slows down.
    fn step(&mut self, now: Instant) -> Vector {
        let elapsed = self
            .last
            .map_or(FRAME, |last| now.saturating_duration_since(last))
            .as_secs_f32();
        let decay = (-elapsed / DECELERATION).exp();

        // The integral of the exponentially decaying velocity.
        let step = self.velocity * (DECELERATION * (1.0 - decay)) + self.pending;

        self.velocity *= decay;
        self.last = Some(now);
        self.pending = Vector::ZERO;

        step
    }

    fn is_over(&self) -> bool {
        self.velocity.x.hypot(self.velocity.y) < STOP_FLING
    }
}

#[derive(Debug, Default)]
struct State {
    modifiers: keyboard::Modifiers,
    scroll: Option<Scroll>,
    touch: Option<Touch>,
    fling: Option<Fling>,
}

thread_local! {
    /// Whether a wheel handling a touch event is dispatching it, so that
    /// the wheels within it leave the momentum to it.
    static FOLLOWING: Cell<bool> = const { Cell::new(false) };
}

/// Reads the translations of every scrollable.
struct Translations(Vec<Vector>);

impl Operation for Translations {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn scrollable(
        &mut self,
        _id: Option<&Id>,
        _bounds: Rectangle,
        _content_bounds: Rectangle,
        translation: Vector,
        _state: &mut dyn operation::Scrollable,
    ) {
        self.0.push(translation);
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Wheel<'_, Message, Theme, Renderer>
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

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
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
        pointer::observe(event);

        let state = tree.state.downcast_mut::<State>();

        if let Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) = event {
            state.modifiers = *modifiers;
        }

        // A click or a wheel stops the momentum.
        if matches!(
            event,
            Event::Mouse(mouse::Event::ButtonPressed(_) | mouse::Event::WheelScrolled { .. })
        ) {
            state.fling = None;
        }

        // The outermost wheel follows the fingers.
        if let Event::Touch(touch) = event
            && !FOLLOWING.get()
        {
            FOLLOWING.set(true);
            self::touch(
                &mut self.content,
                tree,
                event,
                *touch,
                layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
            FOLLOWING.set(false);

            return;
        }

        let Event::Mouse(mouse::Event::WheelScrolled { delta }) = event else {
            self.content.as_widget_mut().update(
                &mut tree.children[0],
                event,
                layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );

            if let Event::Window(window::Event::RedrawRequested(now)) = event {
                frame(
                    &mut self.content,
                    tree,
                    *now,
                    layout,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                );

                fling(
                    &mut self.content,
                    tree,
                    *now,
                    layout,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                );
            }

            return;
        };

        if shell.is_event_captured() {
            return;
        }

        let outer = HANDLED.get();

        // A step of a smooth scroll of an outer wheel: already rewritten.
        if outer.smoothed {
            self.content.as_widget_mut().update(
                &mut tree.children[0],
                event,
                layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );

            return;
        }

        let (pixels, notch) = if outer.rewritten {
            let pixels = match *delta {
                mouse::ScrollDelta::Lines { x, y } => Vector::new(x, y) * LINE,
                mouse::ScrollDelta::Pixels { x, y } => Vector::new(x, y),
            };

            (pixels, outer.notch)
        } else {
            let (pixels, notch) = match *delta {
                mouse::ScrollDelta::Lines { x, y } => (Vector::new(x, y) * LINE, true),
                mouse::ScrollDelta::Pixels { x, y } => {
                    (Vector::new(x, y), x.abs().max(y.abs()) >= NOTCH)
                }
            };

            // macOS swaps the axes of `Shift` + wheel itself.
            let pixels = if state.modifiers.shift() && !cfg!(target_os = "macos") {
                Vector::new(pixels.y, pixels.x)
            } else {
                pixels
            };

            (pixels, notch)
        };

        let smooth = self.smooth.unwrap_or_else(smooth) && notch && animation::enabled();

        // A new scroll somewhere else (or not smooth) ends the one in
        // progress.
        if let Some(scroll) = state.scroll
            && (!smooth || cursor.position() != Some(scroll.position))
        {
            state.scroll = None;

            let _ = self::scroll(
                &mut self.content,
                &mut tree.children[0],
                scroll.delta - scroll.scrolled,
                scroll.position,
                layout,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }

        let state = tree.state.downcast_mut::<State>();

        if !smooth {
            handled(
                Handled {
                    rewritten: true,
                    notch,
                    smoothed: false,
                },
                || {
                    self.content.as_widget_mut().update(
                        &mut tree.children[0],
                        &Event::Mouse(mouse::Event::WheelScrolled {
                            delta: mouse::ScrollDelta::Pixels {
                                x: pixels.x,
                                y: pixels.y,
                            },
                        }),
                        layout,
                        cursor,
                        renderer,
                        clipboard,
                        shell,
                        viewport,
                    );
                },
            );

            return;
        }

        let Some(position) = cursor.position() else {
            return;
        };

        let now = Instant::now();

        // A scroll in progress goes on further, faster.
        if let Some(scroll) = &mut state.scroll {
            scroll.retarget(pixels, now);
            shell.capture_event();
            shell.request_redraw();
            return;
        }

        // The first frame of the scroll, right away: whether something
        // captures it tells whether anything can scroll.
        let mut scroll = Scroll::new(pixels, 0.0, now - FRAME, position);
        let first = scroll.target(now);

        // At least a pixel, for the scrollable to tell whether it scrolls.
        let first = Vector::new(
            at_least_a_pixel(first.x, pixels.x),
            at_least_a_pixel(first.y, pixels.y),
        );

        let captured = self::scroll(
            &mut self.content,
            &mut tree.children[0],
            first,
            position,
            layout,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        if captured {
            scroll.scrolled = first;

            tree.state.downcast_mut::<State>().scroll = Some(scroll);
            shell.capture_event();
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
            layout,
            cursor,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
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
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

/// `step`, or a pixel in the direction of `delta` if it is smaller, but
/// never further than `delta`.
fn at_least_a_pixel(step: f32, delta: f32) -> f32 {
    if delta == 0.0 || step.abs() >= 1.0 {
        step
    } else {
        delta.signum() * delta.abs().min(1.0)
    }
}

/// Scrolls a frame of the scroll in progress, if any.
fn frame<Message, Theme, Renderer>(
    content: &mut Element<'_, Message, Theme, Renderer>,
    tree: &mut Tree,
    now: Instant,
    layout: Layout<'_>,
    renderer: &Renderer,
    clipboard: &mut dyn Clipboard,
    shell: &mut Shell<'_, Message>,
    viewport: &Rectangle,
) where
    Renderer: iced::advanced::Renderer,
{
    let state = tree.state.downcast_mut::<State>();

    let Some(scroll) = &mut state.scroll else {
        return;
    };

    let done = scroll.is_done(now);
    let step = scroll.step(now);
    let position = scroll.position;

    // A scroll that reaches the end stops.
    let scrolled = step.is_none_or(|step| {
        self::scroll(
            content,
            &mut tree.children[0],
            step,
            position,
            layout,
            renderer,
            clipboard,
            shell,
            viewport,
        )
    });

    let state = tree.state.downcast_mut::<State>();

    if done || !scrolled {
        state.scroll = None;
    } else {
        shell.request_redraw();
    }
}

/// Reads the translations of the scrollables of `content`.
fn translations<Message, Theme, Renderer>(
    content: &mut Element<'_, Message, Theme, Renderer>,
    tree: &mut Tree,
    layout: Layout<'_>,
    renderer: &Renderer,
) -> Vec<Vector>
where
    Renderer: iced::advanced::Renderer,
{
    let mut translations = Translations(Vec::new());

    content
        .as_widget_mut()
        .operate(tree, layout, renderer, &mut translations);

    translations.0
}

/// Dispatches a touch event to the content, following the first finger to
/// fling the content when it lifts after a flick.
fn touch<Message, Theme, Renderer>(
    content: &mut Element<'_, Message, Theme, Renderer>,
    tree: &mut Tree,
    event: &Event,
    touch: touch::Event,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    renderer: &Renderer,
    clipboard: &mut dyn Clipboard,
    shell: &mut Shell<'_, Message>,
    viewport: &Rectangle,
) where
    Renderer: iced::advanced::Renderer,
{
    let now = Instant::now();
    let pressed = matches!(touch, touch::Event::FingerPressed { .. });
    let before = pressed.then(|| translations(content, &mut tree.children[0], layout, renderer));

    content.as_widget_mut().update(
        &mut tree.children[0],
        event,
        layout,
        cursor,
        renderer,
        clipboard,
        shell,
        viewport,
    );

    let state = tree.state.downcast_mut::<State>();

    match touch {
        touch::Event::FingerPressed { id, position } => {
            // A finger on the content stops it.
            state.fling = None;

            if state.touch.is_none() {
                state.touch = Some(Touch {
                    finger: id,
                    origin: cursor.position().unwrap_or(position),
                    samples: vec![(now, position)],
                    translations: before.unwrap_or_default(),
                });
            }
        }
        touch::Event::FingerMoved { id, position } => {
            if let Some(touch) = state.touch.as_mut().filter(|touch| touch.finger == id) {
                touch.record(position, now);
            }
        }
        touch::Event::FingerLost { id, .. } => {
            if state.touch.as_ref().is_some_and(|touch| touch.finger == id) {
                state.touch = None;
            }
        }
        touch::Event::FingerLifted { id, .. } => {
            if !state.touch.as_ref().is_some_and(|touch| touch.finger == id) {
                return;
            }

            let Some(mut touch) = state.touch.take() else {
                return;
            };

            let velocity = touch.velocity(now);
            let flick = velocity.x.hypot(velocity.y) >= MIN_FLING;

            // Only a finger that scrolled something flings it.
            if flick
                && animation::enabled()
                && translations(content, &mut tree.children[0], layout, renderer)
                    != touch.translations
            {
                tree.state.downcast_mut::<State>().fling = Some(Fling {
                    velocity,
                    last: None,
                    position: touch.origin,
                    pending: Vector::ZERO,
                });
                shell.request_redraw();
            }
        }
    }
}

/// Scrolls a frame of the fling in progress, if any.
fn fling<Message, Theme, Renderer>(
    content: &mut Element<'_, Message, Theme, Renderer>,
    tree: &mut Tree,
    now: Instant,
    layout: Layout<'_>,
    renderer: &Renderer,
    clipboard: &mut dyn Clipboard,
    shell: &mut Shell<'_, Message>,
    viewport: &Rectangle,
) where
    Renderer: iced::advanced::Renderer,
{
    let state = tree.state.downcast_mut::<State>();

    let Some(fling) = &mut state.fling else {
        return;
    };

    if !animation::enabled() {
        state.fling = None;
        return;
    }

    let step = fling.step(now);
    let position = fling.position;
    let over = fling.is_over();

    // Steps under half a pixel may not move the content: they add up.
    let small = step.x.abs().max(step.y.abs()) < 0.5;
    let mut moved = true;

    if small {
        fling.pending = step;
    } else {
        let before = translations(content, &mut tree.children[0], layout, renderer);

        let _ = scroll(
            content,
            &mut tree.children[0],
            step,
            position,
            layout,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        // A scroll that reaches the end stops.
        moved = translations(content, &mut tree.children[0], layout, renderer) != before;
    }

    if over || !moved {
        tree.state.downcast_mut::<State>().fling = None;
    } else {
        shell.request_redraw();
    }
}

/// Scrolls the content under `position` by `pixels` (in the direction of
/// the wheel), returning whether something scrolled.
fn scroll<Message, Theme, Renderer>(
    content: &mut Element<'_, Message, Theme, Renderer>,
    tree: &mut Tree,
    pixels: Vector,
    position: Point,
    layout: Layout<'_>,
    renderer: &Renderer,
    clipboard: &mut dyn Clipboard,
    shell: &mut Shell<'_, Message>,
    viewport: &Rectangle,
) -> bool
where
    Renderer: iced::advanced::Renderer,
{
    let mut messages = Vec::new();
    let mut inner = Shell::new(&mut messages);

    handled(
        Handled {
            rewritten: true,
            notch: true,
            smoothed: true,
        },
        || {
            content.as_widget_mut().update(
                tree,
                &Event::Mouse(mouse::Event::WheelScrolled {
                    delta: mouse::ScrollDelta::Pixels {
                        x: pixels.x,
                        y: pixels.y,
                    },
                }),
                layout,
                mouse::Cursor::Available(position),
                renderer,
                clipboard,
                &mut inner,
                viewport,
            );
        },
    );

    let captured = inner.is_event_captured();

    // The capture is of the synthetic event, not of the current one.
    let redraw = inner.redraw_request();
    let layout_invalid = inner.is_layout_invalid();
    let widgets_invalid = inner.are_widgets_invalid();
    drop(inner);

    for message in messages {
        shell.publish(message);
    }

    if layout_invalid {
        shell.invalidate_layout();
    }

    if widgets_invalid {
        shell.invalidate_widgets();
    }

    match redraw {
        window::RedrawRequest::NextFrame => shell.request_redraw(),
        window::RedrawRequest::At(at) => shell.request_redraw_at(at),
        window::RedrawRequest::Wait => {}
    }

    captured
}

impl<'a, Message, Theme, Renderer> From<Wheel<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(wheel: Wheel<'a, Message, Theme, Renderer>) -> Self {
        Element::new(wheel)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// Scrolls a whole segment of `delta` frame by frame, returning the
    /// distance sent to the content.
    fn sent(delta: Vector, frame: Duration) -> Vector {
        let start = Instant::now();
        let mut scroll = Scroll::new(delta, 0.0, start, Point::ORIGIN);
        let mut sent = Vector::ZERO;
        let mut now = start;

        while !scroll.is_done(now) {
            now += frame;

            if let Some(step) = scroll.step(now) {
                sent += step;
            }
        }

        sent
    }

    #[test]
    fn a_scroll_sends_its_whole_distance() {
        for frame in [1, 4, 16] {
            let sent = sent(Vector::new(0.0, 100.0), Duration::from_millis(frame));

            assert!((sent.y - 100.0).abs() < 1e-3, "{frame}ms: {sent:?}");
        }
    }

    #[test]
    fn a_short_scroll_sends_its_whole_distance() {
        let sent = sent(Vector::new(0.0, -3.0), Duration::from_millis(1));

        assert!((sent.y + 3.0).abs() < 1e-3, "{sent:?}");
    }
}
