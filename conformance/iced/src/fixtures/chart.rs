use super::Fixture;
use crate::store::Msg;

use shadcn_iced::chart::{self, Series};

const MONTHS: [&str; 6] = ["January", "February", "March", "April", "May", "June"];

fn desktop() -> Series {
    Series::new("Desktop", [186.0, 305.0, 237.0, 73.0, 209.0, 214.0])
}

fn mobile() -> Series {
    Series::new("Mobile", [80.0, 200.0, 120.0, 190.0, 130.0, 140.0])
}

/// `tickFormatter={(value) => value.slice(0, 3)}`
fn month(value: &str) -> String {
    value.chars().take(3).collect()
}

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "bar" => Fixture::new(|_| {
            chart::bar(MONTHS)
                .series(desktop())
                .series(mobile())
                .tick_margin(10.0)
                .tick_formatter(month)
                .height(250.0)
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
                .height(250.0)
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
                .height(250.0)
                .into()
        }),
        _ => return None,
    })
}
