//! Keyboard shortcuts scoped to a region of the interface.
//!
//! On the web, a region handles the keys pressed while the focus is within
//! it (`onKeyDownCapture` on a container), e.g. the arrow keys of a
//! carousel. [`Keys`] handles the keys pressed while one of the controls of
//! its content is focused, and also stands a press inside of its content
//! for the focus of its controls (most of its content is not focusable in
//! iced) and a press outside of it, or <kbd>Tab</kbd>, for their blur. It
//! turns those keys into messages, before the content sees them.
//!
//! ```no_run
//! # use shadcn_iced_core::primitive::keys;
//! # use iced::keyboard::{Key, key::Named};
//! # #[derive(Clone)] enum Message { Previous, Next }
//! # fn view<'a>(slides: iced::Element<'a, Message>) -> iced::Element<'a, Message> {
//! keys(slides, |key, _modifiers| match key {
//!     Key::Named(Named::ArrowLeft) => Some(Message::Previous),
//!     Key::Named(Named::ArrowRight) => Some(Message::Next),
//!     _ => None,
//! })
//! .into()
//! # }
//! ```
use super::focus;
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::keyboard;
use iced::{Element, Event, Length, Rectangle, Size, Vector, mouse};

/// A region turning the keys pressed once it was clicked, or while the focus
/// is within it, into messages. See the [module documentation](self).
pub struct Keys<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    on_key: Box<dyn Fn(&keyboard::Key, keyboard::Modifiers) -> Option<Message> + 'a>,
}

/// Creates a [`Keys`] region around `content`; `on_key` maps the keys it
/// handles to a message, and the others reach the content.
pub fn keys<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    on_key: impl Fn(&keyboard::Key, keyboard::Modifiers) -> Option<Message> + 'a,
) -> Keys<'a, Message, Theme, Renderer> {
    Keys {
        content: content.into(),
        on_key: Box::new(on_key),
    }
}

#[derive(Debug, Default)]
struct State {
    /// Whether the last press was inside of the region.
    active: bool,
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Keys<'a, Message, Theme, Renderer>
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

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let child = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);

        layout::Node::with_children(child.size(), vec![child])
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
        let content = layout.children().next().unwrap();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(_)) => {
                tree.state.downcast_mut::<State>().active = cursor.is_over(layout.bounds());
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                let active = tree.state.downcast_ref::<State>().active
                    || focus::is_focus_within(
                        &mut self.content,
                        &mut tree.children[0],
                        content,
                        renderer,
                    );

                // The focus moves on with the keyboard: from now on, it
                // tells whether the region is active.
                if let keyboard::Key::Named(keyboard::key::Named::Tab) = key {
                    tree.state.downcast_mut::<State>().active = false;
                }

                if active && let Some(message) = (self.on_key)(key, *modifiers) {
                    shell.publish(message);
                    shell.capture_event();
                    return;
                }
            }
            _ => {}
        }

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            content,
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

impl<'a, Message, Theme, Renderer> From<Keys<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(keys: Keys<'a, Message, Theme, Renderer>) -> Self {
        Element::new(keys)
    }
}
