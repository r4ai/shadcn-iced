//! The settings of a fictional app, after the forms example of shadcn/ui
//! (<https://ui.shadcn.com/examples/forms>): a section per page of a
//! sidebar, each a form. Nothing leaves the app: saving a form only says so.
//!
//! ```sh
//! cargo run -p settings
//! ```
//!
//! The website shows it as a live preview too (`/examples/settings`).
use shadcn_iced::button::Variant;
use shadcn_iced::calendar::{Date, Month};
use shadcn_iced::field::{self, choice_card, field_group};
use shadcn_iced::prelude::*;
use shadcn_iced::textarea::{Action, Content};
use shadcn_iced::{Mode, font, style, tokens};

use iced::widget::{Column, column, container, row, space, text};
use iced::{Center, Element, Fill, Length, Theme};

const EMAILS: [&str; 3] = ["m@example.com", "m@google.com", "m@support.com"];
const LANGUAGES: [&str; 6] = [
    "English", "French", "German", "Spanish", "Japanese", "Chinese",
];
const FONTS: [&str; 3] = ["Geist", "Inter", "System"];
const SIDEBAR: [&str; 6] = [
    "Recents",
    "Home",
    "Applications",
    "Desktop",
    "Downloads",
    "Documents",
];

/// The state of the settings.
#[derive(Debug)]
pub struct Settings {
    section: Section,
    /// The section whose form was saved last, until something changes.
    saved: Option<Section>,
    // Profile
    username: String,
    email: Option<&'static str>,
    bio: Content,
    urls: Vec<String>,
    // Account
    name: String,
    dob: Option<Date>,
    dob_month: Month,
    dob_open: bool,
    language: Option<&'static str>,
    // Appearance
    font: Option<&'static str>,
    mode: Mode,
    /// The mode of the last saved appearance, which the app uses.
    applied_mode: Mode,
    // Notifications
    notify: Notify,
    communication: bool,
    marketing: bool,
    social: bool,
    mobile: bool,
    // Display
    sidebar: Vec<&'static str>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Section(Section),
    Save,
    Username(String),
    Email(&'static str),
    Bio(Action),
    Url(usize, String),
    AddUrl,
    Name(String),
    Dob(Date),
    DobMonth(Month),
    DobOpen(bool),
    Language(&'static str),
    Font(&'static str),
    Mode(Mode),
    Notify(Notify),
    Communication(bool),
    Marketing(bool),
    Social(bool),
    Mobile(bool),
    Sidebar(&'static str, bool),
}

/// A section of the settings, a link of the sidebar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    Profile,
    Account,
    Appearance,
    Notifications,
    Display,
}

impl Section {
    const ALL: [Section; 5] = [
        Section::Profile,
        Section::Account,
        Section::Appearance,
        Section::Notifications,
        Section::Display,
    ];

    fn title(self) -> &'static str {
        match self {
            Section::Profile => "Profile",
            Section::Account => "Account",
            Section::Appearance => "Appearance",
            Section::Notifications => "Notifications",
            Section::Display => "Display",
        }
    }

    fn icon(self) -> shadcn_iced::Icon {
        match self {
            Section::Profile => lucide::USER,
            Section::Account => lucide::SETTINGS,
            Section::Appearance => lucide::PALETTE,
            Section::Notifications => lucide::BELL,
            Section::Display => lucide::MONITOR,
        }
    }

    fn description(self) -> &'static str {
        match self {
            Section::Profile => "This is how others will see you on the site.",
            Section::Account => {
                "Update your account settings. Set your preferred language and timezone."
            }
            Section::Appearance => {
                "Customize the appearance of the app. Automatically switch between day and night themes."
            }
            Section::Notifications => "Configure how you receive notifications.",
            Section::Display => "Turn items on or off to control what's displayed in the app.",
        }
    }

    fn submit(self) -> &'static str {
        match self {
            Section::Profile => "Update profile",
            Section::Account => "Update account",
            Section::Appearance => "Update preferences",
            Section::Notifications => "Update notifications",
            Section::Display => "Update display",
        }
    }
}

/// Who to notify about (`Notify me about...`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Notify {
    All,
    Mentions,
    Nothing,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            section: Section::Profile,
            saved: None,
            username: "shadcn".to_owned(),
            email: None,
            bio: Content::with_text("I own a computer."),
            urls: vec![
                "https://shadcn.com".to_owned(),
                "http://twitter.com/shadcn".to_owned(),
            ],
            name: String::new(),
            dob: None,
            dob_month: Month {
                year: 1995,
                month: 6,
            },
            dob_open: false,
            language: Some("English"),
            font: Some("Geist"),
            mode: Mode::Light,
            applied_mode: Mode::Light,
            notify: Notify::Mentions,
            communication: false,
            marketing: false,
            social: true,
            mobile: false,
            sidebar: vec!["Recents", "Home"],
        }
    }
}

impl Settings {
    pub fn update(&mut self, message: Message) {
        // Any change makes the saved form unsaved again.
        if !matches!(message, Message::Section(_) | Message::DobMonth(_)) {
            self.saved = None;
        }

        match message {
            Message::Section(section) => self.section = section,
            Message::Save => {
                if self.username_error().is_none() {
                    self.saved = Some(self.section);
                    if self.section == Section::Appearance {
                        self.applied_mode = self.mode;
                    }
                }
            }
            Message::Username(username) => self.username = username,
            Message::Email(email) => self.email = Some(email),
            Message::Bio(action) => self.bio.perform(action),
            Message::Url(index, url) => self.urls[index] = url,
            Message::AddUrl => self.urls.push(String::new()),
            Message::Name(name) => self.name = name,
            Message::Dob(date) => {
                self.dob = Some(date);
                self.dob_open = false;
            }
            Message::DobMonth(month) => self.dob_month = month,
            Message::DobOpen(open) => self.dob_open = open,
            Message::Language(language) => self.language = Some(language),
            Message::Font(font) => self.font = Some(font),
            Message::Mode(mode) => self.mode = mode,
            Message::Notify(notify) => self.notify = notify,
            Message::Communication(on) => self.communication = on,
            Message::Marketing(on) => self.marketing = on,
            Message::Social(on) => self.social = on,
            Message::Mobile(on) => self.mobile = on,
            Message::Sidebar(item, on) => {
                self.sidebar.retain(|i| *i != item);
                if on {
                    self.sidebar.push(item);
                }
            }
        }
    }

    /// The mode of the saved appearance.
    pub fn applied_mode(&self) -> Mode {
        self.applied_mode
    }

    fn username_error(&self) -> Option<String> {
        (self.username.trim().chars().count() < 2)
            .then(|| "Username must be at least 2 characters.".to_owned())
    }

    /// The settings, at their natural height (the app scrolls them).
    pub fn view(&self) -> Element<'_, Message> {
        let header = column![
            text("Settings").size(style::text::XL2).font(font::bold()),
            text("Manage your account settings and set e-mail preferences.").style(muted),
        ]
        .spacing(2);

        let nav = Section::ALL
            .into_iter()
            .fold(column![].spacing(4).width(200), |nav, section| {
                nav.push(
                    // `justify-start`: the label on the left.
                    shadcn_iced::button::Button::new(
                        row![
                            section.icon().sized(16),
                            text(section.title())
                                .size(style::text::SM)
                                .font(font::medium()),
                        ]
                        .spacing(8)
                        .align_y(Center)
                        .width(Fill),
                    )
                    .variant(if section == self.section {
                        Variant::Secondary
                    } else {
                        Variant::Ghost
                    })
                    .width(Fill)
                    .on_press(Message::Section(section)),
                )
            });

        let form = match self.section {
            Section::Profile => self.profile(),
            Section::Account => self.account(),
            Section::Appearance => self.appearance(),
            Section::Notifications => self.notifications(),
            Section::Display => self.display(),
        };

        let submit = row![button(self.section.submit()).on_press(Message::Save)]
            .push((self.saved == Some(self.section)).then(|| {
                row![
                    lucide::CHECK.sized(16).style(muted),
                    text("Saved.").size(style::text::SM).style(muted),
                ]
                .spacing(6)
                .align_y(Center)
            }))
            .spacing(16)
            .align_y(Center);

        let section = column![
            column![
                text(self.section.title())
                    .size(style::text::LG)
                    .font(font::medium()),
                text(self.section.description())
                    .size(style::text::SM)
                    .style(muted),
            ]
            .spacing(4),
            separator(),
            form,
            submit,
        ]
        .spacing(24)
        .max_width(640);

        // The row makes a filling child at least as wide as its share: the
        // container takes it, and lays out the section within it loosely.
        let section = container(section).width(Fill);

        column![header, separator(), row![nav, section].spacing(48),]
            .spacing(24)
            .padding(40)
            .into()
    }

    fn profile(&self) -> Element<'_, Message> {
        let urls =
            self.urls
                .iter()
                .enumerate()
                .fold(Column::new().spacing(8), |urls, (index, url)| {
                    urls.push(input("https://", url).on_input(move |url| Message::Url(index, url)))
                });

        field_group([
            field(
                "Username",
                input("shadcn", &self.username).on_input(Message::Username),
            )
            .description(
                "This is your public display name. It can be your real name or a pseudonym. \
                 You can only change this once every 30 days.",
            )
            .error_maybe(self.username_error())
            .into(),
            field(
                "Email",
                select(self.email, Message::Email)
                    .placeholder("Select a verified email to display")
                    .options(EMAILS)
                    .width(Fill),
            )
            .description("You can manage verified email addresses in your email settings.")
            .into(),
            field(
                "Bio",
                textarea(&self.bio)
                    .placeholder("Tell us a little bit about yourself")
                    .on_action(Message::Bio),
            )
            .description("You can @mention other users and organizations to link to them.")
            .into(),
            field(
                "URLs",
                column![
                    urls,
                    button("Add URL")
                        .variant(Variant::Outline)
                        .size(shadcn_iced::button::Size::Sm)
                        .on_press(Message::AddUrl),
                ]
                .spacing(8),
            )
            .description("Add links to your website, blog, or social media profiles.")
            .into(),
        ])
        .into()
    }

    fn account(&self) -> Element<'_, Message> {
        field_group([
            field(
                "Name",
                input("Your name", &self.name).on_input(Message::Name),
            )
            .description("This is the name that will be displayed on your profile and in emails.")
            .into(),
            field(
                "Date of birth",
                date_picker(self.dob, Message::Dob, self.dob_month, Message::DobMonth)
                    .placeholder("Pick a date")
                    .icon(true)
                    .dropdowns(1940, 2026)
                    .open(self.dob_open, Message::DobOpen(false))
                    .on_open(Message::DobOpen(true))
                    .width(240),
            )
            .description("Your date of birth is used to calculate your age.")
            .into(),
            field(
                "Language",
                select(self.language, Message::Language)
                    .placeholder("Select language")
                    .options(LANGUAGES)
                    .width(240),
            )
            .description("This is the language that will be used in the app.")
            .into(),
        ])
        .into()
    }

    fn appearance(&self) -> Element<'_, Message> {
        let theme = row![
            theme_card(Mode::Light, self.mode),
            theme_card(Mode::Dark, self.mode),
        ]
        .spacing(16);

        field_group([
            field(
                "Font",
                select(self.font, Message::Font).options(FONTS).width(200),
            )
            .description("Set the font you want to use in the app.")
            .into(),
            field("Theme", theme)
                .description("Select the theme for the app.")
                .into(),
        ])
        .into()
    }

    fn notifications(&self) -> Element<'_, Message> {
        let email = |label: &'static str,
                     description: &'static str,
                     on: bool,
                     message: Option<fn(bool) -> Message>| {
            let control = switch(on);
            let control = match message {
                Some(message) => control.on_toggle(message),
                None => control.disabled(true),
            };

            choice_card(
                field(label, control)
                    .description(description)
                    .orientation(field::Orientation::Responsive),
                on,
                message.map(|message| message(!on)),
            )
        };

        field_group([
            field(
                "Notify me about...",
                radio_group(Some(self.notify), Message::Notify)
                    .option(Notify::All, "All new messages")
                    .option(Notify::Mentions, "Direct messages and mentions")
                    .option(Notify::Nothing, "Nothing"),
            )
            .into(),
            column![
                text("Email Notifications")
                    .size(style::text::LG)
                    .font(font::medium()),
                email(
                    "Communication emails",
                    "Receive emails about your account activity.",
                    self.communication,
                    Some(Message::Communication),
                ),
                email(
                    "Marketing emails",
                    "Receive emails about new products, features, and more.",
                    self.marketing,
                    Some(Message::Marketing),
                ),
                email(
                    "Social emails",
                    "Receive emails for friend requests, follows, and more.",
                    self.social,
                    Some(Message::Social),
                ),
                email(
                    "Security emails",
                    "Receive emails about your account activity and security.",
                    true,
                    None,
                ),
            ]
            .spacing(12)
            .into(),
            checkbox(self.mobile)
                .label("Use different settings for my mobile devices")
                .description(
                    "You can manage your mobile notifications in the mobile settings page.",
                )
                .on_toggle(Message::Mobile)
                .into(),
        ])
        .into()
    }

    fn display(&self) -> Element<'_, Message> {
        let items = SIDEBAR
            .into_iter()
            .fold(column![].spacing(12), |items, item| {
                items.push(
                    checkbox(self.sidebar.contains(&item))
                        .label(item)
                        .on_toggle(move |on| Message::Sidebar(item, on)),
                )
            });

        field_group([column![
            text("Sidebar").size(style::text::BASE).font(font::medium()),
            text("Select the items you want to display in the sidebar.")
                .size(style::text::SM)
                .style(muted),
            space().height(4),
            items,
        ]
        .spacing(4)
        .into()])
        .into()
    }
}

/// A theme to pick: a sketch of a page in that mode, and its name.
fn theme_card<'a>(mode: Mode, selected: Mode) -> Element<'a, Message> {
    let dark = mode == Mode::Dark;
    let (page, block, line) = if dark {
        (0x020817, 0x1e293b, 0x94a3b8)
    } else {
        (0xecedef, 0xffffff, 0xecedef)
    };

    let shape = move |color: u32, width: Length, height: f32| {
        container(space())
            .width(width)
            .height(height)
            .style(move |_: &Theme| container::Style {
                background: Some(
                    iced::Color::from_rgb8((color >> 16) as u8, (color >> 8) as u8, color as u8)
                        .into(),
                ),
                border: style::rounded(4.0),
                ..Default::default()
            })
    };

    let bar = move |dot: bool| {
        container(
            row![]
                .push(dot.then(|| shape(line, Length::Fixed(16.0), 16.0)))
                .push(shape(
                    line,
                    Length::Fixed(if dot { 100.0 } else { 80.0 }),
                    8.0,
                ))
                .push((!dot).then(|| shape(line, Length::Fixed(100.0), 8.0)))
                .spacing(8)
                .align_y(Center),
        )
        .padding(8)
        .width(Fill)
        .style(move |_: &Theme| container::Style {
            background: Some(
                iced::Color::from_rgb8((block >> 16) as u8, (block >> 8) as u8, block as u8).into(),
            ),
            border: style::rounded(6.0),
            ..Default::default()
        })
    };

    let sketch = container(column![bar(false), bar(true), bar(true)].spacing(8))
        .padding(8)
        .width(200)
        .style(move |_: &Theme| container::Style {
            background: Some(
                iced::Color::from_rgb8((page >> 16) as u8, (page >> 8) as u8, page as u8).into(),
            ),
            border: style::rounded(6.0),
            ..Default::default()
        });

    let active = mode == selected;

    let frame = container(sketch).padding(4).style(move |theme: &Theme| {
        let t = tokens(theme);
        container::Style {
            border: style::border(
                if active {
                    t.palette.primary
                } else {
                    t.palette.muted
                },
                t.radius_md() + 4.0,
            )
            .width(2),
            ..Default::default()
        }
    });

    iced::widget::mouse_area(
        column![
            frame,
            text(if dark { "Dark" } else { "Light" })
                .size(style::text::SM)
                .font(font::medium())
        ]
        .spacing(8)
        .align_x(Center),
    )
    .interaction(iced::mouse::Interaction::Pointer)
    .on_press(Message::Mode(mode))
    .into()
}

fn muted(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(tokens(theme).palette.muted_foreground),
    }
}
