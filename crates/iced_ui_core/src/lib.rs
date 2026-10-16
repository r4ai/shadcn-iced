//! The foundation of [`iced_ui`](https://github.com/r4ai/iced-ui): design
//! tokens ported from [shadcn/ui](https://ui.shadcn.com), bundled fonts and
//! icons, and headless primitives (anchored overlays, modal layers, ...).
//!
//! Components built on top of this crate only depend on `iced` and
//! `iced_ui_core`, which is what makes them copy-pasteable.
pub mod animation;
pub mod font;
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
