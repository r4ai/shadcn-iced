//! Keyboard scrolling of a scrollable, like a scrollable `div` of a browser.
//!
//! A click inside of a scrollable `div` makes it the target of the
//! scrolling keys: <kbd>Home</kbd> and <kbd>End</kbd> go to the start and
//! the end, <kbd>PageUp</kbd>, <kbd>PageDown</kbd> and <kbd>Space</kbd>
//! (with <kbd>Shift</kbd> upwards) scroll by a page, <kbd>↑</kbd> and
//! <kbd>↓</kbd> by a line. [`ScrollKeys`] does the same for the vertical
//! scrollables of its content (the horizontal ones with
//! [`ScrollKeys::horizontal`]), and shows the focus once a key scrolled
//! them (`:focus-visible`) through [`ScrollKeys::outline`]. A tap focuses
//! it like a click does.
//!
//! ```no_run
//! # use shadcn_iced_core::primitive::scroll_keys;
//! # use iced::widget::{scrollable, text};
//! # fn view<'a, Message: 'a>() -> iced::Element<'a, Message> {
//! scroll_keys(scrollable(text("A long text...")))
//!     .outline(|_theme, status| iced::Border {
//!         color: iced::Color::BLACK,
//!         width: if status.focus_visible { 1.0 } else { 0.0 },
//!         ..iced::Border::default()
//!     })
//!     .into()
//! # }
//! ```
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Id, Operation, Tree, Widget, operation, tree};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::keyboard::{self, key};
use iced::widget::scrollable::{AbsoluteOffset, RelativeOffset};
use iced::{Border, Color, Element, Event, Length, Rectangle, Size, Vector, mouse, touch};

/// How far an arrow key scrolls, like browsers do.
pub const LINE: f32 = 40.0;

/// The focus of a [`ScrollKeys`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Status {
    /// Whether the last press was inside of it, making it the target of the
    /// scrolling keys.
    pub focused: bool,
    /// Whether a key scrolled it since then (`:focus-visible`).
    pub focus_visible: bool,
}

/// Scrolls the scrollables of its content with the keyboard. See the
/// [module documentation](self).
pub struct ScrollKeys<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    anchor_end: bool,
    vertical: bool,
    horizontal: bool,
    outline: Option<Box<dyn Fn(&Theme, Status) -> Border + 'a>>,
}

/// Creates a [`ScrollKeys`] around `content`, which holds the scrollable.
pub fn scroll_keys<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> ScrollKeys<'a, Message, Theme, Renderer> {
    ScrollKeys {
        content: content.into(),
        anchor_end: false,
        vertical: true,
        horizontal: false,
        outline: None,
    }
}

impl<'a, Message, Theme, Renderer> ScrollKeys<'a, Message, Theme, Renderer> {
    /// Tells that the scrollable is anchored at the end
    /// ([`anchor_bottom`]), whose offset grows towards the start.
    ///
    /// [`anchor_bottom`]: iced::widget::Scrollable::anchor_bottom
    pub fn anchor_end(mut self, anchor_end: bool) -> Self {
        self.anchor_end = anchor_end;
        self
    }

    /// Sets whether the scrollables scroll vertically (the default): the
    /// page and boundary keys then move along the vertical axis, else along
    /// the horizontal one.
    pub fn vertical(mut self, vertical: bool) -> Self {
        self.vertical = vertical;
        self
    }

    /// Sets whether <kbd>←</kbd> and <kbd>→</kbd> scroll horizontally.
    pub fn horizontal(mut self, horizontal: bool) -> Self {
        self.horizontal = horizontal;
        self
    }

    /// Draws a border over the bounds, depending on the [`Status`].
    pub fn outline(mut self, outline: impl Fn(&Theme, Status) -> Border + 'a) -> Self {
        self.outline = Some(Box::new(outline));
        self
    }
}

/// Where a scrolling key moves the viewport.
#[derive(Debug, Clone, Copy)]
enum Scroll {
    /// The start of the vertical (or else horizontal) axis.
    Start(bool),
    End(bool),
    By(Vector),
}

/// Applies a [`Scroll`] to the scrollables of the content.
struct ScrollOperation {
    scroll: Scroll,
    anchor_end: bool,
}

impl Operation for ScrollOperation {
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
        let (start, end, sign) = if self.anchor_end {
            (1.0, 0.0, -1.0)
        } else {
            (0.0, 1.0, 1.0)
        };

        match self.scroll {
            Scroll::Start(vertical) => state.snap_to(RelativeOffset {
                x: (!vertical).then_some(start),
                y: vertical.then_some(start),
            }),
            Scroll::End(vertical) => state.snap_to(RelativeOffset {
                x: (!vertical).then_some(end),
                y: vertical.then_some(end),
            }),
            Scroll::By(by) => {
                state.scroll_by(
                    AbsoluteOffset {
                        x: sign * by.x,
                        y: sign * by.y,
                    },
                    bounds,
                    content_bounds,
                );
            }
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for ScrollKeys<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Status>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(Status::default())
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
        if let Event::Mouse(mouse::Event::ButtonPressed(_))
        | Event::Touch(touch::Event::FingerPressed { .. }) = event
        {
            let status = tree.state.downcast_mut::<Status>();
            let focused = cursor.is_over(layout.bounds());

            if status.focus_visible || status.focused != focused {
                status.focused = focused;
                status.focus_visible = false;
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

        let status = tree.state.downcast_mut::<Status>();

        if !status.focused || modifiers.command() || modifiers.alt() {
            return;
        }

        let vertical = self.vertical;
        let bounds = layout.bounds();
        let along = |distance: f32| {
            if vertical {
                Vector::new(0.0, distance)
            } else {
                Vector::new(distance, 0.0)
            }
        };
        let page = (if vertical {
            bounds.height
        } else {
            bounds.width
        } - LINE)
            .max(LINE);
        let scroll = match key.as_ref() {
            keyboard::Key::Named(key::Named::Home) => Scroll::Start(vertical),
            keyboard::Key::Named(key::Named::End) => Scroll::End(vertical),
            keyboard::Key::Named(key::Named::PageUp) => Scroll::By(along(-page)),
            keyboard::Key::Named(key::Named::PageDown) => Scroll::By(along(page)),
            keyboard::Key::Named(key::Named::Space) if modifiers.shift() => {
                Scroll::By(along(-page))
            }
            keyboard::Key::Named(key::Named::Space) => Scroll::By(along(page)),
            keyboard::Key::Named(key::Named::ArrowUp) if vertical => {
                Scroll::By(Vector::new(0.0, -LINE))
            }
            keyboard::Key::Named(key::Named::ArrowDown) if vertical => {
                Scroll::By(Vector::new(0.0, LINE))
            }
            keyboard::Key::Named(key::Named::ArrowLeft) if self.horizontal => {
                Scroll::By(Vector::new(-LINE, 0.0))
            }
            keyboard::Key::Named(key::Named::ArrowRight) if self.horizontal => {
                Scroll::By(Vector::new(LINE, 0.0))
            }
            _ => return,
        };

        self.content.as_widget_mut().operate(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            &mut ScrollOperation {
                scroll,
                anchor_end: self.anchor_end,
            },
        );

        tree.state.downcast_mut::<Status>().focus_visible = true;
        shell.capture_event();
        shell.request_redraw();
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

        let Some(outline) = &self.outline else {
            return;
        };

        let border = outline(theme, *tree.state.downcast_ref::<Status>());

        if border.width > 0.0 && border.color.a > 0.0 {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds(),
                    border,
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

impl<'a, Message, Theme, Renderer> From<ScrollKeys<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(keys: ScrollKeys<'a, Message, Theme, Renderer>) -> Self {
        Element::new(keys)
    }
}
