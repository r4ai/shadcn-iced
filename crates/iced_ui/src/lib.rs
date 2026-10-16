//! [shadcn/ui](https://ui.shadcn.com) components for [iced](https://iced.rs).
//!
//! Every component lives in its own file under `src/components/` and only
//! depends on `iced`, [`iced_ui_core`] and sibling components (through
//! `super::`). You can either use them from this crate, or copy their source
//! into your project with `cargo iced-ui add <component>` and own the code.
//!
//! ```no_run
//! use iced_ui::prelude::*;
//!
//! #[derive(Debug, Clone)]
//! enum Message {
//!     Save,
//! }
//!
//! fn view(_: &()) -> iced::Element<'_, Message> {
//!     button("Save").on_press(Message::Save).into()
//! }
//!
//! fn main() -> iced::Result {
//!     iced::application(|| (), |_: &mut (), _: Message| {}, view)
//!         .theme(|_: &()| iced_ui::theme::light().build())
//!         .iced_ui()
//!         .run()
//! }
//! ```
pub mod components;

pub use components::*;
pub use iced_ui_core as core;
pub use iced_ui_core::{
    ApplicationExt, Element, Icon, Mode, Palette, Tokens, font, icon, primitive,
    style, theme, tokens,
};

/// Everything you need to build an interface with iced_ui.
pub mod prelude {
    pub use crate::components::prelude::*;
    pub use iced_ui_core::icon::{Icon, icon, lucide};
    pub use iced_ui_core::{ApplicationExt, Mode};
}
