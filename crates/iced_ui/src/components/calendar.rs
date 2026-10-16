//! A date field component that allows users to enter and edit date.
//!
//! Port of <https://ui.shadcn.com/docs/components/calendar> (react-day-picker).
//! Both the displayed month and the selection live in your state:
//!
//! ```no_run
//! # use iced_ui::calendar::{calendar, Date, Month};
//! # #[derive(Clone)] enum Message { Navigate(Month), Select(Date) }
//! # fn view<'a>(month: Month, selected: Option<Date>) -> iced::Element<'a, Message> {
//! calendar(month, Message::Navigate)
//!     .selected(selected)
//!     .on_select(Message::Select)
//!     .into()
//! # }
//! ```
use iced_ui_core::icon::lucide;
use iced_ui_core::primitive::Pressable;
use iced_ui_core::theme::alpha;
use iced_ui_core::{Element, font, style, tokens};

use iced::widget::button::{Status, Style};
use iced::widget::{Column, Row, container, text};
use iced::{Background, Length, Theme, border};

use std::fmt;

/// A calendar date (proleptic Gregorian).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    /// The year.
    pub year: i32,
    /// The month, from 1 to 12.
    pub month: u32,
    /// The day of the month, from 1.
    pub day: u32,
}

impl Date {
    /// Creates a date. Returns `None` if it does not exist.
    pub fn new(year: i32, month: u32, day: u32) -> Option<Self> {
        ((1..=12).contains(&month) && day >= 1 && day <= days_in_month(year, month))
            .then_some(Self { year, month, day })
    }

    /// Today, in UTC.
    pub fn today() -> Self {
        let seconds = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_secs() as i64)
            .unwrap_or_default();

        Self::from_days(seconds.div_euclid(86_400))
    }

    /// The number of days since 1970-01-01.
    pub fn to_days(self) -> i64 {
        // Howard Hinnant's `days_from_civil`.
        let y = if self.month <= 2 {
            self.year - 1
        } else {
            self.year
        } as i64;
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let m = self.month as i64;
        let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + self.day as i64 - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;

        era * 146_097 + doe - 719_468
    }

    /// The date `days` days after 1970-01-01.
    pub fn from_days(days: i64) -> Self {
        // Howard Hinnant's `civil_from_days`.
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
        let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
        let year = (yoe + era * 400 + i64::from(month <= 2)) as i32;

        Self { year, month, day }
    }

    /// Adds a number of days (possibly negative).
    pub fn add_days(self, days: i64) -> Self {
        Self::from_days(self.to_days() + days)
    }

    /// The day of the week, from 0 (Sunday) to 6 (Saturday).
    pub fn weekday(self) -> u32 {
        (self.to_days() + 4).rem_euclid(7) as u32
    }

    /// The month of the date.
    pub fn month_of(self) -> Month {
        Month {
            year: self.year,
            month: self.month,
        }
    }
}

impl fmt::Display for Date {
    /// Formats like `October 16, 2026`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}, {}",
            MONTHS[self.month as usize - 1],
            self.day,
            self.year
        )
    }
}

/// A month of a year.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Month {
    /// The year.
    pub year: i32,
    /// The month, from 1 to 12.
    pub month: u32,
}

impl Month {
    /// The current month, in UTC.
    pub fn current() -> Self {
        Date::today().month_of()
    }

    /// The next month.
    pub fn next(self) -> Self {
        if self.month == 12 {
            Self {
                year: self.year + 1,
                month: 1,
            }
        } else {
            Self {
                month: self.month + 1,
                ..self
            }
        }
    }

    /// The previous month.
    pub fn previous(self) -> Self {
        if self.month == 1 {
            Self {
                year: self.year - 1,
                month: 12,
            }
        } else {
            Self {
                month: self.month - 1,
                ..self
            }
        }
    }

    /// The first day of the month.
    pub fn first(self) -> Date {
        Date {
            year: self.year,
            month: self.month,
            day: 1,
        }
    }
}

impl fmt::Display for Month {
    /// Formats like `October 2026`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", MONTHS[self.month as usize - 1], self.year)
    }
}

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

const WEEKDAYS: [&str; 7] = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];

/// The number of days in a month.
pub fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 => 29,
        2 => 28,
        _ => 0,
    }
}

enum Selection {
    None,
    Single(Option<Date>),
    Range(Option<Date>, Option<Date>),
}

/// A calendar. See the [module documentation](self).
pub struct Calendar<'a, Message> {
    month: Month,
    on_navigate: Box<dyn Fn(Month) -> Message + 'a>,
    selection: Selection,
    on_select: Option<Box<dyn Fn(Date) -> Message + 'a>>,
    today: Option<Date>,
    disabled: Option<Box<dyn Fn(Date) -> bool + 'a>>,
    months: usize,
    outside_days: bool,
    bordered: bool,
}

/// Creates a [`Calendar`] displaying `month`.
pub fn calendar<'a, Message: Clone + 'a>(
    month: Month,
    on_navigate: impl Fn(Month) -> Message + 'a,
) -> Calendar<'a, Message> {
    Calendar {
        month,
        on_navigate: Box::new(on_navigate),
        selection: Selection::None,
        on_select: None,
        today: Some(Date::today()),
        disabled: None,
        months: 1,
        outside_days: true,
        bordered: false,
    }
}

impl<'a, Message: Clone + 'a> Calendar<'a, Message> {
    /// Highlights a single selected date (`mode="single"`).
    pub fn selected(mut self, date: Option<Date>) -> Self {
        self.selection = Selection::Single(date);
        self
    }

    /// Highlights a range of dates (`mode="range"`).
    pub fn range(mut self, start: Option<Date>, end: Option<Date>) -> Self {
        self.selection = Selection::Range(start, end);
        self
    }

    /// Sets the message produced when a day is pressed.
    pub fn on_select(mut self, f: impl Fn(Date) -> Message + 'a) -> Self {
        self.on_select = Some(Box::new(f));
        self
    }

    /// Sets the date highlighted as today, if any.
    pub fn today(mut self, today: Option<Date>) -> Self {
        self.today = today;
        self
    }

    /// Disables the days for which `f` returns `true`.
    pub fn disabled(mut self, f: impl Fn(Date) -> bool + 'a) -> Self {
        self.disabled = Some(Box::new(f));
        self
    }

    /// Sets how many months are displayed side by side.
    pub fn months(mut self, months: usize) -> Self {
        self.months = months.max(1);
        self
    }

    /// Shows the days of the adjacent months. Defaults to `true`.
    pub fn outside_days(mut self, show: bool) -> Self {
        self.outside_days = show;
        self
    }

    /// Draws a border around the calendar (`rounded-md border`).
    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }
}

#[derive(Clone, Copy, PartialEq)]
enum DayKind {
    Normal,
    Today,
    Selected,
    RangeMiddle,
    RangeStart,
    RangeEnd,
}

fn day_style(theme: &Theme, status: Status, kind: DayKind, outside: bool) -> Style {
    let t = tokens(theme);
    let p = &t.palette;
    let hovered = matches!(status, Status::Hovered | Status::Pressed);
    let radius = t.radius_md();

    let (background, text_color, corners) = match kind {
        DayKind::Selected => (
            Some(p.primary),
            p.primary_foreground,
            border::radius(radius),
        ),
        DayKind::RangeStart => (
            Some(p.primary),
            p.primary_foreground,
            border::radius(radius),
        ),
        DayKind::RangeEnd => (
            Some(p.primary),
            p.primary_foreground,
            border::radius(radius),
        ),
        DayKind::RangeMiddle => (Some(p.accent), p.accent_foreground, border::radius(0)),
        DayKind::Today => (Some(p.accent), p.accent_foreground, border::radius(radius)),
        DayKind::Normal if hovered => (Some(p.accent), p.accent_foreground, border::radius(radius)),
        DayKind::Normal => (None, p.foreground, border::radius(radius)),
    };

    let mut text_color = if outside && kind == DayKind::Normal {
        p.muted_foreground
    } else {
        text_color
    };

    if status == Status::Disabled {
        text_color = alpha(p.muted_foreground, style::DISABLED_OPACITY);
    }

    Style {
        background: background.map(Background::Color),
        text_color,
        border: iced::Border {
            radius: corners,
            ..iced::Border::default()
        },
        ..Style::default()
    }
}

fn nav_button<'a, Message: Clone + 'a>(
    icon: iced_ui_core::Icon,
    message: Message,
) -> Element<'a, Message> {
    super::button::icon_button(icon)
        .variant(super::button::Variant::Ghost)
        .size(super::button::Size::IconSm)
        .on_press(message)
        .into()
}

impl<'a, Message: Clone + 'a> Calendar<'a, Message> {
    fn month_grid(&self, month: Month) -> Element<'a, Message> {
        const CELL: f32 = 32.0;

        let weekdays = Row::with_children(WEEKDAYS.iter().map(|day| {
            container(text(*day).size(12.8).style(|theme: &Theme| text::Style {
                color: Some(tokens(theme).palette.muted_foreground),
            }))
            .center_x(CELL)
            .into()
        }));

        let first = month.first();
        let start = first.add_days(-(first.weekday() as i64));
        let mut weeks = Column::new().push(weekdays).spacing(8);

        let (range_start, range_end) = match self.selection {
            Selection::Range(start, end) => (start, end.or(start)),
            _ => (None, None),
        };

        for week in 0..6 {
            let mut row = Row::new();
            let mut any_inside = false;

            for weekday in 0..7 {
                let date = start.add_days(week * 7 + weekday);
                let outside = date.month_of() != month;
                any_inside |= !outside;

                if outside && !self.outside_days {
                    row = row.push(container(iced::widget::space()).width(CELL).height(CELL));
                    continue;
                }

                let disabled = self.disabled.as_ref().is_some_and(|f| f(date));

                let kind = match self.selection {
                    Selection::Single(Some(selected)) if selected == date => DayKind::Selected,
                    Selection::Range(..) if Some(date) == range_start => DayKind::RangeStart,
                    Selection::Range(..) if Some(date) == range_end => DayKind::RangeEnd,
                    Selection::Range(..)
                        if range_start
                            .zip(range_end)
                            .is_some_and(|(a, b)| date > a && date < b) =>
                    {
                        DayKind::RangeMiddle
                    }
                    _ if Some(date) == self.today => DayKind::Today,
                    _ => DayKind::Normal,
                };

                let on_select = (!disabled)
                    .then(|| self.on_select.as_ref().map(|f| f(date)))
                    .flatten();

                let label = text(date.day.to_string()).size(style::text::SM).font(
                    if matches!(
                        kind,
                        DayKind::Selected | DayKind::RangeStart | DayKind::RangeEnd
                    ) {
                        font::MEDIUM
                    } else {
                        font::SANS
                    },
                );

                row = row.push(
                    Pressable::new(container(label).center(CELL))
                        .on_press_maybe(on_select)
                        .disabled(disabled)
                        .style(move |theme, status| day_style(theme, status, kind, outside)),
                );
            }

            if any_inside || week < 5 {
                weeks = weeks.push(row);
            }
        }

        weeks.into()
    }
}

impl<'a, Message: Clone + 'a> From<Calendar<'a, Message>> for Element<'a, Message> {
    fn from(calendar: Calendar<'a, Message>) -> Self {
        let mut months = Row::new().spacing(16);
        let mut month = calendar.month;
        let count = calendar.months;

        for index in 0..count {
            let caption = text(month.to_string())
                .size(style::text::SM)
                .font(font::MEDIUM);

            let header = Row::new()
                .push((index == 0).then(|| {
                    nav_button(
                        lucide::CHEVRON_LEFT,
                        (calendar.on_navigate)(calendar.month.previous()),
                    )
                }))
                .push(container(caption).center_x(Length::Fill).center_y(32))
                .push((index + 1 == count).then(|| {
                    nav_button(
                        lucide::CHEVRON_RIGHT,
                        (calendar.on_navigate)(calendar.month.next()),
                    )
                }))
                .width(7.0 * 32.0)
                .align_y(iced::Alignment::Center);

            months = months.push(
                Column::new()
                    .push(header)
                    .push(calendar.month_grid(month))
                    .spacing(16),
            );
            month = month.next();
        }

        let bordered = calendar.bordered;

        container(months)
            .padding(12)
            .style(move |theme: &Theme| {
                let t = tokens(theme);
                container::Style {
                    background: Some(Background::Color(t.palette.background)),
                    border: if bordered {
                        style::border(t.palette.border, t.radius_md())
                    } else {
                        style::rounded(t.radius_md())
                    },
                    shadow: if bordered {
                        style::shadow_sm()
                    } else {
                        iced::Shadow::default()
                    },
                    ..container::Style::default()
                }
            })
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn days_round_trip() {
        for days in [-1_000_000, -1, 0, 1, 19_000, 20_742, 1_000_000] {
            assert_eq!(Date::from_days(days).to_days(), days);
        }
    }

    #[test]
    fn known_dates() {
        let date = Date::new(2026, 10, 16).unwrap();
        assert_eq!(date.weekday(), 5); // Friday
        assert_eq!(Date::from_days(0), Date::new(1970, 1, 1).unwrap());
        assert_eq!(
            Date::new(2024, 2, 29).map(|d| d.add_days(1)),
            Date::new(2024, 3, 1)
        );
        assert!(Date::new(2023, 2, 29).is_none());
        assert_eq!(date.to_string(), "October 16, 2026");
    }

    #[test]
    fn month_navigation() {
        let month = Month {
            year: 2026,
            month: 12,
        };
        assert_eq!(
            month.next(),
            Month {
                year: 2027,
                month: 1
            }
        );
        assert_eq!(month.next().previous(), month);
    }
}
