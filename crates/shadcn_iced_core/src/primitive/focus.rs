//! A container whose look reacts to the focus of its descendants.
//!
//! iced draws text inputs on their own and has no `:focus-within` selector.
//! [`FocusWithin`] fills the gap: it checks, on every event, whether a
//! focusable descendant (a text input, a text editor...) is focused, and
//! styles itself accordingly. Components use it to draw shadcn/ui's focus
//! ring around inputs, input groups and OTP fields.
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Id, Operation, Tree, Widget, operation, tree};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::{
    Background, Border, Color, Element, Event, Length, Padding, Rectangle, Size, Vector, border,
    mouse,
};

/// The look of a [`FocusWithin`].
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Style {
    /// The background, drawn behind the content.
    pub background: Option<Background>,
    /// The border, drawn behind the content.
    pub border: Border,
    /// A ring drawn *outside* of the bounds, like CSS `ring-[3px]`.
    pub ring: Option<Ring>,
    /// The text color inherited by the content.
    pub text_color: Option<Color>,
}

/// A ring drawn around a widget, outside of its bounds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ring {
    /// The color of the ring.
    pub color: Color,
    /// The width of the ring.
    pub width: f32,
    /// The corner radii of the widget the ring surrounds.
    pub radius: border::Radius,
}

impl Ring {
    /// The border drawn around the bounds expanded by the width of the
    /// ring, following the radius of each corner.
    pub fn border(&self) -> Border {
        let radius = self.radius;
        let width = self.width;

        Border {
            color: self.color,
            width,
            radius: border::Radius {
                top_left: radius.top_left + width,
                top_right: radius.top_right + width,
                bottom_right: radius.bottom_right + width,
                bottom_left: radius.bottom_left + width,
            },
        }
    }
}

/// The interaction status of a [`FocusWithin`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Status {
    /// Whether a descendant is focused.
    pub focused: bool,
    /// Whether the cursor is over the container.
    pub hovered: bool,
}

/// A container styled by the focus of its descendants.
pub struct FocusWithin<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    padding: Padding,
    width: Length,
    height: Length,
    style: Box<dyn Fn(&Theme, Status) -> Style + 'a>,
}

impl<'a, Message, Theme, Renderer> FocusWithin<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    /// Wraps the given content.
    pub fn new(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
        style: impl Fn(&Theme, Status) -> Style + 'a,
    ) -> Self {
        let content = content.into();
        let size = content.as_widget().size_hint();

        Self {
            content,
            padding: Padding::ZERO,
            width: size.width.fluid(),
            height: size.height.fluid(),
            style: Box::new(style),
        }
    }

    /// Sets the padding.
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
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
}

#[derive(Debug, Default)]
struct State {
    focused: bool,
}

struct FocusQuery {
    focused: bool,
}

impl Operation for FocusQuery {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<()>)) {
        operate(self);
    }

    fn focusable(
        &mut self,
        _id: Option<&Id>,
        _bounds: Rectangle,
        state: &mut dyn operation::Focusable,
    ) {
        self.focused |= state.is_focused();
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for FocusWithin<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer + 'a,
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
        Size::new(self.width, self.height)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::padded(limits, self.width, self.height, self.padding, |limits| {
            self.content
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, limits)
        })
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
        let content_layout = layout.children().next().unwrap();

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            content_layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        // Moving the pointer or scrolling never moves the focus: skip the
        // walk of the content for those events, the most frequent ones.
        if let Event::Mouse(
            mouse::Event::CursorMoved { .. }
            | mouse::Event::CursorEntered
            | mouse::Event::CursorLeft
            | mouse::Event::WheelScrolled { .. },
        ) = event
        {
            return;
        }

        let mut query = FocusQuery { focused: false };

        self.content.as_widget_mut().operate(
            &mut tree.children[0],
            content_layout,
            renderer,
            &mut query,
        );

        let state = tree.state.downcast_mut::<State>();

        if state.focused != query.focused {
            state.focused = query.focused;
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
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();

        let appearance = (self.style)(
            theme,
            Status {
                focused: state.focused,
                hovered: cursor.is_over(bounds),
            },
        );

        if let Some(ring) = appearance.ring {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: bounds.expand(ring.width),
                    border: ring.border(),
                    ..renderer::Quad::default()
                },
                Background::Color(Color::TRANSPARENT),
            );
        }

        if appearance.background.is_some() || appearance.border.width > 0.0 {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: appearance.border,
                    ..renderer::Quad::default()
                },
                appearance
                    .background
                    .unwrap_or(Background::Color(Color::TRANSPARENT)),
            );
        }

        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            &renderer::Style {
                text_color: appearance.text_color.unwrap_or(style.text_color),
            },
            layout.children().next().unwrap(),
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

impl<'a, Message, Theme, Renderer> From<FocusWithin<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(widget: FocusWithin<'a, Message, Theme, Renderer>) -> Self {
        Element::new(widget)
    }
}

/// Creates a new [`FocusWithin`].
pub fn focus_within<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    style: impl Fn(&Theme, Status) -> Style + 'a,
) -> FocusWithin<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    FocusWithin::new(content, style)
}

/// The keyboard focus of a control, with the `:focus-visible` heuristic of
/// browsers.
///
/// Controls keep it in their widget state and expose it to operations:
///
/// - as a [`Focusable`](operation::Focusable), so that iced's focus
///   operations ([`focus_next`](operation::focusable::focus_next), ...) move
///   the focus to them; focus moved by an operation is *visible*, like
///   focus moved with <kbd>Tab</kbd> in a browser;
/// - as a custom state (`operation.custom(id, bounds, &mut focus)`), so that
///   [`Labelled`](super::labelled::Labelled) and
///   [`Roving`](super::roving::Roving) can focus and activate them.
///
/// Focus gained with the pointer is not visible until the user presses a key,
/// like `:focus-visible` in browsers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Focus {
    focused: bool,
    visible: bool,
    activate: bool,
}

impl Focus {
    /// Whether the control is focused.
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Whether the control shows its focus (`:focus-visible`).
    pub fn is_visible(&self) -> bool {
        self.focused && self.visible
    }

    /// Focuses the control with the pointer: the focus is not visible.
    pub fn focus_with_pointer(&mut self) {
        self.focused = true;
        self.visible = false;
    }

    /// Focuses the control with the keyboard: the focus is visible.
    pub fn focus_with_keyboard(&mut self) {
        self.focused = true;
        self.visible = true;
    }

    /// Records a key press while focused, which makes the focus visible.
    pub fn key_pressed(&mut self) {
        if self.focused {
            self.visible = true;
        }
    }

    /// Unfocuses the control.
    pub fn blur(&mut self) {
        self.focused = false;
        self.visible = false;
    }

    /// Asks the control to activate itself (e.g. to toggle a checkbox whose
    /// label was clicked) the next time it handles an event.
    pub fn request_activation(&mut self) {
        self.activate = true;
    }

    /// Returns whether an activation was requested, and clears the request.
    pub fn take_activation(&mut self) -> bool {
        std::mem::take(&mut self.activate)
    }
}

impl operation::Focusable for Focus {
    fn is_focused(&self) -> bool {
        self.focused
    }

    fn focus(&mut self) {
        self.focus_with_keyboard();
    }

    fn unfocus(&mut self) {
        self.blur();
    }
}

/// Reports whether a descendant of a widget is focused.
pub fn is_focus_within<Message, Theme, Renderer>(
    element: &mut Element<'_, Message, Theme, Renderer>,
    tree: &mut Tree,
    layout: Layout<'_>,
    renderer: &Renderer,
) -> bool
where
    Renderer: iced::advanced::Renderer,
{
    let mut query = FocusQuery { focused: false };

    element
        .as_widget_mut()
        .operate(tree, layout, renderer, &mut query);

    query.focused
}

/// An operation visiting the [`Focus`] states of controls, in order.
pub(crate) struct Controls<F: FnMut(usize, &mut Focus) + Send> {
    pub(crate) index: usize,
    pub(crate) f: F,
}

impl<F: FnMut(usize, &mut Focus) + Send> Operation for Controls<F> {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<()>)) {
        operate(self);
    }

    fn custom(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn std::any::Any) {
        if let Some(focus) = state.downcast_mut::<Focus>() {
            (self.f)(self.index, focus);
            self.index += 1;
        }
    }
}

/// Calls `f` with the index and [`Focus`] of every control of a widget, in
/// order.
pub fn for_each_control<Message, Theme, Renderer>(
    element: &mut Element<'_, Message, Theme, Renderer>,
    tree: &mut Tree,
    layout: Layout<'_>,
    renderer: &Renderer,
    f: impl FnMut(usize, &mut Focus) + Send,
) where
    Renderer: iced::advanced::Renderer,
{
    element
        .as_widget_mut()
        .operate(tree, layout, renderer, &mut Controls { index: 0, f });
}
