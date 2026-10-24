//! <kbd>Tab</kbd> and <kbd>Shift</kbd>+<kbd>Tab</kbd> focus traversal.
//!
//! Browsers move the focus to the next or the previous control on
//! <kbd>Tab</kbd>, but iced leaves it to applications: without
//! [`TabOrder`], no control can be reached by keyboard unless it is
//! clicked first. Wrap the root of the view with [`tab_order`]:
//!
//! ```no_run
//! # use shadcn_iced_core::primitive::tab_order;
//! # fn view<'a, Message: 'a>(content: iced::Element<'a, Message>) -> iced::Element<'a, Message> {
//! tab_order(content).into()
//! # }
//! ```
//!
//! The focus wraps around at the ends, like within a page with nothing else
//! to focus. Controls that handle <kbd>Tab</kbd> themselves (a text editor)
//! capture it, and the focus stays. [`cycle`] and [`focus_first`] run the
//! same traversal over the content of an overlay, so that a modal dialog
//! keeps the focus within it (a focus trap).
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::operation::{Focusable, Operation};
use iced::advanced::widget::{Id, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::keyboard::{self, key::Named};
use iced::{Element, Event, Length, Rectangle, Size, Vector, mouse};

/// Moves the focus to the next focusable widget that `operate` visits, or
/// to the previous one when `backwards`, wrapping around at the ends; with
/// nothing focused yet, it focuses the first (or the last) one. Returns
/// whether there was anything to focus.
pub fn cycle(mut operate: impl FnMut(&mut dyn Operation), backwards: bool) -> bool {
    let mut count = Count::default();
    operate(&mut count);

    if count.total == 0 {
        return false;
    }

    let target = match (count.focused, backwards) {
        (None, false) => 0,
        (None, true) => count.total - 1,
        (Some(focused), false) => (focused + 1) % count.total,
        (Some(focused), true) => (focused + count.total - 1) % count.total,
    };

    operate(&mut FocusAt { target, current: 0 });

    true
}

/// Focuses the first focusable widget that `operate` visits, unless one of
/// them already has the focus (like the `autoFocus` of a dialog). Returns
/// whether a widget has the focus afterwards.
pub fn focus_first(mut operate: impl FnMut(&mut dyn Operation)) -> bool {
    let mut count = Count::default();
    operate(&mut count);

    if count.focused.is_some() {
        return true;
    }

    if count.total == 0 {
        return false;
    }

    operate(&mut FocusAt {
        target: 0,
        current: 0,
    });

    true
}

/// Whether one of the focusable widgets that `operate` visits is focused.
pub fn has_focus(mut operate: impl FnMut(&mut dyn Operation)) -> bool {
    let mut count = Count::default();
    operate(&mut count);

    count.focused.is_some()
}

/// Whether `event` is an uncaptured press of <kbd>Tab</kbd>, and whether it
/// goes backwards (<kbd>Shift</kbd>+<kbd>Tab</kbd>).
pub fn tab_pressed(event: &Event) -> Option<bool> {
    match event {
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(Named::Tab),
            modifiers,
            ..
        }) if !modifiers.control() && !modifiers.alt() && !modifiers.logo() => {
            Some(modifiers.shift())
        }
        _ => None,
    }
}

#[derive(Default)]
struct Count {
    focused: Option<usize>,
    total: usize,
}

impl Operation for Count {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
        if state.is_focused() {
            self.focused = Some(self.total);
        }

        self.total += 1;
    }
}

struct FocusAt {
    target: usize,
    current: usize,
}

impl Operation for FocusAt {
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

/// The root of a view, moving the focus on <kbd>Tab</kbd>. See the
/// [module documentation](self).
pub struct TabOrder<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
}

/// Creates a [`TabOrder`] around `content`.
pub fn tab_order<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> TabOrder<'a, Message, Theme, Renderer> {
    TabOrder {
        content: content.into(),
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for TabOrder<'a, Message, Theme, Renderer>
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

        if shell.is_event_captured() {
            return;
        }

        if let Some(backwards) = tab_pressed(event) {
            let content = self.content.as_widget_mut();

            if cycle(
                |operation| content.operate(tree, layout, renderer, operation),
                backwards,
            ) {
                shell.capture_event();
                shell.request_redraw();
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

impl<'a, Message, Theme, Renderer> From<TabOrder<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(order: TabOrder<'a, Message, Theme, Renderer>) -> Self {
        Element::new(order)
    }
}
