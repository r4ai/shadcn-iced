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

use shadcn_iced_core::primitive::{Align, Anchored, Side};
use shadcn_iced_core::{Element, style, tokens};

/// Creates a dropdown menu opened by clicking `trigger`.
///
/// The menu opens on a click, a tap, or <kbd>Enter</kbd>, <kbd>Space</kbd>,
/// <kbd>↓</kbd> and <kbd>↑</kbd> while the trigger is focused.
///
/// The menu is placed below the trigger, aligned with its start
/// (`side="bottom" align="start" sideOffset={4}`). The returned
/// [`Anchored`] can be customized further, e.g. with [`Anchored::side`] or
/// [`Anchored::align`]. Once the menu is dismissed with the keyboard, the
/// focus ring of a button (`rounded-lg`) is drawn around the trigger; use
/// [`Anchored::focus_ring`] to change it.
pub fn dropdown_menu<'a, Message: Clone + 'a>(
    trigger: impl Into<Element<'a, Message>>,
    menu: Menu<'a, Message>,
) -> Anchored<'a, Message> {
    let menu = menu.kind(Kind::Dropdown);
    let anchor_width = menu.anchor_min_width();

    let anchored = Anchored::new(trigger, menu)
        .side(Side::Bottom)
        .align(Align::Start)
        .offset(4.0)
        .outset(1.0)
        .close_on_select(true)
        .modal(true)
        // `Enter`, `Space` and `ArrowDown` open it with its first item
        // highlighted, `ArrowUp` with its last one; `Tab` closes it.
        .menu_keys(true)
        // `focus-visible:border-ring focus-visible:ring-3` of a button.
        .focus_ring(|theme| {
            let t = tokens(theme);
            style::focus_visible(&t, t.radius_lg())
        });

    // `w-(--anchor-width) min-w-32`
    match anchor_width {
        Some(min) => anchored.anchor_width(min),
        None => anchored,
    }
}
