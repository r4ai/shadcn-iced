//! The foundation of [`shadcn_iced`](https://github.com/r4ai/shadcn-iced): design
//! tokens ported from [shadcn/ui](https://ui.shadcn.com), bundled fonts and
//! icons, and headless primitives (anchored overlays, modal layers, ...).
//!
//! Components built on top of this crate only depend on `iced` and
//! `shadcn_iced_core`, which is what makes them copy-pasteable.

// Compiles the examples of the README (`cargo test --doc`).
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;

pub mod animation;
pub mod font;
pub mod highlight;
pub mod icon;
pub mod primitive;
pub mod style;
pub mod theme;

pub use font::ApplicationExt;
pub use icon::{Icon, icon};
pub use theme::{Mode, Palette, Tokens, tokens};

/// A convenient alias for an [`iced::Element`] using the default theme and
/// renderer.
pub type Element<'a, Message> = iced::Element<'a, Message>;
