//! One page per component.
pub mod agent;
mod basics;
mod button;
pub mod charts;
pub mod chat;
pub mod extra;
pub mod interactive;
pub mod overlays;

use crate::{Gallery, Message};

use iced::{Element, Task};

/// A page of the gallery.
#[derive(Debug)]
pub struct Page {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub view: fn(&Gallery) -> Element<'_, Message>,
}

/// The pages of the chart showcase of the website (`/charts`), which are
/// not components.
pub const CHARTS: &[Page] = &[charts::AREA, charts::BAR, charts::LINE, charts::PIE];

/// Finds a page of [`PAGES`] or [`CHARTS`].
pub fn find(id: &str) -> Option<&'static Page> {
    PAGES.iter().chain(CHARTS).find(|page| page.id == id)
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
    agent::CODE_BLOCK,
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
    agent::QUESTIONNAIRE,
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
    agent::TOAST,
    interactive::TOGGLE,
    interactive::TOGGLE_GROUP,
    overlays::TOOLTIP,
    agent::TRACE,
    basics::TYPOGRAPHY,
];

/// The state of every demo.
#[derive(Debug)]
pub struct Demo {
    pub clicks: u32,
    pub email: String,
    pub password: String,
    pub search: String,
    pub message: shadcn_iced::textarea::Content,
    pub progress: f32,
    pub page: usize,
    pub terms: bool,
    pub notifications: bool,
    pub airplane: bool,
    pub density: u8,
    pub volume: f32,
    /// The values of the other sliders of the Slider page, by [`slider`]
    /// slot.
    pub sliders: Vec<Vec<f32>>,
    pub formats: Vec<interactive::Format>,
    pub tab: interactive::Tab,
    pub accordion: Option<usize>,
    pub name: String,
    pub username: String,
    pub url: String,
    pub chat: String,
    pub card: String,
    pub card_month: Option<&'static str>,
    pub card_year: Option<&'static str>,
    pub cvv: String,
    pub fruit: Option<&'static str>,
    pub otp: String,
    pub dialog: bool,
    pub alert_dialog: bool,
    pub small_alert_dialog: bool,
    pub sheet: Option<shadcn_iced::sheet::Side>,
    /// The side of the last sheet opened, which it keeps while it closes.
    pub sheet_side: shadcn_iced::sheet::Side,
    pub default_sheet: bool,
    pub drawer: bool,
    pub goal: i32,
    pub status_bar: bool,
    pub position: u8,
    pub query: String,
    pub framework: Option<&'static str>,
    pub toasts: Vec<shadcn_iced::sonner::Toast<Message>>,
    pub next_toast: u64,
    pub month: shadcn_iced::calendar::Month,
    pub date: Option<shadcn_iced::calendar::Date>,
    pub range: (
        Option<shadcn_iced::calendar::Date>,
        Option<shadcn_iced::calendar::Date>,
    ),
    pub slide: usize,
    pub sidebar_collapsed: bool,
    /// The items of the Sidebar page whose sub-items are shown.
    pub sidebar_open: Vec<&'static str>,
    /// The item of the Sidebar page that was pressed last.
    pub sidebar_active: &'static str,
    pub scroller_at_end: bool,
    pub dob: Option<shadcn_iced::calendar::Date>,
    pub dob_open: bool,
    pub copied: Option<String>,
    pub code_wrap: bool,
    pub code_expanded: bool,
    pub questions: Vec<shadcn_iced::questionnaire::Question>,
    pub questionnaire: shadcn_iced::questionnaire::State,
    pub base_toasts: Vec<shadcn_iced::toast::Toast<Message>>,
}

pub const FRUITS: [&str; 5] = ["Apple", "Banana", "Blueberry", "Grapes", "Pineapple"];

impl Default for Demo {
    fn default() -> Self {
        Self {
            clicks: 0,
            email: String::new(),
            password: String::new(),
            search: String::new(),
            message: shadcn_iced::textarea::Content::new(),
            progress: 60.0,
            page: 2,
            terms: true,
            notifications: false,
            airplane: false,
            density: 1,
            volume: 50.0,
            sliders: vec![
                vec![25.0, 50.0],
                vec![10.0, 20.0, 70.0],
                vec![50.0],
                vec![25.0],
                vec![0.3, 0.7],
            ],
            formats: vec![interactive::Format::Bold],
            tab: interactive::Tab::Account,
            accordion: Some(0),
            name: String::new(),
            username: String::new(),
            url: String::new(),
            chat: String::new(),
            card: String::new(),
            card_month: None,
            card_year: None,
            cvv: String::new(),
            fruit: None,
            otp: String::new(),
            dialog: false,
            alert_dialog: false,
            small_alert_dialog: false,
            sheet: None,
            sheet_side: shadcn_iced::sheet::Side::default(),
            default_sheet: false,
            drawer: false,
            goal: 350,
            status_bar: true,
            position: 1,
            query: String::new(),
            framework: None,
            toasts: Vec::new(),
            next_toast: 0,
            month: shadcn_iced::calendar::Month {
                year: 2026,
                month: 10,
            },
            date: shadcn_iced::calendar::Date::new(2026, 10, 16),
            range: (
                shadcn_iced::calendar::Date::new(2026, 10, 12),
                shadcn_iced::calendar::Date::new(2026, 10, 22),
            ),
            slide: 0,
            sidebar_collapsed: false,
            sidebar_open: vec!["Playground"],
            sidebar_active: "History",
            scroller_at_end: true,
            dob: None,
            dob_open: false,
            copied: None,
            code_wrap: false,
            code_expanded: false,
            questions: agent::questions(),
            questionnaire: shadcn_iced::questionnaire::State::new(),
            base_toasts: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum DemoMessage {
    Clicked,
    Email(String),
    Password(String),
    Search(String),
    Edit(shadcn_iced::textarea::Action),
    Progress(f32),
    /// A thumb of a slider of the Slider page moved: its slot, the thumb
    /// and the value.
    Slider(usize, usize, f32),
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
    CardMonth(&'static str),
    CardYear(&'static str),
    Cvv(String),
    /// Clears the payment form of the Field page.
    ResetPayment,
    Fruit(&'static str),
    Otp(String),
    Dialog(bool),
    AlertDialog(bool),
    SmallAlertDialog(bool),
    Sheet(Option<shadcn_iced::sheet::Side>),
    DefaultSheet(bool),
    Drawer(bool),
    Goal(i32),
    StatusBar(bool),
    Position(u8),
    Query(String),
    Framework(&'static str),
    Toast(shadcn_iced::sonner::Kind),
    DismissToast(u64),
    Month(shadcn_iced::calendar::Month),
    Date(shadcn_iced::calendar::Date),
    Range(shadcn_iced::calendar::Date),
    Slide(usize),
    SidebarCollapsed(bool),
    /// Shows or hides the sub-items of an item of the Sidebar page.
    SidebarToggle(&'static str),
    /// Presses an item of the Sidebar page.
    SidebarActive(&'static str),
    ScrollerAtEnd(bool),
    ScrollToEnd,
    Preset(i64),
    Dob(shadcn_iced::calendar::Date),
    DobOpen(bool),
    CopyCode(String),
    CodeWrap(bool),
    CodeExpanded(bool),
    Questionnaire(shadcn_iced::questionnaire::Event),
    BaseToast(Option<shadcn_iced::toast::Kind>),
    CloseBaseToast(u64),
}

impl Demo {
    pub fn update(&mut self, message: DemoMessage) -> Task<Message> {
        match message {
            DemoMessage::Clicked => self.clicks += 1,
            DemoMessage::Email(value) => self.email = value,
            DemoMessage::Password(value) => self.password = value,
            DemoMessage::Search(value) => self.search = value,
            DemoMessage::Edit(action) => self.message.perform(action),
            DemoMessage::Progress(value) => self.progress = value.clamp(0.0, 100.0),
            DemoMessage::Page(page) => self.page = page,
            DemoMessage::Terms(value) => self.terms = value,
            DemoMessage::Notifications(value) => self.notifications = value,
            DemoMessage::Airplane(value) => self.airplane = value,
            DemoMessage::Density(value) => self.density = value,
            DemoMessage::Volume(value) => self.volume = value,
            DemoMessage::Slider(slot, thumb, value) => {
                if let Some(value_of) = self
                    .sliders
                    .get_mut(slot)
                    .and_then(|values| values.get_mut(thumb))
                {
                    *value_of = value;
                }
            }
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
            DemoMessage::CardMonth(value) => self.card_month = Some(value),
            DemoMessage::CardYear(value) => self.card_year = Some(value),
            DemoMessage::Cvv(value) => {
                self.cvv = value.chars().filter(char::is_ascii_digit).take(4).collect();
            }
            DemoMessage::ResetPayment => {
                self.name.clear();
                self.card.clear();
                self.card_month = None;
                self.card_year = None;
                self.cvv.clear();
            }
            DemoMessage::Fruit(value) => self.fruit = Some(value),
            DemoMessage::Otp(value) => self.otp = value,
            DemoMessage::Dialog(open) => self.dialog = open,
            DemoMessage::AlertDialog(open) => self.alert_dialog = open,
            DemoMessage::SmallAlertDialog(open) => self.small_alert_dialog = open,
            DemoMessage::Sheet(side) => {
                self.sheet = side;
                if let Some(side) = side {
                    self.sheet_side = side;
                }
            }
            DemoMessage::DefaultSheet(open) => self.default_sheet = open,
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
                use shadcn_iced::sonner::{Kind, Toast};

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
            DemoMessage::SidebarToggle(item) => {
                if let Some(index) = self.sidebar_open.iter().position(|open| *open == item) {
                    let _ = self.sidebar_open.remove(index);
                } else {
                    self.sidebar_open.push(item);
                }
            }
            DemoMessage::SidebarActive(item) => self.sidebar_active = item,
            DemoMessage::ScrollerAtEnd(at_end) => self.scroller_at_end = at_end,
            DemoMessage::Preset(days) => {
                let date = shadcn_iced::calendar::Date::today().add_days(days);
                self.date = Some(date);
                self.month = date.month_of();
            }
            DemoMessage::Dob(date) => {
                self.dob = Some(date);
                self.dob_open = false;
            }
            DemoMessage::DobOpen(open) => self.dob_open = open,
            DemoMessage::CopyCode(code) => {
                self.copied = Some(code.clone());
                return iced::clipboard::write(code);
            }
            DemoMessage::CodeWrap(wrap) => self.code_wrap = wrap,
            DemoMessage::CodeExpanded(expanded) => self.code_expanded = expanded,
            DemoMessage::Questionnaire(event) => {
                if self.questionnaire.update(&self.questions, event) {
                    let answers = self.questionnaire.answers(&self.questions);
                    let id = self.next_toast;
                    self.next_toast += 1;
                    self.toasts.push(
                        shadcn_iced::sonner::Toast::new(id, "Agent plan saved").description(
                            format!(
                                "Direction: {} · Progress signals: {} · Timing: {}",
                                answers[0].join(", "),
                                if answers[1].is_empty() {
                                    "None".to_owned()
                                } else {
                                    answers[1].join(", ")
                                },
                                answers[2].join(", ")
                            ),
                        ),
                    );
                    self.questionnaire = shadcn_iced::questionnaire::State::new();
                }
            }
            DemoMessage::BaseToast(kind) => {
                use shadcn_iced::toast::{Kind, Toast};

                let id = self.next_toast;
                self.next_toast += 1;

                let toast = match kind {
                    None => Toast::new(id, "Event created")
                        .description("Sunday, December 3 at 9:00 AM")
                        .action("Undo", Message::Demo(DemoMessage::CloseBaseToast(id))),
                    Some(kind) => Toast::new(id, "")
                        .description(match kind {
                            Kind::Default | Kind::Success => "Event has been created.",
                            Kind::Info => "Arrive 10 minutes before the event.",
                            Kind::Warning => "The event cannot start before 8:00 AM.",
                            Kind::Error => "The event could not be created.",
                            Kind::Loading => "Creating the event…",
                        })
                        .kind(kind),
                };

                self.base_toasts.push(toast);
            }
            DemoMessage::CloseBaseToast(id) => self.base_toasts.retain(|toast| toast.id != id),
            DemoMessage::ScrollToEnd => {
                self.scroller_at_end = true;
                return shadcn_iced::message_scroller::scroll_to_end(chat::SCROLLER_ID);
            }
        }

        Task::none()
    }
}
