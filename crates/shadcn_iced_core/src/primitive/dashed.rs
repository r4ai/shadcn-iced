//! Dashed borders (`border border-dashed`), which iced quads cannot draw.
//!
//! ```no_run
//! use shadcn_iced_core::primitive::dashed_border;
//! use shadcn_iced_core::tokens;
//! # fn view<'a, Message: 'a>(content: iced::Element<'a, Message>) -> iced::Element<'a, Message> {
//! // `rounded-xl border border-dashed`
//! dashed_border(content, |theme| {
//!     let t = tokens(theme);
//!     (t.palette.border, t.radius_xl())
//! })
//! .into()
//! # }
//! ```
use iced::advanced::graphics::geometry::Renderer as _;
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Renderer as _};
use iced::advanced::widget::{Operation, Tree, Widget};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::widget::canvas::{self, Frame, Path, Stroke};
use iced::{Color, Element, Event, Length, Point, Rectangle, Size, Theme, Vector, mouse};

/// The dashes of a 1px dashed border, as browsers draw them: 3px dashes
/// with 2px gaps.
pub const DASHES: &[f32] = &[3.0, 2.0];

/// Draws a 1px dashed border of the given color and corner radius just
/// inside `bounds`.
pub fn draw(renderer: &mut iced::Renderer, bounds: Rectangle, color: Color, radius: f32) {
    renderer.with_translation(Vector::new(bounds.x, bounds.y), |renderer| {
        let mut frame = Frame::new(renderer, bounds.size());

        // Centered on the 1px border.
        frame.stroke(
            &Path::rounded_rectangle(
                Point::new(0.5, 0.5),
                Size::new(bounds.width - 1.0, bounds.height - 1.0),
                (radius - 0.5).max(0.0).into(),
            ),
            Stroke {
                line_dash: canvas::LineDash {
                    segments: DASHES,
                    offset: 0,
                },
                ..Stroke::default().with_color(color).with_width(1.0)
            },
        );

        renderer.draw_geometry(frame.into_geometry());
    });
}

/// Content with a dashed border drawn over its edge. See [`dashed_border`].
pub struct DashedBorder<'a, Message> {
    content: Element<'a, Message>,
    style: Box<dyn Fn(&Theme) -> (Color, f32) + 'a>,
}

/// Draws a 1px dashed border over the edge of `content`, with the color and
/// the corner radius given by `style`. The content keeps no border of its
/// own (or a transparent one) to make room for it.
pub fn dashed_border<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    style: impl Fn(&Theme) -> (Color, f32) + 'a,
) -> DashedBorder<'a, Message> {
    DashedBorder {
        content: content.into(),
        style: Box::new(style),
    }
}

impl<Message> Widget<Message, Theme, iced::Renderer> for DashedBorder<'_, Message> {
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        self.content.as_widget().tag()
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
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
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content.as_widget_mut().layout(tree, renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
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
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            tree, event, layout, cursor, renderer, clipboard, shell, viewport,
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
        self.content
            .as_widget()
            .draw(tree, renderer, theme, style, layout, cursor, viewport);

        let (color, radius) = (self.style)(theme);
        draw(renderer, layout.bounds(), color, radius);
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.content
            .as_widget()
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, iced::Renderer>> {
        self.content
            .as_widget_mut()
            .overlay(tree, layout, renderer, viewport, translation)
    }
}

impl<'a, Message: 'a> From<DashedBorder<'a, Message>> for Element<'a, Message> {
    fn from(border: DashedBorder<'a, Message>) -> Self {
        Element::new(border)
    }
}
