//! Animations, ported from the `tw-animate-css` utilities used by shadcn/ui.
//!
//! iced has no CSS transitions, so animated widgets keep a [`Transition`] in
//! their state and redraw themselves until it settles. Two effects are
//! available while drawing:
//!
//! - **transforms** (`zoom-in-95`, `slide-in-from-top-2`, sheets sliding in)
//!   through [`iced::Transformation`], and
//! - **opacity** (`fade-in-0`), by drawing a subtree with a copy of the
//!   theme whose colors are translucent (see [`Fade`]).
//!
//! Animations can be turned off globally with [`set_enabled`], e.g. for
//! deterministic screenshots or to honor a reduced-motion preference.
use iced::Transformation;
use iced::time::{Duration, Instant};

use std::sync::atomic::{self, AtomicBool};

static ENABLED: AtomicBool = AtomicBool::new(true);

/// Turns every animation of shadcn_iced on or off.
pub fn set_enabled(enabled: bool) {
    ENABLED.store(enabled, atomic::Ordering::Relaxed);
}

/// Returns whether animations are enabled.
pub fn enabled() -> bool {
    ENABLED.load(atomic::Ordering::Relaxed)
}

/// The default duration of `transition-*` and `animate-in` utilities.
pub const FAST: Duration = Duration::from_millis(150);

/// The duration of dialogs (`duration-200`).
pub const NORMAL: Duration = Duration::from_millis(200);

/// The duration of a sheet sliding out (`duration-300`).
pub const SLOW: Duration = Duration::from_millis(300);

/// The duration of a sheet sliding in (`duration-500`).
pub const SLOWER: Duration = Duration::from_millis(500);

/// An easing function, mapping linear progress to eased progress.
pub type Easing = fn(f32) -> f32;

/// Evaluates a CSS `cubic-bezier(x1, y1, x2, y2)` timing function.
pub fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32, t: f32) -> f32 {
    if t <= 0.0 {
        return 0.0;
    }
    if t >= 1.0 {
        return 1.0;
    }

    let bezier = |a: f32, b: f32, s: f32| {
        let inv = 1.0 - s;
        3.0 * inv * inv * s * a + 3.0 * inv * s * s * b + s * s * s
    };

    // Solve x(s) = t by bisection; precise enough for UI animations.
    let (mut low, mut high) = (0.0_f32, 1.0_f32);
    let mut s = t;

    for _ in 0..24 {
        let x = bezier(x1, x2, s);

        if (x - t).abs() < 1e-4 {
            break;
        }

        if x < t {
            low = s;
        } else {
            high = s;
        }

        s = (low + high) / 2.0;
    }

    bezier(y1, y2, s)
}

/// Linear progress.
pub fn linear(t: f32) -> f32 {
    t.clamp(0.0, 1.0)
}

/// `ease-out`: `cubic-bezier(0, 0, 0.2, 1)`.
pub fn ease_out(t: f32) -> f32 {
    cubic_bezier(0.0, 0.0, 0.2, 1.0, t)
}

/// `ease-in`: `cubic-bezier(0.4, 0, 1, 1)`.
pub fn ease_in(t: f32) -> f32 {
    cubic_bezier(0.4, 0.0, 1.0, 1.0, t)
}

/// `ease-in-out` (Tailwind's default): `cubic-bezier(0.4, 0, 0.2, 1)`.
pub fn ease_in_out(t: f32) -> f32 {
    cubic_bezier(0.4, 0.0, 0.2, 1.0, t)
}

/// A value moving towards a target over time.
#[derive(Debug, Clone, Copy)]
pub struct Transition {
    from: f32,
    to: f32,
    start: Instant,
    duration: Duration,
    easing: Easing,
}

impl Transition {
    /// Creates a settled transition at the given value.
    pub fn new(value: f32) -> Self {
        Self {
            from: value,
            to: value,
            start: Instant::now(),
            duration: Duration::ZERO,
            easing: linear,
        }
    }

    /// Starts moving towards `to`, from wherever the value currently is.
    ///
    /// Settles instantly if animations are disabled.
    pub fn go(&mut self, to: f32, duration: Duration, easing: Easing, now: Instant) {
        if (to - self.to).abs() < f32::EPSILON {
            return;
        }

        if !enabled() || duration.is_zero() {
            *self = Self::new(to);
            return;
        }

        self.from = self.value(now);
        self.to = to;
        self.start = now;
        self.duration = duration;
        self.easing = easing;
    }

    /// Jumps to a value without animating.
    pub fn set(&mut self, value: f32) {
        *self = Self::new(value);
    }

    /// The target of the transition.
    pub fn target(&self) -> f32 {
        self.to
    }

    /// The current value.
    pub fn value(&self, now: Instant) -> f32 {
        if self.duration.is_zero() {
            return self.to;
        }

        let t =
            now.saturating_duration_since(self.start).as_secs_f32() / self.duration.as_secs_f32();

        if t >= 1.0 {
            self.to
        } else {
            self.from + (self.to - self.from) * (self.easing)(t)
        }
    }

    /// Whether the value is still moving.
    pub fn is_animating(&self, now: Instant) -> bool {
        !self.duration.is_zero() && now.saturating_duration_since(self.start) < self.duration
    }
}

impl Default for Transition {
    fn default() -> Self {
        Self::new(0.0)
    }
}

/// A theme that can be drawn translucent, to fade a subtree in or out.
pub trait Fade: Sized {
    /// Returns a copy of the theme whose colors are multiplied by
    /// `opacity`, or `None` if the opacity is (almost) 1.
    fn faded(&self, opacity: f32) -> Option<Self>;
}

impl Fade for iced::Theme {
    fn faded(&self, opacity: f32) -> Option<Self> {
        crate::theme::faded(self, opacity)
    }
}

/// The transformation scaling around `center` by `scale` and then
/// translating by `offset`.
pub fn transform(center: iced::Point, scale: f32, offset: iced::Vector) -> Transformation {
    Transformation::translate(center.x + offset.x, center.y + offset.y)
        * Transformation::scale(scale)
        * Transformation::translate(-center.x, -center.y)
}

/// The visual state of an element entering or leaving the screen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Appearance {
    /// The opacity, from 0 to 1.
    pub opacity: f32,
    /// The scale around the center of the element.
    pub scale: f32,
    /// An offset applied after scaling.
    pub offset: iced::Vector,
}

impl Appearance {
    /// Fully visible, in place.
    pub const VISIBLE: Self = Self {
        opacity: 1.0,
        scale: 1.0,
        offset: iced::Vector::ZERO,
    };

    /// Whether the appearance changes nothing.
    pub fn is_identity(&self) -> bool {
        self.opacity >= 0.999
            && (self.scale - 1.0).abs() < 1e-4
            && self.offset == iced::Vector::ZERO
    }

    /// The transformation of an element with the given bounds.
    pub fn transformation(&self, bounds: iced::Rectangle) -> Option<Transformation> {
        ((self.scale - 1.0).abs() >= 1e-4 || self.offset != iced::Vector::ZERO)
            .then(|| transform(bounds.center(), self.scale, self.offset))
    }
}

/// How far the largest shadow (`shadow-lg`) reaches outside its quad.
const SHADOW_EXTENT: f32 = 32.0;

/// Draws an element with the given [`Appearance`].
#[allow(clippy::too_many_arguments)]
pub fn draw<Message, Theme, Renderer>(
    element: &iced::Element<'_, Message, Theme, Renderer>,
    tree: &iced::advanced::widget::Tree,
    renderer: &mut Renderer,
    theme: &Theme,
    style: &iced::advanced::renderer::Style,
    layout: iced::advanced::Layout<'_>,
    cursor: iced::mouse::Cursor,
    viewport: &iced::Rectangle,
    appearance: Appearance,
) where
    Theme: Fade,
    Renderer: iced::advanced::Renderer,
{
    if appearance.opacity <= 0.001 {
        return;
    }

    let faded = theme.faded(appearance.opacity);
    let theme = faded.as_ref().unwrap_or(theme);
    let style = iced::advanced::renderer::Style {
        text_color: crate::theme::alpha(style.text_color, appearance.opacity.min(1.0)),
    };

    let animating = !appearance.is_identity();
    let bounds = layout.bounds();

    let draw = |renderer: &mut Renderer| {
        // Renderers with damage tracking (tiny-skia) only repaint the bounds
        // of quads that changed, not their shadows, so a moving or fading
        // element would leave its shadow behind. An invisible quad covering
        // the shadows, different on every frame, repaints them.
        if animating {
            renderer.fill_quad(
                iced::advanced::renderer::Quad {
                    bounds: bounds.expand(SHADOW_EXTENT),
                    ..Default::default()
                },
                iced::Color {
                    r: appearance.opacity,
                    g: appearance.scale,
                    b: 0.0,
                    a: 0.0,
                },
            );
        }

        element
            .as_widget()
            .draw(tree, renderer, theme, &style, layout, cursor, viewport);
    };

    match appearance.transformation(layout.bounds()) {
        Some(transformation) => renderer.with_transformation(transformation, draw),
        None => draw(renderer),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn easing_bounds() {
        for easing in [linear, ease_out, ease_in, ease_in_out] {
            assert_eq!(easing(0.0), 0.0);
            assert_eq!(easing(1.0), 1.0);
            assert!((0.0..=1.0).contains(&easing(0.5)));
        }

        // `ease-out` starts fast, `ease-in` starts slow.
        assert!(ease_out(0.25) > 0.25);
        assert!(ease_in(0.25) < 0.25);
    }

    #[test]
    fn transitions_settle() {
        let now = Instant::now();
        let mut transition = Transition::new(0.0);

        transition.go(1.0, Duration::from_millis(100), linear, now);

        assert!(transition.is_animating(now));
        assert!((transition.value(now + Duration::from_millis(50)) - 0.5).abs() < 0.01);
        assert_eq!(transition.value(now + Duration::from_millis(200)), 1.0);
        assert!(!transition.is_animating(now + Duration::from_millis(200)));

        // Reversing mid-way starts from the current value.
        let mut transition = Transition::new(0.0);
        transition.go(1.0, Duration::from_millis(100), linear, now);
        transition.go(
            0.0,
            Duration::from_millis(100),
            linear,
            now + Duration::from_millis(50),
        );
        assert!((transition.value(now + Duration::from_millis(50)) - 0.5).abs() < 0.01);
    }
}
