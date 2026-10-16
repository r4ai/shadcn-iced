//! Pages of the interactive components.
//!
//! The demos mirror the examples of the shadcn/ui docs (Base UI, Nova).
use super::{DemoMessage, Page};
use crate::{Message, example};

use iced_ui::prelude::*;
use iced_ui::{button, button_group, field, font, input_group, radio_group, tabs, toggle};

use iced::widget::{column, container, row, text};
use iced::{Center, Element, Fill, Theme};

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
    Notifications,
    Overview,
    Analytics,
    Reports,
    Settings,
}

/// A static control that does nothing when used.
fn noop<T>(_: T) -> Message {
    Message::Noop
}

/// `text-sm text-muted-foreground`
fn muted<'a>(content: &'a str) -> iced::widget::Text<'a> {
    text(content).size(14).style(crate::muted)
}

pub const CHECKBOX: Page = Page {
    id: "checkbox",
    title: "Checkbox",
    description: "A control that allows the user to toggle between checked and not checked.",
    view: |gallery| {
        let d = &gallery.demo;

        column![
            example(
                "Demo",
                container(
                    field::field_group([
                        checkbox(d.terms)
                            .label("Accept terms and conditions")
                            .on_toggle(|v| demo(DemoMessage::Terms(v)))
                            .into(),
                        checkbox(!d.terms)
                            .label("Accept terms and conditions")
                            .description("By clicking this checkbox, you agree to the terms.")
                            .on_toggle(|v| demo(DemoMessage::Terms(!v)))
                            .into(),
                        checkbox::<Message>(false)
                            .label("Enable notifications")
                            .disabled(true)
                            .into(),
                        field::choice_card(
                            checkbox(d.notifications)
                                .label("Enable notifications")
                                .description("You can enable or disable notifications at any time.")
                                .on_toggle(|v| demo(DemoMessage::Notifications(v))),
                            d.notifications,
                            Some(demo(DemoMessage::Notifications(!d.notifications))),
                        ),
                    ])
                    .width(Fill),
                )
                .width(Fill)
                .max_width(384),
            ),
            example(
                "Invalid",
                container(
                    checkbox(false)
                        .label("Accept terms and conditions")
                        .invalid(true)
                        .on_toggle(noop)
                )
                .width(224),
            ),
            example(
                "Group",
                container(
                    field_set("Show these items on the desktop:")
                        .legend_variant(field::LegendVariant::Label)
                        .description("Select the items you want to show on the desktop.")
                        .spacing(12.0)
                        .push(
                            checkbox(true)
                                .label("Hard disks")
                                .font(font::SANS)
                                .on_toggle(noop)
                        )
                        .push(
                            checkbox(true)
                                .label("External disks")
                                .font(font::SANS)
                                .on_toggle(noop)
                        )
                        .push(
                            checkbox(false)
                                .label("CDs, DVDs, and iPods")
                                .font(font::SANS)
                                .on_toggle(noop)
                        )
                        .push(
                            checkbox(false)
                                .label("Connected servers")
                                .font(font::SANS)
                                .on_toggle(noop)
                        ),
                )
                .width(Fill)
                .max_width(384),
            ),
        ]
        .into()
    },
};

pub const SWITCH: Page =
    Page {
        id: "switch",
        title: "Switch",
        description: "A control that allows the user to toggle between checked and not checked.",
        view: |gallery| {
            let d = &gallery.demo;

            column![
            example(
                "Demo",
                switch(d.airplane)
                    .label("Airplane Mode")
                    .on_toggle(|v| demo(DemoMessage::Airplane(v)))
            ),
            example(
                "Description",
                container(
                    field(
                        "Share across devices",
                        switch(d.notifications)
                            .on_toggle(|v| demo(DemoMessage::Notifications(v)))
                    )
                    .description(
                        "Focus is shared across devices, and turns off when you leave the app."
                    )
                    .orientation(field::Orientation::Responsive)
                )
                .width(Fill).max_width(384),
            ),
            example(
                "Choice Card",
                container(field::field_group([
                    field::choice_card(
                        field(
                            "Share across devices",
                            switch(d.notifications)
                                .on_toggle(|v| demo(DemoMessage::Notifications(v)))
                        )
                        .description(
                            "Focus is shared across devices, and turns off when you leave the app."
                        )
                        .orientation(field::Orientation::Responsive),
                        d.notifications,
                        Some(demo(DemoMessage::Notifications(!d.notifications))),
                    ),
                    field::choice_card(
                        field(
                            "Enable notifications",
                            switch(!d.airplane).on_toggle(|v| demo(DemoMessage::Airplane(!v)))
                        )
                        .description(
                            "Receive notifications when focus mode is enabled or disabled."
                        )
                        .orientation(field::Orientation::Responsive),
                        !d.airplane,
                        Some(demo(DemoMessage::Airplane(!d.airplane))),
                    ),
                ]))
                .width(Fill).max_width(384),
            ),
            example(
                "Sizes",
                container(field::field_group([
                    switch(false)
                        .size(iced_ui::switch::Size::Sm)
                        .label("Small")
                        .on_toggle(noop)
                        .into(),
                    switch(false).label("Default").on_toggle(noop).into(),
                ]))
                .width(Fill).max_width(160),
            ),
            example(
                "Disabled",
                switch::<Message>(false).label("Disabled").disabled(true)
            ),
            example(
                "Invalid",
                container(
                    field(
                        "Accept terms and conditions",
                        switch(false).invalid(true).on_toggle(noop)
                    )
                    .description("You must accept the terms and conditions to continue.")
                    .orientation(field::Orientation::Responsive)
                    .invalid(true)
                )
                .width(Fill).max_width(384),
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
        let density = gallery.demo.density;
        let on_density = |v| demo(DemoMessage::Density(v));

        let plan = |value: u8, title: &'static str, description: &'static str| {
            field::choice_card(
                field(
                    title,
                    radio_group::indicator(density == value, false, false),
                )
                .description(description)
                .orientation(field::Orientation::Responsive),
                density == value,
                Some(on_density(value)),
            )
        };

        column![
            example(
                "Demo",
                radio_group(Some(density), on_density)
                    .option(0, "Default")
                    .option(1, "Comfortable")
                    .option(2, "Compact")
            ),
            example(
                "Description",
                radio_group(Some(density), on_density)
                    .option_with_description(0, "Default", "Standard spacing for most use cases.")
                    .option_with_description(1, "Comfortable", "More space between elements.")
                    .option_with_description(2, "Compact", "Minimal spacing for dense layouts.")
            ),
            example(
                "Choice Card",
                container(
                    column![
                        plan(0, "Plus", "For individuals and small teams."),
                        plan(1, "Pro", "For growing businesses."),
                        plan(2, "Enterprise", "For large teams and enterprises."),
                    ]
                    .spacing(8)
                )
                .width(Fill)
                .max_width(384),
            ),
            example(
                "Fieldset",
                container(
                    field_set("Subscription Plan")
                        .legend_variant(field::LegendVariant::Label)
                        .description("Yearly and lifetime plans offer significant savings.")
                        .spacing(12.0)
                        .push(
                            radio_group(Some(gallery.demo.position), |v| demo(
                                DemoMessage::Position(v)
                            ))
                            .font(font::SANS)
                            .option(0, "Monthly ($9.99/month)")
                            .option(1, "Yearly ($99.99/year)")
                            .option(2, "Lifetime ($299.99)")
                        )
                )
                .width(Fill)
                .max_width(320),
            ),
            example(
                "Disabled",
                radio_group(Some(1), noop)
                    .disabled_option(0, "Disabled")
                    .option(1, "Option 2")
                    .option(2, "Option 3")
            ),
            example(
                "Invalid",
                container(
                    field_set("Notification Preferences")
                        .legend_variant(field::LegendVariant::Label)
                        .description("Choose how you want to receive notifications.")
                        .spacing(12.0)
                        .push(
                            radio_group(Some(0), noop)
                                .font(font::SANS)
                                .invalid(true)
                                .option(0, "Email only")
                                .option(1, "SMS only")
                                .option(2, "Both Email & SMS")
                        )
                )
                .width(Fill)
                .max_width(320),
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
        let narrow =
            |content: Element<'static, Message>| container(content).width(Fill).max_width(320);

        column![
            example(
                "Demo",
                narrow(
                    slider(0.0..=100.0, gallery.demo.volume, |v| demo(
                        DemoMessage::Volume(v)
                    ))
                    .into()
                )
            ),
            example(
                "Range",
                narrow(
                    iced_ui::slider::range_slider(0.0..=100.0, &[25.0, 50.0], |_, _| Message::Noop)
                        .step(5.0)
                        .into()
                )
            ),
            example(
                "Multiple Thumbs",
                narrow(
                    iced_ui::slider::range_slider(0.0..=100.0, &[10.0, 20.0, 70.0], |_, _| {
                        Message::Noop
                    })
                    .step(10.0)
                    .into()
                )
            ),
            example(
                "Vertical",
                row![
                    iced_ui::slider::vertical(0.0..=100.0, 50.0, noop),
                    iced_ui::slider::vertical(0.0..=100.0, 25.0, noop),
                ]
                .spacing(24)
            ),
            example(
                "Controlled",
                narrow(
                    column![
                        row![
                            text("Temperature").size(14).font(font::MEDIUM),
                            iced::widget::space().width(Fill),
                            muted("0.3, 0.7"),
                        ]
                        .align_y(Center),
                        iced_ui::slider::range_slider(0.0..=1.0, &[0.3, 0.7], |_, _| Message::Noop)
                            .step(0.1),
                    ]
                    .spacing(12)
                    .into()
                )
            ),
            example(
                "Disabled",
                narrow(slider(0.0..=100.0, 50.0, noop).disabled(true).into())
            ),
        ]
        .into()
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
                "Demo",
                toggle(has(Format::Underline))
                    .icon(lucide::BOOKMARK)
                    .label("Bookmark")
                    .size(toggle::Size::Sm)
                    .variant(toggle::Variant::Outline)
                    .on_toggle(|_| demo(DemoMessage::Format(Format::Underline)))
            ),
            example(
                "Outline",
                row![
                    toggle(has(Format::Italic))
                        .icon(lucide::ITALIC)
                        .label("Italic")
                        .variant(toggle::Variant::Outline)
                        .on_toggle(|_| demo(DemoMessage::Format(Format::Italic))),
                    toggle(has(Format::Bold))
                        .icon(lucide::BOLD)
                        .label("Bold")
                        .variant(toggle::Variant::Outline)
                        .on_toggle(|_| demo(DemoMessage::Format(Format::Bold))),
                ]
                .spacing(8)
            ),
            example(
                "With Text",
                toggle(has(Format::Italic))
                    .icon(lucide::ITALIC)
                    .label("Italic")
                    .on_toggle(|_| demo(DemoMessage::Format(Format::Italic)))
            ),
            example(
                "Size",
                row![
                    toggle(false)
                        .label("Small")
                        .size(toggle::Size::Sm)
                        .variant(toggle::Variant::Outline)
                        .on_toggle(noop),
                    toggle(false)
                        .label("Default")
                        .variant(toggle::Variant::Outline)
                        .on_toggle(noop),
                    toggle(false)
                        .label("Large")
                        .size(toggle::Size::Lg)
                        .variant(toggle::Variant::Outline)
                        .on_toggle(noop),
                ]
                .spacing(8)
                .align_y(Center)
            ),
            example(
                "Disabled",
                row![
                    toggle::<Message>(false).label("Disabled").disabled(true),
                    toggle::<Message>(false)
                        .label("Disabled")
                        .variant(toggle::Variant::Outline)
                        .disabled(true),
                ]
                .spacing(8)
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

        let positions = |size| {
            toggle_group(&["top"], noop)
                .variant(toggle::Variant::Outline)
                .size(size)
                .label("top", "Top")
                .label("bottom", "Bottom")
                .label("left", "Left")
                .label("right", "Right")
        };

        column![
            example(
                "Demo",
                toggle_group(formats, |f| demo(DemoMessage::Format(f)))
                    .variant(toggle::Variant::Outline)
                    .icon(Format::Bold, lucide::BOLD)
                    .icon(Format::Italic, lucide::ITALIC)
                    .icon(Format::Underline, lucide::UNDERLINE)
            ),
            example(
                "Outline",
                toggle_group(&["all"], noop)
                    .variant(toggle::Variant::Outline)
                    .label("all", "All")
                    .label("missed", "Missed")
            ),
            example(
                "Size",
                column![
                    positions(toggle::Size::Sm),
                    positions(toggle::Size::Default),
                ]
                .spacing(16)
            ),
            example("Spacing", positions(toggle::Size::Sm)),
            example(
                "Vertical",
                toggle_group(formats, |f| demo(DemoMessage::Format(f)))
                    .orientation(iced_ui::toggle_group::Orientation::Vertical)
                    .spacing(4.0)
                    .icon(Format::Bold, lucide::BOLD)
                    .icon(Format::Italic, lucide::ITALIC)
                    .icon(Format::Underline, lucide::UNDERLINE)
            ),
            example(
                "Disabled",
                toggle_group(formats, |f| demo(DemoMessage::Format(f)))
                    .disabled(true)
                    .icon(Format::Bold, lucide::BOLD)
                    .icon(Format::Italic, lucide::ITALIC)
                    .icon(Format::Underline, lucide::UNDERLINE)
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
        let selected = gallery.demo.tab;
        let on_tab = |t| demo(DemoMessage::Tab(t));

        // The demo starts on its first tab.
        let overview = match selected {
            Tab::Overview | Tab::Analytics | Tab::Reports | Tab::Settings => selected,
            _ => Tab::Overview,
        };

        let (title, description, content) = match overview {
            Tab::Analytics => (
                "Analytics",
                "Track performance and user engagement metrics. Monitor trends and identify growth opportunities.",
                "Page views are up 25% compared to last month.",
            ),
            Tab::Reports => (
                "Reports",
                "Generate and download your detailed reports. Export data in multiple formats for analysis.",
                "You have 5 reports ready and available to export.",
            ),
            Tab::Settings => (
                "Settings",
                "Manage your account preferences and options. Customize your experience to fit your needs.",
                "Configure notifications, security, and themes.",
            ),
            _ => (
                "Overview",
                "View your key metrics and recent project activity. Track progress across all your active projects.",
                "You have 12 active projects and 3 pending tasks.",
            ),
        };

        let account = match selected {
            Tab::Account | Tab::Password | Tab::Notifications => selected,
            _ => Tab::Account,
        };

        column![
            example(
                "Demo",
                container(
                    tabs(overview, on_tab)
                        .tab(Tab::Overview, "Overview")
                        .tab(Tab::Analytics, "Analytics")
                        .tab(Tab::Reports, "Reports")
                        .tab(Tab::Settings, "Settings")
                        .content(
                            card()
                                .title(title)
                                .description(description)
                                .content(muted(content))
                        )
                )
                .width(400)
            ),
            example(
                "Line",
                tabs(overview, on_tab)
                    .variant(tabs::Variant::Line)
                    .tab(Tab::Overview, "Overview")
                    .tab(Tab::Analytics, "Analytics")
                    .tab(Tab::Reports, "Reports")
            ),
            example(
                "Vertical",
                tabs(account, on_tab)
                    .orientation(tabs::Orientation::Vertical)
                    .tab(Tab::Account, "Account")
                    .tab(Tab::Password, "Password")
                    .tab(Tab::Notifications, "Notifications")
            ),
            example(
                "Disabled",
                tabs("home", noop)
                    .tab("home", "Home")
                    .disabled_tab("settings", "Disabled")
            ),
            example(
                "Icons",
                tabs("preview", noop)
                    .tab_with_icon("preview", lucide::APP_WINDOW, "Preview")
                    .tab_with_icon("code", lucide::CODE, "Code")
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
                "Demo",
                container(
                    accordion()
                        .single(gallery.demo.accordion, |open| demo(DemoMessage::Accordion(open)))
                        .item("What are your shipping options?", answer("We offer standard (5-7 days), express (2-3 days), and overnight shipping. Free shipping on international orders."))
                        .item("What is your return policy?", answer("Returns accepted within 30 days. Items must be unused and in original packaging. Refunds processed within 5-7 business days."))
                        .item("How can I contact customer support?", answer("Reach us via email, live chat, or phone. We respond within 24 hours during business days."))
                )
                .width(Fill).max_width(512)
            ),
            example(
                "Disabled",
                accordion()
                    .item("Can I access my account history?", answer("Yes, you can view your complete account history including all transactions, plan changes, and support tickets in the Account History section of your dashboard."))
                    .disabled_item("Premium feature information", answer("This section contains information about premium features. Upgrade your plan to access this content."))
                    .item("How do I update my email address?", answer("You can update your email address in your account settings. You'll receive a verification email at your new address to confirm the change."))
            ),
            example(
                "Borders",
                container(
                    accordion()
                        .bordered(true)
                        .open_item("How does billing work?", answer("We offer monthly and annual subscription plans. Billing is charged at the beginning of each cycle, and you can cancel anytime. All plans include automatic backups, 24/7 support, and unlimited team members."))
                        .item("Is my data secure?", answer("Yes. We use end-to-end encryption, SOC 2 Type II compliance, and regular third-party security audits. All data is encrypted at rest and in transit using industry-standard protocols."))
                        .item("What integrations do you support?", answer("We integrate with 500+ popular tools including Slack, Zapier, Salesforce, HubSpot, and more. You can also build custom integrations using our REST API and webhooks."))
                )
                .width(Fill).max_width(512)
            ),
            example(
                "Card",
                container(
                    card()
                        .title("Subscription & Billing")
                        .description("Common questions about your account, plans, payments and cancellations.")
                        .content(
                            accordion()
                                .open_item("What subscription plans do you offer?", answer("We offer three subscription tiers: Starter ($9/month), Professional ($29/month), and Enterprise ($99/month). Each plan includes increasing storage limits, API access, priority support, and team collaboration features."))
                                .item("How does billing work?", answer("Billing occurs automatically at the start of each billing cycle. We accept all major credit cards, PayPal, and ACH transfers for enterprise customers. You'll receive an invoice via email after each payment."))
                                .item("How do I cancel my subscription?", answer("You can cancel your subscription anytime from your account settings. There are no cancellation fees or penalties. Your access will continue until the end of your current billing period."))
                        )
                )
                .width(Fill).max_width(384)
            ),
        ]
        .into()
    },
};

/// A `rounded-md border px-4 py-2 text-sm` box.
fn bordered<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content)
        .padding([8, 16])
        .width(Fill)
        .style(|theme: &Theme| {
            let t = iced_ui::tokens(theme);
            container::Style {
                border: iced_ui::style::border(t.palette.border, t.radius_md()),
                ..Default::default()
            }
        })
        .into()
}

pub const COLLAPSIBLE: Page = Page {
    id: "collapsible",
    title: "Collapsible",
    description: "An interactive component which expands/collapses a panel.",
    view: |_| {
        let detail = |title: &'static str, value: &'static str| {
            bordered(
                column![
                    text(title).size(14).font(font::MEDIUM),
                    muted(value).line_height(1.43),
                ]
                .spacing(0),
            )
        };

        // The trigger row and the always visible status.
        let trigger = || -> Element<'static, Message> {
            column![
                row![
                    text("Order #4189").size(14).font(font::SEMIBOLD),
                    iced::widget::space().width(Fill),
                    button::icon_button::<Message>(lucide::CHEVRONS_UP_DOWN)
                        .variant(button::Variant::Ghost)
                        .size(button::Size::Icon),
                ]
                .spacing(16)
                .padding([0, 16])
                .align_y(Center),
                bordered(
                    row![
                        muted("Status"),
                        iced::widget::space().width(Fill),
                        text("Shipped").size(14).font(font::MEDIUM),
                    ]
                    .align_y(Center)
                ),
            ]
            .spacing(8)
            .into()
        };

        column![
            example(
                "Demo",
                container(
                    iced_ui::collapsible::custom(
                        trigger(),
                        trigger(),
                        column![
                            detail("Shipping address", "100 Market St, San Francisco"),
                            detail("Items", "2x Studio Headphones"),
                        ]
                        .spacing(8),
                    )
                    .spacing(8.0)
                )
                .width(350),
            ),
            example(
                "Basic",
                container(collapsible(
                    "@peduarte starred 3 repositories",
                    column![
                        bordered(text("@radix-ui/colors").size(14).font(font::MONO)),
                        bordered(text("@stitches/react").size(14).font(font::MONO)),
                    ]
                    .spacing(8)
                ))
                .width(350),
            ),
        ]
        .into()
    },
};

pub const BUTTON_GROUP: Page = Page {
    id: "button-group",
    title: "Button Group",
    description: "A container that groups related buttons together with consistent styling.",
    view: |_| {
        use button::Variant::{Outline, Secondary};

        let outline = |label: &'static str, size| {
            button::button(label)
                .variant(Outline)
                .size(size)
                .on_press(Message::Noop)
        };
        let plus = |size| {
            button::icon_button(lucide::PLUS)
                .variant(Outline)
                .size(size)
                .on_press(Message::Noop)
        };

        column![
            example(
                "Demo",
                row![
                    button_group([button::icon_button(lucide::ARROW_LEFT)
                        .variant(Outline)
                        .on_press(Message::Noop)]),
                    button_group([
                        outline("Archive", button::Size::Default),
                        outline("Report", button::Size::Default),
                    ]),
                    button_group([
                        outline("Snooze", button::Size::Default),
                        button::icon_button(lucide::ELLIPSIS)
                            .variant(Outline)
                            .on_press(Message::Noop),
                    ]),
                ]
                .spacing(8)
            ),
            example(
                "Orientation",
                button_group([
                    plus(button::Size::Icon),
                    button::icon_button(lucide::MINUS)
                        .variant(Outline)
                        .on_press(Message::Noop),
                ])
                .orientation(button_group::Orientation::Vertical)
            ),
            example(
                "Size",
                column![
                    button_group([
                        outline("Small", button::Size::Sm),
                        outline("Button", button::Size::Sm),
                        outline("Group", button::Size::Sm),
                        plus(button::Size::IconSm),
                    ]),
                    button_group([
                        outline("Default", button::Size::Default),
                        outline("Button", button::Size::Default),
                        outline("Group", button::Size::Default),
                        plus(button::Size::Icon),
                    ]),
                    button_group([
                        outline("Large", button::Size::Lg),
                        outline("Button", button::Size::Lg),
                        outline("Group", button::Size::Lg),
                        plus(button::Size::IconLg),
                    ]),
                ]
                .spacing(32)
            ),
            example(
                "Separator",
                button_group([button::button("Copy")
                    .variant(Secondary)
                    .size(button::Size::Sm)
                    .on_press(Message::Noop)])
                .separator()
                .push(
                    button::button("Paste")
                        .variant(Secondary)
                        .size(button::Size::Sm)
                        .on_press(Message::Noop)
                )
            ),
            example(
                "Split",
                button_group([button::button("Button")
                    .variant(Secondary)
                    .on_press(Message::Noop)])
                .separator()
                .push(
                    button::icon_button(lucide::PLUS)
                        .variant(Secondary)
                        .on_press(Message::Noop)
                )
            ),
            example(
                "Text",
                button_group([outline("Button", button::Size::Default)])
                    .text("Text")
                    .push(outline("Button", button::Size::Default))
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
        let wide =
            |content: Element<'static, Message>| container(content).width(Fill).max_width(384);

        column![
            example(
                "Demo",
                container(
                    input_group(
                        input("Search...", &d.search).on_input(|v| demo(DemoMessage::Search(v)))
                    )
                    .leading_icon(lucide::SEARCH)
                    .trailing_text("12 results")
                )
                .width(Fill)
                .max_width(320),
            ),
            example(
                "Icon",
                wide(
                    column![
                        input_group(input("Search...", "")).leading_icon(lucide::SEARCH),
                        input_group(input("Enter your email", "")).leading_icon(lucide::MAIL),
                        input_group(input("Card number", ""))
                            .leading_icon(lucide::CREDIT_CARD)
                            .trailing_icon(lucide::CHECK),
                        input_group(input("Card number", ""))
                            .trailing_icon(lucide::STAR)
                            .trailing_icon(lucide::INFO),
                    ]
                    .spacing(24)
                    .into()
                ),
            ),
            example(
                "Text",
                wide(
                    column![
                        input_group(input("0.00", ""))
                            .leading_text("$")
                            .trailing_text("USD"),
                        input_group(
                            input("example.com", &d.url).on_input(|v| demo(DemoMessage::Url(v)))
                        )
                        .leading_text("https://")
                        .trailing_text(".com"),
                        input_group(input("Enter your username", "")).trailing_text("@company.com"),
                    ]
                    .spacing(24)
                    .into()
                ),
            ),
            example(
                "Button",
                wide(
                    column![
                        input_group(input("https://x.com/shadcn", "")).trailing_button(
                            input_group::group_icon_button(lucide::COPY).on_press(Message::Noop)
                        ),
                        input_group(input("Type to search...", "")).trailing_button(
                            input_group::group_button("Search")
                                .variant(button::Variant::Secondary)
                                .on_press(Message::Noop)
                        ),
                    ]
                    .spacing(24)
                    .into()
                ),
            ),
            example(
                "Kbd",
                wide(
                    input_group(input("Search...", ""))
                        .leading_icon(lucide::SEARCH)
                        .trailing_kbd("⌘K")
                        .into()
                ),
            ),
            example(
                "Spinner",
                wide(
                    column![
                        input_group(input("Searching...", ""))
                            .trailing(iced_ui::spinner::spinner()),
                        input_group(input("Saving changes...", ""))
                            .trailing_text("Saving...")
                            .trailing(iced_ui::spinner::spinner()),
                    ]
                    .spacing(16)
                    .into()
                ),
            ),
        ]
        .into()
    },
};

pub const FIELD: Page = Page {
    id: "field",
    title: "Field",
    description: "Combine labels, controls, and help text to compose accessible form fields and grouped inputs.",
    view: |gallery| {
        let d = &gallery.demo;

        let month = select(None::<&'static str>, noop)
            .placeholder("MM")
            .options([
                "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11", "12",
            ])
            .width(Fill);
        let year = select(None::<&'static str>, noop)
            .placeholder("YYYY")
            .options(["2024", "2025", "2026", "2027", "2028", "2029"])
            .width(Fill);

        column![
            example(
                "Demo",
                container(field::field_group([
                    field_set("Payment Method")
                        .description("All transactions are secure and encrypted")
                        .push(field(
                            "Name on Card",
                            input("Evil Rabbit", &d.name)
                                .on_input(|v| demo(DemoMessage::Name(v))),
                        ))
                        .push(
                            field(
                                "Card Number",
                                input("1234 5678 9012 3456", &d.card)
                                    .on_input(|v| demo(DemoMessage::Card(v))),
                            )
                            .description("Enter your 16-digit card number")
                            .error_maybe(
                                (!d.card.is_empty() && d.card.len() < 16)
                                    .then(|| "Card number is too short.".to_owned()),
                            ),
                        )
                        .push(
                            row![
                                field("Month", month),
                                field("Year", year),
                                field("CVV", input("123", "")),
                            ]
                            .spacing(16)
                        )
                        .into(),
                    field_separator::<Message>(None),
                    field_set("Billing Address")
                        .description("The billing address associated with your payment method")
                        .push(
                            checkbox(d.terms)
                                .label("Same as shipping address")
                                .font(font::SANS)
                                .on_toggle(|v| demo(DemoMessage::Terms(v)))
                        )
                        .into(),
                    field(
                        "Comments",
                        textarea(&d.message)
                            .placeholder("Add any additional comments")
                            .on_action(|a| demo(DemoMessage::Edit(a)))
                    )
                    .into(),
                    row![
                        button::button("Submit").on_press(Message::Noop),
                        button::button("Cancel")
                            .variant(button::Variant::Outline)
                            .on_press(Message::Noop),
                    ]
                    .spacing(8)
                    .into(),
                ]))
                .width(Fill).max_width(448),
            ),
            example(
                "Choice Card",
                container(
                    field_set("Compute Environment")
                        .legend_variant(field::LegendVariant::Label)
                        .description("Select the compute environment for your cluster.")
                        .spacing(12.0)
                        .push(field::choice_card(
                            field(
                                "Kubernetes",
                                radio_group::indicator(d.density != 2, false, false)
                            )
                            .description(
                                "Run GPU workloads on a K8s configured cluster. This is the default."
                            )
                            .orientation(field::Orientation::Responsive),
                            d.density != 2,
                            Some(demo(DemoMessage::Density(1))),
                        ))
                        .push(field::choice_card(
                            field(
                                "Virtual Machine",
                                radio_group::indicator(d.density == 2, false, false)
                            )
                            .description(
                                "Access a VM configured cluster to run workloads. (Coming soon)"
                            )
                            .orientation(field::Orientation::Responsive),
                            d.density == 2,
                            Some(demo(DemoMessage::Density(2))),
                        ))
                )
                .width(Fill).max_width(448),
            ),
        ]
        .into()
    },
};

const STATUSES: [&str; 4] = ["Todo", "In Progress", "Done", "Cancelled"];

pub const NATIVE_SELECT: Page = Page {
    id: "native-select",
    title: "Native Select",
    description: "A styled native HTML select element with consistent design system integration.",
    view: |gallery| {
        let status = gallery.demo.fruit.filter(|s| STATUSES.contains(s));

        column![
            example(
                "Demo",
                native_select(&STATUSES[..], status, |s| demo(DemoMessage::Fruit(s)))
                    .placeholder("Select status"),
            ),
            example(
                "Small",
                iced_ui::native_select::small(
                    native_select(&STATUSES[..], status, |s| demo(DemoMessage::Fruit(s)))
                        .placeholder("Select status")
                ),
            ),
            example(
                "Invalid",
                iced_ui::native_select::invalid(
                    native_select(&super::FRUITS[..3], None::<&str>, noop)
                        .placeholder("Error state")
                ),
            ),
        ]
        .into()
    },
};

pub const INPUT_OTP: Page = Page {
    id: "input-otp",
    title: "Input OTP",
    description: "Accessible one-time password component with copy paste functionality.",
    view: |gallery| {
        column![
            example(
                "Demo",
                input_otp(6, &gallery.demo.otp, |v| demo(DemoMessage::Otp(v)))
            ),
            example("Separator", input_otp(6, "", noop).groups(&[2, 2, 2])),
            example(
                "Disabled",
                input_otp(6, "123456", noop).groups(&[3, 3]).disabled(true)
            ),
            example(
                "Invalid",
                input_otp(6, "000000", noop)
                    .groups(&[2, 2, 2])
                    .invalid(true)
            ),
            example("Four Digits", input_otp(4, "", noop)),
        ]
        .into()
    },
};

/// A resizable panel group: `max-w-sm rounded-lg border`.
fn panels<'a>(
    content: impl Into<Element<'a, Message>>,
    height: f32,
) -> iced::widget::Container<'a, Message> {
    container(content)
        .width(Fill)
        .width(Fill)
        .max_width(384)
        .height(height)
        .padding(1)
        .style(|theme: &Theme| {
            let t = iced_ui::tokens(theme);
            container::Style {
                border: iced_ui::style::border(t.palette.border, t.radius_lg()),
                ..Default::default()
            }
        })
}

pub const RESIZABLE: Page = Page {
    id: "resizable",
    title: "Resizable",
    description: "Accessible resizable panel groups and layouts with keyboard support.",
    view: |_| {
        // `flex items-center justify-center p-6`, `font-semibold`
        let panel = |label: &'static str| container(text(label).font(font::SEMIBOLD)).center(Fill);

        column![
            example(
                "Demo",
                panels(
                    resizable(
                        panel("One"),
                        resizable(panel("Two"), panel("Three"))
                            .axis(iced_ui::resizable::Axis::Vertical)
                            .default_ratio(0.25)
                            .grip(true),
                    )
                    .grip(true),
                    200.0,
                ),
            ),
            example(
                "Vertical",
                panels(
                    resizable(panel("Header"), panel("Content"))
                        .axis(iced_ui::resizable::Axis::Vertical)
                        .default_ratio(0.25),
                    200.0,
                ),
            ),
            example(
                "Handle",
                panels(
                    resizable(panel("Sidebar"), panel("Content"))
                        .default_ratio(0.25)
                        .grip(true),
                    200.0,
                ),
            ),
        ]
        .into()
    },
};
