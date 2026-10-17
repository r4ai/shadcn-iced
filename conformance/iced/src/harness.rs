//! Runs a fixture headlessly, the way the iced runtime would: the widget tree
//! is kept across interactions (so focus, hover and self-managed overlays
//! survive), messages update the [`Store`], and the view is rebuilt after
//! every event.
use crate::fixtures::Fixture;
use crate::store::{Msg, Store};

use iced::keyboard::{self, Key, Modifiers, key::Named};
use iced::{Event, Point, Size, Theme, mouse, window};
use iced_test::core::clipboard;
use iced_test::core::renderer::{Headless, Style};
use iced_test::core::widget::operation::{self, Operation, Outcome};
use iced_test::renderer::Renderer;
use iced_test::runtime::user_interface::{self, UserInterface};
use iced_test::selector::{Candidate, Selector};

use iced::time::Instant;

/// The scale factor of the snapshots (the shadcn/ui runner uses the same
/// device scale factor).
pub const SCALE_FACTOR: f32 = 2.0;

pub struct Harness {
    fixture: Fixture,
    store: Store,
    renderer: Renderer,
    theme: Theme,
    size: Size,
    cache: Option<user_interface::Cache>,
    cursor: mouse::Cursor,
    modifiers: Modifiers,
    clipboard: Clipboard,
}

impl Harness {
    pub fn new(fixture: Fixture, theme: Theme, size: Size) -> Self {
        let renderer = iced::futures::executor::block_on(Renderer::new(
            shadcn_iced::font::sans(),
            iced::Pixels(16.0),
            Some("tiny-skia"),
        ))
        .expect("create a headless renderer");

        let mut store = Store::default();
        (fixture.init)(&mut store);

        let mut harness = Self {
            fixture,
            store,
            renderer,
            theme,
            size,
            cache: Some(user_interface::Cache::default()),
            cursor: mouse::Cursor::Unavailable,
            modifiers: Modifiers::default(),
            clipboard: Clipboard(None),
        };

        harness.redraw();
        harness
    }

    /// Builds the interface, runs `f` on it, then applies the produced messages.
    fn with_interface<T>(
        &mut self,
        f: impl FnOnce(&mut UserInterface<'_, Msg, Theme, Renderer>, &mut Context<'_>) -> T,
    ) -> T {
        let mut messages = Vec::new();

        let output = {
            let mut interface = UserInterface::build(
                crate::fixtures::frame((self.fixture.view)(&self.store)),
                self.size,
                self.cache.take().expect("cache"),
                &mut self.renderer,
            );

            let output = f(
                &mut interface,
                &mut Context {
                    renderer: &mut self.renderer,
                    cursor: self.cursor,
                    clipboard: &mut self.clipboard,
                    messages: &mut messages,
                    theme: &self.theme,
                },
            );

            self.cache = Some(interface.into_cache());
            output
        };

        for message in messages {
            self.store.update(message);
        }

        output
    }

    /// Dispatches a single event, as a frame of its own, returning whether
    /// a widget captured it.
    pub fn dispatch(&mut self, event: Event) -> bool {
        let captured = self.with_interface(|interface, context| {
            let (_state, statuses) = interface.update(
                &[event],
                context.cursor,
                context.renderer,
                context.clipboard,
                context.messages,
            );

            statuses.contains(&iced::event::Status::Captured)
        });

        self.redraw();
        captured
    }

    fn redraw(&mut self) {
        self.with_interface(|interface, context| {
            let _ = interface.update(
                &[Event::Window(
                    window::Event::RedrawRequested(Instant::now()),
                )],
                context.cursor,
                context.renderer,
                context.clipboard,
                context.messages,
            );
        });
    }

    /// Finds the visible center of the `nth` text with the given content.
    pub fn locate_text(&mut self, text: &str, nth: usize, exact: bool) -> Option<Point> {
        self.visible_texts(text, exact)
            .into_iter()
            .nth(nth)
            .map(|bounds| bounds.center())
    }

    /// Returns the visible bounds of every text with the given content, or
    /// containing it when not `exact`.
    pub fn visible_texts(&mut self, text: &str, exact: bool) -> Vec<iced::Rectangle> {
        let text = text.to_owned();

        self.with_interface(|interface, context| {
            let mut operation = (move |candidate: Candidate<'_>| {
                let content = match &candidate {
                    Candidate::Text { content, .. } => *content,
                    Candidate::TextInput { state, .. } => state.text(),
                    _ => return None,
                };

                let matches = if exact {
                    content == text
                } else {
                    content.contains(text.as_str())
                };

                matches.then(|| candidate.visible_bounds()).flatten()
            })
            .find_all();

            interface.operate(context.renderer, &mut operation::black_box(&mut operation));

            match operation.finish() {
                Outcome::Some(bounds) => bounds
                    .into_iter()
                    .filter(|bounds| bounds.width > 0.0 && bounds.height > 0.0)
                    .collect(),
                _ => Vec::new(),
            }
        })
    }

    pub fn move_to(&mut self, position: Point) {
        self.cursor = mouse::Cursor::Available(position);
        self.dispatch(Event::Mouse(mouse::Event::CursorMoved { position }));
    }

    /// Moves the pointer in `steps` moves from its current position, like
    /// Playwright's `mouse.move(x, y, { steps })`.
    pub fn move_in_steps(&mut self, position: Point, steps: u32) {
        let from = match self.cursor {
            mouse::Cursor::Available(from) => from,
            _ => position,
        };

        for i in 1..=steps.max(1) {
            let t = i as f32 / steps.max(1) as f32;
            self.move_to(Point::new(
                from.x + (position.x - from.x) * t,
                from.y + (position.y - from.y) * t,
            ));
        }
    }

    pub fn press(&mut self) {
        self.press_button(mouse::Button::Left);
    }

    pub fn release(&mut self) {
        self.release_button(mouse::Button::Left);
    }

    fn press_button(&mut self, button: mouse::Button) {
        self.dispatch(Event::Mouse(mouse::Event::ButtonPressed(button)));
    }

    fn release_button(&mut self, button: mouse::Button) {
        self.dispatch(Event::Mouse(mouse::Event::ButtonReleased(button)));
    }

    pub fn click(&mut self, position: Point) {
        self.move_to(position);
        self.press();
        self.release();
    }

    pub fn right_click(&mut self, position: Point) {
        self.move_to(position);
        self.press_button(mouse::Button::Right);
        self.release_button(mouse::Button::Right);
    }

    pub fn drag(&mut self, from: Point, to: Point) {
        self.move_to(from);
        self.press();

        // Playwright moves the mouse in a single step by default, but some
        // widgets only start dragging after some movement.
        let steps = 5;
        for i in 1..=steps {
            let t = i as f32 / steps as f32;
            self.move_to(Point::new(
                from.x + (to.x - from.x) * t,
                from.y + (to.y - from.y) * t,
            ));
        }

        self.release();
    }

    /// Presses a key combination using Playwright's names, e.g. `ArrowDown`,
    /// `Shift+Tab` or `a`.
    pub fn press_key(&mut self, combination: &str) -> Result<(), String> {
        let mut parts: Vec<&str> = combination.split('+').collect();
        let name = parts.pop().ok_or("empty key")?;

        let mut modifiers = Modifiers::default();
        for part in parts {
            modifiers |= match part {
                "Shift" => Modifiers::SHIFT,
                "Control" => Modifiers::CTRL,
                "Alt" => Modifiers::ALT,
                "Meta" => Modifiers::LOGO,
                "ControlOrMeta" => Modifiers::COMMAND,
                other => return Err(format!("unknown modifier {other:?}")),
            };
        }

        let (key, text) = parse_key(name)?;

        self.set_modifiers(modifiers);
        let captured = self.key(key.clone(), text);

        // Like iced applications do (there is no built-in focus traversal),
        // Tab moves the focus when no widget handles it.
        if key == Key::Named(Named::Tab) && !captured {
            let backwards = modifiers.shift();

            self.with_interface(|interface, context| {
                let mut operation: Box<dyn Operation<()>> = if backwards {
                    Box::new(operation::focusable::focus_previous::<()>())
                } else {
                    Box::new(operation::focusable::focus_next::<()>())
                };

                // Like the iced runtime, run the operations they chain
                // (`focus_next` counts the focusable widgets first).
                loop {
                    interface.operate(context.renderer, operation.as_mut());

                    match operation.finish() {
                        Outcome::Chain(next) => operation = next,
                        _ => break,
                    }
                }
            });
            self.redraw();
        }

        self.set_modifiers(Modifiers::default());

        Ok(())
    }

    pub fn type_text(&mut self, text: &str) {
        for c in text.chars() {
            let text = c.to_string();
            let _ = self.key(Key::Character(text.as_str().into()), Some(text));
        }
    }

    /// Taps a key, returning whether a widget captured its press.
    fn key(&mut self, key: Key, text: Option<String>) -> bool {
        let captured = self.dispatch(Event::Keyboard(keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key.clone(),
            physical_key: keyboard::key::Physical::Unidentified(
                keyboard::key::NativeCode::Unidentified,
            ),
            location: keyboard::Location::Standard,
            modifiers: self.modifiers,
            repeat: false,
            text: text.map(Into::into),
        }));

        self.dispatch(Event::Keyboard(keyboard::Event::KeyReleased {
            key: key.clone(),
            modified_key: key,
            physical_key: keyboard::key::Physical::Unidentified(
                keyboard::key::NativeCode::Unidentified,
            ),
            location: keyboard::Location::Standard,
            modifiers: self.modifiers,
        }));

        captured
    }

    fn set_modifiers(&mut self, modifiers: Modifiers) {
        if self.modifiers != modifiers {
            self.modifiers = modifiers;
            self.dispatch(Event::Keyboard(keyboard::Event::ModifiersChanged(
                modifiers,
            )));
        }
    }

    pub fn wait(&mut self, ms: u64) {
        // Wake up regularly, like a real event loop would for widgets that
        // request redraws at a given time (hover delays, timers, ...).
        let end = Instant::now() + std::time::Duration::from_millis(ms);

        while Instant::now() < end {
            std::thread::sleep(std::time::Duration::from_millis(16).min(end - Instant::now()));
            self.redraw();
        }

        self.redraw();
    }

    /// Draws the interface and returns its RGBA pixels at [`SCALE_FACTOR`].
    pub fn snapshot(&mut self) -> (Vec<u8>, u32, u32) {
        self.redraw();

        let size = self.size;

        self.with_interface(|interface, context| {
            let base = iced::theme::Base::base(context.theme);

            // A freshly built interface only draws its overlays once they
            // have been laid out, by an update.
            let _ = interface.update(
                &[Event::Window(
                    window::Event::RedrawRequested(Instant::now()),
                )],
                context.cursor,
                context.renderer,
                context.clipboard,
                context.messages,
            );

            interface.draw(
                context.renderer,
                context.theme,
                &Style {
                    text_color: base.text_color,
                },
                context.cursor,
            );

            let width = (size.width * SCALE_FACTOR).round() as u32;
            let height = (size.height * SCALE_FACTOR).round() as u32;

            let rgba = context.renderer.screenshot(
                Size::new(width, height),
                SCALE_FACTOR,
                base.background_color,
            );

            (rgba, width, height)
        })
    }
}

struct Context<'a> {
    renderer: &'a mut Renderer,
    cursor: mouse::Cursor,
    clipboard: &'a mut Clipboard,
    messages: &'a mut Vec<Msg>,
    theme: &'a Theme,
}

struct Clipboard(Option<String>);

impl clipboard::Clipboard for Clipboard {
    fn read(&self, _kind: clipboard::Kind) -> Option<String> {
        self.0.clone()
    }

    fn write(&mut self, _kind: clipboard::Kind, contents: String) {
        self.0 = Some(contents);
    }
}

fn parse_key(name: &str) -> Result<(Key, Option<String>), String> {
    let named = match name {
        "Enter" => Named::Enter,
        "Escape" => Named::Escape,
        "Tab" => Named::Tab,
        "Space" | " " => return Ok((Key::Named(Named::Space), Some(" ".to_owned()))),
        "Backspace" => Named::Backspace,
        "Delete" => Named::Delete,
        "ArrowDown" => Named::ArrowDown,
        "ArrowUp" => Named::ArrowUp,
        "ArrowLeft" => Named::ArrowLeft,
        "ArrowRight" => Named::ArrowRight,
        "Home" => Named::Home,
        "End" => Named::End,
        "PageUp" => Named::PageUp,
        "PageDown" => Named::PageDown,
        other if other.chars().count() == 1 => {
            return Ok((Key::Character(other.into()), Some(other.to_owned())));
        }
        other => return Err(format!("unknown key {other:?}")),
    };

    Ok((Key::Named(named), None))
}
