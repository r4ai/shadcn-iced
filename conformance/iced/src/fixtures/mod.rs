//! The iced counterparts of `conformance/src/fixtures/*.tsx`.
//!
//! Every component has a module with a `get` function that maps the ids of
//! its scenarios (`conformance/scenarios/<component>.json`) to fixtures.
use crate::store::{Msg, Store};

use iced::Element;

mod button;

/// What a scenario renders.
pub struct Fixture {
    /// Sets the initial state of the store.
    pub init: Box<dyn Fn(&mut Store)>,
    pub view: for<'a> fn(&'a Store) -> Element<'a, Msg>,
}

impl Fixture {
    pub fn new(view: for<'a> fn(&'a Store) -> Element<'a, Msg>) -> Self {
        Self {
            init: Box::new(|_| {}),
            view,
        }
    }

    pub fn init(mut self, init: impl Fn(&mut Store) + 'static) -> Self {
        self.init = Box::new(init);
        self
    }
}

/// Returns the fixture of a scenario, if implemented.
pub fn get(component: &str, id: &str) -> Option<Fixture> {
    match component {
        "button" => button::get(id),
        _ => None,
    }
}

/// The page around every fixture: `<div class="p-4">` on the web.
pub fn frame(content: Element<'_, Msg>) -> Element<'_, Msg> {
    iced::widget::container(content)
        .padding(16)
        .width(iced::Fill)
        .height(iced::Fill)
        .into()
}
