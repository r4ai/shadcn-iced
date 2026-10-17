//! Floating content anchored to another element.
//!
//! [`Anchored`] is the headless building block behind popovers, dropdown
//! menus, selects, tooltips, hover cards and context menus. It takes care of:
//!
//! - positioning the floating content next to the anchor (on a [`Side`],
//!   with an [`Align`]ment and an offset), flipping it to the opposite side
//!   and shifting it so it stays inside the window,
//! - opening it, either controlled by your application ([`Anchored::open`])
//!   or self-managed: on click ([`Anchored::new`]'s default), on hover
//!   ([`Anchored::hover`]), on right click ([`Anchored::context_menu`]) or
//!   like the popup of a combobox ([`Anchored::combobox`]),
//! - dismissing it on <kbd>Escape</kbd>, on a click outside, and (for menus)
//!   once an item has been activated, with a click or <kbd>Enter</kbd>,
//! - switching between the menus of a [`Group`] (a menubar) when another
//!   trigger is hovered, or with <kbd>←</kbd> and <kbd>→</kbd>.
use super::focus;
use crate::animation::{self, Appearance, Fade, Transition};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, overlay};

use iced::keyboard::{self, key};
use iced::time::{Duration, Instant};
use iced::{
    Background, Border, Color, Element, Event, Length, Point, Rectangle, Size, Vector, border,
    mouse, touch, window,
};

use std::sync::atomic::{self, AtomicU64};

/// Incremented whenever a menu closes because one of its items was
/// selected. Menus opened before compare it on every event and close too,
/// which closes a whole chain of submenus at once.
static SELECT_EPOCH: AtomicU64 = AtomicU64::new(0);

fn select_epoch() -> u64 {
    SELECT_EPOCH.load(atomic::Ordering::Relaxed)
}

/// The side of the anchor the floating content is placed on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Side {
    /// Above the anchor.
    Top,
    /// On the right of the anchor.
    Right,
    /// Below the anchor.
    #[default]
    Bottom,
    /// On the left of the anchor.
    Left,
}

impl Side {
    /// Returns the opposite side.
    pub fn opposite(self) -> Self {
        match self {
            Side::Top => Side::Bottom,
            Side::Right => Side::Left,
            Side::Bottom => Side::Top,
            Side::Left => Side::Right,
        }
    }
}

/// The alignment of the floating content along the side of the anchor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Align {
    /// Aligned with the start (left or top) of the anchor.
    Start,
    /// Centered on the anchor.
    #[default]
    Center,
    /// Aligned with the end (right or bottom) of the anchor.
    End,
}

enum Trigger<Message> {
    Controlled {
        open: bool,
        on_dismiss: Option<Message>,
    },
    Click,
    Hover {
        open_delay: Duration,
        close_delay: Duration,
    },
    ContextMenu,
    Input,
}

/// Floating content anchored to another element.
pub struct Anchored<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    anchor: Element<'a, Message, Theme, Renderer>,
    content: Element<'a, Message, Theme, Renderer>,
    trigger: Trigger<Message>,
    side: Side,
    align: Align,
    offset: f32,
    align_offset: f32,
    collision_padding: f32,
    match_anchor_width: bool,
    close_on_select: bool,
    modal: bool,
    on_open_change: Option<Box<dyn Fn(bool) -> Message + 'a>>,
    open_highlight: Option<Box<dyn Fn(&Theme) -> (Background, Border) + 'a>>,
    focus_ring: Option<Box<dyn Fn(&Theme) -> focus::Style + 'a>>,
    animate: bool,
}

impl<'a, Message, Theme, Renderer> Anchored<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    /// Creates floating `content` anchored to `anchor`, toggled by clicking
    /// the anchor.
    pub fn new(
        anchor: impl Into<Element<'a, Message, Theme, Renderer>>,
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        Self {
            anchor: anchor.into(),
            content: content.into(),
            trigger: Trigger::Click,
            side: Side::Bottom,
            align: Align::Center,
            offset: 4.0,
            align_offset: 0.0,
            collision_padding: 8.0,
            match_anchor_width: false,
            close_on_select: false,
            modal: false,
            on_open_change: None,
            open_highlight: None,
            focus_ring: None,
            animate: true,
        }
    }

    /// Controls whether the content is open from your application.
    ///
    /// Pair it with [`Anchored::on_dismiss`] to be notified when the user
    /// wants to close it.
    pub fn open(mut self, open: bool) -> Self {
        let on_dismiss = match std::mem::replace(&mut self.trigger, Trigger::Click) {
            Trigger::Controlled { on_dismiss, .. } => on_dismiss,
            _ => None,
        };

        self.trigger = Trigger::Controlled { open, on_dismiss };
        self
    }

    /// Sets the message produced when a controlled [`Anchored`] should be
    /// dismissed (<kbd>Escape</kbd>, click outside, item selected...).
    pub fn on_dismiss(mut self, message: Message) -> Self {
        let open = match self.trigger {
            Trigger::Controlled { open, .. } => open,
            _ => false,
        };

        self.trigger = Trigger::Controlled {
            open,
            on_dismiss: Some(message),
        };
        self
    }

    /// Opens the content while the anchor (or the content itself) is
    /// hovered, after the given delays.
    pub fn hover(mut self, open_delay: Duration, close_delay: Duration) -> Self {
        self.trigger = Trigger::Hover {
            open_delay,
            close_delay,
        };
        self
    }

    /// Opens the content at the cursor position when the anchor is right
    /// clicked.
    pub fn context_menu(mut self) -> Self {
        self.trigger = Trigger::ContextMenu;
        self.side = Side::Bottom;
        self.align = Align::Start;
        self.offset = 2.0;
        self.close_on_select = true;
        self.modal = true;
        self
    }

    /// Opens the content like the popup of a combobox: when a [`Toggle`]
    /// inside the anchor is pressed, or when the focused anchor receives
    /// text or <kbd>↓</kbd> / <kbd>↑</kbd> (which also highlight the first
    /// or last item). Pressing the rest of the anchor (the input) neither
    /// opens nor closes it.
    pub fn combobox(mut self) -> Self {
        self.trigger = Trigger::Input;
        self
    }

    /// Sets the side of the anchor the content is placed on.
    pub fn side(mut self, side: Side) -> Self {
        self.side = side;
        self
    }

    /// Sets the alignment of the content along the side.
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    /// Sets the distance between the anchor and the content.
    pub fn offset(mut self, offset: f32) -> Self {
        self.offset = offset;
        self
    }

    /// Sets an offset along the alignment axis.
    pub fn align_offset(mut self, offset: f32) -> Self {
        self.align_offset = offset;
        self
    }

    /// Sets the minimum distance kept between the content and the edges of
    /// the window.
    pub fn collision_padding(mut self, padding: f32) -> Self {
        self.collision_padding = padding;
        self
    }

    /// Makes the content at least as wide as the anchor.
    pub fn match_anchor_width(mut self, match_width: bool) -> Self {
        self.match_anchor_width = match_width;
        self
    }

    /// Closes the content once one of its buttons has been pressed.
    pub fn close_on_select(mut self, close: bool) -> Self {
        self.close_on_select = close;
        self
    }

    /// Blocks interactions with the rest of the interface while open.
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self
    }

    /// Sets whether the content fades and zooms in and out
    /// (`animate-in fade-in-0 zoom-in-95 duration-100`). Defaults to `true`.
    pub fn animate(mut self, animate: bool) -> Self {
        self.animate = animate;
        self
    }

    /// Draws a background behind the anchor while the content is open,
    /// like `aria-expanded:bg-muted` on a trigger.
    pub fn open_highlight(mut self, f: impl Fn(&Theme) -> (Background, Border) + 'a) -> Self {
        self.open_highlight = Some(Box::new(f));
        self
    }

    /// Draws a focus ring around the anchor once the content has been
    /// dismissed with the keyboard (<kbd>Escape</kbd>, or <kbd>Enter</kbd>
    /// on an item), like the `focus-visible` style of a trigger the focus
    /// returns to. It lasts until the next press. Only the `ring` and
    /// `border` of the style are drawn, the border over the anchor.
    pub fn focus_ring(mut self, f: impl Fn(&Theme) -> focus::Style + 'a) -> Self {
        self.focus_ring = Some(Box::new(f));
        self
    }

    /// Sets a closure producing a message whenever a self-managed
    /// [`Anchored`] opens or closes.
    pub fn on_open_change(mut self, f: impl Fn(bool) -> Message + 'a) -> Self {
        self.on_open_change = Some(Box::new(f));
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum Hover {
    #[default]
    Idle,
    Opening(Instant),
    Open,
    Closing(Instant),
}

#[derive(Debug, Default)]
struct State {
    presence: Transition,
    open: bool,
    epoch: u64,
    point: Option<Point>,
    hover: Hover,
    anchor_hovered: bool,
    content_hovered: bool,
    /// Whether hover content was dismissed (<kbd>Escape</kbd>) and stays
    /// closed until the pointer leaves.
    suppressed: bool,
    /// Whether the content was dismissed with the keyboard, which leaves
    /// the anchor `focus-visible`.
    focus_visible: bool,
    /// The key event that opened the content, replayed into it once it is
    /// shown (<kbd>↓</kbd> highlights the first item).
    replay: Option<Event>,
}

impl State {
    fn is_open<Message>(&self, trigger: &Trigger<Message>) -> bool {
        match trigger {
            Trigger::Controlled { open, .. } => *open,
            Trigger::Click | Trigger::ContextMenu | Trigger::Input => self.open,
            Trigger::Hover { .. } => {
                matches!(self.hover, Hover::Open | Hover::Closing(_))
            }
        }
    }

    fn tick<Message>(
        &mut self,
        open_delay: Duration,
        close_delay: Duration,
        shell: &mut Shell<'_, Message>,
    ) {
        let now = Instant::now();
        let mut hovered = self.anchor_hovered || self.content_hovered;

        if self.suppressed {
            self.suppressed = hovered;
            hovered = false;
        }

        let next = match self.hover {
            Hover::Idle if hovered => {
                if open_delay.is_zero() {
                    Hover::Open
                } else {
                    shell.request_redraw_at(now + open_delay);
                    Hover::Opening(now)
                }
            }
            Hover::Opening(_) if !hovered => Hover::Idle,
            Hover::Opening(at) if at.elapsed() >= open_delay => Hover::Open,
            Hover::Opening(at) => {
                shell.request_redraw_at(at + open_delay);
                Hover::Opening(at)
            }
            Hover::Open if !hovered => {
                if close_delay.is_zero() {
                    Hover::Idle
                } else {
                    shell.request_redraw_at(now + close_delay);
                    Hover::Closing(now)
                }
            }
            Hover::Closing(_) if hovered => Hover::Open,
            Hover::Closing(at) if at.elapsed() >= close_delay => Hover::Idle,
            Hover::Closing(at) => {
                shell.request_redraw_at(at + close_delay);
                Hover::Closing(at)
            }
            hover => hover,
        };

        let was_open = matches!(self.hover, Hover::Open | Hover::Closing(_));
        let is_open = matches!(next, Hover::Open | Hover::Closing(_));

        if is_open && !was_open {
            self.epoch = select_epoch();
        }

        self.hover = next;

        if was_open != is_open {
            shell.invalidate_layout();
            shell.request_redraw();
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Anchored<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Fade + 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.anchor), Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.anchor.as_widget(), self.content.as_widget()]);
    }

    fn size(&self) -> Size<Length> {
        self.anchor.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.anchor.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.anchor
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
        self.anchor
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
        self.anchor.as_widget_mut().update(
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
        let bounds = layout.bounds();

        if let Event::Mouse(mouse::Event::ButtonPressed(_))
        | Event::Touch(touch::Event::FingerPressed { .. })
        | Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(key::Named::Tab),
            ..
        }) = event
            && state.focus_visible
        {
            state.focus_visible = false;
            shell.request_redraw();
        }

        match &self.trigger {
            Trigger::Controlled { .. } => {}
            Trigger::Click => {
                if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerPressed { .. }) = event
                    && cursor.is_over(bounds)
                    && !state.open
                {
                    state.open = true;
                    state.epoch = select_epoch();
                    shell.invalidate_layout();
                    shell.request_redraw();

                    if let Some(f) = &self.on_open_change {
                        shell.publish(f(true));
                    }
                }
            }
            Trigger::ContextMenu => {
                if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) = event
                    && let Some(position) = cursor.position_over(bounds)
                {
                    let was_open = state.open;

                    state.open = true;
                    state.epoch = select_epoch();
                    state.point = Some(position);
                    shell.invalidate_layout();
                    shell.request_redraw();
                    shell.capture_event();

                    if !was_open && let Some(f) = &self.on_open_change {
                        shell.publish(f(true));
                    }
                }
            }
            Trigger::Hover {
                open_delay,
                close_delay,
            } => {
                if let Event::Mouse(_) | Event::Window(window::Event::RedrawRequested(_)) = event {
                    state.anchor_hovered = cursor.is_over(bounds);
                    state.tick(*open_delay, *close_delay, shell);
                }
            }
            Trigger::Input => {
                let toggle = match event {
                    Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                    | Event::Touch(touch::Event::FingerPressed { .. }) => {
                        cursor.position_over(bounds).is_some_and(|position| {
                            let mut toggles = FindToggles(Vec::new());

                            self.anchor.as_widget_mut().operate(
                                &mut tree.children[0],
                                layout,
                                renderer,
                                &mut toggles,
                            );

                            toggles.0.iter().any(|toggle| toggle.contains(position))
                        })
                    }
                    Event::Keyboard(keyboard::Event::KeyPressed {
                        key,
                        text,
                        modifiers,
                        ..
                    }) if !state.open => {
                        let arrow = matches!(
                            key,
                            keyboard::Key::Named(key::Named::ArrowDown | key::Named::ArrowUp)
                        );
                        let typed = !modifiers.command()
                            && (text
                                .as_ref()
                                .is_some_and(|text| !text.chars().any(char::is_control))
                                || matches!(
                                    key,
                                    keyboard::Key::Named(
                                        key::Named::Backspace | key::Named::Delete
                                    )
                                ));

                        let opens = (arrow || typed) && {
                            let mut focused = FindFocused(false);

                            self.anchor.as_widget_mut().operate(
                                &mut tree.children[0],
                                layout,
                                renderer,
                                &mut focused,
                            );

                            focused.0
                        };

                        if opens && arrow {
                            state.replay = Some(event.clone());
                            shell.capture_event();
                        }

                        opens
                    }
                    _ => false,
                };

                if toggle {
                    // Like a click on the trigger of a Base UI combobox, which
                    // focuses its input.
                    if !state.open {
                        self.anchor.as_widget_mut().operate(
                            &mut tree.children[0],
                            layout,
                            renderer,
                            &mut FocusFirst(false),
                        );
                    }

                    state.open = !state.open;
                    state.epoch = select_epoch();
                    shell.invalidate_layout();
                    shell.request_redraw();

                    if let Some(f) = &self.on_open_change {
                        shell.publish(f(state.open));
                    }
                }
            }
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
        let focus_ring = self
            .focus_ring
            .as_ref()
            .filter(|_| state.focus_visible && !state.is_open(&self.trigger))
            .map(|f| f(theme));

        // `focus-visible:ring-3`
        if let Some(ring) = focus_ring.and_then(|style| style.ring) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds().expand(ring.width),
                    border: Border {
                        color: ring.color,
                        width: ring.width,
                        radius: border::radius(ring.radius + ring.width),
                    },
                    ..renderer::Quad::default()
                },
                Background::Color(Color::TRANSPARENT),
            );
        }

        if let Some(highlight) = &self.open_highlight
            && state.is_open(&self.trigger)
        {
            let (background, border) = highlight(theme);

            renderer.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds(),
                    border,
                    ..renderer::Quad::default()
                },
                background,
            );
        }

        self.anchor.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );

        // `focus-visible:border-ring`
        if let Some(style) = focus_ring
            && style.border.width > 0.0
        {
            renderer.with_layer(*viewport, |renderer| {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: layout.bounds(),
                        border: style.border,
                        ..renderer::Quad::default()
                    },
                    Background::Color(Color::TRANSPARENT),
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
        self.anchor.as_widget().mouse_interaction(
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
        self.overlay_in(tree, layout, renderer, viewport, translation, None)
    }
}

impl<'a, Message, Theme, Renderer> Anchored<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Fade + 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn overlay_in<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
        group: Option<GroupLink<'b>>,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let Tree {
            state, children, ..
        } = tree;
        let state = state.downcast_mut::<State>();
        let (anchor_tree, content_tree) = children.split_at_mut(1);

        let anchor = self.anchor.as_widget_mut().overlay(
            &mut anchor_tree[0],
            layout,
            renderer,
            viewport,
            translation,
        );

        let now = Instant::now();
        let open = state.is_open(&self.trigger);

        state.presence.go(
            if open { 1.0 } else { 0.0 },
            // `duration-100`
            if self.animate {
                Duration::from_millis(100)
            } else {
                Duration::ZERO
            },
            animation::ease_out,
            now,
        );

        // Closed content stays mounted while it fades out.
        let floating = if open || state.presence.is_animating(now) {
            let anchor_bounds = layout.bounds() + translation;
            let point = match self.trigger {
                Trigger::ContextMenu => state.point.map(|point| point + translation),
                _ => None,
            };

            Some(overlay::Element::new(Box::new(Floating {
                content: &mut self.content,
                tree: &mut content_tree[0],
                state,
                trigger: &self.trigger,
                anchor_bounds,
                point,
                side: self.side,
                align: self.align,
                offset: self.offset,
                align_offset: self.align_offset,
                collision_padding: self.collision_padding,
                match_anchor_width: self.match_anchor_width,
                close_on_select: self.close_on_select,
                modal: self.modal,
                on_open_change: self.on_open_change.as_deref(),
                closing: !open,
                resolved_side: self.side,
                group: group.filter(|_| open),
            })))
        } else {
            None
        };

        match (anchor, floating) {
            (None, None) => None,
            (Some(overlay), None) | (None, Some(overlay)) => Some(overlay),
            (Some(anchor), Some(floating)) => {
                Some(overlay::Group::with_children(vec![anchor, floating]).overlay())
            }
        }
    }
}

impl<'a, Message, Theme, Renderer> From<Anchored<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Fade + 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(anchored: Anchored<'a, Message, Theme, Renderer>) -> Self {
        Element::new(anchored)
    }
}

/// Creates floating `content` anchored to `anchor`. See [`Anchored`].
pub fn anchored<'a, Message, Theme, Renderer>(
    anchor: impl Into<Element<'a, Message, Theme, Renderer>>,
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Anchored<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    Anchored::new(anchor, content)
}

struct Floating<'a, 'b, Message, Theme, Renderer> {
    content: &'b mut Element<'a, Message, Theme, Renderer>,
    tree: &'b mut Tree,
    state: &'b mut State,
    trigger: &'b Trigger<Message>,
    anchor_bounds: Rectangle,
    point: Option<Point>,
    side: Side,
    align: Align,
    offset: f32,
    align_offset: f32,
    collision_padding: f32,
    match_anchor_width: bool,
    close_on_select: bool,
    modal: bool,
    on_open_change: Option<&'b (dyn Fn(bool) -> Message + 'a)>,
    closing: bool,
    resolved_side: Side,
    group: Option<GroupLink<'b>>,
}

/// What the open content of a [`Group`] knows about its siblings.
struct GroupLink<'b> {
    /// The index of the [`Anchored`] in its group.
    index: usize,
    /// The bounds of the anchors of the group.
    anchors: Vec<Rectangle>,
    /// Where to ask the group to open another [`Anchored`].
    request: &'b mut Option<usize>,
}

impl<Message, Theme, Renderer> Floating<'_, '_, Message, Theme, Renderer>
where
    Message: Clone,
{
    fn dismiss(&mut self, shell: &mut Shell<'_, Message>) {
        match self.trigger {
            Trigger::Controlled { on_dismiss, .. } => {
                if let Some(message) = on_dismiss {
                    shell.publish(message.clone());
                }
            }
            Trigger::Click | Trigger::ContextMenu | Trigger::Input => {
                if self.state.open {
                    self.state.open = false;
                    shell.invalidate_layout();
                    shell.request_redraw();

                    if let Some(f) = self.on_open_change {
                        shell.publish(f(false));
                    }
                }
            }
            Trigger::Hover { .. } => {
                self.state.hover = Hover::Idle;
                // Until the pointer leaves, or it would reopen right away.
                self.state.suppressed = true;
                shell.invalidate_layout();
                shell.request_redraw();
            }
        }
    }
}

/// Computes the position of floating content of the given size.
///
/// Exposed for components that want to position things themselves.
pub fn position(
    anchor: Rectangle,
    size: Size,
    viewport: Size,
    side: Side,
    align: Align,
    offset: f32,
    align_offset: f32,
    padding: f32,
) -> (Point, Side) {
    let place = |side: Side| {
        let cross_x = || match align {
            Align::Start => anchor.x + align_offset,
            Align::Center => anchor.center_x() - size.width / 2.0,
            Align::End => anchor.x + anchor.width - size.width - align_offset,
        };
        let cross_y = || match align {
            Align::Start => anchor.y + align_offset,
            Align::Center => anchor.center_y() - size.height / 2.0,
            Align::End => anchor.y + anchor.height - size.height - align_offset,
        };

        match side {
            Side::Bottom => Point::new(cross_x(), anchor.y + anchor.height + offset),
            Side::Top => Point::new(cross_x(), anchor.y - size.height - offset),
            Side::Right => Point::new(anchor.x + anchor.width + offset, cross_y()),
            Side::Left => Point::new(anchor.x - size.width - offset, cross_y()),
        }
    };

    let overflow = |side: Side, point: Point| match side {
        Side::Bottom => point.y + size.height - (viewport.height - padding),
        Side::Top => padding - point.y,
        Side::Right => point.x + size.width - (viewport.width - padding),
        Side::Left => padding - point.x,
    };

    let mut side = side;
    let mut point = place(side);

    let current = overflow(side, point);

    if current > 0.0 {
        let opposite = side.opposite();
        let flipped = place(opposite);

        if overflow(opposite, flipped) < current {
            side = opposite;
            point = flipped;
        }
    }

    let clamp = |value: f32, length: f32, max: f32| {
        let upper = max - padding - length;

        if upper < padding {
            padding.min(max - length).max(0.0)
        } else {
            value.clamp(padding, upper)
        }
    };

    (
        Point::new(
            clamp(point.x, size.width, viewport.width),
            clamp(point.y, size.height, viewport.height),
        ),
        side,
    )
}

impl<Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for Floating<'_, '_, Message, Theme, Renderer>
where
    Theme: Fade,
    Message: Clone,
    Renderer: iced::advanced::Renderer,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        let padding = self.collision_padding;
        let max = Size::new(
            (bounds.width - padding * 2.0).max(0.0),
            (bounds.height - padding * 2.0).max(0.0),
        );

        let min_width = if self.match_anchor_width && self.point.is_none() {
            self.anchor_bounds.width.min(max.width)
        } else {
            0.0
        };

        let node = self.content.as_widget_mut().layout(
            self.tree,
            renderer,
            &layout::Limits::new(Size::new(min_width, 0.0), max),
        );

        let anchor = match self.point {
            Some(point) => Rectangle::new(point, Size::ZERO),
            None => self.anchor_bounds,
        };

        let (position, side) = position(
            anchor,
            node.size(),
            bounds,
            self.side,
            self.align,
            self.offset,
            self.align_offset,
            padding,
        );

        self.resolved_side = side;

        node.move_to(position)
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
        let progress = self.state.presence.value(Instant::now());

        // `fade-in-0 zoom-in-95 slide-in-from-{side}-2`
        let distance = 8.0 * (1.0 - progress);
        let offset = match self.resolved_side {
            Side::Bottom => Vector::new(0.0, -distance),
            Side::Top => Vector::new(0.0, distance),
            Side::Right => Vector::new(-distance, 0.0),
            Side::Left => Vector::new(distance, 0.0),
        };

        animation::draw(
            self.content,
            self.tree,
            renderer,
            theme,
            style,
            layout,
            cursor,
            &bounds,
            Appearance {
                opacity: progress,
                scale: 0.95 + 0.05 * progress,
                offset,
            },
        );
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(self.tree, layout, renderer, operation);
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
        let was_captured = shell.is_event_captured();

        if let Event::Window(window::Event::RedrawRequested(now)) = event
            && self.state.presence.is_animating(*now)
        {
            shell.request_redraw();
        }

        // Content fading out is only a picture of what it was.
        if self.closing {
            return;
        }

        if self.close_on_select
            && !matches!(self.trigger, Trigger::Controlled { .. })
            && self.state.epoch != select_epoch()
        {
            self.dismiss(shell);
            return;
        }

        // The key that opened the content (`ArrowDown` in a combobox) acts
        // on it too.
        if let Some(replay) = self.state.replay.take() {
            let mut messages = Vec::new();
            let mut inner = Shell::new(&mut messages);

            self.content.as_widget_mut().update(
                self.tree, &replay, layout, cursor, renderer, clipboard, &mut inner, &bounds,
            );

            // Not merged: the replayed event must not capture this one.
            let invalid = inner.is_layout_invalid();

            for message in messages {
                shell.publish(message);
            }

            if invalid {
                shell.invalidate_layout();
            }

            shell.request_redraw();
        }

        // Another trigger of the group is hovered: open its menu instead.
        if let Some(group) = &mut self.group
            && let Event::Mouse(mouse::Event::CursorMoved { .. }) = event
            && let Some(position) = cursor.position()
            && !bounds.contains(position)
            && let Some(other) = group
                .anchors
                .iter()
                .position(|anchor| anchor.contains(position))
            && other != group.index
        {
            *group.request = Some(other);
            self.dismiss(shell);
            shell.capture_event();
            return;
        }

        self.content.as_widget_mut().update(
            self.tree, event, layout, cursor, renderer, clipboard, shell, &bounds,
        );

        let captured_by_content = !was_captured && shell.is_event_captured();

        // `ArrowLeft` and `ArrowRight` move to the previous and next menus of
        // the group.
        if let Some(group) = &mut self.group
            && !captured_by_content
            && let Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(named @ (key::Named::ArrowLeft | key::Named::ArrowRight)),
                ..
            }) = event
            && group.anchors.len() > 1
        {
            let count = group.anchors.len();

            *group.request = Some(if *named == key::Named::ArrowRight {
                (group.index + 1) % count
            } else {
                (group.index + count - 1) % count
            });
            self.dismiss(shell);
            shell.capture_event();
            return;
        }

        if let Trigger::Hover {
            open_delay,
            close_delay,
        } = *self.trigger
            && let Event::Mouse(_) | Event::Window(window::Event::RedrawRequested(_)) = event
        {
            self.state.content_hovered = cursor.is_over(bounds);
            self.state.tick(open_delay, close_delay, shell);
        }

        match event {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key::Named::Escape),
                ..
            }) => {
                self.dismiss(shell);
                self.state.focus_visible = true;
                shell.capture_event();
            }
            Event::Mouse(mouse::Event::ButtonPressed(_))
            | Event::Touch(touch::Event::FingerPressed { .. })
                if !captured_by_content && !cursor.is_over(bounds) =>
            {
                let over_anchor = self.point.is_none() && cursor.is_over(self.anchor_bounds);

                if matches!(self.trigger, Trigger::Input) {
                    // The anchor handles its own presses (its toggle).
                    if !over_anchor {
                        self.dismiss(shell);
                    }
                } else if matches!(self.trigger, Trigger::Hover { .. }) {
                    if !over_anchor {
                        self.dismiss(shell);
                    }
                } else {
                    self.dismiss(shell);

                    if self.modal || over_anchor {
                        shell.capture_event();
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
                if captured_by_content && self.close_on_select =>
            {
                let _ = SELECT_EPOCH.fetch_add(1, atomic::Ordering::Relaxed);
                self.state.epoch = select_epoch();
                self.dismiss(shell);
            }
            // An item activated with the keyboard.
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key::Named::Enter),
                ..
            }) if captured_by_content && self.close_on_select => {
                let _ = SELECT_EPOCH.fetch_add(1, atomic::Ordering::Relaxed);
                self.state.epoch = select_epoch();
                self.dismiss(shell);
                self.state.focus_visible = true;
            }
            Event::Mouse(_) | Event::Touch(_) if self.modal => {
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
        let interaction = self
            .content
            .as_widget()
            .mouse_interaction(self.tree, layout, cursor, &bounds, renderer);

        if self.modal || cursor.is_over(bounds) {
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

        self.content
            .as_widget_mut()
            .overlay(self.tree, layout, renderer, &bounds, Vector::ZERO)
    }
}

/// A row of [`Anchored`] menus that behave as one, like a menubar: while
/// one of them is open, hovering another trigger opens its menu instead,
/// and <kbd>←</kbd> / <kbd>→</kbd> move to the previous or next menu.
///
/// Menus opened this way do not call their [`Anchored::on_open_change`].
pub struct Group<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    children: Vec<Anchored<'a, Message, Theme, Renderer>>,
    spacing: f32,
}

/// Creates a [`Group`] of [`Anchored`] menus.
pub fn group<'a, Message, Theme, Renderer>(
    children: impl IntoIterator<Item = Anchored<'a, Message, Theme, Renderer>>,
) -> Group<'a, Message, Theme, Renderer> {
    Group {
        children: children.into_iter().collect(),
        spacing: 0.0,
    }
}

impl<Message, Theme, Renderer> Group<'_, Message, Theme, Renderer> {
    /// Sets the horizontal space between the anchors.
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }
}

#[derive(Debug, Default)]
struct GroupState {
    /// The menu to open, asked for by the open one.
    request: Option<usize>,
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Group<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Fade + 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<GroupState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(GroupState::default())
    }

    fn children(&self) -> Vec<Tree> {
        self.children
            .iter()
            .map(|child| Tree::new(child as &dyn Widget<Message, Theme, Renderer>))
            .collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(
            &self
                .children
                .iter()
                .map(|child| child as &dyn Widget<Message, Theme, Renderer>)
                .collect::<Vec<_>>(),
        );
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
        let loose = layout::Limits::new(Size::ZERO, limits.max());
        let mut nodes: Vec<layout::Node> = self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .map(|(child, tree)| child.layout(tree, renderer, &loose))
            .collect();

        let height = nodes
            .iter()
            .map(|node| node.size().height)
            .fold(0.0, f32::max);
        let mut x = 0.0;

        for node in &mut nodes {
            let size = node.size();
            node.move_to_mut(Point::new(x, (height - size.height) / 2.0));
            x += size.width + self.spacing;
        }

        let width = if nodes.is_empty() {
            0.0
        } else {
            x - self.spacing
        };

        layout::Node::with_children(
            limits.resolve(Length::Shrink, Length::Shrink, Size::new(width, height)),
            nodes,
        )
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
            for ((child, tree), layout) in self
                .children
                .iter_mut()
                .zip(&mut tree.children)
                .zip(layout.children())
            {
                child.operate(tree, layout, renderer, operation);
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
        viewport: &Rectangle,
    ) {
        for ((child, tree), layout) in self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            child.update(
                tree, event, layout, cursor, renderer, clipboard, shell, viewport,
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
        for ((child, tree), layout) in self
            .children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
        {
            child.draw(tree, renderer, theme, style, layout, cursor, viewport);
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
        self.children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, tree), layout)| {
                child.mouse_interaction(tree, layout, cursor, viewport, renderer)
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
        let Tree {
            state, children, ..
        } = tree;
        let group = state.downcast_mut::<GroupState>();

        // Opens the menu the previously open one asked for.
        if let Some(index) = group.request.take()
            && let Some(tree) = children.get_mut(index)
        {
            let state = tree.state.downcast_mut::<State>();
            state.open = true;
            state.epoch = select_epoch();
        }

        let anchors: Vec<Rectangle> = layout
            .children()
            .map(|layout| layout.bounds() + translation)
            .collect();
        let mut request = Some(&mut group.request);

        let overlays: Vec<_> = self
            .children
            .iter_mut()
            .zip(children.iter_mut())
            .zip(layout.children())
            .enumerate()
            .filter_map(|(index, ((child, tree), layout))| {
                let link = if tree.state.downcast_ref::<State>().is_open(&child.trigger) {
                    request.take().map(|request| GroupLink {
                        index,
                        anchors: anchors.clone(),
                        request,
                    })
                } else {
                    None
                };

                child.overlay_in(tree, layout, renderer, viewport, translation, link)
            })
            .collect();

        (!overlays.is_empty()).then(|| overlay::Group::with_children(overlays).overlay())
    }
}

impl<'a, Message, Theme, Renderer> From<Group<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Fade + 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(group: Group<'a, Message, Theme, Renderer>) -> Self {
        Element::new(group)
    }
}

/// The part of the anchor of a [`combobox`](Anchored::combobox) that opens
/// and closes the content when pressed, e.g. a chevron button.
pub struct Toggle<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
}

/// Creates a [`Toggle`].
pub fn toggle<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Toggle<'a, Message, Theme, Renderer> {
    Toggle {
        content: content.into(),
    }
}

/// The state of a [`Toggle`], which [`Anchored`] looks for.
#[derive(Debug, Default)]
struct ToggleMarker;

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Toggle<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<ToggleMarker>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(ToggleMarker)
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
        operation.custom(
            None,
            layout.bounds(),
            tree.state.downcast_mut::<ToggleMarker>(),
        );
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

impl<'a, Message, Theme, Renderer> From<Toggle<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(toggle: Toggle<'a, Message, Theme, Renderer>) -> Self {
        Element::new(toggle)
    }
}

/// Collects the bounds of the [`Toggle`]s of a widget tree.
struct FindToggles(Vec<Rectangle>);

impl Operation for FindToggles {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn custom(
        &mut self,
        _id: Option<&widget::Id>,
        bounds: Rectangle,
        state: &mut dyn std::any::Any,
    ) {
        if state.is::<ToggleMarker>() {
            self.0.push(bounds);
        }
    }
}

/// Focuses the first focusable widget of a widget tree.
struct FocusFirst(bool);

impl Operation for FocusFirst {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn focusable(
        &mut self,
        _id: Option<&widget::Id>,
        _bounds: Rectangle,
        state: &mut dyn widget::operation::Focusable,
    ) {
        if !self.0 {
            state.focus();
            self.0 = true;
        }
    }
}

/// Finds out whether a widget tree holds the keyboard focus.
struct FindFocused(bool);

impl Operation for FindFocused {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn focusable(
        &mut self,
        _id: Option<&widget::Id>,
        _bounds: Rectangle,
        state: &mut dyn widget::operation::Focusable,
    ) {
        self.0 |= state.is_focused();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VIEWPORT: Size = Size::new(800.0, 600.0);

    #[test]
    fn places_below_and_centered() {
        let anchor = Rectangle::new(Point::new(100.0, 100.0), Size::new(80.0, 36.0));
        let (point, side) = position(
            anchor,
            Size::new(200.0, 100.0),
            VIEWPORT,
            Side::Bottom,
            Align::Center,
            4.0,
            0.0,
            8.0,
        );

        assert_eq!(side, Side::Bottom);
        assert_eq!(point, Point::new(40.0, 140.0));
    }

    #[test]
    fn flips_when_overflowing() {
        let anchor = Rectangle::new(Point::new(100.0, 550.0), Size::new(80.0, 36.0));
        let (point, side) = position(
            anchor,
            Size::new(200.0, 100.0),
            VIEWPORT,
            Side::Bottom,
            Align::Start,
            4.0,
            0.0,
            8.0,
        );

        assert_eq!(side, Side::Top);
        assert_eq!(point, Point::new(100.0, 446.0));
    }

    #[test]
    fn shifts_inside_viewport() {
        let anchor = Rectangle::new(Point::new(780.0, 100.0), Size::new(10.0, 10.0));
        let (point, _) = position(
            anchor,
            Size::new(200.0, 100.0),
            VIEWPORT,
            Side::Bottom,
            Align::Start,
            4.0,
            0.0,
            8.0,
        );

        assert_eq!(point.x, 592.0);
    }
}
