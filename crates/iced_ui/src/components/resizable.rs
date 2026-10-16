//! Accessible resizable panel groups and layouts with keyboard support.
//!
//! Port of <https://ui.shadcn.com/docs/components/resizable>. Nest
//! [`resizable`] calls for more than two panels.
//!
//! ```no_run
//! # use iced_ui::resizable::resizable;
//! # fn view<'a>() -> iced::Element<'a, ()> {
//! resizable(iced::widget::text("One"), iced::widget::text("Two")).grip(true).into()
//! # }
//! ```
use iced_ui_core::primitive::Split;
use iced_ui_core::primitive::split::Style;
use iced_ui_core::{Element, tokens};

use iced::Theme;

pub use iced_ui_core::primitive::split::Axis;

/// Creates two resizable panels separated by a draggable handle.
pub fn resizable<'a, Message: 'a>(
    first: impl Into<Element<'a, Message>>,
    second: impl Into<Element<'a, Message>>,
) -> Split<'a, Message> {
    Split::new(first, second, style)
}

/// The style of the resize handle.
pub fn style(theme: &Theme) -> Style {
    let p = tokens(theme).palette;

    Style {
        divider: p.border,
        divider_active: p.ring,
        grip: p.border,
        grip_border: p.border,
    }
}
