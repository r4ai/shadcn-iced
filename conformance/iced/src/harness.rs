//! Runs a fixture headlessly, the way the iced runtime would: the widget tree
//! is kept across interactions (so focus, hover and self-managed overlays
//! survive), messages update the [`Store`], and the view is rebuilt after
//! every event.
use crate::fixtures::Fixture;
use crate::store::{Msg, Store};

use iced::keyboard::{self, Key, Modifiers, key::Named};
use iced::{Event, Point, Rectangle, Size, Theme, Vector, mouse, window};
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

        // Like the iced runtime, tell the widgets about the window (e.g. for
        // `text-base md:text-sm`).
        harness.dispatch(Event::Window(window::Event::Opened {
            position: None,
            size,
        }));
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
                crate::page::page((self.fixture.view)(&self.store)),
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
    pub fn visible_texts(&mut self, text: &str, exact: bool) -> Vec<Rectangle> {
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
                Outcome::Some(bounds) => bounds,
                _ => Vec::new(),
            }
        })
        .into_iter()
        .filter_map(|bounds| self.clip_to_viewport(bounds))
        .collect()
    }

    /// Finds the visible center of the widget with the given id: the
    /// counterpart of an accessible name (`aria-label`) on the web, for
    /// controls without text.
    pub fn locate_id(&mut self, id: &str) -> Option<Point> {
        let id = iced::widget::Id::from(id.to_owned());

        self.with_interface(|interface, context| {
            let mut operation = (move |candidate: Candidate<'_>| {
                (candidate.id() == Some(&id))
                    .then(|| candidate.visible_bounds())
                    .flatten()
            })
            .find();

            interface.operate(context.renderer, &mut operation::black_box(&mut operation));

            match operation.finish() {
                Outcome::Some(bounds) => bounds,
                _ => None,
            }
        })
        .and_then(|bounds| self.clip_to_viewport(bounds))
        .map(|bounds| bounds.center())
    }

    /// The part of `bounds` within the viewport, if any: content that
    /// overflows the page is not visible, like on the web.
    fn clip_to_viewport(&self, bounds: Rectangle) -> Option<Rectangle> {
        bounds
            .intersection(&Rectangle::with_size(self.size))
            .filter(|bounds| bounds.width > 0.0 && bounds.height > 0.0)
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

    /// Scrolls the wheel by `delta` logical pixels at `position`, like
    /// Playwright's `mouse.wheel(dx, dy)`: a positive `y` scrolls down.
    pub fn scroll(&mut self, position: Point, delta: Vector) {
        self.move_to(position);
        self.dispatch(Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Pixels {
                x: -delta.x,
                y: -delta.y,
            },
        }));
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

#[cfg(test)]
mod tests {
    use super::*;

    use iced::widget::{column, container, scrollable, text};
    use iced::{Element, Pixels};

    fn harness(view: for<'a> fn(&'a Store) -> Element<'a, Msg>, size: Size) -> Harness {
        static FONTS: std::sync::Once = std::sync::Once::new();
        FONTS.call_once(crate::fonts::load);

        Harness::new(Fixture::new(view), Theme::Light, size)
    }

    /// Ten lines of 20px: 200px in all.
    fn items<'a>() -> iced::widget::Column<'a, Msg> {
        column((0..10).map(|i| {
            text(format!("Item {i}"))
                .size(14)
                .line_height(Pixels(20.0))
                .into()
        }))
    }

    #[test]
    fn content_taller_than_the_viewport_overflows() {
        let mut harness = harness(|_| items().into(), Size::new(200.0, 100.0));

        // Laid out with its own height, not squeezed into the viewport...
        let item = harness.visible_texts("Item 3", true);
        assert_eq!(item.len(), 1);
        assert_eq!(item[0].y, 16.0 + 3.0 * 20.0);
        assert_eq!(item[0].height, 20.0);

        // ...and cropped by it, like a browser page.
        assert!(harness.visible_texts("Item 4", true)[0].height < 20.0);
        assert!(harness.visible_texts("Item 9", true).is_empty());
    }

    #[test]
    fn content_filling_the_viewport_keeps_its_height() {
        let mut harness = harness(
            |_| {
                container(text("Bottom"))
                    .height(iced::Fill)
                    .align_bottom(iced::Fill)
                    .into()
            },
            Size::new(200.0, 100.0),
        );

        let bottom = harness.visible_texts("Bottom", true);
        assert_eq!(bottom.len(), 1);
        assert_eq!(bottom[0].y + bottom[0].height, 100.0 - 16.0);
    }

    #[test]
    fn wheel_scrolls_by_pixels() {
        let mut harness = harness(
            |_| scrollable(items()).height(60).into(),
            Size::new(200.0, 120.0),
        );

        assert!(!harness.visible_texts("Item 0", true).is_empty());
        assert!(harness.visible_texts("Item 6", true).is_empty());

        harness.scroll(Point::new(50.0, 40.0), Vector::new(0.0, 100.0));

        assert!(harness.visible_texts("Item 0", true).is_empty());
        let item = harness.visible_texts("Item 5", true);
        assert_eq!(item.len(), 1);
        assert_eq!(item[0].y, 16.0 + 5.0 * 20.0 - 100.0);
    }

    #[test]
    fn locates_controls_by_id() {
        let mut harness = harness(
            |_| {
                column![
                    text("Above").line_height(Pixels(20.0)),
                    container(text("x")).id("close").width(40).height(20)
                ]
                .into()
            },
            Size::new(200.0, 100.0),
        );

        assert_eq!(harness.locate_id("close"), Some(Point::new(36.0, 46.0)));
        assert_eq!(harness.locate_id("missing"), None);
    }
}
