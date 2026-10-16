//! The iced_ui gallery: a documentation-like showcase of every component.
pub mod pages;

use iced_ui::prelude::*;
use iced_ui::theme::{Accent, BaseColor};
use iced_ui::{font, style, tokens};

use iced::widget::{Column, column, container, row, rule, scrollable, space, text};
use iced::{Element, Fill, Length, Task, Theme};

pub use pages::{PAGES, Page};

/// The state of the gallery.
pub struct Gallery {
    pub page: usize,
    pub mode: Mode,
    pub base: BaseColor,
    pub accent: Option<Accent>,
    pub radius: f32,
    pub theme: Theme,
    pub demo: pages::Demo,
}

#[derive(Debug, Clone)]
pub enum Message {
    Navigate(usize),
    ToggleMode,
    Base(BaseColor),
    Accent(Option<Accent>),
    Radius(f32),
    Demo(pages::DemoMessage),
    Noop,
}

impl Default for Gallery {
    fn default() -> Self {
        Self::new(0, Mode::Light)
    }
}

impl Gallery {
    pub fn new(page: usize, mode: Mode) -> Self {
        let mut gallery = Self {
            page,
            mode,
            base: BaseColor::Neutral,
            accent: None,
            radius: iced_ui::Tokens::DEFAULT_RADIUS,
            theme: Theme::Light,
            demo: pages::Demo::default(),
        };
        gallery.rebuild_theme();
        gallery
    }

    pub fn page_by_id(id: &str) -> Option<usize> {
        PAGES.iter().position(|page| page.id == id)
    }

    fn rebuild_theme(&mut self) {
        let mut config = iced_ui::theme::Config::new(self.base, self.mode).radius(self.radius);

        if let Some(accent) = self.accent {
            config = config.accent(accent);
        }

        self.theme = config.build();
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Navigate(page) => self.page = page,
            Message::ToggleMode => {
                self.mode = self.mode.toggle();
                self.rebuild_theme();
            }
            Message::Base(base) => {
                self.base = base;
                self.rebuild_theme();
            }
            Message::Accent(accent) => {
                self.accent = accent;
                self.rebuild_theme();
            }
            Message::Radius(radius) => {
                self.radius = radius;
                self.rebuild_theme();
            }
            Message::Demo(message) => return self.demo.update(message),
            Message::Noop => {}
        }

        Task::none()
    }

    pub fn theme(&self) -> Theme {
        self.theme.clone()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let page = &PAGES[self.page];

        let sidebar = {
            let items = PAGES.iter().enumerate().map(|(index, info)| {
                nav_item(info.title, index == self.page, Message::Navigate(index))
            });

            container(
                scrollable(
                    Column::new()
                        .push(
                            text("Components")
                                .size(style::text::XS)
                                .font(font::MEDIUM)
                                .style(muted),
                        )
                        .push(space().height(4))
                        .extend(items)
                        .spacing(2)
                        .padding([16, 12]),
                )
                .height(Fill),
            )
            .width(220)
            .height(Fill)
        };

        let header = row![
            text("iced-ui").size(style::text::SM).font(font::SEMIBOLD),
            space::horizontal(),
            iced_ui::button::button(match self.mode {
                Mode::Light => "Dark",
                Mode::Dark => "Light",
            })
            .icon(match self.mode {
                Mode::Light => lucide::MOON,
                Mode::Dark => lucide::SUN,
            })
            .variant(iced_ui::button::Variant::Ghost)
            .size(iced_ui::button::Size::Sm)
            .on_press(Message::ToggleMode),
        ]
        .align_y(iced::Center)
        .padding([10, 16])
        .height(56);

        let content = scrollable(
            column![
                text(page.title).size(style::text::XL3).font(font::BOLD),
                text(page.description).size(style::text::BASE).style(muted),
                space().height(16),
                (page.view)(self),
            ]
            .spacing(8)
            .padding([32, 40])
            .max_width(860),
        )
        .width(Fill)
        .height(Fill);

        column![
            header,
            rule::horizontal(1).style(separator),
            row![sidebar, rule::vertical(1).style(separator), content].height(Fill),
        ]
        .into()
    }
}

fn nav_item(label: &'static str, active: bool, message: Message) -> Element<'static, Message> {
    iced_ui::primitive::pressable(
        container(text(label).size(style::text::SM).font(if active {
            font::MEDIUM
        } else {
            iced::Font::with_name("Geist")
        }))
        .padding([6, 8])
        .width(Fill),
    )
    .on_press(message)
    .width(Fill)
    .style(move |theme, status| {
        let t = tokens(theme);
        let p = t.palette;
        let hovered = matches!(
            status,
            iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed
        );

        iced::widget::button::Style {
            background: (active || hovered).then_some(iced::Background::Color(p.accent)),
            text_color: if active || hovered {
                p.accent_foreground
            } else {
                p.foreground
            },
            border: style::rounded(t.radius_md()),
            ..Default::default()
        }
    })
    .into()
}

pub fn muted(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(tokens(theme).palette.muted_foreground),
    }
}

fn separator(theme: &Theme) -> rule::Style {
    rule::Style {
        color: tokens(theme).palette.border,
        radius: 0.0.into(),
        fill_mode: rule::FillMode::Full,
        snap: true,
    }
}

/// A documentation-like example: a title and a bordered preview area.
pub fn example<'a>(
    title: &'a str,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    column![
        text(title).size(style::text::LG).font(font::SEMIBOLD),
        container(content.into())
            .padding(32)
            .center_x(Fill)
            .center_y(Length::Shrink)
            .height(Length::Shrink)
            .style(|theme: &Theme| {
                let t = tokens(theme);
                container::Style {
                    border: style::border(t.palette.border, t.radius_lg()),
                    ..Default::default()
                }
            }),
    ]
    .spacing(12)
    .padding([12, 0])
    .into()
}

/// Settings shared by the app and the snapshot renderer.
pub fn settings() -> iced::Settings {
    iced::Settings {
        fonts: iced_ui::font::all(),
        default_font: iced_ui::font::SANS,
        ..Default::default()
    }
}
