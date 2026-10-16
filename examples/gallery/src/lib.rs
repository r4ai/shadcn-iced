//! The iced_ui gallery: a documentation-like showcase of every component.
pub mod pages;

use iced_ui::prelude::*;
use iced_ui::theme::{Accent, BaseColor};
use iced_ui::{font, style, tokens};

use iced::widget::{column, container, row, rule, space, text};
use iced::{Element, Fill, Length, Task, Theme};
use iced_ui::scroll_area::scroll_area as scrollable;

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
        Self::new(Self::page_by_id("button").unwrap_or(0), Mode::Light)
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
        use iced_ui::sidebar::{group, item};

        let page = &PAGES[self.page];

        let nav =
            iced_ui::sidebar::sidebar()
                .header(
                    row![
                        container(lucide::COMMAND.sized(16))
                            .center(32)
                            .style(|theme: &Theme| {
                                let t = tokens(theme);
                                container::Style {
                                    background: Some(t.palette.sidebar_primary.into()),
                                    text_color: Some(t.palette.sidebar_primary_foreground),
                                    border: style::rounded(t.radius_lg()),
                                    ..Default::default()
                                }
                            }),
                        column![
                            text("iced-ui").size(style::text::SM).font(font::SEMIBOLD),
                            text(format!("{} components", PAGES.len())).size(style::text::XS),
                        ],
                    ]
                    .spacing(8)
                    .align_y(iced::Center),
                )
                .group(PAGES.iter().enumerate().fold(
                    group("Components"),
                    |group, (index, info)| {
                        group.item(
                            item(info.title)
                                .active(index == self.page)
                                .on_press(Message::Navigate(index)),
                        )
                    },
                ))
                .width(240.0);

        let base = iced_ui::select::select(Some(self.base), Message::Base)
            .options(BaseColor::ALL.iter().copied())
            .size(iced_ui::select::Size::Sm)
            .width(120);

        let accent = Accent::ALL
            .iter()
            .fold(
                iced_ui::select::select(Some(self.accent), Message::Accent)
                    .option(None, "No accent"),
                |select, accent| select.option(Some(*accent), accent.to_string()),
            )
            .size(iced_ui::select::Size::Sm)
            .width(130);

        let radius = iced_ui::toggle_group::toggle_group(&[self.radius as u32], |r| {
            Message::Radius(r as f32)
        })
        .variant(iced_ui::toggle::Variant::Outline)
        .size(iced_ui::toggle::Size::Sm)
        .label(0, "0")
        .label(6, "0.3")
        .label(10, "0.6")
        .label(16, "1.0");

        let header = row![
            text(page.title).size(style::text::SM).font(font::MEDIUM),
            space::horizontal(),
            base,
            accent,
            radius,
            iced_ui::button::icon_button(match self.mode {
                Mode::Light => lucide::MOON,
                Mode::Dark => lucide::SUN,
            })
            .variant(iced_ui::button::Variant::Outline)
            .size(iced_ui::button::Size::IconSm)
            .on_press(Message::ToggleMode),
        ]
        .spacing(8)
        .align_y(iced::Center)
        .padding([10, 16])
        .height(52);

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

        let main = column![header, rule::horizontal(1).style(separator), content];

        iced_ui::sonner::toaster(
            iced_ui::sidebar::layout(nav, main),
            &self.demo.toasts,
            |id| Message::Demo(pages::DemoMessage::DismissToast(id)),
        )
        .into()
    }
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
