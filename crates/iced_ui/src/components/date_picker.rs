//! A date picker component with range and presets.
//!
//! Port of <https://ui.shadcn.com/docs/components/date-picker>: a
//! [`popover`](super::popover) containing a [`calendar`](super::calendar).
//!
//! ```no_run
//! # use iced_ui::calendar::{Date, Month};
//! # use iced_ui::date_picker::date_picker;
//! # #[derive(Clone)] enum Message { Navigate(Month), Select(Date) }
//! # fn view<'a>(month: Month, date: Option<Date>) -> iced::Element<'a, Message> {
//! date_picker(date, Message::Select, month, Message::Navigate).into()
//! # }
//! ```
use super::button::{Button, Variant};
use super::calendar::{Date, Month, calendar};

use iced_ui_core::icon::lucide;
use iced_ui_core::primitive::{Align, Anchored, Side};
use iced_ui_core::{Element, font, style, tokens};

use iced::widget::{container, text};
use iced::{Length, Theme};

enum Selection {
    Single(Option<Date>),
    Range(Option<Date>, Option<Date>),
}

/// A date picker. See the [module documentation](self).
pub struct DatePicker<'a, Message> {
    selection: Selection,
    on_select: Box<dyn Fn(Date) -> Message + 'a>,
    month: Month,
    on_navigate: Box<dyn Fn(Month) -> Message + 'a>,
    placeholder: String,
    width: Length,
    open: Option<(bool, Message)>,
    on_open: Option<Message>,
    format: fn(Date) -> String,
    icon: bool,
    chevron: bool,
    months: usize,
    dropdowns: Option<(i32, i32)>,
}

/// Creates a [`DatePicker`] for a single date. The displayed month lives in
/// your state.
pub fn date_picker<'a, Message: Clone + 'a>(
    selected: Option<Date>,
    on_select: impl Fn(Date) -> Message + 'a,
    month: Month,
    on_navigate: impl Fn(Month) -> Message + 'a,
) -> DatePicker<'a, Message> {
    DatePicker {
        selection: Selection::Single(selected),
        on_select: Box::new(on_select),
        month,
        on_navigate: Box::new(on_navigate),
        placeholder: "Pick a date".to_owned(),
        // `w-[212px]`
        width: Length::Fixed(212.0),
        open: None,
        on_open: None,
        format: long_date,
        icon: false,
        chevron: true,
        months: 1,
        dropdowns: None,
    }
}

/// Creates a [`DatePicker`] for a range of dates, showing two months
/// (`mode="range"`). `on_select` receives every pressed day; extending the
/// range is up to you.
pub fn date_range_picker<'a, Message: Clone + 'a>(
    (start, end): (Option<Date>, Option<Date>),
    on_select: impl Fn(Date) -> Message + 'a,
    month: Month,
    on_navigate: impl Fn(Month) -> Message + 'a,
) -> DatePicker<'a, Message> {
    DatePicker {
        selection: Selection::Range(start, end),
        format: short_date,
        icon: true,
        chevron: false,
        months: 2,
        width: Length::Fill,
        ..date_picker(None, on_select, month, on_navigate)
    }
}

impl<'a, Message: Clone + 'a> DatePicker<'a, Message> {
    /// Sets the text shown when no date is selected.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Sets the width of the trigger.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Controls whether the calendar is open, e.g. to close it once a date
    /// is picked. `on_dismiss` is produced on a click outside or
    /// <kbd>Escape</kbd>. By default, the popover manages itself and stays
    /// open until dismissed.
    pub fn open(mut self, open: bool, on_dismiss: Message) -> Self {
        self.open = Some((open, on_dismiss));
        self
    }

    /// Sets the message produced when the trigger is pressed, to open a
    /// controlled date picker (see [`open`](Self::open)).
    pub fn on_open(mut self, message: Message) -> Self {
        self.on_open = Some(message);
        self
    }

    /// Sets how dates are written in the trigger: [`long_date`] (`PPP`) by
    /// default, [`short_date`] (`LLL dd, y`) for ranges.
    pub fn format(mut self, format: fn(Date) -> String) -> Self {
        self.format = format;
        self
    }

    /// Shows a calendar icon before the label (`data-icon="inline-start"`).
    pub fn icon(mut self, icon: bool) -> Self {
        self.icon = icon;
        self
    }

    /// Shows a chevron after the label (`data-icon="inline-end"`).
    pub fn chevron(mut self, chevron: bool) -> Self {
        self.chevron = chevron;
        self
    }

    /// Sets how many months the calendar shows.
    pub fn months(mut self, months: usize) -> Self {
        self.months = months;
        self
    }

    /// Shows month and year dropdowns in the calendar
    /// (`captionLayout="dropdown"`).
    pub fn dropdowns(mut self, from_year: i32, to_year: i32) -> Self {
        self.dropdowns = Some((from_year, to_year));
        self
    }
}

/// Writes a date like `October 16th, 2026` (`PPP` in date-fns).
pub fn long_date(date: Date) -> String {
    let suffix = match (date.day % 10, date.day % 100) {
        (_, 11..=13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    let month = Month {
        year: date.year,
        month: date.month,
    }
    .to_string();
    let name = month.split(' ').next().unwrap_or_default();

    format!("{name} {}{suffix}, {}", date.day, date.year)
}

/// Writes a date like `Oct 06, 2026` (`LLL dd, y` in date-fns).
pub fn short_date(date: Date) -> String {
    let month = Month {
        year: date.year,
        month: date.month,
    }
    .to_string();

    format!("{} {:02}, {}", &month[..3], date.day, date.year)
}

/// Writes a date like `10/16/2026` (`toLocaleDateString()` in en-US).
pub fn numeric_date(date: Date) -> String {
    format!("{}/{}/{}", date.month, date.day, date.year)
}

impl<'a, Message: Clone + 'a> From<DatePicker<'a, Message>> for Element<'a, Message> {
    fn from(picker: DatePicker<'a, Message>) -> Self {
        let format = picker.format;
        let label = match picker.selection {
            Selection::Single(Some(date)) => Some(format(date)),
            Selection::Range(Some(start), Some(end)) => {
                Some(format!("{} - {}", format(start), format(end)))
            }
            Selection::Range(Some(start), None) => Some(format(start)),
            _ => None,
        };
        let empty = label.is_none();

        // `justify-between text-left font-normal
        // data-[empty=true]:text-muted-foreground`
        let mut trigger = Button::new(
            text(label.unwrap_or_else(|| picker.placeholder.clone()))
                .size(style::text::SM)
                .font(font::sans())
                .width(Length::Fill)
                .wrapping(text::Wrapping::None)
                .style(move |theme: &Theme| text::Style {
                    color: empty.then(|| tokens(theme).palette.muted_foreground),
                }),
        )
        .variant(Variant::Outline)
        .on_press_maybe(picker.on_open)
        .width(picker.width);

        if picker.icon {
            trigger = trigger.icon(lucide::CALENDAR);
        }

        if picker.chevron {
            trigger = trigger.trailing_icon(lucide::CHEVRON_DOWN);
        }

        let mut calendar = calendar(picker.month, picker.on_navigate)
            .on_select(picker.on_select)
            .months(picker.months)
            .transparent(true);

        calendar = match picker.selection {
            Selection::Single(date) => calendar.selected(date),
            Selection::Range(start, end) => calendar.range(start, end),
        };

        if let Some((from, to)) = picker.dropdowns {
            calendar = calendar.dropdowns(from, to);
        }

        // `PopoverContent className="w-auto p-0"`
        let content = container(calendar).style(super::popover::style);

        let anchored = Anchored::new(trigger, content)
            .side(Side::Bottom)
            .align(Align::Start)
            .offset(4.0);

        match picker.open {
            Some((open, on_dismiss)) => anchored.open(open).on_dismiss(on_dismiss).into(),
            None => anchored.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats() {
        let date = Date::new(2026, 10, 1).unwrap();
        assert_eq!(long_date(date), "October 1st, 2026");
        assert_eq!(long_date(date.add_days(11)), "October 12th, 2026");
        assert_eq!(long_date(date.add_days(21)), "October 22nd, 2026");
        assert_eq!(short_date(date.add_days(5)), "Oct 06, 2026");
        assert_eq!(numeric_date(date), "10/1/2026");
    }
}
