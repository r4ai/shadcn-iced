//! One page per component.
mod basics;
mod button;
pub mod chat;
pub mod extra;
pub mod interactive;
pub mod overlays;

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
    overlays::ALERT_DIALOG,
    basics::ASPECT_RATIO,
    chat::ATTACHMENT,
    basics::AVATAR,
    basics::BADGE,
    basics::BREADCRUMB,
    chat::BUBBLE,
    button::PAGE,
    interactive::BUTTON_GROUP,
    extra::CALENDAR,
    basics::CARD,
    extra::CAROUSEL,
    extra::CHART,
    interactive::CHECKBOX,
    interactive::COLLAPSIBLE,
    overlays::COMBOBOX,
    overlays::COMMAND,
    overlays::CONTEXT_MENU,
    extra::DATE_PICKER,
    overlays::DIALOG,
    overlays::DRAWER,
    overlays::DROPDOWN_MENU,
    basics::EMPTY,
    interactive::FIELD,
    overlays::HOVER_CARD,
    basics::INPUT,
    interactive::INPUT_GROUP,
    interactive::INPUT_OTP,
    basics::ITEM,
    basics::KBD,
    basics::LABEL,
    overlays::MENUBAR,
    chat::MARKER,
    chat::MESSAGE,
    chat::MESSAGE_SCROLLER,
    overlays::NAVIGATION_MENU,
    interactive::NATIVE_SELECT,
    basics::PAGINATION,
    overlays::POPOVER,
    basics::PROGRESS,
    interactive::RADIO_GROUP,
    interactive::RESIZABLE,
    basics::SCROLL_AREA,
    overlays::SELECT,
    basics::SEPARATOR,
    extra::SIDEBAR,
    overlays::SHEET,
    basics::SKELETON,
    interactive::SLIDER,
    overlays::SONNER,
    basics::SPINNER,
    interactive::SWITCH,
    basics::TABLE,
    interactive::TABS,
    basics::TEXTAREA,
    interactive::TOGGLE,
    interactive::TOGGLE_GROUP,
    overlays::TOOLTIP,
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
    pub dialog: bool,
    pub alert_dialog: bool,
    pub sheet: Option<iced_ui::sheet::Side>,
    pub drawer: bool,
    pub goal: i32,
    pub status_bar: bool,
    pub position: u8,
    pub query: String,
    pub framework: Option<&'static str>,
    pub toasts: Vec<iced_ui::sonner::Toast<Message>>,
    pub next_toast: u64,
    pub month: iced_ui::calendar::Month,
    pub date: Option<iced_ui::calendar::Date>,
    pub range: (
        Option<iced_ui::calendar::Date>,
        Option<iced_ui::calendar::Date>,
    ),
    pub slide: usize,
    pub sidebar_collapsed: bool,
    pub scroller_at_end: bool,
    pub dob: Option<iced_ui::calendar::Date>,
    pub dob_open: bool,
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
            dialog: false,
            alert_dialog: false,
            sheet: None,
            drawer: false,
            goal: 350,
            status_bar: true,
            position: 1,
            query: String::new(),
            framework: None,
            toasts: Vec::new(),
            next_toast: 0,
            month: iced_ui::calendar::Month {
                year: 2026,
                month: 10,
            },
            date: iced_ui::calendar::Date::new(2026, 10, 16),
            range: (
                iced_ui::calendar::Date::new(2026, 10, 12),
                iced_ui::calendar::Date::new(2026, 10, 22),
            ),
            slide: 0,
            sidebar_collapsed: false,
            scroller_at_end: true,
            dob: None,
            dob_open: false,
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
    Dialog(bool),
    AlertDialog(bool),
    Sheet(Option<iced_ui::sheet::Side>),
    Drawer(bool),
    Goal(i32),
    StatusBar(bool),
    Position(u8),
    Query(String),
    Framework(&'static str),
    Toast(iced_ui::sonner::Kind),
    DismissToast(u64),
    Month(iced_ui::calendar::Month),
    Date(iced_ui::calendar::Date),
    Range(iced_ui::calendar::Date),
    Slide(usize),
    SidebarCollapsed(bool),
    ScrollerAtEnd(bool),
    ScrollToEnd,
    Preset(i64),
    Dob(iced_ui::calendar::Date),
    DobOpen(bool),
}

impl Demo {
    pub fn update(&mut self, message: DemoMessage) -> Task<Message> {
        match message {
            DemoMessage::Clicked => self.clicks += 1,
            DemoMessage::Email(value) => self.email = value,
            DemoMessage::Password(value) => self.password = value,
            DemoMessage::Search(value) => self.search = value,
            DemoMessage::Edit(action) => self.message.perform(action),
            DemoMessage::Progress(delta) => {
                self.progress = (self.progress + delta).clamp(0.0, 100.0)
            }
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
            DemoMessage::Dialog(open) => self.dialog = open,
            DemoMessage::AlertDialog(open) => self.alert_dialog = open,
            DemoMessage::Sheet(side) => self.sheet = side,
            DemoMessage::Drawer(open) => self.drawer = open,
            DemoMessage::Goal(delta) => self.goal = (self.goal + delta).clamp(200, 400),
            DemoMessage::StatusBar(value) => self.status_bar = value,
            DemoMessage::Position(value) => self.position = value,
            DemoMessage::Query(value) => self.query = value,
            DemoMessage::Framework(value) => {
                self.framework = Some(value);
                self.query.clear();
            }
            DemoMessage::Toast(kind) => {
                use iced_ui::sonner::{Kind, Toast};

                let id = self.next_toast;
                self.next_toast += 1;

                let toast = match kind {
                    Kind::Default => Toast::new(id, "Event has been created")
                        .description("Sunday, December 03, 2023 at 9:00 AM")
                        .action("Undo", Message::Noop),
                    Kind::Success => Toast::new(id, "Event has been created"),
                    Kind::Info => Toast::new(id, "Be at the area 10 minutes before the event time"),
                    Kind::Warning => Toast::new(id, "Event start time cannot be earlier than 8am"),
                    Kind::Error => Toast::new(id, "Event has not been created"),
                    Kind::Loading => Toast::new(id, "Loading..."),
                };

                self.toasts.push(toast.kind(kind));
            }
            DemoMessage::DismissToast(id) => self.toasts.retain(|toast| toast.id != id),
            DemoMessage::Month(month) => self.month = month,
            DemoMessage::Date(date) => self.date = Some(date),
            DemoMessage::Range(date) => {
                self.range = match self.range {
                    (Some(start), None) if date >= start => (Some(start), Some(date)),
                    _ => (Some(date), None),
                };
            }
            DemoMessage::Slide(index) => self.slide = index,
            DemoMessage::SidebarCollapsed(collapsed) => self.sidebar_collapsed = collapsed,
            DemoMessage::ScrollerAtEnd(at_end) => self.scroller_at_end = at_end,
            DemoMessage::Preset(days) => {
                let date = iced_ui::calendar::Date::today().add_days(days);
                self.date = Some(date);
                self.month = date.month_of();
            }
            DemoMessage::Dob(date) => {
                self.dob = Some(date);
                self.dob_open = false;
            }
            DemoMessage::DobOpen(open) => self.dob_open = open,
            DemoMessage::ScrollToEnd => {
                self.scroller_at_end = true;
                return iced_ui::message_scroller::scroll_to_end(chat::SCROLLER_ID);
            }
        }

        Task::none()
    }
}
