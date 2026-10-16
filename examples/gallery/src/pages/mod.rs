//! One page per component.
mod button;

use crate::{Gallery, Message};

use iced::{Element, Task};

/// A page of the gallery.
pub struct Page {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub view: fn(&Gallery) -> Element<'_, Message>,
}

pub const PAGES: &[Page] = &[button::PAGE];

/// The state of every demo.
#[derive(Debug, Default)]
pub struct Demo {
    pub clicks: u32,
}

#[derive(Debug, Clone)]
pub enum DemoMessage {
    Clicked,
}

impl Demo {
    pub fn update(&mut self, message: DemoMessage) -> Task<Message> {
        match message {
            DemoMessage::Clicked => self.clicks += 1,
        }

        Task::none()
    }
}
