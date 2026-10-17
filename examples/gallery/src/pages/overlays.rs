//! Pages of the overlay components, mirroring the examples of the Base UI
//! docs of shadcn/ui.
use super::{DemoMessage, Page};
use crate::{Message, example};

use shadcn_iced::button::{Size, Variant, button};
use shadcn_iced::command::item as command_item;
use shadcn_iced::menu::item;
use shadcn_iced::prelude::*;
use shadcn_iced::primitive::{Align, Side};
use shadcn_iced::sonner::Kind;
use shadcn_iced::{alert_dialog as ad, drawer as dr, popover as pop, sheet as sh};

use iced::time::Duration;
use iced::widget::{column, container, row, text};
use iced::{Center, Fill};

fn demo(message: DemoMessage) -> Message {
    Message::Demo(message)
}

/// The Name and Username fields of the profile examples.
fn profile_fields<'a>(gallery: &'a crate::Gallery, spacing: u32) -> iced::Element<'a, Message> {
    let d = &gallery.demo;

    column![
        field(
            "Name",
            input("Pedro Duarte", &d.name).on_input(|v| demo(DemoMessage::Name(v)))
        ),
        field(
            "Username",
            input("@peduarte", &d.username).on_input(|v| demo(DemoMessage::Username(v)))
        ),
    ]
    .spacing(spacing)
    .into()
}

pub const DIALOG: Page = Page {
    id: "dialog",
    title: "Dialog",
    description: "A window overlaid on either the primary window or another dialog window, rendering the content underneath inert.",
    view: |gallery| {
        let d = &gallery.demo;

        example(
            "Default",
            dialog(
                button("Open Dialog")
                    .variant(Variant::Outline)
                    .on_press(demo(DemoMessage::Dialog(true))),
                d.dialog,
            )
            .title("Edit profile")
            .description("Make changes to your profile here. Click save when you're done.")
            // `FieldGroup`: `gap-5`
            .content(profile_fields(gallery, 20))
            .footer(
                row![
                    button("Cancel")
                        .variant(Variant::Outline)
                        .on_press(demo(DemoMessage::Dialog(false))),
                    button("Save changes").on_press(demo(DemoMessage::Dialog(false))),
                ]
                .spacing(8),
            )
            .on_close(demo(DemoMessage::Dialog(false))),
        )
    },
};

pub const ALERT_DIALOG: Page = Page {
    id: "alert-dialog",
    title: "Alert Dialog",
    description: "A modal dialog that interrupts the user with important content and expects a response.",
    view: |gallery| {
        let d = &gallery.demo;

        column![
            example(
                "Default",
                alert_dialog(
                    button("Show Dialog")
                        .variant(Variant::Outline)
                        .on_press(demo(DemoMessage::AlertDialog(true))),
                    d.alert_dialog,
                )
                .title("Are you absolutely sure?")
                .description("This action cannot be undone. This will permanently delete your account from our servers.")
                .cancel("Cancel", demo(DemoMessage::AlertDialog(false)))
                .action("Continue", demo(DemoMessage::AlertDialog(false)))
            ),
            example(
                "Small with media",
                alert_dialog(
                    button("Show Small Dialog")
                        .variant(Variant::Outline)
                        .on_press(demo(DemoMessage::SmallAlertDialog(true))),
                    d.small_alert_dialog,
                )
                .size(ad::Size::Sm)
                .media(lucide::BLUETOOTH)
                .title("Allow accessory to connect?")
                .description("Do you want to allow the USB accessory to connect to this device?")
                .cancel("Don't allow", demo(DemoMessage::SmallAlertDialog(false)))
                .action("Allow", demo(DemoMessage::SmallAlertDialog(false)))
            ),
        ]
        .into()
    },
};

/// The "Edit profile" sheet of the Sheet page, around its `trigger`.
fn profile_sheet<'a>(
    gallery: &'a crate::Gallery,
    trigger: impl Into<iced::Element<'a, Message>>,
    open: bool,
    side: sh::Side,
    close: Message,
) -> iced::Element<'a, Message> {
    sheet(trigger, open)
        .side(side)
        .title("Edit profile")
        .description("Make changes to your profile here. Click save when you're done.")
        // `grid gap-6 px-4`
        .content(profile_fields(gallery, 24))
        .footer(
            column![
                button("Save changes").width(Fill).on_press(close.clone()),
                button("Close")
                    .variant(Variant::Outline)
                    .width(Fill)
                    .on_press(close.clone()),
            ]
            .spacing(8),
        )
        .on_close(close)
        .into()
}

pub const SHEET: Page = Page {
    id: "sheet",
    title: "Sheet",
    description: "Extends the Dialog component to display content that complements the main content of the screen.",
    view: |gallery| {
        let d = &gallery.demo;
        let open = |side| demo(DemoMessage::Sheet(Some(side)));

        column![
            example(
                "Default",
                profile_sheet(
                    gallery,
                    button("Open")
                        .variant(Variant::Outline)
                        .on_press(demo(DemoMessage::DefaultSheet(true))),
                    d.default_sheet,
                    sh::Side::Right,
                    demo(DemoMessage::DefaultSheet(false)),
                )
            ),
            example(
                "Side",
                profile_sheet(
                    gallery,
                    row![
                        button("top")
                            .variant(Variant::Outline)
                            .on_press(open(sh::Side::Top)),
                        button("right")
                            .variant(Variant::Outline)
                            .on_press(open(sh::Side::Right)),
                        button("bottom")
                            .variant(Variant::Outline)
                            .on_press(open(sh::Side::Bottom)),
                        button("left")
                            .variant(Variant::Outline)
                            .on_press(open(sh::Side::Left)),
                    ]
                    .spacing(8),
                    d.sheet.is_some(),
                    d.sheet_side,
                    demo(DemoMessage::Sheet(None)),
                )
            ),
        ]
        .into()
    },
};

const DELIVERY_TIMES: [(&str, &str, Option<&str>); 5] = [
    (
        "Standard delivery",
        "25–35 min · Driver assigned now",
        Some("Fastest"),
    ),
    ("5:00 PM – 5:15 PM", "Prep starts at 4:45 PM", None),
    ("5:30 PM – 5:45 PM", "Good if you're heading home", None),
    ("6:00 PM – 6:15 PM", "Most popular · High demand", None),
    ("6:30 PM – 6:45 PM", "Last slot before kitchen closes", None),
];

/// A delivery time: a `FieldLabel` card holding a horizontal `Field` with a
/// radio item.
fn delivery_time<'a>(index: u8, selected: bool) -> iced::Element<'a, Message> {
    let (label, description, badge_label) = DELIVERY_TIMES[index as usize];

    let title = row![text(label).size(14).font(shadcn_iced::font::medium())]
        .push(badge_label.map(|label| badge(label).variant(shadcn_iced::badge::Variant::Secondary)))
        .spacing(8)
        .align_y(Center);

    let content = row![
        column![title, text(description).size(14).style(crate::muted)]
            .spacing(4)
            .width(Fill),
        shadcn_iced::radio_group::radio_item(
            selected,
            "",
            Some(demo(DemoMessage::Position(index)))
        ),
    ]
    .spacing(12)
    .align_y(Center);

    shadcn_iced::primitive::Pressable::new(content)
        .on_press(demo(DemoMessage::Position(index)))
        .padding(10)
        .width(Fill)
        .style(move |theme: &iced::Theme, status| {
            let t = shadcn_iced::tokens(theme);
            let p = t.palette;
            let hovered = matches!(
                status,
                iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed
            );
            let dark = t.is_dark();

            iced::widget::button::Style {
                background: Some(iced::Background::Color(if selected {
                    shadcn_iced::theme::alpha(p.primary, if dark { 0.1 } else { 0.05 })
                } else if hovered {
                    shadcn_iced::theme::alpha(p.muted, 0.5)
                } else {
                    iced::Color::TRANSPARENT
                })),
                text_color: p.foreground,
                border: shadcn_iced::style::border(
                    if selected {
                        shadcn_iced::theme::alpha(p.primary, if dark { 0.2 } else { 0.3 })
                    } else {
                        p.border
                    },
                    t.radius_lg(),
                ),
                ..Default::default()
            }
        })
        .into()
}

pub const DRAWER: Page = Page {
    id: "drawer",
    title: "Drawer",
    description: "A panel that slides in from an edge of the window.",
    view: |gallery| {
        let d = &gallery.demo;
        let selected = d.position.min(4);

        example(
            "Default",
            drawer(
                button("Open Drawer")
                    .variant(Variant::Secondary)
                    .on_press(demo(DemoMessage::Drawer(true))),
                d.drawer,
            )
            // `swipeDirection={isMobile ? "down" : "right"}`
            .direction(dr::Direction::Right)
            .title("Pick a delivery time")
            .description("We'll prepare your order as soon as possible.")
            .content(
                iced::widget::Column::with_children(
                    (0..5).map(|index| delivery_time(index, index == selected)),
                )
                .spacing(8),
            )
            .footer(
                column![
                    button("Confirm Delivery Time")
                        .width(Fill)
                        .on_press(demo(DemoMessage::Drawer(false))),
                    button("Cancel")
                        .variant(Variant::Outline)
                        .width(Fill)
                        .on_press(demo(DemoMessage::Drawer(false))),
                ]
                .spacing(8),
            )
            .on_close(demo(DemoMessage::Drawer(false))),
        )
    },
};

pub const POPOVER: Page = Page {
    id: "popover",
    title: "Popover",
    description: "Displays rich content in a portal, triggered by a button.",
    view: |_| {
        // `grid grid-cols-3 items-center gap-4`, inputs `col-span-2 h-8`.
        let dimension = |label: &'static str, value: &str| {
            row![
                text(label)
                    .size(14)
                    .font(shadcn_iced::font::medium())
                    .width(Fill),
                container(input("", value).on_input(|_| Message::Noop))
                    .width(iced::Length::FillPortion(2)),
            ]
            .spacing(16)
            .align_y(Center)
        };

        let aligned = |label: &'static str, content: &'static str, align: Align| {
            pop::popover_with_width(
                button(label).variant(Variant::Outline).size(Size::Sm),
                text(content).size(14),
                160.0,
            )
            .align(align)
        };

        column![
            example(
                "Default",
                pop::popover_with_width(
                    button("Open popover").variant(Variant::Outline),
                    column![
                        column![
                            text("Dimensions")
                                .size(14)
                                .font(shadcn_iced::font::medium())
                                .line_height(1.0),
                            text("Set the dimensions for the layer.")
                                .size(14)
                                .style(crate::muted),
                        ]
                        .spacing(8),
                        column![
                            dimension("Width", "100%"),
                            dimension("Max. width", "300px"),
                            dimension("Height", "25px"),
                            dimension("Max. height", "none"),
                        ]
                        .spacing(8),
                    ]
                    .spacing(16),
                    320.0,
                ),
            ),
            example(
                "Alignments",
                row![
                    aligned("Start", "Aligned to start", Align::Start),
                    aligned("Center", "Aligned to center", Align::Center),
                    aligned("End", "Aligned to end", Align::End),
                ]
                .spacing(24),
            ),
        ]
        .into()
    },
};

pub const TOOLTIP: Page = Page {
    id: "tooltip",
    title: "Tooltip",
    description: "A popup that displays information related to an element when the element receives keyboard focus or the mouse hovers over it.",
    view: |_| {
        let side = |label: &'static str, side: Side| {
            tooltip(button(label).variant(Variant::Outline), "Add to library").side(side)
        };

        column![
            example(
                "Default",
                tooltip(button("Hover").variant(Variant::Outline), "Add to library"),
            ),
            example(
                "Sides",
                row![
                    side("left", Side::Left),
                    side("top", Side::Top),
                    side("bottom", Side::Bottom),
                    side("right", Side::Right),
                ]
                .spacing(8),
            ),
        ]
        .into()
    },
};

pub const HOVER_CARD: Page = Page {
    id: "hover-card",
    title: "Hover Card",
    description: "For sighted users to preview content available behind a link.",
    view: |_| {
        example(
            "Default",
            hover_card(
                button("Hover Here").variant(Variant::Link),
                // `flex flex-col gap-0.5`
                column![
                    text("@nextjs").size(14).font(shadcn_iced::font::semibold()),
                    text("The React Framework – created and maintained by @vercel.").size(14),
                    container(text("Joined December 2021").size(12).style(crate::muted))
                        .padding(iced::Padding::ZERO.top(4)),
                ]
                .spacing(2),
            )
            .hover(Duration::from_millis(10), Duration::from_millis(100)),
        )
    },
};

pub const DROPDOWN_MENU: Page = Page {
    id: "dropdown-menu",
    title: "Dropdown Menu",
    description: "Displays a menu to the user — such as a set of actions or functions — triggered by a button.",
    view: |gallery| {
        let d = &gallery.demo;

        column![
            example(
                "Default",
                dropdown_menu(
                    button("Open").variant(Variant::Outline),
                    menu()
                        .label("My Account")
                        .item(item("Profile").shortcut("⇧⌘P").on_select(Message::Noop))
                        .item(item("Billing").shortcut("⌘B").on_select(Message::Noop))
                        .item(item("Settings").shortcut("⌘S").on_select(Message::Noop))
                        .separator()
                        .item(item("Team").on_select(Message::Noop))
                        .sub(
                            "Invite users",
                            menu()
                                .item(item("Email").on_select(Message::Noop))
                                .item(item("Message").on_select(Message::Noop))
                                .separator()
                                .item(item("More...").on_select(Message::Noop)),
                        )
                        .item(item("New Team").shortcut("⌘+T").on_select(Message::Noop))
                        .separator()
                        .item(item("GitHub").on_select(Message::Noop))
                        .item(item("Support").on_select(Message::Noop))
                        .item(item("API"))
                        .separator()
                        .item(item("Log out").shortcut("⇧⌘Q").on_select(Message::Noop))
                        // `w-40`
                        .min_width(160.0),
                ),
            ),
            example(
                "Checkboxes",
                dropdown_menu(
                    button("Checkboxes").variant(Variant::Outline),
                    menu()
                        .label("Appearance")
                        .checkbox(
                            "Status Bar",
                            d.status_bar,
                            demo(DemoMessage::StatusBar(!d.status_bar)),
                        )
                        .checkbox_maybe("Activity Bar", false, None)
                        .checkbox("Panel", false, Message::Noop)
                        .min_width(160.0),
                ),
            ),
            example(
                "Radio Group",
                dropdown_menu(
                    button("Radio Group").variant(Variant::Outline),
                    menu()
                        .label("Panel Position")
                        .radio("Top", d.position == 0, demo(DemoMessage::Position(0)))
                        .radio("Bottom", d.position == 1, demo(DemoMessage::Position(1)))
                        .radio("Right", d.position == 2, demo(DemoMessage::Position(2)))
                        .min_width(128.0),
                ),
            ),
            example(
                "Destructive",
                dropdown_menu(
                    button("Destructive").variant(Variant::Outline),
                    menu()
                        .item(item("Edit").icon(lucide::PENCIL).on_select(Message::Noop))
                        .item(item("Share").icon(lucide::SHARE).on_select(Message::Noop))
                        .separator()
                        .item(
                            item("Delete")
                                .icon(lucide::TRASH)
                                .destructive()
                                .on_select(Message::Noop),
                        ),
                ),
            ),
        ]
        .into()
    },
};

pub const CONTEXT_MENU: Page = Page {
    id: "context-menu",
    title: "Context Menu",
    description: "Displays a menu to the user — such as a set of actions or functions — triggered by a right click.",
    view: |gallery| {
        let d = &gallery.demo;

        example(
            "Default",
            context_menu(
                // `aspect-video w-full max-w-xs rounded-xl border border-dashed
                // text-sm`
                container(text("Right click here").size(14))
                    .center_x(320)
                    .center_y(180)
                    .style(|theme: &iced::Theme| {
                        let t = shadcn_iced::tokens(theme);
                        container::Style {
                            border: shadcn_iced::style::border(t.palette.border, t.radius_xl()),
                            ..Default::default()
                        }
                    }),
                menu()
                    .item(item("Back").shortcut("⌘[").on_select(Message::Noop))
                    .item(item("Forward").shortcut("⌘]"))
                    .item(item("Reload").shortcut("⌘R").on_select(Message::Noop))
                    .sub(
                        "More Tools",
                        menu()
                            .item(item("Save Page...").on_select(Message::Noop))
                            .item(item("Create Shortcut...").on_select(Message::Noop))
                            .item(item("Name Window...").on_select(Message::Noop))
                            .separator()
                            .item(item("Developer Tools").on_select(Message::Noop))
                            .separator()
                            .item(item("Delete").destructive().on_select(Message::Noop))
                            // `w-44`
                            .min_width(176.0),
                    )
                    .separator()
                    .checkbox(
                        "Show Bookmarks",
                        d.status_bar,
                        demo(DemoMessage::StatusBar(!d.status_bar)),
                    )
                    .checkbox("Show Full URLs", false, Message::Noop)
                    .separator()
                    .label("People")
                    .radio(
                        "Pedro Duarte",
                        d.position != 1,
                        demo(DemoMessage::Position(0)),
                    )
                    .radio(
                        "Colm Tuite",
                        d.position == 1,
                        demo(DemoMessage::Position(1)),
                    )
                    // `w-48`
                    .min_width(192.0),
            ),
        )
    },
};

pub const MENUBAR: Page = Page {
    id: "menubar",
    title: "Menubar",
    description: "A visually persistent menu common in desktop applications that provides quick access to a consistent set of commands.",
    view: |gallery| {
        let d = &gallery.demo;

        example(
            "Default",
            menubar()
                .menu(
                    "File",
                    menu()
                        .item(item("New Tab").shortcut("⌘T").on_select(Message::Noop))
                        .item(item("New Window").shortcut("⌘N").on_select(Message::Noop))
                        .item(item("New Incognito Window"))
                        .separator()
                        .sub(
                            "Share",
                            menu()
                                .item(item("Email link").on_select(Message::Noop))
                                .item(item("Messages").on_select(Message::Noop))
                                .item(item("Notes").on_select(Message::Noop)),
                        )
                        .separator()
                        .item(item("Print...").shortcut("⌘P").on_select(Message::Noop)),
                )
                .menu(
                    "Edit",
                    menu()
                        .item(item("Undo").shortcut("⌘Z").on_select(Message::Noop))
                        .item(item("Redo").shortcut("⇧⌘Z").on_select(Message::Noop))
                        .separator()
                        .sub(
                            "Find",
                            menu()
                                .item(item("Search the web").on_select(Message::Noop))
                                .separator()
                                .item(item("Find...").on_select(Message::Noop))
                                .item(item("Find Next").on_select(Message::Noop))
                                .item(item("Find Previous").on_select(Message::Noop)),
                        )
                        .separator()
                        .item(item("Cut").on_select(Message::Noop))
                        .item(item("Copy").on_select(Message::Noop))
                        .item(item("Paste").on_select(Message::Noop)),
                )
                .menu(
                    "View",
                    menu()
                        .checkbox(
                            "Bookmarks Bar",
                            d.status_bar,
                            demo(DemoMessage::StatusBar(!d.status_bar)),
                        )
                        .checkbox("Full URLs", true, Message::Noop)
                        .separator()
                        .item(
                            item("Reload")
                                .inset()
                                .shortcut("⌘R")
                                .on_select(Message::Noop),
                        )
                        .item(item("Force Reload").inset().shortcut("⇧⌘R"))
                        .separator()
                        .item(item("Toggle Fullscreen").inset().on_select(Message::Noop))
                        .separator()
                        .item(item("Hide Sidebar").inset().on_select(Message::Noop))
                        // `w-44`
                        .min_width(176.0),
                )
                .menu(
                    "Profiles",
                    menu()
                        .radio("Andy", d.position == 0, demo(DemoMessage::Position(0)))
                        .radio("Benoit", d.position == 1, demo(DemoMessage::Position(1)))
                        .radio("Luis", d.position == 2, demo(DemoMessage::Position(2)))
                        .separator()
                        .item(item("Edit...").inset().on_select(Message::Noop))
                        .separator()
                        .item(item("Add Profile...").inset().on_select(Message::Noop)),
                ),
        )
    },
};

pub const SELECT: Page = Page {
    id: "select",
    title: "Select",
    description: "Displays a list of options for the user to pick from—triggered by a button.",
    view: |gallery| {
        let fruit = gallery.demo.fruit;

        column![
            example(
                "Default",
                select(fruit, |f| demo(DemoMessage::Fruit(f)))
                    .placeholder("Select a fruit")
                    .label("Fruits")
                    .option("Apple", "Apple")
                    .option("Banana", "Banana")
                    .option("Blueberry", "Blueberry")
                    .option("Grapes", "Grapes")
                    .option("Pineapple", "Pineapple")
                    // `w-full max-w-48`
                    .width(192),
            ),
            example(
                "Align Item With Trigger off",
                select(fruit, |f| demo(DemoMessage::Fruit(f)))
                    .placeholder("Select a fruit")
                    .option("Apple", "Apple")
                    .option("Banana", "Banana")
                    .option("Blueberry", "Blueberry")
                    .separator()
                    .option("Grapes", "Grapes")
                    .disabled_option("Pineapple", "Pineapple")
                    .align_item_with_trigger(false)
                    .width(192),
            ),
        ]
        .into()
    },
};

pub const COMBOBOX: Page = Page {
    id: "combobox",
    title: "Combobox",
    description: "Autocomplete input with a list of suggestions.",
    view: |gallery| {
        let d = &gallery.demo;

        example(
            "Default",
            combobox(
                d.framework,
                |f| demo(DemoMessage::Framework(f)),
                &d.query,
                |q| demo(DemoMessage::Query(q)),
            )
            .placeholder("Select a framework")
            .empty("No items found.")
            .option("Next.js", "Next.js")
            .option("SvelteKit", "SvelteKit")
            .option("Nuxt.js", "Nuxt.js")
            .option("Remix", "Remix")
            .option("Astro", "Astro"),
        )
    },
};

pub const COMMAND: Page = Page {
    id: "command",
    title: "Command",
    description: "Fast, composable, unstyled command menu.",
    view: |gallery| {
        example(
            "Default",
            container(
                command(&gallery.demo.query, |q| demo(DemoMessage::Query(q)))
                    .group(
                        "Suggestions",
                        [
                            command_item("Calendar")
                                .icon(lucide::CALENDAR)
                                .on_select(Message::Noop),
                            command_item("Search Emoji")
                                .icon(lucide::SMILE)
                                .on_select(Message::Noop),
                            command_item("Calculator").icon(lucide::CALCULATOR),
                        ],
                    )
                    .group(
                        "Settings",
                        [
                            command_item("Profile")
                                .icon(lucide::USER)
                                .shortcut("⌘P")
                                .on_select(Message::Noop),
                            command_item("Billing")
                                .icon(lucide::CREDIT_CARD)
                                .shortcut("⌘B")
                                .on_select(Message::Noop),
                            command_item("Settings")
                                .icon(lucide::SETTINGS)
                                .shortcut("⌘S")
                                .on_select(Message::Noop),
                        ],
                    ),
            )
            // `max-w-sm`
            .max_width(384),
        )
    },
};

pub const SONNER: Page = Page {
    id: "sonner",
    title: "Sonner",
    description: "An opinionated toast component.",
    view: |_| {
        example(
            "Types",
            row![
                button("Default")
                    .variant(Variant::Outline)
                    .on_press(demo(DemoMessage::Toast(Kind::Default))),
                button("Success")
                    .variant(Variant::Outline)
                    .on_press(demo(DemoMessage::Toast(Kind::Success))),
                button("Info")
                    .variant(Variant::Outline)
                    .on_press(demo(DemoMessage::Toast(Kind::Info))),
                button("Warning")
                    .variant(Variant::Outline)
                    .on_press(demo(DemoMessage::Toast(Kind::Warning))),
                button("Error")
                    .variant(Variant::Outline)
                    .on_press(demo(DemoMessage::Toast(Kind::Error))),
                button("Loading")
                    .variant(Variant::Outline)
                    .on_press(demo(DemoMessage::Toast(Kind::Loading))),
            ]
            .spacing(8)
            .wrap(),
        )
    },
};

pub const NAVIGATION_MENU: Page = Page {
    id: "navigation-menu",
    title: "Navigation Menu",
    description: "A collection of links for navigating websites.",
    view: |_| {
        use shadcn_iced::navigation_menu::{icon_link, link};

        let components = [
            (
                "Alert Dialog",
                "A modal dialog that interrupts the user with important content and expects a response.",
            ),
            (
                "Hover Card",
                "For sighted users to preview content available behind a link.",
            ),
            (
                "Progress",
                "Displays an indicator showing the completion progress of a task, typically displayed as a progress bar.",
            ),
            ("Scroll-area", "Visually or semantically separates content."),
            (
                "Tabs",
                "A set of layered sections of content—known as tab panels—that are displayed one at a time.",
            ),
            (
                "Tooltip",
                "A popup that displays information related to an element when the element receives keyboard focus or the mouse hovers over it.",
            ),
        ];

        // `grid w-[500px] gap-2 md:grid-cols-2`
        let grid = iced::widget::Column::with_children(components.chunks(2).map(|pair| {
            iced::widget::Row::with_children(
                pair.iter()
                    .map(|(title, description)| link(*title, *description, Message::Noop)),
            )
            .spacing(8)
            .into()
        }))
        .spacing(8)
        .width(500);

        example(
            "Default",
            container(
                navigation_menu()
                    .menu(
                        "Getting started",
                        // `w-96`
                        column![
                            link(
                                "Introduction",
                                "Re-usable components built with Tailwind CSS.",
                                Message::Noop
                            ),
                            link(
                                "Installation",
                                "How to install dependencies and structure your app.",
                                Message::Noop
                            ),
                            link(
                                "Typography",
                                "Styles for headings, paragraphs, lists...etc",
                                Message::Noop
                            ),
                        ]
                        .width(384),
                    )
                    .menu("Components", grid)
                    .menu(
                        "With Icon",
                        // `w-[200px]`
                        column![
                            icon_link(lucide::CIRCLE_ALERT, "Backlog", Message::Noop),
                            icon_link(lucide::CIRCLE_DASHED, "To Do", Message::Noop),
                            icon_link(lucide::CIRCLE_CHECK, "Done", Message::Noop),
                        ]
                        .width(200),
                    )
                    .link("Docs", Message::Noop),
            )
            .padding(iced::Padding::ZERO.bottom(260)),
        )
    },
};
