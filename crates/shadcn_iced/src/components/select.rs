//! Displays a list of options for the user to pick from—triggered by a
//! button.
//!
//! Port of <https://ui.shadcn.com/docs/components/select>.
//!
//! ```no_run
//! # use shadcn_iced::select::select;
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
use super::menu::{self, menu_tokens};

use shadcn_iced_core::icon::lucide;
use shadcn_iced_core::primitive::{Align, Anchored, Pressable, Side, listbox};
use shadcn_iced_core::theme::alpha;
use shadcn_iced_core::{Element, style, tokens};

use iced::widget::button::{Status, Style};
use iced::widget::{Row, container, scrollable, text};
use iced::{Alignment, Background, Border, Color, Length, Theme, border};

enum Entry<T> {
    Option(T, String, bool),
    Label(String),
    Separator,
}

impl<T> Entry<T> {
    /// The height of the entry in the list.
    fn height(&self) -> f32 {
        match self {
            // `py-1` + `text-sm`
            Entry::Option(..) => 28.0,
            // `py-1` + `text-xs`
            Entry::Label(_) => 24.0,
            // `my-1 h-px` between two `p-1` groups.
            Entry::Separator => 17.0,
        }
    }
}

/// The size of the trigger of a [`Select`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Size {
    /// `h-7`
    Sm,
    /// `h-8`
    #[default]
    Default,
}

impl Size {
    fn height(self) -> f32 {
        match self {
            Size::Sm => 28.0,
            Size::Default => 32.0,
        }
    }
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
    align_item_with_trigger: bool,
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
        align_item_with_trigger: true,
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

    /// Sets whether the list opens over the trigger, with the selected item
    /// (or the first one) on top of the trigger (`alignItemWithTrigger`,
    /// the default). Otherwise it opens below the trigger.
    pub fn align_item_with_trigger(mut self, align: bool) -> Self {
        self.align_item_with_trigger = align;
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
        let size = select.size;
        // `disabled:opacity-50`
        let opacity = if select.disabled {
            style::DISABLED_OPACITY
        } else {
            1.0
        };

        // `gap-1.5 py-2 pr-2 pl-2.5 text-sm`
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
                            color: (!has_value)
                                .then(|| alpha(tokens(theme).palette.muted_foreground, opacity)),
                        }),
                )
                .push(
                    lucide::CHEVRON_DOWN
                        .sized(16)
                        .style(move |theme: &Theme| text::Style {
                            color: Some(alpha(tokens(theme).palette.muted_foreground, opacity)),
                        }),
                )
                .spacing(6)
                .align_y(Alignment::Center)
                .height(Length::Fill),
        )
        .width(select.width)
        .height(size.height())
        // `pr-2 pl-2.5` inside the 1px border.
        .padding(iced::Padding {
            top: 0.0,
            right: 9.0,
            bottom: 0.0,
            left: 11.0,
        })
        .disabled(select.disabled)
        .style(move |theme, status| trigger_style(theme, status, invalid, size));

        // Where the selected (or first) option sits in the list, to open
        // the list with it on top of the trigger.
        // (`p-1`, inside the ring.)
        let mut item_top = 4.0;
        let mut aligned = None;
        let mut first = None;

        for entry in &select.entries {
            if let Entry::Option(value, ..) = entry {
                first.get_or_insert(item_top);

                if select.selected.as_ref() == Some(value) {
                    aligned = Some(item_top);
                }
            }

            item_top += entry.height();
        }

        let aligned_top = aligned.or(first).unwrap_or(0.0);

        // The selected option is highlighted when the list opens.
        let selected_index = select.entries.iter().position(|entry| {
            matches!(entry, Entry::Option(value, ..) if select.selected.as_ref() == Some(value))
        });

        let mut list = listbox::Listbox::new(menu::highlight).highlighted(selected_index);

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
                                    menu_tokens(theme).palette.popover_foreground,
                                    style::DISABLED_OPACITY,
                                )
                            }),
                        });

                    // `absolute right-2 size-4`
                    let check = container(is_selected.then(|| lucide::CHECK.sized(16)))
                        .width(16)
                        .into();

                    // `gap-1.5 py-1 pr-8 pl-1.5`
                    list.item(
                        listbox::row(label, Some(check))
                            .spacing(8.0)
                            .padding(menu::ITEM_PADDING.right(8)),
                        (!disabled).then(|| (select.on_select)(value)),
                    )
                }
                Entry::Label(label) => list.push(
                    // `px-1.5 py-1 text-xs text-muted-foreground`
                    container(
                        text(label)
                            .size(style::text::XS)
                            .line_height(iced::Pixels(16.0))
                            .style(|theme: &Theme| text::Style {
                                color: Some(menu_tokens(theme).palette.muted_foreground),
                            }),
                    )
                    .padding(menu::ITEM_PADDING),
                ),
                // A separator ends a `SelectGroup` (`p-1`) and starts another.
                Entry::Separator => list.push(container(menu::separator()).padding([4, 0])),
            };
        }

        // `w-(--anchor-width) min-w-36 rounded-lg shadow-md ring-1`, `p-1`
        // groups.
        let content = menu::shadow(
            container(
                scrollable(list.padding(4).width(Length::Fill))
                    .width(Length::Fill)
                    .direction(scrollable::Direction::Vertical(
                        super::scroll_area::scrollbar(),
                    ))
                    .style(super::scroll_area::style),
            )
            .padding(1)
            .max_height(600)
            .style(menu::panel),
            false,
        );

        let anchored = Anchored::new(trigger, content)
            .align(Align::Start)
            .anchor_width(144.0)
            // `ring-1` around the list.
            .outset(1.0)
            .close_on_select(true)
            .modal(true)
            // `focus-visible:border-ring focus-visible:ring-3`, once
            // dismissed with the keyboard.
            .focus_ring(move |theme| {
                let t = tokens(theme);
                style::focus_visible(
                    &t,
                    match size {
                        Size::Default => t.radius_lg(),
                        Size::Sm => t.radius_md().min(10.0),
                    },
                )
            });

        if select.align_item_with_trigger {
            // The item centered on the trigger, its text aligned with the
            // value, or below the trigger when there is no room for it.
            anchored
                .side(Side::Bottom)
                .offset(-(size.height() + 28.0) / 2.0 - aligned_top)
                .fallback_offset(4.0)
                .animate(false)
                .into()
        } else {
            anchored.side(Side::Bottom).offset(4.0).into()
        }
    }
}

/// The style of the trigger of a select (`rounded-lg border border-input
/// bg-transparent dark:bg-input/30 dark:hover:bg-input/50`).
pub fn trigger(theme: &Theme, status: Status, invalid: bool) -> Style {
    trigger_style(theme, status, invalid, Size::Default)
}

fn trigger_style(theme: &Theme, status: Status, invalid: bool, size: Size) -> Style {
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
            (true, false) => alpha(p.input, 0.3 * opacity),
            (true, true) => alpha(p.input, 0.5 * opacity),
        })),
        text_color: alpha(p.foreground, opacity),
        border: Border {
            color: alpha(
                if invalid {
                    if dark {
                        alpha(p.destructive, 0.5)
                    } else {
                        p.destructive
                    }
                } else {
                    p.input
                },
                opacity,
            ),
            width: 1.0,
            radius: border::radius(match size {
                Size::Default => t.radius_lg(),
                // `rounded-[min(var(--radius-md),10px)]`
                Size::Sm => t.radius_md().min(10.0),
            }),
        },
        ..Style::default()
    }
}
