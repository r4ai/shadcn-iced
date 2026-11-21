//! Text composed with an input method editor (IME), like the Japanese,
//! Chinese and Korean ones.
//!
//! While a text is being composed (the reading typed, then converted), the
//! keys belong to the IME: <kbd>Enter</kbd> commits the conversion,
//! <kbd>Escape</kbd> cancels it, the arrows and <kbd>Space</kbd> pick a
//! candidate and <kbd>Backspace</kbd> edits the reading. Most platforms keep
//! those keys from the application, but some input methods still report
//! them.
//!
//! [`Composition`] wraps a text input: it tells when its input composes and
//! keeps the keys from it meanwhile. The widgets handling keys of their own
//! (a listbox, a dialog closing on <kbd>Escape</kbd>, the focus trap of
//! <kbd>Tab</kbd>...) ignore them while [`is_composing`].
//!
//! ```no_run
//! # use shadcn_iced_core::primitive::composition::composition;
//! # #[derive(Clone)] enum Message { Name(String) }
//! # fn view(name: &str) -> iced::Element<'_, Message> {
//! composition(iced::widget::text_input("Name", name).on_input(Message::Name)).into()
//! # }
//! ```
use super::focus;
use iced::advanced::input_method;
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::{Element, Event, Length, Rectangle, Size, Vector, keyboard, mouse, touch, window};

use std::cell::Cell;

thread_local! {
    /// Whether a focused input is composing a text.
    static COMPOSING: Cell<bool> = const { Cell::new(false) };
}

/// Whether a focused text input is composing a text with an IME: the keys
/// pressed belong to the IME until it commits or cancels the text.
pub fn is_composing() -> bool {
    COMPOSING.get()
}

/// Whether `event` is a key for the IME, composing a text: the widgets
/// handling keys ignore it.
pub fn is_for_ime(event: &Event) -> bool {
    is_composing()
        && matches!(
            event,
            Event::Keyboard(
                keyboard::Event::KeyPressed { .. } | keyboard::Event::KeyReleased { .. }
            )
        )
}

/// Whether the text input is composing after `event`, if it tells.
fn composing_after(event: &Event) -> Option<bool> {
    match event {
        Event::InputMethod(input_method::Event::Preedit(content, _)) => Some(!content.is_empty()),
        Event::InputMethod(input_method::Event::Commit(_) | input_method::Event::Closed)
        | Event::Window(window::Event::Unfocused) => Some(false),
        // A press commits the composition, or leaves it (and the focus).
        Event::Mouse(mouse::Event::ButtonPressed(_))
        | Event::Touch(touch::Event::FingerPressed { .. }) => Some(false),
        _ => None,
    }
}

/// A text input, kept from the keys of the IME while it composes. See the
/// [module documentation](self).
pub struct Composition<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
}

/// Wraps a text input in a [`Composition`].
pub fn composition<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Composition<'a, Message, Theme, Renderer> {
    Composition {
        content: content.into(),
    }
}

#[derive(Debug, Default)]
struct State {
    /// Whether the input is composing.
    composing: bool,
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Composition<'a, Message, Theme, Renderer>
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
        let composing = tree.state.downcast_ref::<State>().composing;

        // The keys of the IME are not for the input.
        if composing
            && matches!(
                event,
                Event::Keyboard(
                    keyboard::Event::KeyPressed { .. } | keyboard::Event::KeyReleased { .. }
                )
            )
        {
            shell.capture_event();
            return;
        }

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

        let after = match composing_after(event) {
            // Only the focused input composes.
            Some(true) => Some(focus::is_focus_within(
                &mut self.content,
                &mut tree.children[0],
                layout,
                renderer,
            )),
            Some(false) => Some(false),
            // The input composing lost the focus (to an operation).
            None if composing
                && matches!(event, Event::Window(window::Event::RedrawRequested(_))) =>
            {
                (!focus::is_focus_within(
                    &mut self.content,
                    &mut tree.children[0],
                    layout,
                    renderer,
                ))
                .then_some(false)
            }
            None => None,
        };

        if let Some(after) = after {
            let state = tree.state.downcast_mut::<State>();

            // Another input may be composing: only the change is told.
            if state.composing != after {
                state.composing = after;
                COMPOSING.set(after);
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

impl<'a, Message, Theme, Renderer> From<Composition<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(composition: Composition<'a, Message, Theme, Renderer>) -> Self {
        Element::new(composition)
    }
}

#[cfg(test)]
mod tests {
    use super::composing_after;
    use iced::advanced::input_method;
    use iced::{Event, mouse};

    #[test]
    fn the_preedit_tells_the_composition() {
        let preedit = |content: &str| {
            Event::InputMethod(input_method::Event::Preedit(content.to_owned(), None))
        };

        assert_eq!(composing_after(&preedit("にほ")), Some(true));
        assert_eq!(composing_after(&preedit("")), Some(false));
        assert_eq!(
            composing_after(&Event::InputMethod(input_method::Event::Commit(
                "日本".to_owned()
            ))),
            Some(false)
        );
        assert_eq!(
            composing_after(&Event::InputMethod(input_method::Event::Opened)),
            None
        );
        assert_eq!(
            composing_after(&Event::Mouse(mouse::Event::ButtonPressed(
                mouse::Button::Left
            ))),
            Some(false)
        );
    }
}
