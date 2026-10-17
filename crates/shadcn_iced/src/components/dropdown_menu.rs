//! Displays a menu to the user, such as a set of actions or functions,
//! triggered by a button.
//!
//! Port of <https://ui.shadcn.com/docs/components/dropdown-menu>. The menu
//! opens when the trigger is clicked and closes on its own; no state is
//! needed in your application.
//!
//! ```no_run
//! # use shadcn_iced::button::{button, Variant};
//! # use shadcn_iced::dropdown_menu::dropdown_menu;
//! # use shadcn_iced::menu::{menu, item};
//! # #[derive(Clone)] enum Message { Profile }
//! # fn view<'a>() -> iced::Element<'a, Message> {
//! dropdown_menu(
//!     button("Open").variant(Variant::Outline),
//!     menu().label("My Account").separator().item(item("Profile").on_select(Message::Profile)),
//! )
//! .into()
//! # }
//! ```
use super::menu::{Kind, Menu};

use shadcn_iced_core::Element;
use shadcn_iced_core::primitive::{Align, Anchored, Side};

/// Creates a dropdown menu opened by clicking `trigger`.
///
/// The menu is placed below the trigger, aligned with its start
/// (`side="bottom" align="start" sideOffset={4}`). The returned
/// [`Anchored`] can be customized further, e.g. with [`Anchored::side`] or
/// [`Anchored::align`].
pub fn dropdown_menu<'a, Message: Clone + 'a>(
    trigger: impl Into<Element<'a, Message>>,
    menu: Menu<'a, Message>,
) -> Anchored<'a, Message> {
    Anchored::new(trigger, menu.kind(Kind::Dropdown))
        .side(Side::Bottom)
        .align(Align::Start)
        .offset(4.0)
        .close_on_select(true)
        .modal(true)
}
