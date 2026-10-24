//! The keyboard and touch of the calendar, like react-day-picker: the days
//! are one tab stop, and the keys move the focus between them.
use iced::keyboard::{self, Key, key::Named};
use iced::widget::container;
use iced::{Element, Event, touch};
use iced_test::simulator::{Simulator, release_key};
use shadcn_iced::calendar::{Calendar, Date, Month, calendar};
use shadcn_iced::date_picker::date_picker;

#[derive(Debug, Clone, PartialEq)]
enum Message {
    Navigate(Month),
    Select(Date),
    Open,
    Dismiss,
}

fn date(year: i32, month: u32, day: u32) -> Date {
    Date::new(year, month, day).unwrap()
}

const OCTOBER: Month = Month {
    year: 2026,
    month: 10,
};

/// A calendar of October 2026 (today is October 20th).
fn october() -> Calendar<'static, Message> {
    calendar(OCTOBER, Message::Navigate)
        .on_select(Message::Select)
        .today(Some(date(2026, 10, 20)))
}

fn simulator(view: impl Into<Element<'static, Message>>) -> Simulator<'static, Message> {
    shadcn_iced::animation::set_enabled(false);

    Simulator::with_size(
        iced::Settings::default(),
        (700.0, 500.0),
        shadcn_iced::primitive::tab_order(container(view).padding(20)),
    )
}

/// Moves the focus to the days: after the previous and next month buttons.
fn focus_days(ui: &mut Simulator<'_, Message>) {
    for _ in 0..3 {
        let _ = ui.tap_key(Named::Tab);
    }
}

fn press(ui: &mut Simulator<'_, Message>, key: Named, modifiers: keyboard::Modifiers) {
    let key = Key::Named(key);

    let _ = ui.simulate([
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key.clone(),
            physical_key: keyboard::key::Physical::Unidentified(
                keyboard::key::NativeCode::Unidentified,
            ),
            location: keyboard::Location::Standard,
            modifiers,
            repeat: false,
            text: None,
        }),
        release_key(key),
    ]);
}

/// Presses the keys, then <kbd>Enter</kbd>, and returns the messages.
fn keys(mut ui: Simulator<'_, Message>, keys: &[Named]) -> Vec<Message> {
    for key in keys {
        let _ = ui.tap_key(*key);
    }

    let _ = ui.tap_key(Named::Enter);

    ui.into_messages().collect()
}

#[test]
fn tab_focuses_the_selected_day() {
    let mut ui = simulator(october().selected(Some(date(2026, 10, 16))));
    focus_days(&mut ui);

    assert_eq!(keys(ui, &[]), vec![Message::Select(date(2026, 10, 16))]);
}

#[test]
fn tab_focuses_today_else_the_first_enabled_day() {
    let mut ui = simulator(october());
    focus_days(&mut ui);

    // Space selects too.
    let _ = ui.tap_key(Named::Space);
    assert_eq!(
        ui.into_messages().collect::<Vec<_>>(),
        vec![Message::Select(date(2026, 10, 20))]
    );

    let mut ui = simulator(october().today(None).disabled(|date| date.day <= 3));
    focus_days(&mut ui);

    assert_eq!(keys(ui, &[]), vec![Message::Select(date(2026, 10, 4))]);
}

#[test]
fn arrows_move_by_days_and_weeks() {
    let mut ui = simulator(october().selected(Some(date(2026, 10, 16))));
    focus_days(&mut ui);

    // 16 + 1 + 7 - 1 - 7 - 7
    use Named::{ArrowDown, ArrowLeft, ArrowRight, ArrowUp};
    assert_eq!(
        keys(ui, &[ArrowRight, ArrowDown, ArrowLeft, ArrowUp, ArrowUp]),
        vec![Message::Select(date(2026, 10, 9))]
    );
}

#[test]
fn home_and_end_move_within_the_week() {
    // October 16th, 2026 is a Friday; the weeks start on Sunday.
    let mut ui = simulator(october().selected(Some(date(2026, 10, 16))));
    focus_days(&mut ui);
    assert_eq!(
        keys(ui, &[Named::Home]),
        vec![Message::Select(date(2026, 10, 11))]
    );

    let mut ui = simulator(october().selected(Some(date(2026, 10, 16))));
    focus_days(&mut ui);
    assert_eq!(
        keys(ui, &[Named::End]),
        vec![Message::Select(date(2026, 10, 17))]
    );
}

#[test]
fn arrows_cross_the_months() {
    let mut ui = simulator(october().selected(Some(date(2026, 10, 31))));
    focus_days(&mut ui);
    assert_eq!(
        keys(ui, &[Named::ArrowRight]),
        vec![
            Message::Navigate(Month {
                year: 2026,
                month: 11
            }),
            Message::Select(date(2026, 11, 1)),
        ]
    );

    let mut ui = simulator(october().selected(Some(date(2026, 10, 1))));
    focus_days(&mut ui);
    assert_eq!(
        keys(ui, &[Named::ArrowUp]),
        vec![
            Message::Navigate(Month {
                year: 2026,
                month: 9
            }),
            Message::Select(date(2026, 9, 24)),
        ]
    );

    // The next month is displayed already.
    let mut ui = simulator(october().months(2).selected(Some(date(2026, 10, 31))));
    focus_days(&mut ui);
    assert_eq!(
        keys(ui, &[Named::ArrowRight]),
        vec![Message::Select(date(2026, 11, 1))]
    );
}

#[test]
fn page_keys_switch_the_month_and_the_year() {
    let mut ui = simulator(october().selected(Some(date(2026, 10, 31))));
    focus_days(&mut ui);
    // The same day, or the last one of a shorter month.
    press(&mut ui, Named::PageUp, keyboard::Modifiers::empty());
    assert_eq!(
        keys(ui, &[]),
        vec![
            Message::Navigate(Month {
                year: 2026,
                month: 9
            }),
            Message::Select(date(2026, 9, 30)),
        ]
    );

    let mut ui = simulator(october().selected(Some(date(2026, 10, 31))));
    focus_days(&mut ui);
    press(&mut ui, Named::PageDown, keyboard::Modifiers::SHIFT);
    assert_eq!(
        keys(ui, &[]),
        vec![
            Message::Navigate(Month {
                year: 2027,
                month: 10
            }),
            Message::Select(date(2027, 10, 31)),
        ]
    );
}

#[test]
fn moves_skip_the_disabled_days() {
    let disabled = |date: Date| date.month == 10 && (17..=18).contains(&date.day);

    let mut ui = simulator(
        october()
            .selected(Some(date(2026, 10, 16)))
            .disabled(disabled),
    );
    focus_days(&mut ui);
    assert_eq!(
        keys(ui, &[Named::ArrowRight]),
        vec![Message::Select(date(2026, 10, 19))]
    );

    // Nothing to move to at the end of a week starting disabled.
    let mut ui = simulator(
        october()
            .selected(Some(date(2026, 10, 16)))
            .disabled(disabled),
    );
    focus_days(&mut ui);
    assert_eq!(
        keys(ui, &[Named::End]),
        vec![Message::Select(date(2026, 10, 16))]
    );
}

#[test]
fn moves_stay_within_the_dropdown_years() {
    let mut ui = simulator(
        october()
            .dropdowns(2020, 2026)
            .selected(Some(date(2026, 10, 16))),
    );
    // The days are the last tab stop.
    press(&mut ui, Named::Tab, keyboard::Modifiers::SHIFT);
    press(&mut ui, Named::PageDown, keyboard::Modifiers::SHIFT);

    assert_eq!(
        keys(ui, &[]),
        vec![
            Message::Navigate(Month {
                year: 2026,
                month: 12
            }),
            Message::Select(date(2026, 12, 31)),
        ]
    );
}

#[test]
fn a_tap_selects_and_focuses_a_day() {
    let mut ui = simulator(october());
    let center = ui
        .find("20")
        .expect("October 20th")
        .visible_bounds()
        .expect("visible")
        .center();
    let finger = touch::Finger(0);

    ui.point_at(center);
    let _ = ui.simulate([Event::Touch(touch::Event::FingerPressed {
        id: finger,
        position: center,
    })]);
    ui.point_at(center);
    let _ = ui.simulate([Event::Touch(touch::Event::FingerLifted {
        id: finger,
        position: center,
    })]);

    assert_eq!(
        keys(ui, &[Named::ArrowLeft]),
        vec![
            Message::Select(date(2026, 10, 20)),
            Message::Select(date(2026, 10, 19)),
        ]
    );
}

#[test]
fn the_date_picker_focuses_the_days_when_a_key_opens_it() {
    let picker = |open| {
        date_picker(
            Some(date(2026, 10, 16)),
            Message::Select,
            OCTOBER,
            Message::Navigate,
        )
        .open(open, Message::Dismiss)
        .on_open(Message::Open)
    };

    // The trigger is the only tab stop while the calendar is closed.
    let mut ui = simulator(picker(false));
    let _ = ui.tap_key(Named::Tab);
    let _ = ui.tap_key(Named::Enter);
    assert_eq!(ui.into_messages().collect::<Vec<_>>(), vec![Message::Open]);

    let ui = simulator(picker(true));
    assert_eq!(
        keys(ui, &[Named::ArrowRight]),
        vec![Message::Select(date(2026, 10, 17))]
    );
}
