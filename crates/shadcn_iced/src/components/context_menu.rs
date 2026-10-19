//! Displays a menu to the user — such as a set of actions or functions —
//! triggered by a right click.
//!
//! Port of <https://ui.shadcn.com/docs/components/context-menu>.
//!
//! ```no_run
//! # use shadcn_iced::context_menu::context_menu;
//! # use shadcn_iced::menu::{menu, item};
//! # #[derive(Clone)] enum Message { Back }
//! # fn view<'a>() -> iced::Element<'a, Message> {
//! context_menu(
//!     iced::widget::text("Right click here"),
//!     menu().item(item("Back").shortcut("⌘[").on_select(Message::Back)),
//! )
//! .into()
//! # }
//! ```
use super::menu::{Kind, Menu};

use shadcn_iced_core::Element;
use shadcn_iced_core::primitive::Anchored;

/// Shows `menu` at the cursor when `area` is right clicked, or long pressed
/// on a touch screen (`side="right" align="start" alignOffset={4}`: just
/// below the cursor). <kbd>Shift</kbd> + <kbd>F10</kbd> and the context menu
/// key open it at the area while the focus is within it.
pub fn context_menu<'a, Message: Clone + 'a>(
    area: impl Into<Element<'a, Message>>,
    menu: Menu<'a, Message>,
) -> Anchored<'a, Message> {
    Anchored::new(area, menu.kind(Kind::Context))
        .context_menu()
        .offset(4.0)
        // `ring-1` around the menu.
        .outset(1.0)
}
