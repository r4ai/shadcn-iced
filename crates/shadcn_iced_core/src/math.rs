//! Arithmetic behind the components, total over its inputs.
//!
//! Every function here is defined for every input: none panics, overflows
//! or gives a value out of its documented range. They depend on nothing
//! but `core`, which lets [Kani](https://model-checking.github.io/kani/)
//! prove those properties for every input (`crates/verify`), on top of the
//! property tests.
//!
//! - [`clamp`] takes the bounds `f32::clamp` panics on: out of order or
//!   `NaN`, as `limits(max, min)` or a value computed by an application
//!   easily gives.
//! - [`days_from_civil`] and [`civil_from_days`] convert the dates of the
//!   proleptic Gregorian calendar, of every `i32` year, to days since
//!   1970-01-01 and back.
//! - [`pages`] lists the pages of a pagination.

/// Clamps `value` between `a` and `b`, which are swapped when out of order.
/// A `NaN` bound is no bound, and a `NaN` value gives the lower bound (or
/// the upper one if the lower one is `NaN` too).
pub fn clamp(value: f32, a: f32, b: f32) -> f32 {
    let (low, high) = if a > b { (b, a) } else { (a, b) };

    if value.is_nan() {
        return if low.is_nan() { high } else { low };
    }

    // `f32::max` and `f32::min` ignore a `NaN` operand.
    value.max(low).min(high)
}

/// Whether `year` is a leap year of the proleptic Gregorian calendar.
pub const fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// The number of days of `month` (from 1 to 12) in `year`, or 0 for a month
/// out of that range.
pub const fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

/// The number of days from 1970-01-01 to a date (Howard Hinnant's
/// `days_from_civil`), negative before it.
///
/// For a month out of `1..=12` or a day out of the month, it gives the
/// value of the formula, without panicking: the day of an invalid date is
/// unspecified but finite.
pub const fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    // The years start in March, so that February ends them.
    let y = year as i64 - if month <= 2 { 1 } else { 0 };
    let era = y.div_euclid(400);
    let yoe = (y - era * 400) as u32;

    era * ERA + start_of_year(yoe) as i64 + day_of_year(month, day) - EPOCH
}

/// The first day that [`civil_from_days`] gives: January 1st of `i32::MIN`.
pub const MIN_DAYS: i64 = days_from_civil(i32::MIN, 1, 1);

/// The last day that [`civil_from_days`] gives: December 31st of
/// `i32::MAX`.
pub const MAX_DAYS: i64 = days_from_civil(i32::MAX, 12, 31);

/// The date (`year`, `month`, `day`) `days` days after 1970-01-01 (Howard
/// Hinnant's `civil_from_days`). The days out of the years of an `i32`
/// give the first or the last of its dates: the date is always valid.
pub fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let z = days.clamp(MIN_DAYS, MAX_DAYS) + EPOCH;
    let era = z.div_euclid(ERA);
    let (yoe, doy) = year_of_era((z - era * ERA) as u32);
    let (month, day) = month_of_year(doy);
    let year = (era * 400 + yoe as i64 + if month <= 2 { 1 } else { 0 }) as i32;

    (year, month, day)
}

/// The days of an era: 400 years of the Gregorian calendar, which repeats
/// after them.
const ERA: i64 = 146_097;

/// The day of 1970-01-01 from 0000-03-01, the first day of an era.
const EPOCH: i64 = 719_468;

/// The day of an era (from 0 to 146 096) of the first day of its year
/// `yoe` (from 0 to 399), the years starting in March.
const fn start_of_year(yoe: u32) -> u32 {
    yoe * 365 + yoe / 4 - yoe / 100
}

/// The year of the era (from 0 to 399) of the day of an era `doe` (from 0
/// to 146 096), and the day of that year (from 0 to 365).
const fn year_of_era(doe: u32) -> (u32, u32) {
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;

    (yoe, doe - start_of_year(yoe))
}

/// The day of the year (from 0, the years starting in March) of a day of a
/// month, from the formula for any `month` and `day`.
const fn day_of_year(month: u32, day: u32) -> i64 {
    let month = month as i64;

    (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day as i64 - 1
}

/// The month (from 1 to 12) and the day of the month of the day of the
/// year `doy` (from 0 to 365, the years starting in March).
const fn month_of_year(doy: u32) -> (u32, u32) {
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };

    (month, day)
}

/// The day of the week of the day `days` days after 1970-01-01 (a
/// Thursday), from 0 (Sunday) to 6 (Saturday).
pub const fn weekday(days: i64) -> u32 {
    // `days + 4`, without overflowing.
    ((days % 7 + 7 + 4) % 7) as u32
}

/// The month (`year`, `month`) `months` months after `month` of `year`
/// (possibly negative), from 1 to 12. A month out of `1..=12` counts from
/// January: 0 is the December before, 13 the January after. Past the years
/// of an `i32`, it stays on the first or the last month.
pub fn add_months(year: i32, month: u32, months: i64) -> (i32, u32) {
    const FIRST: i64 = i32::MIN as i64 * 12;
    const LAST: i64 = i32::MAX as i64 * 12 + 11;

    let index = (i64::from(year) * 12 + i64::from(month) - 1)
        .saturating_add(months)
        .clamp(FIRST, LAST);

    (index.div_euclid(12) as i32, index.rem_euclid(12) as u32 + 1)
}

/// The pages of a pagination to show, with `None` standing for an
/// ellipsis: the first and the last pages, and the `siblings` pages on each
/// side of `current` (clamped to the pages), an ellipsis standing for the
/// pages between them, or the page itself when it is the only one.
///
/// The pages count from 1 to `total`, and none is shown for no page.
pub fn pages(current: usize, total: usize, siblings: usize) -> Vec<Option<usize>> {
    let Some((start, end)) = window(current, total, siblings) else {
        return Vec::new();
    };

    let mut pages = Vec::new();

    if start > 1 {
        pages.push(Some(1));
        pages.extend(gap(1, start));
    }

    pages.extend((start..=end).map(Some));

    if end < total {
        pages.extend(gap(end, total));
        pages.push(Some(total));
    }

    pages
}

/// The first and last pages around `current` (clamped to the pages) that
/// [`pages`] shows, `siblings` on each side, or `None` without pages.
pub fn window(current: usize, total: usize, siblings: usize) -> Option<(usize, usize)> {
    if total == 0 {
        return None;
    }

    let current = current.clamp(1, total);
    let start = current.saturating_sub(siblings).max(1);
    let end = current.saturating_add(siblings).min(total);

    Some((start, end))
}

/// What stands between the pages `from` and `to` (`from < to`): nothing
/// when they follow each other, the page between them when there is one,
/// else an ellipsis (`Some(None)`).
pub const fn gap(from: usize, to: usize) -> Option<Option<usize>> {
    match to.saturating_sub(from) {
        0 | 1 => None,
        2 => Some(Some(from + 1)),
        _ => Some(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn clamp_known_values() {
        assert_eq!(clamp(0.5, 0.0, 1.0), 0.5);
        assert_eq!(clamp(-1.0, 0.0, 1.0), 0.0);
        assert_eq!(clamp(2.0, 0.0, 1.0), 1.0);
        assert_eq!(clamp(2.0, 1.0, 0.0), 1.0);
        assert_eq!(clamp(-2.0, 1.0, 0.0), 0.0);
        assert_eq!(clamp(f32::NAN, 0.2, 0.8), 0.2);
        assert_eq!(clamp(f32::NAN, f32::NAN, 0.8), 0.8);
        assert_eq!(clamp(0.9, f32::NAN, 0.8), 0.8);
        assert_eq!(clamp(0.7, f32::NAN, 0.8), 0.7);
        assert_eq!(clamp(-5.0, 0.1, f32::NAN), 0.1);
        assert_eq!(clamp(5.0, 0.1, f32::NAN), 5.0);
        assert!(clamp(f32::NAN, f32::NAN, f32::NAN).is_nan());
    }

    #[test]
    fn civil_known_values() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(days_from_civil(2000, 3, 1), 11_017);
        assert_eq!(days_from_civil(1969, 12, 31), -1);
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(-1), (1969, 12, 31));
        assert_eq!(civil_from_days(20_742), (2026, 10, 16));
        // The ends of the eras and of their centuries.
        for (year, month, day) in [
            (2000, 2, 29),
            (2000, 3, 1),
            (1900, 2, 28),
            (1900, 3, 1),
            (2100, 2, 28),
            (2400, 2, 29),
            (1600, 2, 29),
            (-400, 2, 29),
            (-1, 12, 31),
            (0, 1, 1),
        ] {
            let days = days_from_civil(year, month, day);
            assert_eq!(civil_from_days(days), (year, month, day));
            assert_eq!(
                civil_from_days(days + 1).2 == 1,
                day == days_in_month(year, month)
            );
        }
        assert_eq!(
            days_from_civil(2000, 2, 29) + 1,
            days_from_civil(2000, 3, 1)
        );
        assert_eq!(
            days_from_civil(2001, 1, 1) - days_from_civil(2000, 1, 1),
            366
        );
        assert_eq!(
            days_from_civil(2101, 1, 1) - days_from_civil(2100, 1, 1),
            365
        );
        assert_eq!(civil_from_days(i64::MIN), (i32::MIN, 1, 1));
        assert_eq!(civil_from_days(i64::MAX), (i32::MAX, 12, 31));
        assert_eq!(weekday(0), 4);
        assert_eq!(weekday(-1), 3);
        assert_eq!(weekday(20_742), 5);
        assert!(weekday(i64::MIN) < 7 && weekday(i64::MAX) < 7);
        assert!(is_leap_year(2000) && is_leap_year(2024) && !is_leap_year(1900));
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2023, 2), 28);
        assert_eq!(days_in_month(2023, 0), 0);
        assert_eq!(days_in_month(2023, 13), 0);
    }

    #[test]
    fn add_months_known_values() {
        assert_eq!(add_months(2026, 12, 1), (2027, 1));
        assert_eq!(add_months(2026, 1, -1), (2025, 12));
        assert_eq!(add_months(2026, 5, -17), (2024, 12));
        assert_eq!(add_months(2026, 0, 0), (2025, 12));
        assert_eq!(add_months(2026, 13, 0), (2027, 1));
        assert_eq!(add_months(i32::MAX, 12, 1), (i32::MAX, 12));
        assert_eq!(add_months(i32::MIN, 1, -1), (i32::MIN, 1));
        assert_eq!(add_months(0, u32::MAX, i64::MAX), (i32::MAX, 12));
        assert_eq!(add_months(0, 1, i64::MIN), (i32::MIN, 1));
    }

    #[test]
    fn pages_known_values() {
        let some = |pages: &[usize]| pages.iter().copied().map(Some).collect::<Vec<_>>();

        assert_eq!(pages(1, 0, 1), vec![]);
        assert_eq!(pages(1, 1, 1), some(&[1]));
        assert_eq!(
            pages(5, 10, 1),
            vec![Some(1), None, Some(4), Some(5), Some(6), None, Some(10)]
        );
        // A single hidden page shows instead of an ellipsis.
        assert_eq!(pages(4, 7, 1), some(&[1, 2, 3, 4, 5, 6, 7]));
        assert_eq!(pages(1, 4, 1), some(&[1, 2, 3, 4]));
        // No ellipsis between pages that follow each other.
        assert_eq!(
            pages(3, 10, 1),
            vec![Some(1), Some(2), Some(3), Some(4), None, Some(10)]
        );
        assert_eq!(gap(1, 2), None);
        assert_eq!(gap(1, 3), Some(Some(2)));
        assert_eq!(gap(1, 4), Some(None));
        assert_eq!(pages(usize::MAX, 3, usize::MAX), some(&[1, 2, 3]));
    }

    fn any_f32() -> impl Strategy<Value = f32> {
        prop_oneof![
            Just(f32::NAN),
            Just(f32::INFINITY),
            Just(f32::NEG_INFINITY),
            Just(0.0),
            Just(-0.0),
            any::<f32>(),
            -10.0f32..10.0,
        ]
    }

    proptest! {
        #[test]
        fn clamp_stays_within_the_bounds(value in any_f32(), a in any_f32(), b in any_f32()) {
            let out = clamp(value, a, b);

            if !a.is_nan() || !b.is_nan() {
                prop_assert!(!out.is_nan());
            }
            if !a.is_nan() && !b.is_nan() {
                prop_assert!(a.min(b) <= out && out <= a.max(b));
                prop_assert_eq!(out, clamp(value, b, a));
                // It agrees with `f32::clamp` where that one does not panic.
                if !value.is_nan() {
                    prop_assert_eq!(out, value.clamp(a.min(b), a.max(b)));
                }
            }
        }

        #[test]
        fn civil_round_trips(days in prop_oneof![
            any::<i64>(),
            MIN_DAYS - 10..=MIN_DAYS + 1_000_000,
            MAX_DAYS - 1_000_000..=MAX_DAYS + 10,
            MIN_DAYS..=MAX_DAYS,
            -1_000_000i64..1_000_000,
        ]) {
            let (year, month, day) = civil_from_days(days);

            prop_assert!((1..=12).contains(&month));
            prop_assert!(day >= 1 && day <= days_in_month(year, month));
            prop_assert_eq!(days_from_civil(year, month, day), days.clamp(MIN_DAYS, MAX_DAYS));
        }

        #[test]
        fn civil_days_are_consecutive(year in any::<i32>(), month in 1u32..=12) {
            let first = days_from_civil(year, month, 1);
            let last = days_from_civil(year, month, days_in_month(year, month));
            let (next_year, next_month) = add_months(year, month, 1);

            prop_assert_eq!(last - first + 1, i64::from(days_in_month(year, month)));
            if (next_year, next_month) != (year, month) {
                prop_assert_eq!(days_from_civil(next_year, next_month, 1), last + 1);
            }
            prop_assert_eq!(weekday(last + 1), (weekday(last) + 1) % 7);
        }

        #[test]
        fn add_months_composes(
            year in -1_000_000i32..1_000_000,
            month in 1u32..=12,
            a in -100_000i64..100_000,
            b in -100_000i64..100_000,
        ) {
            let (y, m) = add_months(year, month, a);

            prop_assert!((1..=12).contains(&m));
            prop_assert_eq!(add_months(y, m, b), add_months(year, month, a + b));
            prop_assert_eq!(add_months(y, m, -a), (year, month));
        }

        #[test]
        fn pages_are_increasing_and_complete(
            current in prop_oneof![0usize..=60, Just(usize::MAX)],
            total in 0usize..=60,
            siblings in prop_oneof![0usize..=10, Just(usize::MAX)],
        ) {
            let shown = pages(current, total, siblings);

            if total == 0 {
                prop_assert!(shown.is_empty());
                return Ok(());
            }

            let current = current.clamp(1, total);
            let numbers: Vec<usize> = shown.iter().flatten().copied().collect();

            prop_assert_eq!(numbers.first(), Some(&1));
            prop_assert_eq!(numbers.last(), Some(&total));
            prop_assert!(numbers.windows(2).all(|pair| pair[0] < pair[1]));

            // Every sibling of the current page is shown.
            for page in current.saturating_sub(siblings).max(1)..=current.saturating_add(siblings).min(total) {
                prop_assert!(numbers.contains(&page));
            }

            // An ellipsis stands for two pages or more, and only for one.
            for (index, page) in shown.iter().enumerate() {
                match page {
                    None => {
                        let before = shown[index - 1].unwrap();
                        let after = shown[index + 1].unwrap();
                        prop_assert!(after - before > 2, "{shown:?}");
                    }
                    Some(page) => {
                        if let Some(Some(next)) = shown.get(index + 1) {
                            prop_assert_eq!(*next, page + 1, "{:?}", shown);
                        }
                    }
                }
            }

            // At most the siblings, the first and last pages, and an
            // ellipsis on each side.
            prop_assert!(shown.len() <= siblings.saturating_mul(2).saturating_add(5));
        }
    }
}

/// The proofs of Kani, for every input (`cargo kani` in `crates/verify`).
#[cfg(kani)]
mod verification {
    use super::*;

    #[kani::proof]
    fn clamp_is_total_and_bounded() {
        let value: f32 = kani::any();
        let a: f32 = kani::any();
        let b: f32 = kani::any();

        let out = clamp(value, a, b);

        if !a.is_nan() || !b.is_nan() {
            assert!(!out.is_nan());
        }
        if !a.is_nan() && !b.is_nan() {
            assert!(a.min(b) <= out && out <= a.max(b));
            assert!(out == clamp(value, b, a));
            if !value.is_nan() && a.min(b) <= value && value <= a.max(b) {
                assert!(out == value);
            }
        }
    }

    #[kani::proof]
    fn days_in_month_is_a_month_length() {
        let year: i32 = kani::any();
        let month: u32 = kani::any();
        let days = days_in_month(year, month);

        if (1..=12).contains(&month) {
            assert!((28..=31).contains(&days));
            assert!((days == 29) == (month == 2 && is_leap_year(year)));
        } else {
            assert!(days == 0);
        }
    }

    /// No overflow for any input, valid or not.
    #[kani::proof]
    fn days_from_civil_never_overflows() {
        let _ = days_from_civil(kani::any(), kani::any(), kani::any());
    }

    // The conversions of the dates, in two parts: the days of an era
    // (`civil_from_days` divides the days into eras of 400 years, with
    // `div_euclid`), and the days within an era, proven to be inverse for
    // every day of an era, year of an era, day of a year and day of a
    // month. The leap years repeat with the eras.

    /// The number of days of the year `yoe` of an era (from March).
    fn year_length(yoe: u32) -> u32 {
        // February ends the year: the one of the next calendar year.
        if is_leap_year(yoe as i32 + 1) {
            366
        } else {
            365
        }
    }

    #[kani::proof]
    fn year_of_era_inverts_day_of_era() {
        let doe: u32 = kani::any_where(|doe| (*doe as i64) < ERA);
        let (yoe, doy) = year_of_era(doe);

        assert!(yoe < 400);
        assert!(doy < year_length(yoe));
        assert!(start_of_year(yoe) + doy == doe);
    }

    #[kani::proof]
    fn day_of_era_inverts_year_of_era() {
        let yoe: u32 = kani::any_where(|yoe| *yoe < 400);
        let doy: u32 = kani::any_where(|doy| *doy < year_length(yoe));
        let doe = start_of_year(yoe) + doy;

        assert!((doe as i64) < ERA);
        assert!(year_of_era(doe) == (yoe, doy));
    }

    /// The days of the months, in a leap year or not.
    fn month_length(month: u32, leap: bool) -> u32 {
        days_in_month(if leap { 2000 } else { 2001 }, month)
    }

    #[kani::proof]
    fn month_of_year_inverts_day_of_year() {
        let leap: bool = kani::any();
        let doy: u32 = kani::any_where(|doy| *doy < if leap { 366 } else { 365 });
        let (month, day) = month_of_year(doy);

        assert!((1..=12).contains(&month));
        assert!(day >= 1 && day <= month_length(month, leap));
        assert!(day_of_year(month, day) == doy as i64);
    }

    #[kani::proof]
    fn day_of_year_inverts_month_of_year() {
        let leap: bool = kani::any();
        let month: u32 = kani::any_where(|month| (1..=12).contains(month));
        let day: u32 = kani::any_where(|day| *day >= 1 && *day <= month_length(month, leap));
        let doy = day_of_year(month, day);

        assert!(0 <= doy && doy < if leap { 366 } else { 365 });
        assert!(month_of_year(doy as u32) == (month, day));
    }

    #[kani::proof]
    fn leap_years_repeat_every_era() {
        let year: i32 = kani::any_where(|year| *year <= i32::MAX - 400);

        assert!(is_leap_year(year + 400) == is_leap_year(year));
        assert!(is_leap_year(year) == is_leap_year(year.rem_euclid(400)));
    }

    /// The bounds of `civil_from_days` are the first and last days of the
    /// `i32` years.
    #[kani::proof]
    fn civil_bounds_are_the_ends_of_the_years() {
        assert!(civil_from_days(MIN_DAYS) == (i32::MIN, 1, 1));
        assert!(civil_from_days(MAX_DAYS) == (i32::MAX, 12, 31));
        assert!(civil_from_days(MIN_DAYS - 1) == (i32::MIN, 1, 1));
        assert!(civil_from_days(MAX_DAYS + 1) == (i32::MAX, 12, 31));
        assert!(civil_from_days(0) == (1970, 1, 1));
    }

    #[kani::proof]
    fn weekday_is_a_day_of_the_week() {
        let days: i64 = kani::any();
        let day = weekday(days);

        assert!(day < 7);
        if days < i64::MAX {
            assert!(weekday(days + 1) == (day + 1) % 7);
        }
    }

    #[kani::proof]
    fn add_months_is_total() {
        let year: i32 = kani::any();
        let month: u32 = kani::any();
        let months: i64 = kani::any();

        let (y, m) = add_months(year, month, months);
        assert!((1..=12).contains(&m));

        // One month forward then back is the identity, away from the ends.
        if (1..=12).contains(&month) && (year, month) != (i32::MAX, 12) {
            let (y, m) = add_months(year, month, 1);
            assert!(add_months(y, m, -1) == (year, month));
        }
        let _ = y;
    }

    /// The window of the pages around the current one, for every
    /// pagination.
    #[kani::proof]
    fn window_holds_the_current_page_and_its_siblings() {
        let current: usize = kani::any();
        let total: usize = kani::any();
        let siblings: usize = kani::any();

        let Some((start, end)) = window(current, total, siblings) else {
            assert!(total == 0);
            return;
        };

        let current = current.clamp(1, total);
        assert!(1 <= start && start <= current && current <= end && end <= total);
        // Every sibling on each side, as far as the pages go.
        assert!(start == 1 || current - start == siblings);
        assert!(end == total || end - current == siblings);
    }

    /// An ellipsis stands for two pages or more, and a single page shows.
    #[kani::proof]
    fn gap_stands_for_the_pages_between() {
        let from: usize = kani::any();
        let to: usize = kani::any_where(|to| *to > from);

        match gap(from, to) {
            None => assert!(to == from + 1),
            Some(Some(page)) => assert!(page == from + 1 && to == from + 2),
            Some(None) => assert!(to - from > 2),
        }
    }
}
