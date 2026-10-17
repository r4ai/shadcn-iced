//! Shared style values mirroring Tailwind CSS defaults, as used by shadcn/ui.
use crate::theme::{Tokens, alpha};

use iced::{Border, Color, Shadow, Vector, border};

/// Where an item sits in a joined group (button group, toggle group...),
/// which decides its corners.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Position {
    /// A standalone item.
    #[default]
    Alone,
    /// The first item of a group.
    First,
    /// An item in the middle of a group.
    Middle,
    /// The last item of a group.
    Last,
}

impl Position {
    /// Computes the position of the item at `index` in a group of `count`.
    pub fn of(index: usize, count: usize) -> Self {
        match (index, count) {
            (_, 0 | 1) => Position::Alone,
            (0, _) => Position::First,
            (i, n) if i + 1 == n => Position::Last,
            _ => Position::Middle,
        }
    }

    /// The corner radius of a horizontal group item for the given radius.
    pub fn radius(self, radius: f32) -> border::Radius {
        match self {
            Position::Alone => border::radius(radius),
            Position::First => border::radius(0).top_left(radius).bottom_left(radius),
            Position::Middle => border::radius(0),
            Position::Last => border::radius(0).top_right(radius).bottom_right(radius),
        }
    }

    /// The corner radius of a vertical group item for the given radius.
    pub fn vertical_radius(self, radius: f32) -> border::Radius {
        match self {
            Position::Alone => border::radius(radius),
            Position::First => border::radius(0).top_left(radius).top_right(radius),
            Position::Middle => border::radius(0),
            Position::Last => border::radius(0).bottom_left(radius).bottom_right(radius),
        }
    }
}

/// Font sizes of Tailwind CSS, in logical pixels.
pub mod text {
    /// `text-xs`
    pub const XS: f32 = 12.0;
    /// `text-sm`
    pub const SM: f32 = 14.0;
    /// `text-base`
    pub const BASE: f32 = 16.0;
    /// `text-lg`
    pub const LG: f32 = 18.0;
    /// `text-xl`
    pub const XL: f32 = 20.0;
    /// `text-2xl`
    pub const XL2: f32 = 24.0;
    /// `text-3xl`
    pub const XL3: f32 = 30.0;
    /// `text-4xl`
    pub const XL4: f32 = 36.0;
}

/// Tailwind's spacing scale: `spacing(n)` is `n * 0.25rem`.
pub const fn spacing(n: f32) -> f32 {
    n * 4.0
}

/// `shadow-xs`
pub fn shadow_xs() -> Shadow {
    Shadow {
        color: Color::from_rgba(0.0, 0.0, 0.0, 0.05),
        offset: Vector::new(0.0, 1.0),
        blur_radius: 2.0,
    }
}

/// `shadow-sm`
pub fn shadow_sm() -> Shadow {
    Shadow {
        color: Color::from_rgba(0.0, 0.0, 0.0, 0.1),
        offset: Vector::new(0.0, 1.0),
        blur_radius: 3.0,
    }
}

/// `shadow-md`
pub fn shadow_md() -> Shadow {
    Shadow {
        color: Color::from_rgba(0.0, 0.0, 0.0, 0.1),
        offset: Vector::new(0.0, 4.0),
        blur_radius: 6.0,
    }
}

/// `shadow-lg`
pub fn shadow_lg() -> Shadow {
    Shadow {
        color: Color::from_rgba(0.0, 0.0, 0.0, 0.1),
        offset: Vector::new(0.0, 10.0),
        blur_radius: 15.0,
    }
}

/// The focus ring of shadcn/ui (`focus-visible:ring-[3px] ring-ring/50`),
/// approximated with a soft shadow since iced has no outline.
pub fn focus_ring(tokens: &Tokens) -> Shadow {
    Shadow {
        color: alpha(tokens.palette.ring, 0.5),
        offset: Vector::ZERO,
        blur_radius: 6.0,
    }
}

/// The focus ring as an outset [`Ring`](crate::primitive::focus::Ring)
/// (`ring-[3px] ring-ring/50`) around a control of the given radius.
pub fn ring(tokens: &Tokens, radius: f32) -> crate::primitive::focus::Ring {
    crate::primitive::focus::Ring {
        color: alpha(tokens.palette.ring, 0.5),
        width: 3.0,
        radius,
    }
}

/// The ring of an invalid control as an outset ring
/// (`ring-destructive/20`, `/40` in dark mode).
pub fn invalid_outset_ring(tokens: &Tokens, radius: f32) -> crate::primitive::focus::Ring {
    crate::primitive::focus::Ring {
        color: alpha(
            tokens.palette.destructive,
            if tokens.is_dark() { 0.4 } else { 0.2 },
        ),
        width: 3.0,
        radius,
    }
}

/// The ring of an invalid control (`aria-invalid:ring-destructive/20`).
pub fn invalid_ring(tokens: &Tokens) -> Shadow {
    Shadow {
        color: alpha(
            tokens.palette.destructive,
            if tokens.is_dark() { 0.4 } else { 0.2 },
        ),
        offset: Vector::ZERO,
        blur_radius: 6.0,
    }
}

/// A 1px border of the given color and radius.
pub fn border(color: Color, radius: f32) -> Border {
    Border {
        color,
        width: 1.0,
        radius: border::radius(radius),
    }
}

/// A rounded border without stroke.
pub fn rounded(radius: f32) -> Border {
    Border {
        color: Color::TRANSPARENT,
        width: 0.0,
        radius: border::radius(radius),
    }
}

/// `opacity-50`, applied to disabled controls.
pub const DISABLED_OPACITY: f32 = 0.5;

/// `ring-1 ring-foreground/10`, the hairline outline of Nova's cards,
/// dialogs and popovers, as a 1px border of the given radius.
pub fn ring_border(tokens: &Tokens, radius: f32) -> Border {
    border(alpha(tokens.palette.foreground, 0.1), radius)
}
