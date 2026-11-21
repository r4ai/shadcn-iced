//! Typing with an input method editor (IME), like the Japanese ones: the
//! text inputs must ask the window for the IME where their caret is, show
//! the text being composed and take the text committed.
use iced::advanced::input_method::{self, InputMethod};
use iced::advanced::widget;
use iced::keyboard::{self, Key, key::Named};
use iced::widget::{column, container, text};
use iced::{Element, Event, Point, Rectangle, Size, Theme, mouse, window};
use iced_test::runtime::{UserInterface, user_interface};
use iced_test::selector::Selector;

use iced::advanced::renderer::Headless as _;

/// A headless user interface that keeps the [`InputMethod`] its widgets
/// ask for, rebuilt from `view` after every batch of events.
struct Ui<S, Message: 'static> {
    state: S,
    view: fn(&S) -> Element<'_, Message>,
    apply: fn(&mut S, Message),
    cache: Option<user_interface::Cache>,
    renderer: iced::Renderer,
    cursor: mouse::Cursor,
    input_method: InputMethod,
    messages: Vec<Message>,
}

const SIZE: Size = Size::new(800.0, 600.0);

impl<S, Message: Clone + 'static> Ui<S, Message> {
    fn new(state: S, view: fn(&S) -> Element<'_, Message>, apply: fn(&mut S, Message)) -> Self {
        shadcn_iced::animation::set_enabled(false);

        let renderer = iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::with_name("Fira Sans"),
            iced::Pixels(16.0),
            None,
        ))
        .expect("a headless renderer");

        let mut ui = Self {
            state,
            view,
            apply,
            cache: Some(user_interface::Cache::default()),
            renderer,
            cursor: mouse::Cursor::Unavailable,
            input_method: InputMethod::Disabled,
            messages: Vec::new(),
        };

        ui.redraw();
        ui
    }

    /// Runs `f` with the user interface built from the current state.
    fn with<T>(
        &mut self,
        f: impl FnOnce(&mut UserInterface<'_, Message, Theme, iced::Renderer>, &mut iced::Renderer) -> T,
    ) -> T {
        let element = (self.view)(&self.state);
        let mut ui = UserInterface::build(
            element,
            SIZE,
            self.cache.take().unwrap_or_default(),
            &mut self.renderer,
        );

        let output = f(&mut ui, &mut self.renderer);
        self.cache = Some(ui.into_cache());
        output
    }

    fn simulate(&mut self, events: &[Event]) {
        let cursor = self.cursor;
        let mut messages = Vec::new();

        let state = self.with(|ui, renderer| {
            let (state, _) = ui.update(
                events,
                cursor,
                renderer,
                &mut iced::advanced::clipboard::Null,
                &mut messages,
            );

            state
        });

        if let user_interface::State::Updated { input_method, .. } = state {
            self.input_method = input_method;
        }

        for message in messages {
            self.messages.push(message.clone());
            (self.apply)(&mut self.state, message);
        }
    }

    /// Simulates a frame: the window asks the widgets to redraw, and they
    /// request the input method then.
    fn redraw(&mut self) {
        self.simulate(&[Event::Window(window::Event::RedrawRequested(
            iced::time::Instant::now(),
        ))]);
    }

    fn find<Sel>(&mut self, selector: Sel) -> Option<Sel::Output>
    where
        Sel: Selector + Send,
        Sel::Output: Clone + Send,
    {
        use iced::advanced::widget::Operation as _;

        let mut operation = selector.find();

        self.with(|ui, renderer| {
            ui.operate(renderer, &mut widget::operation::black_box(&mut operation));
        });

        match operation.finish() {
            widget::operation::Outcome::Some(output) => output,
            _ => None,
        }
    }

    fn bounds(&mut self, label: &'static str) -> Rectangle {
        self.find(label)
            .and_then(|target| target.visible_bounds())
            .unwrap_or_else(|| panic!("{label:?} is visible"))
    }

    fn click_at(&mut self, position: Point) {
        self.cursor = mouse::Cursor::Available(position);
        self.simulate(&[
            Event::Mouse(mouse::Event::CursorMoved { position }),
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
        ]);
        self.redraw();
    }

    fn click(&mut self, label: &'static str) -> Rectangle {
        let bounds = self.bounds(label);
        self.click_at(bounds.center());
        bounds
    }

    fn ime(&mut self, event: input_method::Event) {
        self.simulate(&[Event::InputMethod(event)]);
        self.redraw();
    }

    /// Composes `preedit`, as an IME does while the reading is typed and
    /// converted, then commits `text`, as with <kbd>Enter</kbd>.
    fn compose(&mut self, preedit: &str, text: &str) {
        self.ime(input_method::Event::Opened);
        self.ime(input_method::Event::Preedit(
            preedit.to_owned(),
            Some(preedit.len()..preedit.len()),
        ));
        self.ime(input_method::Event::Preedit(String::new(), None));
        self.ime(input_method::Event::Commit(text.to_owned()));
    }

    /// Presses and releases `key` (with the given text).
    fn key(&mut self, key: Key, text: Option<&str>) {
        let physical_key =
            keyboard::key::Physical::Unidentified(keyboard::key::NativeCode::Unidentified);

        self.simulate(&[
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: key.clone(),
                modified_key: key.clone(),
                physical_key,
                location: keyboard::Location::Standard,
                modifiers: keyboard::Modifiers::empty(),
                text: text.map(Into::into),
                repeat: false,
            }),
            Event::Keyboard(keyboard::Event::KeyReleased {
                key: key.clone(),
                modified_key: key,
                physical_key,
                location: keyboard::Location::Standard,
                modifiers: keyboard::Modifiers::empty(),
            }),
        ]);
        self.redraw();
    }

    /// The caret area the IME is asked for, if it is enabled.
    fn caret(&self) -> Option<Rectangle> {
        match &self.input_method {
            InputMethod::Enabled { cursor, .. } => Some(*cursor),
            InputMethod::Disabled => None,
        }
    }

    fn preedit(&self) -> Option<(String, Option<f32>)> {
        match &self.input_method {
            InputMethod::Enabled {
                preedit: Some(preedit),
                ..
            } => Some((
                preedit.content.clone(),
                preedit.text_size.map(|size| size.0),
            )),
            _ => None,
        }
    }

    /// Checks that the IME is enabled with its caret inside `area`, that it
    /// shows what is composed and that it gives the committed text.
    #[track_caller]
    fn assert_composes(&mut self, area: Rectangle) {
        let caret = self.caret().expect("the IME is enabled");
        assert!(
            area.expand(1.0).contains(caret.center()),
            "the caret {caret:?} of the IME is in {area:?}"
        );

        self.ime(input_method::Event::Opened);
        self.ime(input_method::Event::Preedit(
            "にほんご".to_owned(),
            Some(12..12),
        ));

        let (preedit, size) = self.preedit().expect("the preedit is shown");
        assert_eq!(preedit, "にほんご");
        assert!(
            size.is_none_or(|size| size >= 12.0),
            "the preedit is readable ({size:?}px)"
        );

        self.ime(input_method::Event::Preedit(String::new(), None));
        self.ime(input_method::Event::Commit("日本語".to_owned()));
    }
}

// Inputs

#[derive(Debug, Clone, PartialEq)]
enum Message {
    Value(String),
    Submit,
    Close,
    Select(u8),
}

#[derive(Default)]
struct Form {
    value: String,
    submitted: usize,
    closed: usize,
    selected: Option<u8>,
}

fn apply(form: &mut Form, message: Message) {
    match message {
        Message::Value(value) => form.value = value,
        Message::Submit => form.submitted += 1,
        Message::Close => form.closed += 1,
        Message::Select(value) => form.selected = Some(value),
    }
}

fn page(content: Element<'_, Message>) -> Element<'_, Message> {
    container(column![text("Outside"), content].spacing(20))
        .padding(20)
        .into()
}

fn input(form: &Form) -> Element<'_, Message> {
    page(
        shadcn_iced::input::input("Name", &form.value)
            .on_input(Message::Value)
            .on_submit(Message::Submit)
            .width(240)
            .into(),
    )
}

#[test]
fn an_input_takes_japanese() {
    let mut ui = Ui::new(Form::default(), input, apply);
    let area = ui.click("Name");

    ui.assert_composes(area);
    assert_eq!(ui.state.value, "日本語");

    // The caret moved after the text.
    ui.compose("てすと", "テスト");
    assert_eq!(ui.state.value, "日本語テスト");
    assert_eq!(ui.state.submitted, 0);

    // The IME is closed once the focus leaves.
    ui.click("Outside");
    assert_eq!(ui.caret(), None);
}

#[test]
fn keys_pressed_while_composing_belong_to_the_ime() {
    let mut ui = Ui::new(Form::default(), input, apply);
    ui.click("Name");
    ui.compose("あ", "亜");

    ui.ime(input_method::Event::Opened);
    ui.ime(input_method::Event::Preedit("にほ".to_owned(), Some(6..6)));

    // Some platforms report the keys the IME handles, too.
    ui.key(Key::Named(Named::Backspace), None);
    ui.key(Key::Named(Named::ArrowLeft), None);
    ui.key(Key::Named(Named::Enter), Some("\r"));
    ui.key(Key::Character("a".into()), Some("a"));

    assert_eq!(ui.state.value, "亜");
    assert_eq!(ui.state.submitted, 0);

    ui.ime(input_method::Event::Preedit(String::new(), None));
    ui.ime(input_method::Event::Commit("日本".to_owned()));
    assert_eq!(ui.state.value, "亜日本");

    // Done composing: the keys are the input's again.
    ui.key(Key::Named(Named::Backspace), None);
    ui.key(Key::Named(Named::Enter), Some("\r"));
    assert_eq!(ui.state.value, "亜日");
    assert_eq!(ui.state.submitted, 1);
}

#[test]
fn a_password_input_asks_for_a_secure_ime() {
    fn view(form: &Form) -> Element<'_, Message> {
        page(
            shadcn_iced::input::input("Password", &form.value)
                .on_input(Message::Value)
                .secure(true)
                .width(240)
                .into(),
        )
    }

    let mut ui = Ui::new(Form::default(), view, apply);
    ui.click("Password");

    assert!(matches!(
        ui.input_method,
        InputMethod::Enabled {
            purpose: input_method::Purpose::Secure,
            ..
        }
    ));
}

#[test]
fn an_input_group_takes_japanese() {
    fn view(form: &Form) -> Element<'_, Message> {
        page(
            shadcn_iced::input_group::input_group(
                shadcn_iced::input::input("Search", &form.value).on_input(Message::Value),
            )
            .leading_icon(shadcn_iced::icon::lucide::SEARCH)
            .trailing_text("12 results")
            .width(300)
            .into(),
        )
    }

    let mut ui = Ui::new(Form::default(), view, apply);
    let area = ui.click("Search");

    ui.assert_composes(area);
    assert_eq!(ui.state.value, "日本語");
}

#[derive(Default)]
struct Notes {
    content: shadcn_iced::textarea::Content,
}

#[test]
fn a_textarea_takes_japanese() {
    fn view(notes: &Notes) -> Element<'_, shadcn_iced::textarea::Action> {
        container(column![
            text("Outside"),
            shadcn_iced::textarea::textarea(&notes.content)
                .placeholder("Notes")
                .on_action(|action| action)
                .width(300),
        ])
        .padding(20)
        .into()
    }

    fn apply(notes: &mut Notes, action: shadcn_iced::textarea::Action) {
        notes.content.perform(action);
    }

    let mut ui = Ui::new(Notes::default(), view, apply);
    let outside = ui.bounds("Outside");
    let area = Rectangle::new(
        Point::new(outside.x, outside.y + outside.height),
        Size::new(300.0, 64.0),
    );
    ui.click_at(area.center());

    ui.assert_composes(area);
    assert_eq!(ui.state.content.text(), "日本語");

    ui.ime(input_method::Event::Opened);
    ui.ime(input_method::Event::Preedit(
        "かいぎょう".to_owned(),
        Some(15..15),
    ));
    ui.key(Key::Named(Named::Enter), Some("\r"));
    ui.ime(input_method::Event::Preedit(String::new(), None));
    ui.ime(input_method::Event::Commit("改行".to_owned()));

    assert_eq!(ui.state.content.text(), "日本語改行");
}

#[test]
fn an_input_in_a_dialog_takes_japanese() {
    fn view(form: &Form) -> Element<'_, Message> {
        shadcn_iced::dialog::dialog(page(text("Base").into()), true)
            .title("Edit profile")
            .content(
                shadcn_iced::input::input("Name", &form.value)
                    .on_input(Message::Value)
                    .on_submit(Message::Submit),
            )
            .on_close(Message::Close)
            .into()
    }

    let mut ui = Ui::new(Form::default(), view, apply);
    let area = ui.click("Name");

    ui.assert_composes(area);
    assert_eq!(ui.state.value, "日本語");

    // Escape cancels the conversion, not the dialog.
    ui.ime(input_method::Event::Opened);
    ui.ime(input_method::Event::Preedit("にほ".to_owned(), Some(6..6)));
    ui.key(Key::Named(Named::Escape), None);
    ui.ime(input_method::Event::Preedit(String::new(), None));
    ui.ime(input_method::Event::Closed);

    assert_eq!(ui.state.closed, 0);
    assert_eq!(ui.state.value, "日本語");

    // Done composing: Escape leaves the input, then closes the dialog.
    ui.key(Key::Named(Named::Escape), None);
    assert_eq!(ui.caret(), None);
    ui.key(Key::Named(Named::Escape), None);
    assert_eq!(ui.state.closed, 1);
}

#[test]
fn a_command_takes_japanese() {
    fn view(form: &Form) -> Element<'_, Message> {
        page(
            container(
                shadcn_iced::command::command(&form.value, Message::Value)
                    .placeholder("Search commands")
                    .items([
                        shadcn_iced::command::item("設定").on_select(Message::Select(1)),
                        shadcn_iced::command::item("日本語の設定").on_select(Message::Select(2)),
                    ]),
            )
            .width(300)
            .into(),
        )
    }

    let mut ui = Ui::new(Form::default(), view, apply);
    let area = ui.click("Search commands");

    ui.assert_composes(area);
    assert_eq!(ui.state.value, "日本語");

    // The arrows and Enter of the conversion do not pick an item.
    ui.ime(input_method::Event::Opened);
    ui.ime(input_method::Event::Preedit("の".to_owned(), Some(3..3)));
    ui.key(Key::Named(Named::ArrowDown), None);
    ui.key(Key::Named(Named::Enter), Some("\r"));
    ui.ime(input_method::Event::Preedit(String::new(), None));
    ui.ime(input_method::Event::Commit("の".to_owned()));

    assert_eq!(ui.state.selected, None);
    assert_eq!(ui.state.value, "日本語の");

    ui.key(Key::Named(Named::Enter), Some("\r"));
    assert_eq!(ui.state.selected, Some(2));
}

#[test]
fn a_combobox_takes_japanese() {
    fn view(form: &Form) -> Element<'_, Message> {
        page(
            shadcn_iced::combobox::combobox(
                form.selected,
                Message::Select,
                &form.value,
                Message::Value,
            )
            .placeholder("Pick a city")
            .option(1, "東京")
            .option(2, "大阪")
            .into(),
        )
    }

    let mut ui = Ui::new(Form::default(), view, apply);
    let area = ui.click("Pick a city");

    ui.compose("おおさか", "大阪");
    assert_eq!(ui.state.value, "大阪");

    let caret = ui.caret().expect("the IME is enabled");
    assert!(area.expand(1.0).contains(caret.center()));

    ui.ime(input_method::Event::Opened);
    ui.ime(input_method::Event::Preedit("し".to_owned(), Some(3..3)));
    ui.key(Key::Named(Named::Enter), Some("\r"));
    ui.ime(input_method::Event::Preedit(String::new(), None));
    ui.ime(input_method::Event::Closed);
    assert_eq!(ui.state.selected, None);

    ui.key(Key::Named(Named::Enter), Some("\r"));
    assert_eq!(ui.state.selected, Some(2));
}

#[test]
fn a_one_time_password_takes_full_width_digits() {
    fn view(form: &Form) -> Element<'_, Message> {
        page(
            column![
                text("Code"),
                shadcn_iced::input_otp::input_otp(6, &form.value, Message::Value)
                    .on_submit(Message::Submit),
            ]
            .into(),
        )
    }

    let mut ui = Ui::new(Form::default(), view, apply);
    let code = ui.bounds("Code");
    ui.click_at(Point::new(code.x + 10.0, code.y + code.height + 20.0));

    let caret = ui.caret().expect("the IME is enabled");

    ui.ime(input_method::Event::Opened);
    ui.ime(input_method::Event::Preedit(
        "１２３".to_owned(),
        Some(9..9),
    ));
    let (preedit, size) = ui.preedit().expect("the preedit is shown");
    assert_eq!(preedit, "１２３");
    assert!(
        size.is_some_and(|size| size >= 12.0),
        "a readable preedit ({size:?})"
    );
    assert!(caret.height >= 12.0, "the caret is the slot: {caret:?}");

    ui.ime(input_method::Event::Preedit(String::new(), None));
    ui.ime(input_method::Event::Commit("１２３　４５６".to_owned()));

    // Full-width digits, as typed with a Japanese IME on, are the digits.
    assert_eq!(ui.state.value, "123456");
}

#[test]
fn a_one_time_password_leaves_the_keys_of_the_ime() {
    fn view(form: &Form) -> Element<'_, Message> {
        page(
            column![
                text("Code"),
                shadcn_iced::input_otp::input_otp(6, &form.value, Message::Value)
                    .on_submit(Message::Submit),
            ]
            .into(),
        )
    }

    let mut ui = Ui::new(Form::default(), view, apply);
    let code = ui.bounds("Code");
    ui.click_at(Point::new(code.x + 10.0, code.y + code.height + 20.0));

    ui.ime(input_method::Event::Opened);
    ui.ime(input_method::Event::Preedit("１２".to_owned(), Some(6..6)));
    ui.key(Key::Named(Named::Enter), Some("\r"));
    ui.ime(input_method::Event::Preedit(String::new(), None));
    ui.ime(input_method::Event::Commit("１２".to_owned()));

    assert_eq!(ui.state.value, "12");
    assert_eq!(ui.state.submitted, 0);
}
