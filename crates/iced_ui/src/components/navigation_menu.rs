//! A collection of links for navigating websites.
//!
//! Port of <https://ui.shadcn.com/docs/components/navigation-menu>.
//!
//! ```no_run
//! # use iced_ui::navigation_menu::{navigation_menu, link};
//! # #[derive(Clone)] enum Message { Docs, Intro }
//! # fn view<'a>() -> iced::Element<'a, Message> {
//! navigation_menu()
//!     .menu("Getting started", link("Introduction", "Re-usable components.", Message::Intro))
//!     .link("Docs", Message::Docs)
//!     .into()
//! # }
//! ```
use iced_ui_core::icon::lucide;
use iced_ui_core::primitive::{Align, Anchored, Pressable, Side};
use iced_ui_core::theme::alpha;
use iced_ui_core::{Element, font, style, tokens};

use iced::time::Duration;
use iced::widget::button::{Status, Style};
use iced::widget::{Column, Row, container, text};
use iced::{Alignment, Background, Length, Theme, border};

enum Entry<'a, Message> {
    Menu(String, Element<'a, Message>),
    Link(String, Message),
}

/// A navigation menu. See the [module documentation](self).
pub struct NavigationMenu<'a, Message> {
    entries: Vec<Entry<'a, Message>>,
}

/// Creates an empty [`NavigationMenu`].
pub fn navigation_menu<'a, Message: Clone + 'a>() -> NavigationMenu<'a, Message> {
    NavigationMenu {
        entries: Vec::new(),
    }
}

impl<'a, Message: Clone + 'a> NavigationMenu<'a, Message> {
    /// Appends a trigger revealing `content` on hover.
    pub fn menu(
        mut self,
        label: impl Into<String>,
        content: impl Into<Element<'a, Message>>,
    ) -> Self {
        self.entries.push(Entry::Menu(label.into(), content.into()));
        self
    }

    /// Appends a plain link.
    pub fn link(mut self, label: impl Into<String>, on_press: Message) -> Self {
        self.entries.push(Entry::Link(label.into(), on_press));
        self
    }
}

/// The style of a trigger (`navigationMenuTriggerStyle`).
pub fn trigger_style(theme: &Theme, status: Status) -> Style {
    let t = tokens(theme);
    let hovered = matches!(status, Status::Hovered | Status::Pressed);

    Style {
        background: Some(Background::Color(if hovered {
            t.palette.accent
        } else {
            t.palette.background
        })),
        text_color: if hovered {
            t.palette.accent_foreground
        } else {
            t.palette.foreground
        },
        border: border::rounded(t.radius_md()),
        ..Style::default()
    }
}

/// A link of the content of a navigation menu: a title and a short
/// description (`ListItem` in the shadcn/ui example).
pub fn link<'a, Message: Clone + 'a>(
    title: impl Into<String>,
    description: impl Into<String>,
    on_press: Message,
) -> Element<'a, Message> {
    Pressable::new(
        Column::new()
            .push(
                text(title.into())
                    .size(style::text::SM)
                    .font(font::MEDIUM)
                    .line_height(1.0),
            )
            .push(
                text(description.into())
                    .size(style::text::SM)
                    .line_height(1.375)
                    .style(|theme: &Theme| text::Style {
                        color: Some(tokens(theme).palette.muted_foreground),
                    }),
            )
            .spacing(4),
    )
    .on_press(on_press)
    .padding(8)
    .width(Length::Fill)
    .style(|theme, status| {
        let t = tokens(theme);
        let hovered = matches!(status, Status::Hovered | Status::Pressed);

        Style {
            background: hovered.then_some(Background::Color(t.palette.accent)),
            text_color: if hovered {
                t.palette.accent_foreground
            } else {
                t.palette.popover_foreground
            },
            border: border::rounded(t.radius_sm()),
            ..Style::default()
        }
    })
    .into()
}

impl<'a, Message: Clone + 'a> From<NavigationMenu<'a, Message>> for Element<'a, Message> {
    fn from(menu: NavigationMenu<'a, Message>) -> Self {
        let items = menu.entries.into_iter().map(|entry| match entry {
            Entry::Menu(label, content) => {
                let trigger =
                    Pressable::new(
                        Row::new()
                            .push(text(label).size(style::text::SM).font(font::MEDIUM))
                            .push(lucide::CHEVRON_DOWN.sized(12).style(|theme: &Theme| {
                                text::Style {
                                    color: Some(alpha(tokens(theme).palette.foreground, 0.8)),
                                }
                            }))
                            .spacing(4)
                            .align_y(Alignment::Center),
                    )
                    .padding([8, 16])
                    .height(36)
                    .style(trigger_style);

                let panel = container(content).padding(8).style(super::popover::style);

                Anchored::new(trigger, panel)
                    .hover(Duration::from_millis(100), Duration::from_millis(150))
                    .side(Side::Bottom)
                    .align(Align::Start)
                    .offset(6.0)
                    .into()
            }
            Entry::Link(label, on_press) => {
                Pressable::new(text(label).size(style::text::SM).font(font::MEDIUM))
                    .on_press(on_press)
                    .padding([8, 16])
                    .height(36)
                    .style(trigger_style)
                    .into()
            }
        });

        Row::with_children(items)
            .spacing(4)
            .align_y(Alignment::Center)
            .into()
    }
}
