//! Pages of the interactive components.
use super::{DemoMessage, Page};
use crate::{Message, example};

use iced_ui::prelude::*;
use iced_ui::{button, field, tabs, toggle};

use iced::widget::{column, container, row, text};
use iced::{Center, Fill};

fn demo(message: DemoMessage) -> Message {
    Message::Demo(message)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Bold,
    Italic,
    Underline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Account,
    Password,
}

pub const CHECKBOX: Page = Page {
    id: "checkbox",
    title: "Checkbox",
    description: "A control that allows the user to toggle between checked and not checked.",
    view: |gallery| {
        let d = &gallery.demo;

        example(
            "Default",
            column![
                checkbox(d.terms).label("Accept terms and conditions").on_toggle(|v| demo(DemoMessage::Terms(v))),
                checkbox(d.notifications)
                    .label("Enable notifications")
                    .description("You can enable or disable notifications at any time.")
                    .on_toggle(|v| demo(DemoMessage::Notifications(v))),
                checkbox::<Message>(false).label("Disabled").disabled(true),
                checkbox::<Message>(true).label("Checked and disabled").disabled(true),
            ]
            .spacing(24),
        )
    },
};

pub const SWITCH: Page = Page {
    id: "switch",
    title: "Switch",
    description: "A control that allows the user to toggle between checked and not checked.",
    view: |gallery| {
        let d = &gallery.demo;

        column![
            example(
                "Default",
                switch(d.airplane).label("Airplane Mode").on_toggle(|v| demo(DemoMessage::Airplane(v)))
            ),
            example(
                "States",
                column![
                    switch(d.notifications).label("Notifications").on_toggle(|v| demo(DemoMessage::Notifications(v))),
                    switch(d.airplane)
                        .size(iced_ui::switch::Size::Sm)
                        .label("Small")
                        .on_toggle(|v| demo(DemoMessage::Airplane(v))),
                    switch::<Message>(false).label("Disabled").disabled(true),
                    switch::<Message>(true).label("Checked and disabled").disabled(true),
                ]
                .spacing(16)
            ),
        ]
        .into()
    },
};

pub const RADIO_GROUP: Page = Page {
    id: "radio-group",
    title: "Radio Group",
    description: "A set of checkable buttons where no more than one of the buttons can be checked at a time.",
    view: |gallery| {
        column![
            example(
                "Default",
                radio_group(Some(gallery.demo.density), |v| demo(DemoMessage::Density(v)))
                    .option(0, "Default")
                    .option(1, "Comfortable")
                    .option(2, "Compact")
            ),
            example(
                "With description",
                container(
                    radio_group(Some(gallery.demo.density), |v| demo(DemoMessage::Density(v)))
                        .option_with_description(0, "Default", "Standard spacing for most use cases.")
                        .option_with_description(1, "Comfortable", "More space between elements.")
                        .disabled_option(2, "Compact (disabled)")
                )
                .max_width(360)
            ),
        ]
        .into()
    },
};

pub const SLIDER: Page = Page {
    id: "slider",
    title: "Slider",
    description: "An input where the user selects a value from within a given range.",
    view: |gallery| {
        example(
            "Default",
            column![
                container(slider(0.0..=100.0, gallery.demo.volume, |v| demo(DemoMessage::Volume(v)))).max_width(360),
                text(format!("{:.0}", gallery.demo.volume)).size(14).style(crate::muted),
            ]
            .spacing(16)
            .align_x(Center),
        )
    },
};

pub const TOGGLE: Page = Page {
    id: "toggle",
    title: "Toggle",
    description: "A two-state button that can be either on or off.",
    view: |gallery| {
        let has = |format| gallery.demo.formats.contains(&format);

        column![
            example(
                "Default",
                row![
                    toggle(has(Format::Bold)).icon(lucide::BOLD).on_toggle(|_| demo(DemoMessage::Format(Format::Bold))),
                    toggle(has(Format::Italic))
                        .icon(lucide::ITALIC)
                        .variant(toggle::Variant::Outline)
                        .on_toggle(|_| demo(DemoMessage::Format(Format::Italic))),
                    toggle(has(Format::Underline))
                        .icon(lucide::UNDERLINE)
                        .label("Underline")
                        .on_toggle(|_| demo(DemoMessage::Format(Format::Underline))),
                    toggle::<Message>(false).icon(lucide::BOLD).disabled(true),
                ]
                .spacing(8)
                .align_y(Center)
            ),
            example(
                "Sizes",
                row![
                    toggle(has(Format::Italic)).icon(lucide::ITALIC).size(toggle::Size::Sm).variant(toggle::Variant::Outline).on_toggle(|_| demo(DemoMessage::Format(Format::Italic))),
                    toggle(has(Format::Italic)).icon(lucide::ITALIC).variant(toggle::Variant::Outline).on_toggle(|_| demo(DemoMessage::Format(Format::Italic))),
                    toggle(has(Format::Italic)).icon(lucide::ITALIC).size(toggle::Size::Lg).variant(toggle::Variant::Outline).on_toggle(|_| demo(DemoMessage::Format(Format::Italic))),
                ]
                .spacing(8)
                .align_y(Center)
            ),
        ]
        .into()
    },
};

pub const TOGGLE_GROUP: Page = Page {
    id: "toggle-group",
    title: "Toggle Group",
    description: "A set of two-state buttons that can be toggled on or off.",
    view: |gallery| {
        let formats = &gallery.demo.formats;

        column![
            example(
                "Default",
                toggle_group(formats, |f| demo(DemoMessage::Format(f)))
                    .icon(Format::Bold, lucide::BOLD)
                    .icon(Format::Italic, lucide::ITALIC)
                    .icon(Format::Underline, lucide::UNDERLINE)
            ),
            example(
                "Outline",
                toggle_group(formats, |f| demo(DemoMessage::Format(f)))
                    .variant(toggle::Variant::Outline)
                    .icon(Format::Bold, lucide::BOLD)
                    .icon(Format::Italic, lucide::ITALIC)
                    .icon(Format::Underline, lucide::UNDERLINE)
            ),
            example(
                "Spacing",
                toggle_group(formats, |f| demo(DemoMessage::Format(f)))
                    .variant(toggle::Variant::Outline)
                    .size(toggle::Size::Sm)
                    .spacing(8.0)
                    .item(Format::Bold, Some(lucide::BOLD), Some("Bold".into()))
                    .item(Format::Italic, Some(lucide::ITALIC), Some("Italic".into()))
                    .item(Format::Underline, Some(lucide::UNDERLINE), Some("Underline".into()))
            ),
        ]
        .into()
    },
};

pub const TABS: Page = Page {
    id: "tabs",
    title: "Tabs",
    description: "A set of layered sections of content—known as tab panels—that are displayed one at a time.",
    view: |gallery| {
        let d = &gallery.demo;

        let content = match d.tab {
            Tab::Account => card()
                .title("Account")
                .description("Make changes to your account here. Click save when you're done.")
                .content(
                    column![
                        field("Name", input("Pedro Duarte", &d.name).on_input(|v| demo(DemoMessage::Name(v)))),
                        field("Username", input("@peduarte", &d.username).on_input(|v| demo(DemoMessage::Username(v)))),
                    ]
                    .spacing(24),
                )
                .footer(button("Save changes").on_press(Message::Noop)),
            Tab::Password => card()
                .title("Password")
                .description("Change your password here. After saving, you'll be logged out.")
                .content(
                    column![
                        field("Current password", input("", &d.password).secure(true).on_input(|v| demo(DemoMessage::Password(v)))),
                        field("New password", input("", "").secure(true)),
                    ]
                    .spacing(24),
                )
                .footer(button("Save password").on_press(Message::Noop)),
        };

        column![
            example(
                "Default",
                container(
                    tabs(d.tab, |t| demo(DemoMessage::Tab(t)))
                        .tab(Tab::Account, "Account")
                        .tab(Tab::Password, "Password")
                        .width(Fill)
                        .content(content)
                )
                .max_width(400)
            ),
            example(
                "Line",
                tabs(d.tab, |t| demo(DemoMessage::Tab(t)))
                    .variant(tabs::Variant::Line)
                    .tab_with_icon(Tab::Account, lucide::USER, "Account")
                    .tab_with_icon(Tab::Password, lucide::LOCK, "Password")
            ),
        ]
        .into()
    },
};

pub const ACCORDION: Page = Page {
    id: "accordion",
    title: "Accordion",
    description: "A vertically stacked set of interactive headings that each reveal a section of content.",
    view: |gallery| {
        let answer = |content: &'static str| text(content).size(14).line_height(1.5);

        column![
            example(
                "Single",
                container(
                    accordion()
                        .single(gallery.demo.accordion, |open| demo(DemoMessage::Accordion(open)))
                        .item("Product Information", answer("Our flagship product combines cutting-edge technology with sleek design. Built with premium materials, it offers unparalleled performance and reliability."))
                        .item("Shipping Details", answer("We offer worldwide shipping through trusted courier partners. Standard delivery takes 3-5 business days."))
                        .item("Return Policy", answer("We stand behind our products with a comprehensive 30-day return policy."))
                )
                .max_width(480)
            ),
            example(
                "Multiple",
                container(
                    accordion()
                        .open_item("Is it accessible?", answer("Yes. It adheres to the WAI-ARIA design pattern."))
                        .item("Is it styled?", answer("Yes. It comes with default styles that match the other components' aesthetic."))
                        .item("Is it animated?", answer("No, iced has no CSS animations; it opens instantly."))
                )
                .max_width(480)
            ),
        ]
        .into()
    },
};

pub const COLLAPSIBLE: Page = Page {
    id: "collapsible",
    title: "Collapsible",
    description: "An interactive component which expands/collapses a panel.",
    view: |_| {
        let repo = |name: &'static str| {
            container(text(name).size(14).font(iced_ui::font::MONO))
                .padding([8, 16])
                .width(Fill)
                .style(|theme: &iced::Theme| {
                    let t = iced_ui::tokens(theme);
                    container::Style {
                        border: iced_ui::style::border(t.palette.border, t.radius_md()),
                        ..Default::default()
                    }
                })
        };

        example(
            "Default",
            container(
                column![
                    collapsible(
                        "@peduarte starred 3 repositories",
                        column![repo("@radix-ui/colors"), repo("@stitches/react")].spacing(8)
                    ),
                ]
                .spacing(8),
            )
            .max_width(350),
        )
    },
};

pub const BUTTON_GROUP: Page = Page {
    id: "button-group",
    title: "Button Group",
    description: "A container that groups related buttons together with consistent styling.",
    view: |_| {
        use button::Variant::Outline;

        column![
            example(
                "Default",
                row![
                    button_group([button::button("Archive").variant(Outline).on_press(Message::Noop), button::button("Report").variant(Outline).on_press(Message::Noop)]),
                    button_group([
                        button::button("Snooze").variant(Outline).on_press(Message::Noop),
                        button::icon_button(lucide::ELLIPSIS).variant(Outline).on_press(Message::Noop),
                    ]),
                ]
                .spacing(8)
            ),
            example(
                "Vertical",
                button_group([
                    button::icon_button(lucide::PLUS).variant(Outline).size(button::Size::IconSm).on_press(Message::Noop),
                    button::icon_button(lucide::MINUS).variant(Outline).size(button::Size::IconSm).on_press(Message::Noop),
                ])
                .orientation(iced_ui::button_group::Orientation::Vertical)
            ),
        ]
        .into()
    },
};

pub const INPUT_GROUP: Page = Page {
    id: "input-group",
    title: "Input Group",
    description: "Display additional information or actions to an input or textarea.",
    view: |gallery| {
        let d = &gallery.demo;

        example(
            "Addons",
            container(
                column![
                    input_group(input("Search...", &d.search).on_input(|v| demo(DemoMessage::Search(v))))
                        .leading_icon(lucide::SEARCH)
                        .trailing_text("12 results"),
                    input_group(input("example.com", &d.url).on_input(|v| demo(DemoMessage::Url(v))))
                        .leading_text("https://")
                        .trailing_icon(lucide::INFO),
                    input_group(input("Enter your message", &d.chat).on_input(|v| demo(DemoMessage::Chat(v))))
                        .trailing(
                            button::icon_button(lucide::ARROW_UP)
                                .size(button::Size::IconXs)
                                .on_press(Message::Noop)
                        ),
                ]
                .spacing(16),
            )
            .max_width(384),
        )
    },
};

pub const FIELD: Page = Page {
    id: "field",
    title: "Field",
    description: "Combine labels, controls, and help text to compose accessible form fields and grouped inputs.",
    view: |gallery| {
        let d = &gallery.demo;

        example(
            "Form",
            container(
                field_set("Payment Method")
                    .description("All transactions are secure and encrypted")
                    .push(
                        field("Name on Card", input("Evil Rabbit", &d.name).on_input(|v| demo(DemoMessage::Name(v))))
                    )
                    .push(
                        field("Card Number", input("1234 5678 9012 3456", &d.card).on_input(|v| demo(DemoMessage::Card(v))))
                            .description("Enter your 16-digit card number")
                            .error_maybe((!d.card.is_empty() && d.card.len() < 16).then(|| "Card number is too short.".to_owned()))
                    )
                    .push(field_separator::<Message>(Some("Or")))
                    .push(
                        field("Same as shipping address", checkbox(d.terms).on_toggle(|v| demo(DemoMessage::Terms(v))))
                            .orientation(field::Orientation::Horizontal)
                    )
                    .push(
                        field("Marketing emails", switch(d.notifications).on_toggle(|v| demo(DemoMessage::Notifications(v))))
                            .description("Receive emails about new products, features, and more.")
                            .orientation(field::Orientation::Responsive)
                    ),
            )
            .max_width(420),
        )
    },
};

pub const NATIVE_SELECT: Page = Page {
    id: "native-select",
    title: "Native Select",
    description: "A styled native HTML select element with consistent design system integration.",
    view: |gallery| {
        example(
            "Default",
            container(
                native_select(&super::FRUITS[..], gallery.demo.fruit, |f| demo(DemoMessage::Fruit(f)))
                    .placeholder("Select a fruit")
                    .width(Fill),
            )
            .max_width(240),
        )
    },
};

pub const INPUT_OTP: Page = Page {
    id: "input-otp",
    title: "Input OTP",
    description: "Accessible one-time password component with copy paste functionality.",
    view: |gallery| {
        example(
            "Default",
            column![
                input_otp(6, &gallery.demo.otp, |v| demo(DemoMessage::Otp(v))).groups(&[3, 3]),
                text(if gallery.demo.otp.is_empty() {
                    "Enter your one-time password.".to_owned()
                } else {
                    format!("You entered: {}", gallery.demo.otp)
                })
                .size(14)
                .style(crate::muted),
            ]
            .spacing(12)
            .align_x(Center),
        )
    },
};

pub const RESIZABLE: Page = Page {
    id: "resizable",
    title: "Resizable",
    description: "Accessible resizable panel groups and layouts with keyboard support.",
    view: |_| {
        let panel = |label: &'static str| container(text(label).size(14).font(iced_ui::font::SEMIBOLD)).center(Fill);

        example(
            "Default",
            container(
                resizable(
                    panel("One"),
                    resizable(panel("Two"), panel("Three"))
                        .axis(iced_ui::resizable::Axis::Vertical)
                        .default_ratio(0.25)
                        .grip(true),
                )
                .grip(true),
            )
            .width(Fill)
            .max_width(500)
            .height(200)
            .style(|theme: &iced::Theme| {
                let t = iced_ui::tokens(theme);
                container::Style {
                    border: iced_ui::style::border(t.palette.border, t.radius_lg()),
                    ..Default::default()
                }
            }),
        )
    },
};
