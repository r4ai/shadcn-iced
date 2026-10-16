//! Displays a list of options for the user to pick from—triggered by a
//! button.
//!
//! Port of <https://ui.shadcn.com/docs/components/select>.
//!
//! ```no_run
//! # use iced_ui::select::select;
//! # #[derive(Clone)] enum Message { Fruit(&'static str) }
//! # fn view<'a>(fruit: Option<&'static str>) -> iced::Element<'a, Message> {
//! select(fruit, Message::Fruit)
//!     .placeholder("Select a fruit")
//!     .label("Fruits")
//!     .option("apple", "Apple")
//!     .option("banana", "Banana")
//!     .into()
//! # }
//! ```
use super::menu;

use iced_ui_core::icon::lucide;
use iced_ui_core::primitive::{Align, Anchored, Pressable, Side, listbox};
use iced_ui_core::theme::alpha;
use iced_ui_core::{Element, style, tokens};

use iced::widget::button::{Status, Style};
use iced::widget::{Row, container, scrollable, text};
use iced::{Alignment, Background, Border, Color, Length, Padding, Theme, border};

enum Entry<T> {
    Option(T, String, bool),
    Label(String),
    Separator,
}

/// The size of the trigger of a [`Select`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Size {
    /// `h-8`
    Sm,
    /// `h-9`
    #[default]
    Default,
}

/// A select. See the [module documentation](self).
pub struct Select<'a, T, Message> {
    selected: Option<T>,
    entries: Vec<Entry<T>>,
    on_select: Box<dyn Fn(T) -> Message + 'a>,
    placeholder: String,
    width: Length,
    size: Size,
    disabled: bool,
    invalid: bool,
}

/// Creates a [`Select`] with the given selected value.
pub fn select<'a, T: Clone + PartialEq + 'a, Message: Clone + 'a>(
    selected: Option<T>,
    on_select: impl Fn(T) -> Message + 'a,
) -> Select<'a, T, Message> {
    Select {
        selected,
        entries: Vec::new(),
        on_select: Box::new(on_select),
        placeholder: String::new(),
        width: Length::Shrink,
        size: Size::Default,
        disabled: false,
        invalid: false,
    }
}

impl<'a, T: Clone + PartialEq + 'a, Message: Clone + 'a> Select<'a, T, Message> {
    /// Appends an option.
    pub fn option(mut self, value: T, label: impl Into<String>) -> Self {
        self.entries.push(Entry::Option(value, label.into(), false));
        self
    }

    /// Appends every option of an iterator, using their [`ToString`] as label.
    pub fn options(mut self, values: impl IntoIterator<Item = T>) -> Self
    where
        T: ToString,
    {
        for value in values {
            let label = value.to_string();
            self.entries.push(Entry::Option(value, label, false));
        }
        self
    }

    /// Appends a disabled option.
    pub fn disabled_option(mut self, value: T, label: impl Into<String>) -> Self {
        self.entries.push(Entry::Option(value, label.into(), true));
        self
    }

    /// Appends a group label (`SelectLabel`).
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.entries.push(Entry::Label(label.into()));
        self
    }

    /// Appends a separator.
    pub fn separator(mut self) -> Self {
        self.entries.push(Entry::Separator);
        self
    }

    /// Sets the text shown when nothing is selected.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Sets the width of the trigger.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the size of the trigger.
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// Disables the select.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Marks the select as invalid.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }
}

impl<'a, T: Clone + PartialEq + 'a, Message: Clone + 'a> From<Select<'a, T, Message>>
    for Element<'a, Message>
{
    fn from(select: Select<'a, T, Message>) -> Self {
        let selected_label = select.entries.iter().find_map(|entry| match entry {
            Entry::Option(value, label, _) if Some(value) == select.selected.as_ref() => {
                Some(label.clone())
            }
            _ => None,
        });

        let has_value = selected_label.is_some();
        let invalid = select.invalid;

        let trigger = Pressable::new(
            Row::new()
                .push(
                    text(selected_label.unwrap_or(select.placeholder))
                        .size(style::text::SM)
                        .line_height(iced::Pixels(20.0))
                        .wrapping(text::Wrapping::None)
                        .width(if select.width == Length::Shrink {
                            Length::Shrink
                        } else {
                            Length::Fill
                        })
                        .style(move |theme: &Theme| text::Style {
                            color: (!has_value).then(|| tokens(theme).palette.muted_foreground),
                        }),
                )
                .push(
                    lucide::CHEVRON_DOWN
                        .sized(16)
                        .style(|theme: &Theme| text::Style {
                            color: Some(alpha(tokens(theme).palette.muted_foreground, 0.5)),
                        }),
                )
                .spacing(8)
                .align_y(Alignment::Center)
                .height(Length::Fill),
        )
        .width(select.width)
        .height(match select.size {
            Size::Sm => 32,
            Size::Default => 36,
        })
        .padding([0, 12])
        .disabled(select.disabled)
        .style(move |theme, status| trigger(theme, status, invalid));

        let mut list = listbox::Listbox::new(menu::highlight);

        for entry in select.entries {
            list = match entry {
                Entry::Option(value, label, disabled) => {
                    let is_selected = select.selected.as_ref() == Some(&value);

                    let label = text(label)
                        .size(style::text::SM)
                        .line_height(iced::Pixels(20.0))
                        .wrapping(text::Wrapping::None)
                        .style(move |theme: &Theme| text::Style {
                            color: disabled.then(|| {
                                alpha(
                                    tokens(theme).palette.popover_foreground,
                                    style::DISABLED_OPACITY,
                                )
                            }),
                        });

                    let check = container(is_selected.then(|| lucide::CHECK.sized(16)))
                        .width(16)
                        .into();

                    list.item(
                        listbox::row(label, Some(check))
                            .spacing(8.0)
                            .padding(Padding {
                                top: 6.0,
                                bottom: 6.0,
                                left: 8.0,
                                right: 8.0,
                            }),
                        (!disabled).then(|| (select.on_select)(value)),
                    )
                }
                Entry::Label(label) => list.push(
                    container(text(label).size(style::text::XS).style(|theme: &Theme| {
                        text::Style {
                            color: Some(tokens(theme).palette.muted_foreground),
                        }
                    }))
                    .padding([6, 8]),
                ),
                Entry::Separator => {
                    list.push(container(super::separator::separator()).padding([4, 0]))
                }
            };
        }

        let content = container(
            scrollable(list.padding(4))
                .width(Length::Shrink)
                .direction(scrollable::Direction::Vertical(
                    super::scroll_area::scrollbar(),
                ))
                .style(super::scroll_area::style),
        )
        .max_height(300)
        .style(menu::panel);

        Anchored::new(trigger, content)
            .side(Side::Bottom)
            .align(Align::Start)
            .offset(4.0)
            .match_anchor_width(true)
            .close_on_select(true)
            .modal(true)
            .into()
    }
}

/// The style of the trigger of a select.
pub fn trigger(theme: &Theme, status: Status, invalid: bool) -> Style {
    let t = tokens(theme);
    let p = &t.palette;
    let dark = t.is_dark();
    let hovered = matches!(status, Status::Hovered | Status::Pressed);
    let opacity = if status == Status::Disabled {
        style::DISABLED_OPACITY
    } else {
        1.0
    };

    Style {
        background: Some(Background::Color(match (dark, hovered) {
            (false, _) => Color::TRANSPARENT,
            (true, false) => alpha(p.input, 0.3),
            (true, true) => alpha(p.input, 0.5),
        })),
        text_color: alpha(p.foreground, opacity),
        border: Border {
            color: alpha(if invalid { p.destructive } else { p.input }, opacity),
            width: 1.0,
            radius: border::radius(t.radius_md()),
        },
        ..Style::default()
    }
}
