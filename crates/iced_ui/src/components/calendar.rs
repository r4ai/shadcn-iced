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
use std::rc::Rc;

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
    on_navigate: Rc<dyn Fn(Month) -> Message + 'a>,
    selection: Selection,
    on_select: Option<Box<dyn Fn(Date) -> Message + 'a>>,
    today: Option<Date>,
    disabled: Option<Box<dyn Fn(Date) -> bool + 'a>>,
    struck: Option<Box<dyn Fn(Date) -> bool + 'a>>,
    months: usize,
    outside_days: bool,
    bordered: bool,
    transparent: bool,
    cell: f32,
    padding: f32,
    fixed_weeks: bool,
    week_numbers: bool,
    dropdowns: Option<(i32, i32)>,
}

/// Creates a [`Calendar`] displaying `month`.
pub fn calendar<'a, Message: Clone + 'a>(
    month: Month,
    on_navigate: impl Fn(Month) -> Message + 'a,
) -> Calendar<'a, Message> {
    Calendar {
        month,
        on_navigate: Rc::new(on_navigate),
        selection: Selection::None,
        on_select: None,
        today: Some(Date::today()),
        disabled: None,
        struck: None,
        months: 1,
        outside_days: true,
        bordered: false,
        transparent: false,
        cell: 28.0,
        padding: 8.0,
        fixed_weeks: false,
        week_numbers: false,
        dropdowns: None,
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

    /// Disables the days for which `f` returns `true`
    /// (`text-muted-foreground opacity-50`).
    pub fn disabled(mut self, f: impl Fn(Date) -> bool + 'a) -> Self {
        self.disabled = Some(Box::new(f));
        self
    }

    /// Strikes through the days for which `f` returns `true`, e.g. booked
    /// dates (`[&>button]:line-through`).
    pub fn struck(mut self, f: impl Fn(Date) -> bool + 'a) -> Self {
        self.struck = Some(Box::new(f));
        self
    }

    /// Sets how many months are displayed side by side
    /// (`numberOfMonths`).
    pub fn months(mut self, months: usize) -> Self {
        self.months = months.max(1);
        self
    }

    /// Shows the days of the adjacent months. Defaults to `true`.
    pub fn outside_days(mut self, show: bool) -> Self {
        self.outside_days = show;
        self
    }

    /// Removes the background, e.g. inside a popover or a card
    /// (`in-data-[slot=card-content]:bg-transparent`).
    pub fn transparent(mut self, transparent: bool) -> Self {
        self.transparent = transparent;
        self
    }

    /// Draws a border around the calendar (`rounded-lg border`).
    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }

    /// Sets the size of a day (`--cell-size`, `--spacing(7)` by default).
    pub fn cell_size(mut self, size: f32) -> Self {
        self.cell = size;
        self
    }

    /// Sets the padding around the months (`p-2` by default).
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    /// Always shows six weeks (`fixedWeeks`).
    pub fn fixed_weeks(mut self, fixed: bool) -> Self {
        self.fixed_weeks = fixed;
        self
    }

    /// Shows the number of each week (`showWeekNumber`).
    pub fn week_numbers(mut self, show: bool) -> Self {
        self.week_numbers = show;
        self
    }

    /// Replaces the caption with month and year dropdowns covering the
    /// given years (`captionLayout="dropdown"`).
    pub fn dropdowns(mut self, from_year: i32, to_year: i32) -> Self {
        self.dropdowns = Some((from_year.min(to_year), from_year.max(to_year)));
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

/// Which corners of a day are rounded (`rounded-(--cell-radius)`).
#[derive(Clone, Copy, PartialEq)]
struct Corners {
    left: bool,
    right: bool,
}

impl Corners {
    const ALL: Self = Self {
        left: true,
        right: true,
    };

    fn radius(self, radius: f32) -> border::Radius {
        let left = if self.left { radius } else { 0.0 };
        let right = if self.right { radius } else { 0.0 };

        border::radius(0)
            .top_left(left)
            .bottom_left(left)
            .top_right(right)
            .bottom_right(right)
    }
}

/// The style of a day button.
fn day_style(
    theme: &Theme,
    status: Status,
    kind: DayKind,
    outside: bool,
    corners: Corners,
) -> Style {
    let t = tokens(theme);
    let p = &t.palette;
    let dark = t.is_dark();
    let hovered = matches!(status, Status::Hovered | Status::Pressed);
    // `--cell-radius: var(--radius-md)`
    let radius = t.radius_md();

    // Ghost button: `hover:bg-muted dark:hover:bg-muted/50`
    let hover = if dark { alpha(p.muted, 0.5) } else { p.muted };

    let (background, text_color) = match kind {
        DayKind::Selected | DayKind::RangeStart | DayKind::RangeEnd => {
            (Some(p.primary), p.primary_foreground)
        }
        DayKind::RangeMiddle => (Some(p.muted), p.foreground),
        DayKind::Today => (Some(p.muted), p.foreground),
        DayKind::Normal if hovered && status != Status::Disabled => (Some(hover), p.foreground),
        DayKind::Normal => (None, p.foreground),
    };

    let mut text_color = if outside && kind != DayKind::Selected {
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
            radius: corners.radius(radius),
            ..iced::Border::default()
        },
        ..Style::default()
    }
}

fn nav_button<'a, Message: Clone + 'a>(
    icon: iced_ui_core::Icon,
    message: Message,
    cell: f32,
) -> Element<'a, Message> {
    // `buttonVariants({ variant: "ghost" }) size-(--cell-size) p-0`
    super::button::icon_button(icon)
        .variant(super::button::Variant::Ghost)
        .size(super::button::Size::IconSm)
        .width(cell)
        .on_press(message)
        .into()
}

fn muted(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(tokens(theme).palette.muted_foreground),
    }
}

/// The week number of the week ending with `saturday`: the week holding
/// January 1st is the first one.
fn week_number(saturday: Date) -> i64 {
    let jan1 = Date {
        year: saturday.year,
        month: 1,
        day: 1,
    };
    let first_sunday = jan1.to_days() - jan1.weekday() as i64;

    (saturday.to_days() - first_sunday) / 7 + 1
}

impl<'a, Message: Clone + 'a> Calendar<'a, Message> {
    fn month_grid(&self, month: Month) -> Element<'a, Message> {
        let cell = self.cell;

        // `text-[0.8rem] font-normal text-muted-foreground`
        let weekdays = Row::new()
            .push(
                self.week_numbers
                    .then(|| iced::widget::space().width(cell).height(1)),
            )
            .extend(WEEKDAYS.iter().map(|day| {
                container(text(*day).size(12.8).style(muted))
                    .center_x(cell)
                    .into()
            }));

        let first = month.first();
        let start = first.add_days(-(first.weekday() as i64));
        // `mt-2` on every week
        let mut weeks = Column::new().push(weekdays).spacing(8);

        let (range_start, range_end) = match self.selection {
            Selection::Range(start, end) => (start, end.or(start)),
            _ => (None, None),
        };

        for week in 0..6 {
            let mut row = Row::new();
            let mut any_inside = false;

            if self.week_numbers {
                row = row.push(
                    container(
                        text(week_number(start.add_days(week * 7 + 6)).to_string())
                            .size(12.8)
                            .style(muted),
                    )
                    .center(cell),
                );
            }

            for weekday in 0..7 {
                let date = start.add_days(week * 7 + weekday);
                let outside = date.month_of() != month;
                any_inside |= !outside;

                if outside && !self.outside_days {
                    row = row.push(iced::widget::space().width(cell).height(cell));
                    continue;
                }

                let disabled = self.disabled.as_ref().is_some_and(|f| f(date));
                let struck = self.struck.as_ref().is_some_and(|f| f(date));

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

                // Middle days are square, except at the ends of a week
                // (`[&:first-child[data-selected=true]_button]:rounded-l`).
                let corners = if kind == DayKind::RangeMiddle {
                    Corners {
                        left: weekday == 0,
                        right: weekday == 6,
                    }
                } else {
                    Corners::ALL
                };

                let on_select = (!disabled)
                    .then(|| self.on_select.as_ref().map(|f| f(date)))
                    .flatten();

                // `text-sm font-normal leading-none`
                let label = text(date.day.to_string())
                    .size(style::text::SM)
                    .font(font::sans())
                    .line_height(1.0);

                let label: Element<'a, Message> = if struck {
                    let span: iced::widget::text::Span<'a, (), iced::Font> =
                        iced::widget::span(date.day.to_string()).strikethrough(true);

                    iced::widget::rich_text([span])
                        .size(style::text::SM)
                        .line_height(1.0)
                        .into()
                } else {
                    label.into()
                };

                let day = Pressable::new(container(label).center(cell))
                    .on_press_maybe(on_select)
                    .disabled(disabled)
                    .style(move |theme, status| day_style(theme, status, kind, outside, corners));

                // The range continues behind its ends
                // (`range_start: rounded-l-(--cell-radius) bg-muted`).
                let day: Element<'a, Message> = match kind {
                    DayKind::RangeStart | DayKind::RangeEnd if range_start != range_end => {
                        let start = kind == DayKind::RangeStart;
                        container(day)
                            .style(move |theme: &Theme| {
                                let t = tokens(theme);
                                container::Style {
                                    background: Some(Background::Color(t.palette.muted)),
                                    border: iced::Border {
                                        radius: Corners {
                                            left: start || weekday == 0,
                                            right: !start || weekday == 6,
                                        }
                                        .radius(t.radius_md()),
                                        ..iced::Border::default()
                                    },
                                    ..container::Style::default()
                                }
                            })
                            .into()
                    }
                    _ => day.into(),
                };

                row = row.push(day);
            }

            if any_inside || self.fixed_weeks {
                weeks = weeks.push(row);
            }
        }

        weeks.into()
    }

    fn caption(&self, month: Month) -> Element<'a, Message> {
        let Some((from, to)) = self.dropdowns else {
            // `text-sm font-medium`
            return text(month.to_string())
                .size(style::text::SM)
                .font(font::medium())
                .into();
        };

        let year = month.year;
        let current = month.month;
        let on_month = self.on_navigate.clone();
        let on_year = self.on_navigate.clone();

        let months = (1..=12).fold(
            super::select::select(Some(current), move |month| on_month(Month { year, month })),
            |select, m| select.option(m, &MONTHS[m as usize - 1][..3]),
        );
        let years = (from..=to).fold(
            super::select::select(Some(year), move |year| {
                on_year(Month {
                    year,
                    month: current,
                })
            }),
            |select, y| select.option(y, y.to_string()),
        );

        // `gap-1.5`
        Row::new()
            .push(months.size(super::select::Size::Sm))
            .push(years.size(super::select::Size::Sm))
            .spacing(6)
            .into()
    }
}

impl<'a, Message: Clone + 'a> From<Calendar<'a, Message>> for Element<'a, Message> {
    fn from(calendar: Calendar<'a, Message>) -> Self {
        let cell = calendar.cell;
        let count = calendar.months;
        let grid_width = cell * if calendar.week_numbers { 8.0 } else { 7.0 };
        let mut months = Row::new().spacing(16);
        let mut month = calendar.month;

        for index in 0..count {
            // `month_caption: h-(--cell-size) px-(--cell-size)`, with the
            // navigation at both ends of the months (`nav: absolute`).
            let header = Row::new()
                .push(if index == 0 {
                    nav_button(
                        lucide::CHEVRON_LEFT,
                        (calendar.on_navigate)(calendar.month.previous()),
                        cell,
                    )
                } else {
                    iced::widget::space().width(cell).into()
                })
                .push(
                    container(calendar.caption(month))
                        .center_x(Length::Fill)
                        .center_y(cell),
                )
                .push(if index + 1 == count {
                    nav_button(
                        lucide::CHEVRON_RIGHT,
                        (calendar.on_navigate)(calendar.month.next()),
                        cell,
                    )
                } else {
                    iced::widget::space().width(cell).into()
                })
                .width(grid_width)
                .height(cell)
                .align_y(iced::Alignment::Center);

            // `month: gap-4`
            months = months.push(
                Column::new()
                    .push(header)
                    .push(calendar.month_grid(month))
                    .spacing(16),
            );
            month = month.next();
        }

        let bordered = calendar.bordered;
        let transparent = calendar.transparent;

        container(months)
            .padding(calendar.padding)
            .style(move |theme: &Theme| {
                let t = tokens(theme);
                container::Style {
                    background: (!transparent).then_some(Background::Color(t.palette.background)),
                    border: if bordered {
                        style::border(t.palette.border, t.radius_lg())
                    } else {
                        style::rounded(t.radius_lg())
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
        // The week of January 1st is the first one.
        assert_eq!(week_number(Date::new(2026, 1, 3).unwrap()), 1);
        assert_eq!(week_number(Date::new(2026, 1, 10).unwrap()), 2);
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
