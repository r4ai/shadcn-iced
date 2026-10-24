//! The menu shared by [`dropdown_menu`](super::dropdown_menu),
//! [`context_menu`](super::context_menu) and [`menubar`](super::menubar).
//!
//! ```no_run
//! # use shadcn_iced::menu::{menu, item};
//! # use shadcn_iced_core::icon::lucide;
//! # #[derive(Clone)] enum Message { Profile, Settings, Logout }
//! # fn view<'a>() -> shadcn_iced::menu::Menu<'a, Message> {
//! menu()
//!     .label("My Account")
//!     .separator()
//!     .item(item("Profile").icon(lucide::USER).shortcut("⇧⌘P").on_select(Message::Profile))
//!     .item(item("Settings").shortcut("⌘S").on_select(Message::Settings))
//!     .separator()
//!     .item(item("Log out").destructive().on_select(Message::Logout))
//! # }
//! ```
use shadcn_iced_core::icon::{Icon, lucide};
use shadcn_iced_core::primitive::anchored::{self, TouchOpen};
use shadcn_iced_core::primitive::{Align, Anchored, Side, box_shadow, listbox, tracked};
use shadcn_iced_core::theme::alpha;
use shadcn_iced_core::{Element, Tokens, font, style, tokens};

use iced::time::Duration;
use iced::widget::{Row, container, space, text};
use iced::{Alignment, Background, Color, Length, Padding, Theme, border};

/// The tokens menu surfaces read their colors from.
///
/// Every color of menus, selects and combobox popups goes through this
/// function, so menu color variants (inverted, translucent...) only need to
/// change it.
pub fn menu_tokens(theme: &Theme) -> Tokens {
    tokens(theme).menu()
}

/// Whether menus are translucent (`cn-menu-translucent`).
fn translucent(theme: &Theme) -> bool {
    tokens(theme).menu.translucent
}

/// The flavor of a [`Menu`]: dropdown menus, context menus and menubar
/// menus differ in a few details.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Kind {
    /// A dropdown menu (`min-w-32`, indicators on the right).
    #[default]
    Dropdown,
    /// A context menu (`min-w-36`, indicators on the right).
    Context,
    /// A menubar menu (`min-w-36`, indicators on the left, `text-sm`
    /// labels).
    Menubar,
}

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
    /// Sets the icon (`size-4`), placed before the label.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Sets the keyboard shortcut hint, displayed on the right (`ml-auto
    /// text-xs text-muted-foreground`).
    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Sets the message produced when the item is selected. Items without
    /// one are disabled (`opacity-50`).
    pub fn on_select(mut self, message: Message) -> Self {
        self.on_select = Some(message);
        self
    }

    /// Colors the item as destructive (`variant="destructive"`).
    pub fn destructive(mut self) -> Self {
        self.destructive = true;
        self
    }

    /// Indents the item (`inset`, `pl-7`), to align it with items that have
    /// an icon.
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
        icon: Option<Icon>,
    },
    Radio {
        label: String,
        selected: bool,
        on_select: Option<Message>,
        icon: Option<Icon>,
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
    min_width: Option<f32>,
    width: Option<f32>,
    kind: Kind,
    sub: bool,
}

/// Creates an empty [`Menu`].
pub fn menu<'a, Message: Clone + 'a>() -> Menu<'a, Message> {
    Menu {
        entries: Vec::new(),
        min_width: None,
        width: None,
        kind: Kind::Dropdown,
        sub: false,
    }
}

impl<'a, Message: Clone + 'a> Menu<'a, Message> {
    /// Appends an item.
    pub fn item(mut self, item: Item<Message>) -> Self {
        self.entries.push(Entry::Item(item));
        self
    }

    /// Appends a checkbox item; its check mark is shown on the right (on
    /// the left in a menubar).
    pub fn checkbox(mut self, label: impl Into<String>, checked: bool, on_toggle: Message) -> Self {
        self.entries.push(Entry::Check {
            label: label.into(),
            checked,
            on_toggle: Some(on_toggle),
            icon: None,
        });
        self
    }

    /// Appends a checkbox item that is disabled when `on_toggle` is `None`.
    pub fn checkbox_maybe(
        mut self,
        label: impl Into<String>,
        checked: bool,
        on_toggle: Option<Message>,
    ) -> Self {
        self.entries.push(Entry::Check {
            label: label.into(),
            checked,
            on_toggle,
            icon: None,
        });
        self
    }

    /// Appends a checkbox item with an icon before its label.
    pub fn checkbox_with_icon(
        mut self,
        icon: Icon,
        label: impl Into<String>,
        checked: bool,
        on_toggle: Message,
    ) -> Self {
        self.entries.push(Entry::Check {
            label: label.into(),
            checked,
            on_toggle: Some(on_toggle),
            icon: Some(icon),
        });
        self
    }

    /// Appends a radio item; the selected one shows a check mark.
    pub fn radio(mut self, label: impl Into<String>, selected: bool, on_select: Message) -> Self {
        self.entries.push(Entry::Radio {
            label: label.into(),
            selected,
            on_select: Some(on_select),
            icon: None,
        });
        self
    }

    /// Appends a radio item with an icon before its label.
    pub fn radio_with_icon(
        mut self,
        icon: Icon,
        label: impl Into<String>,
        selected: bool,
        on_select: Message,
    ) -> Self {
        self.entries.push(Entry::Radio {
            label: label.into(),
            selected,
            on_select: Some(on_select),
            icon: Some(icon),
        });
        self
    }

    /// Appends a label (`DropdownMenuLabel`).
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.entries.push(Entry::Label(label.into(), false));
        self
    }

    /// Appends an indented label (`inset`, `pl-7`).
    pub fn inset_label(mut self, label: impl Into<String>) -> Self {
        self.entries.push(Entry::Label(label.into(), true));
        self
    }

    /// Appends a separator (`-mx-1 my-1 h-px bg-border`).
    pub fn separator(mut self) -> Self {
        self.entries.push(Entry::Separator);
        self
    }

    /// Appends a submenu, opened when its trigger is hovered or tapped, or
    /// with <kbd>→</kbd>, <kbd>Enter</kbd> or <kbd>Space</kbd> while its
    /// trigger is highlighted (<kbd>←</kbd> closes it again).
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

    /// Sets the minimum width. Defaults to `min-w-32` in dropdown menus and
    /// `min-w-36` in context and menubar menus.
    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = Some(width);
        self
    }

    /// Sets the width of the menu (e.g. `className="w-40"`); longer items
    /// wrap. By default, dropdown and menubar menus are as wide as their
    /// trigger but at least their minimum width (`w-(--anchor-width)`),
    /// and the others fit their items.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// The minimum width of a menu that is as wide as its trigger
    /// (`w-(--anchor-width)`), or `None` when its width is set.
    pub fn anchor_min_width(&self) -> Option<f32> {
        self.width
            .is_none()
            .then(|| self.min_width.unwrap_or_else(|| self.default_min_width()))
    }

    /// Sets the flavor of the menu. [`dropdown_menu`](super::dropdown_menu),
    /// [`context_menu`](super::context_menu) and [`menubar`](super::menubar)
    /// set it for you.
    pub fn kind(mut self, kind: Kind) -> Self {
        self.kind = kind;
        self
    }

    fn default_min_width(&self) -> f32 {
        match (self.kind, self.sub) {
            // `min-w-[96px]`
            (Kind::Dropdown, true) => 96.0,
            // `min-w-32`
            (Kind::Dropdown, false) | (Kind::Menubar, true) => 128.0,
            // `min-w-36`
            (Kind::Context, _) | (Kind::Menubar, false) => 144.0,
        }
    }
}

fn opacity(disabled: bool) -> f32 {
    if disabled {
        style::DISABLED_OPACITY
    } else {
        1.0
    }
}

/// The color of an item's text and icons, or `None` to follow the
/// highlight.
fn item_color(theme: &Theme, disabled: bool, destructive: bool) -> Option<Color> {
    let p = menu_tokens(theme).palette;

    if destructive {
        // `**:data-[variant=destructive]:text-accent-foreground!` when
        // translucent.
        let color = if translucent(theme) {
            p.accent_foreground
        } else {
            p.destructive
        };

        Some(alpha(color, opacity(disabled)))
    } else if disabled {
        Some(alpha(p.popover_foreground, opacity(disabled)))
    } else {
        None
    }
}

fn label_text<'a>(label: String, disabled: bool, destructive: bool) -> text::Text<'a> {
    text(label)
        .size(style::text::SM)
        .line_height(iced::Pixels(20.0))
        .style(move |theme: &Theme| text::Style {
            color: item_color(theme, disabled, destructive),
        })
}

fn icon_of<'a, Message: 'a>(icon: Icon, disabled: bool, destructive: bool) -> Element<'a, Message> {
    icon.sized(16)
        .style(move |theme: &Theme| text::Style {
            color: item_color(theme, disabled, destructive),
        })
        .into()
}

/// A check mark indicator (`size-4`), or the space it takes.
fn indicator<'a, Message: 'a>(checked: bool, disabled: bool) -> Element<'a, Message> {
    container(checked.then(|| {
        lucide::CHECK
            .sized(16)
            .style(move |theme: &Theme| text::Style {
                color: item_color(theme, disabled, false),
            })
    }))
    .width(16)
    .center_y(20)
    .into()
}

/// A shortcut hint (`ml-auto text-xs tracking-widest`), colored by
/// [`shortcut_color`].
pub fn shortcut<'a, Message: 'a>(shortcut: impl Into<String>) -> Element<'a, Message> {
    // `tracking-widest` is `0.1em`.
    tracked(shortcut, style::text::XS, iced::Pixels(16.0), None, 1.2).into()
}

/// The color of a shortcut hint (`text-muted-foreground
/// group-focus:text-accent-foreground`).
pub fn shortcut_color(theme: &Theme, highlighted: bool, disabled: bool) -> Color {
    if highlighted && !disabled {
        highlight(theme).text_color
    } else {
        alpha(
            menu_tokens(theme).palette.muted_foreground,
            opacity(disabled),
        )
    }
}

/// `px-1.5 py-1`
pub const ITEM_PADDING: Padding = Padding {
    top: 4.0,
    right: 6.0,
    bottom: 4.0,
    left: 6.0,
};

/// `pl-7`, the left padding of inset items.
pub const INSET: f32 = 28.0;

/// `gap-1.5`, between the icon and the label of an item.
pub const GAP: f32 = 6.0;

/// The style of the highlighted item of a menu (`focus:bg-accent
/// focus:text-accent-foreground rounded-md`).
pub fn highlight(theme: &Theme) -> listbox::Style {
    let t = menu_tokens(theme);

    listbox::Style {
        background: Background::Color(if translucent(theme) {
            // `**:data-[slot$=-item]:focus:bg-foreground/10`
            alpha(t.palette.foreground, 0.1)
        } else {
            t.palette.accent
        }),
        border: border::rounded(t.radius_md()),
        text_color: t.palette.accent_foreground,
    }
}

/// The style of a highlighted destructive item (`focus:bg-destructive/10`,
/// `/20` in dark mode).
pub fn destructive_highlight(theme: &Theme) -> listbox::Style {
    let t = menu_tokens(theme);

    if translucent(theme) {
        return highlight(theme);
    }

    listbox::Style {
        background: Background::Color(alpha(
            t.palette.destructive,
            if t.is_dark() { 0.2 } else { 0.1 },
        )),
        border: border::rounded(t.radius_md()),
        text_color: t.palette.destructive,
    }
}

/// The style of the menu panel (`rounded-lg bg-popover p-1 shadow-md ring-1
/// ring-foreground/10`), `bg-popover/70` when translucent (iced cannot blur
/// what is behind it).
pub fn panel(theme: &Theme) -> container::Style {
    let t = menu_tokens(theme);
    let background = if translucent(theme) {
        alpha(t.palette.popover, 0.7)
    } else {
        t.palette.popover
    };

    container::Style {
        background: Some(Background::Color(background)),
        text_color: Some(t.palette.popover_foreground),
        border: style::ring_border(&t, t.radius_lg()),
        ..container::Style::default()
    }
}

/// Draws the shadow of a menu panel (`shadow-md`, `shadow-lg` for
/// submenus) behind it, which a container style cannot (see
/// [`box_shadow`]).
pub fn shadow<'a, Message: 'a>(
    panel: impl Into<Element<'a, Message>>,
    sub: bool,
) -> Element<'a, Message> {
    box_shadow(
        panel,
        if sub {
            style::SHADOW_LG
        } else {
            style::SHADOW_MD
        },
        |theme| menu_tokens(theme).radius_lg(),
    )
    // The `ring-1` is outside of the box that casts the shadow.
    .inset(1.0)
    .into()
}

/// The style of a submenu panel, whose shadow is drawn by [`shadow`].
pub fn sub_panel(theme: &Theme) -> container::Style {
    panel(theme)
}

/// The style of a context submenu panel (`border shadow-lg`).
fn context_sub_panel(theme: &Theme) -> container::Style {
    let t = menu_tokens(theme);

    container::Style {
        border: style::border(t.palette.border, t.radius_lg()),
        ..sub_panel(theme)
    }
}

/// Builds a menu panel from a [`Listbox`](listbox::Listbox).
pub fn panel_with<'a, Message: Clone + 'a>(
    list: listbox::Listbox<'a, Message>,
) -> Element<'a, Message> {
    shadow(container(list.padding(4)).padding(1).style(panel), false)
}

/// A menu separator (`-mx-1 my-1 h-px bg-border`), for a listbox with `p-1`.
pub fn separator<'a, Message: 'a>() -> Element<'a, Message> {
    container(
        container(space().width(iced::Length::Fill).height(1)).style(|theme: &Theme| {
            container::Style {
                background: Some(Background::Color(if translucent(theme) {
                    // `**:data-[slot$=-separator]:bg-foreground/5`
                    alpha(menu_tokens(theme).palette.foreground, 0.05)
                } else {
                    menu_tokens(theme).palette.border
                })),
                ..container::Style::default()
            }
        }),
    )
    .padding(Padding {
        top: 4.0,
        bottom: 4.0,
        left: -4.0,
        right: -4.0,
    })
    .into()
}

/// A menu label (`px-1.5 py-1 text-xs font-medium text-muted-foreground`).
pub fn group_label<'a, Message: 'a>(label: impl Into<String>, inset: bool) -> Element<'a, Message> {
    container(
        text(label.into())
            .size(style::text::XS)
            .font(font::medium())
            .line_height(iced::Pixels(16.0))
            .style(|theme: &Theme| text::Style {
                color: Some(menu_tokens(theme).palette.muted_foreground),
            }),
    )
    .padding(if inset {
        ITEM_PADDING.left(INSET)
    } else {
        ITEM_PADDING
    })
    .into()
}

impl<'a, Message: Clone + 'a> From<Menu<'a, Message>> for Element<'a, Message> {
    fn from(menu: Menu<'a, Message>) -> Self {
        let kind = menu.kind;
        let min_width = menu.min_width.unwrap_or_else(|| menu.default_min_width());
        let width = menu.width;
        // Radix menus select with `Space` and highlight the item whose label
        // is typed (`typeahead`), skipping the disabled ones.
        let mut list = listbox::Listbox::new(highlight).typeahead(true);

        for entry in menu.entries {
            let typed = match &entry {
                Entry::Item(item) => item.on_select.as_ref().map(|_| item.label.clone()),
                Entry::Check {
                    label, on_toggle, ..
                } => on_toggle.as_ref().map(|_| label.clone()),
                Entry::Radio {
                    label, on_select, ..
                } => on_select.as_ref().map(|_| label.clone()),
                Entry::Sub { label, .. } => Some(label.clone()),
                Entry::Label(..) | Entry::Separator | Entry::Custom(_) => None,
            };

            list = match entry {
                Entry::Item(item) => {
                    let disabled = item.on_select.is_none();
                    let destructive = item.destructive;

                    let row = Row::new()
                        .push(item.icon.map(|icon| icon_of(icon, disabled, destructive)))
                        .push(label_text(item.label, disabled, destructive))
                        .spacing(GAP)
                        .align_y(Alignment::Center);

                    let has_shortcut = item.shortcut.is_some();
                    let shortcut = item.shortcut.map(shortcut);

                    let padding = if item.inset {
                        ITEM_PADDING.left(INSET)
                    } else {
                        ITEM_PADDING
                    };

                    let mut row = listbox::row(row, shortcut).spacing(16.0).padding(padding);

                    if has_shortcut {
                        row = row.trailing_color(move |theme, highlighted| {
                            shortcut_color(theme, highlighted, disabled)
                        });
                    }

                    if destructive {
                        list.styled_item(row, item.on_select, destructive_highlight)
                    } else {
                        list.item(row, item.on_select)
                    }
                }
                Entry::Check {
                    label,
                    checked: on,
                    on_toggle: on_select,
                    icon,
                }
                | Entry::Radio {
                    label,
                    selected: on,
                    on_select,
                    icon,
                } => {
                    let disabled = on_select.is_none();

                    let content = Row::new()
                        .push(icon.map(|icon| icon_of(icon, disabled, false)))
                        .push(label_text(label, disabled, false))
                        .spacing(GAP)
                        .align_y(Alignment::Center);

                    let row = if kind == Kind::Menubar {
                        // `pl-7 pr-1.5`, indicator at `left-1.5`.
                        listbox::row(
                            Row::new()
                                .push(indicator(on, disabled))
                                .push(content)
                                .spacing(GAP)
                                .align_y(Alignment::Center),
                            None,
                        )
                        .padding(ITEM_PADDING)
                    } else {
                        // `pr-8 pl-1.5`, indicator at `right-2`.
                        listbox::row(content, Some(indicator(on, disabled)))
                            .spacing(8.0)
                            .padding(ITEM_PADDING.right(8))
                    };

                    list.item(row, on_select)
                }
                Entry::Label(label, inset) => {
                    if kind == Kind::Menubar {
                        // `px-1.5 py-1 text-sm font-medium`
                        list.push(
                            container(
                                text(label)
                                    .size(style::text::SM)
                                    .font(font::medium())
                                    .line_height(iced::Pixels(20.0)),
                            )
                            .padding(if inset {
                                ITEM_PADDING.left(INSET)
                            } else {
                                ITEM_PADDING
                            }),
                        )
                    } else {
                        list.push(group_label(label, inset))
                    }
                }
                Entry::Separator => list.push(separator()),
                Entry::Sub {
                    label,
                    icon,
                    mut menu,
                } => {
                    menu.kind = kind;
                    menu.sub = true;

                    let trigger = Row::new()
                        .push(icon.map(|icon| icon_of(icon, false, false)))
                        .push(label_text(label, false, false))
                        .spacing(GAP)
                        .align_y(Alignment::Center);

                    let (offset, align_offset) = match kind {
                        // `side="right"` with the content's defaults.
                        Kind::Context => (0.0, 4.0),
                        // `sideOffset={0} alignOffset={-3}`
                        Kind::Dropdown | Kind::Menubar => (0.0, -3.0),
                    };

                    list.passive(
                        Anchored::new(
                            listbox::row(trigger, Some(lucide::CHEVRON_RIGHT.sized(16).into()))
                                .spacing(16.0)
                                .padding(ITEM_PADDING)
                                // The color the chevron inherits, telling the
                                // trigger whether it is highlighted: the keys
                                // that open the submenu are meant for it.
                                .trailing_color(|theme, highlighted| {
                                    anchored::report_highlighted(highlighted);

                                    if highlighted {
                                        highlight(theme).text_color
                                    } else {
                                        menu_tokens(theme).palette.popover_foreground
                                    }
                                }),
                            menu,
                        )
                        .hover(Duration::ZERO, Duration::from_millis(150))
                        // Radix opens submenus on click too: a tap, without
                        // hover.
                        .touch_open(TouchOpen::Tap)
                        .menu_keys(true)
                        .side(Side::Right)
                        .align(Align::Start)
                        .offset(offset)
                        .align_offset(align_offset)
                        .close_on_select(true)
                        // `ring-1` around the panel.
                        .outset(1.0)
                        // `data-open:bg-accent`
                        .open_highlight(|theme| {
                            let style = highlight(theme);
                            (style.background, style.border)
                        }),
                    )
                }
                Entry::Custom(element) => list.push(element),
            };

            if let Some(typed) = typed {
                list = list.text_value(typed);
            }
        }

        let style = match (menu.sub, kind) {
            (false, _) => panel,
            (true, Kind::Context) => context_sub_panel,
            (true, _) => sub_panel,
        };

        // `p-1` inside a 1px ring, outside of the width.
        let list = list.padding(4).min_width(min_width - 8.0);

        let panel = match width {
            Some(width) => container(list.width(Length::Fill)).width(width + 2.0),
            None => container(list),
        }
        .padding(1)
        .style(style);

        shadow(panel, menu.sub)
    }
}
