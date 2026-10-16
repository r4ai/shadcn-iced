//! The menu shared by [`dropdown_menu`](super::dropdown_menu),
//! [`context_menu`](super::context_menu) and [`menubar`](super::menubar).
//!
//! ```no_run
//! # use iced_ui::menu::{menu, item};
//! # use iced_ui_core::icon::lucide;
//! # #[derive(Clone)] enum Message { Profile, Settings, Logout }
//! # fn view<'a>() -> iced_ui::menu::Menu<'a, Message> {
//! menu()
//!     .label("My Account")
//!     .separator()
//!     .item(item("Profile").icon(lucide::USER).shortcut("⇧⌘P").on_select(Message::Profile))
//!     .item(item("Settings").shortcut("⌘S").on_select(Message::Settings))
//!     .separator()
//!     .item(item("Log out").destructive().on_select(Message::Logout))
//! # }
//! ```
use iced_ui_core::icon::{Icon, lucide};
use iced_ui_core::primitive::{Align, Anchored, Side, listbox};
use iced_ui_core::theme::alpha;
use iced_ui_core::{Element, font, style, tokens};

use iced::time::Duration;
use iced::widget::{Row, container, space, text};
use iced::{Alignment, Background, Padding, Theme, border};

/// An item of a [`Menu`].
pub struct Item<Message> {
    label: String,
    icon: Option<Icon>,
    shortcut: Option<String>,
    on_select: Option<Message>,
    destructive: bool,
    inset: bool,
}

/// Creates a menu [`Item`] with a label.
pub fn item<Message>(label: impl Into<String>) -> Item<Message> {
    Item {
        label: label.into(),
        icon: None,
        shortcut: None,
        on_select: None,
        destructive: false,
        inset: false,
    }
}

impl<Message> Item<Message> {
    /// Sets the icon.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Sets the keyboard shortcut hint, displayed on the right.
    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Sets the message produced when the item is selected. Items without
    /// one are disabled.
    pub fn on_select(mut self, message: Message) -> Self {
        self.on_select = Some(message);
        self
    }

    /// Colors the item as destructive.
    pub fn destructive(mut self) -> Self {
        self.destructive = true;
        self
    }

    /// Indents the item, to align it with checkbox items.
    pub fn inset(mut self) -> Self {
        self.inset = true;
        self
    }
}

enum Entry<'a, Message> {
    Item(Item<Message>),
    Check {
        label: String,
        checked: bool,
        on_toggle: Option<Message>,
    },
    Radio {
        label: String,
        selected: bool,
        on_select: Option<Message>,
    },
    Label(String, bool),
    Separator,
    Sub {
        label: String,
        icon: Option<Icon>,
        menu: Menu<'a, Message>,
    },
    Custom(Element<'a, Message>),
}

/// A menu. See the [module documentation](self).
pub struct Menu<'a, Message> {
    entries: Vec<Entry<'a, Message>>,
    min_width: f32,
}

/// Creates an empty [`Menu`].
pub fn menu<'a, Message: Clone + 'a>() -> Menu<'a, Message> {
    Menu {
        entries: Vec::new(),
        min_width: 128.0,
    }
}

impl<'a, Message: Clone + 'a> Menu<'a, Message> {
    /// Appends an item.
    pub fn item(mut self, item: Item<Message>) -> Self {
        self.entries.push(Entry::Item(item));
        self
    }

    /// Appends a checkbox item.
    pub fn checkbox(mut self, label: impl Into<String>, checked: bool, on_toggle: Message) -> Self {
        self.entries.push(Entry::Check {
            label: label.into(),
            checked,
            on_toggle: Some(on_toggle),
        });
        self
    }

    /// Appends a radio item.
    pub fn radio(mut self, label: impl Into<String>, selected: bool, on_select: Message) -> Self {
        self.entries.push(Entry::Radio {
            label: label.into(),
            selected,
            on_select: Some(on_select),
        });
        self
    }

    /// Appends a label (`DropdownMenuLabel`).
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.entries.push(Entry::Label(label.into(), false));
        self
    }

    /// Appends an indented label, aligned with checkbox items.
    pub fn inset_label(mut self, label: impl Into<String>) -> Self {
        self.entries.push(Entry::Label(label.into(), true));
        self
    }

    /// Appends a separator.
    pub fn separator(mut self) -> Self {
        self.entries.push(Entry::Separator);
        self
    }

    /// Appends a submenu.
    pub fn sub(mut self, label: impl Into<String>, menu: Menu<'a, Message>) -> Self {
        self.entries.push(Entry::Sub {
            label: label.into(),
            icon: None,
            menu,
        });
        self
    }

    /// Appends a submenu with an icon.
    pub fn sub_with_icon(
        mut self,
        icon: Icon,
        label: impl Into<String>,
        menu: Menu<'a, Message>,
    ) -> Self {
        self.entries.push(Entry::Sub {
            label: label.into(),
            icon: Some(icon),
            menu,
        });
        self
    }

    /// Appends a custom element.
    pub fn push(mut self, element: impl Into<Element<'a, Message>>) -> Self {
        self.entries.push(Entry::Custom(element.into()));
        self
    }

    /// Sets the minimum width. Defaults to `min-w-[8rem]`.
    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = width;
        self
    }
}

fn muted(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(tokens(theme).palette.muted_foreground),
    }
}

fn label_text<'a>(label: String, disabled: bool, destructive: bool) -> text::Text<'a> {
    text(label)
        .size(style::text::SM)
        .line_height(iced::Pixels(20.0))
        .wrapping(text::Wrapping::None)
        .style(move |theme: &Theme| {
            let p = tokens(theme).palette;
            let opacity = if disabled {
                style::DISABLED_OPACITY
            } else {
                1.0
            };

            text::Style {
                color: if destructive {
                    Some(alpha(p.destructive, opacity))
                } else if disabled {
                    Some(alpha(p.popover_foreground, opacity))
                } else {
                    None
                },
            }
        })
}

const ITEM_PADDING: Padding = Padding {
    top: 6.0,
    right: 8.0,
    bottom: 6.0,
    left: 8.0,
};

/// The style of the highlighted item of a menu.
pub fn highlight(theme: &Theme) -> listbox::Style {
    let t = tokens(theme);

    listbox::Style {
        background: Background::Color(t.palette.accent),
        border: border::rounded(t.radius_sm()),
        text_color: t.palette.accent_foreground,
    }
}

/// The style of the menu panel (`rounded-md border bg-popover p-1
/// shadow-md`).
pub fn panel(theme: &Theme) -> container::Style {
    let t = tokens(theme);

    container::Style {
        background: Some(Background::Color(t.palette.popover)),
        text_color: Some(t.palette.popover_foreground),
        border: style::border(t.palette.border, t.radius_md()),
        shadow: style::shadow_md(),
        ..container::Style::default()
    }
}

/// Builds a menu panel from a [`Listbox`](listbox::Listbox).
pub fn panel_with<'a, Message: Clone + 'a>(
    list: listbox::Listbox<'a, Message>,
) -> Element<'a, Message> {
    container(list.padding(4)).style(panel).into()
}

impl<'a, Message: Clone + 'a> From<Menu<'a, Message>> for Element<'a, Message> {
    fn from(menu: Menu<'a, Message>) -> Self {
        let mut list = listbox::Listbox::new(highlight);

        for entry in menu.entries {
            list = match entry {
                Entry::Item(item) => {
                    let disabled = item.on_select.is_none();
                    let row = Row::new()
                        .push(item.icon.map(|icon| {
                            icon.sized(16).style(move |theme: &Theme| text::Style {
                                color: Some(alpha(
                                    if item.destructive {
                                        tokens(theme).palette.destructive
                                    } else {
                                        tokens(theme).palette.muted_foreground
                                    },
                                    if disabled {
                                        style::DISABLED_OPACITY
                                    } else {
                                        1.0
                                    },
                                )),
                            })
                        }))
                        .push(label_text(item.label, disabled, item.destructive))
                        .spacing(8)
                        .align_y(Alignment::Center);

                    let shortcut = item.shortcut.map(|shortcut| {
                        text(shortcut)
                            .size(style::text::XS)
                            .wrapping(text::Wrapping::None)
                            .style(muted)
                            .into()
                    });

                    let padding = if item.inset {
                        ITEM_PADDING.left(32)
                    } else {
                        ITEM_PADDING
                    };

                    list.item(listbox::row(row, shortcut).padding(padding), item.on_select)
                }
                Entry::Check {
                    label,
                    checked,
                    on_toggle,
                } => {
                    let disabled = on_toggle.is_none();
                    let row = Row::new()
                        .push(
                            container(checked.then(|| lucide::CHECK.sized(16)))
                                .width(16)
                                .center_x(16),
                        )
                        .push(label_text(label, disabled, false))
                        .spacing(8)
                        .align_y(Alignment::Center);

                    list.item(listbox::row(row, None).padding(ITEM_PADDING), on_toggle)
                }
                Entry::Radio {
                    label,
                    selected,
                    on_select,
                } => {
                    let disabled = on_select.is_none();
                    let dot = selected.then(|| {
                        container(space().width(8).height(8)).style(|theme: &Theme| {
                            container::Style {
                                background: Some(Background::Color(
                                    tokens(theme).palette.popover_foreground,
                                )),
                                border: border::rounded(f32::MAX),
                                ..container::Style::default()
                            }
                        })
                    });

                    let row = Row::new()
                        .push(container(dot).center(16))
                        .push(label_text(label, disabled, false))
                        .spacing(8)
                        .align_y(Alignment::Center);

                    list.item(listbox::row(row, None).padding(ITEM_PADDING), on_select)
                }
                Entry::Label(label, inset) => list.push(
                    container(
                        text(label)
                            .size(style::text::SM)
                            .font(font::MEDIUM)
                            .line_height(iced::Pixels(20.0)),
                    )
                    .padding(if inset {
                        ITEM_PADDING.left(32)
                    } else {
                        ITEM_PADDING
                    }),
                ),
                Entry::Separator => {
                    list.push(container(super::separator::separator()).padding(Padding {
                        top: 4.0,
                        bottom: 4.0,
                        left: -4.0,
                        right: -4.0,
                    }))
                }
                Entry::Sub { label, icon, menu } => {
                    let trigger = Row::new()
                        .push(icon.map(|icon| icon.sized(16).style(muted)))
                        .push(label_text(label, false, false))
                        .spacing(8)
                        .align_y(Alignment::Center);

                    list.passive(
                        Anchored::new(
                            listbox::row(trigger, Some(lucide::CHEVRON_RIGHT.sized(16).into()))
                                .padding(ITEM_PADDING),
                            menu,
                        )
                        .hover(Duration::ZERO, Duration::from_millis(150))
                        .side(Side::Right)
                        .align(Align::Start)
                        .offset(8.0)
                        .align_offset(-5.0)
                        .close_on_select(true),
                    )
                }
                Entry::Custom(element) => list.push(element),
            };
        }

        container(list.padding(4).min_width(menu.min_width - 10.0))
            .style(panel)
            .into()
    }
}
