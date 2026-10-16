//! A set of two-state buttons that can be toggled on or off.
//!
//! Port of <https://ui.shadcn.com/docs/components/toggle-group>.
//!
//! The group reports which value was pressed; whether several values can be
//! on at once is up to your update logic:
//!
//! ```no_run
//! # use iced_ui::toggle_group::toggle_group;
//! # use iced_ui_core::icon::lucide;
//! # #[derive(Clone, Copy, PartialEq)] enum Format { Bold, Italic }
//! # #[derive(Clone)] enum Message { Toggle(Format) }
//! # fn view<'a>(formats: &[Format]) -> iced::Element<'a, Message> {
//! toggle_group(formats, Message::Toggle)
//!     .icon(Format::Bold, lucide::BOLD)
//!     .icon(Format::Italic, lucide::ITALIC)
//!     .into()
//! # }
//! ```
use super::toggle::{self, Position, Size, Variant};

use iced_ui_core::icon::Icon;
use iced_ui_core::Element;

use iced::widget::Row;
use iced::Length;

/// A toggle group. See the [module documentation](self).
pub struct ToggleGroup<'a, T, Message> {
    selected: Vec<T>,
    items: Vec<(T, Option<Icon>, Option<String>, bool)>,
    on_toggle: Box<dyn Fn(T) -> Message + 'a>,
    variant: Variant,
    size: Size,
    spacing: f32,
    width: Length,
}

/// Creates a [`ToggleGroup`] where `selected` values are on.
pub fn toggle_group<'a, T: Copy + PartialEq + 'a, Message: Clone + 'a>(
    selected: &[T],
    on_toggle: impl Fn(T) -> Message + 'a,
) -> ToggleGroup<'a, T, Message> {
    ToggleGroup {
        selected: selected.to_vec(),
        items: Vec::new(),
        on_toggle: Box::new(on_toggle),
        variant: Variant::Default,
        size: Size::Default,
        spacing: 0.0,
        width: Length::Shrink,
    }
}

impl<'a, T: Copy + PartialEq + 'a, Message: Clone + 'a> ToggleGroup<'a, T, Message> {
    /// Appends an item showing an icon.
    pub fn icon(mut self, value: T, icon: Icon) -> Self {
        self.items.push((value, Some(icon), None, false));
        self
    }

    /// Appends an item showing a label.
    pub fn label(mut self, value: T, label: impl Into<String>) -> Self {
        self.items.push((value, None, Some(label.into()), false));
        self
    }

    /// Appends an item showing an icon and a label.
    pub fn item(mut self, value: T, icon: Option<Icon>, label: Option<String>) -> Self {
        self.items.push((value, icon, label, false));
        self
    }

    /// Appends a disabled item showing an icon.
    pub fn disabled_icon(mut self, value: T, icon: Icon) -> Self {
        self.items.push((value, Some(icon), None, true));
        self
    }

    /// Sets the variant.
    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets the size.
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// Sets the space between items. With `0.0` (the default), items are
    /// joined.
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Sets the width. With [`Length::Fill`], items share the width.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
}

impl<'a, T: Copy + PartialEq + 'a, Message: Clone + 'a> From<ToggleGroup<'a, T, Message>>
    for Element<'a, Message>
{
    fn from(group: ToggleGroup<'a, T, Message>) -> Self {
        let count = group.items.len();
        let joined = group.spacing == 0.0;
        let fill = group.width != Length::Shrink;
        let variant = group.variant;

        let items = group
            .items
            .into_iter()
            .enumerate()
            .map(|(index, (value, icon, label, disabled))| {
                let on = group.selected.contains(&value);
                let message = (group.on_toggle)(value);

                let mut toggle = toggle::toggle(on)
                    .variant(group.variant)
                    .size(group.size)
                    .position(if joined { Position::of(index, count) } else { Position::Alone })
                    .disabled(disabled)
                    .on_toggle(move |_| message.clone());

                if let Some(icon) = icon {
                    toggle = toggle.icon(icon);
                }

                if let Some(label) = label {
                    toggle = toggle.label(label);
                }

                if fill {
                    toggle = toggle.width(Length::Fill);
                }

                Element::from(toggle)
            });

        // Joined outline toggles share their borders, like `border-l-0`.
        let spacing = if joined && variant == Variant::Outline { -1.0 } else { group.spacing };

        Row::with_children(items).spacing(spacing).width(group.width).into()
    }
}
