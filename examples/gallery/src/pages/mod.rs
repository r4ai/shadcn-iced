//! One page per component.
mod basics;
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

pub const PAGES: &[Page] = &[
    basics::ALERT,
    basics::ASPECT_RATIO,
    basics::AVATAR,
    basics::BADGE,
    basics::BREADCRUMB,
    button::PAGE,
    basics::CARD,
    basics::EMPTY,
    basics::INPUT,
    basics::ITEM,
    basics::KBD,
    basics::PAGINATION,
    basics::PROGRESS,
    basics::SCROLL_AREA,
    basics::SEPARATOR,
    basics::SKELETON,
    basics::SPINNER,
    basics::TABLE,
    basics::TEXTAREA,
    basics::TYPOGRAPHY,
];

/// The state of every demo.
#[derive(Debug)]
pub struct Demo {
    pub clicks: u32,
    pub email: String,
    pub password: String,
    pub search: String,
    pub message: iced_ui::textarea::Content,
    pub progress: f32,
    pub page: usize,
}

impl Default for Demo {
    fn default() -> Self {
        Self {
            clicks: 0,
            email: String::new(),
            password: String::new(),
            search: String::new(),
            message: iced_ui::textarea::Content::new(),
            progress: 60.0,
            page: 2,
        }
    }
}

#[derive(Debug, Clone)]
pub enum DemoMessage {
    Clicked,
    Email(String),
    Password(String),
    Search(String),
    Edit(iced_ui::textarea::Action),
    Progress(f32),
    Page(usize),
}

impl Demo {
    pub fn update(&mut self, message: DemoMessage) -> Task<Message> {
        match message {
            DemoMessage::Clicked => self.clicks += 1,
            DemoMessage::Email(value) => self.email = value,
            DemoMessage::Password(value) => self.password = value,
            DemoMessage::Search(value) => self.search = value,
            DemoMessage::Edit(action) => self.message.perform(action),
            DemoMessage::Progress(delta) => self.progress = (self.progress + delta).clamp(0.0, 100.0),
            DemoMessage::Page(page) => self.page = page,
        }

        Task::none()
    }
}
