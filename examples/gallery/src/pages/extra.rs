//! Pages of the calendar, charts, carousel and sidebar.
use super::{DemoMessage, Page};
use crate::{Message, example};

use shadcn_iced::button::{Size as ButtonSize, Variant as ButtonVariant};
use shadcn_iced::calendar::Date;
use shadcn_iced::chart::{self, Series};
use shadcn_iced::date_picker::date_range_picker;
use shadcn_iced::prelude::*;
use shadcn_iced::separator::vertical_separator;
use shadcn_iced::sidebar::{Item, Size as SidebarSize, group, item, menu_action};

use iced::widget::{column, container, row, text};
use iced::{Center, Fill, Length};

fn demo(message: DemoMessage) -> Message {
    Message::Demo(message)
}

pub const CALENDAR: Page = Page {
    id: "calendar",
    title: "Calendar",
    description: "A calendar component that allows users to select a date or a range of dates.",
    view: |gallery| {
        let d = &gallery.demo;
        let month = |m| demo(DemoMessage::Month(m));
        let select = |date| demo(DemoMessage::Date(date));

        let presets = row![
            preset("Today", 0),
            preset("Tomorrow", 1),
            preset("In 3 days", 3),
            preset("In a week", 7),
            preset("In 2 weeks", 14),
        ]
        .spacing(8)
        .wrap();

        let booked = |date: Date| {
            date.year == d.month.year
                && date.month == d.month.month
                && (17..=31).contains(&date.day)
        };

        column![
            example(
                "Demo",
                calendar(d.month, month)
                    .selected(d.date)
                    .on_select(select)
                    .dropdowns(2016, 2036)
                    .bordered(true)
            ),
            example("Basic", calendar(d.month, month).bordered(true).today(None)),
            example(
                "Range Calendar",
                calendar(d.month, month)
                    .range(d.range.0, d.range.1)
                    .on_select(|date| demo(DemoMessage::Range(date)))
                    .months(2)
                    .bordered(true)
            ),
            example(
                "Month and Year Selector",
                calendar(d.month, month)
                    .dropdowns(2016, 2036)
                    .on_select(select)
                    .bordered(true)
                    .today(None)
            ),
            example(
                "Presets",
                card()
                    .content(
                        calendar(d.month, month)
                            .selected(d.date)
                            .on_select(select)
                            .fixed_weeks(true)
                            .cell_size(38.0)
                            .padding(0.0)
                            .transparent(true)
                    )
                    .footer(presets)
                    .width(7.0 * 38.0 + 48.0)
            ),
            example(
                "Date and Time Picker",
                card()
                    .content(
                        calendar(d.month, month)
                            .selected(d.date)
                            .on_select(select)
                            .padding(0.0)
                            .transparent(true)
                    )
                    .footer(
                        column![
                            field("Start Time", input("", "10:30:00").icon(lucide::CLOCK_2)),
                            field("End Time", input("", "12:30:00").icon(lucide::CLOCK_2)),
                        ]
                        .spacing(16)
                        .width(Fill)
                    )
                    .width(7.0 * 28.0 + 48.0)
            ),
            example(
                "Booked dates",
                card()
                    .content(
                        calendar(d.month, month)
                            .selected(d.date)
                            .on_select(select)
                            .disabled(booked)
                            .struck(booked)
                            .transparent(true)
                    )
                    .width(Length::Shrink)
            ),
            example(
                "Week Numbers",
                card()
                    .content(
                        calendar(d.month, month)
                            .selected(d.date)
                            .on_select(select)
                            .week_numbers(true)
                            .transparent(true)
                    )
                    .width(Length::Shrink)
            ),
        ]
        .into()
    },
};

fn preset<'a>(label: &'a str, days: i64) -> iced::Element<'a, Message> {
    button(label)
        .variant(ButtonVariant::Outline)
        .size(ButtonSize::Sm)
        .on_press(demo(DemoMessage::Preset(days)))
        .into()
}

pub const DATE_PICKER: Page = Page {
    id: "date-picker",
    title: "Date Picker",
    description: "A date picker component with range and presets.",
    view: |gallery| {
        let d = &gallery.demo;
        let month = |m| demo(DemoMessage::Month(m));
        let select = |date| demo(DemoMessage::Date(date));

        column![
            example("Demo", date_picker(d.date, select, d.month, month)),
            example(
                "Basic",
                container(field(
                    "Date",
                    date_picker(d.date, select, d.month, month)
                        .chevron(false)
                        .width(Fill)
                ))
                .width(176)
            ),
            example(
                "Range Picker",
                container(field(
                    "Date Picker Range",
                    date_range_picker(
                        d.range,
                        |date| demo(DemoMessage::Range(date)),
                        d.month,
                        month
                    )
                ))
                .width(240)
            ),
            example(
                "Date of Birth",
                container(field(
                    "Date of birth",
                    date_picker(d.dob, |date| demo(DemoMessage::Dob(date)), d.month, month)
                        .placeholder("Select date")
                        .format(shadcn_iced::date_picker::numeric_date)
                        .chevron(false)
                        .dropdowns(1940, 2026)
                        .open(d.dob_open, demo(DemoMessage::DobOpen(false)))
                        .on_open(demo(DemoMessage::DobOpen(true)))
                        .width(Fill)
                ))
                .width(176)
            ),
        ]
        .into()
    },
};

/// A slide of the carousel examples: a card with a square content
/// (`p-1`, `aspect-square p-6`).
fn slide(n: usize, size: f32, square: bool) -> iced::Element<'static, Message> {
    let number = container(
        text(n.to_string())
            .size(size)
            .font(shadcn_iced::font::semibold()),
    )
    .center_x(Fill)
    .center_y(Fill)
    .padding(24);

    let content: iced::Element<'static, Message> = if square {
        shadcn_iced::aspect_ratio::aspect_ratio(1.0, number).into()
    } else {
        number.height(Length::Shrink).into()
    };

    container(card().content(content)).padding(4).into()
}

pub const CAROUSEL: Page = Page {
    id: "carousel",
    title: "Carousel",
    description: "A carousel with motion and swipe built using iced.",
    view: |gallery| {
        let index = gallery.demo.slide;
        let on_change = |i| demo(DemoMessage::Slide(i));

        column![
            example(
                "Demo",
                // `max-w-xs`, plus the buttons outside of it.
                container(carousel(index, on_change).items((1..=5).map(|n| slide(n, 36.0, true))))
                    .max_width(320 + 96)
            ),
            example(
                "Sizes",
                container(
                    carousel(index, on_change)
                        .items((1..=5).map(|n| slide(n, 30.0, true)))
                        .per_view(3)
                )
                .max_width(384 + 96)
            ),
            example(
                "Spacing",
                container(
                    carousel(index, on_change)
                        .items((1..=5).map(|n| slide(n, 24.0, true)))
                        .per_view(3)
                        .spacing(4.0)
                )
                .max_width(384 + 96)
            ),
            example(
                "Orientation",
                container(
                    carousel(index, on_change)
                        .items((1..=5).map(|n| slide(n, 30.0, false)))
                        .per_view(2)
                        .orientation(shadcn_iced::carousel::Orientation::Vertical)
                )
                .max_width(320)
            ),
            example(
                "API",
                container(column![
                    carousel(index, on_change).items((1..=5).map(|n| slide(n, 36.0, true))),
                    container(
                        text(format!("Slide {} of 5", index.min(4) + 1))
                            .size(14)
                            .style(crate::muted)
                    )
                    .center_x(Fill)
                    .padding([8, 0]),
                ])
                .max_width(320 + 96)
            ),
        ]
        .into()
    },
};

const MONTHS: [&str; 6] = ["January", "February", "March", "April", "May", "June"];

pub const CHART: Page = Page {
    id: "chart",
    title: "Chart",
    description: "Beautiful charts. Built using iced's canvas.",
    view: |_| {
        let desktop = || Series::new("Desktop", [186.0, 305.0, 237.0, 73.0, 209.0, 214.0]);
        let mobile = || Series::new("Mobile", [80.0, 200.0, 120.0, 190.0, 130.0, 140.0]);
        let months = MONTHS.map(|month| &month[..3]);

        let frame = |title: &'static str,
                     description: &'static str,
                     content: iced::Element<'static, Message>| {
            card()
                .title(title)
                .description(description)
                .content(content)
        };

        column![
            example(
                "Your First Chart",
                chart::bar(months)
                    .series(desktop())
                    .series(mobile())
                    .grid(false)
                    .legend(false)
                    .tooltip(false)
            ),
            example(
                "Add a Grid",
                chart::bar(months)
                    .series(desktop())
                    .series(mobile())
                    .legend(false)
                    .tooltip(false)
            ),
            example(
                "Add Tooltip",
                chart::bar(months)
                    .series(desktop())
                    .series(mobile())
                    .legend(false)
            ),
            example(
                "Add Legend",
                chart::bar(months).series(desktop()).series(mobile())
            ),
            example(
                "Area Chart - Stacked",
                frame(
                    "Area Chart - Stacked",
                    "Showing total visitors for the last 6 months",
                    chart::area(months)
                        .series(desktop())
                        .series(mobile())
                        .stacked(true)
                        .into()
                )
            ),
            example(
                "Line Chart - Dots",
                frame(
                    "Line Chart - Dots",
                    "January - June 2024",
                    chart::line(months)
                        .series(desktop())
                        .series(mobile())
                        .dots(true)
                        .into()
                )
            ),
            example(
                "Pie Chart - Donut",
                frame(
                    "Pie Chart - Donut",
                    "January - June 2024",
                    chart::pie(
                        ["Chrome", "Safari", "Firefox", "Edge", "Other"],
                        [275.0, 200.0, 187.0, 173.0, 90.0]
                    )
                    .donut(true)
                    .into()
                )
            ),
        ]
        .into()
    },
};

/// Two lines of text in a large sidebar item
/// (`grid flex-1 text-left text-sm leading-tight`).
fn two_lines(
    title: &'static str,
    subtitle: &'static str,
    trailing: shadcn_iced::Icon,
) -> iced::Element<'static, Message> {
    row![
        column![
            text(title)
                .size(14)
                .font(shadcn_iced::font::medium())
                .line_height(1.25),
            text(subtitle).size(12).line_height(1.25),
        ]
        .width(Fill),
        trailing.sized(16),
    ]
    .align_y(Center)
    .into()
}

/// The square logo of the team switcher (`size-8 rounded-lg
/// bg-sidebar-primary`).
fn logo(icon: shadcn_iced::Icon) -> iced::Element<'static, Message> {
    container(icon.sized(16))
        .center(32)
        .style(|theme: &iced::Theme| {
            let t = shadcn_iced::tokens(theme);
            container::Style {
                background: Some(t.palette.sidebar_primary.into()),
                text_color: Some(t.palette.sidebar_primary_foreground),
                border: shadcn_iced::style::rounded(t.radius_lg()),
                ..Default::default()
            }
        })
        .into()
}

/// A collapsible item of the main navigation, with a chevron.
fn nav_item(
    title: &'static str,
    icon: shadcn_iced::Icon,
    open: bool,
    children: &[&'static str],
) -> Item<'static, Message> {
    let mut item = item(title)
        .icon(icon)
        .content(
            row![
                text(title).size(14).width(Fill),
                if open {
                    lucide::CHEVRON_DOWN.sized(16)
                } else {
                    lucide::CHEVRON_RIGHT.sized(16)
                },
            ]
            .align_y(Center),
        )
        .on_press(Message::Noop);

    if open {
        for child in children {
            item = item.child(shadcn_iced::sidebar::item(*child).on_press(Message::Noop));
        }
    }

    item
}

pub const SIDEBAR: Page = Page {
    id: "sidebar",
    title: "Sidebar",
    description: "A composable, themeable and customizable sidebar component.",
    view: |gallery| {
        let collapsed = gallery.demo.sidebar_collapsed;

        let nav = sidebar()
            .header_item(
                item("Acme Inc")
                    .size(SidebarSize::Lg)
                    .icon_element(logo(lucide::GALLERY_VERTICAL_END))
                    .content(two_lines(
                        "Acme Inc",
                        "Enterprise",
                        lucide::CHEVRONS_UP_DOWN,
                    ))
                    .on_press(Message::Noop),
            )
            .group(
                group("Platform")
                    .item(nav_item(
                        "Playground",
                        lucide::SQUARE_TERMINAL,
                        true,
                        &["History", "Starred", "Settings"],
                    ))
                    .item(nav_item("Models", lucide::BOT, false, &[]))
                    .item(nav_item("Documentation", lucide::BOOK_OPEN, false, &[]))
                    .item(nav_item("Settings", lucide::SETTINGS_2, false, &[])),
            )
            .group(
                group("Projects")
                    .item(
                        item("Design Engineering")
                            .icon(lucide::FRAME)
                            .action(menu_action(lucide::ELLIPSIS, Message::Noop))
                            .on_press(Message::Noop),
                    )
                    .item(
                        item("Sales & Marketing")
                            .icon(lucide::CHART_PIE)
                            .action(menu_action(lucide::ELLIPSIS, Message::Noop))
                            .on_press(Message::Noop),
                    )
                    .item(
                        item("Travel")
                            .icon(lucide::MAP)
                            .action(menu_action(lucide::ELLIPSIS, Message::Noop))
                            .on_press(Message::Noop),
                    )
                    .item(
                        item("More")
                            .icon(lucide::ELLIPSIS)
                            .badge("24")
                            .on_press(Message::Noop),
                    ),
            )
            .footer_item(
                item("shadcn")
                    .size(SidebarSize::Lg)
                    .icon_element(avatar("CN"))
                    .content(two_lines(
                        "shadcn",
                        "m@example.com",
                        lucide::CHEVRONS_UP_DOWN,
                    ))
                    .on_press(Message::Noop),
            )
            .collapsed(collapsed)
            .variant(shadcn_iced::sidebar::Variant::Inset);

        // `h-16 shrink-0 items-center gap-2 px-4`, then `gap-4 p-4 pt-0`
        let main = column![
            row![
                shadcn_iced::sidebar::trigger(demo(DemoMessage::SidebarCollapsed(!collapsed))),
                container(vertical_separator()).height(16),
                breadcrumb()
                    .link("Build Your Application", Message::Noop)
                    .page("Data Fetching"),
            ]
            .spacing(8)
            .height(64)
            .padding([0, 16])
            .align_y(Center),
            column![
                row![
                    skeleton(Fill, 120).radius(12.0),
                    skeleton(Fill, 120).radius(12.0),
                    skeleton(Fill, 120).radius(12.0)
                ]
                .spacing(16),
                skeleton(Fill, Fill).radius(12.0),
            ]
            .spacing(16)
            .padding(iced::Padding::new(16.0).top(0)),
        ];

        example(
            "Demo",
            container(shadcn_iced::sidebar::layout(nav, main))
                .height(560)
                .width(Fill)
                .clip(true)
                .style(|theme: &iced::Theme| {
                    let t = shadcn_iced::tokens(theme);
                    container::Style {
                        border: shadcn_iced::style::border(t.palette.border, t.radius_lg()),
                        ..Default::default()
                    }
                }),
        )
    },
};
