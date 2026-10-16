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
//!   ([`Anchored::hover`]) or on right click ([`Anchored::context_menu`]),
//! - dismissing it on <kbd>Escape</kbd>, on a click outside, and (for menus)
//!   once an item has been activated.
use crate::animation::{self, Appearance, Fade, Transition};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, overlay};

use iced::keyboard::{self, key};
use iced::time::{Duration, Instant};
use iced::{Element, Event, Length, Point, Rectangle, Size, Vector, mouse, touch, window};

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
}

impl State {
    fn is_open<Message>(&self, trigger: &Trigger<Message>) -> bool {
        match trigger {
            Trigger::Controlled { open, .. } => *open,
            Trigger::Click | Trigger::ContextMenu => self.open,
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
        let hovered = self.anchor_hovered || self.content_hovered;

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
        self.anchor.as_widget().draw(
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
            animation::FAST,
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
            Trigger::Click | Trigger::ContextMenu => {
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

        self.content.as_widget_mut().update(
            self.tree, event, layout, cursor, renderer, clipboard, shell, &bounds,
        );

        let captured_by_content = !was_captured && shell.is_event_captured();

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
                shell.capture_event();
            }
            Event::Mouse(mouse::Event::ButtonPressed(_))
            | Event::Touch(touch::Event::FingerPressed { .. })
                if !captured_by_content && !cursor.is_over(bounds) =>
            {
                let over_anchor = self.point.is_none() && cursor.is_over(self.anchor_bounds);

                if matches!(self.trigger, Trigger::Hover { .. }) {
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
