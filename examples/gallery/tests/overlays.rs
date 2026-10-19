//! Keyboard behaviour of nested floating layers.
use iced::widget::{column, container, text};
use iced::{Element, keyboard};
use iced_test::simulator::Simulator;
use shadcn_iced::primitive::Anchored;

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
