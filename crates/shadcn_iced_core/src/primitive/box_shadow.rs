//! Layered drop shadows with a spread, like CSS `box-shadow`.
//!
//! iced draws a single shadow per quad, without a spread, so Tailwind's
//! shadows (two layers with negative spreads, see [`style::SHADOW_MD`])
//! cannot be set on a container style. [`BoxShadow`] draws each layer as an
//! invisible quad of its own behind the content, grown or shrunk by its
//! spread.
//!
//! ```no_run
//! use shadcn_iced_core::primitive::box_shadow;
//! use shadcn_iced_core::{style, tokens};
//! # fn view<'a, Message: 'a>(panel: iced::Element<'a, Message>) -> iced::Element<'a, Message> {
//! box_shadow(panel, style::SHADOW_MD, |theme| tokens(theme).radius_lg()).into()
//! # }
//! ```
//!
//! [`style::SHADOW_MD`]: crate::style::SHADOW_MD
use crate::theme::tokens;

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::{Operation, Tree, Widget};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::{Color, Element, Event, Length, Rectangle, Size, Theme, Vector, mouse};

/// A layer of a [`BoxShadow`], in logical pixels.
///
/// The fade differs from CSS: iced fades a shadow out over `blur_radius`
/// from half its opacity at its edge, while CSS blurs it with a Gaussian of
/// half the blur radius. The layers of [`style`](crate::style) are fitted to
/// the CSS ones rather than copied.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layer {
    /// The color of the shadow.
    pub color: Color,
    /// How far the shadow is moved.
    pub offset: Vector,
    /// How far the shadow fades out.
    pub blur_radius: f32,
    /// How far the shadow grows (or shrinks, when negative) on every side.
    pub spread: f32,
}

/// Content with drop shadows behind it. See the [module
/// documentation](self).
pub struct BoxShadow<'a, Message, Renderer = iced::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    layers: &'a [Layer],
    radius: Box<dyn Fn(&Theme) -> f32 + 'a>,
    inset: f32,
}

/// Draws `layers` behind `content`, whose corners have the given radius.
pub fn box_shadow<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    layers: &'a [Layer],
    radius: impl Fn(&Theme) -> f32 + 'a,
) -> BoxShadow<'a, Message, Renderer> {
    BoxShadow {
        content: content.into(),
        layers,
        radius: Box::new(radius),
        inset: 0.0,
    }
}

impl<Message, Renderer> BoxShadow<'_, Message, Renderer> {
    /// Shrinks the box that casts the shadows, e.g. by the width of a
    /// `ring-1`: CSS draws rings outside of the box, while shadcn_iced
    /// draws them as a border inside of it.
    pub fn inset(mut self, inset: f32) -> Self {
        self.inset = inset;
        self
    }
}

/// The quads that draw `layers` behind content with the given bounds and
/// corner radius, faded by `opacity`.
pub fn quads(
    layers: &[Layer],
    bounds: Rectangle,
    radius: f32,
    opacity: f32,
) -> impl Iterator<Item = Quad> + '_ {
    layers.iter().map(move |layer| {
        let bounds = bounds.expand(layer.spread);

        Quad {
            bounds,
            border: iced::Border {
                radius: (radius + layer.spread)
                    .max(0.0)
                    .min(bounds.width.min(bounds.height) / 2.0)
                    .into(),
                ..iced::Border::default()
            },
            shadow: iced::Shadow {
                color: layer.color.scale_alpha(opacity),
                offset: layer.offset,
                blur_radius: layer.blur_radius,
            },
            snap: false,
        }
    })
}

impl<Message, Renderer> Widget<Message, Theme, Renderer> for BoxShadow<'_, Message, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
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
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        // A theme faded by an animation (`animation::Fade`) fades the
        // shadows too.
        let opacity = tokens(theme).palette.foreground.a;

        let bounds = layout.bounds().shrink(self.inset);
        let radius = ((self.radius)(theme) - self.inset).max(0.0);

        for quad in quads(self.layers, bounds, radius, opacity) {
            renderer.fill_quad(quad, Color::TRANSPARENT);
        }

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

impl<'a, Message: 'a, Renderer> From<BoxShadow<'a, Message, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(box_shadow: BoxShadow<'a, Message, Renderer>) -> Self {
        Element::new(box_shadow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spreads_shrink_the_quads_and_their_radius() {
        let layers = [Layer {
            color: Color::BLACK,
            offset: Vector::new(0.0, 4.0),
            blur_radius: 6.0,
            spread: -2.0,
        }];

        let quad = quads(
            &layers,
            Rectangle::new([10.0, 10.0].into(), [100.0, 50.0].into()),
            8.0,
            0.5,
        )
        .next()
        .unwrap();

        assert_eq!(
            quad.bounds,
            Rectangle::new([12.0, 12.0].into(), [96.0, 46.0].into())
        );
        assert_eq!(quad.border.radius, 6.0.into());
        assert_eq!(quad.shadow.color.a, 0.5);
        assert_eq!(quad.shadow.offset, Vector::new(0.0, 4.0));
    }
}
