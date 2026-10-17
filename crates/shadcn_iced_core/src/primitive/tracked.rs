//! Text with letter spacing (`tracking-*`).
//!
//! iced cannot space the letters of a text, so [`Tracked`] lays out every
//! character as a text of its own, adding the spacing after each of them
//! like CSS `letter-spacing` does (including after the last one). It still
//! behaves as a single text for operations (e.g. finding a text).
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget};
use iced::advanced::{self, Clipboard, Shell};
use iced::widget::text::{self, LineHeight};
use iced::{Element, Event, Font, Length, Padding, Pixels, Rectangle, Size, mouse};

/// Text with letter spacing. See the [module documentation](self).
pub struct Tracked<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    content: String,
    row: Element<'a, Message, Theme, Renderer>,
}

/// Creates [`Tracked`] text with the given size, line height, font and
/// spacing between letters (e.g. `tracking-widest` is `0.1em`).
///
/// The text inherits its color.
pub fn tracked<'a, Message, Theme, Renderer>(
    content: impl Into<String>,
    size: f32,
    line_height: impl Into<LineHeight>,
    font: Option<Font>,
    tracking: f32,
) -> Tracked<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: text::Catalog + 'a,
    Renderer: advanced::text::Renderer<Font = Font> + 'a,
{
    let content = content.into();
    let line_height = line_height.into();

    let row = iced::widget::Row::with_children(content.chars().map(|c| {
        let mut text = iced::widget::text(c.to_string())
            .size(Pixels(size))
            .line_height(line_height)
            .wrapping(text::Wrapping::None);

        if let Some(font) = font {
            text = text.font(font);
        }

        text.into()
    }))
    .spacing(tracking)
    .padding(Padding::ZERO.right(tracking))
    .into();

    Tracked { content, row }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Tracked<'a, Message, Theme, Renderer>
where
    Renderer: advanced::Renderer + 'a,
{
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.row)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.row));
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let node = self
            .row
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);

        layout::Node::with_children(node.size(), vec![node])
    }

    fn operate(
        &mut self,
        _tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.text(None, layout.bounds(), &self.content);
    }

    fn update(
        &mut self,
        _tree: &mut Tree,
        _event: &Event,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        _shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
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
        if let Some(row) = layout.children().next() {
            self.row.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                row,
                cursor,
                viewport,
            );
        }
    }
}

impl<'a, Message, Theme, Renderer> From<Tracked<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: advanced::Renderer + 'a,
{
    fn from(tracked: Tracked<'a, Message, Theme, Renderer>) -> Self {
        Element::new(tracked)
    }
}
