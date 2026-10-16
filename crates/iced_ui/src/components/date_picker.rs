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
use iced_ui_core::{Element, style, tokens};

use iced::widget::{Row, container, text};
use iced::{Alignment, Length, Theme};

/// A date picker. See the [module documentation](self).
pub struct DatePicker<'a, Message> {
    selected: Option<Date>,
    on_select: Box<dyn Fn(Date) -> Message + 'a>,
    month: Month,
    on_navigate: Box<dyn Fn(Month) -> Message + 'a>,
    placeholder: String,
    width: f32,
    open: Option<(bool, Message)>,
}

/// Creates a [`DatePicker`]. The displayed month lives in your state.
pub fn date_picker<'a, Message: Clone + 'a>(
    selected: Option<Date>,
    on_select: impl Fn(Date) -> Message + 'a,
    month: Month,
    on_navigate: impl Fn(Month) -> Message + 'a,
) -> DatePicker<'a, Message> {
    DatePicker {
        selected,
        on_select: Box::new(on_select),
        month,
        on_navigate: Box::new(on_navigate),
        placeholder: "Pick a date".to_owned(),
        width: 240.0,
        open: None,
    }
}

impl<'a, Message: Clone + 'a> DatePicker<'a, Message> {
    /// Sets the text shown when no date is selected.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Sets the width of the trigger.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
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
}

impl<'a, Message: Clone + 'a> From<DatePicker<'a, Message>> for Element<'a, Message> {
    fn from(picker: DatePicker<'a, Message>) -> Self {
        let has_value = picker.selected.is_some();
        let label = picker
            .selected
            .map_or(picker.placeholder.clone(), |date| date.to_string());

        let trigger = Button::new(
            Row::new()
                .push(lucide::CALENDAR.sized(16))
                .push(
                    text(label)
                        .size(style::text::SM)
                        .width(Length::Fill)
                        .wrapping(text::Wrapping::None)
                        .style(move |theme: &Theme| text::Style {
                            color: (!has_value).then(|| tokens(theme).palette.muted_foreground),
                        }),
                )
                .spacing(8)
                .align_y(Alignment::Center),
        )
        .variant(Variant::Outline)
        .width(picker.width);

        let content = container(
            calendar(picker.month, picker.on_navigate)
                .selected(picker.selected)
                .on_select(picker.on_select),
        )
        .style(super::popover::style);

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
