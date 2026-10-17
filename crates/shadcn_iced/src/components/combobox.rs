//! Autocomplete input with a list of suggestions.
//!
//! Port of <https://ui.shadcn.com/docs/components/combobox> (Base UI
//! combobox): an input with a trigger button, opening a list of the items
//! matching what has been typed. The query lives in your state.
//!
//! ```no_run
//! # use shadcn_iced::combobox::combobox;
//! # #[derive(Clone)] enum Message { Query(String), Framework(&'static str) }
//! # fn view<'a>(query: &'a str, framework: Option<&'static str>) -> iced::Element<'a, Message> {
//! combobox(framework, Message::Framework, query, Message::Query)
//!     .placeholder("Select a framework")
//!     .option("next", "Next.js")
//!     .option("svelte", "SvelteKit")
//!     .into()
//! # }
//! ```
use super::button::{Size, Variant, icon_button};
use super::menu::{self, menu_tokens};

use shadcn_iced_core::icon::lucide;
use shadcn_iced_core::primitive::anchored::toggle;
use shadcn_iced_core::primitive::focus::{self, FocusWithin};
use shadcn_iced_core::primitive::{Align, Anchored, Side, listbox};
use shadcn_iced_core::theme::alpha;
use shadcn_iced_core::{Element, style, tokens};

use iced::widget::{Row, container, scrollable, text};
use iced::{Alignment, Length, Padding, Theme};

/// A combobox. See the [module documentation](self).
pub struct Combobox<'a, T, Message> {
    selected: Option<T>,
    on_select: Box<dyn Fn(T) -> Message + 'a>,
    query: String,
    on_query: Box<dyn Fn(String) -> Message + 'a>,
    options: Vec<(T, String)>,
    placeholder: String,
    empty: String,
    width: f32,
    disabled: bool,
    invalid: bool,
}

/// Creates a [`Combobox`]. The search query is kept in your state.
pub fn combobox<'a, T: Clone + PartialEq + 'a, Message: Clone + 'a>(
    selected: Option<T>,
    on_select: impl Fn(T) -> Message + 'a,
    query: &str,
    on_query: impl Fn(String) -> Message + 'a,
) -> Combobox<'a, T, Message> {
    Combobox {
        selected,
        on_select: Box::new(on_select),
        query: query.to_owned(),
        on_query: Box::new(on_query),
        options: Vec::new(),
        placeholder: String::new(),
        empty: "No items found.".to_owned(),
        width: 200.0,
        disabled: false,
        invalid: false,
    }
}

impl<'a, T: Clone + PartialEq + 'a, Message: Clone + 'a> Combobox<'a, T, Message> {
    /// Appends an option.
    pub fn option(mut self, value: T, label: impl Into<String>) -> Self {
        self.options.push((value, label.into()));
        self
    }

    /// Sets the placeholder of the input.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Sets the placeholder of the input. The Base UI combobox has no
    /// separate search field, so this is the same as [`Self::placeholder`].
    pub fn search_placeholder(self, placeholder: impl Into<String>) -> Self {
        self.placeholder(placeholder)
    }

    /// Sets the text shown when nothing matches (`ComboboxEmpty`).
    pub fn empty(mut self, empty: impl Into<String>) -> Self {
        self.empty = empty.into();
        self
    }

    /// Sets the width of the input. The list is at least as wide.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Disables the combobox.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Marks the combobox as invalid (`aria-invalid`).
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }
}

impl<'a, T: Clone + PartialEq + 'a, Message: Clone + 'a> From<Combobox<'a, T, Message>>
    for Element<'a, Message>
{
    fn from(combobox: Combobox<'a, T, Message>) -> Self {
        let selected_label = combobox
            .options
            .iter()
            .find(|(value, _)| Some(value) == combobox.selected.as_ref())
            .map(|(_, label)| label.clone());

        // The input shows the query while typing, the selection otherwise.
        let value = if combobox.query.is_empty() {
            selected_label.clone().unwrap_or_default()
        } else {
            combobox.query.clone()
        };

        let filter = (!combobox.query.is_empty()
            && Some(&combobox.query) != selected_label.as_ref())
        .then(|| combobox.query.trim().to_lowercase());

        let disabled = combobox.disabled;
        let invalid = combobox.invalid;

        let mut input = super::input::input(&combobox.placeholder, &value)
            .chromeless(true)
            // `px-2.5`, `pr-1.5` before the addon; less the 1px border.
            .padding(Padding {
                top: 5.0,
                right: 6.0,
                bottom: 5.0,
                left: 9.0,
            });

        if !disabled {
            input = input.on_input(combobox.on_query);
        }

        // `InputGroupButton size="icon-xs" variant="ghost"` at `pr-2
        // mr-[-0.3rem]`.
        let trigger = icon_button(lucide::CHEVRON_DOWN)
            .variant(Variant::Ghost)
            .size(Size::IconXs)
            .disabled(disabled);

        // `InputGroup`: `h-8 rounded-lg border border-input dark:bg-input/30`.
        // `InputGroup`: `h-8 rounded-lg border border-input dark:bg-input/30`,
        // with `border-ring ring-3 ring-ring/50` while the input is focused.
        let field = FocusWithin::new(
            Row::new()
                .push(input)
                // Only the trigger opens and closes the list on click.
                .push(container(toggle(trigger)).padding(Padding::ZERO.right(3)))
                .align_y(Alignment::Center),
            move |theme: &Theme, status| focus::Style {
                text_color: disabled
                    .then(|| alpha(tokens(theme).palette.foreground, style::DISABLED_OPACITY)),
                ..super::input_group::style(theme, status, invalid, disabled)
            },
        )
        .padding(1)
        .width(combobox.width)
        .height(32);

        let on_select = &combobox.on_select;
        let selected = combobox.selected.clone();

        let mut list = listbox::Listbox::new(menu::highlight).highlight_first(filter.is_some());
        let mut count = 0;

        for (value, label) in combobox.options {
            if let Some(filter) = &filter
                && !label.to_lowercase().contains(filter)
            {
                continue;
            }

            count += 1;

            let is_selected = selected.as_ref() == Some(&value);

            // `absolute right-2 size-4`
            let check = container(is_selected.then(|| lucide::CHECK.sized(16)))
                .width(16)
                .into();

            // `gap-2 rounded-md py-1 pr-8 pl-1.5 text-sm`
            list = list.item(
                listbox::row(
                    text(label)
                        .size(style::text::SM)
                        .line_height(iced::Pixels(20.0))
                        .wrapping(text::Wrapping::None),
                    Some(check),
                )
                .spacing(8.0)
                .padding(menu::ITEM_PADDING.right(8)),
                Some(on_select(value)),
            );
        }

        let body: Element<'a, Message> = if count == 0 {
            // `py-2 text-center text-sm text-muted-foreground`
            container(
                text(combobox.empty)
                    .size(style::text::SM)
                    .line_height(iced::Pixels(20.0))
                    .style(|theme: &Theme| text::Style {
                        color: Some(menu_tokens(theme).palette.muted_foreground),
                    }),
            )
            .padding([8, 0])
            .center_x(Length::Fill)
            .into()
        } else {
            // `max-h-72 p-1`, no scrollbar.
            scrollable(list.padding(4).width(Length::Fill))
                .direction(scrollable::Direction::Vertical(
                    scrollable::Scrollbar::new().width(0).scroller_width(0),
                ))
                .into()
        };

        // `min-w-[calc(var(--anchor-width)+--spacing(7))] rounded-lg
        // shadow-md ring-1`, where the anchor is the input: the group less
        // its border and the addon (`pr-2 mr-[-0.3rem]` and a `size-6`
        // button).
        let anchor_width = combobox.width - 2.0 - (8.0 - 4.8 + 24.0);
        let popup = container(body)
            .padding(1)
            .width(anchor_width + 28.0 + 2.0)
            .max_height(288)
            .style(menu::panel);

        if disabled {
            return field.into();
        }

        // `side="bottom" align="start" sideOffset={6}`; the chevron, typing
        // or `ArrowDown` open the list, not a click on the input.
        Anchored::new(field, popup)
            .combobox()
            .side(Side::Bottom)
            .align(Align::Start)
            .offset(6.0)
            .close_on_select(true)
            .into()
    }
}
