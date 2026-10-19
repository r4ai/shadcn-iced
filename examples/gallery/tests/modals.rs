//! Focus management and touch gestures of modal layers: dialogs, alert
//! dialogs and drawers.
//!
//! Unlike `Simulator`, the harness of these tests feeds the messages back
//! into the state of a small application and rebuilds its view, so that a
//! layer can be opened and closed.
use iced::advanced::widget::{Id, Operation};
use iced::keyboard::{self, key::Named};
use iced::widget::column;
use iced::{Element, Event, Font, Pixels, Point, Rectangle, Size, mouse, touch, window};
use iced_test::core::renderer::Headless;
use iced_test::runtime::user_interface::{Cache, UserInterface};
use shadcn_iced::button::button;
use shadcn_iced::primitive::{Focus, tab_order};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq)]
enum Message {
    Open,
    Close,
    Page,
    Action,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Kind {
    Dialog,
    AlertDialog,
    Drawer,
}

struct App {
    kind: Kind,
    open: bool,
}

impl App {
    fn view(&self) -> Element<'static, Message> {
        let page = column![
            button("Open").on_press(Message::Open),
            button("Page").on_press(Message::Page),
        ]
        .spacing(8)
        .padding(20);

        let modal: Element<'static, Message> = match self.kind {
            Kind::Dialog => shadcn_iced::dialog::dialog(page, self.open)
                .title("Edit profile")
                .content(button("First").on_press(Message::Action))
                .footer(button("Save").on_press(Message::Action))
                .on_close(Message::Close)
                .into(),
            Kind::AlertDialog => shadcn_iced::alert_dialog::alert_dialog(page, self.open)
                .title("Are you absolutely sure?")
                .cancel("Cancel", Message::Close)
                .action("Continue", Message::Action)
                .into(),
            Kind::Drawer => shadcn_iced::drawer::drawer(page, self.open)
                .title("Move Goal")
                .description("Set your daily activity goal.")
                .content(button("Submit").on_press(Message::Action))
                .on_close(Message::Close)
                .into(),
        };

        tab_order(modal).into()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Open => self.open = true,
            Message::Close => self.open = false,
            Message::Page | Message::Action => {}
        }
    }
}

/// An [`App`] in a window of 600x500, driven by raw events.
struct Harness {
    app: App,
    cache: Option<Cache>,
    renderer: iced::Renderer,
    cursor: mouse::Cursor,
    messages: Vec<Message>,
}

const SIZE: Size = Size::new(600.0, 500.0);

impl Harness {
    fn new(kind: Kind, open: bool) -> Self {
        shadcn_iced::animation::set_enabled(false);

        let renderer = iced_test::futures::futures::executor::block_on(iced::Renderer::new(
            Font::with_name("Fira Sans"),
            Pixels(16.0),
            None,
        ))
        .expect("a headless renderer");

        let mut harness = Self {
            app: App { kind, open },
            cache: Some(Cache::default()),
            renderer,
            cursor: mouse::Cursor::Unavailable,
            messages: Vec::new(),
        };

        harness.redraw();
        harness
    }

    /// Runs `f` on the interface, then applies the messages it produced.
    fn with_ui<T>(
        &mut self,
        f: impl FnOnce(&mut UserInterface<'_, Message, iced::Theme, iced::Renderer>, &mut Self) -> T,
    ) -> T {
        let view = self.app.view();
        let cache = self.cache.take().unwrap();
        let mut ui = UserInterface::build(view, SIZE, cache, &mut self.renderer);

        let output = f(&mut ui, self);

        self.cache = Some(ui.into_cache());

        output
    }

    /// Sends `events`, applies the messages and redraws (which lets layers
    /// notice that they opened or closed).
    fn send(&mut self, events: impl IntoIterator<Item = Event>) {
        let events: Vec<Event> = events.into_iter().collect();
        let mut messages = Vec::new();

        self.with_ui(|ui, harness| {
            let _ = ui.update(
                &events,
                harness.cursor,
                &mut harness.renderer,
                &mut iced::advanced::clipboard::Null,
                &mut messages,
            );
        });

        for message in &messages {
            self.app.update(*message);
        }

        self.messages.extend(messages);
        self.redraw();
    }

    fn redraw(&mut self) {
        let mut messages = Vec::new();

        self.with_ui(|ui, harness| {
            let _ = ui.update(
                &[Event::Window(
                    window::Event::RedrawRequested(Instant::now()),
                )],
                harness.cursor,
                &mut harness.renderer,
                &mut iced::advanced::clipboard::Null,
                &mut messages,
            );
        });

        self.messages.extend(messages);
    }

    fn take_messages(&mut self) -> Vec<Message> {
        std::mem::take(&mut self.messages)
    }

    /// The bounds of the text `label`.
    fn find(&mut self, label: &str) -> Option<Rectangle> {
        let mut find = FindText {
            label: label.to_owned(),
            bounds: None,
        };

        self.with_ui(|ui, harness| ui.operate(&harness.renderer, &mut find));

        find.bounds
    }

    /// The labels of the focused controls, and whether they show it.
    fn focused(&mut self, labels: &[&str]) -> Vec<(String, bool)> {
        let mut probe = Focused::default();
        self.with_ui(|ui, harness| ui.operate(&harness.renderer, &mut probe));

        let texts: Vec<_> = labels
            .iter()
            .filter_map(|label| Some((*label, self.find(label)?)))
            .collect();

        probe
            .controls
            .into_iter()
            .map(|(bounds, visible)| {
                let label = texts
                    .iter()
                    .find(|(_, text)| bounds.contains(text.center()))
                    .map_or("?", |(label, _)| label);

                (label.to_owned(), visible)
            })
            .collect()
    }

    fn click(&mut self, label: &str) {
        let center = self.find(label).expect(label).center();

        self.cursor = mouse::Cursor::Available(center);
        self.send([
            Event::Mouse(mouse::Event::CursorMoved { position: center }),
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
        ]);
    }

    fn key(&mut self, key: Named, modifiers: keyboard::Modifiers) {
        let key = keyboard::Key::Named(key);
        let physical =
            keyboard::key::Physical::Unidentified(keyboard::key::NativeCode::Unidentified);

        self.send([
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: key.clone(),
                modified_key: key.clone(),
                physical_key: physical,
                location: keyboard::Location::Standard,
                modifiers,
                repeat: false,
                text: None,
            }),
            Event::Keyboard(keyboard::Event::KeyReleased {
                key: key.clone(),
                modified_key: key,
                physical_key: physical,
                location: keyboard::Location::Standard,
                modifiers,
            }),
        ]);
    }

    fn tab(&mut self) {
        self.key(Named::Tab, keyboard::Modifiers::empty());
    }

    /// Moves a finger, pointing the cursor at it like iced_winit does.
    fn finger(&mut self, event: fn(touch::Finger, Point) -> touch::Event, position: Point) {
        self.cursor = mouse::Cursor::Available(position);
        self.send([Event::Touch(event(touch::Finger(0), position))]);
    }

    /// Drags a finger from `from` to `to` in a few steps, waiting `pause`
    /// before lifting it.
    fn drag(&mut self, from: Point, to: Point, pause: Duration) {
        self.finger(pressed, from);

        for step in 1..=4 {
            let t = step as f32 / 4.0;
            self.finger(
                moved,
                Point::new(from.x + (to.x - from.x) * t, from.y + (to.y - from.y) * t),
            );
        }

        std::thread::sleep(pause);
        self.finger(lifted, to);
    }
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

struct FindText {
    label: String,
    bounds: Option<Rectangle>,
}

impl Operation for FindText {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn text(&mut self, _id: Option<&Id>, bounds: Rectangle, text: &str) {
        if text == self.label && self.bounds.is_none() {
            self.bounds = Some(bounds);
        }
    }
}

#[derive(Default)]
struct Focused {
    controls: Vec<(Rectangle, bool)>,
}

impl Operation for Focused {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn custom(&mut self, _id: Option<&Id>, bounds: Rectangle, state: &mut dyn std::any::Any) {
        if let Some(focus) = state.downcast_mut::<Focus>()
            && focus.is_focused()
        {
            self.controls.push((bounds, focus.is_visible()));
        }
    }
}

const DIALOG: &[&str] = &["Open", "Page", "First", "Save"];

fn labels(focused: Vec<(String, bool)>) -> Vec<String> {
    focused.into_iter().map(|(label, _)| label).collect()
}

#[test]
fn opening_a_dialog_focuses_its_first_control() {
    let mut ui = Harness::new(Kind::Dialog, false);

    ui.click("Open");

    assert_eq!(ui.focused(DIALOG), vec![("First".to_owned(), false)]);
}

#[test]
fn tab_cycles_within_an_open_dialog() {
    let mut ui = Harness::new(Kind::Dialog, false);
    ui.click("Open");

    let mut seen = Vec::new();

    for _ in 0..7 {
        ui.tab();
        seen.push(labels(ui.focused(DIALOG)));
    }

    // First, Save, the close button (no label), and around again.
    assert_eq!(
        seen,
        [
            ["Save"],
            ["?"],
            ["First"],
            ["Save"],
            ["?"],
            ["First"],
            ["Save"]
        ]
        .map(|labels| labels.map(str::to_owned).to_vec())
        .to_vec()
    );

    for _ in 0..4 {
        ui.key(Named::Tab, keyboard::Modifiers::SHIFT);
        let focused = labels(ui.focused(DIALOG));

        assert!(
            !focused.contains(&"Open".to_owned()) && !focused.contains(&"Page".to_owned()),
            "the focus left the dialog: {focused:?}"
        );
    }
}

#[test]
fn focus_operations_skip_the_page_behind_a_dialog() {
    let mut ui = Harness::new(Kind::Dialog, false);
    ui.click("Open");

    // Like Tab, `focus_next` of the application stays in the dialog.
    for _ in 0..5 {
        let mut next = iced::advanced::widget::operation::focusable::focus_next::<()>();
        ui.with_ui(|ui, harness| ui.operate(&harness.renderer, &mut next));

        let focused = labels(ui.focused(DIALOG));
        assert!(
            !focused.contains(&"Open".to_owned()) && !focused.contains(&"Page".to_owned()),
            "the focus reached the page: {focused:?}"
        );
    }
}

#[test]
fn closing_a_dialog_returns_the_focus_to_its_trigger() {
    let mut ui = Harness::new(Kind::Dialog, false);
    ui.click("Open");
    ui.tab();

    ui.key(Named::Escape, keyboard::Modifiers::empty());

    assert!(!ui.app.open);
    // Closed with the keyboard: the focus shows.
    assert_eq!(ui.focused(DIALOG), vec![("Open".to_owned(), true)]);

    // Closed with the pointer: it does not.
    ui.click("Open");
    ui.click("Save");
    ui.cursor = mouse::Cursor::Available(Point::new(5.0, 5.0));
    ui.send([
        Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(5.0, 5.0),
        }),
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
    ]);

    assert!(!ui.app.open);
    assert_eq!(ui.focused(DIALOG), vec![("Open".to_owned(), false)]);
}

#[test]
fn opening_an_alert_dialog_focuses_its_cancel_button() {
    let mut ui = Harness::new(Kind::AlertDialog, false);

    ui.click("Open");

    // Below `sm`, the cancel button comes after the action.
    assert_eq!(
        labels(ui.focused(&["Open", "Cancel", "Continue"])),
        ["Cancel"]
    );

    ui.tab();
    assert_eq!(
        labels(ui.focused(&["Open", "Cancel", "Continue"])),
        ["Continue"]
    );
}

#[test]
fn swiping_a_drawer_down_closes_it() {
    let mut ui = Harness::new(Kind::Drawer, true);
    let title = ui.find("Move Goal").expect("the drawer is open").center();

    ui.drag(
        title,
        Point::new(title.x, title.y + 150.0),
        Duration::from_millis(50),
    );

    assert_eq!(ui.take_messages(), [Message::Close]);
}

#[test]
fn a_short_swipe_brings_a_drawer_back() {
    let mut ui = Harness::new(Kind::Drawer, true);
    let title = ui.find("Move Goal").expect("the drawer is open").center();

    // Past the slop, but neither far nor fast: it comes back.
    ui.drag(
        title,
        Point::new(title.x, title.y + 20.0),
        Duration::from_millis(250),
    );
    assert_eq!(ui.take_messages(), []);

    // Up, against the edge: a rubber band.
    ui.drag(
        title,
        Point::new(title.x, title.y - 150.0),
        Duration::from_millis(50),
    );
    assert_eq!(ui.take_messages(), []);

    // A tap of its button still works.
    let submit = ui.find("Submit").unwrap().center();
    ui.finger(pressed, submit);
    ui.finger(lifted, submit);
    assert_eq!(ui.take_messages(), [Message::Action]);
}

#[test]
fn a_swipe_from_a_button_does_not_press_it() {
    let mut ui = Harness::new(Kind::Drawer, true);
    let submit = ui.find("Submit").unwrap().center();

    ui.drag(
        submit,
        Point::new(submit.x, submit.y + 20.0),
        Duration::from_millis(250),
    );

    assert_eq!(ui.take_messages(), []);
}

#[test]
fn a_drag_on_the_backdrop_does_not_dismiss() {
    for kind in [Kind::Dialog, Kind::Drawer] {
        let mut ui = Harness::new(kind, true);

        ui.drag(
            Point::new(10.0, 10.0),
            Point::new(200.0, 60.0),
            Duration::ZERO,
        );
        assert_eq!(ui.take_messages(), [], "{kind:?}");

        ui.finger(pressed, Point::new(10.0, 10.0));
        ui.finger(lifted, Point::new(12.0, 10.0));
        assert_eq!(ui.take_messages(), [Message::Close], "{kind:?}");
    }
}
