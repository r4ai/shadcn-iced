//! Touch and keyboard access to toasts, charts and tables.
use iced::keyboard::{self, Key, Modifiers, key::Named};
use iced::time::{Duration, Instant};
use iced::widget::{column, container, text};
use iced::{Element, Event, Point, touch, window};
use iced_test::simulator::Simulator;
use shadcn_iced::chart::{self, Series};
use shadcn_iced::{sonner, table, toast};

#[derive(Debug, Clone, PartialEq)]
enum Message {
    Dismiss(u64),
    Undo,
}

fn finger(id: u64, event: fn(touch::Finger, Point) -> touch::Event, at: Point) -> Event {
    Event::Touch(event(touch::Finger(id), at))
}

fn pressed(id: touch::Finger, position: Point) -> touch::Event {
    touch::Event::FingerPressed { id, position }
}

fn moved(id: touch::Finger, position: Point) -> touch::Event {
    touch::Event::FingerMoved { id, position }
}

fn lifted(id: touch::Finger, position: Point) -> touch::Event {
    touch::Event::FingerLifted { id, position }
}

/// Touches the screen with a finger, as iced does: the cursor moves to it
/// first.
fn touch(ui: &mut Simulator<'_, Message>, event: Event) {
    let Event::Touch(
        touch::Event::FingerPressed { position, .. }
        | touch::Event::FingerMoved { position, .. }
        | touch::Event::FingerLifted { position, .. }
        | touch::Event::FingerLost { position, .. },
    ) = event
    else {
        unreachable!()
    };

    ui.point_at(position);
    let _ = ui.simulate([event]);
}

fn tap(ui: &mut Simulator<'_, Message>, at: Point) {
    touch(ui, finger(0, pressed, at));
    touch(ui, finger(0, lifted, at));
}

fn redraw(at: Instant) -> Event {
    Event::Window(window::Event::RedrawRequested(at))
}

fn press(key: impl Into<Key>, modifiers: Modifiers) -> Event {
    let key = key.into();

    Event::Keyboard(keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: keyboard::key::Physical::Unidentified(
            keyboard::key::NativeCode::Unidentified,
        ),
        location: keyboard::Location::Standard,
        modifiers,
        repeat: false,
        text: None,
    })
}

const SIZE: (f32, f32) = (800.0, 600.0);

fn page() -> Element<'static, Message> {
    container(text("Outside")).padding(20).into()
}

fn sonner_toasts(action: bool) -> Simulator<'static, Message> {
    shadcn_iced::animation::set_enabled(false);

    let toast = sonner::Toast::new(1, "Saved");
    let toast = if action {
        toast.action("Undo", Message::Undo)
    } else {
        toast
    };

    Simulator::with_size(
        iced::Settings::default(),
        SIZE,
        sonner::toaster(page(), &[toast], Message::Dismiss),
    )
}

/// The center of the toast titled "Saved".
fn saved(ui: &mut Simulator<'_, Message>) -> Point {
    ui.find("Saved")
        .expect("the toast")
        .visible_bounds()
        .expect("a visible toast")
        .center()
}

fn dismissed(ui: Simulator<'_, Message>) -> bool {
    ui.into_messages()
        .any(|message| message == Message::Dismiss(1))
}

/// Taps the stack, and lets more than the lifetime of the toast pass.
fn tap_and_wait(outside: bool) -> bool {
    let mut ui = sonner_toasts(false);
    let start = Instant::now();
    let _ = ui.simulate([redraw(start)]);

    let toast = saved(&mut ui);
    tap(&mut ui, toast);

    if outside {
        tap(&mut ui, Point::new(30.0, 30.0));
    }

    let _ = ui.simulate([
        redraw(start + Duration::from_secs(1)),
        redraw(start + Duration::from_secs(6)),
    ]);

    dismissed(ui)
}

#[test]
fn a_tap_on_the_toasts_does_not_freeze_their_timers() {
    assert!(tap_and_wait(true), "tapped elsewhere");
    assert!(tap_and_wait(false), "the finger lifted");
}

/// Presses the toast with a finger and somewhere else with a second one,
/// then swipes to the right with the second finger, or with the first one.
fn swipe(first: bool) -> bool {
    let mut ui = sonner_toasts(false);
    let _ = ui.simulate([redraw(Instant::now())]);

    let toast = saved(&mut ui);
    let elsewhere = Point::new(100.0, 100.0);
    let right = iced::Vector::new(150.0, 0.0);
    let id = if first { 0 } else { 1 };
    let from = if first { toast } else { elsewhere };

    touch(&mut ui, finger(0, pressed, toast));
    touch(&mut ui, finger(1, pressed, elsewhere));
    touch(&mut ui, finger(id, moved, from + right));
    touch(&mut ui, finger(id, lifted, from + right));

    dismissed(ui)
}

#[test]
fn a_second_finger_does_not_swipe_a_toast() {
    assert!(!swipe(false), "the second finger swiped the toast");
    assert!(swipe(true), "the first finger did not swipe the toast");
}

#[test]
fn alt_t_focuses_the_toasts_of_sonner() {
    let mut ui = sonner_toasts(true);
    let _ = ui.simulate([redraw(Instant::now())]);

    let _ = ui.simulate([press(Key::Character("t".into()), Modifiers::ALT)]);
    let _ = ui.tap_key(Named::Enter);

    let messages: Vec<_> = ui.into_messages().collect();
    assert!(messages.contains(&Message::Undo), "{messages:?}");
}

#[test]
fn f8_focuses_the_toasts_and_escape_closes_one() {
    shadcn_iced::animation::set_enabled(false);

    let toasts = [toast::Toast::new(1, "Saved")];
    let mut ui = Simulator::with_size(
        iced::Settings::default(),
        SIZE,
        toast::toaster(page(), &toasts, Message::Dismiss),
    );
    let _ = ui.simulate([redraw(Instant::now())]);

    // Without the focus, Escape does nothing.
    let _ = ui.tap_key(Named::Escape);
    let _ = ui.simulate([press(Named::F8, Modifiers::empty())]);
    let _ = ui.tap_key(Named::Escape);

    let messages: Vec<_> = ui.into_messages().collect();
    assert_eq!(messages, vec![Message::Dismiss(1)]);
}

#[test]
fn the_focus_pauses_the_timers_of_the_toasts() {
    let mut ui = sonner_toasts(true);
    let start = Instant::now();

    let _ = ui.simulate([
        redraw(start),
        press(Key::Character("t".into()), Modifiers::ALT),
        redraw(start + Duration::from_secs(1)),
        redraw(start + Duration::from_secs(10)),
    ]);

    assert!(!dismissed(ui), "the focused toast expired");
}

fn bar_chart() -> Simulator<'static, Message> {
    shadcn_iced::animation::set_enabled(false);

    let chart = chart::bar(["January", "February", "March"])
        .series(Series::new("Desktop", [186.0, 305.0, 237.0]))
        .legend(false);

    Simulator::with_size(
        iced::Settings::default(),
        SIZE,
        shadcn_iced::primitive::tab_order(
            container(column![chart, text("Below")].spacing(50)).width(300),
        ),
    )
}

/// Whether the tooltip of `category` shows.
fn tooltip(ui: &mut Simulator<'_, Message>, category: &str) -> bool {
    ui.find(category).is_ok()
}

#[test]
fn a_finger_shows_the_tooltip_of_a_chart() {
    let mut ui = bar_chart();
    assert!(!tooltip(&mut ui, "February"));

    // A press shows the tooltip, without being captured: the page scrolls.
    let at = Point::new(150.0, 100.0);
    ui.point_at(at);
    let statuses = ui.simulate([finger(0, pressed, at)]);
    assert_eq!(statuses, vec![iced::event::Status::Ignored]);
    assert!(tooltip(&mut ui, "February"));

    // Scrubbing sideways moves it, and is captured.
    let at = Point::new(280.0, 100.0);
    ui.point_at(at);
    let statuses = ui.simulate([finger(0, moved, at)]);
    assert_eq!(statuses, vec![iced::event::Status::Captured]);
    assert!(tooltip(&mut ui, "March"));

    // It stays once the finger lifted, until a press elsewhere.
    touch(&mut ui, finger(0, lifted, at));
    assert!(tooltip(&mut ui, "March"));

    tap(&mut ui, Point::new(150.0, 500.0));
    assert!(!tooltip(&mut ui, "March"));
}

#[test]
fn arrows_move_the_tooltip_of_a_focused_chart() {
    let mut ui = bar_chart();

    let _ = ui.tap_key(Named::Tab);
    let _ = ui.simulate([redraw(Instant::now())]);
    assert!(tooltip(&mut ui, "January"), "the focus shows the first one");

    let _ = ui.tap_key(Named::ArrowRight);
    assert!(tooltip(&mut ui, "February"));

    let _ = ui.tap_key(Named::End);
    assert!(tooltip(&mut ui, "March"));

    let _ = ui.tap_key(Named::ArrowRight);
    assert!(tooltip(&mut ui, "March"), "the last one stays");

    let _ = ui.tap_key(Named::Home);
    assert!(tooltip(&mut ui, "January"));

    // Leaving the chart hides it.
    let _ = ui.simulate([finger(0, pressed, Point::new(150.0, 500.0))]);
    let _ = ui.simulate([redraw(Instant::now())]);
    assert!(!tooltip(&mut ui, "January"));
}

fn invoices() -> Simulator<'static, Message> {
    shadcn_iced::animation::set_enabled(false);

    Simulator::with_size(
        iced::Settings::default(),
        (400.0, 200.0),
        container(
            table::table(["Invoice", "Amount"])
                .row(["INV001", "$250.00"])
                .row(["INV002", "$150.00"]),
        )
        .padding(10),
    )
}

#[test]
fn a_tapped_table_row_is_not_hovered_once_the_finger_lifted() {
    let theme = iced::Theme::Light;
    let path = std::env::temp_dir().join(format!(
        "shadcn-iced-touch-feedback-{}/table",
        std::process::id()
    ));

    let mut ui = invoices();
    assert!(ui.snapshot(&theme).unwrap().matches_hash(&path).unwrap());

    let mut hovered = invoices();
    let row = {
        hovered
            .find("INV001")
            .unwrap()
            .visible_bounds()
            .unwrap()
            .center()
    };

    // While the finger touches the row, it is hovered.
    hovered.point_at(row);
    let _ = hovered.simulate([finger(0, pressed, row)]);
    assert!(
        !hovered
            .snapshot(&theme)
            .unwrap()
            .matches_hash(&path)
            .unwrap()
    );

    let _ = hovered.simulate([finger(0, lifted, row)]);
    assert!(
        hovered
            .snapshot(&theme)
            .unwrap()
            .matches_hash(&path)
            .unwrap()
    );

    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}
