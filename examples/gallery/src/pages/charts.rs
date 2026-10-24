//! The chart showcase of the website (`/charts`), after
//! <https://ui.shadcn.com/charts>: one page per kind of chart, each example a
//! card with a chart.
use super::Page;
use crate::{Message, example};

use shadcn_iced::chart::{self, Chart, Curve, GridShape, Indicator, Paint, Series};
use shadcn_iced::prelude::*;

use iced::widget::{column, container, row, text};
use iced::{Center, Element, Padding};

const MONTHS: [&str; 6] = ["January", "February", "March", "April", "May", "June"];

/// `desktop: { label: "Desktop", color: "var(--chart-1)" }`
fn desktop() -> Series {
    Series::new("Desktop", [186.0, 305.0, 237.0, 73.0, 209.0, 214.0]).color(Paint::chart(1))
}

/// `mobile: { label: "Mobile", color: "var(--chart-2)" }`
fn mobile() -> Series {
    Series::new("Mobile", [80.0, 200.0, 120.0, 190.0, 130.0, 140.0]).color(Paint::chart(2))
}

const BROWSERS: [&str; 5] = ["Chrome", "Safari", "Firefox", "Edge", "Other"];
const VISITORS: [f32; 5] = [275.0, 200.0, 187.0, 173.0, 90.0];

/// The visitors of each browser, a color each.
fn visitors() -> Series {
    Series::new("Visitors", VISITORS).cells((1..=5).map(Paint::chart))
}

/// The desktop visitors of a radar, every month.
fn radar_desktop() -> Series {
    Series::new("Desktop", [186.0, 305.0, 237.0, 273.0, 209.0, 214.0])
}

const DAYS: [&str; 6] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

/// The calories burnt running and swimming every day.
fn activities() -> Chart {
    chart::bar(DAYS)
        .series(Series::new(
            "Running",
            [450.0, 380.0, 520.0, 140.0, 600.0, 480.0],
        ))
        .series(Series::new(
            "Swimming",
            [300.0, 420.0, 120.0, 550.0, 350.0, 400.0],
        ))
        .stacked(true)
        .grid(false)
        .tick_margin(10.0)
        .cursor(false)
        .legend(false)
        .default_index(1)
}

/// A chart in a card, with the footer of the charts of shadcn/ui.
fn chart_card<'a>(
    title: &'a str,
    description: &'a str,
    chart: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    let footer = column![
        row![
            text("Trending up by 5.2% this month")
                .size(14)
                .font(shadcn_iced::font::medium()),
            lucide::TRENDING_UP.sized(16),
        ]
        .spacing(8)
        .align_y(Center),
        text("Showing total visitors for the last 6 months")
            .size(14)
            .style(crate::muted),
    ]
    .spacing(8);

    card()
        .title(title)
        .description(description)
        .content(chart)
        .footer(footer)
        .width(380)
        .into()
}

/// A round chart in a card: a square of 250 in the middle
/// (`mx-auto aspect-square max-h-[250px]`).
fn round_card<'a>(title: &'a str, description: &'a str, chart: Chart) -> Element<'a, Message> {
    chart_card(
        title,
        description,
        container(chart).width(250).center_x(iced::Fill),
    )
}

/// `margin={{ left: 12, right: 12 }}`
fn sides() -> Padding {
    Padding::ZERO.left(12).right(12)
}

/// `tickFormatter={(value) => value.slice(0, 3)}`
fn month(value: &str) -> String {
    value[..3].to_owned()
}

pub const AREA: Page = Page {
    id: "charts-area",
    title: "Area Charts",
    description: "Area charts: natural, linear and step curves, stacked and expanded series, gradients, icons and axes.",
    view: |_| {
        column![
            example(
                "Area Chart",
                chart_card(
                    "Area Chart",
                    "Showing total visitors for the last 6 months",
                    chart::area(MONTHS)
                        .series(desktop())
                        .margin(sides())
                        .tick_formatter(month)
                        .cursor(false)
                        .tooltip_indicator(Indicator::Line)
                        .legend(false)
                )
            ),
            example(
                "Area Chart - Linear",
                chart_card(
                    "Area Chart - Linear",
                    "Showing total visitors for the last 6 months",
                    chart::area(MONTHS)
                        .series(desktop())
                        .curve(Curve::Linear)
                        .margin(sides())
                        .tick_formatter(month)
                        .cursor(false)
                        .tooltip_label(false)
                        .legend(false)
                )
            ),
            example(
                "Area Chart - Step",
                chart_card(
                    "Area Chart - Step",
                    "Showing total visitors for the last 6 months",
                    chart::area(MONTHS)
                        .series(desktop().icon(lucide::ACTIVITY))
                        .curve(Curve::Step)
                        .margin(sides())
                        .tick_formatter(month)
                        .cursor(false)
                        .tooltip_label(false)
                        .legend(false)
                )
            ),
            example(
                "Area Chart - Legend",
                chart_card(
                    "Area Chart - Legend",
                    "Showing total visitors for the last 6 months",
                    chart::area(MONTHS)
                        .series(mobile())
                        .series(desktop())
                        .stacked(true)
                        .margin(sides())
                        .tick_formatter(month)
                        .cursor(false)
                        .tooltip_indicator(Indicator::Line)
                )
            ),
            example(
                "Area Chart - Stacked",
                chart_card(
                    "Area Chart - Stacked",
                    "Showing total visitors for the last 6 months",
                    chart::area(MONTHS)
                        .series(mobile())
                        .series(desktop())
                        .stacked(true)
                        .margin(sides())
                        .tick_formatter(month)
                        .cursor(false)
                        .legend(false)
                )
            ),
            example(
                "Area Chart - Stacked Expanded",
                chart_card(
                    "Area Chart - Stacked Expanded",
                    "Showing total visitors for the last 6 months",
                    chart::area(MONTHS)
                        .series(
                            Series::new("Other", [45.0, 100.0, 150.0, 50.0, 100.0, 160.0])
                                .color(Paint::chart(3))
                                .fill_opacity(0.1)
                        )
                        .series(mobile())
                        .series(desktop())
                        .expanded(true)
                        .margin(sides().top(12))
                        .tick_formatter(month)
                        .cursor(false)
                        .tooltip_indicator(Indicator::Line)
                        .legend(false)
                )
            ),
            example(
                "Area Chart - Icons",
                chart_card(
                    "Area Chart - Icons",
                    "Showing total visitors for the last 6 months",
                    chart::area(MONTHS)
                        .series(mobile().icon(lucide::TRENDING_UP))
                        .series(desktop().icon(lucide::TRENDING_DOWN))
                        .stacked(true)
                        .margin(sides())
                        .tick_formatter(month)
                        .cursor(false)
                        .tooltip_indicator(Indicator::Line)
                )
            ),
            example(
                "Area Chart - Gradient",
                chart_card(
                    "Area Chart - Gradient",
                    "Showing total visitors for the last 6 months",
                    chart::area(MONTHS)
                        .series(mobile())
                        .series(desktop())
                        .stacked(true)
                        .gradient(true)
                        .margin(sides())
                        .tick_formatter(month)
                        .cursor(false)
                        .legend(false)
                )
            ),
            example(
                "Area Chart - Axes",
                chart_card(
                    "Area Chart - Axes",
                    "Showing total visitors for the last 6 months",
                    chart::area(MONTHS)
                        .series(mobile())
                        .series(desktop())
                        .stacked(true)
                        .value_axis(true)
                        .value_ticks(3)
                        .margin(Padding::ZERO.left(-20.0).right(12))
                        .tick_formatter(month)
                        .cursor(false)
                        .legend(false)
                )
            ),
        ]
        .into()
    },
};

pub const BAR: Page = Page {
    id: "charts-bar",
    title: "Bar Charts",
    description: "Bar charts: vertical and horizontal, multiple and stacked series, labels, colors per bar and negative values.",
    view: |_| {
        column![
            example(
                "Bar Chart",
                chart_card(
                    "Bar Chart",
                    "January - June 2024",
                    chart::bar(MONTHS)
                        .series(desktop())
                        .radius(8.0)
                        .tick_margin(10.0)
                        .tick_formatter(month)
                        .cursor(false)
                        .tooltip_label(false)
                        .legend(false)
                )
            ),
            example(
                "Bar Chart - Horizontal",
                chart_card(
                    "Bar Chart - Horizontal",
                    "January - June 2024",
                    chart::bar(MONTHS)
                        .series(desktop())
                        .horizontal(true)
                        .grid(false)
                        .radius(5.0)
                        .margin(Padding::ZERO.left(-20.0))
                        .tick_margin(10.0)
                        .tick_formatter(month)
                        .cursor(false)
                        .tooltip_label(false)
                        .legend(false)
                )
            ),
            example(
                "Bar Chart - Multiple",
                chart_card(
                    "Bar Chart - Multiple",
                    "January - June 2024",
                    chart::bar(MONTHS)
                        .series(desktop())
                        .series(mobile())
                        .tick_margin(10.0)
                        .tick_formatter(month)
                        .cursor(false)
                        .tooltip_indicator(Indicator::Dashed)
                        .legend(false)
                )
            ),
            example(
                "Bar Chart - Stacked + Legend",
                chart_card(
                    "Bar Chart - Stacked + Legend",
                    "January - June 2024",
                    chart::bar(MONTHS)
                        .series(desktop())
                        .series(mobile())
                        .stacked(true)
                        .tick_margin(10.0)
                        .tick_formatter(month)
                        .tooltip_label(false)
                )
            ),
            example(
                "Bar Chart - Label",
                chart_card(
                    "Bar Chart - Label",
                    "January - June 2024",
                    chart::bar(MONTHS)
                        .series(desktop())
                        .radius(8.0)
                        .value_labels(true)
                        .margin(Padding::ZERO.top(20))
                        .tick_margin(10.0)
                        .tick_formatter(month)
                        .cursor(false)
                        .tooltip_label(false)
                        .legend(false)
                )
            ),
            example(
                "Bar Chart - Custom Label",
                chart_card(
                    "Bar Chart - Custom Label",
                    "January - June 2024",
                    chart::bar(MONTHS)
                        .series(desktop().color(Paint::chart(2)))
                        .horizontal(true)
                        .category_axis(false)
                        .category_labels(true)
                        .value_labels(true)
                        .margin(Padding::ZERO.right(16))
                        .cursor(false)
                        .tooltip_indicator(Indicator::Line)
                        .legend(false)
                )
            ),
            example(
                "Bar Chart - Mixed",
                chart_card(
                    "Bar Chart - Mixed",
                    "January - June 2024",
                    chart::bar(BROWSERS)
                        .series(visitors())
                        .horizontal(true)
                        .grid(false)
                        .radius(5.0)
                        .margin(Padding::ZERO)
                        .tick_margin(10.0)
                        .cursor(false)
                        .tooltip_label(false)
                        .legend(false)
                )
            ),
            example(
                "Bar Chart - Active",
                chart_card(
                    "Bar Chart - Active",
                    "January - June 2024",
                    chart::bar(BROWSERS)
                        .series(
                            Series::new("Visitors", [187.0, 200.0, 275.0, 173.0, 90.0])
                                .cells((1..=5).map(Paint::chart))
                        )
                        .radius(8.0)
                        .highlight(2)
                        .tick_margin(10.0)
                        .cursor(false)
                        .tooltip_label(false)
                        .legend(false)
                )
            ),
            example("Bar Chart - Negative", {
                let values = [186.0, 205.0, -207.0, 173.0, -209.0, 214.0];
                let colors = values.map(|value| {
                    if value > 0.0 {
                        Paint::chart(1)
                    } else {
                        Paint::chart(2)
                    }
                });

                chart_card(
                    "Bar Chart - Negative",
                    "January - June 2024",
                    chart::bar(MONTHS)
                        .series(Series::new("Visitors", values).cells(colors))
                        .radius(0.0)
                        .category_axis(false)
                        .category_labels(true)
                        .cursor(false)
                        .tooltip_label(false)
                        .tooltip_indicator(Indicator::None)
                        .legend(false),
                )
            }),
        ]
        .into()
    },
};

pub const LINE: Page = Page {
    id: "charts-line",
    title: "Line Charts",
    description: "Line charts: natural, linear, step and monotone curves, dots and labels.",
    view: |_| {
        column![
            example(
                "Line Chart",
                chart_card(
                    "Line Chart",
                    "January - June 2024",
                    chart::line(MONTHS)
                        .series(desktop())
                        .margin(sides())
                        .tick_formatter(month)
                        .cursor(false)
                        .tooltip_label(false)
                        .legend(false)
                )
            ),
            example(
                "Line Chart - Linear",
                chart_card(
                    "Line Chart - Linear",
                    "January - June 2024",
                    chart::line(MONTHS)
                        .series(desktop())
                        .curve(Curve::Linear)
                        .margin(sides())
                        .tick_formatter(month)
                        .cursor(false)
                        .tooltip_label(false)
                        .legend(false)
                )
            ),
            example(
                "Line Chart - Step",
                chart_card(
                    "Line Chart - Step",
                    "January - June 2024",
                    chart::line(MONTHS)
                        .series(desktop())
                        .curve(Curve::Step)
                        .margin(sides())
                        .tick_formatter(month)
                        .cursor(false)
                        .tooltip_label(false)
                        .legend(false)
                )
            ),
            example(
                "Line Chart - Multiple",
                chart_card(
                    "Line Chart - Multiple",
                    "January - June 2024",
                    chart::line(MONTHS)
                        .series(desktop())
                        .series(mobile())
                        .curve(Curve::Monotone)
                        .margin(sides())
                        .tick_formatter(month)
                        .cursor(false)
                        .legend(false)
                )
            ),
            example(
                "Line Chart - Dots",
                chart_card(
                    "Line Chart - Dots",
                    "January - June 2024",
                    chart::line(MONTHS)
                        .series(desktop())
                        .dots(true)
                        .active_dot(6.0)
                        .margin(sides())
                        .tick_formatter(month)
                        .cursor(false)
                        .tooltip_label(false)
                        .legend(false)
                )
            ),
            example(
                "Line Chart - Dots Colors",
                chart_card(
                    "Line Chart - Dots Colors",
                    "January - June 2024",
                    chart::line(BROWSERS)
                        .series(visitors().color(Paint::chart(2)))
                        .dots(true)
                        .dot_radius(5.0)
                        .category_axis(false)
                        .margin(Padding::new(24.0).bottom(0))
                        .cursor(false)
                        .tooltip_label(false)
                        .tooltip_indicator(Indicator::Line)
                        .legend(false)
                )
            ),
            example(
                "Line Chart - Label",
                chart_card(
                    "Line Chart - Label",
                    "January - June 2024",
                    chart::line(MONTHS)
                        .series(desktop())
                        .dots(true)
                        .active_dot(6.0)
                        .value_labels(true)
                        .margin(sides().top(20))
                        .tick_formatter(month)
                        .cursor(false)
                        .tooltip_indicator(Indicator::Line)
                        .legend(false)
                )
            ),
            example(
                "Line Chart - Custom Label",
                chart_card(
                    "Line Chart - Custom Label",
                    "January - June 2024",
                    chart::line(BROWSERS)
                        .series(Series::new("Visitors", VISITORS).color(Paint::chart(2)))
                        .dots(true)
                        .active_dot(6.0)
                        .category_labels(true)
                        .category_axis(false)
                        .margin(Padding::new(24.0).bottom(0))
                        .cursor(false)
                        .tooltip_label(false)
                        .tooltip_indicator(Indicator::Line)
                        .legend(false)
                )
            ),
        ]
        .into()
    },
};

pub const PIE: Page = Page {
    id: "charts-pie",
    title: "Pie Charts",
    description: "Pie and donut charts: labels, legends, an active slice, a text in the middle and nested rings.",
    view: |_| {
        column![
            example(
                "Pie Chart",
                round_card(
                    "Pie Chart",
                    "January - June 2024",
                    chart::pie(BROWSERS, VISITORS).legend(false)
                )
            ),
            example(
                "Pie Chart - Label",
                round_card(
                    "Pie Chart - Label",
                    "January - June 2024",
                    chart::pie(BROWSERS, VISITORS)
                        .value_labels(true)
                        .legend(false)
                )
            ),
            example(
                "Pie Chart - Custom Label",
                round_card(
                    "Pie Chart - Custom Label",
                    "January - June 2024",
                    chart::pie(BROWSERS, VISITORS)
                        .value_labels(true)
                        .label_lines(false)
                        .legend(false)
                )
            ),
            example(
                "Pie Chart - Label List",
                round_card(
                    "Pie Chart - Label List",
                    "January - June 2024",
                    chart::pie(BROWSERS, VISITORS)
                        .category_labels(true)
                        .legend(false)
                )
            ),
            example(
                "Pie Chart - Legend",
                round_card(
                    "Pie Chart - Legend",
                    "January - June 2024",
                    chart::pie(BROWSERS, VISITORS).tooltip(false)
                )
            ),
            example(
                "Pie Chart - Donut",
                round_card(
                    "Pie Chart - Donut",
                    "January - June 2024",
                    chart::pie(BROWSERS, VISITORS).donut(true).legend(false)
                )
            ),
            example(
                "Pie Chart - Donut Active",
                round_card(
                    "Pie Chart - Donut Active",
                    "January - June 2024",
                    chart::pie(BROWSERS, VISITORS)
                        .donut(true)
                        .highlight(0)
                        .legend(false)
                )
            ),
            example(
                "Pie Chart - Donut with Text",
                round_card(
                    "Pie Chart - Donut with Text",
                    "January - June 2024",
                    chart::pie(BROWSERS, VISITORS)
                        .donut(true)
                        .center_label("925", "Visitors")
                        .legend(false)
                )
            ),
            example(
                "Pie Chart - Stacked",
                round_card(
                    "Pie Chart - Stacked",
                    "January - June 2024",
                    chart::rings(["January", "February", "March", "April", "May"])
                        .series(
                            Series::new("Desktop", [186.0, 305.0, 237.0, 173.0, 209.0])
                                .radii(0.0, 60.0)
                        )
                        .series(
                            Series::new("Mobile", [80.0, 200.0, 120.0, 190.0, 130.0])
                                .radii(70.0, 90.0)
                        )
                        .tooltip_label(true)
                        .tooltip_indicator(Indicator::Line)
                        .legend(false)
                )
            ),
        ]
        .into()
    },
};

pub const RADAR: Page = Page {
    id: "charts-radar",
    title: "Radar Charts",
    description: "Radar charts: dots, multiple series, lines only, a radius axis, and polygon or circle grids.",
    view: |_| {
        column![
            example(
                "Radar Chart",
                round_card(
                    "Radar Chart",
                    "Showing total visitors for the last 6 months",
                    chart::radar(MONTHS)
                        .series(radar_desktop())
                        .cursor(false)
                        .legend(false)
                )
            ),
            example(
                "Radar Chart - Dots",
                round_card(
                    "Radar Chart - Dots",
                    "Showing total visitors for the last 6 months",
                    chart::radar(MONTHS)
                        .series(radar_desktop())
                        .dots(true)
                        .cursor(false)
                        .legend(false)
                )
            ),
            example(
                "Radar Chart - Multiple",
                round_card(
                    "Radar Chart - Multiple",
                    "Showing total visitors for the last 6 months",
                    chart::radar(MONTHS)
                        .series(desktop())
                        .series(mobile().fill_opacity(1.0))
                        .cursor(false)
                        .tooltip_indicator(Indicator::Line)
                        .legend(false)
                )
            ),
            example(
                "Radar Chart - Lines Only",
                round_card(
                    "Radar Chart - Lines Only",
                    "Showing total visitors for the last 6 months",
                    chart::radar(MONTHS)
                        .series(
                            Series::new("Desktop", [186.0, 185.0, 207.0, 173.0, 160.0, 174.0])
                                .fill_opacity(0.0)
                        )
                        .series(
                            Series::new("Mobile", [160.0, 170.0, 180.0, 160.0, 190.0, 204.0])
                                .fill_opacity(0.0)
                        )
                        .stroke_width(2.0)
                        .radial_lines(false)
                        .cursor(false)
                        .tooltip_indicator(Indicator::Line)
                        .legend(false)
                )
            ),
            example(
                "Radar Chart - Radius Axis",
                round_card(
                    "Radar Chart - Radius Axis",
                    "Showing total visitors for the last 6 months",
                    chart::radar(MONTHS)
                        .series(desktop())
                        .series(mobile().fill_opacity(1.0))
                        .category_axis(false)
                        .radius_axis(60.0)
                        .cursor(false)
                        .tooltip_label(false)
                        .tooltip_indicator(Indicator::Line)
                        .legend(false)
                )
            ),
            example(
                "Radar Chart - Grid Custom",
                round_card(
                    "Radar Chart - Grid Custom",
                    "Showing total visitors for the last 6 months",
                    chart::radar(MONTHS)
                        .series(radar_desktop())
                        .radial_lines(false)
                        .grid_radii([90.0])
                        .cursor(false)
                        .tooltip_label(false)
                        .legend(false)
                )
            ),
            example(
                "Radar Chart - Grid Filled",
                round_card(
                    "Radar Chart - Grid Filled",
                    "Showing total visitors for the last 6 months",
                    chart::radar(MONTHS)
                        .series(Series::new(
                            "Desktop",
                            [186.0, 285.0, 237.0, 203.0, 209.0, 264.0]
                        ))
                        .grid_fill(true)
                        .fill_opacity(0.5)
                        .cursor(false)
                        .tooltip_label(false)
                        .legend(false)
                )
            ),
            example(
                "Radar Chart - Grid None",
                round_card(
                    "Radar Chart - Grid None",
                    "Showing total visitors for the last 6 months",
                    chart::radar(MONTHS)
                        .series(radar_desktop())
                        .grid(false)
                        .dots(true)
                        .cursor(false)
                        .tooltip_label(false)
                        .legend(false)
                )
            ),
            example(
                "Radar Chart - Grid Circle",
                round_card(
                    "Radar Chart - Grid Circle",
                    "Showing total visitors for the last 6 months",
                    chart::radar(MONTHS)
                        .series(radar_desktop())
                        .grid_shape(GridShape::Circle)
                        .dots(true)
                        .cursor(false)
                        .tooltip_label(false)
                        .legend(false)
                )
            ),
            example(
                "Radar Chart - Grid Circle - No lines",
                round_card(
                    "Radar Chart - Grid Circle - No lines",
                    "Showing total visitors for the last 6 months",
                    chart::radar(MONTHS)
                        .series(radar_desktop())
                        .grid_shape(GridShape::Circle)
                        .radial_lines(false)
                        .dots(true)
                        .cursor(false)
                        .tooltip_label(false)
                        .legend(false)
                )
            ),
            example(
                "Radar Chart - Grid Circle Filled",
                round_card(
                    "Radar Chart - Grid Circle Filled",
                    "Showing total visitors for the last 6 months",
                    chart::radar(MONTHS)
                        .series(Series::new(
                            "Desktop",
                            [186.0, 285.0, 237.0, 203.0, 209.0, 264.0]
                        ))
                        .grid_shape(GridShape::Circle)
                        .grid_fill(true)
                        .fill_opacity(0.5)
                        .cursor(false)
                        .legend(false)
                )
            ),
            example(
                "Radar Chart - Legend",
                round_card(
                    "Radar Chart - Legend",
                    "Showing total visitors for the last 6 months",
                    chart::radar(MONTHS)
                        .series(desktop())
                        .series(mobile().fill_opacity(1.0))
                        .cursor(false)
                        .tooltip_indicator(Indicator::Line)
                )
            ),
            example(
                "Radar Chart - Icons",
                round_card(
                    "Radar Chart - Icons",
                    "Showing total visitors for the last 6 months",
                    chart::radar(MONTHS)
                        .series(desktop().icon(lucide::ARROW_DOWN_FROM_LINE))
                        .series(mobile().icon(lucide::ARROW_UP_FROM_LINE).fill_opacity(1.0))
                        .cursor(false)
                        .tooltip_indicator(Indicator::Line)
                )
            ),
        ]
        .into()
    },
};

pub const RADIAL: Page = Page {
    id: "charts-radial",
    title: "Radial Charts",
    description: "Radial bar charts: backgrounds, labels, grids, a text in the middle and stacked series.",
    view: |_| {
        column![
            example(
                "Radial Chart",
                round_card(
                    "Radial Chart",
                    "January - June 2024",
                    chart::radial(BROWSERS)
                        .series(Series::new("Visitors", VISITORS))
                        .inner_radius(30.0)
                        .outer_radius(110.0)
                        .background(true)
                        .cursor(false)
                        .legend(false)
                )
            ),
            example(
                "Radial Chart - Label",
                round_card(
                    "Radial Chart - Label",
                    "January - June 2024",
                    chart::radial(BROWSERS)
                        .series(Series::new("Visitors", VISITORS))
                        .angles(-90.0, 380.0)
                        .inner_radius(30.0)
                        .outer_radius(110.0)
                        .background(true)
                        .category_labels(true)
                        .cursor(false)
                        .legend(false)
                )
            ),
            example(
                "Radial Chart - Grid",
                round_card(
                    "Radial Chart - Grid",
                    "January - June 2024",
                    chart::radial(BROWSERS)
                        .series(Series::new("Visitors", VISITORS))
                        .inner_radius(30.0)
                        .outer_radius(100.0)
                        .grid(true)
                        .cursor(false)
                        .legend(false)
                )
            ),
            example(
                "Radial Chart - Text",
                round_card(
                    "Radial Chart - Text",
                    "January - June 2024",
                    chart::radial(["Safari"])
                        .series(Series::new("Visitors", [200.0]).cells([Paint::chart(2)]))
                        .angles(0.0, 250.0)
                        .inner_radius(80.0)
                        .outer_radius(90.0)
                        .track(80.0, 90.0)
                        .background(true)
                        .corner_radius(10.0)
                        .center_label("200", "Visitors")
                        .center_label_size(36.0)
                        .tooltip(false)
                        .legend(false)
                )
            ),
            example(
                "Radial Chart - Shape",
                round_card(
                    "Radial Chart - Shape",
                    "January - June 2024",
                    chart::radial(["Safari"])
                        .series(Series::new("Visitors", [1260.0]).cells([Paint::chart(2)]))
                        .angles(0.0, 100.0)
                        .inner_radius(65.0)
                        .outer_radius(95.0)
                        .track(74.0, 86.0)
                        .background(true)
                        .center_label("1,260", "Visitors")
                        .center_label_size(36.0)
                        .tooltip(false)
                        .legend(false)
                )
            ),
            example(
                "Radial Chart - Stacked",
                round_card(
                    "Radial Chart - Stacked",
                    "January - June 2024",
                    chart::radial(["January"])
                        .series(Series::new("Mobile", [570.0]).color(Paint::chart(2)))
                        .series(Series::new("Desktop", [1260.0]).color(Paint::chart(1)))
                        .stacked(true)
                        .angles(0.0, 180.0)
                        .inner_radius(80.0)
                        .outer_radius(110.0)
                        .corner_radius(5.0)
                        .center_label("1,830", "Visitors")
                        .center_label_size(24.0)
                        .center_label_offset(-24.0)
                        .cursor(false)
                        .tooltip_label(false)
                        .legend(false)
                )
            ),
        ]
        .into()
    },
};

pub const TOOLTIP: Page = Page {
    id: "charts-tooltip",
    title: "Tooltips",
    description: "Tooltips: indicators, labels, formatters, units, icons and totals.",
    view: |_| {
        column![
            example(
                "Tooltip - Default",
                chart_card(
                    "Tooltip - Default",
                    "Default tooltip with ChartTooltipContent.",
                    activities()
                )
            ),
            example(
                "Tooltip - Line Indicator",
                chart_card(
                    "Tooltip - Line Indicator",
                    "Tooltip with line indicator.",
                    activities().tooltip_indicator(Indicator::Line)
                )
            ),
            example(
                "Tooltip - No Indicator",
                chart_card(
                    "Tooltip - No Indicator",
                    "Tooltip with no indicator.",
                    activities().tooltip_indicator(Indicator::None)
                )
            ),
            example(
                "Tooltip - Custom label",
                chart_card(
                    "Tooltip - Custom label",
                    "Tooltip with custom label from chartConfig.",
                    activities()
                        .tooltip_label_formatter(|_| "Activities".to_owned())
                        .tooltip_indicator(Indicator::Line)
                )
            ),
            example(
                "Tooltip - Label Formatter",
                chart_card(
                    "Tooltip - Label Formatter",
                    "Tooltip with label formatter.",
                    activities().tooltip_label_formatter(|day| format!("{day}, July 16, 2024"))
                )
            ),
            example(
                "Tooltip - No Label",
                chart_card(
                    "Tooltip - No Label",
                    "Tooltip with no label.",
                    activities()
                        .tooltip_label(false)
                        .tooltip_indicator(Indicator::None)
                )
            ),
            example(
                "Tooltip - Formatter",
                chart_card(
                    "Tooltip - Formatter",
                    "Tooltip with custom formatter .",
                    activities()
                        .tooltip_label(false)
                        .tooltip_indicator(Indicator::None)
                        .tooltip_unit("kcal")
                )
            ),
            example(
                "Tooltip - Icons",
                chart_card(
                    "Tooltip - Icons",
                    "Tooltip with icons.",
                    chart::bar(DAYS)
                        .series(
                            Series::new("Running", [450.0, 380.0, 520.0, 140.0, 600.0, 480.0])
                                .icon(lucide::FOOTPRINTS)
                        )
                        .series(
                            Series::new("Swimming", [300.0, 420.0, 120.0, 550.0, 350.0, 400.0])
                                .icon(lucide::WAVES)
                        )
                        .stacked(true)
                        .grid(false)
                        .tick_margin(10.0)
                        .cursor(false)
                        .tooltip_label(false)
                        .default_index(1)
                        .legend(false)
                )
            ),
            example(
                "Tooltip - Advanced",
                chart_card(
                    "Tooltip - Advanced",
                    "Tooltip with custom formatter and total.",
                    activities()
                        .tooltip_label(false)
                        .tooltip_unit("kcal")
                        .tooltip_total(true)
                )
            ),
        ]
        .into()
    },
};
