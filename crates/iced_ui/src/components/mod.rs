//! The components of iced_ui, one module per shadcn/ui component.
//!
//! Every module is self-contained: it only uses `iced`, `iced_ui_core` and
//! sibling components through `super::`, so it can be copied into your
//! project as-is.
pub mod button;

/// The constructors of every component.
pub mod prelude {
    pub use super::button::{button, icon_button};
}
