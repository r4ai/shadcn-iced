//! Arrow key navigation between the controls of a group (roving focus).
//!
//! Radio groups, toggle groups, tab lists and accordions are a single stop
//! in the <kbd>Tab</kbd> order; the arrow keys move the focus between their
//! items, <kbd>Home</kbd> and <kbd>End</kbd> to the first and last ones.
//! [`Roving`] implements this around controls that keep a
//! [`Focus`](super::focus::Focus) (e.g. focusable
//! [`Pressable`](super::Pressable)s, made [`tab_stop(false)`] except for the
//! current item), skipping disabled ones and wrapping around.
//!
//! [`tab_stop(false)`]: super::Pressable::tab_stop
use super::focus;
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Id, Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::keyboard::{self, key::Named};
use iced::{Element, Event, Length, Rectangle, Size, Vector, mouse};

/// The arrow keys that move the focus of a [`Roving`] group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Keys {
    /// <kbd>←</kbd> and <kbd>→</kbd>.
    #[default]
    Horizontal,
    /// <kbd>↑</kbd> and <kbd>↓</kbd>.
    Vertical,
    /// All of them (radio groups).
    Both,
}

/// A group whose controls are navigated with the arrow keys.
pub struct Roving<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    keys: Keys,
    activate: bool,
    home_end: bool,
    only: Option<Id>,
}

impl<'a, Message, Theme, Renderer> Roving<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    /// Wraps the given group.
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            content: content.into(),
            keys: Keys::Horizontal,
            activate: false,
            home_end: true,
            only: None,
        }
    }

    /// Sets the arrow keys that move the focus.
    pub fn keys(mut self, keys: Keys) -> Self {
        self.keys = keys;
        self
    }

    /// Sets whether <kbd>Home</kbd> and <kbd>End</kbd> move the focus to
    /// the first and last controls. Defaults to `true`; Base UI radio
    /// groups ignore them.
    pub fn home_end(mut self, home_end: bool) -> Self {
        self.home_end = home_end;
        self
    }

    /// Only moves the focus between the controls reported with this [`Id`]
    /// (e.g. the triggers of an accordion, and not the buttons of its
    /// panels), and only while one of them has the focus.
    pub fn only(mut self, id: impl Into<Id>) -> Self {
        self.only = Some(id.into());
        self
    }

    /// Presses the newly focused control, e.g. to select radios as the
    /// focus moves (`activateOnFocus`).
    pub fn activate_on_focus(mut self, activate: bool) -> Self {
        self.activate = activate;
        self
    }

    /// The move requested by a key: `Some(Ok(offset))` or `Some(Err(end))`
    /// with `end` being `false` for the first control and `true` for the
    /// last one.
    fn step(&self, key: &keyboard::Key) -> Option<Result<isize, bool>> {
        let keyboard::Key::Named(named) = key else {
            return None;
        };

        let (horizontal, vertical) = match self.keys {
            Keys::Horizontal => (true, false),
            Keys::Vertical => (false, true),
            Keys::Both => (true, true),
        };

        match named {
            Named::ArrowLeft if horizontal => Some(Ok(-1)),
            Named::ArrowRight if horizontal => Some(Ok(1)),
            Named::ArrowUp if vertical => Some(Ok(-1)),
            Named::ArrowDown if vertical => Some(Ok(1)),
            Named::Home if self.home_end => Some(Err(false)),
            Named::End if self.home_end => Some(Err(true)),
            _ => None,
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Roving<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer + 'a,
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
        // The focus moves before the controls see the key, so that the
        // newly focused one activates itself while handling it.
        if let Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = event
            && !shell.is_event_captured()
            && !modifiers.alt()
            && !modifiers.control()
            && !modifiers.logo()
            && let Some(step) = self.step(key)
        {
            let mut count = 0;
            let mut focused = None;

            let only = self.only.as_ref();

            for_each_control(
                &mut self.content,
                tree,
                layout,
                renderer,
                only,
                |index, focus| {
                    count += 1;

                    if focus.is_focused() {
                        focused = Some(index);
                    }
                },
            );

            if let Some(current) = focused {
                let target = match step {
                    Ok(offset) => (current as isize + offset).rem_euclid(count as isize) as usize,
                    Err(false) => 0,
                    Err(true) => count - 1,
                };

                let activate = self.activate;

                for_each_control(
                    &mut self.content,
                    tree,
                    layout,
                    renderer,
                    only,
                    |index, focus| {
                        if index == target {
                            focus.focus_with_keyboard();

                            if activate && index != current {
                                focus.request_activation();
                            }
                        } else {
                            focus.blur();
                        }
                    },
                );

                shell.capture_event();
                shell.request_redraw();
            }
        }

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

impl<'a, Message, Theme, Renderer> From<Roving<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(widget: Roving<'a, Message, Theme, Renderer>) -> Self {
        Element::new(widget)
    }
}

/// Calls `f` with the index and [`Focus`](focus::Focus) of every control of
/// a widget reported with the id `only` (of all of them without), in order.
fn for_each_control<Message, Theme, Renderer>(
    element: &mut Element<'_, Message, Theme, Renderer>,
    tree: &mut Tree,
    layout: Layout<'_>,
    renderer: &Renderer,
    only: Option<&Id>,
    mut f: impl FnMut(usize, &mut focus::Focus) + Send,
) where
    Renderer: iced::advanced::Renderer,
{
    struct Only<'a, F> {
        id: &'a Id,
        index: usize,
        f: F,
    }

    impl<F: FnMut(usize, &mut focus::Focus) + Send> Operation for Only<'_, F> {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<()>)) {
            operate(self);
        }

        fn custom(&mut self, id: Option<&Id>, _bounds: Rectangle, state: &mut dyn std::any::Any) {
            if id == Some(self.id)
                && let Some(focus) = state.downcast_mut::<focus::Focus>()
            {
                (self.f)(self.index, focus);
                self.index += 1;
            }
        }
    }

    let Some(id) = only else {
        return focus::for_each_control(element, tree, layout, renderer, f);
    };

    element.as_widget_mut().operate(
        tree,
        layout,
        renderer,
        &mut Only {
            id,
            index: 0,
            f: &mut f,
        },
    );
}

/// Creates a new [`Roving`] group.
pub fn roving<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Roving<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    Roving::new(content)
}
