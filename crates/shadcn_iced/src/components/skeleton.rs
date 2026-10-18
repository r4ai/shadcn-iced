//! Use to show a placeholder while content is loading.
//!
//! Port of <https://ui.shadcn.com/docs/components/skeleton>. Pulses like
//! Tailwind's `animate-pulse`.
use shadcn_iced_core::theme::{alpha, tokens};
use shadcn_iced_core::{Element, animation};

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Tree, Widget};
use iced::advanced::{Clipboard, Shell};
use iced::time::Instant;
use iced::{Background, Event, Length, Rectangle, Size, Theme, border, mouse, window};

use std::sync::LazyLock;

static START: LazyLock<Instant> = LazyLock::new(Instant::now);

/// A pulsing placeholder (`bg-muted rounded-md animate-pulse`). See the
/// [module documentation](self).
pub struct Skeleton {
    width: Length,
    height: Length,
    radius: Option<f32>,
}

/// Creates a [`Skeleton`] of the given size.
pub fn skeleton(width: impl Into<Length>, height: impl Into<Length>) -> Skeleton {
    Skeleton {
        width: width.into(),
        height: height.into(),
        radius: None,
    }
}

impl Skeleton {
    /// Sets the corner radius. Defaults to `rounded-md`.
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = Some(radius);
        self
    }

    /// Makes the skeleton fully rounded (`rounded-full`), e.g. for avatars.
    pub fn rounded_full(self) -> Self {
        self.radius(f32::MAX)
    }
}

impl<Message> Widget<Message, Theme, iced::Renderer> for Skeleton {
    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, self.width, self.height)
    }

    fn update(
        &mut self,
        _tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        // Pulses only while animations are enabled and the skeleton is
        // visible: the redraw of a scroll brings it back into view.
        if let Event::Window(window::Event::RedrawRequested(now)) = event
            && animation::enabled()
            && layout.bounds().intersects(viewport)
        {
            shell.request_redraw_at(*now + std::time::Duration::from_millis(33));
        }
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        use iced::advanced::Renderer as _;

        let t = tokens(theme);

        // animate-pulse: opacity 1 -> 0.5 -> 1 over 2s, cubic-bezier(0.4, 0, 0.6, 1),
        // at rest (opaque) when animations are disabled.
        let opacity = if animation::enabled() {
            let phase = (START.elapsed().as_secs_f32() % 2.0) / 2.0;
            0.75 + 0.25 * (phase * std::f32::consts::TAU).cos()
        } else {
            1.0
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds: layout.bounds(),
                border: border::rounded(self.radius.unwrap_or(t.radius_md())),
                ..renderer::Quad::default()
            },
            Background::Color(alpha(t.palette.muted, opacity)),
        );
    }
}

impl<'a, Message: 'a> From<Skeleton> for Element<'a, Message> {
    fn from(skeleton: Skeleton) -> Self {
        Element::new(skeleton)
    }
}
