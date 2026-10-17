//! A styled native-like select, backed by iced's [`PickList`].
//!
//! Port of <https://ui.shadcn.com/docs/components/native-select>. For the
//! fully custom select with groups and separators, see
//! [`select`](super::select).
//!
//! ```no_run
//! # use shadcn_iced::native_select::native_select;
//! # #[derive(Clone)] enum Message { Fruit(&'static str) }
//! # fn view<'a>(fruit: Option<&'static str>) -> iced::Element<'a, Message> {
//! native_select(&["Apple", "Banana", "Blueberry"][..], fruit, Message::Fruit)
//!     .placeholder("Select a fruit")
//!     .into()
//! # }
//! ```
use shadcn_iced_core::icon::lucide;
use shadcn_iced_core::theme::alpha;
use shadcn_iced_core::{font, style, tokens};

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
        // `h-8 pl-2.5 pr-8 py-1`
        .padding([6, 10])
        .font(font::sans())
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

/// Makes a select small (`size="sm"`: `h-7 py-0.5
/// rounded-[min(var(--radius-md),10px)]`).
pub fn small<'a, T, L, V, Message: Clone + 'a>(
    select: PickList<'a, T, L, V, Message>,
) -> PickList<'a, T, L, V, Message>
where
    T: ToString + PartialEq + Clone + 'a,
    L: Borrow<[T]> + 'a,
    V: Borrow<T> + 'a,
{
    select.padding([4, 10]).style(|theme, status| {
        let mut style = self::style(theme, status);
        style.border.radius = border::radius(tokens(theme).radius_md().min(10.0));
        style
    })
}

/// Marks a select as invalid (`aria-invalid:border-destructive`).
pub fn invalid<'a, T, L, V, Message: Clone + 'a>(
    select: PickList<'a, T, L, V, Message>,
) -> PickList<'a, T, L, V, Message>
where
    T: ToString + PartialEq + Clone + 'a,
    L: Borrow<[T]> + 'a,
    V: Borrow<T> + 'a,
{
    select.style(|theme, status| {
        let t = tokens(theme);
        let mut style = self::style(theme, status);
        style.border.color = if t.is_dark() {
            alpha(t.palette.destructive, 0.5)
        } else {
            t.palette.destructive
        };
        style
    })
}

/// The style of the select field: `rounded-lg border border-input
/// bg-transparent dark:bg-input/30 dark:hover:bg-input/50`.
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
        // A native select shows its placeholder option like any other.
        placeholder_color: p.foreground,
        handle_color: p.muted_foreground,
        background: Background::Color(background),
        border: Border {
            color: match status {
                Status::Opened { .. } => p.ring,
                _ => p.input,
            },
            width: 1.0,
            radius: border::radius(t.radius_lg()),
        },
    }
}

/// The style of the dropdown menu of a select.
pub fn menu_style(theme: &Theme) -> menu::Style {
    let t = tokens(theme);
    let p = &t.palette;

    menu::Style {
        background: Background::Color(p.popover),
        border: style::border(alpha(p.foreground, 0.1), t.radius_lg()),
        text_color: p.popover_foreground,
        selected_text_color: p.accent_foreground,
        selected_background: Background::Color(p.accent),
        shadow: style::shadow_md(),
    }
}
