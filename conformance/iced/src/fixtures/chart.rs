use super::Fixture;
use crate::store::Msg;

use shadcn_iced::chart::{self, Chart, Curve, GridShape, Indicator, Paint, Series};
use shadcn_iced::icon::lucide;

const MONTHS: [&str; 6] = ["January", "February", "March", "April", "May", "June"];
const BROWSERS: [&str; 5] = ["Chrome", "Safari", "Firefox", "Edge", "Other"];
const VISITORS: [f32; 5] = [275.0, 200.0, 187.0, 173.0, 90.0];
const DATES: [&str; 6] = [
    "2024-07-15",
    "2024-07-16",
    "2024-07-17",
    "2024-07-18",
    "2024-07-19",
    "2024-07-20",
];

/// `desktop: { color: "var(--chart-1)" }`
fn desktop() -> Series {
    Series::new("Desktop", [186.0, 305.0, 237.0, 73.0, 209.0, 214.0]).color(Paint::chart(1))
}

/// `mobile: { color: "var(--chart-2)" }`
fn mobile() -> Series {
    Series::new("Mobile", [80.0, 200.0, 120.0, 190.0, 130.0, 140.0]).color(Paint::chart(2))
}

/// The visitors of each browser, a color each (`fill: "var(--color-chrome)"`).
fn visitors(values: [f32; 5]) -> Series {
    Series::new("Visitors", values).cells((1..=5).map(Paint::chart))
}

fn running() -> Series {
    Series::new("Running", [450.0, 380.0, 520.0, 140.0, 600.0, 480.0])
}

fn swimming() -> Series {
    Series::new("Swimming", [300.0, 420.0, 120.0, 550.0, 350.0, 400.0])
}

/// `tickFormatter={(value) => value.slice(0, 3)}`
fn month(value: &str) -> String {
    value.chars().take(3).collect()
}

/// `toLocaleDateString("en-US", { weekday: "short" })`
fn weekday(value: &str) -> String {
    let index = DATES.iter().position(|date| *date == value).unwrap_or(0);
    ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat"][index].to_owned()
}

/// A cartesian chart over the months, like most examples:
/// `margin={{ left: 12, right: 12 }}`, abbreviated months, no cursor.
fn months(chart: Chart) -> Chart {
    chart
        .margin([0, 12])
        .tick_formatter(month)
        .cursor(false)
        .legend(false)
}

/// A bar chart over the months: `tickMargin={10}`.
fn bars() -> Chart {
    chart::bar(MONTHS)
        .tick_margin(10.0)
        .tick_formatter(month)
        .cursor(false)
        .legend(false)
}

/// The stacked bars of the tooltip examples, on `defaultIndex={1}`.
fn activities() -> Chart {
    chart::bar(DATES)
        .series(running())
        .series(swimming())
        .stacked(true)
        .grid(false)
        .tick_margin(10.0)
        .tick_formatter(weekday)
        .cursor(false)
        .legend(false)
        .default_index(1)
}

/// A polar chart in a 250 square.
fn square(chart: Chart) -> iced::Element<'static, Msg> {
    iced::widget::container(chart.legend(false).cursor(false))
        .width(250)
        .into()
}

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "bar" => Fixture::new(|_| {
            chart::bar(MONTHS)
                .series(desktop())
                .series(mobile())
                .tick_margin(10.0)
                .tick_formatter(month)
                .radius(4.0)
                .into()
        }),
        "line" => Fixture::new(|_| {
            chart::line(MONTHS)
                .series(desktop())
                .series(mobile())
                // `margin={{ left: 12, right: 12 }}`: Recharts does not merge it
                // with its default margin of 5.
                .margin([0, 12])
                .tick_formatter(month)
                .cursor(false)
                .dots(true)
                .active_dot(6.0)
                .into()
        }),
        "area-stacked" => Fixture::new(|_| -> iced::Element<'_, Msg> {
            chart::area(MONTHS)
                .series(desktop())
                .series(mobile())
                .margin([0, 12])
                .tick_formatter(month)
                .cursor(false)
                .stacked(true)
                .into()
        }),

        // Area charts.
        "area-axes" => Fixture::new(|_| {
            months(chart::area(MONTHS))
                .margin(iced::Padding {
                    top: 0.0,
                    right: 12.0,
                    bottom: 0.0,
                    left: -20.0,
                })
                .series(mobile())
                .series(desktop())
                .stacked(true)
                .value_axis(true)
                .value_ticks(3)
                .into()
        }),
        "area-default" => Fixture::new(|_| {
            months(chart::area(MONTHS))
                .series(desktop())
                .tooltip_indicator(Indicator::Line)
                .into()
        }),
        "area-gradient" => Fixture::new(|_| {
            months(chart::area(MONTHS))
                .series(mobile())
                .series(desktop())
                .stacked(true)
                .gradient(true)
                .into()
        }),
        "area-icons" => Fixture::new(|_| {
            months(chart::area(MONTHS))
                .series(mobile().color(Paint::chart(2)).icon(lucide::TRENDING_UP))
                .series(desktop().color(Paint::chart(1)).icon(lucide::TRENDING_DOWN))
                .stacked(true)
                .tooltip_indicator(Indicator::Line)
                .legend(true)
                .into()
        }),
        "area-legend" => Fixture::new(|_| {
            months(chart::area(MONTHS))
                .series(mobile().color(Paint::chart(2)))
                .series(desktop().color(Paint::chart(1)))
                .stacked(true)
                .tooltip_indicator(Indicator::Line)
                .legend(true)
                .into()
        }),
        "area-linear" => Fixture::new(|_| {
            months(chart::area(MONTHS))
                .series(desktop())
                .curve(Curve::Linear)
                .tooltip_label(false)
                .into()
        }),
        "area-stacked-expand" => Fixture::new(|_| {
            months(chart::area(MONTHS))
                .margin(iced::Padding {
                    top: 12.0,
                    right: 12.0,
                    bottom: 0.0,
                    left: 12.0,
                })
                .series(
                    Series::new("Other", [45.0, 100.0, 150.0, 50.0, 100.0, 160.0])
                        .color(Paint::chart(3))
                        .fill_opacity(0.1),
                )
                .series(mobile().color(Paint::chart(2)))
                .series(desktop().color(Paint::chart(1)))
                .expanded(true)
                .tooltip_indicator(Indicator::Line)
                .into()
        }),
        "area-step" => Fixture::new(|_| {
            months(chart::area(MONTHS))
                .series(desktop().icon(lucide::ACTIVITY))
                .curve(Curve::Step)
                .tooltip_label(false)
                .into()
        }),

        // Bar charts.
        "bar-active" => Fixture::new(|_| {
            chart::bar(BROWSERS)
                .series(visitors([187.0, 200.0, 275.0, 173.0, 90.0]))
                .radius(8.0)
                .highlight(2)
                .tick_margin(10.0)
                .cursor(false)
                .legend(false)
                .tooltip_label(false)
                .into()
        }),
        "bar-default" => Fixture::new(|_| {
            bars()
                .series(desktop())
                .radius(8.0)
                .tooltip_label(false)
                .into()
        }),
        "bar-horizontal" => Fixture::new(|_| {
            bars()
                .margin(iced::Padding {
                    left: -20.0,
                    ..iced::Padding::ZERO
                })
                .series(desktop())
                .horizontal(true)
                .grid(false)
                .radius(5.0)
                .tooltip_label(false)
                .into()
        }),
        "bar-label" => Fixture::new(|_| {
            bars()
                .margin(iced::Padding {
                    top: 20.0,
                    ..iced::Padding::ZERO
                })
                .series(desktop())
                .radius(8.0)
                .value_labels(true)
                .tooltip_label(false)
                .into()
        }),
        "bar-label-custom" => Fixture::new(|_| {
            bars()
                .margin(iced::Padding {
                    right: 16.0,
                    ..iced::Padding::ZERO
                })
                .series(desktop().color(Paint::chart(2)))
                .horizontal(true)
                .category_axis(false)
                .value_labels(true)
                .category_labels(true)
                .tick_formatter(str::to_owned)
                .tooltip_indicator(Indicator::Line)
                .into()
        }),
        "bar-mixed" => Fixture::new(|_| {
            chart::bar(BROWSERS)
                .margin(iced::Padding {
                    left: 0.0,
                    ..iced::Padding::ZERO
                })
                .series(visitors(VISITORS))
                .horizontal(true)
                .grid(false)
                .radius(5.0)
                .tick_margin(10.0)
                .cursor(false)
                .legend(false)
                .tooltip_label(false)
                .into()
        }),
        "bar-multiple" => Fixture::new(|_| {
            bars()
                .series(desktop())
                .series(mobile())
                .tooltip_indicator(Indicator::Dashed)
                .into()
        }),
        "bar-negative" => Fixture::new(|_| {
            let values = [186.0, 205.0, -207.0, 173.0, -209.0, 214.0];

            bars()
                .series(Series::new("Visitors", values).cells(values.map(|value| {
                    if value > 0.0 {
                        Paint::chart(1)
                    } else {
                        Paint::chart(2)
                    }
                })))
                .radius(0.0)
                .category_axis(false)
                .category_labels(true)
                .tick_formatter(str::to_owned)
                .tooltip_label(false)
                .tooltip_indicator(Indicator::None)
                .into()
        }),
        "bar-stacked" => Fixture::new(|_| {
            bars()
                .series(desktop())
                .series(mobile())
                .stacked(true)
                .cursor(true)
                .legend(true)
                .tooltip_label(false)
                .into()
        }),

        // Line charts.
        "line-default" => Fixture::new(|_| {
            months(chart::line(MONTHS))
                .series(desktop())
                .tooltip_label(false)
                .into()
        }),
        "line-dots" => Fixture::new(|_| {
            months(chart::line(MONTHS))
                .series(desktop())
                .dots(true)
                .active_dot(6.0)
                .tooltip_label(false)
                .into()
        }),
        "line-dots-colors" => Fixture::new(|_| {
            chart::line(BROWSERS)
                .margin(iced::Padding {
                    top: 24.0,
                    right: 24.0,
                    bottom: 0.0,
                    left: 24.0,
                })
                .series(visitors(VISITORS).color(Paint::chart(2)))
                .category_axis(false)
                .dots(true)
                .dot_radius(5.0)
                .cursor(false)
                .legend(false)
                .tooltip_label(false)
                .tooltip_indicator(Indicator::Line)
                .into()
        }),
        "line-label" => Fixture::new(|_| {
            months(chart::line(MONTHS))
                .margin(iced::Padding {
                    top: 20.0,
                    right: 12.0,
                    bottom: 0.0,
                    left: 12.0,
                })
                .series(desktop())
                .dots(true)
                .active_dot(6.0)
                .value_labels(true)
                .tooltip_indicator(Indicator::Line)
                .into()
        }),
        "line-label-custom" => Fixture::new(|_| {
            chart::line(BROWSERS)
                .margin(iced::Padding {
                    top: 24.0,
                    right: 24.0,
                    bottom: 0.0,
                    left: 24.0,
                })
                .series(Series::new("Visitors", VISITORS).color(Paint::chart(2)))
                .category_axis(false)
                .dots(true)
                .active_dot(6.0)
                .category_labels(true)
                .cursor(false)
                .legend(false)
                .tooltip_label(false)
                .tooltip_indicator(Indicator::Line)
                .into()
        }),
        "line-linear" => Fixture::new(|_| {
            months(chart::line(MONTHS))
                .series(desktop())
                .curve(Curve::Linear)
                .tooltip_label(false)
                .into()
        }),
        "line-multiple" => Fixture::new(|_| {
            months(chart::line(MONTHS))
                .series(desktop())
                .series(mobile())
                .curve(Curve::Monotone)
                .into()
        }),
        "line-step" => Fixture::new(|_| {
            months(chart::line(MONTHS))
                .series(desktop())
                .curve(Curve::Step)
                .tooltip_label(false)
                .into()
        }),

        // Pie charts.
        "pie-donut" => Fixture::new(|_| square(chart::pie(BROWSERS, VISITORS).donut(true))),
        "pie-donut-active" => {
            Fixture::new(|_| square(chart::pie(BROWSERS, VISITORS).donut(true).highlight(0)))
        }
        "pie-donut-text" => Fixture::new(|_| {
            square(
                chart::pie(BROWSERS, [275.0, 200.0, 287.0, 173.0, 190.0])
                    .donut(true)
                    .center_label("1,125", "Visitors"),
            )
        }),
        "pie-label" => Fixture::new(|_| square(chart::pie(BROWSERS, VISITORS).value_labels(true))),
        "pie-label-custom" => Fixture::new(|_| {
            square(
                chart::pie(BROWSERS, VISITORS)
                    .value_labels(true)
                    .label_lines(false),
            )
        }),
        "pie-label-list" => {
            Fixture::new(|_| square(chart::pie(BROWSERS, VISITORS).category_labels(true)))
        }
        "pie-legend" => Fixture::new(|_| {
            iced::widget::container(chart::pie(BROWSERS, VISITORS).tooltip(false))
                .width(250)
                .into()
        }),
        "pie-separator-none" => Fixture::new(|_| square(chart::pie(BROWSERS, VISITORS))),
        "pie-simple" => Fixture::new(|_| square(chart::pie(BROWSERS, VISITORS))),
        "pie-stacked" => Fixture::new(|_| {
            let months = ["January", "February", "March", "April", "May"];

            square(
                chart::rings(months)
                    .series(
                        Series::new("Desktop", [186.0, 305.0, 237.0, 173.0, 209.0])
                            .radii(0.0, 60.0),
                    )
                    .series(
                        Series::new("Mobile", [80.0, 200.0, 120.0, 190.0, 130.0]).radii(70.0, 90.0),
                    )
                    .tooltip_label(true)
                    .tooltip_indicator(Indicator::Line),
            )
        }),

        // Radar charts.
        "radar-default" => Fixture::new(|_| {
            square(chart::radar(MONTHS).series(Series::new(
                "Desktop",
                [186.0, 305.0, 237.0, 273.0, 209.0, 214.0],
            )))
        }),
        "radar-dots" => Fixture::new(|_| {
            square(
                chart::radar(MONTHS)
                    .series(Series::new(
                        "Desktop",
                        [186.0, 305.0, 237.0, 273.0, 209.0, 214.0],
                    ))
                    .dots(true),
            )
        }),
        "radar-grid-circle" => Fixture::new(|_| {
            square(
                chart::radar(MONTHS)
                    .series(Series::new(
                        "Desktop",
                        [186.0, 305.0, 237.0, 273.0, 209.0, 214.0],
                    ))
                    .grid_shape(GridShape::Circle)
                    .dots(true)
                    .tooltip_label(false),
            )
        }),
        "radar-grid-circle-fill" => Fixture::new(|_| {
            square(
                chart::radar(MONTHS)
                    .series(Series::new(
                        "Desktop",
                        [186.0, 285.0, 237.0, 203.0, 209.0, 264.0],
                    ))
                    .grid_shape(GridShape::Circle)
                    .grid_fill(true)
                    .fill_opacity(0.5),
            )
        }),
        "radar-grid-circle-no-lines" => Fixture::new(|_| {
            square(
                chart::radar(MONTHS)
                    .series(Series::new(
                        "Desktop",
                        [186.0, 305.0, 237.0, 203.0, 209.0, 214.0],
                    ))
                    .grid_shape(GridShape::Circle)
                    .radial_lines(false)
                    .dots(true)
                    .tooltip_label(false),
            )
        }),
        "radar-grid-custom" => Fixture::new(|_| {
            square(
                chart::radar(MONTHS)
                    .series(Series::new(
                        "Desktop",
                        [186.0, 305.0, 237.0, 273.0, 209.0, 214.0],
                    ))
                    .radial_lines(false)
                    .grid_radii([90.0])
                    .tooltip_label(false),
            )
        }),
        "radar-grid-fill" => Fixture::new(|_| {
            square(
                chart::radar(MONTHS)
                    .series(Series::new(
                        "Desktop",
                        [186.0, 285.0, 237.0, 203.0, 209.0, 264.0],
                    ))
                    .grid_fill(true)
                    .fill_opacity(0.5)
                    .tooltip_label(false),
            )
        }),
        "radar-grid-none" => Fixture::new(|_| {
            square(
                chart::radar(MONTHS)
                    .series(Series::new(
                        "Desktop",
                        [186.0, 305.0, 237.0, 273.0, 209.0, 214.0],
                    ))
                    .grid(false)
                    .dots(true)
                    .tooltip_label(false),
            )
        }),
        "radar-icons" => Fixture::new(|_| {
            iced::widget::container(
                chart::radar(MONTHS)
                    .series(desktop().icon(lucide::ARROW_DOWN_FROM_LINE))
                    .series(mobile().icon(lucide::ARROW_UP_FROM_LINE).fill_opacity(1.0))
                    .cursor(false)
                    .tooltip_indicator(Indicator::Line),
            )
            .width(250)
            .into()
        }),
        "radar-legend" => Fixture::new(|_| {
            iced::widget::container(
                chart::radar(MONTHS)
                    .series(desktop())
                    .series(mobile().fill_opacity(1.0))
                    .cursor(false)
                    .tooltip_indicator(Indicator::Line),
            )
            .width(250)
            .into()
        }),
        "radar-lines-only" => Fixture::new(|_| {
            square(
                chart::radar(MONTHS)
                    .series(
                        Series::new("Desktop", [186.0, 185.0, 207.0, 173.0, 160.0, 174.0])
                            .fill_opacity(0.0),
                    )
                    .series(
                        Series::new("Mobile", [160.0, 170.0, 180.0, 160.0, 190.0, 204.0])
                            .fill_opacity(0.0),
                    )
                    .stroke_width(2.0)
                    .radial_lines(false)
                    .tooltip_indicator(Indicator::Line),
            )
        }),
        "radar-multiple" => Fixture::new(|_| {
            square(
                chart::radar(MONTHS)
                    .series(desktop())
                    .series(mobile().fill_opacity(1.0))
                    .tooltip_indicator(Indicator::Line),
            )
        }),
        "radar-radius" => Fixture::new(|_| {
            square(
                chart::radar(MONTHS)
                    .series(desktop())
                    .series(mobile().fill_opacity(1.0))
                    .category_axis(false)
                    .radius_axis(60.0)
                    .tooltip_label(false)
                    .tooltip_indicator(Indicator::Line),
            )
        }),

        // Radial charts.
        "radial-grid" => Fixture::new(|_| {
            square(
                chart::radial(BROWSERS)
                    .series(Series::new("Visitors", VISITORS))
                    .inner_radius(30.0)
                    .outer_radius(100.0)
                    .grid(true),
            )
        }),
        "radial-label" => Fixture::new(|_| {
            square(
                chart::radial(BROWSERS)
                    .series(Series::new("Visitors", VISITORS))
                    .angles(-90.0, 380.0)
                    .inner_radius(30.0)
                    .outer_radius(110.0)
                    .background(true)
                    .category_labels(true),
            )
        }),
        "radial-shape" => Fixture::new(|_| {
            square(
                chart::radial(["Safari"])
                    .series(Series::new("Visitors", [1260.0]).cells([Paint::chart(2)]))
                    .angles(0.0, 100.0)
                    .inner_radius(65.0)
                    .outer_radius(95.0)
                    .track(74.0, 86.0)
                    .background(true)
                    .center_label("1,260", "Visitors")
                    .center_label_size(36.0)
                    .tooltip(false),
            )
        }),
        "radial-simple" => Fixture::new(|_| {
            square(
                chart::radial(BROWSERS)
                    .series(Series::new("Visitors", VISITORS))
                    .inner_radius(30.0)
                    .outer_radius(110.0)
                    .background(true),
            )
        }),
        "radial-stacked" => Fixture::new(|_| {
            square(
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
                    .center_label_offset(-24.0),
            )
        }),
        "radial-text" => Fixture::new(|_| {
            square(
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
                    .tooltip(false),
            )
        }),

        // Tooltips.
        "tooltip-advanced" => Fixture::new(|_| {
            activities()
                .tooltip_label(false)
                .tooltip_unit("kcal")
                .tooltip_total(true)
                .into()
        }),
        "tooltip-default" => Fixture::new(|_| activities().into()),
        "tooltip-formatter" => Fixture::new(|_| {
            activities()
                .tooltip_label(false)
                .tooltip_indicator(Indicator::None)
                .tooltip_unit("kcal")
                .into()
        }),
        "tooltip-icons" => Fixture::new(|_| {
            chart::bar(DATES)
                .series(running().icon(lucide::FOOTPRINTS))
                .series(swimming().icon(lucide::WAVES))
                .stacked(true)
                .grid(false)
                .tick_margin(10.0)
                .tick_formatter(weekday)
                .cursor(false)
                .legend(false)
                .default_index(1)
                .tooltip_label(false)
                .into()
        }),
        "tooltip-indicator-line" => {
            Fixture::new(|_| activities().tooltip_indicator(Indicator::Line).into())
        }
        "tooltip-indicator-none" => {
            Fixture::new(|_| activities().tooltip_indicator(Indicator::None).into())
        }
        "tooltip-label-custom" => Fixture::new(|_| {
            activities()
                .tooltip_label_formatter(|_| "Activities".to_owned())
                .tooltip_indicator(Indicator::Line)
                .into()
        }),
        "tooltip-label-formatter" => Fixture::new(|_| {
            activities()
                .tooltip_label_formatter(|_| "July 16, 2024".to_owned())
                .into()
        }),
        "tooltip-label-none" => Fixture::new(|_| {
            activities()
                .tooltip_label(false)
                .tooltip_indicator(Indicator::None)
                .into()
        }),
        _ => return None,
    })
}
