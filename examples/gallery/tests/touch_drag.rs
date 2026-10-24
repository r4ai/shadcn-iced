//! Dragging with fingers: dividers, scrollbars, carousels and scroll areas.
use iced::time::Instant;
use iced::widget::{Column, container, text};
use iced::{Element, Event, Point, touch, window};
use iced_test::simulator::Simulator;
use shadcn_iced::carousel::{self, carousel};
use shadcn_iced::resizable::resizable;
use shadcn_iced::scroll_area::scroll_area;

#[derive(Debug, Clone, PartialEq)]
enum Message {
    Resized(f32),
    Scrolled(f32),
    Slide(usize),
}

fn press(id: u64, x: f32, y: f32) -> Event {
    Event::Touch(touch::Event::FingerPressed {
        id: touch::Finger(id),
        position: Point::new(x, y),
    })
}

fn moved(id: u64, x: f32, y: f32) -> Event {
    Event::Touch(touch::Event::FingerMoved {
        id: touch::Finger(id),
        position: Point::new(x, y),
    })
}

fn lifted(id: u64, x: f32, y: f32) -> Event {
    Event::Touch(touch::Event::FingerLifted {
        id: touch::Finger(id),
        position: Point::new(x, y),
    })
}

fn lost(id: u64, x: f32, y: f32) -> Event {
    Event::Touch(touch::Event::FingerLost {
        id: touch::Finger(id),
        position: Point::new(x, y),
    })
}

/// Sends `event` with the cursor at its position, like the shell does.
fn send(ui: &mut Simulator<'_, Message>, event: Event, at: (f32, f32)) {
    ui.point_at(at);
    let _ = ui.simulate([event]);
}

fn simulator(view: Element<'static, Message>) -> Simulator<'static, Message> {
    shadcn_iced::animation::set_enabled(false);

    Simulator::with_size(iced::Settings::default(), (400.0, 300.0), view)
}

fn panels() -> Element<'static, Message> {
    resizable(text("One"), text("Two"))
        .on_resize(Message::Resized)
        .into()
}

fn resized(ui: Simulator<'_, Message>) -> Vec<f32> {
    ui.into_messages()
        .filter_map(|message| match message {
            Message::Resized(ratio) => Some(ratio),
            _ => None,
        })
        .collect()
}

#[test]
fn a_finger_grabs_the_divider_from_a_little_farther() {
    // The divider is at x = 200: 8px wide for the mouse, 12px more around
    // for a finger.
    let mut ui = simulator(panels());

    send(&mut ui, press(0, 212.0, 100.0), (212.0, 100.0));
    send(&mut ui, moved(0, 252.0, 100.0), (252.0, 100.0));
    send(&mut ui, lifted(0, 252.0, 100.0), (252.0, 100.0));

    let ratios = resized(ui);

    assert!(
        ratios.last().is_some_and(|ratio| *ratio > 0.6),
        "{ratios:?}"
    );
}

#[test]
fn a_finger_too_far_does_not_grab_the_divider() {
    let mut ui = simulator(panels());

    send(&mut ui, press(0, 230.0, 100.0), (230.0, 100.0));
    send(&mut ui, moved(0, 290.0, 100.0), (290.0, 100.0));

    assert!(resized(ui).is_empty());
}

#[test]
fn the_divider_ignores_a_second_finger() {
    let mut ui = simulator(panels());

    send(&mut ui, press(0, 200.0, 100.0), (200.0, 100.0));
    send(&mut ui, press(1, 100.0, 100.0), (100.0, 100.0));
    send(&mut ui, moved(1, 60.0, 100.0), (60.0, 100.0));
    send(&mut ui, lifted(1, 60.0, 100.0), (60.0, 100.0));

    assert!(resized(ui).is_empty());
}

#[test]
fn the_divider_lets_go_when_the_finger_is_lost() {
    let mut ui = simulator(panels());

    send(&mut ui, press(0, 200.0, 100.0), (200.0, 100.0));
    send(&mut ui, moved(0, 240.0, 100.0), (240.0, 100.0));
    send(&mut ui, lost(0, 240.0, 100.0), (240.0, 100.0));
    send(&mut ui, moved(0, 300.0, 100.0), (300.0, 100.0));

    let ratios = resized(ui);

    assert_eq!(ratios.len(), 1, "{ratios:?}");
}

fn area() -> Element<'static, Message> {
    let lines = (0..60).map(|i| text(format!("Line {i}")).into());

    container(
        scroll_area(Column::with_children(lines))
            .on_scroll(|viewport| Message::Scrolled(viewport.absolute_offset().y))
            .width(300)
            .height(200),
    )
    .into()
}

fn scrolled(ui: Simulator<'_, Message>) -> f32 {
    ui.into_messages()
        .filter_map(|message| match message {
            Message::Scrolled(y) => Some(y),
            _ => None,
        })
        .last()
        .unwrap_or_default()
}

fn frame(ui: &mut Simulator<'_, Message>) {
    let _ = ui.simulate([Event::Window(
        window::Event::RedrawRequested(Instant::now()),
    )]);
}

#[test]
fn a_finger_drags_the_scrollbar() {
    let mut ui = simulator(area());

    // The thumb is at the top of the track, along the right edge.
    send(&mut ui, press(0, 295.0, 12.0), (295.0, 12.0));
    send(&mut ui, moved(0, 295.0, 80.0), (295.0, 80.0));
    frame(&mut ui);
    send(&mut ui, lifted(0, 295.0, 80.0), (295.0, 80.0));

    assert!(scrolled(ui) > 100.0);
}

#[test]
fn the_scrollbar_ignores_a_second_finger() {
    let mut ui = simulator(area());

    send(&mut ui, press(0, 295.0, 12.0), (295.0, 12.0));
    send(&mut ui, press(1, 295.0, 100.0), (295.0, 100.0));
    send(&mut ui, moved(1, 295.0, 180.0), (295.0, 180.0));
    frame(&mut ui);

    assert!(scrolled(ui) < 1.0);
}

#[test]
fn the_arrow_keys_scroll_after_a_click() {
    let mut ui = simulator(area());

    let _ = ui.click("Line 1").expect("a line");
    let _ = ui.tap_key(iced::keyboard::key::Named::ArrowDown);
    frame(&mut ui);

    assert!(scrolled(ui) > 0.0);
}

#[test]
fn the_arrow_keys_do_nothing_before_a_press() {
    let mut ui = simulator(area());

    let _ = ui.tap_key(iced::keyboard::key::Named::ArrowDown);
    frame(&mut ui);

    assert_eq!(scrolled(ui), 0.0);
}

fn slides(vertical: bool) -> Element<'static, Message> {
    let carousel = carousel(0, Message::Slide)
        .item(text("One"))
        .item(text("Two"))
        .item(text("Three"));

    if vertical {
        carousel
            .orientation(carousel::Orientation::Vertical)
            .height(100)
            .into()
    } else {
        carousel.into()
    }
}

fn slid(ui: Simulator<'_, Message>) -> Vec<usize> {
    ui.into_messages()
        .filter_map(|message| match message {
            Message::Slide(index) => Some(index),
            _ => None,
        })
        .collect()
}

#[test]
fn a_swipe_along_the_carousel_slides() {
    let mut ui = simulator(container(slides(false)).padding(20).into());

    send(&mut ui, press(0, 300.0, 40.0), (300.0, 40.0));
    send(&mut ui, moved(0, 200.0, 42.0), (200.0, 42.0));
    send(&mut ui, moved(0, 100.0, 42.0), (100.0, 42.0));
    send(&mut ui, lifted(0, 100.0, 42.0), (100.0, 42.0));

    assert_eq!(slid(ui), vec![1]);
}

#[test]
fn a_vertical_swipe_does_not_move_a_horizontal_carousel() {
    let mut ui = simulator(container(slides(false)).padding(20).into());

    send(&mut ui, press(0, 200.0, 20.0), (200.0, 20.0));
    send(&mut ui, moved(0, 190.0, 80.0), (190.0, 80.0));
    send(&mut ui, moved(0, 100.0, 90.0), (100.0, 90.0));
    send(&mut ui, lifted(0, 100.0, 90.0), (100.0, 90.0));

    assert!(slid(ui).is_empty());
}

#[test]
fn the_carousel_ignores_a_second_finger() {
    let mut ui = simulator(container(slides(false)).padding(20).into());

    send(&mut ui, press(0, 300.0, 40.0), (300.0, 40.0));
    send(&mut ui, press(1, 100.0, 40.0), (100.0, 40.0));
    send(&mut ui, moved(1, 20.0, 40.0), (20.0, 40.0));
    send(&mut ui, lifted(1, 20.0, 40.0), (20.0, 40.0));
    send(&mut ui, lifted(0, 300.0, 40.0), (300.0, 40.0));

    assert!(slid(ui).is_empty());
}
