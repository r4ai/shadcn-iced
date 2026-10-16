//! Fast, composable command menu.
//!
//! Port of <https://ui.shadcn.com/docs/components/command>. The query lives
//! in your state; the command filters its items and supports keyboard
//! navigation (<kbd>↑</kbd>, <kbd>↓</kbd>, <kbd>Enter</kbd>).
//!
//! ```no_run
//! # use iced_ui::command::{command, item};
//! # use iced_ui_core::icon::lucide;
//! # #[derive(Clone)] enum Message { Query(String), Calendar }
//! # fn view(query: &str) -> iced::Element<'_, Message> {
//! command(query, Message::Query)
//!     .group("Suggestions", [item("Calendar").icon(lucide::CALENDAR).on_select(Message::Calendar)])
//!     .into()
//! # }
//! ```
use super::menu;

use iced_ui_core::icon::{Icon, lucide};
use iced_ui_core::primitive::listbox;
use iced_ui_core::theme::alpha;
use iced_ui_core::{Element, font, style, tokens};

use iced::widget::{Column, Row, container, scrollable, text};
use iced::{Alignment, Background, Length, Padding, Theme};

/// An item of a [`Command`].
pub struct Item<Message> {
    label: String,
    icon: Option<Icon>,
    shortcut: Option<String>,
    keywords: Vec<String>,
    on_select: Option<Message>,
}

/// Creates a command [`Item`].
pub fn item<Message>(label: impl Into<String>) -> Item<Message> {
    Item {
        label: label.into(),
        icon: None,
        shortcut: None,
        keywords: Vec::new(),
        on_select: None,
    }
}

impl<Message> Item<Message> {
    /// Sets the icon.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Sets the shortcut hint.
    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Adds keywords that also match the query.
    pub fn keywords(mut self, keywords: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.keywords.extend(keywords.into_iter().map(Into::into));
        self
    }

    /// Sets the message produced when the item is selected. Items without
    /// one are disabled.
    pub fn on_select(mut self, message: Message) -> Self {
        self.on_select = Some(message);
        self
    }

    /// Whether the item matches the query (case-insensitive substring).
    pub fn matches(&self, query: &str) -> bool {
        let query = query.trim().to_lowercase();

        query.is_empty()
            || self.label.to_lowercase().contains(&query)
            || self.keywords.iter().any(|keyword| keyword.to_lowercase().contains(&query))
    }
}

/// A command menu. See the [module documentation](self).
pub struct Command<'a, Message> {
    query: String,
    on_query: Box<dyn Fn(String) -> Message + 'a>,
    placeholder: String,
    groups: Vec<(Option<String>, Vec<Item<Message>>)>,
    empty: String,
    bordered: bool,
    width: Length,
    input_height: f32,
}

/// Creates a [`Command`] with the current query.
pub fn command<'a, Message: Clone + 'a>(
    query: &str,
    on_query: impl Fn(String) -> Message + 'a,
) -> Command<'a, Message> {
    Command {
        query: query.to_owned(),
        on_query: Box::new(on_query),
        placeholder: "Type a command or search...".to_owned(),
        groups: Vec::new(),
        empty: "No results found.".to_owned(),
        bordered: true,
        width: Length::Fill,
        input_height: 36.0,
    }
}

impl<'a, Message: Clone + 'a> Command<'a, Message> {
    /// Appends a group of items with a heading.
    pub fn group(mut self, heading: impl Into<String>, items: impl IntoIterator<Item = Item<Message>>) -> Self {
        self.groups.push((Some(heading.into()), items.into_iter().collect()));
        self
    }

    /// Appends items without heading.
    pub fn items(mut self, items: impl IntoIterator<Item = Item<Message>>) -> Self {
        self.groups.push((None, items.into_iter().collect()));
        self
    }

    /// Sets the placeholder of the search input.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Sets the text shown when nothing matches.
    pub fn empty(mut self, empty: impl Into<String>) -> Self {
        self.empty = empty.into();
        self
    }

    /// Draws a border and a shadow around the command (`rounded-lg border
    /// shadow-md`). Defaults to `true`.
    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }

    /// Sets the width.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the search row (`h-12` in a command dialog).
    pub fn input_height(mut self, height: f32) -> Self {
        self.input_height = height;
        self
    }
}

impl<'a, Message: Clone + 'a> From<Command<'a, Message>> for Element<'a, Message> {
    fn from(command: Command<'a, Message>) -> Self {
        let query = command.query.clone();

        let input = Row::new()
            .push(lucide::SEARCH.sized(16).style(|theme: &Theme| text::Style {
                color: Some(alpha(tokens(theme).palette.foreground, 0.5)),
            }))
            .push(
                super::input::input(&command.placeholder, &command.query)
                    .on_input(command.on_query)
                    .chromeless(true)
                    .padding(Padding::ZERO.top(10).bottom(10)),
            )
            .spacing(8)
            .padding([0, 12])
            .height(command.input_height)
            .align_y(Alignment::Center);

        let mut list = listbox::Listbox::new(menu::highlight)
            .width(Length::Fill)
            .highlight_first(true)
            .clear_on_leave(false);

        let mut visible_groups = 0;

        for (heading, items) in command.groups {
            let items: Vec<_> = items.into_iter().filter(|item| item.matches(&query)).collect();

            if items.is_empty() {
                continue;
            }

            if visible_groups > 0 {
                list = list.push(container(super::separator::separator()).padding([4, 0]));
            }

            visible_groups += 1;

            if let Some(heading) = heading {
                list = list.push(
                    container(text(heading).size(style::text::XS).font(font::MEDIUM).style(|theme: &Theme| {
                        text::Style {
                            color: Some(tokens(theme).palette.muted_foreground),
                        }
                    }))
                    .padding([6, 8]),
                );
            }

            for item in items {
                let disabled = item.on_select.is_none();

                let row = Row::new()
                    .push(item.icon.map(|icon| {
                        icon.sized(16).style(|theme: &Theme| text::Style {
                            color: Some(tokens(theme).palette.muted_foreground),
                        })
                    }))
                    .push(
                        text(item.label)
                            .size(style::text::SM)
                            .line_height(iced::Pixels(20.0))
                            .style(move |theme: &Theme| text::Style {
                                color: disabled.then(|| {
                                    alpha(tokens(theme).palette.popover_foreground, style::DISABLED_OPACITY)
                                }),
                            }),
                    )
                    .spacing(8)
                    .align_y(Alignment::Center);

                let shortcut = item.shortcut.map(|shortcut| {
                    text(shortcut)
                        .size(style::text::XS)
                        .style(|theme: &Theme| text::Style {
                            color: Some(tokens(theme).palette.muted_foreground),
                        })
                        .into()
                });

                list = list.item(listbox::row(row, shortcut).padding([6, 8]), item.on_select);
            }
        }

        let body: Element<'a, Message> = if visible_groups == 0 {
            container(text(command.empty).size(style::text::SM))
                .padding([24, 0])
                .center_x(Length::Fill)
                .into()
        } else {
            container(
                scrollable(list.padding(4))
                    .direction(scrollable::Direction::Vertical(super::scroll_area::scrollbar()))
                    .style(super::scroll_area::style),
            )
            .max_height(300)
            .into()
        };

        let bordered = command.bordered;

        container(
            Column::new()
                .push(input)
                .push(super::separator::separator())
                .push(body)
                .width(Length::Fill),
        )
        .width(command.width)
        .style(move |theme: &Theme| {
            let t = tokens(theme);

            container::Style {
                background: Some(Background::Color(t.palette.popover)),
                text_color: Some(t.palette.popover_foreground),
                border: if bordered {
                    style::border(t.palette.border, t.radius_lg())
                } else {
                    style::rounded(t.radius_md())
                },
                shadow: if bordered { style::shadow_md() } else { iced::Shadow::default() },
                ..container::Style::default()
            }
        })
        .into()
    }
}
