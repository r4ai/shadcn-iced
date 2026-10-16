//! Pages of the overlay components.
use super::{DemoMessage, Page};
use crate::{Message, example};

use iced_ui::button::{Variant, button};
use iced_ui::command::item as command_item;
use iced_ui::menu::item;
use iced_ui::prelude::*;
use iced_ui::sonner::Kind;
use iced_ui::{avatar, popover as pop, sheet as sh};

use iced::widget::{column, container, row, text};
use iced::{Center, Fill};

fn demo(message: DemoMessage) -> Message {
    Message::Demo(message)
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
                button("Edit Profile").variant(Variant::Outline).on_press(demo(DemoMessage::Dialog(true))),
                d.dialog,
            )
            .title("Edit profile")
            .description("Make changes to your profile here. Click save when you're done.")
            .content(
                column![
                    iced_ui::field::field("Name", input("Pedro Duarte", &d.name).on_input(|v| demo(DemoMessage::Name(v)))),
                    iced_ui::field::field("Username", input("@peduarte", &d.username).on_input(|v| demo(DemoMessage::Username(v)))),
                ]
                .spacing(16),
            )
            .footer(
                row![
                    button("Cancel").variant(Variant::Outline).on_press(demo(DemoMessage::Dialog(false))),
                    button("Save changes").on_press(demo(DemoMessage::Dialog(false))),
                ]
                .spacing(8),
            )
            .width(425.0)
            .on_close(demo(DemoMessage::Dialog(false))),
        )
    },
};

pub const ALERT_DIALOG: Page = Page {
    id: "alert-dialog",
    title: "Alert Dialog",
    description: "A modal dialog that interrupts the user with important content and expects a response.",
    view: |gallery| {
        example(
            "Default",
            alert_dialog(
                button("Show Dialog").variant(Variant::Outline).on_press(demo(DemoMessage::AlertDialog(true))),
                gallery.demo.alert_dialog,
            )
            .title("Are you absolutely sure?")
            .description("This action cannot be undone. This will permanently delete your account and remove your data from our servers.")
            .cancel("Cancel", demo(DemoMessage::AlertDialog(false)))
            .action("Continue", demo(DemoMessage::AlertDialog(false))),
        )
    },
};

pub const SHEET: Page = Page {
    id: "sheet",
    title: "Sheet",
    description: "Extends the Dialog component to display content that complements the main content of the screen.",
    view: |gallery| {
        let d = &gallery.demo;

        let open = |side| demo(DemoMessage::Sheet(Some(side)));

        example(
            "Sides",
            sheet(
                row![
                    button("Top").variant(Variant::Outline).on_press(open(sh::Side::Top)),
                    button("Right").variant(Variant::Outline).on_press(open(sh::Side::Right)),
                    button("Bottom").variant(Variant::Outline).on_press(open(sh::Side::Bottom)),
                    button("Left").variant(Variant::Outline).on_press(open(sh::Side::Left)),
                ]
                .spacing(8),
                d.sheet.is_some(),
            )
            .side(d.sheet.unwrap_or_default())
            .title("Edit profile")
            .description("Make changes to your profile here. Click save when you're done.")
            .content(
                column![
                    iced_ui::field::field("Name", input("Pedro Duarte", &d.name).on_input(|v| demo(DemoMessage::Name(v)))),
                    iced_ui::field::field("Username", input("@peduarte", &d.username).on_input(|v| demo(DemoMessage::Username(v)))),
                ]
                .spacing(24),
            )
            .footer(
                column![
                    button("Save changes").width(Fill).on_press(demo(DemoMessage::Sheet(None))),
                    button("Close").variant(Variant::Outline).width(Fill).on_press(demo(DemoMessage::Sheet(None))),
                ]
                .spacing(8),
            )
            .on_close(demo(DemoMessage::Sheet(None))),
        )
    },
};

pub const DRAWER: Page = Page {
    id: "drawer",
    title: "Drawer",
    description: "A drawer component for iced.",
    view: |gallery| {
        let goal = gallery.demo.goal;

        example(
            "Default",
            drawer(
                button("Open Drawer").variant(Variant::Outline).on_press(demo(DemoMessage::Drawer(true))),
                gallery.demo.drawer,
            )
            .title("Move Goal")
            .description("Set your daily activity goal.")
            .content(
                row![
                    iced_ui::button::icon_button(lucide::MINUS)
                        .variant(Variant::Outline)
                        .size(iced_ui::button::Size::IconSm)
                        .on_press(demo(DemoMessage::Goal(-10))),
                    column![
                        text(goal.to_string()).size(72).font(iced_ui::font::BOLD),
                        text("CALORIES/DAY").size(12).style(crate::muted),
                    ]
                    .align_x(Center)
                    .width(Fill),
                    iced_ui::button::icon_button(lucide::PLUS)
                        .variant(Variant::Outline)
                        .size(iced_ui::button::Size::IconSm)
                        .on_press(demo(DemoMessage::Goal(10))),
                ]
                .align_y(Center)
                .padding([16, 0]),
            )
            .footer(
                column![
                    button("Submit").width(Fill).on_press(demo(DemoMessage::Drawer(false))),
                    button("Cancel").variant(Variant::Outline).width(Fill).on_press(demo(DemoMessage::Drawer(false))),
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
        let dimension = |label: &'static str, value: &str| {
            row![
                text(label).size(14).width(96),
                input("", value).on_input(|_| Message::Noop),
            ]
            .spacing(16)
            .align_y(Center)
        };

        example(
            "Default",
            popover(
                button("Open popover").variant(Variant::Outline),
                column![
                    pop::header("Dimensions", "Set the dimensions for the layer."),
                    column![
                        dimension("Width", "100%"),
                        dimension("Max. width", "300px"),
                        dimension("Height", "25px"),
                        dimension("Max. height", "none"),
                    ]
                    .spacing(8),
                ]
                .spacing(16),
            )
        )
    },
};

pub const TOOLTIP: Page = Page {
    id: "tooltip",
    title: "Tooltip",
    description: "A popup that displays information related to an element when the element receives keyboard focus or the mouse hovers over it.",
    view: |_| {
        example(
            "Sides",
            row![
                tooltip(button("Top").variant(Variant::Outline), "Add to library"),
                tooltip(button("Bottom").variant(Variant::Outline), "Add to library")
                    .side(iced_ui::primitive::Side::Bottom),
                tooltip(button("Left").variant(Variant::Outline), "Add to library")
                    .side(iced_ui::primitive::Side::Left),
                tooltip(button("Right").variant(Variant::Outline), "Add to library")
                    .side(iced_ui::primitive::Side::Right),
            ]
            .spacing(8),
        )
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
                button("@nextjs").variant(Variant::Link),
                row![
                    avatar("VC").size(avatar::Size::Lg),
                    column![
                        text("@nextjs").size(14).font(iced_ui::font::SEMIBOLD),
                        text("The React Framework – created and maintained by @vercel.").size(14),
                        text("Joined December 2021").size(12).style(crate::muted),
                    ]
                    .spacing(4),
                ]
                .spacing(16),
            ),
        )
    },
};

fn account_menu<'a>(gallery: &crate::Gallery) -> iced_ui::menu::Menu<'a, Message> {
    let d = &gallery.demo;

    menu()
        .label("My Account")
        .separator()
        .item(item("Profile").icon(lucide::USER).shortcut("⇧⌘P").on_select(Message::Noop))
        .item(item("Billing").icon(lucide::CREDIT_CARD).shortcut("⌘B").on_select(Message::Noop))
        .item(item("Settings").icon(lucide::SETTINGS).shortcut("⌘S").on_select(Message::Noop))
        .item(item("Keyboard shortcuts").icon(lucide::KEYBOARD).shortcut("⌘K").on_select(Message::Noop))
        .separator()
        .item(item("Team").icon(lucide::USERS).on_select(Message::Noop))
        .sub_with_icon(
            lucide::USER_PLUS,
            "Invite users",
            menu()
                .item(item("Email").icon(lucide::MAIL).on_select(Message::Noop))
                .item(item("Message").icon(lucide::MESSAGE_SQUARE).on_select(Message::Noop))
                .separator()
                .item(item("More...").icon(lucide::CIRCLE_PLUS).on_select(Message::Noop)),
        )
        .item(item("New Team").icon(lucide::PLUS).shortcut("⌘+T").on_select(Message::Noop))
        .separator()
        .checkbox("Status Bar", d.status_bar, demo(DemoMessage::StatusBar(!d.status_bar)))
        .item(item("API").icon(lucide::CLOUD))
        .separator()
        .item(item("Log out").icon(lucide::LOG_OUT).shortcut("⇧⌘Q").destructive().on_select(Message::Noop))
}

pub const DROPDOWN_MENU: Page = Page {
    id: "dropdown-menu",
    title: "Dropdown Menu",
    description: "Displays a menu to the user — such as a set of actions or functions — triggered by a button.",
    view: |gallery| {
        let d = &gallery.demo;

        column![
            example("Default", dropdown_menu(button("Open").variant(Variant::Outline), account_menu(gallery).min_width(224.0))),
            example(
                "Radio group",
                dropdown_menu(
                    button("Panel position").variant(Variant::Outline),
                    menu()
                        .label("Panel Position")
                        .separator()
                        .radio("Top", d.position == 0, demo(DemoMessage::Position(0)))
                        .radio("Bottom", d.position == 1, demo(DemoMessage::Position(1)))
                        .radio("Right", d.position == 2, demo(DemoMessage::Position(2)))
                        .min_width(224.0),
                )
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
                container(text("Right click here").size(14))
                    .center_x(300)
                    .center_y(150)
                    .style(|theme: &iced::Theme| {
                        let t = iced_ui::tokens(theme);
                        container::Style {
                            border: iced_ui::style::border(t.palette.border, t.radius_md()),
                            ..Default::default()
                        }
                    }),
                menu()
                    .item(item("Back").inset().shortcut("⌘[").on_select(Message::Noop))
                    .item(item("Forward").inset().shortcut("⌘]"))
                    .item(item("Reload").inset().shortcut("⌘R").on_select(Message::Noop))
                    .sub(
                        "More Tools",
                        menu()
                            .item(item("Save Page...").on_select(Message::Noop))
                            .item(item("Create Shortcut...").on_select(Message::Noop))
                            .separator()
                            .item(item("Developer Tools").on_select(Message::Noop))
                            .separator()
                            .item(item("Delete").destructive().on_select(Message::Noop)),
                    )
                    .separator()
                    .checkbox("Show Bookmarks", d.status_bar, demo(DemoMessage::StatusBar(!d.status_bar)))
                    .checkbox("Show Full URLs", false, Message::Noop)
                    .separator()
                    .inset_label("People")
                    .radio("Pedro Duarte", d.position == 0, demo(DemoMessage::Position(0)))
                    .radio("Colm Tuite", d.position == 1, demo(DemoMessage::Position(1)))
                    .min_width(208.0),
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
                        .item(item("Cut").on_select(Message::Noop))
                        .item(item("Copy").on_select(Message::Noop))
                        .item(item("Paste").on_select(Message::Noop)),
                )
                .menu(
                    "View",
                    menu()
                        .checkbox("Always Show Bookmarks Bar", d.status_bar, demo(DemoMessage::StatusBar(!d.status_bar)))
                        .checkbox("Always Show Full URLs", true, Message::Noop)
                        .separator()
                        .item(item("Reload").inset().shortcut("⌘R").on_select(Message::Noop))
                        .item(item("Force Reload").inset().shortcut("⇧⌘R"))
                        .separator()
                        .item(item("Toggle Fullscreen").inset().on_select(Message::Noop)),
                )
                .menu(
                    "Profiles",
                    menu()
                        .radio("Andy", d.position == 0, demo(DemoMessage::Position(0)))
                        .radio("Benoit", d.position == 1, demo(DemoMessage::Position(1)))
                        .radio("Luis", d.position == 2, demo(DemoMessage::Position(2)))
                        .separator()
                        .item(item("Edit...").inset().on_select(Message::Noop)),
                ),
        )
    },
};

pub const SELECT: Page = Page {
    id: "select",
    title: "Select",
    description: "Displays a list of options for the user to pick from—triggered by a button.",
    view: |gallery| {
        example(
            "Default",
            select(gallery.demo.fruit, |f| demo(DemoMessage::Fruit(f)))
                .placeholder("Select a fruit")
                .label("Fruits")
                .option("Apple", "Apple")
                .option("Banana", "Banana")
                .option("Blueberry", "Blueberry")
                .option("Grapes", "Grapes")
                .disabled_option("Pineapple", "Pineapple")
                .width(180),
        )
    },
};

pub const COMBOBOX: Page = Page {
    id: "combobox",
    title: "Combobox",
    description: "Autocomplete input and command palette with a list of suggestions.",
    view: |gallery| {
        let d = &gallery.demo;

        example(
            "Default",
            combobox(d.framework, |f| demo(DemoMessage::Framework(f)), &d.query, |q| demo(DemoMessage::Query(q)))
                .placeholder("Select framework...")
                .search_placeholder("Search framework...")
                .empty("No framework found.")
                .option("next", "Next.js")
                .option("sveltekit", "SvelteKit")
                .option("nuxt", "Nuxt.js")
                .option("remix", "Remix")
                .option("astro", "Astro"),
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
                            command_item("Calendar").icon(lucide::CALENDAR).on_select(Message::Noop),
                            command_item("Search Emoji").icon(lucide::SMILE).on_select(Message::Noop),
                            command_item("Calculator").icon(lucide::CALCULATOR),
                        ],
                    )
                    .group(
                        "Settings",
                        [
                            command_item("Profile").icon(lucide::USER).shortcut("⌘P").on_select(Message::Noop),
                            command_item("Billing").icon(lucide::CREDIT_CARD).shortcut("⌘B").on_select(Message::Noop),
                            command_item("Settings").icon(lucide::SETTINGS).shortcut("⌘S").on_select(Message::Noop),
                        ],
                    ),
            )
            .max_width(450),
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
                button("Default").variant(Variant::Outline).on_press(demo(DemoMessage::Toast(Kind::Default))),
                button("Success").variant(Variant::Outline).on_press(demo(DemoMessage::Toast(Kind::Success))),
                button("Info").variant(Variant::Outline).on_press(demo(DemoMessage::Toast(Kind::Info))),
                button("Warning").variant(Variant::Outline).on_press(demo(DemoMessage::Toast(Kind::Warning))),
                button("Error").variant(Variant::Outline).on_press(demo(DemoMessage::Toast(Kind::Error))),
                button("Loading").variant(Variant::Outline).on_press(demo(DemoMessage::Toast(Kind::Loading))),
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
        use iced_ui::navigation_menu::link;

        example(
            "Default",
            container(
                navigation_menu()
                    .menu(
                        "Getting started",
                        row![
                            container(
                                column![
                                    lucide::BOXES.sized(24),
                                    text("iced-ui").size(18).font(iced_ui::font::MEDIUM),
                                    text("Beautifully designed components built with iced.").size(14).style(crate::muted),
                                ]
                                .spacing(8)
                            )
                            .padding(16)
                            .width(180)
                            .height(Fill)
                            .style(|theme: &iced::Theme| {
                                let t = iced_ui::tokens(theme);
                                container::Style {
                                    background: Some(t.palette.muted.into()),
                                    border: iced_ui::style::rounded(t.radius_md()),
                                    ..Default::default()
                                }
                            }),
                            column![
                                link("Introduction", "Re-usable components built using iced.", Message::Noop),
                                link("Installation", "How to install dependencies and structure your app.", Message::Noop),
                                link("Typography", "Styles for headings, paragraphs, lists...etc", Message::Noop),
                            ]
                            .width(260),
                        ]
                        .spacing(8)
                        .height(220),
                    )
                    .menu(
                        "Components",
                        column![
                            link("Alert Dialog", "A modal dialog that interrupts the user.", Message::Noop),
                            link("Hover Card", "Preview content available behind a link.", Message::Noop),
                            link("Progress", "Displays the completion progress of a task.", Message::Noop),
                        ]
                        .width(300),
                    )
                    .link("Docs", Message::Noop),
            )
            .padding(iced::Padding::ZERO.bottom(240)),
        )
    },
};
