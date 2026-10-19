//! Content displayed on top of the whole window.
//!
//! [`Layer`] is the headless building block behind dialogs, alert dialogs,
//! sheets, drawers and toasts. It places some content relative to the window
//! (centered, against an edge or in a corner), optionally dims everything
//! behind it with a backdrop and blocks interactions with the rest of the
//! interface.
//!
//! Like Base UI, a modal layer returns the focus to its trigger once it
//! closes: the control of `base` that was focused when it opened (typically
//! the button that opened it) keeps the focus, and shows it
//! (`focus-visible`) once the user has interacted with the layer using the
//! keyboard, like browsers do for focus moved by a script.
use super::focus;
use crate::animation::{self, Appearance, Fade, Transition};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Operation, Tree, Widget};
use iced::advanced::{Clipboard, Shell, overlay};

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

        let towards = |edge: Edge, distance: f32| match edge {
            Edge::Top => Vector::new(0.0, -distance),
            Edge::Right => Vector::new(distance, 0.0),
            Edge::Bottom => Vector::new(0.0, distance),
            Edge::Left => Vector::new(-distance, 0.0),
        };

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
        }
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
        self.base
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
        let state = tree.state.downcast_mut::<State>();

        // While a modal layer is open, `base` receives no event: the focused
        // trigger stays focused. Once closed, it shows its focus if the
        // keyboard was used, as if the focus had come back to it from the
        // layer.
        if !self.open && std::mem::take(&mut state.was_open) && state.keyboard {
            focus::for_each_control(
                &mut self.base,
                &mut tree.children[0],
                layout,
                renderer,
                |_, focus| {
                    if focus.is_focused() {
                        focus.focus_with_keyboard();
                    }
                },
            );

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

        if self.open && self.modal {
            state.was_open = true;
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
            }))
        });

        match (base, layer) {
            (None, None) => None,
            (Some(overlay), None) | (None, Some(overlay)) => Some(overlay),
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
        let progress = self.state.presence.value(iced::time::Instant::now());

        if let Some(backdrop) = self.backdrop {
            let color = backdrop(theme);

            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    ..renderer::Quad::default()
                },
                Background::Color(Color {
                    a: color.a * progress,
                    ..color
                }),
            );
        }

        let content = layout.children().next().unwrap();
        let appearance = self.animation.appearance(progress, content.bounds());

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

        if let Event::Window(window::Event::RedrawRequested(now)) = event
            && self.state.presence.is_animating(*now)
        {
            shell.request_redraw();
        }

        // A closing layer is only a picture of what it was.
        if self.closing {
            return;
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

        if !was_captured && shell.is_event_captured() {
            return;
        }

        match event {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key::Named::Escape),
                ..
            }) if self.modal => {
                if let Some(message) = self.on_dismiss {
                    shell.publish(message.clone());
                }

                shell.capture_event();
            }
            Event::Mouse(mouse::Event::ButtonPressed(_))
            | Event::Touch(touch::Event::FingerPressed { .. })
                if self.modal && !cursor.is_over(content_bounds) =>
            {
                if self.dismiss_on_backdrop
                    && let Some(message) = self.on_dismiss
                {
                    shell.publish(message.clone());
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
