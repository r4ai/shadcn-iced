//! A tiny key-value state shared by every fixture, so that fixtures do not
//! need their own `State` and `Message` types.
//!
//! On the shadcn/ui side, fixtures are uncontrolled React components; here,
//! the store plays the part of their internal state.
use iced::widget::text_editor;

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Bool(bool),
    Num(f32),
    Str(String),
    None,
}

#[derive(Debug, Clone)]
pub enum Msg {
    Set(&'static str, Value),
    Toggle(&'static str),
    /// Applies an action to the text editor content of a key.
    Edit(&'static str, text_editor::Action),
    Noop,
}

#[derive(Debug, Default)]
pub struct Store {
    values: HashMap<&'static str, Value>,
    editors: HashMap<&'static str, text_editor::Content>,
}

impl Store {
    pub fn update(&mut self, message: Msg) {
        match message {
            Msg::Set(key, value) => {
                let _ = self.values.insert(key, value);
            }
            Msg::Toggle(key) => {
                let value = !self.bool(key);
                let _ = self.values.insert(key, Value::Bool(value));
            }
            Msg::Edit(key, action) => {
                self.editors.entry(key).or_default().perform(action);
            }
            Msg::Noop => {}
        }
    }

    pub fn set(&mut self, key: &'static str, value: impl Into<Value>) -> &mut Self {
        let _ = self.values.insert(key, value.into());
        self
    }

    /// Sets the text of the text editor content of a key.
    pub fn set_editor(&mut self, key: &'static str, text: &str) -> &mut Self {
        let _ = self
            .editors
            .insert(key, text_editor::Content::with_text(text));
        self
    }

    /// The text editor content of a key, set with [`Store::set_editor`].
    pub fn editor(&self, key: &str) -> &text_editor::Content {
        self.editors
            .get(key)
            .unwrap_or_else(|| panic!("no editor content for {key:?}: call `set_editor` in `init`"))
    }

    pub fn bool(&self, key: &str) -> bool {
        matches!(self.values.get(key), Some(Value::Bool(true)))
    }

    pub fn num(&self, key: &str) -> f32 {
        match self.values.get(key) {
            Some(Value::Num(value)) => *value,
            _ => 0.0,
        }
    }

    pub fn str(&self, key: &str) -> &str {
        match self.values.get(key) {
            Some(Value::Str(value)) => value,
            _ => "",
        }
    }

    pub fn opt(&self, key: &str) -> Option<&str> {
        match self.values.get(key) {
            Some(Value::Str(value)) => Some(value),
            _ => None,
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Value::Num(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::Str(value.to_owned())
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::Str(value)
    }
}

impl<T: Into<Value>> From<Option<T>> for Value {
    fn from(value: Option<T>) -> Self {
        value.map_or(Value::None, Into::into)
    }
}

/// Returns a function producing a [`Msg`] that edits the text editor
/// content of `key`, e.g. `textarea(store.editor("text")).on_action(edit("text"))`.
pub fn edit(key: &'static str) -> impl Fn(text_editor::Action) -> Msg + Clone {
    move |action| Msg::Edit(key, action)
}

/// Returns a function producing a [`Msg`] that sets `key` to its argument,
/// e.g. `checkbox(..).on_toggle(set("checked"))`.
pub fn set<T: Into<Value>>(key: &'static str) -> impl Fn(T) -> Msg + Clone {
    move |value| Msg::Set(key, value.into())
}
