//! The chart showcase of the website (`/charts`), after
//! <https://ui.shadcn.com/charts>: one page per kind of chart, each example a
//! card with a chart.
use super::Page;
use crate::{Message, example};

use shadcn_iced::chart::{self, Series};
use shadcn_iced::prelude::*;

use iced::widget::{column, row, text};
use iced::{Center, Element};

const MONTHS: [&str; 6] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun"];

fn desktop() -> Series {
    Series::new("Desktop", [186.0, 305.0, 237.0, 73.0, 209.0, 214.0])
}

fn mobile() -> Series {
    Series::new("Mobile", [80.0, 200.0, 120.0, 190.0, 130.0, 140.0])
}

const BROWSERS: [&str; 5] = ["Chrome", "Safari", "Firefox", "Edge", "Other"];
const VISITORS: [f32; 5] = [275.0, 200.0, 187.0, 173.0, 90.0];

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

pub const AREA: Page = Page {
    id: "charts-area",
    title: "Area Charts",
    description: "Area charts: single, multiple and stacked series.",
    view: |_| {
        column![
            example(
                "Area Chart",
                chart_card(
                    "Area Chart",
                    "January - June 2024",
                    chart::area(MONTHS)
                        .margin([5, 12])
                        .series(desktop())
                        .legend(false)
                )
            ),
            example(
                "Area Chart - Multiple",
                chart_card(
                    "Area Chart - Multiple",
                    "January - June 2024",
                    chart::area(MONTHS)
                        .margin([5, 12])
                        .series(desktop())
                        .series(mobile())
                        .legend(false)
                )
            ),
            example(
                "Area Chart - Stacked",
                chart_card(
                    "Area Chart - Stacked",
                    "January - June 2024",
                    chart::area(MONTHS)
                        .margin([5, 12])
                        .series(desktop())
                        .series(mobile())
                        .stacked(true)
                        .legend(false)
                )
            ),
            example(
                "Area Chart - Legend",
                chart_card(
                    "Area Chart - Legend",
                    "January - June 2024",
                    chart::area(MONTHS)
                        .margin([5, 12])
                        .series(desktop())
                        .series(mobile())
                        .stacked(true)
                )
            ),
        ]
        .into()
    },
};

pub const BAR: Page = Page {
    id: "charts-bar",
    title: "Bar Charts",
    description: "Bar charts: single, multiple and stacked series.",
    view: |_| {
        column![
            example(
                "Bar Chart",
                chart_card(
                    "Bar Chart",
                    "January - June 2024",
                    chart::bar(MONTHS).series(desktop()).legend(false)
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
                )
            ),
            example(
                "Bar Chart - No Grid",
                chart_card(
                    "Bar Chart - No Grid",
                    "January - June 2024",
                    chart::bar(MONTHS)
                        .series(desktop())
                        .grid(false)
                        .cursor(false)
                        .legend(false)
                )
            ),
        ]
        .into()
    },
};

pub const LINE: Page = Page {
    id: "charts-line",
    title: "Line Charts",
    description: "Line charts: single and multiple series, with or without dots.",
    view: |_| {
        column![
            example(
                "Line Chart",
                chart_card(
                    "Line Chart",
                    "January - June 2024",
                    chart::line(MONTHS)
                        .margin([5, 12])
                        .series(desktop())
                        .legend(false)
                )
            ),
            example(
                "Line Chart - Multiple",
                chart_card(
                    "Line Chart - Multiple",
                    "January - June 2024",
                    chart::line(MONTHS)
                        .margin([5, 12])
                        .series(desktop())
                        .series(mobile())
                        .legend(false)
                )
            ),
            example(
                "Line Chart - Dots",
                chart_card(
                    "Line Chart - Dots",
                    "January - June 2024",
                    chart::line(MONTHS)
                        .margin([5, 12])
                        .series(desktop())
                        .series(mobile())
                        .dots(true)
                        .legend(false)
                )
            ),
            example(
                "Line Chart - Legend",
                chart_card(
                    "Line Chart - Legend",
                    "January - June 2024",
                    chart::line(MONTHS)
                        .margin([5, 12])
                        .series(desktop())
                        .series(mobile())
                        .dots(true)
                )
            ),
        ]
        .into()
    },
};

pub const PIE: Page = Page {
    id: "charts-pie",
    title: "Pie Charts",
    description: "Pie and donut charts.",
    view: |_| {
        column![
            example(
                "Pie Chart",
                chart_card(
                    "Pie Chart",
                    "January - June 2024",
                    chart::pie(BROWSERS, VISITORS).legend(false)
                )
            ),
            example(
                "Pie Chart - Donut",
                chart_card(
                    "Pie Chart - Donut",
                    "January - June 2024",
                    chart::pie(BROWSERS, VISITORS).donut(true).legend(false)
                )
            ),
            example(
                "Pie Chart - Legend",
                chart_card(
                    "Pie Chart - Legend",
                    "January - June 2024",
                    chart::pie(BROWSERS, VISITORS)
                )
            ),
            example(
                "Pie Chart - Donut with Legend",
                chart_card(
                    "Pie Chart - Donut with Legend",
                    "January - June 2024",
                    chart::pie(BROWSERS, VISITORS).donut(true)
                )
            ),
        ]
        .into()
    },
};
