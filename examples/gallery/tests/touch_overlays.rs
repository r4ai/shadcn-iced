//! Floating layers with a finger and with the keyboard: menus, tooltips and
//! hover cards open with taps, long presses and keys.
use iced::keyboard::{Key, key::Named};
use iced::time::{Duration, Instant};
use iced::widget::{column, container, scrollable, text};
use iced::{Element, Event, Point, Theme, keyboard, mouse, touch, window};
use iced_test::simulator::Simulator;
use shadcn_iced::button::{Variant, button};
use shadcn_iced::menu::{item, menu};
use shadcn_iced::primitive::tab_order;

#[derive(Debug, Clone, PartialEq)]
enum Message {
    Pressed,
    Picked(&'static str),
}

fn simulator(view: impl Into<Element<'static, Message>>) -> Simulator<'static, Message> {
    shadcn_iced::animation::set_enabled(false);

    Simulator::with_size(
        iced::Settings::default(),
        (600.0, 500.0),
        container(tab_order(view)).padding(40),
    )
}

/// The center of the text `label`.
fn center(ui: &mut Simulator<'_, Message>, label: &'static str) -> Point {
    ui.find(label)
        .unwrap_or_else(|_| panic!("{label} is shown"))
        .visible_bounds()
        .expect("visible")
        .center()
}

fn finger(ui: &mut Simulator<'_, Message>, position: Point, event: touch::Event) {
    ui.point_at(position);
    let _ = ui.simulate([Event::Touch(event)]);
}

fn press(ui: &mut Simulator<'_, Message>, position: Point) {
    finger(
        ui,
        position,
        touch::Event::FingerPressed {
            id: touch::Finger(0),
            position,
        },
    );
}

fn lift(ui: &mut Simulator<'_, Message>, position: Point) {
    finger(
        ui,
        position,
        touch::Event::FingerLifted {
            id: touch::Finger(0),
            position,
        },
    );
}

fn tap(ui: &mut Simulator<'_, Message>, label: &'static str) {
    let position = center(ui, label);
    press(ui, position);
    lift(ui, position);
}

/// Lets `after` pass, for delays and long presses.
fn wait(ui: &mut Simulator<'_, Message>, after: Duration) {
    let _ = ui.simulate([Event::Window(window::Event::RedrawRequested(
        Instant::now() + after,
    ))]);
}

/// A finger resting on `label` long enough, then lifting.
fn long_press(ui: &mut Simulator<'_, Message>, label: &'static str) -> Point {
    let position = center(ui, label);
    press(ui, position);
    wait(ui, Duration::from_millis(600));
    position
}

fn key(ui: &mut Simulator<'_, Message>, named: Named) {
    let _ = ui.tap_key(named);
}

fn is_shown(ui: &mut Simulator<'_, Message>, label: &'static str) -> bool {
    ui.find(label).is_ok()
}

fn dropdown() -> Element<'static, Message> {
    shadcn_iced::dropdown_menu::dropdown_menu(
        button("Open").variant(Variant::Outline),
        menu()
            .item(item("Profile").on_select(Message::Picked("Profile")))
            .item(item("Settings").on_select(Message::Picked("Settings"))),
    )
    .into()
}

#[test]
fn a_tap_opens_a_dropdown_menu_once_the_finger_lifts() {
    let mut ui = simulator(dropdown());
    let position = center(&mut ui, "Open");

    press(&mut ui, position);
    assert!(!is_shown(&mut ui, "Profile"), "open on press");

    lift(&mut ui, position);
    assert!(is_shown(&mut ui, "Profile"), "closed after a tap");
}

#[test]
fn a_drag_from_a_trigger_does_not_open_it() {
    let mut ui = simulator(dropdown());
    let position = center(&mut ui, "Open");
    let away = position + iced::Vector::new(0.0, 40.0);

    press(&mut ui, position);
    finger(
        &mut ui,
        away,
        touch::Event::FingerMoved {
            id: touch::Finger(0),
            position: away,
        },
    );
    lift(&mut ui, position);

    assert!(!is_shown(&mut ui, "Profile"));
}

#[test]
fn enter_opens_a_dropdown_menu_with_its_first_item_highlighted() {
    let mut ui = simulator(dropdown());

    key(&mut ui, Named::Tab);
    key(&mut ui, Named::Enter);
    assert!(is_shown(&mut ui, "Profile"));

    // The highlight is moved once the menu is shown.
    wait(&mut ui, Duration::ZERO);
    key(&mut ui, Named::Enter);

    assert!(!is_shown(&mut ui, "Profile"), "closed on select");
    assert_eq!(
        ui.into_messages().collect::<Vec<_>>(),
        [Message::Picked("Profile")]
    );
}

#[test]
fn arrow_up_opens_a_dropdown_menu_with_its_last_item_highlighted() {
    let mut ui = simulator(dropdown());

    key(&mut ui, Named::Tab);
    key(&mut ui, Named::ArrowUp);
    wait(&mut ui, Duration::ZERO);
    key(&mut ui, Named::Enter);

    assert_eq!(
        ui.into_messages().collect::<Vec<_>>(),
        [Message::Picked("Settings")]
    );
}

#[test]
fn tab_closes_a_menu() {
    let mut ui = simulator(dropdown());

    key(&mut ui, Named::Tab);
    key(&mut ui, Named::Enter);
    assert!(is_shown(&mut ui, "Profile"));

    key(&mut ui, Named::Tab);
    assert!(!is_shown(&mut ui, "Profile"));
}

#[test]
fn tab_stays_within_an_open_popover() {
    let mut ui = simulator(column![
        shadcn_iced::popover::popover(
            button("Open").variant(Variant::Outline),
            column![
                button("First").on_press(Message::Picked("First")),
                button("Second").on_press(Message::Picked("Second")),
            ],
        ),
        button("Behind").on_press(Message::Picked("Behind")),
    ]);

    key(&mut ui, Named::Tab);
    key(&mut ui, Named::Enter);
    // The first control is focused once the popover is shown.
    wait(&mut ui, Duration::ZERO);

    // Around the popover, never to the button behind it: from `First` to
    // `Second`, and back to `First`.
    key(&mut ui, Named::Tab);
    key(&mut ui, Named::Space);
    key(&mut ui, Named::Tab);
    key(&mut ui, Named::Space);
    assert!(is_shown(&mut ui, "First"));

    key(&mut ui, Named::Escape);
    assert!(!is_shown(&mut ui, "First"));

    assert_eq!(
        ui.into_messages().collect::<Vec<_>>(),
        [Message::Picked("Second"), Message::Picked("First")]
    );
}

fn submenu() -> Element<'static, Message> {
    shadcn_iced::dropdown_menu::dropdown_menu(
        button("Open").variant(Variant::Outline),
        menu()
            .sub(
                "More",
                menu()
                    .item(item("Email").on_select(Message::Picked("Email")))
                    .item(item("Message").on_select(Message::Picked("Message"))),
            )
            .item(item("Profile").on_select(Message::Picked("Profile"))),
    )
    .into()
}

#[test]
fn arrow_right_opens_a_submenu_and_arrow_left_closes_it() {
    let mut ui = simulator(submenu());

    key(&mut ui, Named::Tab);
    key(&mut ui, Named::Enter);
    wait(&mut ui, Duration::ZERO);
    // Drawn, the highlighted trigger knows it is.
    let _ = ui.snapshot(&Theme::Light).expect("a snapshot");

    key(&mut ui, Named::ArrowRight);
    assert!(is_shown(&mut ui, "Email"), "the submenu opens");

    key(&mut ui, Named::ArrowLeft);
    assert!(!is_shown(&mut ui, "Email"), "the submenu closes");
    assert!(is_shown(&mut ui, "More"), "the menu stays");

    key(&mut ui, Named::ArrowRight);
    // Its first item is highlighted once shown.
    wait(&mut ui, Duration::ZERO);
    key(&mut ui, Named::Enter);

    assert!(!is_shown(&mut ui, "More"), "the whole menu closes");
    assert_eq!(
        ui.into_messages().collect::<Vec<_>>(),
        [Message::Picked("Email")]
    );
}

#[test]
fn a_tap_toggles_a_submenu() {
    let mut ui = simulator(submenu());

    tap(&mut ui, "Open");
    tap(&mut ui, "More");
    assert!(is_shown(&mut ui, "Email"), "opened with a tap");
    // Lingering redraws do not close it.
    wait(&mut ui, Duration::from_secs(1));
    assert!(is_shown(&mut ui, "Email"));

    tap(&mut ui, "More");
    wait(&mut ui, Duration::from_secs(1));
    assert!(!is_shown(&mut ui, "Email"), "closed with a second tap");
    assert!(is_shown(&mut ui, "Profile"));

    tap(&mut ui, "More");
    tap(&mut ui, "Message");
    // The menu closes with the redraw that follows the selection.
    wait(&mut ui, Duration::ZERO);
    assert!(!is_shown(&mut ui, "Profile"));
    assert_eq!(
        ui.into_messages().collect::<Vec<_>>(),
        [Message::Picked("Message")]
    );
}

fn tooltip() -> Element<'static, Message> {
    column![
        shadcn_iced::tooltip::tooltip(button("Save").on_press(Message::Pressed), "Saves the file"),
        button("Next"),
    ]
    .spacing(20)
    .into()
}

#[test]
fn a_tap_presses_a_tooltip_trigger_without_showing_the_tooltip() {
    let mut ui = simulator(tooltip());

    tap(&mut ui, "Save");
    wait(&mut ui, Duration::from_secs(1));

    assert!(!is_shown(&mut ui, "Saves the file"));
    assert_eq!(ui.into_messages().collect::<Vec<_>>(), [Message::Pressed]);
}

#[test]
fn a_long_press_shows_a_tooltip_without_pressing_its_trigger() {
    let mut ui = simulator(tooltip());

    let position = long_press(&mut ui, "Save");
    assert!(is_shown(&mut ui, "Saves the file"), "shown while held");

    lift(&mut ui, position);
    assert!(is_shown(&mut ui, "Saves the file"), "lingers");

    wait(&mut ui, Duration::from_secs(2));
    assert!(!is_shown(&mut ui, "Saves the file"), "hidden after a while");

    assert!(
        ui.into_messages().next().is_none(),
        "the trigger was pressed"
    );
}

#[test]
fn a_tooltip_opens_on_keyboard_focus() {
    let mut ui = simulator(tooltip());

    key(&mut ui, Named::Tab);
    assert!(is_shown(&mut ui, "Saves the file"), "shown on focus");

    key(&mut ui, Named::Escape);
    assert!(!is_shown(&mut ui, "Saves the file"), "hidden on Escape");

    key(&mut ui, Named::Tab);
    wait(&mut ui, Duration::ZERO);
    assert!(!is_shown(&mut ui, "Saves the file"), "hidden on blur");
}

#[test]
fn a_hover_card_opens_on_a_long_press_until_a_tap_outside() {
    let mut ui = simulator(
        column![
            shadcn_iced::hover_card::hover_card(
                button("@nextjs").variant(Variant::Link),
                text("The React Framework"),
            ),
            text("Elsewhere"),
        ]
        // Below the card.
        .spacing(200),
    );

    tap(&mut ui, "@nextjs");
    wait(&mut ui, Duration::from_secs(1));
    assert!(!is_shown(&mut ui, "The React Framework"), "a tap opens it");

    let position = long_press(&mut ui, "@nextjs");
    lift(&mut ui, position);
    wait(&mut ui, Duration::from_secs(3));
    assert!(is_shown(&mut ui, "The React Framework"), "stays open");

    tap(&mut ui, "Elsewhere");
    wait(&mut ui, Duration::from_secs(1));
    assert!(!is_shown(&mut ui, "The React Framework"));
}

fn context_menu() -> Element<'static, Message> {
    shadcn_iced::context_menu::context_menu(
        container(button("Area")).padding(40),
        menu()
            .item(item("Back").on_select(Message::Picked("Back")))
            .item(item("Forward").on_select(Message::Picked("Forward"))),
    )
    .into()
}

#[test]
fn a_long_press_opens_a_context_menu_at_the_finger() {
    let mut ui = simulator(context_menu());

    let position = long_press(&mut ui, "Area");
    assert!(is_shown(&mut ui, "Back"));

    let back = center(&mut ui, "Back");
    assert!(back.y > position.y, "below the finger");

    // Lifting the finger selects nothing.
    lift(&mut ui, position);
    assert!(is_shown(&mut ui, "Back"));

    tap(&mut ui, "Forward");
    assert!(!is_shown(&mut ui, "Back"));
    assert_eq!(
        ui.into_messages().collect::<Vec<_>>(),
        [Message::Picked("Forward")]
    );
}

#[test]
fn shift_f10_opens_a_context_menu() {
    let mut ui = simulator(context_menu());

    key(&mut ui, Named::Tab);
    let _ = ui.simulate([Event::Keyboard(keyboard::Event::KeyPressed {
        key: Key::Named(Named::F10),
        modified_key: Key::Named(Named::F10),
        physical_key: keyboard::key::Physical::Unidentified(
            keyboard::key::NativeCode::Unidentified,
        ),
        location: keyboard::Location::Standard,
        modifiers: keyboard::Modifiers::SHIFT,
        text: None,
        repeat: false,
    })]);
    assert!(is_shown(&mut ui, "Back"));

    wait(&mut ui, Duration::ZERO);
    key(&mut ui, Named::Enter);
    assert_eq!(
        ui.into_messages().collect::<Vec<_>>(),
        [Message::Picked("Back")]
    );
}

fn menubar() -> Element<'static, Message> {
    shadcn_iced::menubar::menubar()
        .menu(
            "File",
            menu().item(item("New Tab").on_select(Message::Picked("New Tab"))),
        )
        .menu(
            "Edit",
            menu().item(item("Undo").on_select(Message::Picked("Undo"))),
        )
        .into()
}

#[test]
fn tapping_another_menubar_trigger_switches_menus() {
    let mut ui = simulator(menubar());

    tap(&mut ui, "File");
    assert!(is_shown(&mut ui, "New Tab"));

    tap(&mut ui, "Edit");
    assert!(!is_shown(&mut ui, "New Tab"));
    assert!(is_shown(&mut ui, "Undo"));
}

#[test]
fn the_arrows_move_between_menubar_triggers() {
    let mut ui = simulator(menubar());

    key(&mut ui, Named::Tab);
    key(&mut ui, Named::ArrowRight);
    key(&mut ui, Named::ArrowDown);
    assert!(is_shown(&mut ui, "Undo"));

    wait(&mut ui, Duration::ZERO);
    // To the previous menu, with its first item highlighted.
    key(&mut ui, Named::ArrowLeft);
    assert!(is_shown(&mut ui, "New Tab"));
    wait(&mut ui, Duration::ZERO);
    key(&mut ui, Named::Enter);

    assert_eq!(
        ui.into_messages().collect::<Vec<_>>(),
        [Message::Picked("New Tab")]
    );
}

fn navigation() -> Element<'static, Message> {
    shadcn_iced::navigation_menu::navigation_menu()
        .menu(
            "Getting started",
            shadcn_iced::navigation_menu::link(
                "Introduction",
                "Re-usable components.",
                Message::Picked("Introduction"),
            ),
        )
        .menu(
            "Components",
            shadcn_iced::navigation_menu::link(
                "Alert",
                "Displays a callout.",
                Message::Picked("Alert"),
            ),
        )
        .into()
}

#[test]
fn a_tap_toggles_a_navigation_menu() {
    let mut ui = simulator(navigation());

    tap(&mut ui, "Getting started");
    wait(&mut ui, Duration::from_secs(1));
    assert!(is_shown(&mut ui, "Introduction"));

    tap(&mut ui, "Components");
    wait(&mut ui, Duration::from_secs(1));
    assert!(!is_shown(&mut ui, "Introduction"));
    assert!(is_shown(&mut ui, "Alert"));

    tap(&mut ui, "Components");
    wait(&mut ui, Duration::from_secs(1));
    assert!(!is_shown(&mut ui, "Alert"));
}

#[test]
fn the_keyboard_opens_a_navigation_menu() {
    let mut ui = simulator(navigation());

    key(&mut ui, Named::Tab);
    key(&mut ui, Named::ArrowRight);
    key(&mut ui, Named::Enter);
    wait(&mut ui, Duration::from_millis(100));
    assert!(is_shown(&mut ui, "Alert"));

    key(&mut ui, Named::Escape);
    wait(&mut ui, Duration::from_millis(100));
    assert!(!is_shown(&mut ui, "Alert"));

    // The focus is back on the trigger; its first link is focused.
    key(&mut ui, Named::ArrowDown);
    wait(&mut ui, Duration::from_millis(100));
    key(&mut ui, Named::Enter);

    assert_eq!(
        ui.into_messages().collect::<Vec<_>>(),
        [Message::Picked("Alert")]
    );
}

fn mouse(ui: &mut Simulator<'_, Message>, position: Point, events: &[mouse::Event]) {
    ui.point_at(position);
    let _ = ui.simulate(
        std::iter::once(Event::Mouse(mouse::Event::CursorMoved { position }))
            .chain(events.iter().cloned().map(Event::Mouse)),
    );
}

fn click(ui: &mut Simulator<'_, Message>, position: Point) {
    mouse(
        ui,
        position,
        &[
            mouse::Event::ButtonPressed(mouse::Button::Left),
            mouse::Event::ButtonReleased(mouse::Button::Left),
        ],
    );
}

fn label(prefix: &str, i: usize) -> &'static str {
    Box::leak(format!("{prefix} {i}").into_boxed_str())
}

#[test]
fn a_tap_in_a_scrolled_list_opens_the_menu_of_the_row_tapped() {
    let rows = (0..20).map(|i| {
        shadcn_iced::dropdown_menu::dropdown_menu(
            button(label("Row", i)).variant(Variant::Outline),
            menu().item(item(label("Edit", i)).on_select(Message::Picked(label("Edit", i)))),
        )
        .into()
    });
    let mut ui =
        simulator(scrollable(iced::widget::Column::with_children(rows).spacing(4)).height(300));

    // Scrolled by 300px.
    mouse(
        &mut ui,
        Point::new(100.0, 100.0),
        &[mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Pixels { x: 0.0, y: -300.0 },
        }],
    );

    tap(&mut ui, "Row 10");

    assert!(is_shown(&mut ui, "Edit 10"), "the menu of the row tapped");
    assert!(!is_shown(&mut ui, "Edit 7"));
}

#[test]
fn a_long_press_in_a_scrolled_list_opens_a_context_menu_at_the_finger() {
    let rows = (0..20).map(|i| {
        shadcn_iced::context_menu::context_menu(
            container(text(label("Row", i))).padding(8),
            menu().item(item(label("Copy", i)).on_select(Message::Picked(label("Copy", i)))),
        )
        .into()
    });
    let mut ui = simulator(scrollable(iced::widget::Column::with_children(rows)).height(300));

    mouse(
        &mut ui,
        Point::new(100.0, 100.0),
        &[mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Pixels { x: 0.0, y: -300.0 },
        }],
    );

    let position = long_press(&mut ui, "Row 10");
    assert!(is_shown(&mut ui, "Copy 10"), "the menu of the row pressed");

    let copy = center(&mut ui, "Copy 10");
    assert!(
        (copy.y - position.y) > 0.0 && (copy.y - position.y) < 40.0,
        "just below the finger: {copy:?} for {position:?}"
    );
}

#[test]
fn enter_on_a_submenu_trigger_opens_it_and_the_menu_stays() {
    let mut ui = simulator(submenu());

    key(&mut ui, Named::Tab);
    key(&mut ui, Named::Enter);
    wait(&mut ui, Duration::ZERO);
    // Drawn, the highlighted trigger knows it is.
    let _ = ui.snapshot(&Theme::Light).expect("a snapshot");

    key(&mut ui, Named::Enter);
    assert!(is_shown(&mut ui, "Email"), "the submenu opens");
    wait(&mut ui, Duration::ZERO);
    assert!(is_shown(&mut ui, "Profile"), "the menu stays");
    assert!(ui.into_messages().next().is_none());
}

#[test]
fn space_selects_in_a_menu_and_closes_it() {
    let mut ui = simulator(dropdown());

    key(&mut ui, Named::Tab);
    key(&mut ui, Named::Enter);
    wait(&mut ui, Duration::ZERO);
    key(&mut ui, Named::Space);
    wait(&mut ui, Duration::ZERO);

    assert!(!is_shown(&mut ui, "Profile"), "closed on select");
    assert_eq!(
        ui.into_messages().collect::<Vec<_>>(),
        [Message::Picked("Profile")]
    );
}

#[test]
fn space_selects_in_a_select_and_closes_it() {
    let mut ui = simulator(
        shadcn_iced::select::select(None::<&'static str>, Message::Picked)
            .placeholder("Pick a fruit")
            .option("Apple", "Apple")
            .option("Banana", "Banana"),
    );

    key(&mut ui, Named::Tab);
    key(&mut ui, Named::Enter);
    wait(&mut ui, Duration::ZERO);
    assert!(is_shown(&mut ui, "Banana"), "opened");

    key(&mut ui, Named::ArrowDown);
    key(&mut ui, Named::Space);
    wait(&mut ui, Duration::ZERO);

    assert!(!is_shown(&mut ui, "Banana"), "closed on select");
    assert_eq!(ui.into_messages().count(), 1);
}

#[test]
fn a_key_after_a_click_does_not_show_a_tooltip() {
    let mut ui = simulator(tooltip());
    let save = center(&mut ui, "Save");

    click(&mut ui, save);
    mouse(&mut ui, Point::new(500.0, 450.0), &[]);
    wait(&mut ui, Duration::from_secs(1));

    let _ = ui.simulate([Event::Keyboard(keyboard::Event::KeyPressed {
        key: Key::Character("s".into()),
        modified_key: Key::Character("s".into()),
        physical_key: keyboard::key::Physical::Unidentified(
            keyboard::key::NativeCode::Unidentified,
        ),
        location: keyboard::Location::Standard,
        modifiers: keyboard::Modifiers::CTRL,
        text: None,
        repeat: false,
    })]);
    wait(&mut ui, Duration::from_secs(2));

    assert!(!is_shown(&mut ui, "Saves the file"));
}

#[test]
fn pressing_a_tooltip_trigger_closes_the_tooltip() {
    let mut ui = simulator(tooltip());
    let save = center(&mut ui, "Save");

    mouse(&mut ui, save, &[]);
    wait(&mut ui, Duration::from_secs(1));
    assert!(is_shown(&mut ui, "Saves the file"), "shown on hover");

    mouse(
        &mut ui,
        save,
        &[mouse::Event::ButtonPressed(mouse::Button::Left)],
    );
    wait(&mut ui, Duration::from_secs(1));
    assert!(!is_shown(&mut ui, "Saves the file"), "hidden on press");
}

#[test]
fn enter_on_a_trigger_button_that_presses_does_not_open_it() {
    let mut ui = simulator(shadcn_iced::dropdown_menu::dropdown_menu(
        button("Open").on_press(Message::Pressed),
        menu().item(item("Profile").on_select(Message::Picked("Profile"))),
    ));

    key(&mut ui, Named::Tab);
    key(&mut ui, Named::Enter);

    assert!(!is_shown(&mut ui, "Profile"));
    assert_eq!(ui.into_messages().collect::<Vec<_>>(), [Message::Pressed]);
}

fn text_popover() -> Element<'static, Message> {
    column![
        shadcn_iced::popover::popover(
            button("Open").variant(Variant::Outline),
            text("Nothing to focus"),
        ),
        button("Behind").on_press(Message::Picked("Behind")),
    ]
    .spacing(200)
    .into()
}

#[test]
fn tab_closes_a_popover_with_nothing_to_focus() {
    let mut ui = simulator(text_popover());

    key(&mut ui, Named::Tab);
    key(&mut ui, Named::Enter);
    wait(&mut ui, Duration::ZERO);
    assert!(is_shown(&mut ui, "Nothing to focus"));

    // Closed, and on to the next control.
    key(&mut ui, Named::Tab);
    assert!(!is_shown(&mut ui, "Nothing to focus"));
    key(&mut ui, Named::Enter);

    assert_eq!(
        ui.into_messages().collect::<Vec<_>>(),
        [Message::Picked("Behind")]
    );
}

#[test]
fn the_arrows_do_not_open_a_popover() {
    let mut ui = simulator(text_popover());

    key(&mut ui, Named::Tab);
    key(&mut ui, Named::ArrowDown);
    key(&mut ui, Named::ArrowUp);

    assert!(!is_shown(&mut ui, "Nothing to focus"));
}
