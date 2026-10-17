//! The shadcn_iced gallery: a documentation-like showcase of every component.
pub mod customizer;
pub mod pages;

use shadcn_iced::prelude::*;
use shadcn_iced::theme::{Accent, BaseColor, MenuAccent, MenuColor, Radius, ThemeColor};
use shadcn_iced::{font, style, tokens};

use iced::widget::{column, container, row, rule, space, text};
use iced::{Element, Fill, Length, Task, Theme};
use shadcn_iced::scroll_area::scroll_area as scrollable;

pub use pages::{APPS, CHARTS, PAGES, Page};

/// The state of the gallery.
pub struct Gallery {
    pub page: usize,
    pub mode: Mode,
    pub base: BaseColor,
    pub accent: Option<Accent>,
    pub chart: Option<ThemeColor>,
    pub radius: Radius,
    pub menu_color: MenuColor,
    pub menu_accent: MenuAccent,
    pub heading: customizer::Heading,
    pub customize: bool,
    pub copied: bool,
    pub theme: Theme,
    pub demo: pages::Demo,
    /// The state of the example apps ([`APPS`]).
    pub dashboard: dashboard::Dashboard,
    pub settings: settings::Settings,
}

#[derive(Debug, Clone)]
pub enum Message {
    Navigate(usize),
    ToggleMode,
    Base(BaseColor),
    Accent(Option<Accent>),
    Chart(Option<ThemeColor>),
    Radius(Radius),
    MenuColor(MenuColor),
    MenuAccent(MenuAccent),
    Heading(customizer::Heading),
    Customize(bool),
    CopyTheme,
    ResetTheme,
    Demo(pages::DemoMessage),
    Dashboard(dashboard::Message),
    Settings(settings::Message),
    Noop,
}

impl Default for Gallery {
    fn default() -> Self {
        // `GALLERY_PAGE=sonner cargo run -p gallery` opens a given page.
        let page = std::env::var("GALLERY_PAGE").unwrap_or_else(|_| "button".to_owned());

        Self::new(Self::page_by_id(&page).unwrap_or(0), Mode::Light)
    }
}

impl Gallery {
    pub fn new(page: usize, mode: Mode) -> Self {
        let mut gallery = Self {
            page,
            mode,
            base: BaseColor::Neutral,
            accent: None,
            chart: None,
            radius: Radius::Default,
            menu_color: MenuColor::Default,
            menu_accent: MenuAccent::Subtle,
            heading: customizer::Heading::Inherit,
            customize: false,
            copied: false,
            theme: Theme::Light,
            demo: pages::Demo::default(),
            dashboard: dashboard::Dashboard::default(),
            settings: settings::Settings::default(),
        };
        gallery.rebuild_theme();
        gallery
    }

    pub fn page_by_id(id: &str) -> Option<usize> {
        PAGES.iter().position(|page| page.id == id)
    }

    fn rebuild_theme(&mut self) {
        let mut config = shadcn_iced::theme::Config::new(self.base, self.mode)
            .accent_maybe(self.accent)
            .radius_preset(self.radius)
            .menu_accent(self.menu_accent)
            .menu_color(self.menu_color);

        if let Some(chart) = self.chart {
            config = config.chart_color(chart);
        }

        font::set_heading(self.heading.family());
        self.copied = false;
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
                if matches!(self.chart, Some(ThemeColor::Base(_))) {
                    self.chart = Some(ThemeColor::Base(base));
                }
                self.rebuild_theme();
            }
            Message::Accent(accent) => {
                self.accent = accent;
                self.rebuild_theme();
            }
            Message::Chart(chart) => {
                self.chart = chart;
                self.rebuild_theme();
            }
            Message::Radius(radius) => {
                self.radius = radius;
                self.rebuild_theme();
            }
            Message::MenuColor(color) => {
                self.menu_color = color;
                self.rebuild_theme();
            }
            Message::MenuAccent(accent) => {
                self.menu_accent = accent;
                self.rebuild_theme();
            }
            Message::Heading(heading) => {
                self.heading = heading;
                self.rebuild_theme();
            }
            Message::Customize(open) => self.customize = open,
            Message::CopyTheme => {
                self.copied = true;
                return iced::clipboard::write(customizer::code(self));
            }
            Message::ResetTheme => {
                let customize = self.customize;
                *self = Self {
                    customize,
                    demo: std::mem::take(&mut self.demo),
                    dashboard: std::mem::take(&mut self.dashboard),
                    settings: std::mem::take(&mut self.settings),
                    ..Self::new(self.page, self.mode)
                };
            }
            Message::Demo(message) => return self.demo.update(message),
            Message::Dashboard(message) => self.dashboard.update(message),
            Message::Settings(message) => self.settings.update(message),
            Message::Noop => {}
        }

        Task::none()
    }

    pub fn theme(&self) -> Theme {
        self.theme.clone()
    }

    pub fn view(&self) -> Element<'_, Message> {
        use shadcn_iced::sidebar::{group, item};

        let page = &PAGES[self.page];

        let nav =
            shadcn_iced::sidebar::sidebar()
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
                            text("shadcn-iced")
                                .size(style::text::SM)
                                .font(font::semibold()),
                            text(format!("{} component pages", PAGES.len())).size(style::text::XS),
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

        let header = row![
            text(page.title).size(style::text::SM).font(font::medium()),
            space::horizontal(),
            shadcn_iced::button::button("Customize")
                .icon(lucide::PALETTE)
                .variant(shadcn_iced::button::Variant::Outline)
                .size(shadcn_iced::button::Size::Sm)
                .on_press(Message::Customize(true)),
            shadcn_iced::button::icon_button(match self.mode {
                Mode::Light => lucide::MOON,
                Mode::Dark => lucide::SUN,
            })
            .variant(shadcn_iced::button::Variant::Outline)
            .size(shadcn_iced::button::Size::IconSm)
            .on_press(Message::ToggleMode),
        ]
        .spacing(8)
        .align_y(iced::Center)
        .padding([10, 16])
        .height(52);

        let content = scrollable(
            column![
                text(page.title).size(style::text::XL3).font(font::bold()),
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

        let main = shadcn_iced::sheet::sheet(
            column![header, rule::horizontal(1).style(separator), content],
            self.customize,
        )
        .title("Customize")
        .description("Pick a style and color for your components.")
        .content(scrollable(customizer::view(self)).height(Fill))
        .on_close(Message::Customize(false));

        // Smooth wheel scrolling, horizontal with `Shift`.
        shadcn_iced::primitive::wheel(self.toasters(shadcn_iced::sidebar::layout(nav, main), true))
            .into()
    }

    /// Shows the toasts of the Sonner and Toast pages on top of `content`,
    /// or none of them (without changing the widget tree) when `show` is
    /// `false`.
    pub fn toasters<'a>(
        &'a self,
        content: impl Into<Element<'a, Message>>,
        show: bool,
    ) -> Element<'a, Message> {
        use pages::DemoMessage;

        let (toasts, base_toasts) = if show {
            (&self.demo.toasts[..], &self.demo.base_toasts[..])
        } else {
            (&[][..], &[][..])
        };

        let content = shadcn_iced::toast::toaster(content, base_toasts, |id| {
            Message::Demo(DemoMessage::CloseBaseToast(id))
        });

        shadcn_iced::sonner::toaster(content, toasts, |id| {
            Message::Demo(DemoMessage::DismissToast(id))
        })
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

thread_local! {
    /// The example that [`view_example`] is rendering, if any.
    static FOCUS: std::cell::Cell<Option<usize>> = const { std::cell::Cell::new(None) };
    /// The number of examples built since the page view started.
    static BUILT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    /// The examples built while [`examples`] runs.
    static RECORDED: std::cell::RefCell<Option<Vec<ExampleInfo>>> = const { std::cell::RefCell::new(None) };
}

/// Renders only the `index`-th example of a page, without its title and
/// frame (the website draws those around each preview).
pub fn view_example<'a>(gallery: &'a Gallery, page: &Page, index: usize) -> Element<'a, Message> {
    FOCUS.set(Some(index));
    BUILT.set(0);
    let view = (page.view)(gallery);
    FOCUS.set(None);
    view
}

/// An example of a page, as built by its view.
#[derive(Debug, Clone)]
pub struct ExampleInfo {
    pub title: String,
    /// Where [`example`] is called, to find the source of the example.
    pub location: &'static std::panic::Location<'static>,
}

/// The examples of a page, in the order they are built.
pub fn examples(gallery: &Gallery, page: &Page) -> Vec<ExampleInfo> {
    RECORDED.set(Some(Vec::new()));
    let _ = (page.view)(gallery);
    RECORDED.take().unwrap_or_default()
}

/// A documentation-like example: a title and a bordered preview area.
#[track_caller]
pub fn example<'a>(
    title: &'a str,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    if let Some(focus) = FOCUS.get() {
        let index = BUILT.get();
        BUILT.set(index + 1);

        // Centered, like in the frame below: the column of the page aligns
        // its examples to the left, which shows when one fills its width
        // up to a maximum (e.g. `w-full max-w-sm`).
        return if index == focus {
            container(content.into()).center_x(Fill).into()
        } else {
            space().width(0).height(0).into()
        };
    }

    let location = std::panic::Location::caller();
    RECORDED.with_borrow_mut(|recorded| {
        if let Some(recorded) = recorded {
            recorded.push(ExampleInfo {
                title: title.to_owned(),
                location,
            });
        }
    });

    column![
        text(title).size(style::text::LG).font(font::semibold()),
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
        fonts: shadcn_iced::font::all(),
        default_font: {
            shadcn_iced::font::set_sans(customizer::body_font());
            shadcn_iced::font::sans()
        },
        ..Default::default()
    }
}
