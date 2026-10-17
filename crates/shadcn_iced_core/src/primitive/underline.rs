//! Underlines some text while the pointer is over it (`hover:underline`).
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::{
    Background, Color, Element, Event, Length, Padding, Rectangle, Size, Theme, Vector, mouse,
};

/// Underlines its content while hovered.
///
/// The content is expected to be a single line of text, vertically centered
/// in its bounds, like the label of a button. The underline spans the width
/// of the content and sits `offset` pixels below the baseline
/// (`underline-offset-4` by default).
pub struct Underline<'a, Message, Renderer = iced::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    text_size: f32,
    offset: f32,
    hover_area: Padding,
    always: bool,
    color: Option<Box<dyn Fn(&Theme) -> Color + 'a>>,
}

/// Creates a new [`Underline`].
pub fn underline<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Underline<'a, Message, Renderer> {
    Underline {
        content: content.into(),
        text_size: 14.0,
        offset: 4.0,
        hover_area: Padding::ZERO,
        always: false,
        color: None,
    }
}

impl<'a, Message, Renderer> Underline<'a, Message, Renderer> {
    /// The size of the text, to find its baseline.
    pub fn text_size(mut self, size: f32) -> Self {
        self.text_size = size;
        self
    }

    /// The distance between the baseline and the underline.
    pub fn offset(mut self, offset: f32) -> Self {
        self.offset = offset;
        self
    }

    /// Extends the area that triggers the underline beyond the content, e.g.
    /// to the padding of the surrounding button.
    pub fn hover_area(mut self, padding: impl Into<Padding>) -> Self {
        self.hover_area = padding.into();
        self
    }

    /// Underlines the content even when not hovered (`underline`).
    pub fn always(mut self, always: bool) -> Self {
        self.always = always;
        self
    }

    /// The color of the underline; the color of the text by default.
    pub fn color(mut self, color: impl Fn(&Theme) -> Color + 'a) -> Self {
        self.color = Some(Box::new(color));
        self
    }
}

/// The distance between the vertical center of a line of Geist text and its
/// baseline, relative to the font size.
const BASELINE: f32 = 0.357;

impl<Message, Renderer> Widget<Message, Theme, Renderer> for Underline<'_, Message, Renderer>
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
        // The underline depends on the position of the cursor only.
        if let Event::Mouse(mouse::Event::CursorMoved { .. }) = event {
            shell.request_redraw();
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

        let bounds = layout.bounds();

        if !self.always && !cursor.is_over(expand(bounds, self.hover_area)) {
            return;
        }

        let color = self
            .color
            .as_ref()
            .map_or(style.text_color, |color| color(theme));

        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: bounds.x,
                    y: bounds.center_y() + self.text_size * BASELINE + self.offset,
                    width: bounds.width,
                    height: 1.0,
                },
                ..renderer::Quad::default()
            },
            Background::Color(color),
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

fn expand(bounds: Rectangle, padding: Padding) -> Rectangle {
    Rectangle {
        x: bounds.x - padding.left,
        y: bounds.y - padding.top,
        width: bounds.width + padding.left + padding.right,
        height: bounds.height + padding.top + padding.bottom,
    }
}

impl<'a, Message: 'a, Renderer> From<Underline<'a, Message, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(underline: Underline<'a, Message, Renderer>) -> Self {
        Element::new(underline)
    }
}
