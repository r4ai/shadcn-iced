//! An indicator that can be used to show a loading state.
//!
//! Port of <https://ui.shadcn.com/docs/components/spinner>: a spinning
//! `loader-circle` icon.
use shadcn_iced_core::{Element, animation, tokens};

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::time::Instant;
use iced::widget::canvas::{self, Canvas, Path, Stroke, path::Arc};
use iced::{Color, Event, Length, Point, Radians, Rectangle, Renderer, Size, Theme, mouse, window};

use std::sync::LazyLock;

static START: LazyLock<Instant> = LazyLock::new(Instant::now);

/// A spinner. See the [module documentation](self).
#[derive(Debug, Clone, Copy)]
pub struct Spinner {
    size: f32,
    color: Option<Color>,
    style: Option<fn(&Theme) -> Color>,
}

/// Creates a 16x16 [`Spinner`] (`size-4 animate-spin`).
pub fn spinner() -> Spinner {
    Spinner {
        size: 16.0,
        color: None,
        style: None,
    }
}

impl Spinner {
    /// Sets the size.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Sets the color. Defaults to the foreground color.
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Sets the color from the theme, e.g. to match the text of the badge
    /// or button around it (`currentColor`).
    pub fn style(mut self, style: fn(&Theme) -> Color) -> Self {
        self.style = Some(style);
        self
    }
}

impl<Message> canvas::Program<Message> for Spinner {
    type State = ();

    fn update(
        &self,
        _state: &mut (),
        event: &Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        (animation::enabled() && matches!(event, Event::Window(window::Event::RedrawRequested(_))))
            .then(canvas::Action::request_redraw)
    }

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let scale = self.size / 24.0;
        // `animate-spin`: a turn per second, at rest when animations are
        // disabled (like `prefers-reduced-motion`).
        let rotation = if animation::enabled() {
            START.elapsed().as_secs_f32() * std::f32::consts::TAU
        } else {
            0.0
        };
        let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);

        let arc = Path::new(|builder| {
            builder.arc(Arc {
                center,
                radius: 9.0 * scale,
                start_angle: Radians(rotation),
                // `M21 12a9 9 0 1 1-6.219-8.56`: a 288° arc.
                end_angle: Radians(rotation + std::f32::consts::PI * 1.6),
            });
        });

        frame.stroke(
            &arc,
            Stroke::default()
                .with_width(2.0 * scale)
                .with_color(self.color.unwrap_or_else(|| {
                    self.style
                        .map(|style| style(theme))
                        .unwrap_or(tokens(theme).palette.foreground)
                }))
                .with_line_cap(canvas::LineCap::Round),
        );

        vec![frame.into_geometry()]
    }
}

impl<'a, Message: 'a> From<Spinner> for Element<'a, Message> {
    fn from(spinner: Spinner) -> Self {
        let size = Length::Fixed(spinner.size);

        Element::new(OnScreen(
            Canvas::new(spinner).width(size).height(size).into(),
        ))
    }
}

/// Spins only while the spinner is visible: the frames of an off-screen
/// spinner do not reach its canvas, and the redraw of a scroll brings it
/// back into view.
struct OnScreen<'a, Message>(Element<'a, Message>);

impl<Message> Widget<Message, Theme, Renderer> for OnScreen<'_, Message> {
    fn tag(&self) -> tree::Tag {
        self.0.as_widget().tag()
    }

    fn state(&self) -> tree::State {
        self.0.as_widget().state()
    }

    fn children(&self) -> Vec<Tree> {
        self.0.as_widget().children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.0.as_widget().diff(tree);
    }

    fn size(&self) -> Size<Length> {
        self.0.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.0.as_widget_mut().layout(tree, renderer, limits)
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
        if let Event::Window(window::Event::RedrawRequested(_)) = event
            && !layout.bounds().intersects(viewport)
        {
            return;
        }

        self.0.as_widget_mut().update(
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
        self.0
            .as_widget()
            .draw(tree, renderer, theme, style, layout, cursor, viewport);
    }
}
