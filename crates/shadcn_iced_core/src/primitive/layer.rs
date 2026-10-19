//! Content displayed on top of the whole window.
//!
//! [`Layer`] is the headless building block behind dialogs, alert dialogs,
//! sheets, drawers and toasts. It places some content relative to the window
//! (centered, against an edge or in a corner), optionally dims everything
//! behind it with a backdrop and blocks interactions with the rest of the
//! interface.
//!
//! A modal layer manages the focus like the dialogs of Radix:
//!
//! - when it opens, the focus moves into it: to the first focusable control
//!   of the content (`autoFocus`), or to the one marked with [`auto_focus`]
//!   (the cancel button of an alert dialog);
//! - <kbd>Tab</kbd> and <kbd>Shift</kbd>+<kbd>Tab</kbd> cycle through the
//!   controls of the content only (a focus trap), and the page behind it is
//!   `inert`: focus operations of the application skip it;
//! - when it closes, the focus returns to the control of `base` that was
//!   focused when it opened (typically the button that opened it), and
//!   shows it (`focus-visible`) once the user has interacted with the layer
//!   using the keyboard, like browsers do for focus moved by a script.
//!
//! A layer attached to an edge can also be swiped towards it to dismiss it,
//! like the drawers of Vaul (see [`Layer::swipe_to_dismiss`]).
use super::{focus, pointer, tab};
use crate::animation::{self, Appearance, Fade, Transition};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::operation::{Focusable, Outcome, Scrollable, TextInput};
use iced::advanced::widget::{self, Id, Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::time::{Duration, Instant};

use iced::keyboard::{self, key};
use iced::{
    Background, Color, Element, Event, Length, Padding, Point, Rectangle, Size, Vector, alignment,
    mouse, touch, window,
};

/// Content displayed on top of the whole window, over a `base` element.
pub struct Layer<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    base: Element<'a, Message, Theme, Renderer>,
    content: Element<'a, Message, Theme, Renderer>,
    open: bool,
    align_x: alignment::Horizontal,
    align_y: alignment::Vertical,
    padding: Padding,
    backdrop: Option<Box<dyn Fn(&Theme) -> Color + 'a>>,
    modal: bool,
    on_dismiss: Option<Message>,
    dismiss_on_backdrop: bool,
    animation: Animation,
    swipe: Option<Edge>,
}

/// The edge of the window content slides from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Edge {
    /// The top edge.
    Top,
    /// The right edge.
    Right,
    /// The bottom edge.
    Bottom,
    /// The left edge.
    Left,
}

/// How the content of a [`Layer`] enters and leaves the window. The
/// backdrop always fades.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Animation {
    /// No animation.
    None,
    /// `fade-in-0 duration-100`.
    Fade,
    /// `fade-in-0 zoom-in-95 duration-100`, like dialogs.
    #[default]
    FadeZoom,
    /// Slides the whole content in from an edge, like drawers:
    /// `duration-450 ease-[cubic-bezier(0.32,0.72,0,1)]` in, 400ms out.
    Slide(Edge),
    /// Fades the content in while it moves `2.5rem` away from an edge,
    /// like sheets: `transition duration-200 ease-in-out`.
    SlideFade(Edge),
}

/// The offset moving something `distance` towards `edge`.
fn towards(edge: Edge, distance: f32) -> Vector {
    match edge {
        Edge::Top => Vector::new(0.0, -distance),
        Edge::Right => Vector::new(distance, 0.0),
        Edge::Bottom => Vector::new(0.0, distance),
        Edge::Left => Vector::new(-distance, 0.0),
    }
}

/// How far `delta` moves towards `edge`, and across that axis.
fn along(edge: Edge, delta: Vector) -> (f32, f32) {
    match edge {
        Edge::Top => (-delta.y, delta.x),
        Edge::Right => (delta.x, delta.y),
        Edge::Bottom => (delta.y, delta.x),
        Edge::Left => (-delta.x, delta.y),
    }
}

/// The size of `bounds` along the axis of `edge`.
fn extent(edge: Edge, bounds: Rectangle) -> f32 {
    match edge {
        Edge::Top | Edge::Bottom => bounds.height,
        Edge::Left | Edge::Right => bounds.width,
    }
}

/// The easing of drawers: `cubic-bezier(0.32, 0.72, 0, 1)`.
fn drawer_ease(t: f32) -> f32 {
    animation::cubic_bezier(0.32, 0.72, 0.0, 1.0, t)
}

/// `duration-100`, the duration of Nova's overlays.
const OVERLAY: iced::time::Duration = iced::time::Duration::from_millis(100);

impl Animation {
    fn duration(self, opening: bool) -> iced::time::Duration {
        match self {
            Animation::None => iced::time::Duration::ZERO,
            Animation::Fade | Animation::FadeZoom => OVERLAY,
            Animation::Slide(_) if opening => iced::time::Duration::from_millis(450),
            Animation::Slide(_) => iced::time::Duration::from_millis(400),
            Animation::SlideFade(_) => animation::NORMAL,
        }
    }

    fn easing(self) -> animation::Easing {
        match self {
            Animation::Slide(_) => drawer_ease,
            Animation::SlideFade(_) => animation::ease_in_out,
            // `animate-in` / `animate-out`: `var(--tw-ease, ease)`
            _ => animation::ease,
        }
    }

    fn appearance(self, progress: f32, bounds: Rectangle) -> Appearance {
        let hidden = 1.0 - progress;

        match self {
            Animation::None => Appearance::VISIBLE,
            Animation::Fade => Appearance {
                opacity: progress,
                ..Appearance::VISIBLE
            },
            Animation::FadeZoom => Appearance {
                opacity: progress,
                scale: 0.95 + 0.05 * progress,
                offset: Vector::ZERO,
            },
            Animation::Slide(edge) => Appearance {
                opacity: 1.0,
                scale: 1.0,
                offset: match edge {
                    Edge::Top => Vector::new(0.0, -(bounds.y + bounds.height + 2.0) * hidden),
                    Edge::Right => Vector::new((bounds.width + 2.0) * hidden, 0.0),
                    Edge::Bottom => Vector::new(0.0, (bounds.height + 2.0) * hidden),
                    Edge::Left => Vector::new(-(bounds.x + bounds.width + 2.0) * hidden, 0.0),
                },
            },
            Animation::SlideFade(edge) => Appearance {
                opacity: progress,
                scale: 1.0,
                offset: towards(edge, 40.0 * hidden),
            },
        }
    }
}

#[derive(Debug, Default)]
struct State {
    presence: Transition,
    /// Whether the layer was open, to notice when it closes.
    was_open: bool,
    /// Whether a key was pressed in the layer since the last press outside
    /// of its content, which makes the focus returned to the trigger
    /// visible. Like the keyboard modality of browsers, it outlives the
    /// layer being closed and opened again.
    keyboard: bool,
    /// The control of `base` that was focused when the layer opened.
    trigger: Option<Trigger>,
    /// Whether the focus has to move into the content, and whether it is
    /// visible there.
    auto_focus: Option<bool>,
    /// A finger pressing the backdrop: it dismisses the layer once it lifts
    /// without moving, so that a drag starting on the backdrop does not.
    backdrop: Option<pointer::Press>,
    /// A pointer pressing the content, which may swipe it away.
    drag: Option<Drag>,
    /// How far the content was swiped towards its edge.
    swipe: Transition,
    /// Whether a swipe asked to dismiss the layer, which comes back if it
    /// stays open.
    swiped_out: bool,
    /// Whether the focus was returned to the trigger, to redraw it.
    restored: bool,
}

/// How far a swiped layer has to move, relative to its size, to be
/// dismissed when released (`closeThreshold` of Vaul).
const SWIPE_THRESHOLD: f32 = 0.25;

/// How fast a swiped layer has to move when released to be dismissed
/// whatever the distance, in pixels per millisecond (Vaul's
/// `VELOCITY_THRESHOLD`).
const SWIPE_VELOCITY: f32 = 0.4;

/// How long a swiped layer takes to come back (Vaul's `0.5s` transition).
const SWIPE_BACK: Duration = Duration::from_millis(500);

/// A control of `base` (or of its overlays) that was focused, to return the
/// focus to it.
///
/// Widgets have no identity but their [`Id`], so without one the trigger is
/// the n-th focusable widget, as long as it is still where it was and has
/// the same label: a page that changed while the layer was open (a row
/// removed) gets no focus back, like a trigger removed from the DOM.
#[derive(Debug, Clone, PartialEq)]
struct Trigger {
    kind: Kind,
    /// Its position among the widgets of its kind.
    index: usize,
    id: Option<Id>,
    bounds: Rectangle,
    /// Its first text.
    label: Option<String>,
}

/// The kind of a [`Trigger`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    /// A control with a [`focus::Focus`] state, and whether it showed its
    /// focus.
    Control(bool),
    /// A focusable widget of another kind (a text input).
    Focusable,
}

/// A pointer pressing the content of a layer that can be swiped away.
#[derive(Debug, Clone, Copy)]
struct Drag {
    press: pointer::Press,
    at: Instant,
    /// Whether the content follows the pointer: it moved more than
    /// [`pointer::SLOP`] along the axis of the edge.
    active: bool,
    /// Whether only a move towards the edge swipes: the press is on a
    /// scrollable scrolled to that edge, which scrolls the other way.
    towards_only: bool,
}

impl<'a, Message, Theme, Renderer> Layer<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    /// Creates a new [`Layer`] showing `content` over `base` when `open`.
    ///
    /// By default, the layer is modal, centered and dims the window with a
    /// translucent black backdrop (`bg-black/10`).
    pub fn new(
        base: impl Into<Element<'a, Message, Theme, Renderer>>,
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
        open: bool,
    ) -> Self {
        Self {
            base: base.into(),
            content: content.into(),
            open,
            align_x: alignment::Horizontal::Center,
            align_y: alignment::Vertical::Center,
            padding: Padding::ZERO,
            backdrop: Some(Box::new(|_| Color::from_rgba(0.0, 0.0, 0.0, 0.1))),
            modal: true,
            on_dismiss: None,
            dismiss_on_backdrop: true,
            animation: Animation::FadeZoom,
            swipe: None,
        }
    }

    /// Lets the user swipe the content towards `edge` to dismiss the layer,
    /// with a finger or the mouse, like the drawers of Vaul.
    ///
    /// The content follows the pointer once it moved more than
    /// [`pointer::SLOP`] along the axis of the edge, and resists a swipe
    /// away from the edge (a rubber band). Released farther than a quarter
    /// of its size, or flung faster than 0.4px/ms, the layer is dismissed;
    /// otherwise it comes back.
    ///
    /// The controls of the content keep their gestures:
    ///
    /// - the mouse swipes only from where a press does nothing (the handle,
    ///   the header, the gaps between controls), so that text can still be
    ///   selected and sliders dragged;
    /// - a finger swipes from anywhere, but a scrollable that can scroll
    ///   along the axis wins (like Vaul): from one scrolled away from the
    ///   edge, the finger scrolls; from one scrolled to the edge, only a
    ///   swipe towards the edge moves the layer.
    pub fn swipe_to_dismiss(mut self, edge: Edge) -> Self {
        self.swipe = Some(edge);
        self
    }

    /// Sets how the content enters and leaves the window.
    pub fn animation(mut self, animation: Animation) -> Self {
        self.animation = animation;
        self
    }

    /// Sets the horizontal placement of the content in the window.
    pub fn align_x(mut self, align: impl Into<alignment::Horizontal>) -> Self {
        self.align_x = align.into();
        self
    }

    /// Sets the vertical placement of the content in the window.
    pub fn align_y(mut self, align: impl Into<alignment::Vertical>) -> Self {
        self.align_y = align.into();
        self
    }

    /// Sets the space kept between the content and the edges of the window.
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the color of the backdrop, or removes it.
    pub fn backdrop(mut self, backdrop: Option<Color>) -> Self {
        self.backdrop = backdrop
            .map(|color| Box::new(move |_: &Theme| color) as Box<dyn Fn(&Theme) -> Color + 'a>);
        self
    }

    /// Sets the color of the backdrop with a function of the theme.
    pub fn backdrop_with(mut self, f: impl Fn(&Theme) -> Color + 'a) -> Self {
        self.backdrop = Some(Box::new(f));
        self
    }

    /// Sets whether the layer blocks interactions with the rest of the
    /// interface. Non-modal layers let events outside of their content
    /// through.
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self
    }

    /// Sets the message produced when the user wants to close the layer:
    /// <kbd>Escape</kbd> or a click on the backdrop.
    pub fn on_dismiss(mut self, message: Message) -> Self {
        self.on_dismiss = Some(message);
        self
    }

    /// Sets the message produced on dismissal, if any.
    pub fn on_dismiss_maybe(mut self, message: Option<Message>) -> Self {
        self.on_dismiss = message;
        self
    }

    /// Sets whether a click on the backdrop dismisses the layer.
    pub fn dismiss_on_backdrop(mut self, dismiss: bool) -> Self {
        self.dismiss_on_backdrop = dismiss;
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Layer<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Fade + 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::tree::Tag::of::<State>()
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        iced::advanced::widget::tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.base), Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.base.as_widget(), self.content.as_widget()]);
    }

    fn size(&self) -> Size<Length> {
        self.base.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.base.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.base
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
        // Behind a modal layer, the page is `inert`: focus operations of the
        // application (like moving the focus with Tab) skip it.
        if self.open && self.modal {
            self.base.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                &mut Inert { operation },
            );
        } else {
            self.base
                .as_widget_mut()
                .operate(&mut tree.children[0], layout, renderer, operation);
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
        // The focus returned to the trigger in `overlay`, which comes first.
        if std::mem::take(&mut tree.state.downcast_mut::<State>().restored) {
            shell.request_redraw();
        }

        self.base.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
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
        self.base.as_widget().draw(
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
        self.base.as_widget().mouse_interaction(
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
        let Tree {
            state, children, ..
        } = tree;
        let state = state.downcast_mut::<State>();
        let (base_tree, content_tree) = children.split_at_mut(1);
        let now = iced::time::Instant::now();

        // When a modal layer opens, its trigger loses the focus, which moves
        // into the content on its first event. The trigger may be in an
        // overlay of `base`, like a popover.
        if self.open && self.modal && !state.was_open {
            state.was_open = true;

            let mut blur = Blur::default();
            operate_page(
                &mut self.base,
                &mut base_tree[0],
                layout,
                renderer,
                viewport,
                translation,
                &mut blur,
            );

            state.auto_focus = Some(matches!(
                blur.trigger,
                Some(Trigger {
                    kind: Kind::Control(true),
                    ..
                })
            ));
            state.trigger = blur.trigger;
            state.drag = None;
            state.backdrop = None;
            state.swipe.set(0.0);
            state.swiped_out = false;
        }

        // Once closed, the focus returns to the trigger (`onCloseAutoFocus`),
        // visible if the keyboard was used in the layer, as if the focus had
        // come back to it from the layer.
        if !self.open && std::mem::take(&mut state.was_open) {
            if let Some(trigger) = state.trigger.take() {
                let mut find = FindTrigger::new(&trigger);
                operate_page(
                    &mut self.base,
                    &mut base_tree[0],
                    layout,
                    renderer,
                    viewport,
                    translation,
                    &mut find,
                );

                if let Some((kind, index)) = find.result() {
                    operate_page(
                        &mut self.base,
                        &mut base_tree[0],
                        layout,
                        renderer,
                        viewport,
                        translation,
                        &mut Restore {
                            kind,
                            index,
                            keyboard: state.keyboard,
                            controls: 0,
                            focusables: 0,
                        },
                    );
                }
            }

            state.restored = true;
        }

        state.presence.go(
            if self.open { 1.0 } else { 0.0 },
            self.animation.duration(self.open),
            self.animation.easing(),
            now,
        );

        let base = self.base.as_widget_mut().overlay(
            &mut base_tree[0],
            layout,
            renderer,
            viewport,
            translation,
        );

        // Closed content stays mounted while it animates out.
        let visible = self.open || state.presence.is_animating(now);

        let layer = visible.then(|| {
            overlay::Element::new(Box::new(Overlay {
                state,
                closing: !self.open,
                animation: self.animation,
                content: &mut self.content,
                tree: &mut content_tree[0],
                align_x: self.align_x,
                align_y: self.align_y,
                padding: self.padding,
                backdrop: self.backdrop.as_deref(),
                modal: self.modal,
                on_dismiss: self.on_dismiss.as_ref(),
                dismiss_on_backdrop: self.dismiss_on_backdrop,
                swipe: self.swipe,
            }))
        });

        match (base, layer) {
            (None, None) => None,
            (Some(overlay), None) | (None, Some(overlay)) => Some(overlay),
            // The overlays of the page are as `inert` as the page behind an
            // open modal layer: an open popover would otherwise take the
            // keys first, and move the focus behind the layer.
            (Some(base), Some(layer)) if self.open && self.modal => {
                Some(overlay::Element::new(Box::new(Behind { base, layer })))
            }
            (Some(base), Some(layer)) => {
                Some(overlay::Group::with_children(vec![base, layer]).overlay())
            }
        }
    }
}

impl<'a, Message, Theme, Renderer> From<Layer<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Fade + 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(layer: Layer<'a, Message, Theme, Renderer>) -> Self {
        Element::new(layer)
    }
}

/// Creates a new [`Layer`]. See [`Layer::new`].
pub fn layer<'a, Message, Theme, Renderer>(
    base: impl Into<Element<'a, Message, Theme, Renderer>>,
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    open: bool,
) -> Layer<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    Layer::new(base, content, open)
}

struct Overlay<'a, 'b, Message, Theme, Renderer> {
    state: &'b mut State,
    closing: bool,
    animation: Animation,
    content: &'b mut Element<'a, Message, Theme, Renderer>,
    tree: &'b mut Tree,
    align_x: alignment::Horizontal,
    align_y: alignment::Vertical,
    padding: Padding,
    backdrop: Option<&'b (dyn Fn(&Theme) -> Color + 'a)>,
    modal: bool,
    on_dismiss: Option<&'b Message>,
    dismiss_on_backdrop: bool,
    swipe: Option<Edge>,
}

impl<Message, Theme, Renderer> Overlay<'_, '_, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: iced::advanced::Renderer,
{
    /// Focuses the control of the content marked with [`auto_focus`], or
    /// its first focusable control, and unfocuses the others.
    fn focus_initial(&mut self, content: Layout<'_>, renderer: &Renderer, visible: bool) {
        let mut find = FindAutoFocus::default();
        self.content
            .as_widget_mut()
            .operate(self.tree, content, renderer, &mut find);

        if find.total == 0 {
            return;
        }

        self.content.as_widget_mut().operate(
            self.tree,
            content,
            renderer,
            &mut FocusOnly {
                target: find.marked.unwrap_or(0),
                current: 0,
            },
        );

        // Focus moved by a script is visible only after the keyboard was
        // used, like `:focus-visible`.
        if !visible {
            focus::for_each_control(self.content, self.tree, content, renderer, |_, focus| {
                if focus.is_focused() {
                    focus.focus_with_pointer();
                }
            });
        }
    }

    /// Starts following a press of the content that may swipe it away.
    fn start_swipe(
        &mut self,
        event: &Event,
        content: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        captured: bool,
    ) {
        let Some(edge) = self.swipe else {
            return;
        };

        let Some(press) = pointer::Press::start(event, cursor, |position| {
            content.bounds().contains(position)
        }) else {
            return;
        };

        // The mouse leaves its presses to the controls that handle them.
        if !press.is_touch() && captured {
            return;
        }

        let mut scrollables = Scrollables {
            position: press.origin,
            edge,
            blocked: false,
            at_edge: false,
        };

        self.content
            .as_widget_mut()
            .operate(self.tree, content, renderer, &mut scrollables);

        if scrollables.blocked {
            return;
        }

        self.state.drag = Some(Drag {
            press,
            at: Instant::now(),
            active: false,
            towards_only: scrollables.at_edge,
        });
    }

    /// Moves the content with the pointer of a swipe, and dismisses the
    /// layer or brings the content back when it is released. Returns whether
    /// the swipe took the event.
    #[allow(clippy::too_many_arguments)]
    fn follow_swipe(
        &mut self,
        event: &Event,
        content: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> bool {
        let (Some(edge), Some(drag)) = (self.swipe, self.state.drag) else {
            return false;
        };

        enum Step {
            Moved(Point),
            Lifted,
            Lost,
        }

        let step = match (drag.press.pointer, event) {
            (pointer::Pointer::Mouse, Event::Mouse(mouse::Event::CursorMoved { position })) => {
                Step::Moved(*position)
            }
            (
                pointer::Pointer::Mouse,
                Event::Mouse(
                    mouse::Event::ButtonReleased(mouse::Button::Left) | mouse::Event::CursorLeft,
                ),
            ) => Step::Lifted,
            (
                pointer::Pointer::Finger(finger),
                Event::Touch(touch::Event::FingerMoved { id, position }),
            ) if *id == finger => Step::Moved(*position),
            (
                pointer::Pointer::Finger(finger),
                Event::Touch(touch::Event::FingerLifted { id, .. }),
            ) if *id == finger => Step::Lifted,
            (
                pointer::Pointer::Finger(finger),
                Event::Touch(touch::Event::FingerLost { id, .. }),
            ) if *id == finger => Step::Lost,
            _ => return false,
        };

        let position = match step {
            Step::Moved(position) => position,
            Step::Lifted | Step::Lost => {
                self.state.drag = None;

                if !drag.active {
                    return false;
                }

                let now = Instant::now();
                let swiped = self.state.swipe.target();
                let elapsed = now.saturating_duration_since(drag.at).as_secs_f32() * 1000.0;
                let velocity = swiped / elapsed.max(1.0);

                let far = swiped >= SWIPE_THRESHOLD * extent(edge, content.bounds());
                let dismiss = matches!(step, Step::Lifted)
                    && swiped > 0.0
                    && (far || velocity > SWIPE_VELOCITY);

                match self.on_dismiss.filter(|_| dismiss) {
                    // The content leaves from where it was swiped to.
                    Some(message) => {
                        shell.publish(message.clone());
                        self.state.swiped_out = true;
                    }
                    None => self.state.swipe.go(0.0, SWIPE_BACK, drawer_ease, now),
                }

                shell.request_redraw();
                shell.capture_event();

                return true;
            }
        };

        let (towards_edge, across) = along(edge, position - drag.press.origin);

        if !drag.active {
            let takes_over = towards_edge.abs() > pointer::SLOP
                && towards_edge.abs() >= across.abs()
                && !(drag.towards_only && towards_edge < 0.0);

            if !takes_over {
                // Another gesture (a scroll across the axis, or of a
                // scrollable): the swipe gives up.
                if towards_edge.abs() > pointer::SLOP || across.abs() > pointer::SLOP {
                    self.state.drag = None;
                }

                return false;
            }

            self.state.drag = Some(Drag {
                active: true,
                ..drag
            });

            // The controls under the finger lose it: no tap, no scroll.
            if let pointer::Pointer::Finger(id) = drag.press.pointer {
                self.content.as_widget_mut().update(
                    self.tree,
                    &Event::Touch(touch::Event::FingerLost { id, position }),
                    content,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                );
            }
        }

        // A swipe away from the edge is dampened (a rubber band).
        let swiped = if towards_edge >= 0.0 {
            towards_edge
        } else {
            towards_edge / (1.5 + towards_edge.abs() / 20.0)
        };

        self.state.swipe.set(swiped);
        shell.request_redraw();
        shell.capture_event();

        true
    }
}

impl<Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for Overlay<'_, '_, Message, Theme, Renderer>
where
    Theme: Fade,
    Message: Clone,
    Renderer: iced::advanced::Renderer,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        let limits = layout::Limits::new(Size::ZERO, bounds).shrink(self.padding);

        let node = self
            .content
            .as_widget_mut()
            .layout(self.tree, renderer, &limits);

        let size = node.size();
        let available = limits.max();

        let x = self.padding.left
            + match self.align_x {
                alignment::Horizontal::Left => 0.0,
                alignment::Horizontal::Center => (available.width - size.width) / 2.0,
                alignment::Horizontal::Right => available.width - size.width,
            };

        let y = self.padding.top
            + match self.align_y {
                alignment::Vertical::Top => 0.0,
                alignment::Vertical::Center => (available.height - size.height) / 2.0,
                alignment::Vertical::Bottom => available.height - size.height,
            };

        layout::Node::with_children(
            bounds,
            vec![node.move_to(Point::new(x.max(0.0), y.max(0.0)))],
        )
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let bounds = layout.bounds();
        let now = Instant::now();
        let progress = self.state.presence.value(now);
        let content = layout.children().next().unwrap();
        let swipe = self.swipe.map(|edge| (edge, self.state.swipe.value(now)));

        if let Some(backdrop) = self.backdrop {
            let color = backdrop(theme);

            // The backdrop fades as the content is swiped away, like Vaul.
            let swiped = swipe.map_or(0.0, |(edge, swipe)| {
                (swipe / extent(edge, content.bounds()).max(1.0)).clamp(0.0, 1.0)
            });

            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    ..renderer::Quad::default()
                },
                Background::Color(Color {
                    a: color.a * progress * (1.0 - swiped),
                    ..color
                }),
            );
        }

        let mut appearance = self.animation.appearance(progress, content.bounds());

        if let Some((edge, swipe)) = swipe {
            appearance.offset += towards(edge, swipe);
        }

        renderer.with_layer(bounds, |renderer| {
            animation::draw(
                self.content,
                self.tree,
                renderer,
                theme,
                style,
                content,
                cursor,
                &bounds,
                appearance,
            );
        });
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.content.as_widget_mut().operate(
            self.tree,
            layout.children().next().unwrap(),
            renderer,
            operation,
        );
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let bounds = layout.bounds();
        let content = layout.children().next().unwrap();
        let content_bounds = content.bounds();
        let was_captured = shell.is_event_captured();

        pointer::observe(event);

        if let Event::Window(window::Event::RedrawRequested(now)) = event
            && (self.state.presence.is_animating(*now) || self.state.swipe.is_animating(*now))
        {
            shell.request_redraw();
        }

        // A closing layer is only a picture of what it was.
        if self.closing {
            self.state.drag = None;
            self.state.backdrop = None;
            return;
        }

        // Swiped out but still open: the application kept it, it comes back.
        if let Event::Window(window::Event::RedrawRequested(now)) = event
            && std::mem::take(&mut self.state.swiped_out)
        {
            self.state.swipe.go(0.0, SWIPE_BACK, drawer_ease, *now);
            shell.request_redraw();
        }

        // The focus moves into the content once it opened (`autoFocus`).
        if let Some(visible) = self.state.auto_focus.take() {
            self.focus_initial(content, renderer, visible);
            shell.request_redraw();
        }

        if self.follow_swipe(event, content, cursor, renderer, clipboard, shell, &bounds) {
            return;
        }

        // A finger lifting from the backdrop without moving taps it.
        if let Some(press) = self.state.backdrop.as_mut() {
            match press.update(event, cursor) {
                Some(pointer::Gesture::Tapped(position)) => {
                    self.state.backdrop = None;

                    if self.dismiss_on_backdrop
                        && !content_bounds.contains(position)
                        && let Some(message) = self.on_dismiss
                    {
                        shell.publish(message.clone());
                    }
                }
                Some(pointer::Gesture::Released) => self.state.backdrop = None,
                _ => {}
            }
        }

        match event {
            Event::Keyboard(keyboard::Event::KeyPressed { .. }) => self.state.keyboard = true,
            Event::Mouse(mouse::Event::ButtonPressed(_))
            | Event::Touch(touch::Event::FingerPressed { .. })
                if !cursor.is_over(content_bounds) =>
            {
                self.state.keyboard = false;
            }
            _ => {}
        }

        self.content.as_widget_mut().update(
            self.tree, event, content, cursor, renderer, clipboard, shell, &bounds,
        );

        let captured = !was_captured && shell.is_event_captured();

        if self.swipe.is_some() && self.state.drag.is_none() {
            self.start_swipe(event, content, cursor, renderer, captured);
        }

        if captured {
            return;
        }

        match event {
            // The focus trap: Tab cycles through the content only.
            Event::Keyboard(keyboard::Event::KeyPressed { .. }) if self.modal => {
                if let Some(backwards) = tab::tab_pressed(event) {
                    let (element, tree) = (&mut *self.content, &mut *self.tree);

                    if tab::cycle(
                        |operation| {
                            element
                                .as_widget_mut()
                                .operate(tree, content, renderer, operation);
                        },
                        backwards,
                    ) {
                        shell.request_redraw();
                    }
                } else if let Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(key::Named::Escape),
                    ..
                }) = event
                    && let Some(message) = self.on_dismiss
                {
                    shell.publish(message.clone());
                }

                shell.capture_event();
            }
            Event::Mouse(mouse::Event::ButtonPressed(_))
                if self.modal && !cursor.is_over(content_bounds) =>
            {
                if self.dismiss_on_backdrop
                    && let Some(message) = self.on_dismiss
                {
                    shell.publish(message.clone());
                }

                shell.capture_event();
            }
            // A finger dismisses once it lifts without moving (see above).
            Event::Touch(touch::Event::FingerPressed { .. })
                if self.modal && !cursor.is_over(content_bounds) =>
            {
                if self.state.backdrop.is_none() {
                    self.state.backdrop = pointer::Press::start(event, cursor, |_| true);
                }

                shell.capture_event();
            }
            Event::Window(window::Event::RedrawRequested(_)) => {}
            _ if self.modal => {
                shell.capture_event();
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        if self.closing {
            return mouse::Interaction::None;
        }

        let bounds = layout.bounds();
        let content = layout.children().next().unwrap();

        let interaction = self
            .content
            .as_widget()
            .mouse_interaction(self.tree, content, cursor, &bounds, renderer);

        if self.modal || cursor.is_over(content.bounds()) {
            interaction.max(mouse::Interaction::Idle)
        } else {
            interaction
        }
    }

    fn overlay<'c>(
        &'c mut self,
        layout: Layout<'c>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'c, Message, Theme, Renderer>> {
        let bounds = layout.bounds();

        self.content.as_widget_mut().overlay(
            self.tree,
            layout.children().next().unwrap(),
            renderer,
            &bounds,
            Vector::ZERO,
        )
    }

    fn index(&self) -> f32 {
        if self.modal { 2.0 } else { 1.5 }
    }
}

/// Passes an operation on to the page behind a modal layer, but for its
/// focusable widgets: the page is `inert`.
struct Inert<'a> {
    operation: &'a mut dyn Operation,
}

impl Operation for Inert<'_> {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        self.operation
            .traverse(&mut |operation| operate(&mut Inert { operation }));
    }

    fn container(&mut self, id: Option<&Id>, bounds: Rectangle) {
        self.operation.container(id, bounds);
    }

    fn scrollable(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        content_bounds: Rectangle,
        translation: Vector,
        state: &mut dyn Scrollable,
    ) {
        self.operation
            .scrollable(id, bounds, content_bounds, translation, state);
    }

    fn text_input(&mut self, id: Option<&Id>, bounds: Rectangle, state: &mut dyn TextInput) {
        self.operation.text_input(id, bounds, state);
    }

    fn text(&mut self, id: Option<&Id>, bounds: Rectangle, text: &str) {
        self.operation.text(id, bounds, text);
    }

    fn custom(&mut self, id: Option<&Id>, bounds: Rectangle, state: &mut dyn std::any::Any) {
        self.operation.custom(id, bounds, state);
    }

    fn finish(&self) -> Outcome<()> {
        self.operation.finish()
    }
}

/// The overlays of the page behind an open modal layer, and the layer. Like
/// the page, they are `inert`: they show, but only the layer gets the input,
/// and focus operations skip their focusable widgets.
struct Behind<'a, Message, Theme, Renderer> {
    base: overlay::Element<'a, Message, Theme, Renderer>,
    layer: overlay::Element<'a, Message, Theme, Renderer>,
}

impl<Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for Behind<'_, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        layout::Node::with_children(
            bounds,
            vec![
                self.base.as_overlay_mut().layout(renderer, bounds),
                self.layer.as_overlay_mut().layout(renderer, bounds),
            ],
        )
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let mut children = layout.children();
        let (base, layer) = (children.next().unwrap(), children.next().unwrap());

        self.base
            .as_overlay()
            .draw(renderer, theme, style, base, mouse::Cursor::Unavailable);
        self.layer
            .as_overlay()
            .draw(renderer, theme, style, layer, cursor);
    }

    fn operate(&mut self, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
        let mut children = layout.children();
        let (base, layer) = (children.next().unwrap(), children.next().unwrap());

        operation.traverse(&mut |operation| {
            self.base
                .as_overlay_mut()
                .operate(base, renderer, &mut Inert { operation });
            self.layer
                .as_overlay_mut()
                .operate(layer, renderer, operation);
        });
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let mut children = layout.children();
        let (base, layer) = (children.next().unwrap(), children.next().unwrap());

        self.layer
            .as_overlay_mut()
            .update(event, layer, cursor, renderer, clipboard, shell);

        // The overlays of the page still animate, but get no input.
        if let Event::Window(_) = event {
            self.base.as_overlay_mut().update(
                event,
                base,
                mouse::Cursor::Unavailable,
                renderer,
                clipboard,
                shell,
            );
        }
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let layer = layout.children().nth(1).unwrap();

        self.layer
            .as_overlay()
            .mouse_interaction(layer, cursor, renderer)
    }

    // The overlays of the page get no input, so neither do theirs: only the
    // ones of the layer show.
    fn overlay<'c>(
        &'c mut self,
        layout: Layout<'c>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'c, Message, Theme, Renderer>> {
        let layer = layout.children().nth(1).unwrap();

        self.layer.as_overlay_mut().overlay(layer, renderer)
    }

    fn index(&self) -> f32 {
        self.base
            .as_overlay()
            .index()
            .min(self.layer.as_overlay().index())
    }
}

/// Runs `operation` on the page behind a layer: `base`, then its overlays
/// (an open popover), laid out in the window.
fn operate_page<Message, Theme, Renderer>(
    base: &mut Element<'_, Message, Theme, Renderer>,
    tree: &mut Tree,
    layout: Layout<'_>,
    renderer: &Renderer,
    viewport: &Rectangle,
    translation: Vector,
    operation: &mut dyn Operation,
) where
    Renderer: iced::advanced::Renderer,
{
    base.as_widget_mut()
        .operate(tree, layout, renderer, operation);

    if let Some(mut overlay) =
        base.as_widget_mut()
            .overlay(tree, layout, renderer, viewport, translation)
    {
        let node = overlay.as_overlay_mut().layout(renderer, viewport.size());

        overlay
            .as_overlay_mut()
            .operate(Layout::new(&node), renderer, operation);
    }
}

/// Unfocuses the focused control of the page when a layer opens, and
/// remembers it.
#[derive(Default)]
struct Blur {
    trigger: Option<Trigger>,
    /// Whether the next text within the trigger is its label.
    labelling: bool,
    controls: usize,
    focusables: usize,
}

impl Operation for Blur {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn custom(&mut self, id: Option<&Id>, bounds: Rectangle, state: &mut dyn std::any::Any) {
        if let Some(focus) = state.downcast_mut::<focus::Focus>() {
            self.labelling = false;

            // The last focused control wins: an overlay (a popover) comes
            // after the page, and its trigger may still have the focus.
            if focus.is_focused() {
                self.trigger = Some(Trigger {
                    kind: Kind::Control(focus.is_visible()),
                    index: self.controls,
                    id: id.cloned(),
                    bounds,
                    label: None,
                });
                self.labelling = true;

                focus.blur();
            }

            self.controls += 1;
        }
    }

    // Controls report their `Focus` as a custom state first, so only other
    // widgets are still focused here.
    fn focusable(&mut self, id: Option<&Id>, bounds: Rectangle, state: &mut dyn Focusable) {
        if state.is_focused() {
            self.trigger = Some(Trigger {
                kind: Kind::Focusable,
                index: self.focusables,
                id: id.cloned(),
                bounds,
                label: None,
            });
            self.labelling = false;
            state.unfocus();
        }

        self.focusables += 1;
    }

    fn text(&mut self, _id: Option<&Id>, bounds: Rectangle, text: &str) {
        if self.labelling
            && let Some(trigger) = self.trigger.as_mut()
            && trigger.bounds.contains(bounds.center())
        {
            trigger.label = Some(text.to_owned());
            self.labelling = false;
        }
    }
}

/// Finds the [`Trigger`] of a layer in the page once it closes: the widget
/// with its [`Id`], or else the widget of its kind and index, if it is still
/// where it was and has the same label.
struct FindTrigger<'a> {
    trigger: &'a Trigger,
    found: Option<(Kind, usize)>,
    label: Option<String>,
    labelling: bool,
    controls: usize,
    focusables: usize,
}

impl<'a> FindTrigger<'a> {
    fn new(trigger: &'a Trigger) -> Self {
        Self {
            trigger,
            found: None,
            label: None,
            labelling: false,
            controls: 0,
            focusables: 0,
        }
    }

    fn is_trigger(&self, kind: Kind, index: usize, id: Option<&Id>, bounds: Rectangle) -> bool {
        match &self.trigger.id {
            Some(trigger) => id == Some(trigger),
            None => {
                matches!(
                    (kind, self.trigger.kind),
                    (Kind::Control(_), Kind::Control(_)) | (Kind::Focusable, Kind::Focusable)
                ) && index == self.trigger.index
                    && bounds == self.trigger.bounds
            }
        }
    }

    /// The kind and index of the trigger, if it is still there.
    fn result(&self) -> Option<(Kind, usize)> {
        self.found
            .filter(|_| self.trigger.id.is_some() || self.label == self.trigger.label)
    }
}

impl Operation for FindTrigger<'_> {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn custom(&mut self, id: Option<&Id>, bounds: Rectangle, state: &mut dyn std::any::Any) {
        if state.is::<focus::Focus>() {
            self.labelling = false;

            if self.found.is_none()
                && self.is_trigger(Kind::Control(false), self.controls, id, bounds)
            {
                self.found = Some((Kind::Control(false), self.controls));
                self.labelling = true;
            }

            self.controls += 1;
        }
    }

    fn focusable(&mut self, id: Option<&Id>, bounds: Rectangle, _state: &mut dyn Focusable) {
        if self.found.is_none() && self.is_trigger(Kind::Focusable, self.focusables, id, bounds) {
            self.found = Some((Kind::Focusable, self.focusables));
        }

        self.focusables += 1;
    }

    fn text(&mut self, _id: Option<&Id>, bounds: Rectangle, text: &str) {
        if self.labelling && self.trigger.bounds.contains(bounds.center()) {
            self.label = Some(text.to_owned());
            self.labelling = false;
        }
    }
}

/// Returns the focus to the [`Trigger`] of a layer once it closes, found
/// with [`FindTrigger`].
struct Restore {
    kind: Kind,
    index: usize,
    keyboard: bool,
    controls: usize,
    focusables: usize,
}

impl Operation for Restore {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn custom(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn std::any::Any) {
        if let Some(focus) = state.downcast_mut::<focus::Focus>() {
            if matches!(self.kind, Kind::Control(_)) && self.index == self.controls {
                if self.keyboard {
                    focus.focus_with_keyboard();
                } else {
                    focus.focus_with_pointer();
                }
            }

            self.controls += 1;
        }
    }

    fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
        if self.kind == Kind::Focusable && self.index == self.focusables {
            state.focus();
        }

        self.focusables += 1;
    }
}

/// Finds the focusable widget a layer focuses when it opens: the first one
/// in an [`AutoFocus`], if any.
#[derive(Default)]
struct FindAutoFocus {
    total: usize,
    marked: Option<usize>,
    pending: bool,
}

impl Operation for FindAutoFocus {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn custom(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn std::any::Any) {
        if state.is::<Marker>() && self.marked.is_none() {
            self.pending = true;
        }
    }

    fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, _state: &mut dyn Focusable) {
        if std::mem::take(&mut self.pending) {
            self.marked = Some(self.total);
        }

        self.total += 1;
    }
}

/// Focuses the n-th focusable widget, and unfocuses the others.
struct FocusOnly {
    target: usize,
    current: usize,
}

impl Operation for FocusOnly {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
        if self.current == self.target {
            state.focus();
        } else if state.is_focused() {
            state.unfocus();
        }

        self.current += 1;
    }
}

/// Checks the scrollables under a press that may swipe a layer towards
/// `edge`: one that can still scroll along the axis keeps the press
/// (`blocked`), unless it is scrolled to the edge (`at_edge`), where a move
/// towards the edge has nothing to scroll.
struct Scrollables {
    position: Point,
    edge: Edge,
    blocked: bool,
    at_edge: bool,
}

impl Operation for Scrollables {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn scrollable(
        &mut self,
        _id: Option<&Id>,
        bounds: Rectangle,
        content_bounds: Rectangle,
        translation: Vector,
        _state: &mut dyn Scrollable,
    ) {
        if !bounds.contains(self.position) {
            return;
        }

        let overflow = extent(self.edge, content_bounds) - extent(self.edge, bounds);

        if overflow <= 0.5 {
            return;
        }

        let offset = match self.edge {
            Edge::Top | Edge::Bottom => translation.y,
            Edge::Left | Edge::Right => translation.x,
        };

        // Moving towards the bottom (or right) edge scrolls back to the
        // start; towards the top (or left) edge, to the end.
        let at_edge = match self.edge {
            Edge::Bottom | Edge::Right => offset <= 0.5,
            Edge::Top | Edge::Left => offset >= overflow - 0.5,
        };

        if at_edge {
            self.at_edge = true;
        } else {
            self.blocked = true;
        }
    }
}

/// The custom state an [`AutoFocus`] reports to operations.
struct Marker;

/// Marks the control a modal [`Layer`] focuses when it opens, instead of
/// its first focusable control: the first focusable widget within.
///
/// Radix focuses the cancel button of an alert dialog, so that a key press
/// does not confirm a destructive action by accident.
pub struct AutoFocus<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
}

/// Creates an [`AutoFocus`] around `content`.
pub fn auto_focus<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> AutoFocus<'a, Message, Theme, Renderer> {
    AutoFocus {
        content: content.into(),
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for AutoFocus<'_, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn tag(&self) -> tree::Tag {
        self.content.as_widget().tag()
    }

    fn state(&self) -> tree::State {
        self.content.as_widget().state()
    }

    fn children(&self) -> Vec<Tree> {
        self.content.as_widget().children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.content.as_widget().diff(tree);
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
        self.content.as_widget_mut().layout(tree, renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.custom(None, layout.bounds(), &mut Marker);

        self.content
            .as_widget_mut()
            .operate(tree, layout, renderer, operation);
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
        self.content.as_widget_mut().update(
            tree, event, layout, cursor, renderer, clipboard, shell, viewport,
        );
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
        self.content
            .as_widget()
            .draw(tree, renderer, theme, style, layout, cursor, viewport);
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content
            .as_widget()
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content
            .as_widget_mut()
            .overlay(tree, layout, renderer, viewport, translation)
    }
}

impl<'a, Message, Theme, Renderer> From<AutoFocus<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(auto_focus: AutoFocus<'a, Message, Theme, Renderer>) -> Self {
        Element::new(auto_focus)
    }
}
