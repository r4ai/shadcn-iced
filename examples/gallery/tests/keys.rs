//! Keyboard shortcuts scoped to a region (`shadcn_iced::primitive::keys`).
use iced::keyboard::{Key, key::Named};
use iced::widget::{column, container, text, text_input};
use iced_test::simulator::Simulator;

#[derive(Debug, Clone, PartialEq)]
enum Message {
    Next,
    Input(String),
}

fn simulator() -> Simulator<'static, Message> {
    let region = shadcn_iced::primitive::keys(
        column![
            text("Slide"),
            text_input("Inside", "").on_input(Message::Input).width(200),
        ],
        |key, _| (*key == Key::Named(Named::ArrowRight)).then_some(Message::Next),
    );

    Simulator::with_size(
        iced::Settings::default(),
        (400.0, 300.0),
        container(column![region, text("Outside")].spacing(40)).padding(20),
    )
}

/// Presses the right arrow, and counts the slides it moves.
fn arrow(mut ui: Simulator<'_, Message>) -> usize {
    let _ = ui.tap_key(Named::ArrowRight);

    ui.into_messages()
        .filter(|message| *message == Message::Next)
        .count()
}

#[test]
fn the_keys_work_while_the_focus_is_within() {
    let mut ui = simulator();

    // Focused with a click, then the click is forgotten with Tab: the focus
    // still is within.
    let _ = ui.click("Inside").expect("the input");
    let _ = ui.tap_key(Named::Tab);

    assert_eq!(arrow(ui), 1);
}

#[test]
fn the_keys_work_after_a_press_inside() {
    let mut ui = simulator();
    let _ = ui.click("Slide").expect("the slide");

    assert_eq!(arrow(ui), 1);
}

#[test]
fn the_keys_are_ignored_elsewhere() {
    let mut ui = simulator();
    let _ = ui.click("Outside").expect("outside");

    assert_eq!(arrow(ui), 0);

    // Tab moves the focus away from a slide pressed before.
    let mut ui = simulator();
    let _ = ui.click("Slide").expect("the slide");
    let _ = ui.tap_key(Named::Tab);

    assert_eq!(arrow(ui), 0);
}
