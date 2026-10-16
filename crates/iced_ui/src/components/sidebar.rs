//! A composable, themeable and customizable sidebar component.
//!
//! Port of <https://ui.shadcn.com/docs/components/sidebar>. Whether the
//! sidebar is collapsed (to icons) lives in your state:
//!
//! ```no_run
//! # use iced_ui::sidebar::{sidebar, group, item};
//! # use iced_ui_core::icon::lucide;
//! # #[derive(Clone)] enum Message { Home, Inbox }
//! # fn view<'a>(content: iced::Element<'a, Message>) -> iced::Element<'a, Message> {
//! let nav = sidebar()
//!     .group(
//!         group("Application")
//!             .item(item("Home").icon(lucide::HOUSE).active(true).on_press(Message::Home))
//!             .item(item("Inbox").icon(lucide::INBOX).badge("24").on_press(Message::Inbox)),
//!     );
//!
//! iced_ui::sidebar::layout(nav, content).into()
//! # }
//! ```
use iced_ui_core::icon::{Icon, lucide};
use iced_ui_core::primitive::{Pressable, Side};
use iced_ui_core::theme::alpha;
use iced_ui_core::{Element, font, style, tokens};

use iced::widget::button::{Status, Style};
use iced::widget::{Column, Row, container, space, text};
use iced::{Alignment, Background, Length, Theme, border};

/// The width of an expanded sidebar (`16rem`).
pub const WIDTH: f32 = 256.0;

/// The width of a sidebar collapsed to icons (`3rem`).
pub const WIDTH_ICON: f32 = 48.0;

/// The visual style of a [`Sidebar`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Variant {
    /// Attached to the edge of the window, with a border.
    #[default]
    Sidebar,
    /// Detached, rounded and with a shadow.
    Floating,
    /// Like `Floating` but blending with the background; the main content
    /// is drawn as an inset card by [`layout`].
    Inset,
}

/// An item of a [`Group`] (`SidebarMenuButton`).
pub struct Item<'a, Message> {
    label: String,
    icon: Option<Icon>,
    active: bool,
    badge: Option<String>,
    on_press: Option<Message>,
    children: Vec<Item<'a, Message>>,
    action: Option<Element<'a, Message>>,
}

/// Creates a sidebar [`Item`].
pub fn item<'a, Message>(label: impl Into<String>) -> Item<'a, Message> {
    Item {
        label: label.into(),
        icon: None,
        active: false,
        badge: None,
        on_press: None,
        children: Vec::new(),
        action: None,
    }
}

impl<'a, Message> Item<'a, Message> {
    /// Sets the icon.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Marks the item as the current page.
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// Shows a badge on the right (`SidebarMenuBadge`).
    pub fn badge(mut self, badge: impl Into<String>) -> Self {
        self.badge = Some(badge.into());
        self
    }

    /// Sets the message produced when the item is pressed.
    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    /// Appends a nested item (`SidebarMenuSub`).
    pub fn child(mut self, item: Item<'a, Message>) -> Self {
        self.children.push(item);
        self
    }

    /// Shows an element on the right instead of a badge
    /// (`SidebarMenuAction`), e.g. a dropdown menu.
    pub fn action(mut self, action: impl Into<Element<'a, Message>>) -> Self {
        self.action = Some(action.into());
        self
    }
}

/// A group of items with a label (`SidebarGroup`).
pub struct Group<'a, Message> {
    label: Option<String>,
    items: Vec<Item<'a, Message>>,
}

/// Creates a [`Group`] with a label.
pub fn group<'a, Message>(label: impl Into<String>) -> Group<'a, Message> {
    Group {
        label: Some(label.into()),
        items: Vec::new(),
    }
}

/// Creates a [`Group`] without label.
pub fn unlabeled_group<'a, Message>() -> Group<'a, Message> {
    Group {
        label: None,
        items: Vec::new(),
    }
}

impl<'a, Message> Group<'a, Message> {
    /// Appends an item.
    pub fn item(mut self, item: Item<'a, Message>) -> Self {
        self.items.push(item);
        self
    }
}

/// A sidebar. See the [module documentation](self).
pub struct Sidebar<'a, Message> {
    header: Option<Element<'a, Message>>,
    groups: Vec<Group<'a, Message>>,
    footer: Option<Element<'a, Message>>,
    collapsed: bool,
    variant: Variant,
    side: Side,
    width: f32,
}

/// Creates an empty [`Sidebar`].
pub fn sidebar<'a, Message: Clone + 'a>() -> Sidebar<'a, Message> {
    Sidebar {
        header: None,
        groups: Vec::new(),
        footer: None,
        collapsed: false,
        variant: Variant::Sidebar,
        side: Side::Left,
        width: WIDTH,
    }
}

impl<'a, Message: Clone + 'a> Sidebar<'a, Message> {
    /// Sets the header (`SidebarHeader`).
    pub fn header(mut self, header: impl Into<Element<'a, Message>>) -> Self {
        self.header = Some(header.into());
        self
    }

    /// Appends a group.
    pub fn group(mut self, group: Group<'a, Message>) -> Self {
        self.groups.push(group);
        self
    }

    /// Sets the footer (`SidebarFooter`).
    pub fn footer(mut self, footer: impl Into<Element<'a, Message>>) -> Self {
        self.footer = Some(footer.into());
        self
    }

    /// Collapses the sidebar to icons (`collapsible="icon"`); labels are
    /// shown as tooltips.
    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }

    /// Sets the variant.
    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets the side of the window. Only [`Side::Left`] and [`Side::Right`]
    /// are meaningful.
    pub fn side(mut self, side: Side) -> Self {
        self.side = side;
        self
    }

    /// Sets the width when expanded.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
}

/// The style of a menu button.
pub fn button_style(theme: &Theme, status: Status, active: bool) -> Style {
    let t = tokens(theme);
    let p = &t.palette;
    let hovered = matches!(status, Status::Hovered | Status::Pressed);

    Style {
        background: (active || hovered).then_some(Background::Color(p.sidebar_accent)),
        text_color: if active || hovered {
            p.sidebar_accent_foreground
        } else {
            p.sidebar_foreground
        },
        border: border::rounded(t.radius_md()),
        ..Style::default()
    }
}

fn menu_button<'a, Message: Clone + 'a>(
    item: Item<'a, Message>,
    collapsed: bool,
    sub: bool,
) -> Element<'a, Message> {
    let active = item.active;
    let label = item.label.clone();

    let trailing: Option<Element<'a, Message>> = if collapsed {
        None
    } else if let Some(action) = item.action {
        Some(action)
    } else {
        item.badge.map(|badge| {
            text(badge)
                .size(style::text::XS)
                .font(font::MEDIUM)
                .style(|theme: &Theme| text::Style {
                    color: Some(tokens(theme).palette.sidebar_foreground),
                })
                .into()
        })
    };

    let content = Row::new()
        .push(item.icon.map(|icon| icon.sized(16)))
        .push((!collapsed).then(|| {
            text(item.label)
                .size(style::text::SM)
                .font(if active { font::MEDIUM } else { font::SANS })
                .wrapping(text::Wrapping::None)
                .width(Length::Fill)
        }))
        .push(trailing)
        .spacing(8)
        .align_y(Alignment::Center);

    let button: Element<'a, Message> = Pressable::new(container(content).center_y(Length::Fill))
        .on_press_maybe(item.on_press)
        .width(if collapsed {
            Length::Fixed(32.0)
        } else {
            Length::Fill
        })
        .height(if sub { 28 } else { 32 })
        .padding([0, 8])
        .style(move |theme, status| button_style(theme, status, active))
        .into();

    let button = if collapsed {
        super::tooltip::tooltip(button, label)
            .side(Side::Right)
            .offset(8.0)
            .into()
    } else {
        button
    };

    if item.children.is_empty() || collapsed {
        return button;
    }

    let children = Column::with_children(
        item.children
            .into_iter()
            .map(|child| menu_button(child, false, true)),
    )
    .spacing(4)
    .padding(iced::Padding::ZERO.left(11).top(2).bottom(2));

    let line =
        container(space().width(1).height(Length::Fill)).style(|theme: &Theme| container::Style {
            background: Some(Background::Color(tokens(theme).palette.sidebar_border)),
            ..container::Style::default()
        });

    // The guide line spans the nested items: a `Stack` sizes itself after its
    // first layer, so the line can fill its height.
    Column::new()
        .push(button)
        .push(
            container(iced::widget::Stack::new().push(children).push(line))
                .padding(iced::Padding::ZERO.left(14)),
        )
        .spacing(4)
        .into()
}

impl<'a, Message: Clone + 'a> From<Sidebar<'a, Message>> for Element<'a, Message> {
    fn from(sidebar: Sidebar<'a, Message>) -> Self {
        let collapsed = sidebar.collapsed;

        let groups = sidebar.groups.into_iter().map(|group| {
            let label = group.label.filter(|_| !collapsed).map(|label| {
                container(text(label).size(style::text::XS).font(font::MEDIUM).style(
                    |theme: &Theme| text::Style {
                        color: Some(alpha(tokens(theme).palette.sidebar_foreground, 0.7)),
                    },
                ))
                .padding([0, 8])
                .center_y(32)
            });

            Column::new()
                .push(label)
                .push(
                    Column::with_children(
                        group
                            .items
                            .into_iter()
                            .map(|item| menu_button(item, collapsed, false)),
                    )
                    .spacing(4),
                )
                .padding(8)
                .width(Length::Fill)
                .into()
        });

        let body = Column::new()
            .push(
                sidebar
                    .header
                    .map(|header| container(header).padding(8).width(Length::Fill)),
            )
            .push(
                super::scroll_area::scroll_area(Column::with_children(groups)).height(Length::Fill),
            )
            .push(
                sidebar
                    .footer
                    .map(|footer| container(footer).padding(8).width(Length::Fill)),
            )
            .height(Length::Fill);

        let variant = sidebar.variant;
        let width = if collapsed { WIDTH_ICON } else { sidebar.width };

        let panel =
            container(body)
                .width(width)
                .height(Length::Fill)
                .style(move |theme: &Theme| {
                    let t = tokens(theme);
                    let p = &t.palette;

                    container::Style {
                        background: Some(Background::Color(p.sidebar)),
                        text_color: Some(p.sidebar_foreground),
                        border: match variant {
                            Variant::Floating => style::border(p.sidebar_border, t.radius_lg()),
                            _ => border::rounded(0),
                        },
                        shadow: if variant == Variant::Floating {
                            style::shadow_sm()
                        } else {
                            iced::Shadow::default()
                        },
                        ..container::Style::default()
                    }
                });

        match variant {
            Variant::Sidebar => {
                let rule = super::separator::vertical_separator();
                let rule =
                    iced::widget::Rule::style(rule, |theme: &Theme| iced::widget::rule::Style {
                        color: tokens(theme).palette.sidebar_border,
                        radius: 0.0.into(),
                        fill_mode: iced::widget::rule::FillMode::Full,
                        snap: true,
                    });

                if sidebar.side == Side::Right {
                    Row::new().push(rule).push(panel).into()
                } else {
                    Row::new().push(panel).push(rule).into()
                }
            }
            Variant::Floating | Variant::Inset => {
                container(panel).padding(8).height(Length::Fill).into()
            }
        }
    }
}

/// Lays out a sidebar next to the main content (`SidebarProvider` +
/// `SidebarInset`).
pub fn layout<'a, Message: Clone + 'a>(
    sidebar: Sidebar<'a, Message>,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    let variant = sidebar.variant;
    let side = sidebar.side;

    let main = container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |theme: &Theme| {
            let t = tokens(theme);

            container::Style {
                background: Some(Background::Color(t.palette.background)),
                border: if variant == Variant::Inset {
                    border::rounded(t.radius_xl())
                } else {
                    border::rounded(0)
                },
                shadow: if variant == Variant::Inset {
                    style::shadow_sm()
                } else {
                    iced::Shadow::default()
                },
                ..container::Style::default()
            }
        });

    let main: Element<'a, Message> = if variant == Variant::Inset {
        container(main)
            .padding(iced::Padding::new(8.0).left(0))
            .into()
    } else {
        main.into()
    };

    let row = if side == Side::Right {
        Row::new().push(main).push(sidebar)
    } else {
        Row::new().push(sidebar).push(main)
    };

    container(row.height(Length::Fill))
        .style(move |theme: &Theme| container::Style {
            background: (variant != Variant::Sidebar)
                .then(|| Background::Color(tokens(theme).palette.sidebar)),
            ..container::Style::default()
        })
        .into()
}

/// The button that toggles the sidebar (`SidebarTrigger`).
pub fn trigger<'a, Message: Clone + 'a>(on_press: Message) -> Element<'a, Message> {
    super::button::icon_button(lucide::PANEL_LEFT)
        .variant(super::button::Variant::Ghost)
        .size(super::button::Size::IconSm)
        .on_press(on_press)
        .into()
}
