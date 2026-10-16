//! One page per component.
mod basics;
mod button;
pub mod interactive;

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
    interactive::ACCORDION,
    basics::ALERT,
    basics::ASPECT_RATIO,
    basics::AVATAR,
    basics::BADGE,
    basics::BREADCRUMB,
    button::PAGE,
    interactive::BUTTON_GROUP,
    basics::CARD,
    interactive::CHECKBOX,
    interactive::COLLAPSIBLE,
    basics::EMPTY,
    interactive::FIELD,
    basics::INPUT,
    interactive::INPUT_GROUP,
    interactive::INPUT_OTP,
    basics::ITEM,
    basics::KBD,
    interactive::NATIVE_SELECT,
    basics::PAGINATION,
    basics::PROGRESS,
    interactive::RADIO_GROUP,
    interactive::RESIZABLE,
    basics::SCROLL_AREA,
    basics::SEPARATOR,
    basics::SKELETON,
    interactive::SLIDER,
    basics::SPINNER,
    interactive::SWITCH,
    basics::TABLE,
    interactive::TABS,
    basics::TEXTAREA,
    interactive::TOGGLE,
    interactive::TOGGLE_GROUP,
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
    pub terms: bool,
    pub notifications: bool,
    pub airplane: bool,
    pub density: u8,
    pub volume: f32,
    pub formats: Vec<interactive::Format>,
    pub tab: interactive::Tab,
    pub accordion: Option<usize>,
    pub name: String,
    pub username: String,
    pub url: String,
    pub chat: String,
    pub card: String,
    pub fruit: Option<&'static str>,
    pub otp: String,
}

pub const FRUITS: [&str; 5] = ["Apple", "Banana", "Blueberry", "Grapes", "Pineapple"];

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
            terms: true,
            notifications: false,
            airplane: false,
            density: 1,
            volume: 50.0,
            formats: vec![interactive::Format::Bold],
            tab: interactive::Tab::Account,
            accordion: Some(0),
            name: String::new(),
            username: String::new(),
            url: String::new(),
            chat: String::new(),
            card: String::new(),
            fruit: None,
            otp: String::new(),
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
    Terms(bool),
    Notifications(bool),
    Airplane(bool),
    Density(u8),
    Volume(f32),
    Format(interactive::Format),
    Tab(interactive::Tab),
    Accordion(Option<usize>),
    Name(String),
    Username(String),
    Url(String),
    Chat(String),
    Card(String),
    Fruit(&'static str),
    Otp(String),
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
            DemoMessage::Terms(value) => self.terms = value,
            DemoMessage::Notifications(value) => self.notifications = value,
            DemoMessage::Airplane(value) => self.airplane = value,
            DemoMessage::Density(value) => self.density = value,
            DemoMessage::Volume(value) => self.volume = value,
            DemoMessage::Format(format) => {
                if let Some(index) = self.formats.iter().position(|f| *f == format) {
                    self.formats.remove(index);
                } else {
                    self.formats.push(format);
                }
            }
            DemoMessage::Tab(tab) => self.tab = tab,
            DemoMessage::Accordion(open) => self.accordion = open,
            DemoMessage::Name(value) => self.name = value,
            DemoMessage::Username(value) => self.username = value,
            DemoMessage::Url(value) => self.url = value,
            DemoMessage::Chat(value) => self.chat = value,
            DemoMessage::Card(value) => self.card = value,
            DemoMessage::Fruit(value) => self.fruit = Some(value),
            DemoMessage::Otp(value) => self.otp = value,
        }

        Task::none()
    }
}
