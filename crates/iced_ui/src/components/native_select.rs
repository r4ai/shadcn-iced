//! A styled native-like select, backed by iced's [`PickList`].
//!
//! Port of <https://ui.shadcn.com/docs/components/native-select>. For the
//! fully custom select with groups and separators, see
//! [`select`](super::select).
//!
//! ```no_run
//! # use iced_ui::native_select::native_select;
//! # #[derive(Clone)] enum Message { Fruit(&'static str) }
//! # fn view<'a>(fruit: Option<&'static str>) -> iced::Element<'a, Message> {
//! native_select(&["Apple", "Banana", "Blueberry"][..], fruit, Message::Fruit)
//!     .placeholder("Select a fruit")
//!     .into()
//! # }
//! ```
use iced_ui_core::icon::lucide;
use iced_ui_core::theme::alpha;
use iced_ui_core::{font, style, tokens};

use iced::overlay::menu;
use iced::widget::pick_list::{Handle, Icon, PickList, Status, Style};
use iced::widget::text;
use iced::{Background, Border, Color, Pixels, Theme, border};

use std::borrow::Borrow;

/// Creates a select from a list of options.
pub fn native_select<'a, T, L, V, Message: Clone + 'a>(
    options: L,
    selected: Option<V>,
    on_select: impl Fn(T) -> Message + 'a,
) -> PickList<'a, T, L, V, Message>
where
    T: ToString + PartialEq + Clone + 'a,
    L: Borrow<[T]> + 'a,
    V: Borrow<T> + 'a,
{
    PickList::new(options, selected, on_select)
        .text_size(style::text::SM)
        .text_line_height(text::LineHeight::Absolute(Pixels(20.0)))
        .padding([7, 12])
        .font(font::SANS)
        .handle(Handle::Static(Icon {
            font: font::ICONS,
            code_point: lucide::CHEVRON_DOWN.codepoint(),
            size: Some(Pixels(16.0)),
            line_height: text::LineHeight::Relative(1.0),
            shaping: text::Shaping::Basic,
        }))
        .style(self::style)
        .menu_style(menu_style)
}

/// The style of the select field.
pub fn style(theme: &Theme, status: Status) -> Style {
    let t = tokens(theme);
    let p = &t.palette;
    let dark = t.is_dark();

    let background = match (dark, status) {
        (false, _) => Color::TRANSPARENT,
        (true, Status::Hovered | Status::Opened { .. }) => alpha(p.input, 0.5),
        (true, _) => alpha(p.input, 0.3),
    };

    Style {
        text_color: p.foreground,
        placeholder_color: p.muted_foreground,
        handle_color: alpha(p.muted_foreground, 0.5),
        background: Background::Color(background),
        border: Border {
            color: match status {
                Status::Opened { .. } => p.ring,
                _ => p.input,
            },
            width: 1.0,
            radius: border::radius(t.radius_md()),
        },
    }
}

/// The style of the dropdown menu of a select.
pub fn menu_style(theme: &Theme) -> menu::Style {
    let t = tokens(theme);
    let p = &t.palette;

    menu::Style {
        background: Background::Color(p.popover),
        border: style::border(p.border, t.radius_md()),
        text_color: p.popover_foreground,
        selected_text_color: p.accent_foreground,
        selected_background: Background::Color(p.accent),
        shadow: style::shadow_md(),
    }
}

