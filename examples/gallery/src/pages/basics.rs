//! Pages of the static components, mirroring the examples of the shadcn/ui
//! documentation.
use super::{DemoMessage, Page};
use crate::{Message, example};

use shadcn_iced::prelude::*;
use shadcn_iced::theme::alpha;
use shadcn_iced::{alert, avatar, badge, button, card, item, typography as ty};

use iced::widget::{column, container, row, space, text};
use iced::{Center, Color, Fill, FillPortion, Theme};

fn demo(message: DemoMessage) -> Message {
    Message::Demo(message)
}

/// A `text-sm` paragraph.
fn sm<'a>(content: impl text::IntoFragment<'a>) -> text::Text<'a> {
    text(content).size(14).line_height(iced::Pixels(20.0))
}

/// A `text-sm text-muted-foreground` paragraph.
fn sm_muted<'a>(content: impl text::IntoFragment<'a>) -> text::Text<'a> {
    sm(content).style(crate::muted)
}

pub const BADGE: Page = Page {
    id: "badge",
    title: "Badge",
    description: "Displays a badge or a component that looks like a badge.",
    view: |_| {
        column![
            example(
                "Demo",
                row![
                    badge("Badge"),
                    badge("Secondary").variant(badge::Variant::Secondary),
                    badge("Destructive").variant(badge::Variant::Destructive),
                    badge("Outline").variant(badge::Variant::Outline),
                ]
                .spacing(8)
            ),
            example(
                "Variants",
                row![
                    badge("Default"),
                    badge("Secondary").variant(badge::Variant::Secondary),
                    badge("Destructive").variant(badge::Variant::Destructive),
                    badge("Outline").variant(badge::Variant::Outline),
                    badge("Ghost").variant(badge::Variant::Ghost),
                ]
                .spacing(8)
            ),
            example(
                "With Icon",
                row![
                    badge("Verified")
                        .icon(lucide::BADGE_CHECK)
                        .variant(badge::Variant::Secondary),
                    badge("Bookmark")
                        .trailing_icon(lucide::BOOKMARK)
                        .variant(badge::Variant::Outline),
                ]
                .spacing(8)
            ),
            example(
                "With Spinner",
                row![
                    badge("Deleting")
                        .leading(
                            spinner()
                                .size(12.0)
                                .style(|theme| { shadcn_iced::tokens(theme).palette.destructive })
                        )
                        .variant(badge::Variant::Destructive),
                    badge("Generating")
                        .trailing(spinner().size(12.0).style(|theme| {
                            shadcn_iced::tokens(theme).palette.secondary_foreground
                        }))
                        .variant(badge::Variant::Secondary),
                ]
                .spacing(8)
            ),
            example(
                "Link",
                badge("Open Link")
                    .trailing_icon(lucide::ARROW_UP_RIGHT)
                    .on_press(Message::Noop)
            ),
        ]
        .spacing(24)
        .into()
    },
};

pub const CARD: Page = Page {
    id: "card",
    title: "Card",
    description: "Displays a card with header, content, and footer.",
    view: |gallery| {
        let bullet = |label| {
            row![
                container(lucide::CHEVRON_RIGHT.sized(16).style(crate::muted))
                    .padding(iced::Padding::ZERO.top(2)),
                sm(label),
            ]
            .spacing(8)
        };

        column![
            example(
                "Demo",
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
                                    sm("Forgot your password?")
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
            ),
            example(
                "Small",
                card()
                    .size(card::Size::Sm)
                    .title("Scheduled reports")
                    .description("Weekly snapshots. No more manual exports.")
                    .content(
                        column![
                            bullet("Choose a schedule (daily, or weekly)."),
                            bullet("Send to channels or specific teammates."),
                            bullet("Include charts, tables, and key metrics."),
                        ]
                        .spacing(8)
                        .padding([8, 0]),
                    )
                    .footer(
                        column![
                            button("Set up scheduled reports")
                                .size(button::Size::Sm)
                                .width(Fill)
                                .on_press(Message::Noop),
                            button("See what's new")
                                .variant(button::Variant::Outline)
                                .size(button::Size::Sm)
                                .width(Fill)
                                .on_press(Message::Noop),
                        ]
                        .spacing(8),
                    )
                    .width(320),
            ),
        ]
        .spacing(24)
        .into()
    },
};

pub const INPUT: Page = Page {
    id: "input",
    title: "Input",
    description: "Displays a form input field or a component that looks like an input field.",
    view: |gallery| {
        column![
            example(
                "Demo",
                container(
                    field(
                        "API Key",
                        input("sk-...", &gallery.demo.password)
                            .secure(true)
                            .on_input(|v| demo(DemoMessage::Password(v)))
                    )
                    .description("Your API key is encrypted and stored securely.")
                )
                .max_width(384)
            ),
            example(
                "Basic",
                container(
                    input("Enter text", &gallery.demo.name)
                        .on_input(|v| demo(DemoMessage::Name(v)))
                )
                .max_width(384)
            ),
            example(
                "Disabled",
                container(
                    field("Email", input::<Message>("Email", ""))
                        .description("This field is currently disabled.")
                        .disabled(true)
                )
                .max_width(384)
            ),
            example(
                "Invalid",
                container(
                    field(
                        "Invalid Input",
                        input("Error", &gallery.demo.username)
                            .invalid(true)
                            .on_input(|v| demo(DemoMessage::Username(v)))
                    )
                    .description("This field contains validation errors.")
                )
                .max_width(384)
            ),
            example(
                "Inline",
                container(
                    row![
                        input("Search...", &gallery.demo.search)
                            .on_input(|v| demo(DemoMessage::Search(v))),
                        button("Search").on_press(Message::Noop),
                    ]
                    .spacing(12)
                    .align_y(Center)
                )
                .max_width(384)
            ),
        ]
        .spacing(24)
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
                "Demo",
                container(
                    textarea(&gallery.demo.message)
                        .placeholder("Type your message here.")
                        .on_action(|action| demo(DemoMessage::Edit(action)))
                )
                .max_width(384)
            ),
            example(
                "Field",
                container(
                    column![
                        label("Message"),
                        sm_muted("Enter your message below."),
                        textarea(&gallery.demo.message)
                            .placeholder("Type your message here.")
                            .on_action(|action| demo(DemoMessage::Edit(action))),
                    ]
                    .spacing(8)
                )
                .max_width(384)
            ),
            example(
                "Disabled",
                container(
                    column![
                        container(label("Message")).style(|theme: &Theme| container::Style {
                            text_color: Some(shadcn_iced::theme::alpha(
                                shadcn_iced::tokens(theme).palette.foreground,
                                0.5
                            )),
                            ..Default::default()
                        }),
                        textarea::<Message>(&gallery.demo.message)
                            .placeholder("Type your message here."),
                    ]
                    .spacing(8)
                )
                .max_width(384)
            ),
            example(
                "Invalid",
                container(
                    column![
                        label("Message").style(|theme: &Theme| text::Style {
                            color: Some(shadcn_iced::tokens(theme).palette.destructive),
                        }),
                        textarea(&gallery.demo.message)
                            .placeholder("Type your message here.")
                            .invalid(true)
                            .on_action(|action| demo(DemoMessage::Edit(action))),
                        sm_muted("Please enter a valid message."),
                    ]
                    .spacing(8)
                )
                .max_width(384)
            ),
            example(
                "Button",
                container(
                    column![
                        textarea(&gallery.demo.message)
                            .placeholder("Type your message here.")
                            .on_action(|action| demo(DemoMessage::Edit(action))),
                        button("Send message").width(Fill).on_press(Message::Noop),
                    ]
                    .spacing(8)
                )
                .max_width(384)
            ),
        ]
        .spacing(24)
        .into()
    },
};

pub const ALERT: Page = Page {
    id: "alert",
    title: "Alert",
    description: "Displays a callout for user attention.",
    view: |_| {
        column![
            example(
                "Demo",
                column![
                    alert("Payment successful")
                        .icon(lucide::CIRCLE_CHECK)
                        .description(
                            "Your payment of $29.99 has been processed. A receipt has been sent to your email address."
                        ),
                    alert("New feature available").icon(lucide::INFO).description(
                        "We've added dark mode support. You can enable it in your account settings."
                    ),
                ]
                .spacing(16)
                .max_width(448),
            ),
            example(
                "Destructive",
                container(
                    alert("Payment failed")
                        .icon(lucide::CIRCLE_ALERT)
                        .variant(alert::Variant::Destructive)
                        .description(
                            "Your payment could not be processed. Please check your payment method and try again."
                        )
                )
                .max_width(448),
            ),
            example(
                "Action",
                container(
                    alert("Dark mode is now available")
                        .description("Enable it under your profile settings to get started.")
                        .action(
                            button("Enable")
                                .size(button::Size::Xs)
                                .on_press(Message::Noop)
                        )
                )
                .max_width(448),
            ),
        ]
        .spacing(24)
        .into()
    },
};

pub const SEPARATOR: Page = Page {
    id: "separator",
    title: "Separator",
    description: "Visually or semantically separates content.",
    view: |_| {
        let entry =
            |key, value| row![sm(key), space::horizontal(), sm_muted(value)].align_y(Center);

        column![
            example(
                "Demo",
                column![
                    column![
                        text("shadcn/ui")
                            .size(14)
                            .line_height(iced::Pixels(14.0))
                            .font(shadcn_iced::font::medium()),
                        sm_muted("The Foundation for your Design System"),
                    ]
                    .spacing(6),
                    separator(),
                    sm(
                        "A set of beautifully designed components that you can customize, extend, and build on."
                    ),
                ]
                .spacing(16)
                .max_width(384),
            ),
            example(
                "Vertical",
                row![
                    sm("Blog"),
                    vertical_separator(),
                    sm("Docs"),
                    vertical_separator(),
                    sm("Source"),
                ]
                .spacing(16)
                .height(20)
                .align_y(Center),
            ),
            example(
                "List",
                column![
                    entry("Item 1", "Value 1"),
                    separator(),
                    entry("Item 2", "Value 2"),
                    separator(),
                    entry("Item 3", "Value 3"),
                ]
                .spacing(8)
                .max_width(384),
            ),
        ]
        .spacing(24)
        .into()
    },
};

pub const SKELETON: Page = Page {
    id: "skeleton",
    title: "Skeleton",
    description: "Use to show a placeholder while content is loading.",
    view: |_| {
        // `w-2/3`, `w-1/2`... as portions of the available width.
        let fraction = |part: u16, whole: u16, height: f32| -> iced::Element<'static, Message> {
            if part == whole {
                skeleton(Fill, height).into()
            } else {
                row![
                    skeleton(FillPortion(part), height),
                    space().width(FillPortion(whole - part))
                ]
                .into()
            }
        };

        column![
            example(
                "Demo",
                row![
                    skeleton(48, 48).rounded_full(),
                    column![skeleton(250, 16), skeleton(200, 16)].spacing(8),
                ]
                .spacing(16)
                .align_y(Center)
            ),
            example(
                "Avatar",
                row![
                    skeleton(40, 40).rounded_full(),
                    column![skeleton(150, 16), skeleton(100, 16)].spacing(8),
                ]
                .spacing(16)
                .align_y(Center)
            ),
            example(
                "Card",
                card()
                    .title_element(column![fraction(2, 3, 16.0), fraction(1, 2, 16.0)].spacing(4))
                    .content(aspect_ratio(16.0 / 9.0, skeleton(Fill, Fill)))
                    .width(320)
            ),
            example(
                "Text",
                column![
                    fraction(1, 1, 16.0),
                    fraction(1, 1, 16.0),
                    fraction(3, 4, 16.0)
                ]
                .spacing(8)
                .max_width(320)
            ),
            example(
                "Form",
                column![
                    column![skeleton(80, 16), skeleton(Fill, 32)].spacing(12),
                    column![skeleton(96, 16), skeleton(Fill, 32)].spacing(12),
                    skeleton(96, 32),
                ]
                .spacing(28)
                .max_width(320)
            ),
        ]
        .spacing(24)
        .into()
    },
};

pub const SPINNER: Page =
    Page {
        id: "spinner",
        title: "Spinner",
        description: "An indicator that can be used to show a loading state.",
        view: |_| {
            let loading = |label, variant, style: fn(&Theme) -> Color| {
                button::Button::new(
                    row![
                        spinner().size(14.0).style(style),
                        text(label).size(12.8).font(shadcn_iced::font::medium())
                    ]
                    .spacing(4)
                    .align_y(Center),
                )
                .variant(variant)
                .size(button::Size::Sm)
                .disabled(true)
            };

            column![
                example(
                    "Demo",
                    container(
                        item("Processing payment...")
                            .media(spinner())
                            .action(sm("$100.00"))
                            .variant(item::Variant::Muted)
                    )
                    .max_width(320)
                ),
                example(
                    "Size",
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
                    "Button",
                    column![
                        // `disabled:opacity-50`
                        loading("Loading...", button::Variant::Default, |theme| {
                            alpha(shadcn_iced::tokens(theme).palette.primary_foreground, 0.5)
                        }),
                        loading("Please wait", button::Variant::Outline, |theme| {
                            alpha(shadcn_iced::tokens(theme).palette.foreground, 0.5)
                        }),
                        loading("Processing", button::Variant::Secondary, |theme| {
                            alpha(shadcn_iced::tokens(theme).palette.secondary_foreground, 0.5)
                        }),
                    ]
                    .spacing(16)
                    .align_x(Center)
                ),
                example(
                    "Badge",
                    row![
                        badge("Syncing").leading(spinner().size(12.0).style(|theme| {
                            shadcn_iced::tokens(theme).palette.primary_foreground
                        })),
                        badge("Updating")
                            .leading(spinner().size(12.0).style(|theme| {
                                shadcn_iced::tokens(theme).palette.secondary_foreground
                            }))
                            .variant(badge::Variant::Secondary),
                        badge("Processing")
                            .leading(spinner().size(12.0))
                            .variant(badge::Variant::Outline),
                    ]
                    .spacing(16)
                ),
            ]
            .spacing(24)
            .into()
        },
    };

pub const KBD: Page = Page {
    id: "kbd",
    title: "Kbd",
    description: "Used to display textual user input from keyboard.",
    view: |_| {
        column![
            example(
                "Demo",
                column![
                    kbd_group([kbd("⌘"), kbd("⇧"), kbd("⌥"), kbd("⌃")]),
                    kbd_group([kbd("Ctrl"), sm("+").into(), kbd("B")]),
                ]
                .spacing(16)
                .align_x(Center),
            ),
            example(
                "Group",
                row![
                    sm_muted("Use"),
                    kbd_group([kbd("Ctrl + B"), kbd("Ctrl + K")]),
                    sm_muted("to open the command palette"),
                ]
                .spacing(4)
                .align_y(Center),
            ),
            example(
                "Button",
                button::Button::new(
                    row![
                        text("Accept").size(14).font(shadcn_iced::font::medium()),
                        // `translate-x-0.5` moves the key without resizing
                        // the button.
                        iced::widget::float(kbd("⏎")).translate(|_, _| iced::Vector::new(2.0, 0.0)),
                    ]
                    .spacing(6)
                    .align_y(Center)
                )
                .variant(button::Variant::Outline)
                .inline_end(true)
                .on_press(Message::Noop),
            ),
        ]
        .spacing(24)
        .into()
    },
};

pub const PROGRESS: Page = Page {
    id: "progress",
    title: "Progress",
    description: "Displays an indicator showing the completion progress of a task, typically displayed as a progress bar.",
    view: |gallery| {
        let value = gallery.demo.progress;

        column![
            example(
                "Demo",
                row![
                    space().width(FillPortion(1)),
                    container(progress(value)).width(FillPortion(3)),
                    space().width(FillPortion(1)),
                ]
            ),
            example(
                "Label",
                container(shadcn_iced::progress::progress_with_label(
                    "Upload progress",
                    56.0
                ))
                .max_width(384),
            ),
            example(
                "Controlled",
                column![
                    progress(value),
                    slider(0.0..=100.0, value, move |v: f32| demo(
                        DemoMessage::Progress(v.round())
                    )),
                ]
                .spacing(16)
                .max_width(384),
            ),
        ]
        .spacing(24)
        .into()
    },
};

pub const AVATAR: Page = Page {
    id: "avatar",
    title: "Avatar",
    description: "An image element with a fallback for representing the user.",
    view: |_| {
        // `bg-green-600`
        let green = Color::from_rgb8(0x16, 0xa3, 0x4a);

        column![
            example(
                "Demo",
                row![
                    avatar("CN"),
                    avatar("ER").badge(Some(green)),
                    avatar_group([
                        avatar("CN"),
                        avatar("LR"),
                        avatar("ER"),
                        avatar::avatar_group_count("+3"),
                    ]),
                ]
                .spacing(48)
                .align_y(Center)
            ),
            example(
                "Badge",
                row![
                    avatar("CN").badge(Some(green)),
                    avatar("PP").badge_icon(lucide::PLUS),
                ]
                .spacing(48)
                .align_y(Center)
            ),
            example(
                "Avatar Group Count",
                row![
                    avatar_group([
                        avatar("CN"),
                        avatar("LR"),
                        avatar("ER"),
                        avatar::avatar_group_count("+3"),
                    ]),
                    avatar_group([
                        avatar("CN"),
                        avatar("LR"),
                        avatar("ER"),
                        avatar::avatar_group_count_icon(lucide::PLUS),
                    ]),
                ]
                .spacing(48)
                .align_y(Center)
            ),
            example(
                "Sizes",
                row![
                    avatar("CN").size(avatar::Size::Sm),
                    avatar("CN"),
                    avatar("CN").size(avatar::Size::Lg),
                ]
                .spacing(8)
                .align_y(Center)
            ),
        ]
        .spacing(24)
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
                row![ty::p("Run"), ty::inline_code("cargo shadcn-iced add button"), ty::p("to copy it.")].spacing(6).align_y(Center),
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
        let invoices = [
            ("INV001", "Paid", "Credit Card", "$250.00"),
            ("INV002", "Pending", "PayPal", "$150.00"),
            ("INV003", "Unpaid", "Bank Transfer", "$350.00"),
            ("INV004", "Paid", "Credit Card", "$450.00"),
            ("INV005", "Paid", "PayPal", "$550.00"),
            ("INV006", "Pending", "Bank Transfer", "$200.00"),
            ("INV007", "Unpaid", "Credit Card", "$300.00"),
        ];

        let medium = |value: &'static str| -> iced::Element<'static, Message> {
            text(value)
                .size(14)
                .font(shadcn_iced::font::medium())
                .into()
        };

        let invoices_table = invoices.iter().fold(
            table(["Invoice", "Status", "Method", "Amount"]),
            |table, (invoice, status, method, amount)| {
                table.row::<shadcn_iced::table::Cell<'_, Message>>([
                    medium(invoice).into(),
                    (*status).into(),
                    (*method).into(),
                    (*amount).into(),
                ])
            },
        );

        let actions = |product: &'static str,
                       price: &'static str|
         -> [shadcn_iced::table::Cell<'static, Message>; 3] {
            [
                medium(product).into(),
                price.into(),
                iced::Element::from(
                    button::icon_button(lucide::ELLIPSIS)
                        .variant(button::Variant::Ghost)
                        .on_press(Message::Noop),
                )
                .into(),
            ]
        };

        column![
            example(
                "Demo",
                invoices_table
                    .footer(["Total", "", "", "$2,500.00"])
                    .caption("A list of your recent invoices.")
                    .widths([iced::Length::Fixed(100.0), Fill, Fill, Fill])
                    .align(3, iced::alignment::Horizontal::Right),
            ),
            example(
                "Actions",
                table(["Product", "Price", "Actions"])
                    .row(actions("Wireless Mouse", "$29.99"))
                    .row(actions("Mechanical Keyboard", "$129.99"))
                    .row(actions("USB-C Hub", "$49.99"))
                    .align(2, iced::alignment::Horizontal::Right),
            ),
        ]
        .spacing(24)
        .into()
    },
};

pub const BREADCRUMB: Page = Page {
    id: "breadcrumb",
    title: "Breadcrumb",
    description: "Displays the path to the current resource using a hierarchy of links.",
    view: |_| {
        column![
            example(
                "Demo",
                breadcrumb()
                    .link("Home", Message::Noop)
                    .item(
                        button::icon_button(lucide::ELLIPSIS)
                            .size(button::Size::IconSm)
                            .variant(button::Variant::Ghost)
                            .on_press(Message::Noop)
                    )
                    .link("Components", Message::Noop)
                    .page("Breadcrumb")
            ),
            example(
                "Basic",
                breadcrumb()
                    .link("Home", Message::Noop)
                    .link("Components", Message::Noop)
                    .page("Breadcrumb")
            ),
            example(
                "Custom separator",
                breadcrumb()
                    .separator(lucide::DOT)
                    .link("Home", Message::Noop)
                    .link("Components", Message::Noop)
                    .page("Breadcrumb")
            ),
            example(
                "Collapsed",
                breadcrumb()
                    .link("Home", Message::Noop)
                    .ellipsis()
                    .link("Components", Message::Noop)
                    .page("Breadcrumb")
            ),
        ]
        .spacing(24)
        .into()
    },
};

pub const PAGINATION: Page = Page {
    id: "pagination",
    title: "Pagination",
    description: "Pagination with page navigation, next and previous links.",
    view: |gallery| {
        let page = gallery.demo.page;

        column![
            example(
                "Demo",
                pagination(page, 10, |page| demo(DemoMessage::Page(page))),
            ),
            example(
                "Simple",
                pagination(page.min(5), 5, |page| demo(DemoMessage::Page(page)))
                    .siblings(5)
                    .controls(false),
            ),
            example(
                "Icons Only",
                pagination(page, 10, |page| demo(DemoMessage::Page(page))).numbers(false),
            ),
        ]
        .spacing(24)
        .into()
    },
};

pub const EMPTY: Page = Page {
    id: "empty",
    title: "Empty",
    description: "Use the Empty component to display an empty state.",
    view: |_| {
        column![
            example(
                "Demo",
                empty("No Projects Yet")
                    .icon(lucide::FOLDER_CODE)
                    .description("You haven't created any projects yet. Get started by creating your first project.")
                    .content(
                        row![
                            button("Create Project").on_press(Message::Noop),
                            button("Import Project").variant(button::Variant::Outline).on_press(Message::Noop),
                        ]
                        .spacing(8),
                    )
                    .extra(
                        button::Button::new(
                            row![
                                text("Learn More").size(12.8).font(shadcn_iced::font::medium()).style(crate::muted),
                                lucide::ARROW_UP_RIGHT.sized(14).style(crate::muted),
                            ]
                            .spacing(4)
                            .align_y(Center),
                        )
                        .variant(button::Variant::Link)
                        .size(button::Size::Sm)
                        .on_press(Message::Noop),
                    ),
            ),
            example(
                "Outline",
                empty("Cloud Storage Empty")
                    .icon(lucide::CLOUD)
                    .description("Upload files to your cloud storage to access them anywhere.")
                    .content(
                        button("Upload Files")
                            .variant(button::Variant::Outline)
                            .size(button::Size::Sm)
                            .on_press(Message::Noop),
                    )
                    .bordered(true),
            ),
            example(
                "Background",
                empty("No Notifications")
                    .icon(lucide::BELL)
                    .description("You're all caught up. New notifications will appear here.")
                    .content(
                        button("Refresh")
                            .icon(lucide::REFRESH_CCW)
                            .variant(button::Variant::Outline)
                            .on_press(Message::Noop),
                    )
                    .muted(true),
            ),
            example(
                "Avatar",
                empty("User Offline")
                    .media(avatar("LR").diameter(48.0))
                    .description("This user is currently offline. You can leave a message to notify them or try again later.")
                    .content(button("Leave Message").size(button::Size::Sm).on_press(Message::Noop)),
            ),
        ]
        .spacing(24)
        .into()
    },
};

pub const ITEM: Page = Page {
    id: "item",
    title: "Item",
    description: "A versatile component that you can use to display any content.",
    view: |_| {
        let inbox = |title, description, variant, size| {
            item(title)
                .icon(lucide::INBOX)
                .description(description)
                .variant(variant)
                .size(size)
        };

        let person = |name: &'static str, email: &'static str| {
            item(name)
                .media(avatar(&name[..1]))
                .description(email)
                .variant(item::Variant::Outline)
                .action(
                    button::icon_button(lucide::PLUS)
                        .variant(button::Variant::Ghost)
                        .rounded(true)
                        .on_press(Message::Noop),
                )
        };

        column![
            example(
                "Demo",
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
                        .media(lucide::BADGE_CHECK.sized(20))
                        .variant(item::Variant::Outline)
                        .size(item::Size::Sm)
                        .action(lucide::CHEVRON_RIGHT.sized(16))
                        .on_press(Message::Noop),
                ]
                .spacing(24)
                .max_width(448),
            ),
            example(
                "Variant",
                column![
                    inbox(
                        "Default Variant",
                        "Transparent background with no border.",
                        item::Variant::Default,
                        item::Size::Default
                    ),
                    inbox(
                        "Outline Variant",
                        "Outlined style with a visible border.",
                        item::Variant::Outline,
                        item::Size::Default
                    ),
                    inbox(
                        "Muted Variant",
                        "Muted background for secondary content.",
                        item::Variant::Muted,
                        item::Size::Default
                    ),
                ]
                .spacing(24)
                .max_width(448),
            ),
            example(
                "Size",
                column![
                    inbox(
                        "Default Size",
                        "The standard size for most use cases.",
                        item::Variant::Outline,
                        item::Size::Default
                    ),
                    inbox(
                        "Small Size",
                        "A compact size for dense layouts.",
                        item::Variant::Outline,
                        item::Size::Sm
                    ),
                    inbox(
                        "Extra Small Size",
                        "The most compact size available.",
                        item::Variant::Outline,
                        item::Size::Xs
                    ),
                ]
                .spacing(24)
                .max_width(448),
            ),
            example(
                "Avatar",
                column![
                    item("Evil Rabbit")
                        .media(avatar("ER").size(avatar::Size::Lg))
                        .description("Last seen 5 months ago")
                        .variant(item::Variant::Outline)
                        .action(
                            button::icon_button(lucide::PLUS)
                                .size(button::Size::IconSm)
                                .variant(button::Variant::Outline)
                                .rounded(true)
                                .on_press(Message::Noop)
                        ),
                    item("No Team Members")
                        .media(avatar_group([avatar("CN"), avatar("LR"), avatar("ER")]))
                        .description("Invite your team to collaborate on this project.")
                        .variant(item::Variant::Outline)
                        .action(
                            button("Invite")
                                .size(button::Size::Sm)
                                .variant(button::Variant::Outline)
                                .on_press(Message::Noop)
                        ),
                ]
                .spacing(24)
                .max_width(512),
            ),
            example(
                "Group",
                container(item::item_group([
                    person("shadcn", "shadcn@vercel.com"),
                    person("maxleiter", "maxleiter@vercel.com"),
                    person("evilrabbit", "evilrabbit@vercel.com"),
                ]))
                .max_width(384),
            ),
        ]
        .spacing(24)
        .into()
    },
};

/// A `rounded-lg bg-muted` box, standing in for an image.
fn placeholder<'a>(label: &'a str) -> iced::Element<'a, Message> {
    container(sm_muted(label))
        .center(Fill)
        .style(|theme: &Theme| {
            let t = shadcn_iced::tokens(theme);
            container::Style {
                background: Some(t.palette.muted.into()),
                border: shadcn_iced::style::rounded(t.radius_lg()),
                ..Default::default()
            }
        })
        .into()
}

pub const ASPECT_RATIO: Page = Page {
    id: "aspect-ratio",
    title: "Aspect Ratio",
    description: "Displays content within a desired ratio.",
    view: |_| {
        column![
            example(
                "Demo",
                container(aspect_ratio(16.0 / 9.0, placeholder("16 : 9"))).max_width(384),
            ),
            example(
                "Square",
                container(aspect_ratio(1.0, placeholder("1 : 1"))).max_width(192),
            ),
        ]
        .spacing(24)
        .into()
    },
};

pub const SCROLL_AREA: Page = Page {
    id: "scroll-area",
    title: "Scroll Area",
    description: "Augments native scroll functionality for custom, cross-browser styling.",
    view: |_| {
        // `rounded-md border`
        let frame = |theme: &Theme| {
            let t = shadcn_iced::tokens(theme);
            container::Style {
                border: shadcn_iced::style::border(t.palette.border, t.radius_md()),
                ..Default::default()
            }
        };

        let tags = column((1..=50).rev().map(|i| {
            column![sm(format!("v1.2.0-beta.{i}")), separator()]
                .spacing(8)
                .into()
        }))
        .spacing(8);

        let works = row(
            ["Ornella Binni", "Tom Byrom", "Vladimir Malyavko"].map(|artist| {
                column![
                    container(placeholder("3 : 4")).width(150).height(200),
                    row![
                        text("Photo by").size(12).style(crate::muted),
                        text(artist).size(12).font(shadcn_iced::font::semibold()),
                    ]
                    .spacing(4),
                ]
                .spacing(8)
                .into()
            }),
        )
        .spacing(16)
        .padding(16);

        column![
            example(
                "Demo",
                container(
                    scroll_area(
                        column![
                            text("Tags")
                                .size(14)
                                .line_height(iced::Pixels(14.0))
                                .font(shadcn_iced::font::medium()),
                            tags
                        ]
                        .spacing(16)
                        .padding(16),
                    )
                    .height(288),
                )
                .width(192)
                .padding(1)
                .style(frame),
            ),
            example(
                "Horizontal",
                container(shadcn_iced::scroll_area::horizontal_scroll_area(works))
                    .width(384)
                    .padding(1)
                    .style(frame),
            ),
        ]
        .spacing(24)
        .into()
    },
};

pub const LABEL: Page = Page {
    id: "label",
    title: "Label",
    description: "Renders an accessible label associated with controls.",
    view: |gallery| {
        column![
            example(
                "Demo",
                row![
                    checkbox(gallery.demo.terms).on_toggle(|v| demo(DemoMessage::Terms(v))),
                    label("Accept terms and conditions"),
                ]
                .spacing(8)
                .align_y(Center),
            ),
            example(
                "With input",
                column![
                    label("Email"),
                    input("m@example.com", &gallery.demo.email)
                        .on_input(|v| demo(DemoMessage::Email(v)))
                ]
                .spacing(8)
                .width(280),
            ),
        ]
        .spacing(24)
        .into()
    },
};
