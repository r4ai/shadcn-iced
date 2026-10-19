//! A stack of toasts that behaves like [sonner](https://sonner.emilkowal.ski).
//!
//! [`Toaster`] is the headless part of the Sonner and Toast components: it measures the
//! toasts, stacks them, animates them in and out, expands them on hover,
//! dismisses them once their duration has elapsed and lets them be swiped
//! away. The look of a toast is its content plus a [`Toaster::surface`].
//!
//! It ports the model of sonner 2: every toast is anchored to the edge of
//! the toaster and moved with transforms (`translateY` and `scale`), its
//! height and opacity, which all follow CSS transitions (`400ms ease`)
//! retargeted whenever the stack changes:
//!
//! - a new toast slides in from the edge (`translateY(100%)`) and fades in;
//! - while collapsed, the toasts behind the front one take its height, hide
//!   their content and peek out by the gap, 5% smaller for each toast in
//!   front of them, and only the first three are visible;
//! - hovering the stack expands it into a list (and pauses the timers);
//! - a dismissed toast slides back towards the edge and fades out, and is
//!   unmounted (reported with `on_dismiss`) 200ms later.
//!
//! The content of a toast produces [`Action`]s, so that its buttons can
//! dismiss it with the exit animation.
//!
//! These are the motions of sonner, the defaults; a [`Motion`] changes how
//! the toasts enter and leave and how the collapsed stack is drawn, which is
//! how the toasts of Base UI are made from the same stack.
use super::box_shadow;
use crate::animation::{self, Easing, Fade, Transition};

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::time::{Duration, Instant};
use iced::widget::container;
use iced::{
    Background, Color, Element, Event, Length, Point, Rectangle, Size, Transformation, Vector,
    mouse, touch, window,
};

use std::collections::HashMap;

/// The default gap between toasts (sonner's `GAP`).
pub const GAP: f32 = 14.0;

/// The default number of visible toasts (`VISIBLE_TOASTS_AMOUNT`).
pub const VISIBLE: usize = 3;

/// The default lifetime of a toast (`TOAST_LIFETIME`).
pub const LIFETIME: Duration = Duration::from_secs(4);

/// How long a dismissed toast stays before being unmounted
/// (`TIME_BEFORE_UNMOUNT`).
const UNMOUNT: Duration = Duration::from_millis(200);

/// The duration of the transitions of a toast (`transition: transform 400ms,
/// opacity 400ms, height 400ms`).
const TRANSITION: Duration = Duration::from_millis(400);

/// The duration of the swipe-out animation (`swipe-out-*: 200ms`).
const SWIPE_OUT: Duration = Duration::from_millis(200);

/// How far a toast must be swiped to be dismissed (`SWIPE_THRESHOLD`).
const SWIPE_THRESHOLD: f32 = 45.0;

/// How fast a toast must be swiped to be dismissed, in pixels per
/// millisecond.
const SWIPE_VELOCITY: f32 = 0.11;

/// How much each toast behind the front one shrinks.
const SCALE_STEP: f32 = 0.05;

/// How far the shadow of a toast reaches outside of it.
const SHADOW_EXTENT: f32 = 32.0;

/// The edge of the window the toasts are stacked against.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Edge {
    /// The newest toast is at the top.
    Top,
    /// The newest toast is at the bottom.
    #[default]
    Bottom,
}

/// A direction a toast can be swiped away in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    /// Upwards.
    Up,
    /// To the right.
    Right,
    /// Downwards.
    Down,
    /// To the left.
    Left,
}

/// The point of a toast that it is scaled around.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Origin {
    /// The center (`transform-origin: center`).
    #[default]
    Center,
    /// The middle of its top edge (`transform-origin: top`).
    Top,
    /// The middle of its bottom edge (`transform-origin: bottom`).
    Bottom,
}

/// How the toasts of a [`Toaster`] move. The default is sonner's.
///
/// ```
/// use shadcn_iced_core::primitive::toaster::{Motion, Origin};
///
/// // Toasts that slide in from further away without fading, and whose
/// // stack shrinks from the top, like Base UI's.
/// let motion = Motion {
///     travel: 1.5,
///     fade: false,
///     shrink: 0.1,
///     origin: Origin::Top,
///     ..Motion::default()
/// };
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Motion {
    /// How far a toast slides from the edge when it enters, and towards it
    /// when it leaves, in its own height (`translateY(100%)`).
    pub travel: f32,
    /// Whether a toast fades in and out as it enters and leaves.
    pub fade: bool,
    /// How long a toast takes to enter or leave.
    pub presence: Duration,
    /// How long the other transitions take: stacking, expanding and
    /// collapsing.
    pub duration: Duration,
    /// The easing of every transition.
    pub easing: Easing,
    /// How far each toast behind the front one peeks out of the collapsed
    /// stack; `None` uses the gap.
    pub peek: Option<f32>,
    /// How much each toast behind the front one shrinks, as a fraction of
    /// its size.
    pub shrink: f32,
    /// The point the toasts behind the front one are scaled around.
    pub origin: Origin,
    /// How long a dismissed toast stays before `on_dismiss` is published;
    /// at least as long as `presence` to let it leave.
    pub unmount: Duration,
}

impl Default for Motion {
    fn default() -> Self {
        Self {
            travel: 1.0,
            fade: true,
            presence: TRANSITION,
            duration: TRANSITION,
            easing: animation::ease,
            peek: None,
            shrink: SCALE_STEP,
            origin: Origin::Center,
            unmount: UNMOUNT,
        }
    }
}

/// What the content of a toast asks the [`Toaster`] to do.
#[derive(Debug, Clone)]
pub enum Action<Message> {
    /// Publishes a message of the application.
    Publish(Message),
    /// Dismisses the toast, like its close button.
    Dismiss,
    /// Publishes a message, then dismisses the toast, like its action
    /// button.
    PublishAndDismiss(Message),
}

/// A toast of a [`Toaster`].
pub struct Toast<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    id: u64,
    duration: Option<Duration>,
    dismissible: bool,
    content: Element<'a, Action<Message>, Theme, Renderer>,
}

impl<'a, Message, Theme, Renderer> Toast<'a, Message, Theme, Renderer> {
    /// Creates a toast, identified by `id`, which lasts 4 seconds.
    pub fn new(id: u64, content: impl Into<Element<'a, Action<Message>, Theme, Renderer>>) -> Self {
        Self {
            id,
            duration: Some(LIFETIME),
            dismissible: true,
            content: content.into(),
        }
    }

    /// Sets how long the toast stays on screen; `None` keeps it until it
    /// is dismissed.
    pub fn duration(mut self, duration: Option<Duration>) -> Self {
        self.duration = duration;
        self
    }

    /// Sets whether the toast can be swiped away (`dismissible`).
    pub fn dismissible(mut self, dismissible: bool) -> Self {
        self.dismissible = dismissible;
        self
    }
}

/// A stack of toasts. See the [module documentation](self).
pub struct Toaster<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    toasts: Vec<Toast<'a, Message, Theme, Renderer>>,
    on_dismiss: Box<dyn Fn(u64) -> Message + 'a>,
    edge: Edge,
    swipe: Vec<Direction>,
    gap: f32,
    visible: usize,
    expand: bool,
    motion: Motion,
    surface: Box<dyn Fn(&Theme) -> container::Style + 'a>,
    shadows: &'a [box_shadow::Layer],
}

impl<'a, Message, Theme, Renderer> Toaster<'a, Message, Theme, Renderer> {
    /// Creates a toaster showing `toasts`, oldest first, which reports
    /// with `on_dismiss` the toasts that must be removed.
    pub fn new(
        toasts: Vec<Toast<'a, Message, Theme, Renderer>>,
        on_dismiss: impl Fn(u64) -> Message + 'a,
    ) -> Self {
        Self {
            toasts,
            on_dismiss: Box::new(on_dismiss),
            edge: Edge::Bottom,
            swipe: vec![Direction::Down, Direction::Right],
            gap: GAP,
            visible: VISIBLE,
            expand: false,
            motion: Motion::default(),
            surface: Box::new(|_| container::Style::default()),
            shadows: &[],
        }
    }

    /// Sets the edge the toasts are stacked against.
    pub fn edge(mut self, edge: Edge) -> Self {
        self.edge = edge;
        self
    }

    /// Sets the directions the toasts can be swiped away in
    /// (`swipeDirections`).
    pub fn swipe_directions(mut self, directions: impl IntoIterator<Item = Direction>) -> Self {
        self.swipe = directions.into_iter().collect();
        self
    }

    /// Sets the gap between toasts (`gap`).
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Sets how many toasts are visible at once (`visibleToasts`).
    pub fn visible(mut self, visible: usize) -> Self {
        self.visible = visible.max(1);
        self
    }

    /// Always shows the toasts expanded (`expand`).
    pub fn expand(mut self, expand: bool) -> Self {
        self.expand = expand;
        self
    }

    /// Sets how the toasts move.
    pub fn motion(mut self, motion: Motion) -> Self {
        self.motion = motion;
        self
    }

    /// Sets the background, border, shadow and text color of the toasts.
    pub fn surface(mut self, surface: impl Fn(&Theme) -> container::Style + 'a) -> Self {
        self.surface = Box::new(surface);
        self
    }

    /// Draws layered shadows behind the toasts, like
    /// [`box_shadow`](super::box_shadow), in addition to the shadow of the
    /// [surface](Self::surface).
    pub fn shadows(mut self, shadows: &'a [box_shadow::Layer]) -> Self {
        self.shadows = shadows;
        self
    }

    /// `--lift`: the direction the stack grows in.
    fn lift(&self) -> f32 {
        match self.edge {
            Edge::Top => 1.0,
            Edge::Bottom => -1.0,
        }
    }
}

/// The state of a toast that has been dismissed.
#[derive(Debug, Clone, Copy)]
struct Removal {
    at: Instant,
    swiped: bool,
}

/// The animated state of a toast.
#[derive(Debug)]
struct Entry {
    /// The natural height of the toast.
    height: f32,
    /// Whether the enter transition has started (`data-mounted`).
    mounted: bool,
    /// `translateY` of `--y`.
    y: Transition,
    /// `scale` of `--y`.
    scale: Transition,
    /// The opacity of the toast.
    opacity: Transition,
    /// The opacity of its content (`[data-sonner-toast] > *`).
    content: Transition,
    /// The height of its box.
    size: Transition,
    /// `--swipe-amount-x` and `--swipe-amount-y`.
    swipe: (Transition, Transition),
    /// `--offset`: how far the toast is from the edge when expanded.
    offset: f32,
    /// How long the timer has run.
    elapsed: Duration,
    removed: Option<Removal>,
    /// Whether `on_dismiss` has been published.
    unmounted: bool,
}

impl Entry {
    fn new() -> Self {
        Self {
            height: 0.0,
            mounted: false,
            y: Transition::new(0.0),
            scale: Transition::new(1.0),
            opacity: Transition::new(0.0),
            content: Transition::new(1.0),
            size: Transition::new(0.0),
            swipe: (Transition::new(0.0), Transition::new(0.0)),
            offset: 0.0,
            elapsed: Duration::ZERO,
            removed: None,
            unmounted: false,
        }
    }

    fn is_animating(&self, now: Instant) -> bool {
        [
            &self.y,
            &self.scale,
            &self.opacity,
            &self.content,
            &self.size,
            &self.swipe.0,
            &self.swipe.1,
        ]
        .iter()
        .any(|transition| transition.is_animating(now))
    }
}

/// The axis a swipe is locked to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Axis {
    X,
    Y,
}

/// A toast being swiped.
#[derive(Debug, Clone, Copy)]
struct Drag {
    id: u64,
    start: Point,
    at: Instant,
    axis: Option<Axis>,
}

#[derive(Debug, Default)]
struct State {
    ids: Vec<u64>,
    entries: HashMap<u64, Entry>,
    /// Whether the cursor is over the toasts (`expanded`).
    expanded: bool,
    /// Whether a pointer is pressed on the toasts (`interacting`).
    interacting: bool,
    drag: Option<Drag>,
    /// The last time the timers were advanced.
    last: Option<Instant>,
    width: f32,
}

impl State {
    fn paused(&self) -> bool {
        self.expanded || self.interacting
    }

    /// Advances the timers of the toasts to `now`.
    fn tick(&mut self, now: Instant) {
        let delta = self
            .last
            .map(|last| now.saturating_duration_since(last))
            .unwrap_or_default();

        self.last = Some(now);

        if self.paused() {
            return;
        }

        for entry in self.entries.values_mut() {
            if entry.removed.is_none() {
                entry.elapsed += delta;
            }
        }
    }

    /// Keeps an entry for each of the `ids`, and only for them.
    ///
    /// The timers are advanced to `now` first, so that the new toasts start
    /// their timers at `now` rather than at the last frame, which may be long
    /// ago when no toast was shown.
    fn sync(&mut self, ids: &[u64], now: Instant) {
        self.tick(now);
        self.entries.retain(|id, _| ids.contains(id));

        for id in ids {
            self.entries.entry(*id).or_insert_with(Entry::new);
        }
    }
}

/// Where a toast is drawn at some point in time.
struct Placement {
    /// The box of the toast before its transformation.
    bounds: Rectangle,
    /// The transformation of the box.
    transformation: Transformation,
    /// The transformation of the content of the toast, centered in the box.
    content: Transformation,
    /// The box after its transformation.
    visible: Rectangle,
}

impl<'a, Message, Theme, Renderer> Toaster<'a, Message, Theme, Renderer> {
    /// The ids of the mounted toasts, newest first, as sonner indexes them.
    fn live(&self, state: &State) -> Vec<u64> {
        self.toasts
            .iter()
            .rev()
            .map(|toast| toast.id)
            .filter(|id| state.entries.get(id).is_some_and(|entry| !entry.unmounted))
            .collect()
    }

    /// Retargets the transitions of every toast, like CSS transitions do
    /// when the attributes of the toasts change.
    fn refresh(&self, state: &mut State, now: Instant) {
        let lift = self.lift();
        let expanded = state.expanded || self.expand;
        let live = self.live(state);

        // `heights`: the toasts that have not been dismissed.
        let heights: Vec<(u64, f32)> = live
            .iter()
            .filter_map(|id| {
                let entry = &state.entries[id];
                entry.removed.is_none().then_some((*id, entry.height))
            })
            .collect();
        let front_height = heights.first().map_or(0.0, |(_, height)| *height);

        let mut before = 0.0;
        for (index, (id, height)) in heights.iter().enumerate() {
            if let Some(entry) = state.entries.get_mut(id) {
                entry.offset = index as f32 * self.gap + before;
            }
            before += height;
        }

        let motion = self.motion;
        let ease = motion.easing;
        let peek = motion.peek.unwrap_or(self.gap);

        for (index, id) in live.iter().enumerate() {
            let Some(entry) = state.entries.get_mut(id) else {
                continue;
            };

            // The transitions started on the first frame are the enter
            // animation.
            let entering = !entry.mounted;

            if entering {
                // The first frame: `translateY(100%)`, transparent.
                entry.mounted = true;
                entry.y.set(-lift * motion.travel * entry.height);
                entry.opacity.set(if motion.fade { 0.0 } else { 1.0 });
                entry.size.set(entry.height);
            }

            let front = index == 0;
            let shown = if index < self.visible { 1.0 } else { 0.0 };
            let height = entry.height;
            let away = -lift * motion.travel * height;
            let gone = if motion.fade { 0.0 } else { 1.0 };
            let presence = motion.presence;
            let duration = if entering { presence } else { motion.duration };

            match entry.removed {
                // Animated once, when released.
                Some(Removal { swiped: true, .. }) => {}
                Some(_) if front => {
                    entry.y.go(away, presence, ease, now);
                    entry.scale.go(1.0, presence, ease, now);
                    entry.opacity.go(gone, presence, ease, now);
                    entry.size.go(height, presence, ease, now);
                }
                Some(_) if expanded => {
                    entry.y.go(lift * entry.offset + away, presence, ease, now);
                    entry.scale.go(1.0, presence, ease, now);
                    entry.opacity.go(gone, presence, ease, now);
                    entry.size.go(height, presence, ease, now);
                }
                Some(_) if motion.fade => {
                    // `translateY(40%)` with `transition: transform 500ms,
                    // opacity 200ms`.
                    let slow = Duration::from_millis(500);
                    let fast = Duration::from_millis(200);

                    entry.y.go(0.4 * front_height, slow, ease, now);
                    entry.scale.go(1.0, slow, ease, now);
                    entry.opacity.go(0.0, fast, ease, now);
                    entry.size.go(front_height, motion.duration, ease, now);
                }
                Some(_) => {
                    // Slides away from its place in the stack.
                    let y = lift * peek * index as f32 + away;

                    entry.y.go(y, presence, ease, now);
                }
                None if expanded || front => {
                    let y = if expanded { lift * entry.offset } else { 0.0 };

                    entry.y.go(y, duration, ease, now);
                    entry.scale.go(1.0, duration, ease, now);
                    entry.opacity.go(shown, duration, ease, now);
                    entry.content.go(1.0, duration, ease, now);
                    entry.size.go(height, duration, ease, now);
                }
                None => {
                    let before = index as f32;

                    entry.y.go(lift * peek * before, duration, ease, now);
                    entry
                        .scale
                        .go(1.0 - motion.shrink * before, duration, ease, now);
                    entry.opacity.go(shown, duration, ease, now);
                    entry.content.go(0.0, duration, ease, now);
                    entry.size.go(front_height, duration, ease, now);
                }
            }
        }
    }

    /// Dismisses a toast with its exit animation.
    fn remove(
        &self,
        state: &mut State,
        id: u64,
        swiped: bool,
        now: Instant,
        shell: &mut Shell<'_, Message>,
    ) {
        let Some(entry) = state.entries.get_mut(&id) else {
            return;
        };

        if entry.removed.is_some() || entry.unmounted {
            return;
        }

        entry.removed = Some(Removal { at: now, swiped });

        if !animation::enabled() {
            entry.unmounted = true;
            shell.publish((self.on_dismiss)(id));
            shell.invalidate_layout();
        }

        self.refresh(state, now);
        shell.request_redraw();
    }

    /// The edge the toasts are anchored to, in absolute coordinates.
    fn anchor(&self, bounds: Rectangle) -> f32 {
        match self.edge {
            Edge::Top => bounds.y,
            Edge::Bottom => bounds.y + bounds.height,
        }
    }

    fn placement(&self, entry: &Entry, layout: Rectangle, anchor: f32, now: Instant) -> Placement {
        let size = entry.size.value(now);
        let scale = entry.scale.value(now);

        let bounds = Rectangle {
            x: layout.x,
            y: match self.edge {
                Edge::Top => anchor,
                Edge::Bottom => anchor - size,
            },
            width: layout.width,
            height: size,
        };

        let offset = Vector::new(
            entry.swipe.0.value(now),
            entry.y.value(now) + entry.swipe.1.value(now),
        );
        let origin = match self.motion.origin {
            Origin::Center => bounds.center(),
            Origin::Top => Point::new(bounds.center_x(), bounds.y),
            Origin::Bottom => Point::new(bounds.center_x(), bounds.y + bounds.height),
        };
        let transformation = animation::transform(origin, scale, offset);

        // `align-items: center`: the content stays centered in the box.
        let content =
            transformation * Transformation::translate(0.0, bounds.center().y - layout.center().y);

        let visible = Rectangle {
            x: origin.x + (bounds.x - origin.x) * scale + offset.x,
            y: origin.y + (bounds.y - origin.y) * scale + offset.y,
            width: bounds.width * scale,
            height: bounds.height * scale,
        };

        Placement {
            bounds,
            transformation,
            content,
            visible,
        }
    }

    /// Whether the toast with the given index (newest first) can be
    /// interacted with.
    fn is_interactive(&self, state: &State, entry: &Entry, index: usize) -> bool {
        entry.mounted
            && entry.removed.is_none()
            && !entry.unmounted
            && index < self.visible
            && (index == 0 || state.expanded || self.expand)
    }

    /// The area that expands the toasts when hovered: the visible toasts
    /// (leaving ones included) and, when expanded, the gaps between them.
    fn is_hovered(
        &self,
        state: &State,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        now: Instant,
    ) -> bool {
        let Some(position) = cursor.land().position() else {
            return false;
        };

        let anchor = self.anchor(layout.bounds());
        let expanded = state.expanded || self.expand;
        let live = self.live(state);

        self.toasts
            .iter()
            .zip(layout.children())
            .any(|(toast, layout)| {
                let entry = &state.entries[&toast.id];
                let Some(index) = live.iter().position(|id| *id == toast.id) else {
                    return false;
                };

                // A dismissed toast keeps its area while it leaves
                // (`[data-removed='true']::before`), so that the stack does
                // not collapse under the cursor.
                if !entry.mounted || index >= self.visible {
                    return false;
                }

                let mut area = self.placement(entry, layout.bounds(), anchor, now).visible;

                // `[data-expanded='true']::after`
                if expanded {
                    let gap = self.gap + 1.0;
                    area.height += gap;

                    if self.edge == Edge::Bottom {
                        area.y -= gap;
                    }
                }

                area.contains(position)
            })
    }

    /// Expands or collapses the toasts depending on the cursor.
    fn hover(
        &self,
        state: &mut State,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        now: Instant,
        shell: &mut Shell<'_, Message>,
    ) {
        let expanded = if self.is_hovered(state, layout, cursor, now) {
            true
        } else {
            // Swiping a toast out of the stack keeps it expanded.
            state.expanded && (state.interacting || state.drag.is_some())
        };

        if state.expanded != expanded {
            state.tick(now);
            state.expanded = expanded;
            self.refresh(state, now);
            shell.request_redraw();
        }
    }

    /// Moves the swiped toast with the pointer.
    fn swipe(&self, state: &mut State, position: Point) {
        let Some(drag) = state.drag.as_mut() else {
            return;
        };

        let delta = position - drag.start;

        if drag.axis.is_none() && (delta.x.abs() > 1.0 || delta.y.abs() > 1.0) {
            drag.axis = Some(if delta.x.abs() > delta.y.abs() {
                Axis::X
            } else {
                Axis::Y
            });
        }

        // Swiping in a direction that is not allowed is dampened.
        let dampen = |delta: f32, allowed: bool| {
            if allowed {
                delta
            } else {
                let dampened = delta / (1.5 + delta.abs() / 20.0);

                if dampened.abs() < delta.abs() {
                    dampened
                } else {
                    delta
                }
            }
        };

        let has = |direction| self.swipe.contains(&direction);
        let mut amount = Vector::ZERO;

        match drag.axis {
            Some(Axis::Y) if has(Direction::Up) || has(Direction::Down) => {
                let allowed = (has(Direction::Up) && delta.y < 0.0)
                    || (has(Direction::Down) && delta.y > 0.0);
                amount.y = dampen(delta.y, allowed);
            }
            Some(Axis::X) if has(Direction::Left) || has(Direction::Right) => {
                let allowed = (has(Direction::Left) && delta.x < 0.0)
                    || (has(Direction::Right) && delta.x > 0.0);
                amount.x = dampen(delta.x, allowed);
            }
            _ => {}
        }

        let id = drag.id;

        if let Some(entry) = state.entries.get_mut(&id) {
            entry.swipe.0.set(amount.x);
            entry.swipe.1.set(amount.y);
        }
    }

    /// Dismisses the swiped toast if it went far or fast enough, or brings
    /// it back.
    fn release(&self, state: &mut State, now: Instant, shell: &mut Shell<'_, Message>) {
        let Some(drag) = state.drag.take() else {
            return;
        };

        let width = state.width;
        let Some(entry) = state.entries.get_mut(&drag.id) else {
            return;
        };

        let amount = match drag.axis {
            Some(Axis::X) => entry.swipe.0.target(),
            Some(Axis::Y) => entry.swipe.1.target(),
            None => 0.0,
        };

        let elapsed = now.saturating_duration_since(drag.at).as_secs_f32() * 1000.0;
        let velocity = amount.abs() / elapsed.max(1.0);

        let direction = match drag.axis {
            Some(Axis::X) if amount > 0.0 => Some(Direction::Right),
            Some(Axis::X) => Some(Direction::Left),
            Some(Axis::Y) if amount > 0.0 => Some(Direction::Down),
            Some(Axis::Y) => Some(Direction::Up),
            None => None,
        };

        let out = direction.filter(|direction| {
            self.swipe.contains(direction)
                && (amount.abs() >= SWIPE_THRESHOLD || velocity > SWIPE_VELOCITY)
        });

        match out {
            Some(direction) => {
                // `swipe-out-*`: `200ms ease-out forwards`.
                let duration = SWIPE_OUT;
                let easing = animation::css_ease_out;

                let (transition, distance) = match direction {
                    Direction::Left | Direction::Right => (&mut entry.swipe.0, width),
                    Direction::Up | Direction::Down => (&mut entry.swipe.1, entry.height),
                };

                transition.go(amount + amount.signum() * distance, duration, easing, now);
                entry.opacity.go(0.0, duration, easing, now);

                self.remove(state, drag.id, true, now, shell);
            }
            None => {
                let motion = self.motion;

                entry.swipe.0.go(0.0, motion.duration, motion.easing, now);
                entry.swipe.1.go(0.0, motion.duration, motion.easing, now);
                shell.request_redraw();
            }
        }
    }
}

/// The position of a pointer event, and whether it presses or releases.
enum Pointer {
    Pressed(Option<Point>),
    Moved(Point),
    Released,
    Left,
}

fn pointer(event: &Event, cursor: mouse::Cursor) -> Option<Pointer> {
    Some(match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
            Pointer::Pressed(cursor.land().position())
        }
        Event::Mouse(mouse::Event::CursorMoved { position }) => Pointer::Moved(*position),
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => Pointer::Released,
        Event::Mouse(mouse::Event::CursorLeft) => Pointer::Left,
        Event::Touch(touch::Event::FingerPressed { position, .. }) => {
            Pointer::Pressed(Some(*position))
        }
        Event::Touch(touch::Event::FingerMoved { position, .. }) => Pointer::Moved(*position),
        Event::Touch(touch::Event::FingerLifted { .. } | touch::Event::FingerLost { .. }) => {
            Pointer::Released
        }
        _ => return None,
    })
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Toaster<'a, Message, Theme, Renderer>
where
    Theme: Fade,
    Renderer: iced::advanced::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State {
            ids: self.toasts.iter().map(|toast| toast.id).collect(),
            entries: self
                .toasts
                .iter()
                .map(|toast| (toast.id, Entry::new()))
                .collect(),
            ..State::default()
        })
    }

    fn children(&self) -> Vec<Tree> {
        self.toasts
            .iter()
            .map(|toast| Tree::new(&toast.content))
            .collect()
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<State>();
        let ids: Vec<u64> = self.toasts.iter().map(|toast| toast.id).collect();

        // The toasts are keyed by id, so that removing one keeps the state
        // of the others.
        if state.ids != ids {
            let mut previous: HashMap<u64, Tree> =
                state.ids.drain(..).zip(tree.children.drain(..)).collect();

            tree.children = self
                .toasts
                .iter()
                .map(|toast| {
                    previous
                        .remove(&toast.id)
                        .unwrap_or_else(|| Tree::new(&toast.content))
                })
                .collect();

            state.sync(&ids, Instant::now());

            // sonner collapses the toasts when at most one is left.
            if ids.len() <= 1 {
                state.expanded = false;
            }

            state.ids = ids;
        }

        for (toast, tree) in self.toasts.iter().zip(&mut tree.children) {
            tree.diff(&toast.content);
        }
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<State>();
        let limits = layout::Limits::new(Size::ZERO, Size::new(limits.max().width, f32::INFINITY));

        let nodes: Vec<layout::Node> = self
            .toasts
            .iter_mut()
            .zip(&mut tree.children)
            .map(|(toast, tree)| {
                toast
                    .content
                    .as_widget_mut()
                    .layout(tree, renderer, &limits)
            })
            .collect();

        let mut width: f32 = 0.0;
        let mut height = 0.0;

        for (toast, node) in self.toasts.iter().zip(&nodes) {
            let size = node.size();
            let entry = state.entries.entry(toast.id).or_insert_with(Entry::new);

            entry.height = size.height;
            width = width.max(size.width);

            if !entry.unmounted && entry.removed.is_none() {
                height += size.height + self.gap;
            }
        }

        let height = (height - self.gap).max(0.0);
        state.width = width;

        // New toasts start entering, and the others make room for them, as
        // soon as they are measured.
        self.refresh(state, Instant::now());

        // Every toast is anchored to the edge (`position: absolute;
        // bottom: 0`) and moved by its transformation.
        let nodes = nodes
            .into_iter()
            .map(|node| {
                let y = match self.edge {
                    Edge::Top => 0.0,
                    Edge::Bottom => height - node.size().height,
                };

                node.move_to(Point::new(0.0, y))
            })
            .collect();

        layout::Node::with_children(Size::new(width, height), nodes)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        for ((toast, tree), layout) in self
            .toasts
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            toast
                .content
                .as_widget_mut()
                .operate(tree, layout, renderer, operation);
        }
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
        let now = match event {
            Event::Window(window::Event::RedrawRequested(now)) => *now,
            _ => Instant::now(),
        };

        let state = tree.state.downcast_mut::<State>();
        let anchor = self.anchor(layout.bounds());
        let live = self.live(state);

        // The content of the toasts, with the cursor moved into their
        // transformed space.
        let cursors: Vec<mouse::Cursor> = self
            .toasts
            .iter()
            .zip(layout.children())
            .map(|(toast, child_layout)| {
                let interactive = state.drag.is_none()
                    && state.entries.get(&toast.id).is_some_and(|entry| {
                        live.iter()
                            .position(|id| *id == toast.id)
                            .is_some_and(|index| self.is_interactive(state, entry, index))
                    });

                if interactive {
                    let entry = &state.entries[&toast.id];
                    let placement = self.placement(entry, child_layout.bounds(), anchor, now);

                    cursor * placement.content.inverse()
                } else {
                    mouse::Cursor::Unavailable
                }
            })
            .collect();

        let mut actions = Vec::new();
        let mut captured = false;

        for (((toast, child), child_layout), cursor) in self
            .toasts
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .zip(cursors)
        {
            let mut messages = Vec::new();
            let mut inner = Shell::new(&mut messages);

            toast.content.as_widget_mut().update(
                child,
                event,
                child_layout,
                cursor,
                renderer,
                clipboard,
                &mut inner,
                viewport,
            );

            if inner.is_event_captured() {
                captured = true;
                shell.capture_event();
            }
            if inner.is_layout_invalid() {
                shell.invalidate_layout();
            }
            if inner.are_widgets_invalid() {
                shell.invalidate_widgets();
            }
            shell.request_redraw_at(inner.redraw_request());
            shell.input_method_mut().merge(inner.input_method());

            actions.extend(messages.into_iter().map(|action| (toast.id, action)));
        }

        for (id, action) in actions {
            match action {
                Action::Publish(message) => shell.publish(message),
                Action::Dismiss => self.remove(state, id, false, now, shell),
                Action::PublishAndDismiss(message) => {
                    shell.publish(message);
                    self.remove(state, id, false, now, shell);
                }
            }
        }

        match pointer(event, cursor) {
            Some(Pointer::Moved(position)) => {
                if state.drag.is_some() {
                    self.swipe(state, position);
                    shell.request_redraw();
                }

                self.hover(state, layout, cursor, now, shell);
            }
            Some(Pointer::Left) => {
                self.hover(state, layout, mouse::Cursor::Unavailable, now, shell)
            }
            Some(Pointer::Pressed(position)) => {
                if self.is_hovered(state, layout, cursor, now) {
                    state.tick(now);
                    state.interacting = true;

                    if !captured && let Some(position) = position {
                        // The newest toast under the pointer.
                        let target = self
                            .toasts
                            .iter()
                            .zip(layout.children())
                            .rev()
                            .find(|(toast, child_layout)| {
                                let entry = &state.entries[&toast.id];
                                let Some(index) = live.iter().position(|id| *id == toast.id) else {
                                    return false;
                                };

                                toast.dismissible
                                    && self.is_interactive(state, entry, index)
                                    && self
                                        .placement(entry, child_layout.bounds(), anchor, now)
                                        .visible
                                        .contains(position)
                            })
                            .map(|(toast, _)| toast.id);

                        if let Some(id) = target {
                            state.drag = Some(Drag {
                                id,
                                start: position,
                                at: now,
                                axis: None,
                            });
                        }
                    }

                    shell.capture_event();
                }
            }
            Some(Pointer::Released) if state.interacting || state.drag.is_some() => {
                state.tick(now);
                state.interacting = false;
                self.release(state, now, shell);
                self.hover(state, layout, cursor, now, shell);
            }
            _ => {}
        }

        let Event::Window(window::Event::RedrawRequested(now)) = event else {
            return;
        };
        let now = *now;

        state.tick(now);

        // Dismisses the toasts whose time is up, and unmounts the ones
        // dismissed long enough ago.
        let mut next: Option<Instant> = None;
        let mut schedule = |at: Instant| {
            next = Some(next.map_or(at, |next| next.min(at)));
        };

        for toast in &self.toasts {
            let Some(entry) = state.entries.get(&toast.id) else {
                continue;
            };

            match (entry.removed, toast.duration) {
                (Some(removal), _) if !entry.unmounted => {
                    let unmount = if removal.swiped {
                        SWIPE_OUT.max(UNMOUNT)
                    } else {
                        self.motion.unmount
                    };

                    if now >= removal.at + unmount {
                        let entry = state.entries.get_mut(&toast.id).unwrap();
                        entry.unmounted = true;
                        shell.publish((self.on_dismiss)(toast.id));
                        shell.invalidate_layout();
                    } else {
                        schedule(removal.at + unmount);
                    }
                }
                (None, Some(duration)) if entry.mounted => {
                    if entry.elapsed >= duration {
                        self.remove(state, toast.id, false, now, shell);
                    } else if !state.paused() {
                        schedule(now + (duration - entry.elapsed));
                    }
                }
                _ => {}
            }
        }

        self.refresh(state, now);

        if state.entries.values().any(|entry| entry.is_animating(now)) {
            shell.request_redraw();
        } else if let Some(next) = next {
            shell.request_redraw_at(next);
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
        let now = Instant::now();
        let anchor = self.anchor(layout.bounds());
        let surface = (self.surface)(theme);
        let live = self.live(state);

        // Oldest first, so that the newest toast is on top (`--z-index`).
        for ((toast, child), child_layout) in self
            .toasts
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
        {
            let Some(entry) = state.entries.get(&toast.id) else {
                continue;
            };

            if !entry.mounted || entry.unmounted {
                continue;
            }

            let opacity = entry.opacity.value(now);

            if opacity <= 0.001 {
                continue;
            }

            let placement = self.placement(entry, child_layout.bounds(), anchor, now);

            renderer.with_transformation(placement.transformation, |renderer| {
                // Renderers with damage tracking (tiny-skia) only repaint
                // the bounds of quads that changed, not their shadows; an
                // invisible quad, different on every frame, covers them.
                if entry.is_animating(now) {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: placement.bounds.expand(SHADOW_EXTENT),
                            ..renderer::Quad::default()
                        },
                        Color {
                            r: opacity,
                            g: entry.scale.value(now),
                            b: entry.size.value(now).fract(),
                            a: 0.0,
                        },
                    );
                }

                for quad in box_shadow::quads(
                    self.shadows,
                    placement.bounds,
                    surface.border.radius.top_left,
                    opacity,
                ) {
                    renderer.fill_quad(quad, Color::TRANSPARENT);
                }

                renderer.fill_quad(
                    renderer::Quad {
                        bounds: placement.bounds,
                        border: iced::Border {
                            color: surface.border.color.scale_alpha(opacity),
                            ..surface.border
                        },
                        shadow: iced::Shadow {
                            color: surface.shadow.color.scale_alpha(opacity),
                            ..surface.shadow
                        },
                        snap: false,
                    },
                    surface
                        .background
                        .unwrap_or(Background::Color(Color::TRANSPARENT))
                        .scale_alpha(opacity),
                );
            });

            let opacity = opacity * entry.content.value(now);

            if opacity <= 0.001 {
                continue;
            }

            let faded = theme.faded(opacity);
            let theme = faded.as_ref().unwrap_or(theme);
            let style = renderer::Style {
                text_color: crate::theme::alpha(
                    surface.text_color.unwrap_or(style.text_color),
                    opacity.min(1.0),
                ),
            };

            let interactive = live
                .iter()
                .position(|id| *id == toast.id)
                .is_some_and(|index| self.is_interactive(state, entry, index));
            let cursor = if interactive && state.drag.is_none() {
                cursor * placement.content.inverse()
            } else {
                mouse::Cursor::Unavailable
            };

            renderer.with_transformation(placement.content, |renderer| {
                toast.content.as_widget().draw(
                    child,
                    renderer,
                    theme,
                    &style,
                    child_layout,
                    cursor,
                    viewport,
                );
            });
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

        if state.drag.is_some() {
            return mouse::Interaction::Idle;
        }

        let now = Instant::now();
        let anchor = self.anchor(layout.bounds());
        let live = self.live(state);

        self.toasts
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .filter_map(|((toast, child), child_layout)| {
                let entry = state.entries.get(&toast.id)?;
                let index = live.iter().position(|id| *id == toast.id)?;

                if !self.is_interactive(state, entry, index) {
                    return None;
                }

                let placement = self.placement(entry, child_layout.bounds(), anchor, now);

                Some(toast.content.as_widget().mouse_interaction(
                    child,
                    child_layout,
                    cursor * placement.content.inverse(),
                    viewport,
                    renderer,
                ))
            })
            .max()
            .unwrap_or_default()
    }
}

impl<'a, Message, Theme, Renderer> From<Toaster<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Fade + 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(toaster: Toaster<'a, Message, Theme, Renderer>) -> Self {
        Element::new(toaster)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_toasts_start_their_timers_when_they_are_pushed() {
        let start = Instant::now();
        let mut state = State::default();

        state.sync(&[1], start);
        state.tick(start + Duration::from_millis(100));
        state.sync(&[], start + Duration::from_millis(100));

        // No frame reaches the toaster while no toast is shown.
        let pushed = start + Duration::from_secs(30);
        state.sync(&[2], pushed);
        state.tick(pushed + Duration::from_millis(16));

        assert_eq!(state.entries[&2].elapsed, Duration::from_millis(16));
    }

    #[test]
    fn existing_toasts_keep_running_when_a_toast_is_pushed() {
        let start = Instant::now();
        let mut state = State::default();

        state.sync(&[1], start);
        state.sync(&[1, 2], start + Duration::from_secs(1));
        state.tick(start + Duration::from_millis(1100));

        assert_eq!(state.entries[&1].elapsed, Duration::from_millis(1100));
        assert_eq!(state.entries[&2].elapsed, Duration::from_millis(100));
    }
}
