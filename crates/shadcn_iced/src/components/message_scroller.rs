//! A scrolling list of messages that sticks to the newest one.
//!
//! Port of <https://ui.shadcn.com/docs/components/message-scroller>. The
//! scroller stays anchored to the bottom: new messages appear without
//! having to scroll. A "scroll to end" button can be shown while the reader
//! is away from the newest message:
//!
//! ```no_run
//! # use shadcn_iced::message_scroller::{self, message_scroller};
//! # #[derive(Clone)] enum Message { Scrolled(bool), ScrollToEnd }
//! # fn view<'a>(messages: Vec<iced::Element<'a, Message>>, at_end: bool) -> iced::Element<'a, Message> {
//! message_scroller(messages)
//!     .id("chat")
//!     .on_scroll(|viewport| Message::Scrolled(message_scroller::is_at_end(&viewport)))
//!     .button((!at_end).then_some(Message::ScrollToEnd))
//!     .into()
//! # }
//! ```
//!
//! Answer `Message::ScrollToEnd` with [`scroll_to_end`].
//!
//! Like a scrollable `div` of a browser, a click inside the scroller makes
//! it the target of the scrolling keys (<kbd>Home</kbd>, <kbd>End</kbd>,
//! <kbd>PageUp</kbd>, <kbd>PageDown</kbd>, <kbd>Space</kbd>, <kbd>↑</kbd>
//! and <kbd>↓</kbd>).
use super::button::{self, icon_button};

use shadcn_iced_core::icon::lucide;
use shadcn_iced_core::theme::alpha;
use shadcn_iced_core::{Element, tokens};

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{Operation, Tree, Widget, operation, tree};
use iced::advanced::{Clipboard, Shell, overlay, renderer};
use iced::keyboard::{self, key};
use iced::widget::scrollable::{self, AbsoluteOffset, RelativeOffset, Viewport};
use iced::widget::{Column, Id, Scrollable, Stack, container};
use iced::{
    Background, Border, Color, Event, Length, Padding, Rectangle, Size, Task, Theme, Vector, mouse,
};

/// The width of the stable scrollbar gutter (`scrollbar-thin
/// scrollbar-gutter-stable`), reserved at the end of the viewport.
const GUTTER: f32 = 10.0;

/// The width of the scrollbar thumb, centered in the gutter.
const THUMB: f32 = 6.0;

/// How far an arrow key scrolls, like browsers do.
const LINE: f32 = 40.0;

/// A message scroller. See the [module documentation](self).
pub struct MessageScroller<'a, Message> {
    items: Vec<Element<'a, Message>>,
    id: Option<Id>,
    padding: Padding,
    on_scroll: Option<Box<dyn Fn(Viewport) -> Message + 'a>>,
    button: Option<Message>,
}

/// Creates a scroller showing `items` from top to bottom, anchored to the
/// bottom (`MessageScrollerContent`, `gap-6`).
pub fn message_scroller<'a, Message: Clone + 'a>(
    items: impl IntoIterator<Item = Element<'a, Message>>,
) -> MessageScroller<'a, Message> {
    MessageScroller {
        items: items.into_iter().collect(),
        id: None,
        padding: Padding::new(16.0),
        on_scroll: None,
        button: None,
    }
}

impl<'a, Message: Clone + 'a> MessageScroller<'a, Message> {
    /// Sets the id of the scroller, used by [`scroll_to_end`].
    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the padding of the content (`p-(--card-spacing)` in the
    /// examples; 16 by default).
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Reports the viewport whenever it scrolls; see [`is_at_end`].
    pub fn on_scroll(mut self, f: impl Fn(Viewport) -> Message + 'a) -> Self {
        self.on_scroll = Some(Box::new(f));
        self
    }

    /// Shows the scroll to end button (`MessageScrollerButton`) producing
    /// the given message, if any.
    pub fn button(mut self, on_press: Option<Message>) -> Self {
        self.button = on_press;
        self
    }
}

impl<'a, Message: Clone + 'a> From<MessageScroller<'a, Message>> for Element<'a, Message> {
    fn from(scroller: MessageScroller<'a, Message>) -> Self {
        // `scrollbar-gutter-stable`: the gutter is always reserved, and the
        // thin scrollbar is drawn inside it.
        let padding = Padding {
            right: scroller.padding.right + GUTTER,
            ..scroller.padding
        };

        let mut scrollable = Scrollable::new(
            container(
                Column::with_children(scroller.items)
                    .spacing(24)
                    .width(Length::Fill),
            )
            .padding(padding),
        )
        // The anchor belongs to the scrollbar: set it after the direction.
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::new()
                .width(THUMB)
                .scroller_width(THUMB)
                .margin((GUTTER - THUMB) / 2.0),
        ))
        .anchor_bottom()
        .style(style)
        .width(Length::Fill)
        .height(Length::Fill);

        if let Some(id) = scroller.id {
            scrollable = scrollable.id(id);
        }

        if let Some(on_scroll) = scroller.on_scroll {
            scrollable = scrollable.on_scroll(on_scroll);
        }

        // The scrollable stays the first child of the stack whether the
        // button shows or not, so it keeps its offset.
        let mut content = Stack::new()
            .push(scrollable)
            .width(Length::Fill)
            .height(Length::Fill);

        if let Some(on_press) = scroller.button {
            // `variant="secondary" size="icon-sm"`, restyled with
            // `border-border bg-background hover:bg-muted`, centered at
            // `bottom-4`.
            content = content.push(
                container(
                    icon_button(lucide::ARROW_DOWN)
                        .variant(button::Variant::Outline)
                        .size(button::Size::IconSm)
                        .on_press(on_press),
                )
                .center_x(Length::Fill)
                .align_bottom(Length::Fill)
                .padding(Padding::ZERO.bottom(16)),
            );
        }

        let content = content.into();

        Element::new(Focusable { content })
    }
}

/// The style of the viewport: a thin `bg-border` thumb, shown while the
/// pointer is over the conversation, like overlay scrollbars.
fn style(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let style = super::scroll_area::style(theme, status);

    match status {
        scrollable::Status::Hovered { .. } | scrollable::Status::Dragged { .. } => style,
        scrollable::Status::Active { .. } => {
            let mut rail = style.vertical_rail;
            rail.scroller.background = Background::Color(Color::TRANSPARENT);

            scrollable::Style {
                vertical_rail: rail,
                ..style
            }
        }
    }
}

/// Whether the viewport of a [`MessageScroller`] shows the newest message
/// (or everything fits).
///
/// The scroller is anchored at the end, so its offset is measured from the
/// newest message.
pub fn is_at_end(viewport: &Viewport) -> bool {
    viewport.absolute_offset().y <= 1.0
}

/// Scrolls the scroller with the given id to the newest message.
pub fn scroll_to_end<Message: Send + 'static>(id: impl Into<Id>) -> Task<Message> {
    iced::widget::operation::snap_to_end(id)
}

/// The viewport of the scroller, focused by a click inside: it then scrolls
/// with the keyboard and shows its outline (`outline-ring/50`).
struct Focusable<'a, Message> {
    content: Element<'a, Message>,
}

#[derive(Debug, Clone, Copy, Default)]
struct State {
    is_focused: bool,
    is_focus_visible: bool,
}

/// Where a scrolling key moves the viewport.
#[derive(Debug, Clone, Copy)]
enum Scroll {
    Start,
    End,
    By(f32),
}

impl Operation for Scroll {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn scrollable(
        &mut self,
        _id: Option<&Id>,
        bounds: Rectangle,
        content_bounds: Rectangle,
        _translation: Vector,
        state: &mut dyn operation::Scrollable,
    ) {
        // The offset of a scrollable anchored at the end grows towards the
        // start.
        match *self {
            Scroll::Start => state.snap_to(RelativeOffset {
                x: None,
                y: Some(1.0),
            }),
            Scroll::End => state.snap_to(RelativeOffset {
                x: None,
                y: Some(0.0),
            }),
            Scroll::By(y) => {
                state.scroll_by(AbsoluteOffset { x: 0.0, y: -y }, bounds, content_bounds);
            }
        }
    }
}

impl<Message> Widget<Message, Theme, iced::Renderer> for Focusable<'_, Message> {
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
        renderer: &iced::Renderer,
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
        renderer: &iced::Renderer,
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
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        if let Event::Mouse(mouse::Event::ButtonPressed(_)) = event {
            let state = tree.state.downcast_mut::<State>();
            let is_focused = cursor.is_over(layout.bounds());

            if state.is_focus_visible || state.is_focused != is_focused {
                state.is_focused = is_focused;
                state.is_focus_visible = false;
                shell.request_redraw();
            }
        }

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        if shell.is_event_captured() {
            return;
        }

        let Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = event else {
            return;
        };

        let state = tree.state.downcast_mut::<State>();

        if !state.is_focused || modifiers.command() || modifiers.alt() {
            return;
        }

        let page = (layout.bounds().height - LINE).max(LINE);
        let mut scroll = match key.as_ref() {
            keyboard::Key::Named(key::Named::Home) => Scroll::Start,
            keyboard::Key::Named(key::Named::End) => Scroll::End,
            keyboard::Key::Named(key::Named::PageUp) => Scroll::By(-page),
            keyboard::Key::Named(key::Named::PageDown) => Scroll::By(page),
            keyboard::Key::Named(key::Named::Space) if modifiers.shift() => Scroll::By(-page),
            keyboard::Key::Named(key::Named::Space) => Scroll::By(page),
            keyboard::Key::Named(key::Named::ArrowUp) => Scroll::By(-LINE),
            keyboard::Key::Named(key::Named::ArrowDown) => Scroll::By(LINE),
            _ => return,
        };

        self.content.as_widget_mut().operate(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            &mut scroll,
        );

        state.is_focus_visible = true;
        shell.capture_event();
        shell.request_redraw();
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        use iced::advanced::Renderer as _;

        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout.children().next().unwrap(),
            cursor,
            viewport,
        );

        if tree.state.downcast_ref::<State>().is_focus_visible {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds(),
                    border: Border {
                        color: alpha(tokens(theme).palette.ring, 0.5),
                        width: 1.0,
                        ..Border::default()
                    },
                    ..renderer::Quad::default()
                },
                Color::TRANSPARENT,
            );
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
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
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            viewport,
            translation,
        )
    }
}
