use super::Fixture;
use crate::store::{Msg, Store, Value};

use shadcn_iced::calendar::{Date, Month, calendar};

/// The fixed "today" of the browser clock.
fn today() -> Date {
    Date::new(2025, 6, 15).expect("a date")
}

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "default" => Fixture::new(|s| {
            calendar(month(s), navigate)
                .selected(selected(s, "date"))
                .on_select(pick("date"))
                .today(Some(today()))
                .bordered(true)
                .into()
        })
        .init(|s| {
            init(s);
            s.set("date", day(12));
        }),
        "range" => Fixture::new(|s| {
            calendar(month(s), navigate)
                .range(selected(s, "start"), selected(s, "end"))
                .on_select(|date| {
                    // Extends the range past its end.
                    Msg::Set("end", Value::Num(date.to_days() as f32))
                })
                .months(2)
                .today(Some(today()))
                .bordered(true)
                .into()
        })
        .init(|s| {
            init(s);
            s.set("start", day(12));
            s.set("end", day(17));
        }),
        "dropdowns" => Fixture::new(|s| {
            calendar(month(s), navigate)
                .dropdowns(2016, 2036)
                .on_select(pick("date"))
                .today(Some(today()))
                .bordered(true)
                .into()
        })
        .init(init),
        "week-numbers" => Fixture::new(|s| {
            calendar(month(s), navigate)
                .selected(Date::new(2025, 6, 12))
                .on_select(pick("date"))
                .week_numbers(true)
                .today(Some(today()))
                .bordered(true)
                .into()
        })
        .init(init),
        "booked" => Fixture::new(|s| {
            // The 17th to the 30th of June.
            let booked = |date: Date| date.year == 2025 && date.month == 6 && date.day >= 17;

            calendar(month(s), navigate)
                .selected(selected(s, "date"))
                .on_select(pick("date"))
                .disabled(booked)
                .struck(booked)
                .today(Some(today()))
                .bordered(true)
                .into()
        })
        .init(|s| {
            init(s);
            s.set("date", day(12));
        }),
        _ => return None,
    })
}

/// Starts in June 2025.
fn init(s: &mut Store) {
    s.set("month", (2025 * 12 + 5) as f32);
}

fn day(day: u32) -> f32 {
    Date::new(2025, 6, day).expect("a date").to_days() as f32
}

fn month(s: &Store) -> Month {
    let months = s.num("month") as i32;

    Month {
        year: months.div_euclid(12),
        month: months.rem_euclid(12) as u32 + 1,
    }
}

fn navigate(month: Month) -> Msg {
    Msg::Set(
        "month",
        Value::Num((month.year * 12 + month.month as i32 - 1) as f32),
    )
}

/// The date stored under `key`, if any.
fn selected(s: &Store, key: &str) -> Option<Date> {
    let days = s.num(key);

    (days != 0.0).then(|| Date::from_days(days as i64))
}

fn pick(key: &'static str) -> impl Fn(Date) -> Msg {
    move |date| Msg::Set(key, Value::Num(date.to_days() as f32))
}
