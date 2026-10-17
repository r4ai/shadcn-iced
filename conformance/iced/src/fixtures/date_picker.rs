use super::Fixture;
use crate::store::{Msg, Store, Value};

use shadcn_iced::calendar::{Date, Month};
use shadcn_iced::date_picker::date_picker;

use iced::Element;

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "default" | "navigate" => Fixture::new(view).init(|s| {
            // The calendar opens on June 2025 (`defaultMonth`).
            s.set("month", (2025 * 12 + 5) as f32);
        }),
        _ => return None,
    })
}

/// `w-[212px]`, no date selected at first.
fn view(s: &Store) -> Element<'_, Msg> {
    let months = s.num("month") as i32;
    let month = Month {
        year: months.div_euclid(12),
        month: months.rem_euclid(12) as u32 + 1,
    };
    let days = s.num("date");
    let date = (days != 0.0).then(|| Date::from_days(days as i64));

    date_picker(
        date,
        |date| Msg::Set("date", Value::Num(date.to_days() as f32)),
        month,
        |month| {
            Msg::Set(
                "month",
                Value::Num((month.year * 12 + month.month as i32 - 1) as f32),
            )
        },
    )
    .into()
}
