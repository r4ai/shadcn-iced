//! [shadcn/ui](https://ui.shadcn.com) components for [iced](https://iced.rs).
//!
//! Every component lives in its own file under `src/components/` and only
//! depends on `iced`, [`shadcn_iced_core`] and sibling components (through
//! `super::`). You can either use them from this crate, or copy their source
//! into your project with `cargo shadcn-iced add <component>` and own the code.
//!
//! ```no_run
//! use shadcn_iced::prelude::*;
//!
//! #[derive(Debug, Clone)]
//! enum Message {
//!     Save,
//! }
//!
//! fn view(_: &()) -> iced::Element<'_, Message> {
//!     // `tab_order` moves the focus on Tab (iced leaves it to applications).
//!     tab_order(button("Save").on_press(Message::Save)).into()
//! }
//!
//! fn main() -> iced::Result {
//!     iced::application(|| (), |_: &mut (), _: Message| {}, view)
//!         .theme(|_: &()| shadcn_iced::theme::light().build())
//!         .shadcn_iced()
//!         .run()
//! }
//! ```

// Compiles the examples of the README (`cargo test --doc`).
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;

pub mod components;

pub use components::*;
pub use shadcn_iced_core as core;
pub use shadcn_iced_core::{
    ApplicationExt, Element, Icon, Mode, Palette, Tokens, animation, font, icon, primitive, style,
    theme, tokens,
};

/// Everything you need to build an interface with shadcn_iced.
pub mod prelude {
    pub use crate::components::prelude::*;
    pub use shadcn_iced_core::icon::{Icon, icon, lucide};
    pub use shadcn_iced_core::primitive::tab_order;
    pub use shadcn_iced_core::{ApplicationExt, Mode};
}
