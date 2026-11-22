//! Keyboard behaviour of nested floating layers.
use iced::widget::{column, container, text};
use iced::{Element, Event, Rectangle, keyboard, mouse};
use iced_test::selector::Candidate;
use iced_test::simulator::Simulator;
use shadcn_iced::primitive::Anchored;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq)]
enum Message {
    Outer(bool),
    Inner(bool),
    Selected(u8),
}

fn escape_messages(view: Element<'static, Message>, inner_anchor: &str) -> Vec<Message> {
    shadcn_iced::animation::set_enabled(false);

    let mut ui = Simulator::with_size(iced::Settings::default(), (600.0, 500.0), view);

    let _ = ui.click("Outer").expect("outer anchor");
    let _ = ui.click(inner_anchor).expect("outer content is open");
    let _ = ui.find("Inner content").expect("inner content is open");

    let _ = ui.tap_key(keyboard::key::Named::Escape);

    assert!(
        ui.find("Inner content").is_err(),
        "the inner content stays open"
    );
    let _ = ui
        .find("Outer content")
        .expect("the outer content was closed too");

    ui.into_messages().collect()
}

#[test]
fn escape_closes_only_the_topmost_layer() {
    let inner = Anchored::new(
        container(text("Inner")).width(80).height(30),
        container(text("Inner content")).width(100).height(40),
    )
    .on_open_change(Message::Inner);

    let outer = Anchored::new(
        container(text("Outer")).width(80).height(30),
        container(column![text("Outer content"), inner])
            .width(200)
            .height(120),
    )
    .on_open_change(Message::Outer);

    assert_eq!(
        escape_messages(container(outer).padding(20).into(), "Inner"),
        vec![
            Message::Outer(true),
            Message::Inner(true),
            Message::Inner(false)
        ]
    );
}

#[test]
fn escape_in_a_select_leaves_its_popover_open() {
    let select = shadcn_iced::select::select(None, Message::Selected)
        .placeholder("Pick")
        .option(1, "Inner content");

    let popover = shadcn_iced::popover::popover(
        container(text("Outer")).width(80).height(30),
        column![text("Outer content"), select],
    )
    .on_open_change(Message::Outer);

    assert_eq!(
        escape_messages(container(popover).padding(20).into(), "Pick"),
        vec![Message::Outer(true)]
    );
}

const WINDOW: iced::Size = iced::Size::new(400.0, 680.0);

/// A select of 30 options, 32px tall at `y`, whose list does not fit in the
/// window.
fn long_select(y: f32, selected: u8) -> Simulator<'static, Message> {
    shadcn_iced::animation::set_enabled(false);

    let select = (0..30).fold(
        shadcn_iced::select::select(Some(selected), Message::Selected),
        |select, value| select.option(value, format!("Option {value}.")),
    );

    Simulator::with_size(
        iced::Settings::default(),
        WINDOW,
        container(select.width(200)).padding(iced::Padding::ZERO.top(y).left(20.0)),
    )
}

/// The center of the visible part of every text `label`, the one of the
/// trigger first.
fn centers(ui: &mut Simulator<'_, Message>, label: &str) -> Vec<f32> {
    let label = label.to_owned();
    let centers = Arc::new(Mutex::new(Vec::new()));
    let found = Arc::clone(&centers);

    // `find` stops at the first match: none, to see every text.
    let _ = ui.find(move |candidate: Candidate<'_>| -> Option<Rectangle> {
        if let Candidate::Text { content, .. } = &candidate
            && *content == label
            && let Some(bounds) = candidate.visible_bounds()
        {
            found.lock().unwrap().push(bounds.center_y());
        }

        None
    });

    let centers = centers.lock().unwrap();
    centers.clone()
}

fn wheel(ui: &mut Simulator<'_, Message>, y: f32) {
    let _ = ui.simulate([Event::Mouse(mouse::Event::WheelScrolled {
        delta: mouse::ScrollDelta::Pixels { x: 0.0, y },
    })]);
}

#[test]
fn a_long_select_opens_with_its_option_over_the_trigger() {
    // Near the top, in the middle and near the bottom of the window.
    for (y, selected) in [
        (40.0, 0),
        (40.0, 15),
        (324.0, 0),
        (324.0, 1),
        (324.0, 15),
        (324.0, 29),
        (600.0, 15),
        (600.0, 29),
    ] {
        let mut ui = long_select(y, selected);
        let label = format!("Option {selected}.");
        let center = y + 16.0;

        assert_eq!(centers(&mut ui, &label), [center], "closed at {y}");
        let _ = ui.click(label.as_str()).expect("the trigger");

        // The value of the trigger, and the option over it.
        assert_eq!(
            centers(&mut ui, &label),
            [center, center],
            "option {selected} at {y}"
        );

        // A neighbor, to tell the list from the trigger.
        if selected > 0 {
            assert_eq!(
                centers(&mut ui, &format!("Option {}.", selected - 1)),
                [center - 28.0]
            );
        }
    }
}

#[test]
fn a_long_select_is_cut_at_the_edges_of_the_window() {
    let mut ui = long_select(324.0, 15);
    let _ = ui.click("Option 15.").expect("the trigger");

    // From 10 to 670, 330px on each side of the center of the option: the
    // text of the 12th ones is cut there, and the next ones are hidden.
    for (label, visible) in [
        ("Option 2.", false),
        ("Option 3.", true),
        ("Option 27.", true),
        ("Option 28.", false),
    ] {
        assert_eq!(!centers(&mut ui, label).is_empty(), visible, "{label}");
    }

    // The list scrolls, and keeps its position while it is open.
    ui.point_at((100.0, 400.0));
    wheel(&mut ui, -28.0);
    assert_eq!(centers(&mut ui, "Option 15."), [340.0, 312.0]);
    assert_eq!(centers(&mut ui, "Option 16."), [340.0]);

    // It is back in place the next time.
    let _ = ui.tap_key(keyboard::key::Named::Escape);
    assert_eq!(centers(&mut ui, "Option 15."), [340.0]);
    let _ = ui.click("Option 15.").expect("the trigger");
    assert_eq!(centers(&mut ui, "Option 15."), [340.0, 340.0]);
    assert_eq!(centers(&mut ui, "Option 16."), [368.0]);
}

#[test]
fn a_long_select_at_the_edge_of_the_window_opens_next_to_its_trigger() {
    // `sideOffset`, `p-1`, then the middle of the option.
    let mut ui = long_select(8.0, 0);
    let _ = ui.click("Option 0.").expect("the trigger");
    assert_eq!(centers(&mut ui, "Option 0."), [24.0, 40.0 + 4.0 + 18.0]);

    // Scrolled to the selected option, at the bottom of the list, 8px from
    // the edge of the window.
    let mut ui = long_select(8.0, 29);
    let _ = ui.click("Option 29.").expect("the trigger");
    assert_eq!(centers(&mut ui, "Option 29."), [24.0, 680.0 - 8.0 - 14.0]);

    // Above the trigger, when it is at the bottom.
    let mut ui = long_select(640.0, 29);
    let _ = ui.click("Option 29.").expect("the trigger");
    assert_eq!(centers(&mut ui, "Option 29."), [656.0, 636.0 - 14.0]);
}
