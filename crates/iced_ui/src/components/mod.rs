//! The components of iced_ui, one module per shadcn/ui component.
//!
//! Every module is self-contained: it only uses `iced`, `iced_ui_core` and
//! sibling components through `super::`, so it can be copied into your
//! project as-is.
pub mod alert;
pub mod aspect_ratio;
pub mod avatar;
pub mod badge;
pub mod breadcrumb;
pub mod button;
pub mod card;
pub mod empty;
pub mod input;
pub mod item;
pub mod kbd;
pub mod label;
pub mod pagination;
pub mod progress;
pub mod scroll_area;
pub mod separator;
pub mod skeleton;
pub mod spinner;
pub mod table;
pub mod textarea;
pub mod typography;

/// The constructors of every component.
pub mod prelude {
    pub use super::alert::alert;
    pub use super::aspect_ratio::aspect_ratio;
    pub use super::avatar::{avatar, avatar_group};
    pub use super::badge::badge;
    pub use super::breadcrumb::breadcrumb;
    pub use super::button::{button, icon_button};
    pub use super::card::card;
    pub use super::empty::empty;
    pub use super::input::input;
    pub use super::item::item;
    pub use super::kbd::{kbd, kbd_group, kbd_icon};
    pub use super::label::label;
    pub use super::pagination::pagination;
    pub use super::progress::progress;
    pub use super::scroll_area::scroll_area;
    pub use super::separator::{separator, vertical_separator};
    pub use super::skeleton::skeleton;
    pub use super::spinner::spinner;
    pub use super::table::table;
    pub use super::textarea::textarea;
}
