//! The components of iced_ui, one module per shadcn/ui component.
//!
//! Every module is self-contained: it only uses `iced`, `iced_ui_core` and
//! sibling components through `super::`, so it can be copied into your
//! project as-is.
pub mod accordion;
pub mod alert;
pub mod aspect_ratio;
pub mod avatar;
pub mod badge;
pub mod breadcrumb;
pub mod button;
pub mod button_group;
pub mod card;
pub mod checkbox;
pub mod collapsible;
pub mod empty;
pub mod field;
pub mod input;
pub mod input_group;
pub mod input_otp;
pub mod item;
pub mod kbd;
pub mod label;
pub mod native_select;
pub mod pagination;
pub mod progress;
pub mod radio_group;
pub mod resizable;
pub mod scroll_area;
pub mod separator;
pub mod skeleton;
pub mod slider;
pub mod spinner;
pub mod switch;
pub mod table;
pub mod tabs;
pub mod textarea;
pub mod toggle;
pub mod toggle_group;
pub mod typography;

/// The constructors of every component.
pub mod prelude {
    pub use super::accordion::accordion;
    pub use super::alert::alert;
    pub use super::aspect_ratio::aspect_ratio;
    pub use super::avatar::{avatar, avatar_group};
    pub use super::badge::badge;
    pub use super::breadcrumb::breadcrumb;
    pub use super::button::{button, icon_button};
    pub use super::button_group::button_group;
    pub use super::card::card;
    pub use super::checkbox::checkbox;
    pub use super::collapsible::collapsible;
    pub use super::empty::empty;
    pub use super::field::{field, field_set, field_separator};
    pub use super::input::input;
    pub use super::input_group::input_group;
    pub use super::input_otp::input_otp;
    pub use super::item::item;
    pub use super::kbd::{kbd, kbd_group, kbd_icon};
    pub use super::label::label;
    pub use super::native_select::native_select;
    pub use super::pagination::pagination;
    pub use super::progress::progress;
    pub use super::radio_group::radio_group;
    pub use super::resizable::resizable;
    pub use super::scroll_area::scroll_area;
    pub use super::separator::{separator, vertical_separator};
    pub use super::skeleton::skeleton;
    pub use super::slider::slider;
    pub use super::spinner::spinner;
    pub use super::switch::switch;
    pub use super::tabs::tabs;
    pub use super::table::table;
    pub use super::textarea::textarea;
    pub use super::toggle::toggle;
    pub use super::toggle_group::toggle_group;
}
