use dashboard::{Dashboard, Message};
use shadcn_iced::prelude::*;
use shadcn_iced::theme::{BaseColor, Config};

use iced::{Element, Fill};

fn view(dashboard: &Dashboard) -> Element<'_, Message> {
    scroll_area(dashboard.view()).height(Fill).into()
}

fn main() -> iced::Result {
    iced::application(Dashboard::default, Dashboard::update, view)
        .title("Dashboard")
        .theme(|_: &Dashboard| Config::new(BaseColor::Neutral, Mode::Light).build())
        .window_size((1280.0, 900.0))
        .shadcn_iced()
        .run()
}
