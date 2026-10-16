//! Displays a menu to the user, such as a set of actions or functions,
//! triggered by a button.
//!
//! Port of <https://ui.shadcn.com/docs/components/dropdown-menu>. The menu
//! opens when the trigger is clicked and closes on its own; no state is
//! needed in your application.
//!
//! ```no_run
//! # use iced_ui::button::{button, Variant};
//! # use iced_ui::dropdown_menu::dropdown_menu;
//! # use iced_ui::menu::{menu, item};
//! # #[derive(Clone)] enum Message { Profile }
//! # fn view<'a>() -> iced::Element<'a, Message> {
//! dropdown_menu(
//!     button("Open").variant(Variant::Outline),
//!     menu().label("My Account").separator().item(item("Profile").on_select(Message::Profile)),
//! )
//! .into()
//! # }
//! ```
use super::menu::Menu;

use iced_ui_core::Element;
use iced_ui_core::primitive::{Align, Anchored, Side};

/// Creates a dropdown menu opened by clicking `trigger`.
///
/// The returned [`Anchored`] can be customized further, e.g. with
/// [`Anchored::side`] or [`Anchored::align`].
pub fn dropdown_menu<'a, Message: Clone + 'a>(
    trigger: impl Into<Element<'a, Message>>,
    menu: Menu<'a, Message>,
) -> Anchored<'a, Message> {
    Anchored::new(trigger, menu)
        .side(Side::Bottom)
        .align(Align::Start)
        .offset(4.0)
        .close_on_select(true)
        .modal(true)
}
