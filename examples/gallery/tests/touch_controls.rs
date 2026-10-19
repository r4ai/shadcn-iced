//! Controls used with touch and with the keyboard: taps press, drags and
//! other fingers do not, a lifted finger hovers nothing, and Tab and the
//! arrow keys reach every control.
use std::cell::Cell;

use iced::keyboard::{self, Key, Modifiers, key::Named};
use iced::widget::{button::Status, column, container, text};
use iced::{Event, Point, Theme, touch};
use iced_test::simulator::{self, Simulator};
use shadcn_iced::primitive::{Pressable, tab_order};

#[derive(Debug, Clone, PartialEq)]
enum Message {
    Press,
    Page(usize),
    Go,
    Open(Option<usize>),
    Toggle(bool),
    Edit,
}

thread_local! {
    /// The status the pressable was last drawn with.
    static STATUS: Cell<Option<Status>> = const { Cell::new(None) };
}

fn pressable() -> Simulator<'static, Message> {
    shadcn_iced::animation::set_enabled(false);

    let pressable = Pressable::new(text("Press me"))
        .on_press(Message::Press)
        .padding(10)
        .style(|_theme, status| {
            STATUS.with(|cell| cell.set(Some(status)));
            iced::widget::button::Style::default()
        });

    Simulator::with_size(
        iced::Settings::default(),
        (400.0, 300.0),
        container(pressable).padding(20),
    )
}

/// The center of the pressable.
fn center(ui: &mut Simulator<'_, Message>) -> Point {
    ui.find("Press me")
        .expect("the pressable")
        .bounds()
        .center()
}

fn finger(id: u64) -> touch::Finger {
    touch::Finger(id)
}

fn pressed(id: u64, position: Point) -> Event {
    Event::Touch(touch::Event::FingerPressed {
        id: finger(id),
        position,
    })
}

fn moved(id: u64, position: Point) -> Event {
    Event::Touch(touch::Event::FingerMoved {
        id: finger(id),
        position,
    })
}

fn lifted(id: u64, position: Point) -> Event {
    Event::Touch(touch::Event::FingerLifted {
        id: finger(id),
        position,
    })
}

fn presses(ui: Simulator<'_, Message>) -> usize {
    ui.into_messages()
        .filter(|message| *message == Message::Press)
        .count()
}

#[test]
fn a_tap_presses() {
    let mut ui = pressable();
    let at = center(&mut ui);

    ui.point_at(at);
    let _ = ui.simulate([pressed(0, at)]);
    ui.point_at(at);
    let _ = ui.simulate([lifted(0, at)]);

    assert_eq!(presses(ui), 1);
}

#[test]
fn a_drag_past_the_slop_does_not_press() {
    let mut ui = pressable();
    let at = center(&mut ui);
    let away = Point::new(at.x, at.y + 30.0);

    ui.point_at(at);
    let _ = ui.simulate([pressed(0, at)]);
    ui.point_at(away);
    let _ = ui.simulate([moved(0, away)]);
    // Back over the pressable: the press was cancelled for good.
    ui.point_at(at);
    let _ = ui.simulate([moved(0, at)]);
    let _ = ui.simulate([lifted(0, at)]);

    assert_eq!(presses(ui), 0);
}

#[test]
fn a_second_finger_is_ignored() {
    let mut ui = pressable();
    let at = center(&mut ui);
    let near = Point::new(at.x + 2.0, at.y);

    ui.point_at(at);
    let _ = ui.simulate([pressed(0, at)]);
    ui.point_at(near);
    let _ = ui.simulate([pressed(1, near)]);
    let _ = ui.simulate([lifted(1, near)]);
    ui.point_at(at);
    let _ = ui.simulate([lifted(0, at)]);

    assert_eq!(presses(ui), 1);
}

#[test]
fn a_lifted_finger_hovers_nothing() {
    let mut ui = pressable();
    let at = center(&mut ui);

    ui.point_at(at);
    let _ = ui.simulate([pressed(0, at)]);
    let _ = ui.snapshot(&Theme::Light).expect("a snapshot");
    assert_eq!(STATUS.with(Cell::get), Some(Status::Pressed));

    // iced keeps the cursor where the finger lifted.
    let _ = ui.simulate([lifted(0, at)]);
    let _ = ui.snapshot(&Theme::Light).expect("a snapshot");
    assert_eq!(STATUS.with(Cell::get), Some(Status::Active));
}

#[test]
fn the_mouse_still_hovers() {
    let mut ui = pressable();
    let at = center(&mut ui);

    ui.point_at(at);
    let _ = ui.simulate([Event::Mouse(iced::mouse::Event::CursorMoved {
        position: at,
    })]);
    let _ = ui.snapshot(&Theme::Light).expect("a snapshot");

    assert_eq!(STATUS.with(Cell::get), Some(Status::Hovered));
}

/// Presses Tab `tabs` times, then Enter, and returns the messages.
fn tab_then_enter(mut ui: Simulator<'_, Message>, tabs: usize) -> Vec<Message> {
    for _ in 0..tabs {
        let _ = ui.tap_key(Named::Tab);
    }

    let _ = ui.tap_key(Named::Enter);

    ui.into_messages().collect()
}

fn page(content: iced::Element<'static, Message>) -> Simulator<'static, Message> {
    shadcn_iced::animation::set_enabled(false);

    Simulator::with_size(
        iced::Settings::default(),
        (600.0, 400.0),
        tab_order(container(content).padding(20)),
    )
}

#[test]
fn tab_reaches_the_pagination_links() {
    let pagination = || shadcn_iced::pagination::pagination(2, 5, Message::Page).into();

    // "Previous", then the pages.
    assert_eq!(tab_then_enter(page(pagination()), 1), [Message::Page(1)]);
    assert_eq!(tab_then_enter(page(pagination()), 2), [Message::Page(1)]);
    assert_eq!(tab_then_enter(page(pagination()), 4), [Message::Page(3)]);
}

#[test]
fn tab_reaches_the_breadcrumb_links() {
    let breadcrumb = shadcn_iced::breadcrumb::breadcrumb()
        .link("Home", Message::Go)
        .page("Settings");

    assert_eq!(tab_then_enter(page(breadcrumb.into()), 1), [Message::Go]);
}

fn accordion() -> Simulator<'static, Message> {
    page(
        shadcn_iced::accordion::accordion()
            .single(None, Message::Open)
            .item("First", text("One"))
            .item("Second", text("Two"))
            .item("Third", text("Three"))
            .into(),
    )
}

/// Focuses the first trigger, presses `keys`, then Enter.
fn accordion_keys(keys: &[Named]) -> Vec<Message> {
    let mut ui = accordion();
    let _ = ui.tap_key(Named::Tab);

    for key in keys {
        let _ = ui.tap_key(*key);
    }

    let _ = ui.tap_key(Named::Enter);

    ui.into_messages().collect()
}

#[test]
fn the_arrow_keys_move_between_the_accordion_triggers() {
    assert_eq!(accordion_keys(&[]), [Message::Open(Some(0))]);
    assert_eq!(
        accordion_keys(&[Named::ArrowDown]),
        [Message::Open(Some(1))]
    );
    // They wrap around.
    assert_eq!(accordion_keys(&[Named::ArrowUp]), [Message::Open(Some(2))]);
    assert_eq!(
        accordion_keys(&[Named::ArrowDown, Named::ArrowDown, Named::ArrowDown]),
        [Message::Open(Some(0))]
    );
    assert_eq!(accordion_keys(&[Named::End]), [Message::Open(Some(2))]);
    assert_eq!(
        accordion_keys(&[Named::End, Named::Home]),
        [Message::Open(Some(0))]
    );
}

fn ctrl_b() -> Event {
    let key = Key::Character("b".into());

    Event::Keyboard(keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: keyboard::key::Physical::Unidentified(
            keyboard::key::NativeCode::Unidentified,
        ),
        location: keyboard::Location::Standard,
        modifiers: Modifiers::COMMAND,
        repeat: false,
        text: None,
    })
}

#[test]
fn ctrl_b_toggles_the_sidebar() {
    use shadcn_iced::sidebar::{group, item, layout, sidebar};

    let view = |collapsed: bool| {
        let nav = sidebar()
            .collapsed(collapsed)
            .on_toggle(Message::Toggle)
            .group(group("Application").item(item("Home").on_press(Message::Go)));

        page(layout(nav, text("Content")))
    };

    let mut ui = view(false);
    let _ = ui.simulate([ctrl_b()]);
    assert_eq!(
        ui.into_messages().collect::<Vec<_>>(),
        [Message::Toggle(true)]
    );

    let mut ui = view(true);
    let _ = ui.simulate([ctrl_b()]);
    assert_eq!(
        ui.into_messages().collect::<Vec<_>>(),
        [Message::Toggle(false)]
    );

    // Without the modifier, B is only a letter.
    let mut ui = view(false);
    let _ = ui.simulate(simulator::tap_key(Key::Character("b".into()), None));
    assert!(ui.into_messages().next().is_none());
}

#[test]
fn tab_leaves_a_textarea() {
    let content: &'static _ = Box::leak(Box::new(shadcn_iced::textarea::Content::new()));

    let mut ui = page(
        column![
            shadcn_iced::textarea::textarea(content)
                .placeholder("Message")
                .on_action(|_| Message::Edit),
            shadcn_iced::button::button("Send").on_press(Message::Go),
        ]
        .spacing(20)
        .into(),
    );

    // Clicks the textarea, above the button.
    let send = ui.find("Send").expect("the button").bounds();
    ui.point_at(Point::new(send.x + 4.0, send.y - 40.0));
    let _ = ui.simulate(simulator::click());
    let _ = ui.simulate(simulator::tap_key(Named::Tab, Some("\t".into())));
    let _ = ui.tap_key(Named::Enter);

    let messages: Vec<_> = ui.into_messages().collect();

    // The click focused the textarea.
    assert_eq!(messages.first(), Some(&Message::Edit));

    let messages: Vec<_> = messages
        .into_iter()
        .filter(|message| *message != Message::Edit)
        .collect();

    // The Tab moved the focus to the button, which Enter pressed.
    assert_eq!(messages, [Message::Go]);
}
