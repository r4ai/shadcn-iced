//! A container that behaves like an HTML `<label>` around its control.
//!
//! In browsers, clicking the label of a control (`<label for>` or a label
//! wrapping it) focuses the control and, for checkboxes, radios and
//! switches, toggles it. [`Labelled`] does the same for the controls it
//! contains: a click on it that no descendant handles focuses its first
//! control (a focusable [`Pressable`](super::Pressable), a text input, ...)
//! and presses it.
use super::focus::Focus;
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Id, Operation, Tree, Widget, operation, tree};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::{Element, Event, Length, Rectangle, Size, Vector, mouse, touch};

use std::any::Any;

/// A container behaving like the `<label>` of the controls it contains.
pub struct Labelled<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    width: Length,
    height: Length,
}

impl<'a, Message, Theme, Renderer> Labelled<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    /// Wraps the given content, which contains the label and its control.
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        let content = content.into();
        let size = content.as_widget().size_hint();

        Self {
            content,
            width: size.width.fluid(),
            height: size.height.fluid(),
        }
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
    pressed: bool,
}

/// Focuses the first control it finds and, if `activate`, asks it to
/// activate itself.
struct FocusControl {
    activate: bool,
    found: bool,
}

impl Operation for FocusControl {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<()>)) {
        if !self.found {
            operate(self);
        }
    }

    fn custom(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Any) {
        if self.found {
            return;
        }

        if let Some(focus) = state.downcast_mut::<Focus>() {
            focus.focus_with_pointer();

            if self.activate {
                focus.request_activation();
            }

            self.found = true;
        }
    }

    fn focusable(
        &mut self,
        _id: Option<&Id>,
        _bounds: Rectangle,
        state: &mut dyn operation::Focusable,
    ) {
        // Other focusable widgets, e.g. text inputs.
        if !self.found {
            state.focus();
            self.found = true;
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Labelled<'a, Message, Theme, Renderer>
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
        let limits = limits.width(self.width).height(self.height);
        let content = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, &limits);
        let size = limits.resolve(self.width, self.height, content.size());

        layout::Node::with_children(size, vec![content])
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
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
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

        let over = cursor.is_over(layout.bounds());
        let pressed = tree.state.downcast_ref::<State>().pressed;

        let activate = match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. })
                if over && !shell.is_event_captured() =>
            {
                false
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
                if pressed =>
            {
                tree.state.downcast_mut::<State>().pressed = false;

                if !over || shell.is_event_captured() {
                    return;
                }

                true
            }
            Event::Touch(touch::Event::FingerLost { .. })
            | Event::Mouse(mouse::Event::CursorLeft) => {
                tree.state.downcast_mut::<State>().pressed = false;
                return;
            }
            _ => return,
        };

        // A press focuses the control (and keeps it focused, since the
        // control itself blurs when pressed outside), a click activates it.
        let mut operation = FocusControl {
            activate,
            found: false,
        };

        self.content.as_widget_mut().operate(
            &mut tree.children[0],
            content_layout,
            renderer,
            &mut operation,
        );

        if operation.found {
            tree.state.downcast_mut::<State>().pressed = !activate;
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

impl<'a, Message, Theme, Renderer> From<Labelled<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(widget: Labelled<'a, Message, Theme, Renderer>) -> Self {
        Element::new(widget)
    }
}

/// Creates a new [`Labelled`].
pub fn labelled<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Labelled<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    Labelled::new(content)
}
