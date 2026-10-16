//! Pages of the calendar, charts, carousel and sidebar.
use super::{DemoMessage, Page};
use crate::{Message, example};

use iced_ui::button::{Size as ButtonSize, Variant as ButtonVariant};
use iced_ui::calendar::Date;
use iced_ui::chart::{self, Series};
use iced_ui::date_picker::date_range_picker;
use iced_ui::prelude::*;
use iced_ui::sidebar::{group, item};

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
                    .footer_border(true)
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
                    .footer_border(true)
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
                        .format(iced_ui::date_picker::numeric_date)
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

pub const CAROUSEL: Page = Page {
    id: "carousel",
    title: "Carousel",
    description: "A carousel with motion and swipe built using iced.",
    view: |gallery| {
        let slide = |n: usize| -> iced::Element<'static, Message> {
            card()
                .content(
                    container(text(n.to_string()).size(36).font(iced_ui::font::SEMIBOLD))
                        .center_x(Fill)
                        .center_y(160),
                )
                .into()
        };

        column![
            example(
                "Default",
                container(
                    carousel(gallery.demo.slide, |i| demo(DemoMessage::Slide(i)))
                        .items((1..=5).map(slide))
                )
                .max_width(380)
            ),
            example(
                "Multiple per view",
                container(
                    carousel(gallery.demo.slide, |i| demo(DemoMessage::Slide(i)))
                        .items((1..=5).map(slide))
                        .per_view(3)
                )
                .max_width(560)
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

        let frame = |content: iced::Element<'static, Message>| card().content(content);

        column![
            example(
                "Bar",
                frame(chart::bar(MONTHS).series(desktop()).series(mobile()).into())
            ),
            example(
                "Area (stacked)",
                frame(
                    chart::area(MONTHS)
                        .series(desktop())
                        .series(mobile())
                        .stacked(true)
                        .into()
                )
            ),
            example(
                "Line",
                frame(
                    chart::line(MONTHS)
                        .series(desktop())
                        .series(mobile())
                        .dots(true)
                        .into()
                )
            ),
            example(
                "Pie",
                frame(
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

pub const SIDEBAR: Page = Page {
    id: "sidebar",
    title: "Sidebar",
    description: "A composable, themeable and customizable sidebar component.",
    view: |gallery| {
        let collapsed = gallery.demo.sidebar_collapsed;

        let nav = sidebar()
            .header(
                row![
                    container(lucide::GALLERY_VERTICAL_END.sized(16))
                        .center(32)
                        .style(|theme: &iced::Theme| {
                            let t = iced_ui::tokens(theme);
                            container::Style {
                                background: Some(t.palette.sidebar_primary.into()),
                                text_color: Some(t.palette.sidebar_primary_foreground),
                                border: iced_ui::style::rounded(t.radius_lg()),
                                ..Default::default()
                            }
                        }),
                    column![
                        text("Acme Inc").size(14).font(iced_ui::font::SEMIBOLD),
                        text("Enterprise").size(12),
                    ],
                ]
                .spacing(8)
                .align_y(Center),
            )
            .group(
                group("Platform")
                    .item(
                        item("Playground")
                            .icon(lucide::SQUARE_TERMINAL)
                            .active(true)
                            .on_press(Message::Noop)
                            .child(item("History").on_press(Message::Noop))
                            .child(item("Starred").on_press(Message::Noop)),
                    )
                    .item(item("Models").icon(lucide::BOT).on_press(Message::Noop))
                    .item(
                        item("Documentation")
                            .icon(lucide::BOOK_OPEN)
                            .on_press(Message::Noop),
                    )
                    .item(
                        item("Settings")
                            .icon(lucide::SETTINGS_2)
                            .on_press(Message::Noop),
                    ),
            )
            .group(
                group("Projects")
                    .item(
                        item("Design Engineering")
                            .icon(lucide::FRAME)
                            .badge("12")
                            .on_press(Message::Noop),
                    )
                    .item(
                        item("Sales & Marketing")
                            .icon(lucide::CHART_PIE)
                            .on_press(Message::Noop),
                    )
                    .item(item("Travel").icon(lucide::MAP).on_press(Message::Noop)),
            )
            .footer(item_footer())
            .collapsed(collapsed)
            .variant(iced_ui::sidebar::Variant::Inset);

        let main = column![
            row![
                iced_ui::sidebar::trigger(demo(DemoMessage::SidebarCollapsed(!collapsed))),
                breadcrumb()
                    .link("Building Your Application", Message::Noop)
                    .page("Data Fetching"),
            ]
            .spacing(8)
            .align_y(Center),
            row![
                skeleton(Fill, 100).radius(12.0),
                skeleton(Fill, 100).radius(12.0),
                skeleton(Fill, 100).radius(12.0)
            ]
            .spacing(16),
            skeleton(Fill, Fill).radius(12.0),
        ]
        .spacing(16)
        .padding(16);

        example(
            "Inset",
            container(iced_ui::sidebar::layout(nav, main))
                .height(480)
                .width(Fill)
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

fn item_footer() -> iced::Element<'static, Message> {
    row![
        avatar("CN"),
        column![
            text("shadcn").size(14).font(iced_ui::font::SEMIBOLD),
            text("m@example.com").size(12)
        ]
        .width(Fill),
        lucide::CHEVRONS_UP_DOWN.sized(16),
    ]
    .spacing(8)
    .align_y(Center)
    .padding(8)
    .into()
}
