//! The live previews of the website, compiled to WebAssembly.
//!
//! A single [`iced::daemon`] runs every preview of a page: each example is a
//! window of its own, drawn into the canvas that replaces the element the
//! page passes to [`mount`]. The windows share the state of the gallery, like
//! the examples of a page of the native gallery do.
//!
//! The page talks to the previews with the exported functions ([`mount`],
//! [`unmount`], [`set_theme`] and [`theme_code`]), and the previews call back
//! two functions of `window`:
//!
//! - `shadcnIcedPreviewResized(target, height)`, with the natural height of an
//!   example, so the page can size its preview to fit,
//! - `shadcnIcedPreviewWheel(captured)`, with whether an example used the
//!   wheel event it is handling, so the page lets the browser scroll over a
//!   preview otherwise (see [`wheel`]).
//!
//! Build it with `site/scripts/build-wasm.mjs`.
#![cfg(target_arch = "wasm32")]

mod natural;
mod wheel;

use gallery::{Gallery, PAGES};
use shadcn_iced::theme::{Accent, BaseColor, Radius};
use shadcn_iced::{ApplicationExt, Mode};

use iced::futures::channel::mpsc;
use iced::widget::{container, sensor, space};
use iced::{Element, Fill, Size, Subscription, Task, Theme, event, window};
use std::cell::RefCell;
use std::collections::BTreeMap;
use wasm_bindgen::prelude::*;

/// The padding around an example (`p-10` of the preview of shadcn/ui).
const PADDING: f32 = 40.0;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = window, js_name = shadcnIcedPreviewResized)]
    fn preview_resized(target: &str, height: f32) -> Result<(), JsValue>;

    #[wasm_bindgen(catch, js_namespace = window, js_name = shadcnIcedPreviewWheel)]
    fn preview_wheel(captured: bool) -> Result<(), JsValue>;
}

/// A request of the page.
#[derive(Debug, Clone)]
enum Request {
    Mount {
        target: String,
        page: usize,
        example: usize,
    },
    Unmount {
        target: String,
    },
    Theme(Options),
}

/// The theme options of the website.
#[derive(Debug, Clone, Copy)]
struct Options {
    mode: Mode,
    base: BaseColor,
    accent: Option<Accent>,
    radius: Radius,
}

impl Options {
    fn parse(dark: bool, base: &str, accent: &str, radius: &str) -> Self {
        Self {
            mode: if dark { Mode::Dark } else { Mode::Light },
            base: BaseColor::ALL
                .iter()
                .copied()
                .find(|b| b.name() == base)
                .unwrap_or_default(),
            accent: Accent::ALL.iter().copied().find(|a| a.name() == accent),
            radius: match radius {
                "none" => Radius::None,
                "small" => Radius::Small,
                "medium" => Radius::Medium,
                "large" => Radius::Large,
                _ => Radius::Default,
            },
        }
    }

    /// The messages that apply these options to the gallery.
    fn messages(self, gallery: &Gallery) -> Vec<gallery::Message> {
        let mut messages = vec![
            gallery::Message::Base(self.base),
            gallery::Message::Accent(self.accent),
            gallery::Message::Radius(self.radius),
        ];

        if gallery.mode != self.mode {
            messages.push(gallery::Message::ToggleMode);
        }

        messages
    }
}

thread_local! {
    static REQUESTS: RefCell<Option<(mpsc::UnboundedSender<Request>, Option<mpsc::UnboundedReceiver<Request>>)>> =
        const { RefCell::new(None) };
}

fn send(request: Request) {
    REQUESTS.with_borrow(|requests| {
        if let Some((sender, _)) = requests {
            let _ = sender.unbounded_send(request);
        }
    });
}

/// Starts the previews; the page then mounts them with [`mount`].
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();

    let (sender, receiver) = mpsc::unbounded();
    REQUESTS.set(Some((sender, Some(receiver))));

    // winit makes the title of a window the title of the document.
    let title = web_sys::window()
        .and_then(|window| window.document())
        .map(|document| document.title())
        .unwrap_or_default();

    let result = iced::daemon(Previews::new, Previews::update, Previews::view)
        .title(move |_: &Previews, _| title.clone())
        .theme(Previews::theme)
        .subscription(Previews::subscription)
        .shadcn_iced()
        .run();

    if let Err(error) = result {
        web_sys::console::error_1(&format!("{error}").into());
    }
}

/// Replaces the element with the id `target` with the live preview of an
/// example: the `example`-th example of the page with the id `page`.
///
/// Returns `false` when there is no such page.
#[wasm_bindgen]
pub fn mount(target: String, page: &str, example: usize) -> bool {
    let Some(page) = PAGES.iter().position(|p| p.id == page) else {
        return false;
    };

    send(Request::Mount {
        target,
        page,
        example,
    });

    true
}

/// Closes the live preview mounted on `target`, when the page removes it.
#[wasm_bindgen]
pub fn unmount(target: String) {
    send(Request::Unmount { target });
}

/// Sets the theme of every preview: the mode, the base color, the theme
/// (accent) and the radius, with the names of the `/create` page of
/// shadcn/ui (e.g. `"zinc"`, `"blue"`, `"large"`).
#[wasm_bindgen(js_name = setTheme)]
pub fn set_theme(dark: bool, base: &str, accent: &str, radius: &str) {
    send(Request::Theme(Options::parse(dark, base, accent, radius)));
}

/// The Rust code that builds the theme of [`set_theme`].
#[wasm_bindgen(js_name = themeCode)]
pub fn theme_code(dark: bool, base: &str, accent: &str, radius: &str) -> String {
    let options = Options::parse(dark, base, accent, radius);
    let mut gallery = Gallery::new(0, options.mode);

    for message in options.messages(&gallery) {
        let _ = gallery.update(message);
    }

    gallery::customizer::code(&gallery)
}

/// The previews of a page.
struct Previews {
    gallery: Gallery,
    windows: BTreeMap<window::Id, Preview>,
    /// The window that was clicked last, which shows the toasts.
    active: Option<window::Id>,
}

/// The preview of an example.
struct Preview {
    target: String,
    page: usize,
    example: usize,
    height: f32,
}

#[derive(Debug, Clone)]
enum Message {
    Gallery(gallery::Message),
    Request(Request),
    Measured(window::Id, Size),
    Pressed(window::Id),
    Opened,
}

impl Previews {
    fn new() -> Self {
        Self {
            gallery: Gallery::new(0, Mode::Light),
            windows: BTreeMap::new(),
            active: None,
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Gallery(message) => {
                return self.gallery.update(message).map(Message::Gallery);
            }
            Message::Request(Request::Mount {
                target,
                page,
                example,
            }) => {
                let (id, open) = window::open(window::Settings {
                    size: Size::new(800.0, 400.0),
                    platform_specific: window::settings::PlatformSpecific {
                        target: Some(target.clone()),
                    },
                    ..window::Settings::default()
                });

                let _ = self.windows.insert(
                    id,
                    Preview {
                        target,
                        page,
                        example,
                        height: 0.0,
                    },
                );

                return open.map(|_| Message::Opened);
            }
            Message::Request(Request::Unmount { target }) => {
                let Some(id) = self
                    .windows
                    .iter()
                    .find_map(|(id, preview)| (preview.target == target).then_some(*id))
                else {
                    return Task::none();
                };

                let _ = self.windows.remove(&id);
                if self.active == Some(id) {
                    self.active = None;
                }

                return window::close(id);
            }
            Message::Request(Request::Theme(options)) => {
                for message in options.messages(&self.gallery) {
                    let _ = self.gallery.update(message);
                }
            }
            Message::Measured(id, size) => {
                if let Some(preview) = self.windows.get_mut(&id)
                    && (preview.height - size.height).abs() >= 1.0
                {
                    preview.height = size.height;
                    let _ = preview_resized(&preview.target, size.height + 2.0 * PADDING);
                }
            }
            Message::Pressed(id) => self.active = Some(id),
            Message::Opened => {}
        }

        Task::none()
    }

    fn view(&self, id: window::Id) -> Element<'_, Message> {
        let Some(preview) = self.windows.get(&id) else {
            return space().into();
        };

        let example = gallery::view_example(&self.gallery, preview.page, preview.example);

        // The toasts show in the preview that was clicked last.
        let example = self.gallery.toasters(example, self.active == Some(id));

        let example = sensor(natural::natural(example.map(Message::Gallery)))
            .on_resize(move |size| Message::Measured(id, size));

        wheel::wheel(
            container(example).padding(PADDING).center(Fill),
            |captured| {
                let _ = preview_wheel(captured);
            },
        )
        .into()
    }

    fn theme(&self, _window: window::Id) -> Theme {
        self.gallery.theme()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            Subscription::run(requests).map(Message::Request),
            event::listen_with(|event, _status, window| match event {
                iced::Event::Mouse(iced::mouse::Event::ButtonPressed(_))
                | iced::Event::Touch(iced::touch::Event::FingerPressed { .. }) => {
                    Some(Message::Pressed(window))
                }
                _ => None,
            }),
        ])
    }
}

/// The requests of the page, as a stream (it can only be taken once).
fn requests() -> impl iced::futures::Stream<Item = Request> {
    use iced::futures::StreamExt;

    let receiver = REQUESTS
        .with_borrow_mut(|requests| requests.as_mut().and_then(|(_, receiver)| receiver.take()));

    iced::futures::stream::iter(receiver).flatten()
}
