//! An indicator that can be used to show a loading state.
//!
//! Port of <https://ui.shadcn.com/docs/components/spinner>: a spinning
//! `loader-circle` icon.
use iced_ui_core::{Element, tokens};

use iced::time::Instant;
use iced::widget::canvas::{self, Canvas, Path, Stroke, path::Arc};
use iced::{Color, Event, Length, Point, Radians, Rectangle, Renderer, Theme, mouse, window};

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
        matches!(event, Event::Window(window::Event::RedrawRequested(_)))
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
        let rotation = START.elapsed().as_secs_f32() * std::f32::consts::TAU;
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

        Canvas::new(spinner).width(size).height(size).into()
    }
}
