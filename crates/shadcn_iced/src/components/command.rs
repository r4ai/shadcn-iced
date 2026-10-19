//! Fast, composable command menu.
//!
//! Port of <https://ui.shadcn.com/docs/components/command>. The query lives
//! in your state; the command filters its items and supports keyboard
//! navigation (<kbd>↑</kbd>, <kbd>↓</kbd>, <kbd>Enter</kbd>).
//!
//! ```no_run
//! # use shadcn_iced::command::{command, item};
//! # use shadcn_iced_core::icon::lucide;
//! # #[derive(Clone)] enum Message { Query(String), Calendar }
//! # fn view(query: &str) -> iced::Element<'_, Message> {
//! command(query, Message::Query)
//!     .group("Suggestions", [item("Calendar").icon(lucide::CALENDAR).on_select(Message::Calendar)])
//!     .into()
//! # }
//! ```
use shadcn_iced_core::icon::{Icon, lucide};
use shadcn_iced_core::primitive::{listbox, tracked};
use shadcn_iced_core::theme::alpha;
use shadcn_iced_core::{Element, font, style, tokens};

use iced::widget::{Column, Row, container, scrollable, space, text};
use iced::{Alignment, Background, Length, Padding, Theme, border};

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
            || self
                .keywords
                .iter()
                .any(|keyword| keyword.to_lowercase().contains(&query))
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
        input_height: 32.0,
    }
}

impl<'a, Message: Clone + 'a> Command<'a, Message> {
    /// Appends a group of items with a heading.
    pub fn group(
        mut self,
        heading: impl Into<String>,
        items: impl IntoIterator<Item = Item<Message>>,
    ) -> Self {
        self.groups
            .push((Some(heading.into()), items.into_iter().collect()));
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

    /// Draws a border around the command (`border`, as in the shadcn/ui
    /// examples). Defaults to `true`.
    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }

    /// Sets the width.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the minimum height of the search field. Defaults to `h-8`.
    pub fn input_height(mut self, height: f32) -> Self {
        self.input_height = height;
        self
    }
}

/// The style of the highlighted item of a command (`data-selected:bg-muted
/// data-selected:text-foreground rounded-sm`).
pub fn highlight(theme: &Theme) -> listbox::Style {
    let t = tokens(theme);

    listbox::Style {
        background: Background::Color(t.palette.muted),
        border: border::rounded(t.radius_sm()),
        text_color: t.palette.foreground,
    }
}

/// The search field of a command: an input group with a search icon
/// (`bg-input/30 border-input/30 h-8 rounded-lg`).
pub fn search_field<'a, Message: Clone + 'a>(
    placeholder: &str,
    query: &str,
    on_query: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    container(
        Row::new()
            .push(lucide::SEARCH.sized(16).style(|theme: &Theme| text::Style {
                color: Some(alpha(tokens(theme).palette.foreground, 0.5)),
            }))
            .push(
                super::input::input(placeholder, query)
                    .on_input(on_query)
                    .chromeless(true)
                    // `text-sm`, whatever the window.
                    .text_size(style::text::SM)
                    .padding(Padding::ZERO.top(6).bottom(6)),
            )
            .spacing(6)
            .align_y(Alignment::Center),
    )
    // The 1px border, `pl-2` before the icon.
    .padding(Padding::ZERO.left(9).right(1))
    .height(32)
    .center_y(32)
    .width(Length::Fill)
    .style(|theme: &Theme| {
        let t = tokens(theme);

        container::Style {
            background: Some(Background::Color(alpha(t.palette.input, 0.3))),
            border: style::border(alpha(t.palette.input, 0.3), t.radius_lg()),
            ..container::Style::default()
        }
    })
    .into()
}

impl<'a, Message: Clone + 'a> From<Command<'a, Message>> for Element<'a, Message> {
    fn from(command: Command<'a, Message>) -> Self {
        let query = command.query.clone();

        // `p-1 pb-0`
        let input = container(search_field(
            &command.placeholder,
            &command.query,
            command.on_query,
        ))
        .padding(Padding::new(4.0).bottom(0))
        .height(command.input_height.max(32.0) + 4.0)
        .center_y(command.input_height.max(32.0) + 4.0);

        // The first item is highlighted again whenever the query changes.
        let mut list = listbox::Listbox::new(highlight)
            .width(Length::Fill)
            .highlight_first(true)
            .clear_on_leave(false)
            .key(&query);

        let mut visible_groups = 0;

        let separator =
            || {
                container(container(space().width(Length::Fill).height(1)).style(
                    |theme: &Theme| container::Style {
                        background: Some(Background::Color(tokens(theme).palette.border)),
                        ..container::Style::default()
                    },
                ))
                // `-mx-1`: through the `p-1` of the groups, clipped by the
                // list (`overflow-x-hidden`).
                .padding(Padding::ZERO.left(-4.0).right(-4.0))
            };

        for (heading, items) in command.groups {
            let items: Vec<_> = items
                .into_iter()
                .filter(|item| item.matches(&query))
                .collect();

            if items.is_empty() {
                continue;
            }

            if visible_groups > 0 {
                list = list.push(separator());
            }

            visible_groups += 1;

            // `p-1`
            list = list.push(space().height(4));

            if let Some(heading) = heading {
                // `px-2 py-1.5 text-xs font-medium text-muted-foreground`
                list = list.push(
                    container(
                        text(heading)
                            .size(style::text::XS)
                            .line_height(iced::Pixels(16.0))
                            .font(font::medium())
                            .style(|theme: &Theme| text::Style {
                                color: Some(tokens(theme).palette.muted_foreground),
                            }),
                    )
                    .padding([6, 8]),
                );
            }

            for item in items {
                let disabled = item.on_select.is_none();
                let color = move |theme: &Theme| {
                    disabled.then(|| {
                        alpha(
                            tokens(theme).palette.popover_foreground,
                            style::DISABLED_OPACITY,
                        )
                    })
                };

                let row = Row::new()
                    .push(item.icon.map(|icon| {
                        icon.sized(16).style(move |theme: &Theme| text::Style {
                            color: color(theme),
                        })
                    }))
                    .push(
                        text(item.label)
                            .size(style::text::SM)
                            .line_height(iced::Pixels(20.0))
                            .style(move |theme: &Theme| text::Style {
                                color: color(theme),
                            }),
                    )
                    .spacing(8)
                    .align_y(Alignment::Center);

                // `ml-auto text-xs tracking-widest text-muted-foreground
                // group-data-selected:text-foreground`; `tracking-widest` is
                // `0.1em`.
                let shortcut = item.shortcut.map(|shortcut| {
                    tracked(shortcut, style::text::XS, iced::Pixels(16.0), None, 1.2).into()
                });

                // `gap-2 px-2 py-1.5`
                list = list.item(
                    listbox::row(row, shortcut)
                        .spacing(16.0)
                        .padding([6, 8])
                        .trailing_color(|theme, highlighted| {
                            if highlighted {
                                highlight(theme).text_color
                            } else {
                                tokens(theme).palette.muted_foreground
                            }
                        }),
                    item.on_select,
                );
            }

            list = list.push(space().height(4));
        }

        let body: Element<'a, Message> = if visible_groups == 0 {
            // `py-6 text-center text-sm`
            container(
                text(command.empty)
                    .size(style::text::SM)
                    .line_height(iced::Pixels(20.0)),
            )
            .padding([24, 0])
            .center_x(Length::Fill)
            .into()
        } else {
            // `max-h-72`
            container(
                scrollable(list.padding([0, 4]))
                    .direction(scrollable::Direction::Vertical(
                        // `no-scrollbar`
                        scrollable::Scrollbar::new().width(0).scroller_width(0),
                    ))
                    .style(super::scroll_area::style),
            )
            .max_height(288)
            .into()
        };

        let bordered = command.bordered;

        // `rounded-xl bg-popover p-1`, inside the border.
        container(Column::new().push(input).push(body).width(Length::Fill))
            .padding(if bordered { 5 } else { 4 })
            .width(command.width)
            .style(move |theme: &Theme| {
                let t = tokens(theme);

                container::Style {
                    background: Some(Background::Color(t.palette.popover)),
                    text_color: Some(t.palette.popover_foreground),
                    border: if bordered {
                        style::border(t.palette.border, t.radius_xl())
                    } else {
                        style::rounded(t.radius_xl())
                    },
                    ..container::Style::default()
                }
            })
            .into()
    }
}
