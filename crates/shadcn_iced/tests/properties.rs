use proptest::prelude::*;

use shadcn_iced::calendar::{Date, Month, days_in_month};
use shadcn_iced::date_picker::{long_date, numeric_date, short_date};
use shadcn_iced::pagination::pages;

fn arb_date() -> impl Strategy<Value = Date> {
    (-10_000i32..=10_000, 1u32..=12).prop_flat_map(|(year, month)| {
        let max_day = days_in_month(year, month);
        (Just(year), Just(month), 1u32..=max_day)
            .prop_map(|(year, month, day)| Date::new(year, month, day).unwrap())
    })
}

proptest! {
    // -------------------------------------------------------------------------
    // calendar
    // -------------------------------------------------------------------------

    #[test]
    fn calendar_days_bijection_and_weekday(date in arb_date(), delta in -50_000i64..=50_000) {
        let days = date.to_days();
        prop_assert_eq!(Date::from_days(days), date);

        let shifted = date.add_days(delta);
        prop_assert_eq!(shifted.add_days(-delta), date);
        prop_assert_eq!(shifted.to_days(), days + delta);

        let next = date.add_days(1);
        prop_assert_eq!(next.weekday(), (date.weekday() + 1) % 7);
        prop_assert!(date.weekday() < 7);
    }

    #[test]
    fn calendar_month_navigation_round_trip(year in -10_000i32..=10_000, month in 1u32..=12) {
        let m = Month { year, month };
        prop_assert_eq!(m.next().previous(), m);
        prop_assert_eq!(m.previous().next(), m);
        prop_assert_eq!(m.first().month_of(), m);
        prop_assert_eq!(m.first().day, 1);
    }

    #[test]
    fn calendar_days_in_month_leap_rules(year in -10_000i32..=10_000, month in 0u32..=14) {
        let days = days_in_month(year, month);
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => prop_assert_eq!(days, 31),
            4 | 6 | 9 | 11 => prop_assert_eq!(days, 30),
            2 => {
                let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
                prop_assert_eq!(days, if leap { 29 } else { 28 });
            }
            _ => prop_assert_eq!(days, 0),
        }

        // Date::new accepts 1..=days and rejects 0 and days + 1.
        prop_assert!(Date::new(year, month, 0).is_none());
        if days > 0 {
            prop_assert!(Date::new(year, month, 1).is_some());
            prop_assert!(Date::new(year, month, days).is_some());
            prop_assert!(Date::new(year, month, days + 1).is_none());
        }
    }

    // -------------------------------------------------------------------------
    // date_picker
    // -------------------------------------------------------------------------

    #[test]
    fn date_picker_formats_valid_dates(date in arb_date()) {
        let long = long_date(date);
        let expected_suffix = match (date.day % 10, date.day % 100) {
            (_, 11..=13) => "th",
            (1, _) => "st",
            (2, _) => "nd",
            (3, _) => "rd",
            _ => "th",
        };
        prop_assert!(
            long.contains(&format!("{}{expected_suffix}, {}", date.day, date.year)),
            "unexpected long_date: {long}"
        );

        let short = short_date(date);
        prop_assert!(
            short.ends_with(&format!(" {:02}, {}", date.day, date.year)),
            "unexpected short_date: {short}"
        );

        prop_assert_eq!(
            numeric_date(date),
            format!("{}/{}/{}", date.month, date.day, date.year)
        );
    }

    // -------------------------------------------------------------------------
    // pagination
    // -------------------------------------------------------------------------

    #[test]
    fn pagination_pages_invariants(
        current in prop_oneof![0usize..=50, Just(usize::MAX)],
        total in 0usize..=50,
        siblings in prop_oneof![0usize..=10, Just(usize::MAX)],
    ) {
        let result = pages(current, total, siblings);

        if total == 0 {
            prop_assert!(result.is_empty());
            return Ok(());
        }

        let clamped_current = current.clamp(1, total);

        // Starts with 1, ends with total, and includes clamped_current.
        prop_assert_eq!(result.first().copied(), Some(Some(1)));
        prop_assert_eq!(result.last().copied(), Some(Some(total)));
        prop_assert!(result.contains(&Some(clamped_current)));

        // No consecutive ellipses and strictly increasing page numbers.
        let mut prev_page = 0usize;
        for w in result.windows(2) {
            prop_assert!(!(w[0].is_none() && w[1].is_none()), "consecutive ellipses: {result:?}");
        }
        for item in &result {
            if let Some(p) = *item {
                prop_assert!(p > prev_page, "not strictly increasing: {result:?}");
                prop_assert!(p <= total);
                prev_page = p;
            }
        }

        // Every ellipsis bridges a gap of at least 1 hidden page.
        for (i, item) in result.iter().enumerate() {
            if item.is_none() {
                let before = result[i - 1].unwrap();
                let after = result[i + 1].unwrap();
                prop_assert!(
                    after - before >= 2,
                    "redundant ellipsis between {before} and {after}: {result:?}"
                );
            }
        }
    }
}

#[test]
fn calendar_extreme_year_to_days_does_not_overflow() {
    let d = Date::new(i32::MIN, 1, 1).unwrap();
    let _ = d.to_days();
}
