//! Keyboard navigation of a listbox (`shadcn_iced::command`), across
//! rebuilds of the view.
use iced::keyboard::{self, Key, key::Named};
use iced::{Element, Event, Point, Rectangle, Size, Theme, mouse, window};
use iced_test::core::clipboard;
use iced_test::core::renderer::Headless;
use iced_test::core::widget::operation::{self, Operation, Outcome};
use iced_test::renderer::Renderer;
use iced_test::runtime::user_interface::{Cache, UserInterface};
use iced_test::selector::{Candidate, Selector};
use shadcn_iced::command::{command, item};

const SIZE: Size = Size::new(400.0, 500.0);

#[derive(Debug, Clone)]
enum Message {
    Query(String),
    Picked(String),
}

/// A command whose query lives in the application, like in a real one.
struct App {
    labels: Vec<String>,
    query: String,
    picked: Vec<String>,
    renderer: Renderer,
    cache: Option<Cache>,
    cursor: mouse::Cursor,
}

impl App {
    fn new(labels: &[&str]) -> Self {
        let renderer = iced::futures::executor::block_on(Renderer::new(
            shadcn_iced::font::sans(),
            iced::Pixels(16.0),
            Some("tiny-skia"),
        ))
        .expect("a headless renderer");

        Self {
            labels: labels.iter().map(ToString::to_string).collect(),
            query: String::new(),
            picked: Vec::new(),
            renderer,
            cache: Some(Cache::default()),
            cursor: mouse::Cursor::Unavailable,
        }
    }

    fn view(labels: &[String], query: &str) -> Element<'static, Message> {
        command(query, Message::Query)
            .items(
                labels
                    .iter()
                    .map(|label| item(label.clone()).on_select(Message::Picked(label.clone()))),
            )
            .into()
    }

    fn interface(&mut self) -> UserInterface<'static, Message, Theme, Renderer> {
        UserInterface::build(
            Self::view(&self.labels, &self.query),
            SIZE,
            self.cache.take().unwrap(),
            &mut self.renderer,
        )
    }

    fn dispatch(&mut self, event: Event) {
        let mut messages = Vec::new();
        let mut interface = self.interface();

        for event in [
            event,
            Event::Window(window::Event::RedrawRequested(iced::time::Instant::now())),
        ] {
            let _ = interface.update(
                &[event],
                self.cursor,
                &mut self.renderer,
                &mut clipboard::Null,
                &mut messages,
            );
        }

        self.cache = Some(interface.into_cache());

        for message in messages {
            match message {
                Message::Query(query) => self.query = query,
                Message::Picked(label) => self.picked.push(label),
            }
        }
    }

    fn click(&mut self, position: Point) {
        self.cursor = mouse::Cursor::Available(position);
        self.dispatch(Event::Mouse(mouse::Event::CursorMoved { position }));
        self.dispatch(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        )));
        self.dispatch(Event::Mouse(mouse::Event::ButtonReleased(
            mouse::Button::Left,
        )));
    }

    /// Focuses the search field.
    fn focus(&mut self) {
        self.click(Point::new(150.0, 20.0));
    }

    fn press(&mut self, key: Named) {
        self.dispatch(Event::Keyboard(keyboard::Event::KeyPressed {
            key: Key::Named(key),
            modified_key: Key::Named(key),
            physical_key: keyboard::key::Physical::Unidentified(
                keyboard::key::NativeCode::Unidentified,
            ),
            location: keyboard::Location::Standard,
            modifiers: keyboard::Modifiers::default(),
            text: None,
            repeat: false,
        }));
    }

    /// The visible bounds of the text `label`, if any.
    fn visible(&mut self, label: &str) -> Option<Rectangle> {
        let label = label.to_owned();
        let mut interface = self.interface();

        let mut operation = (move |candidate: Candidate<'_>| match &candidate {
            Candidate::Text { content, .. } if *content == label => candidate.visible_bounds(),
            _ => None,
        })
        .find();

        interface.operate(&self.renderer, &mut operation::black_box(&mut operation));
        self.cache = Some(interface.into_cache());

        match operation.finish() {
            Outcome::Some(bounds) => bounds.filter(|bounds| bounds.height > 0.0),
            _ => None,
        }
    }
}

#[test]
fn the_highlight_starts_over_when_the_query_changes() {
    shadcn_iced::animation::set_enabled(false);

    let mut app = App::new(&["Alpha", "Beta", "Gamma", "Delta"]);
    app.focus();
    app.press(Named::ArrowDown);
    app.press(Named::ArrowDown);

    // Every item still matches: they are as many, but the query changed.
    app.query = "a".to_owned();
    app.dispatch(Event::Window(window::Event::Focused));
    app.press(Named::Enter);

    // Fewer items, now.
    app.press(Named::ArrowDown);
    app.query = "ta".to_owned();
    app.dispatch(Event::Window(window::Event::Focused));
    app.press(Named::Enter);

    assert_eq!(app.picked, ["Alpha", "Beta"]);
}

#[test]
fn the_keys_need_the_focus_within_the_command() {
    shadcn_iced::animation::set_enabled(false);

    let mut app = App::new(&["Alpha", "Beta"]);
    app.press(Named::ArrowDown);
    app.press(Named::Enter);
    assert!(app.picked.is_empty(), "{:?}", app.picked);

    app.focus();
    app.press(Named::ArrowDown);
    app.press(Named::Enter);
    assert_eq!(app.picked, ["Beta"]);
}

#[test]
fn the_highlighted_item_scrolls_into_view() {
    shadcn_iced::animation::set_enabled(false);

    let labels: Vec<String> = (0..40).map(|i| format!("Item {i}")).collect();
    let labels: Vec<&str> = labels.iter().map(String::as_str).collect();
    let mut app = App::new(&labels);
    app.focus();

    assert!(app.visible("Item 0").is_some());
    assert!(app.visible("Item 20").is_none());

    for _ in 0..20 {
        app.press(Named::ArrowDown);
    }

    let item = app
        .visible("Item 20")
        .expect("the highlighted item is visible");
    assert!(item.height >= 20.0, "partly visible: {item:?}");
    assert!(app.visible("Item 0").is_none());

    // Back up, then around to the end.
    for _ in 0..20 {
        app.press(Named::ArrowUp);
    }
    assert!(app.visible("Item 0").is_some());
    assert!(app.visible("Item 20").is_none());

    app.press(Named::ArrowUp);
    assert!(app.visible("Item 39").is_some());

    app.press(Named::Enter);
    assert_eq!(app.picked, ["Item 39"]);
}
