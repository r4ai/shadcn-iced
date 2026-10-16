//! Pages of the static components.
use super::{DemoMessage, Page};
use crate::{Message, example};

use iced_ui::prelude::*;
use iced_ui::{alert, avatar, badge, button, item, typography as ty};

use iced::widget::{column, container, row, space, text};
use iced::{Center, Fill};

fn demo(message: DemoMessage) -> Message {
    Message::Demo(message)
}

pub const BADGE: Page = Page {
    id: "badge",
    title: "Badge",
    description: "Displays a badge or a component that looks like a badge.",
    view: |_| {
        column![
            example(
                "Variants",
                row![
                    badge("Badge"),
                    badge("Secondary").variant(badge::Variant::Secondary),
                    badge("Destructive").variant(badge::Variant::Destructive),
                    badge("Outline").variant(badge::Variant::Outline),
                ]
                .spacing(8)
            ),
            example(
                "With icon",
                row![
                    badge("Verified")
                        .icon(lucide::BADGE_CHECK)
                        .variant(badge::Variant::Secondary),
                    badge("8").variant(badge::Variant::Destructive),
                    badge("99+").variant(badge::Variant::Outline),
                ]
                .spacing(8)
            ),
        ]
        .into()
    },
};

pub const CARD: Page = Page {
    id: "card",
    title: "Card",
    description: "Displays a card with header, content, and footer.",
    view: |gallery| {
        example(
            "Login",
            card()
                .title("Login to your account")
                .description("Enter your email below to login to your account")
                .action(
                    button("Sign Up")
                        .variant(button::Variant::Link)
                        .on_press(Message::Noop),
                )
                .content(
                    column![
                        column![
                            label("Email"),
                            input("m@example.com", &gallery.demo.email)
                                .on_input(|value| demo(DemoMessage::Email(value)))
                        ]
                        .spacing(8),
                        column![
                            row![
                                label("Password"),
                                space::horizontal(),
                                text("Forgot your password?").size(14)
                            ]
                            .align_y(Center),
                            input("", &gallery.demo.password)
                                .secure(true)
                                .on_input(|value| demo(DemoMessage::Password(value)))
                        ]
                        .spacing(8),
                    ]
                    .spacing(24),
                )
                .footer(
                    column![
                        button("Login").width(Fill).on_press(Message::Noop),
                        button("Login with Google")
                            .variant(button::Variant::Outline)
                            .width(Fill)
                            .on_press(Message::Noop),
                    ]
                    .spacing(8),
                )
                .width(384),
        )
    },
};

pub const INPUT: Page = Page {
    id: "input",
    title: "Input",
    description: "Displays a form input field or a component that looks like an input field.",
    view: |gallery| {
        column![
            example(
                "Default",
                container(
                    input("Email", &gallery.demo.email).on_input(|v| demo(DemoMessage::Email(v)))
                )
                .max_width(320)
            ),
            example(
                "With label and icon",
                container(
                    column![
                        label("Search"),
                        input("Search...", &gallery.demo.search)
                            .icon(lucide::SEARCH)
                            .on_input(|v| demo(DemoMessage::Search(v))),
                        text("Search across all components.")
                            .size(14)
                            .style(crate::muted),
                    ]
                    .spacing(8)
                )
                .max_width(320)
            ),
            example(
                "States",
                container(
                    column![
                        input::<Message>("Disabled", ""),
                        input("Invalid", "not an email")
                            .invalid(true)
                            .on_input(|_| Message::Noop),
                        row![
                            input("Email", &gallery.demo.email)
                                .on_input(|v| demo(DemoMessage::Email(v))),
                            button("Subscribe")
                                .variant(button::Variant::Outline)
                                .on_press(Message::Noop),
                        ]
                        .spacing(8),
                    ]
                    .spacing(12)
                )
                .max_width(360)
            ),
        ]
        .into()
    },
};

pub const TEXTAREA: Page = Page {
    id: "textarea",
    title: "Textarea",
    description: "Displays a form textarea or a component that looks like a textarea.",
    view: |gallery| {
        column![
            example(
                "Default",
                container(
                    textarea(&gallery.demo.message)
                        .placeholder("Type your message here.")
                        .on_action(|action| demo(DemoMessage::Edit(action)))
                )
                .max_width(420)
            ),
            example(
                "With label and button",
                container(
                    column![
                        label("Your message"),
                        textarea(&gallery.demo.message)
                            .placeholder("Type your message here.")
                            .on_action(|action| demo(DemoMessage::Edit(action))),
                        button("Send message").width(Fill).on_press(Message::Noop),
                    ]
                    .spacing(8)
                )
                .max_width(420)
            ),
        ]
        .into()
    },
};

pub const ALERT: Page = Page {
    id: "alert",
    title: "Alert",
    description: "Displays a callout for user attention.",
    view: |_| {
        example(
            "Default and destructive",
            column![
                alert("Success! Your changes have been saved")
                    .icon(lucide::CIRCLE_CHECK)
                    .description("This is an alert with icon, title and description."),
                alert("This Alert has a title and an icon. No description.").icon(lucide::POPCORN),
                alert("Unable to process your payment.")
                    .icon(lucide::CIRCLE_ALERT)
                    .variant(alert::Variant::Destructive)
                    .description("Please verify your billing information and try again."),
            ]
            .spacing(16)
            .max_width(560),
        )
    },
};

pub const SEPARATOR: Page = Page {
    id: "separator",
    title: "Separator",
    description: "Visually or semantically separates content.",
    view: |_| {
        example(
            "Default",
            column![
                column![
                    text("Radix Primitives")
                        .size(14)
                        .font(iced_ui::font::MEDIUM),
                    text("An open-source UI component library.")
                        .size(14)
                        .style(crate::muted),
                ]
                .spacing(4),
                separator(),
                row![
                    text("Blog").size(14),
                    vertical_separator(),
                    text("Docs").size(14),
                    vertical_separator(),
                    text("Source").size(14),
                ]
                .spacing(16)
                .height(20),
            ]
            .spacing(16)
            .max_width(320),
        )
    },
};

pub const SKELETON: Page = Page {
    id: "skeleton",
    title: "Skeleton",
    description: "Use to show a placeholder while content is loading.",
    view: |_| {
        column![
            example(
                "Default",
                row![
                    skeleton(48, 48).rounded_full(),
                    column![skeleton(250, 16), skeleton(200, 16)].spacing(8),
                ]
                .spacing(16)
                .align_y(Center)
            ),
            example(
                "Card",
                column![
                    skeleton(250, 125).radius(12.0),
                    column![skeleton(250, 16), skeleton(200, 16)].spacing(8),
                ]
                .spacing(12)
            ),
        ]
        .into()
    },
};

pub const SPINNER: Page = Page {
    id: "spinner",
    title: "Spinner",
    description: "An indicator that can be used to show a loading state.",
    view: |_| {
        column![
            example(
                "Sizes",
                row![
                    spinner().size(12.0),
                    spinner(),
                    spinner().size(24.0),
                    spinner().size(32.0)
                ]
                .spacing(24)
                .align_y(Center)
            ),
            example(
                "In a badge",
                row![
                    badge::Badge::new(
                        row![spinner().size(12.0), text("Syncing").size(12)]
                            .spacing(4)
                            .align_y(Center)
                    ),
                    badge::Badge::new(
                        row![spinner().size(12.0), text("Updating").size(12)]
                            .spacing(4)
                            .align_y(Center)
                    )
                    .variant(badge::Variant::Secondary),
                ]
                .spacing(8)
            ),
        ]
        .into()
    },
};

pub const KBD: Page = Page {
    id: "kbd",
    title: "Kbd",
    description: "Used to display textual user input from keyboard.",
    view: |_| {
        example(
            "Default",
            column![
                kbd_group([kbd("⌘"), kbd("⇧"), kbd("⌥"), kbd("⌃")]),
                kbd_group([kbd("Ctrl"), text("+").size(14).into(), kbd("B")]),
                row![
                    text("Use").size(14).style(crate::muted),
                    kbd_group([kbd("Ctrl + B"), kbd("Ctrl + K")]),
                    text("to open the command palette")
                        .size(14)
                        .style(crate::muted),
                ]
                .spacing(6)
                .align_y(Center),
            ]
            .spacing(16)
            .align_x(Center),
        )
    },
};

pub const PROGRESS: Page = Page {
    id: "progress",
    title: "Progress",
    description: "Displays an indicator showing the completion progress of a task, typically displayed as a progress bar.",
    view: |gallery| {
        example(
            "Default",
            column![
                container(progress(gallery.demo.progress)).max_width(360),
                row![
                    button("-10")
                        .variant(button::Variant::Outline)
                        .size(button::Size::Sm)
                        .on_press(demo(DemoMessage::Progress(-10.0))),
                    button("+10")
                        .variant(button::Variant::Outline)
                        .size(button::Size::Sm)
                        .on_press(demo(DemoMessage::Progress(10.0))),
                ]
                .spacing(8),
            ]
            .spacing(16)
            .align_x(Center),
        )
    },
};

pub const AVATAR: Page = Page {
    id: "avatar",
    title: "Avatar",
    description: "An image element with a fallback for representing the user.",
    view: |_| {
        column![
            example(
                "Sizes",
                row![
                    avatar("CN").size(avatar::Size::Sm),
                    avatar("CN"),
                    avatar("ER").size(avatar::Size::Lg),
                ]
                .spacing(16)
                .align_y(Center)
            ),
            example(
                "Group",
                avatar_group([avatar("CN"), avatar("LR"), avatar("ER")])
            ),
        ]
        .into()
    },
};

pub const TYPOGRAPHY: Page = Page {
    id: "typography",
    title: "Typography",
    description: "Styles for headings, paragraphs, lists...etc",
    view: |_| {
        example(
            "Typography",
            column![
                ty::h1("Taxing Laughter: The Joke Tax Chronicles"),
                ty::lead("A modal dialog that interrupts the user with important content and expects a response."),
                ty::h2("The King's Plan"),
                ty::p("The king thought long and hard, and finally came up with a brilliant plan: he would tax the jokes in the kingdom."),
                ty::blockquote("\"After all,\" he said, \"everyone enjoys a good joke, so it's only fair that they should pay for the privilege.\""),
                ty::h3("The Joke Tax"),
                ty::list(["1st level of puns: 5 gold coins", "2nd level of jokes: 10 gold coins", "3rd level of one-liners: 20 gold coins"]),
                ty::h4("People stopped telling jokes"),
                row![ty::p("Run"), ty::inline_code("cargo iced-ui add button"), ty::p("to copy it.")].spacing(6).align_y(Center),
                ty::large("Are you absolutely sure?"),
                ty::small("Email address"),
                ty::muted_text("Enter your email address."),
            ]
            .spacing(16)
            .width(Fill),
        )
    },
};

pub const TABLE: Page = Page {
    id: "table",
    title: "Table",
    description: "A responsive table component.",
    view: |_| {
        example(
            "Invoices",
            table(["Invoice", "Status", "Method", "Amount"])
                .row(["INV001", "Paid", "Credit Card", "$250.00"])
                .row(["INV002", "Pending", "PayPal", "$150.00"])
                .row(["INV003", "Unpaid", "Bank Transfer", "$350.00"])
                .row(["INV004", "Paid", "Credit Card", "$450.00"])
                .row(["INV005", "Paid", "PayPal", "$550.00"])
                .footer(["Total", "", "", "$1,750.00"])
                .caption("A list of your recent invoices.")
                .align(3, iced::alignment::Horizontal::Right),
        )
    },
};

pub const BREADCRUMB: Page = Page {
    id: "breadcrumb",
    title: "Breadcrumb",
    description: "Displays the path to the current resource using a hierarchy of links.",
    view: |_| {
        column![
            example(
                "Default",
                breadcrumb()
                    .link("Home", Message::Noop)
                    .ellipsis()
                    .link("Components", Message::Noop)
                    .page("Breadcrumb")
            ),
            example(
                "Custom separator",
                breadcrumb()
                    .separator(lucide::SLASH)
                    .link("Home", Message::Noop)
                    .link("Components", Message::Noop)
                    .page("Breadcrumb")
            ),
        ]
        .into()
    },
};

pub const PAGINATION: Page = Page {
    id: "pagination",
    title: "Pagination",
    description: "Pagination with page navigation, next and previous links.",
    view: |gallery| {
        example(
            "Default",
            pagination(gallery.demo.page, 10, |page| demo(DemoMessage::Page(page))),
        )
    },
};

pub const EMPTY: Page = Page {
    id: "empty",
    title: "Empty",
    description: "Use the Empty component to display an empty state.",
    view: |_| {
        example(
            "Default",
            empty("No Projects Yet")
                .icon(lucide::FOLDER_CODE)
                .description("You haven't created any projects yet. Get started by creating your first project.")
                .content(
                    row![
                        button("Create Project").on_press(Message::Noop),
                        button("Import Project").variant(button::Variant::Outline).on_press(Message::Noop),
                    ]
                    .spacing(8),
                ),
        )
    },
};

pub const ITEM: Page = Page {
    id: "item",
    title: "Item",
    description: "A versatile component that you can use to display any content.",
    view: |_| {
        example(
            "Variants",
            column![
                item("Basic Item")
                    .description("A simple item with title and description.")
                    .variant(item::Variant::Outline)
                    .action(
                        button("Action")
                            .variant(button::Variant::Outline)
                            .size(button::Size::Sm)
                            .on_press(Message::Noop)
                    ),
                item("Your profile has been verified.")
                    .icon(lucide::BADGE_CHECK)
                    .variant(item::Variant::Outline)
                    .size(item::Size::Sm)
                    .action(lucide::CHEVRON_RIGHT.sized(16))
                    .on_press(Message::Noop),
                item("Muted Item")
                    .description("Use the muted variant for secondary content.")
                    .media(avatar("ER"))
                    .variant(item::Variant::Muted),
            ]
            .spacing(16)
            .max_width(480),
        )
    },
};

pub const ASPECT_RATIO: Page = Page {
    id: "aspect-ratio",
    title: "Aspect Ratio",
    description: "Displays content within a desired ratio.",
    view: |_| {
        example(
            "16 / 9",
            container(aspect_ratio(
                16.0 / 9.0,
                container(text("16 : 9").size(14).style(crate::muted))
                    .center(Fill)
                    .style(|theme: &iced::Theme| {
                        let t = iced_ui::tokens(theme);
                        container::Style {
                            background: Some(t.palette.muted.into()),
                            border: iced_ui::style::rounded(t.radius_lg()),
                            ..Default::default()
                        }
                    }),
            ))
            .max_width(450),
        )
    },
};

pub const SCROLL_AREA: Page = Page {
    id: "scroll-area",
    title: "Scroll Area",
    description: "Augments native scroll functionality for custom, cross-browser styling.",
    view: |_| {
        let tags = column((1..=50).rev().map(|i| {
            column![text(format!("v1.2.0-beta.{i}")).size(14), separator()]
                .spacing(8)
                .into()
        }))
        .spacing(8)
        .padding(16);

        example(
            "Tags",
            container(
                scroll_area(
                    column![text("Tags").size(14).font(iced_ui::font::MEDIUM), tags]
                        .padding(iced::Padding::ZERO.top(16).left(16)),
                )
                .height(288),
            )
            .width(192)
            .style(|theme: &iced::Theme| {
                let t = iced_ui::tokens(theme);
                container::Style {
                    border: iced_ui::style::border(t.palette.border, t.radius_md()),
                    ..Default::default()
                }
            }),
        )
    },
};
