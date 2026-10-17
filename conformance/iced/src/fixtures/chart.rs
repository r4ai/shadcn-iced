use super::Fixture;
use crate::store::Msg;

use shadcn_iced::chart::{self, Series};

const MONTHS: [&str; 6] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun"];

fn desktop() -> Series {
    Series::new("Desktop", [186.0, 305.0, 237.0, 73.0, 209.0, 214.0])
}

fn mobile() -> Series {
    Series::new("Mobile", [80.0, 200.0, 120.0, 190.0, 130.0, 140.0])
}

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "bar" => Fixture::new(|_| {
            chart::bar(MONTHS)
                .series(desktop())
                .series(mobile())
                .height(250.0)
                .into()
        }),
        "line" => Fixture::new(|_| {
            chart::line(MONTHS)
                .series(desktop())
                .series(mobile())
                .dots(true)
                .height(250.0)
                .into()
        }),
        "area-stacked" => Fixture::new(|_| -> iced::Element<'_, Msg> {
            chart::area(MONTHS)
                .series(desktop())
                .series(mobile())
                .stacked(true)
                .height(250.0)
                .into()
        }),
        _ => return None,
    })
}
