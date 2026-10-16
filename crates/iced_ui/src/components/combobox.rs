//! Autocomplete input and command palette with a list of suggestions.
//!
//! Port of <https://ui.shadcn.com/docs/components/combobox>: a
//! [`popover`](super::popover) whose content is a [`command`](super::command).
//!
//! ```no_run
//! # use iced_ui::combobox::combobox;
//! # #[derive(Clone)] enum Message { Query(String), Framework(&'static str) }
//! # fn view<'a>(query: &'a str, framework: Option<&'static str>) -> iced::Element<'a, Message> {
//! combobox(framework, Message::Framework, query, Message::Query)
//!     .placeholder("Select framework...")
//!     .option("next", "Next.js")
//!     .option("svelte", "SvelteKit")
//!     .into()
//! # }
//! ```
use super::button::{Button, Variant};
use super::command::{self, item};

use iced_ui_core::icon::lucide;
use iced_ui_core::primitive::{Align, Anchored, Side};
use iced_ui_core::theme::alpha;
use iced_ui_core::{Element, style, tokens};

use iced::widget::{Row, container, text};
use iced::{Alignment, Length, Theme};

/// A combobox. See the [module documentation](self).
pub struct Combobox<'a, T, Message> {
    selected: Option<T>,
    on_select: Box<dyn Fn(T) -> Message + 'a>,
    query: String,
    on_query: Box<dyn Fn(String) -> Message + 'a>,
    options: Vec<(T, String)>,
    placeholder: String,
    search_placeholder: String,
    empty: String,
    width: f32,
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
        placeholder: "Select...".to_owned(),
        search_placeholder: "Search...".to_owned(),
        empty: "No results found.".to_owned(),
        width: 200.0,
    }
}

impl<'a, T: Clone + PartialEq + 'a, Message: Clone + 'a> Combobox<'a, T, Message> {
    /// Appends an option.
    pub fn option(mut self, value: T, label: impl Into<String>) -> Self {
        self.options.push((value, label.into()));
        self
    }

    /// Sets the text of the trigger when nothing is selected.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Sets the placeholder of the search input.
    pub fn search_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.search_placeholder = placeholder.into();
        self
    }

    /// Sets the text shown when nothing matches.
    pub fn empty(mut self, empty: impl Into<String>) -> Self {
        self.empty = empty.into();
        self
    }

    /// Sets the width of the trigger and of the popover.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
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

        let has_value = selected_label.is_some();

        let trigger = Button::new(
            Row::new()
                .push(
                    text(selected_label.unwrap_or(combobox.placeholder))
                        .size(style::text::SM)
                        .width(Length::Fill)
                        .wrapping(text::Wrapping::None)
                        .style(move |theme: &Theme| text::Style {
                            color: (!has_value).then(|| tokens(theme).palette.muted_foreground),
                        }),
                )
                .push(
                    lucide::CHEVRONS_UP_DOWN
                        .sized(16)
                        .style(|theme: &Theme| text::Style {
                            color: Some(alpha(tokens(theme).palette.foreground, 0.5)),
                        }),
                )
                .spacing(8)
                .align_y(Alignment::Center),
        )
        .variant(Variant::Outline)
        .width(combobox.width);

        let on_select = &combobox.on_select;
        let selected = combobox.selected.clone();

        let items = combobox.options.into_iter().map(|(value, label)| {
            let is_selected = selected.as_ref() == Some(&value);
            let mut item = item(label).on_select(on_select(value));

            if is_selected {
                item = item.icon(lucide::CHECK);
            }

            item
        });

        let content = command::command(&combobox.query, combobox.on_query)
            .placeholder(combobox.search_placeholder)
            .empty(combobox.empty)
            .bordered(false)
            .items(items);

        let panel = container(content)
            .width(combobox.width)
            .style(super::popover::style);

        Anchored::new(trigger, panel)
            .side(Side::Bottom)
            .align(Align::Start)
            .offset(4.0)
            .close_on_select(true)
            .into()
    }
}
