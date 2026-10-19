//! Touch and keyboard behaviour of the lists: a tap selects an item and a
//! drag does not, `Space` selects in a select but types in a command, and
//! typing ahead moves the highlight.
use iced::keyboard::{self, Key, Modifiers, key::Named};
use iced::{Element, Event, Point, touch};
use iced_test::simulator::Simulator;
use shadcn_iced::command::{command, item};
use shadcn_iced::select::select;
use std::ops::{Deref, DerefMut};
use std::sync::{Mutex, MutexGuard};

#[derive(Debug, Clone, PartialEq)]
enum Message {
    Query(String),
    Picked(u8),
}

fn command_view() -> Element<'static, Message> {
    command("", Message::Query)
        .items([
            item("Alpha").on_select(Message::Picked(1)),
            item("Beta").on_select(Message::Picked(2)),
            item("Gamma").on_select(Message::Picked(3)),
        ])
        .into()
}

fn select_view() -> Element<'static, Message> {
    iced::widget::container(
        select(None, Message::Picked)
            .placeholder("Pick")
            .option(1, "Apple")
            .option(2, "Banana")
            .option(3, "Blueberry")
            .option(4, "Cherry"),
    )
    .padding(20)
    .into()
}

/// The tests run one at a time: the touch state of the pointer is shared,
/// and typing ahead is timed.
static SERIAL: Mutex<()> = Mutex::new(());

struct Ui {
    simulator: Simulator<'static, Message>,
    _serial: MutexGuard<'static, ()>,
}

impl Deref for Ui {
    type Target = Simulator<'static, Message>;

    fn deref(&self) -> &Self::Target {
        &self.simulator
    }
}

impl DerefMut for Ui {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.simulator
    }
}

fn simulator(view: Element<'static, Message>) -> Ui {
    let serial = SERIAL
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    shadcn_iced::animation::set_enabled(false);

    Ui {
        simulator: Simulator::with_size(iced::Settings::default(), (400.0, 500.0), view),
        _serial: serial,
    }
}

fn center(ui: &mut Ui, label: &str) -> Point {
    ui.find(label)
        .unwrap_or_else(|_| panic!("{label} is shown"))
        .bounds()
        .center()
}

/// A finger on the screen, where the cursor stays after it lifts, like in
/// `iced_winit`.
fn touch_at(ui: &mut Ui, position: Point, moves_to: Option<Point>) {
    let id = touch::Finger(0);

    ui.point_at(position);
    ui.simulate([Event::Touch(touch::Event::FingerPressed { id, position })]);

    if let Some(position) = moves_to {
        ui.point_at(position);
        ui.simulate([Event::Touch(touch::Event::FingerMoved { id, position })]);
    }

    let position = moves_to.unwrap_or(position);
    ui.simulate([Event::Touch(touch::Event::FingerLifted { id, position })]);
}

fn key_with(ui: &mut Ui, key: Key, modifiers: Modifiers) {
    ui.simulate([Event::Keyboard(keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: keyboard::key::Physical::Unidentified(
            keyboard::key::NativeCode::Unidentified,
        ),
        location: keyboard::Location::Standard,
        modifiers,
        text: None,
        repeat: false,
    })]);
}

fn picked(ui: Ui) -> Vec<Message> {
    ui.simulator
        .into_messages()
        .filter(|message| matches!(message, Message::Picked(_)))
        .collect()
}

#[test]
fn a_tap_selects_an_item_of_a_command() {
    let mut ui = simulator(command_view());

    let beta = center(&mut ui, "Beta");
    touch_at(&mut ui, beta, None);

    assert_eq!(picked(ui), [Message::Picked(2)]);
}

#[test]
fn a_drag_over_an_item_does_not_select_it() {
    let mut ui = simulator(command_view());

    let beta = center(&mut ui, "Beta");
    touch_at(&mut ui, beta, Some(Point::new(beta.x, beta.y + 40.0)));

    assert_eq!(picked(ui), []);
}

#[test]
fn a_tap_selects_an_option_of_a_select() {
    let mut ui = simulator(select_view());

    let trigger = center(&mut ui, "Pick");
    touch_at(&mut ui, trigger, None);

    let banana = center(&mut ui, "Banana");
    touch_at(&mut ui, banana, None);

    assert_eq!(picked(ui), [Message::Picked(2)]);
}

#[test]
fn a_drag_over_an_option_does_not_select_it() {
    let mut ui = simulator(select_view());

    let trigger = center(&mut ui, "Pick");
    touch_at(&mut ui, trigger, None);

    let banana = center(&mut ui, "Banana");
    touch_at(&mut ui, banana, Some(Point::new(banana.x, banana.y + 40.0)));

    assert_eq!(picked(ui), []);
}

#[test]
fn space_selects_the_highlighted_option_of_a_select() {
    let mut ui = simulator(select_view());

    let _ = ui.click("Pick").expect("the trigger");
    // The first option is highlighted when nothing is selected.
    ui.tap_key(Named::ArrowDown);
    ui.tap_key(Named::Space);

    assert_eq!(picked(ui), [Message::Picked(2)]);
}

#[test]
fn typing_highlights_the_matching_option_of_a_select() {
    let mut ui = simulator(select_view());

    let _ = ui.click("Pick").expect("the trigger");
    ui.typewrite("bl");
    ui.tap_key(Named::Enter);

    assert_eq!(picked(ui), [Message::Picked(3)]);
}

#[test]
fn a_character_highlights_the_first_option_starting_with_it() {
    let mut ui = simulator(select_view());

    let _ = ui.click("Pick").expect("the trigger");
    ui.typewrite("b");
    ui.tap_key(Named::Enter);

    assert_eq!(picked(ui), [Message::Picked(2)]);
}

#[test]
fn the_trigger_of_a_select_takes_the_focus() {
    let mut ui = simulator(select_view());

    let _ = ui.click("Pick").expect("the trigger");
    ui.tap_key(Named::Escape);
    assert!(ui.find("Banana").is_err(), "the list is closed");

    // Focused by the click, the trigger opens with the keyboard.
    ui.tap_key(Named::Space);
    assert!(ui.find("Banana").is_ok(), "the list is open");
}

#[test]
fn typing_on_the_closed_trigger_selects_an_option() {
    let mut ui = simulator(select_view());

    let _ = ui.click("Pick").expect("the trigger");
    ui.tap_key(Named::Escape);
    ui.typewrite("c");

    assert_eq!(picked(ui), [Message::Picked(4)]);
}

#[test]
fn ctrl_n_moves_the_highlight_of_a_command() {
    let mut ui = simulator(command_view());

    // Focus the search field.
    ui.point_at(Point::new(150.0, 20.0));
    ui.simulate(iced_test::simulator::click());

    key_with(&mut ui, Key::Character("n".into()), Modifiers::CTRL);
    key_with(&mut ui, Key::Character("j".into()), Modifiers::CTRL);
    key_with(&mut ui, Key::Character("k".into()), Modifiers::CTRL);
    ui.tap_key(Named::Enter);

    assert_eq!(picked(ui), [Message::Picked(2)]);
}

#[test]
fn space_types_in_a_command_instead_of_selecting() {
    let mut ui = simulator(command_view());

    ui.point_at(Point::new(150.0, 20.0));
    ui.simulate(iced_test::simulator::click());
    ui.tap_key(Named::Space);

    assert_eq!(picked(ui), []);
}
