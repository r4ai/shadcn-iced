//! The page around every fixture: `<div class="p-4">` in the `<body>` of the
//! web runner.
//!
//! A browser page lets content taller than the viewport overflow (the
//! screenshot shows its top part), while iced lays everything out within the
//! window, which squeezes it. So the page lays its content out with an
//! unbounded height, and only falls back to the height of the viewport when
//! the content fills it (`Length::Fill`), where an unbounded height has no
//! meaning. The page itself keeps the size of the viewport, so overlays are
//! still placed within the window, like on the web.
use crate::store::Msg;

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::{Element, Event, Length, Padding, Rectangle, Size, Theme, Vector, mouse};

/// The padding of the page (`p-4`).
const PADDING: f32 = 16.0;

pub struct Page<'a> {
    content: Element<'a, Msg>,
}

/// Wraps the content of a fixture in the page.
pub fn page(content: Element<'_, Msg>) -> Element<'_, Msg> {
    Element::new(Page { content })
}

impl Widget<Msg, Theme, iced::Renderer> for Page<'_> {
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = limits.max();
        let padding = Padding::new(PADDING);
        let width = (size.width - padding.x()).max(0.0);

        let unbounded = layout::Limits::new(Size::ZERO, Size::new(width, f32::INFINITY));
        let mut content =
            self.content
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, &unbounded);

        if !content.size().height.is_finite() {
            let bounded = layout::Limits::new(
                Size::ZERO,
                Size::new(width, (size.height - padding.y()).max(0.0)),
            );
            content =
                self.content
                    .as_widget_mut()
                    .layout(&mut tree.children[0], renderer, &bounded);
        }

        layout::Node::with_children(size, vec![content.move_to((PADDING, PADDING))])
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
        shell: &mut Shell<'_, Msg>,
        viewport: &Rectangle,
    ) {
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
    ) -> Option<overlay::Element<'b, Msg, Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            viewport,
            translation,
        )
    }
}
